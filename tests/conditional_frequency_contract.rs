#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
fn reference() -> Value {
  serde_json::from_str(include_str!(
    "fixtures/conditional-frequency-reference.json"
  ))
  .unwrap()
}
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-conditional-frequency-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = String::from("e,r,c,x,y\n");
    for row in reference()["records"].as_array().unwrap() {
      csv.push_str(
        &row
          .as_array()
          .unwrap()
          .iter()
          .map(|value| match value {
            Value::Null => String::new(),
            Value::String(text) => text.clone(),
            value => value.to_string(),
          })
          .collect::<Vec<_>>()
          .join(","),
      );
      csv.push('\n');
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn batch(&self, mode: &str, commands: &[&str]) -> Output {
    let mut args = vec![mode];
    for source in commands {
      args.extend(["-c", source]);
    }
    self.run(&args)
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn value(result: &ExecutionResult) -> Value {
  serde_json::from_str(&tabdat_reporting::json_results::serialize_execution_result(result).unwrap())
    .unwrap()
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: Output) -> Vec<u8> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  output.stdout
}
fn compare(actual: &Value, expected: &Value) {
  match expected {
    Value::Array(values) => {
      let actual = actual.as_array().unwrap();
      assert_eq!(actual.len(), values.len());
      for (actual, expected) in actual.iter().zip(values) {
        compare(actual, expected);
      }
    }
    Value::Number(number) if number.is_f64() => {
      assert!(
        (actual.as_f64().unwrap() - number.as_f64().unwrap()).abs() <= 1e-10,
        "{actual} != {expected}"
      );
    }
    _ => assert_eq!(actual, expected),
  }
}
#[test]
fn pandas_selected_samples_counts_and_denominators_match_without_changing_named_labels_or_cached_fit()
 {
  let fixture = Fixture::new();
  let reference = reference();
  assert_eq!(reference["selected_indices"], json!([0, 1, 3, 5, 6, 7]));
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable r 'Category'");
  execute(&mut session, "label define rows 'A' 'Alpha' 'B' 'Beta'");
  execute(&mut session, "label values r rows");
  execute(&mut session, "label define columns 'X' 'Left' 'Y' 'Right'");
  execute(&mut session, "label values c columns");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let signature = execute(&mut session, "datasignature");
  let inference = execute(&mut session, "lincom x");
  let cached = session.last_regression().unwrap().clone();
  assert_eq!(
    cached.sample.as_ref().unwrap().retained_indices,
    [0, 1, 2, 3, 4, 5, 6, 7]
  );
  for (source, case) in [
    ("tabulate r if e == 1, nolabel", "one_False"),
    ("tabulate r if e == 1, missing nolabel", "one_True"),
    ("tabulate r c if e == 1, row col nolabel", "two_False"),
    (
      "tabulate if e == 1, rows(r) columns(c) row col missing nolabel",
      "two_True",
    ),
  ] {
    let result = value(&execute(&mut session, source));
    assert_eq!(result["result_type"], "TabulateResult");
    let expected = if case.starts_with("one") {
      &reference["cases"][case]
    } else {
      &reference["cases"][case]["rows"]
    };
    compare(&result["data"]["rows"], expected);
    let mut headers = vec![json!("r")];
    if case.starts_with("one") {
      headers.extend([json!("Count"), json!("Percent")]);
    } else {
      for key in reference["cases"][case]["column_keys"].as_array().unwrap() {
        let key = key.as_str().unwrap_or("missing");
        headers.extend([
          json!(format!("{key} Count")),
          json!(format!("{key} Row %")),
          json!(format!("{key} Col %")),
        ]);
      }
    }
    assert_eq!(result["data"]["headers"], json!(headers));
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    assert_eq!(session.last_regression().unwrap(), &cached);
  }
  let labeled = value(&execute(&mut session, "tabulate r if e == 1, missing"));
  assert_eq!(labeled["data"]["rows"][0][0], "Alpha");
  assert_eq!(labeled["data"]["rows"][1][0], "Beta");
  let labeled = value(&execute(
    &mut session,
    "tabulate r c if e == 1, row col missing",
  ));
  assert_eq!(
    labeled["data"]["headers"],
    json!([
      "r",
      "Left Count",
      "Left Row %",
      "Left Col %",
      "Right Count",
      "Right Row %",
      "Right Col %",
      "missing Count",
      "missing Row %",
      "missing Col %"
    ])
  );
  assert_eq!(
    value(&execute(&mut session, "tabulate r if e == null, missing"))["data"]["rows"],
    json!([["Beta", 1, 100.0]])
  );
  let explicit = execute(
    &mut session,
    "tabulate, rows(r) columns(c) row col missing nolabel",
  );
  assert_eq!(
    explicit,
    execute(&mut session, "tabulate r c, row col missing nolabel")
  );
  execute(&mut session, "use cohort");
  assert_eq!(execute(&mut session, "head 10"), preview);
}
#[test]
fn conditional_frequency_streams_text_json_nested_and_persistence_keep_original_data() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "tabulate r if e == 1, missing",
    "tabulate r c if e == 1, row col missing",
    "summarize x if e == 1",
    "regress y x",
    "predict fitted",
    "tabulate r if fitted > 5, missing",
    "save original.parquet, replace",
    "export original.csv, replace",
  ];
  let json = success(fixture.batch("--json", &commands));
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  fs::write(fixture.0.join("child.td"), commands[1..].join("\n")).unwrap();
  assert_eq!(success(fixture.run(&["--json", "-f", "parent.td"])), json);
  let text = success(fixture.batch("--text", &commands));
  assert_eq!(success(fixture.run(&["--text", "-f", "parent.td"])), text);
  let results = lines(&json);
  assert_eq!(results[1]["data"]["rows"].as_array().unwrap().len(), 3);
  let reload = lines(&success(
    fixture.batch("--json", &["use original.parquet", "head 10"]),
  ));
  assert_eq!(reload[1]["data"]["rows"].as_array().unwrap().len(), 8);
  assert_eq!(results[7]["data"]["dataset"]["row_count"], 8);
  let csv = lines(&success(
    fixture.batch("--json", &["use original.csv", "head 10"]),
  ));
  assert_eq!(csv[1]["data"], reload[1]["data"]);
}
#[test]
fn native_null_nonfinite_quoted_wide_and_empty_selections_are_read_only() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select * from (values (1,9007199254740993::bigint,cast('NaN' as double),''),(1,-9007199254740993::bigint,cast('Infinity' as double),NULL),(0,9007199254740993::bigint,cast('-Infinity' as double),'z'),(NULL,NULL,NULL,'a')) as t(e,\"wide 雪\",n,s) into edge",
  );
  let signature = execute(&mut session, "datasignature");
  let result = value(&execute(
    &mut session,
    "tabulate `wide 雪` if e == 1, missing",
  ));
  assert_eq!(
    result["data"]["rows"],
    json!([
      [-9007199254740993_i64, 1, 50.0],
      [9007199254740993_i64, 1, 50.0]
    ])
  );
  let result = value(&execute(&mut session, "tabulate s if e == 1, missing"));
  assert_eq!(
    result["data"]["rows"],
    json!([["", 1, 50.0], [null, 1, 50.0]])
  );
  let result = value(&execute(&mut session, "tabulate n if e == 1, missing"));
  assert_eq!(result["data"]["rows"].as_array().unwrap().len(), 2);
  assert!(
    result["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .all(|row| row[1] == 1 && row[2] == 50.0)
  );
  for source in [
    "tabulate s if e > 10, missing",
    "tabulate s if null, missing",
    "tabulate s if e/0 > 1, missing",
    "tabulate s n if e > 10, row col missing",
  ] {
    assert_eq!(
      value(&execute(&mut session, source))["data"]["rows"],
      json!([])
    );
  }
  assert_eq!(execute(&mut session, "datasignature"), signature);
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  assert_eq!(
    value(&execute(&mut session, "tabulate s if e == 1, missing"))["data"]["rows"],
    json!([])
  );
}
#[test]
fn boolean_columns_select_true_and_interactive_errors_recover_without_filtering_data() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select * from (values (true,'A'),(false,'B'),(NULL,'C'),(true,'A')) as t(b,r) into flags",
  );
  assert_eq!(
    value(&execute(&mut session, "tabulate r if b"))["data"]["rows"],
    json!([["A", 2, 100.0]])
  );
  assert_eq!(
    value(&execute(&mut session, "count"))["data"]["row_count"],
    4
  );
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(b"use data.csv\ntabulate r if e == 1, missing\ntabulate r if absent > 0\ntabulate r if e == null\ncount\nexport recovered.csv\nexit\n").unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(String::from_utf8_lossy(&output.stdout).contains("Percent"));
  assert!(String::from_utf8_lossy(&output.stderr).contains("tabulate unknown variable: absent"));
  let original = lines(&success(
    fixture.batch("--json", &["use data.csv", "head 10"]),
  ));
  let recovered = lines(&success(
    fixture.batch("--json", &["use recovered.csv", "head 10"]),
  ));
  assert_eq!(original[1]["data"], recovered[1]["data"]);
}

