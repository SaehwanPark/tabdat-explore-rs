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
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-conditional-summary-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"),"id,x,y,bad,singleton,g\n1,1.5,2,NaN,NaN,A\n2,,3,Infinity,Infinity,A\n3,NaN,4,-Infinity,-Infinity,A\n4,Infinity,5,,,B\n5,-Infinity,6,,,B\n6,3,6,NaN,5,A\n7,4,8,Infinity,,B\n").unwrap();
    fs::write(root.join("selected.csv"),"id,x,y,bad,singleton,g\n1,1.5,2,NaN,NaN,A\n2,,3,Infinity,Infinity,A\n3,NaN,4,-Infinity,-Infinity,A\n6,3,6,NaN,5,A\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
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
fn success(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
fn batch(fixture: &Fixture, commands: &[&str]) -> Vec<Value> {
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  success(&fixture.batch(&args))
}
fn close(actual: &Value, expected: &Value) {
  if expected.is_null() {
    assert!(actual.is_null());
    return;
  }
  let expected = expected.as_f64().unwrap();
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() <= 1e-10_f64.max(1e-9 * expected.abs()),
    "{actual} != {expected}"
  );
}

#[test]
fn numpy_scipy_selected_samples_order_duplicates_and_independently_filtered_fixture_agree() {
  let fixture = Fixture::new();
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/conditional-summary-reference.json")).unwrap();
  let results = batch(
    &fixture,
    &[
      "use data.csv",
      "summarize singleton x x bad y if g == 'A'",
      "summarize if g == 'A'",
      "count",
      "sql select id from active where g='A' order by id",
    ],
  );
  assert_eq!(results[1]["schema_version"], 1);
  assert_eq!(results[1]["result_type"], "SummarizeResult");
  assert_eq!(results[1]["data"].as_object().unwrap().len(), 1);
  for row in results[1]["data"]["rows"].as_array().unwrap() {
    let expected = reference["rows"]
      .as_array()
      .unwrap()
      .iter()
      .find(|item| item["variable"] == row["variable"])
      .unwrap();
    assert_eq!(row["count"], expected["count"]);
    for key in ["mean", "std_dev", "minimum", "maximum"] {
      close(&row[key], &expected[key]);
    }
  }
  assert_eq!(
    results[1]["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row["variable"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["singleton", "x", "x", "bad", "y"]
  );
  assert_eq!(
    results[2]["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row["variable"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["id", "x", "y", "bad", "singleton"]
  );
  assert_eq!(results[2]["data"]["rows"][0]["count"], 4);
  assert_eq!(results[3]["data"]["row_count"], 7);
  assert_eq!(results[4]["data"]["rows"], json!([[1], [2], [3], [6]]));
  assert_eq!(reference["selected_indices"], json!([0, 1, 2, 5]));
  for (name, indices) in [
    ("x", vec![0, 5]),
    ("y", vec![0, 1, 2, 5]),
    ("bad", vec![]),
    ("singleton", vec![5]),
  ] {
    let expected = reference["rows"]
      .as_array()
      .unwrap()
      .iter()
      .find(|row| row["variable"] == name)
      .unwrap();
    assert_eq!(expected["retained_indices"], json!(indices));
  }
  let selected = batch(
    &fixture,
    &["use selected.csv", "summarize singleton x x bad y"],
  );
  assert_eq!(selected[1]["data"], results[1]["data"]);
}

#[test]
fn predicates_select_true_before_each_variables_finite_sample_without_normalizing_keys() {
  let fixture = Fixture::new();
  let results = batch(
    &fixture,
    &[
      "use data.csv",
      "summarize x y if x > 0",
      "summarize x y if (x + 0) > 0",
      "summarize x y if x == null",
      "summarize x y if null",
      "summarize singleton if g == 'A'",
      "summarize x if null == null",
      "summarize x",
      "replace g = null if g == 'B'",
      "summarize y if g != 'A'",
      "summarize y if g == null",
    ],
  );
  assert_eq!(results[1]["data"]["rows"][0]["count"], 3);
  assert_eq!(results[1]["data"]["rows"][1]["count"], 5);
  assert_eq!(results[2]["data"]["rows"][0]["count"], 3);
  assert_eq!(results[2]["data"]["rows"][1]["count"], 3);
  assert_eq!(results[3]["data"]["rows"][0]["count"], 0);
  assert_eq!(results[3]["data"]["rows"][1]["count"], 1);
  for row in results[4]["data"]["rows"].as_array().unwrap() {
    assert_eq!(row["count"], 0);
    for key in ["mean", "std_dev", "minimum", "maximum"] {
      assert!(row[key].is_null());
    }
  }
  assert_eq!(results[5]["data"]["rows"][0]["count"], 1);
  assert!(results[5]["data"]["rows"][0]["std_dev"].is_null());
  assert_eq!(results[6]["data"], results[7]["data"]);
  assert_eq!(results[9]["data"]["rows"][0]["count"], 0);
  assert_eq!(results[10]["data"]["rows"][0]["count"], 3);
}

#[test]
fn read_only_state_and_cached_inference_survive_inspection_with_same_fit_predict_persistence_stream()
 {
  let fixture = Fixture::new();
  let prefix = [
    "use data.csv",
    "sql select * from active into primary",
    "label variable x 'Predictor'",
    "label define groups 'A' 'Group A' 'B' 'Group B'",
    "label values g groups",
    "regress y x",
  ];
  let inspections = [
    "summarize x y if g == 'A'",
    "summarize if x > 0",
    "summarize singleton x if null",
  ];
  let suffix = [
    "lincom x",
    "regress y x",
    "predict fitted",
    "head 10",
    "save inspected.parquet, replace",
    "export inspected.csv, replace",
    "use primary",
    "head 10",
  ];
  let mut commands = prefix.to_vec();
  commands.extend(inspections);
  commands.extend(suffix);
  let inspected = batch(&fixture, &commands);
  let mut commands = prefix.to_vec();
  commands.extend(suffix);
  let plain = batch(&fixture, &commands);
  assert_eq!(inspected[..6], plain[..6]);
  assert_eq!(inspected[9..], plain[6..]);
  for path in ["inspected.csv", "inspected.parquet"] {
    let reloaded = batch(&fixture, &[&format!("use {path}"), "head 10"]);
    assert_eq!(reloaded[1]["data"], inspected[12]["data"]);
  }
  fs::write(
    fixture.0.join("child.td"),
    format!("{}\n{}\n", inspections.join("\n"), suffix.join("\n")),
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", prefix.join("\n")),
  )
  .unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    inspected
  );
  let mut session = fixture.session();
  for source in &prefix[1..] {
    execute(&mut session, source);
  }
  let state = session.state();
  let metadata = session.active_label_metadata().unwrap().clone();
  let snapshot = "sql select id,cast(x as varchar),cast(bad as varchar),cast(singleton as varchar),g from active order by id";
  let original = execute(&mut session, snapshot);
  let inference = execute(&mut session, "lincom x");
  for source in inspections {
    let result = execute(&mut session, source);
    assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &metadata);
    assert_eq!(execute(&mut session, snapshot), original);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    assert_eq!(
      session
        .last_regression()
        .unwrap()
        .sample
        .as_ref()
        .unwrap()
        .retained_indices,
      [0, 5, 6]
    );
  }
  execute(&mut session, "use primary");
  assert_eq!(execute(&mut session, snapshot), original);
}

#[test]
fn exact_extrema_quoted_and_boolean_predicates_empty_input_and_native_errors_are_bounded() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select id,cast(9007199254740992+id as bigint) as \"x\"\"名\",cast(id as decimal(38,1)) as amount,id<3 as flag from active order by id into wide",
  );
  let result = value(&execute(&mut session, "summarize `x\"名` amount if flag"));
  assert_eq!(
    result["data"]["rows"][0]["minimum"],
    json!(9007199254740993_i64)
  );
  assert_eq!(
    result["data"]["rows"][0]["maximum"],
    json!(9007199254740994_i64)
  );
  assert_eq!(result["data"]["rows"][1]["minimum"], "1.0");
  assert_eq!(result["data"]["rows"][1]["maximum"], "2.0");
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  let result = value(&execute(&mut session, "summarize amount if flag"));
  assert_eq!(result["data"]["rows"][0]["count"], 0);
  assert!(result["data"]["rows"][0]["minimum"].is_null());
  let mut session = fixture.session();
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let snapshot = "sql select id,cast(x as varchar),g from active order by id";
  let original = execute(&mut session, snapshot);
  let inference = execute(&mut session, "lincom x");
  for (source, expected) in [
    (
      "summarize x if absent > 0",
      RuntimeError::SummaryUnknownVariable {
        variables: vec!["absent".into()],
      },
    ),
    ("summarize x if x", RuntimeError::SummaryRequiresBoolean),
    (
      "summarize x if inrange(x, 1, 5)",
      RuntimeError::SummaryUnsupportedPredicate,
    ),
    (
      "summarize g if id > 0",
      RuntimeError::SummaryRequiresNumeric {
        variables: vec!["g".into()],
      },
    ),
    (
      "summarize x if g == 'A\0'",
      RuntimeError::SummaryFailed {
        variable: "x".into(),
      },
    ),
  ] {
    assert_eq!(
      session.execute(parse_command(source).unwrap()).unwrap_err(),
      expected
    );
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, snapshot), original);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  assert!(matches!(
    session.execute(parse_command("summarize x if x == 'A'").unwrap()),
    Err(RuntimeError::SummaryPredicateTypeMismatch { .. })
  ));
  assert_eq!(session.state(), state);
}

#[test]
fn options_stay_outside_admission_and_error_streams_stop_files_with_terminal_recovery() {
  let fixture = Fixture::new();
  assert_eq!(
    fixture
      .batch(&["--json", "-c", "summarize x if id > 0"])
      .status
      .code(),
    Some(1)
  );
  for (source, code, kind) in [
    ("summarize x if absent > 0", 1, "ExecutionError"),
    ("summarize x if x", 1, "ExecutionError"),
    ("summarize x if inrange(x, 1, 5)", 1, "ExecutionError"),
    ("summarize x if id > 0, detail", 2, "JsonResultUnsupported"),
    ("summarize x, detail", 2, "JsonResultUnsupported"),
    ("summarize x if", 2, "ParseError"),
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], kind);
    fs::write(
      fixture.0.join("child.td"),
      format!("count\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      "use data.csv\nrun child.td\nexport never.csv\n",
    )
    .unwrap();
    let output = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(
      output.status.code(),
      Some(if kind == "JsonResultUnsupported" {
        1
      } else {
        code
      })
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[2]["error"]["line"], 2);
    assert_eq!(
      results[2]["error"]["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    assert!(!fixture.0.join("never.csv").exists());
  }
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(b"use data.csv\nsummarize x y if g == 'A'\nsummarize x if absent > 0\ncount\nexport terminal.csv\nexit\n").unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(String::from_utf8_lossy(&output.stdout).contains("2.25"));
  assert!(String::from_utf8_lossy(&output.stderr).contains("unknown variable"));
  let plain = batch(&fixture, &["use data.csv", "head 10"]);
  let reloaded = batch(&fixture, &["use terminal.csv", "head 10"]);
  assert_eq!(reloaded[1]["data"], plain[1]["data"]);
}