#[test]
fn predicate_validation_aggregation_dimension_boundaries_and_error_locations_stop_later_files() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "label variable r 'Category'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let signature = execute(&mut session, "datasignature");
  let inference = execute(&mut session, "lincom x");
  for (source, message) in [
    ("tabulate r if absent > 0", "tabulate unknown variable"),
    (
      "tabulate r if e",
      "tabulate predicate requires boolean expression",
    ),
    ("tabulate r if r > 1", "type"),
    (
      "tabulate r if lower(r) == 'a'",
      "tabulate predicate does not support function calls",
    ),
    ("tabulate r if r == '\0'", "tabulate failed"),
  ] {
    let error = session.execute(parse_command(source).unwrap()).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    for mode in ["--text", "--json"] {
      let output = fixture.run(&[mode, "-f", "parent.td"]);
      assert_eq!(output.status.code(), Some(1));
      assert!(
        String::from_utf8(output.stderr)
          .unwrap()
          .contains("child.td:1:")
      );
      assert!(!fixture.0.join("forbidden.csv").exists());
      if mode == "--json" {
        assert!(
          lines(&output.stdout)
            .iter()
            .all(|row| row["result_type"] != "TabulateResult")
        );
      }
    }
  }
  for source in [
    "tabulate, rows(r c)",
    "tabulate r, values(x) stat(mean)",
    "by r: tabulate c if e == 1",
  ] {
    let command = parse_command(source).unwrap();
    assert!(!tabdat_reporting::json_results::supports_batch_command(
      &command
    ));
    assert!(!tabdat_reporting::json_results::supports_script_command(
      &command
    ));
  }
  let output = fixture.batch("--json", &["tabulate r if e == 1"]);
  assert_eq!(output.status.code(), Some(1));
  for source in [
    "tabulate r if",
    "tabulate r if e == 1, unknown",
    "tabulate r if e == 1, row",
  ] {
    let output = fixture.batch("--json", &["use data.csv", source, "export forbidden.csv"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
}
