#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-rename-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "id,x raw,y,label,wide\n1,1.0,2,A,9007199254740993\n2,2.0,5,,9007199254740994\n3,3.0,5,B,9007199254740995\n4,4.0,9,A,9007199254740996\n5,,8,B,9007199254740997\n6,5.0,,C,9007199254740998\n").unwrap();
    fs::write(root.join("empty.csv"), "id,x raw,y,label,wide\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
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
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}
fn execute(session: &mut Session, source: &str) -> tabdat_runtime::ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

#[test]
fn rename_transform_fit_predict_persist_preserves_schema_values_and_nested_stream() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "rename `x raw` x",
    "rename label `标签\"列`",
    "head 10",
    "generate doubled = x * 2",
    "regress y doubled",
    "predict fitted",
    "head 10",
    "save normalized.parquet, replace",
    "export normalized.csv, replace",
    "describe",
  ];
  let mut args = vec!["--json"];
  for source in commands {
    args.extend(["-c", source]);
  }
  let results = success(&fixture.batch(&args));
  assert_eq!(results.len(), commands.len());
  let mut expected = results[0]["data"]["dataset"].clone();
  for (index, position, name) in [(1, 1, "x"), (2, 3, "标签\"列")] {
    expected["columns"][position]["name"] = json!(name);
    assert_eq!(results[index]["schema_version"], 1);
    assert_eq!(results[index]["result_type"], "RenameResult");
    assert_eq!(results[index]["data"], json!({"dataset":expected}));
  }
  assert_eq!(
    results[3]["data"]["rows"],
    json!([
      [1, 1.0, 2, "A", 9007199254740993_i64],
      [2, 2.0, 5, null, 9007199254740994_i64],
      [3, 3.0, 5, "B", 9007199254740995_i64],
      [4, 4.0, 9, "A", 9007199254740996_i64],
      [5, null, 8, "B", 9007199254740997_i64],
      [6, 5.0, null, "C", 9007199254740998_i64]
    ])
  );
  assert_eq!(results[5]["data"]["observation_count"], 4);
  // Independent OLS on doubled=[2,4,6,8], y=[2,5,5,9]:
  // Sxy=21, Sxx=20, slope=1.05 and intercept=0.
  close(&results[5]["data"]["coefficients"][0]["value"], 0.0);
  close(&results[5]["data"]["coefficients"][1]["value"], 1.05);
  for row in results[7]["data"]["rows"].as_array().unwrap() {
    if row[1].is_null() {
      assert!(row[6].is_null());
    } else {
      close(&row[6], 2.1 * row[1].as_f64().unwrap());
    }
  }
  for path in ["normalized.csv", "normalized.parquet"] {
    let reload =
      success(&fixture.batch(&["--json", "-c", &format!("use {path}"), "-c", "head 10"]));
    assert_eq!(reload[1]["data"], results[7]["data"]);
    if path == "normalized.parquet" {
      assert_eq!(
        reload[0]["data"]["dataset"]["columns"],
        results[6]["data"]["dataset"]["columns"]
      );
    }
  }
  fs::write(fixture.0.join("grandchild.td"), commands[5..].join("\n")).unwrap();
  fs::write(
    fixture.0.join("child.td"),
    format!("{}\nrun grandchild.td\n", commands[1..5].join("\n")),
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  assert_eq!(
    success(&fixture.batch(&["--json", "-f", "parent.td"])),
    results
  );
}

#[test]
fn empty_quoted_rename_is_metadata_only_without_guessing_old_new_or_changed_counts() {
  let fixture = Fixture::new();
  let results = success(&fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "rename `x raw` `测\"量`",
    "-c",
    "head 10",
    "-c",
    "save empty.parquet",
  ]));
  let mut expected = results[0]["data"]["dataset"].clone();
  expected["columns"][1]["name"] = json!("测\"量");
  assert_eq!(results[1]["data"], json!({"dataset":expected}));
  assert_eq!(results[1]["data"]["dataset"]["row_count"], 0);
  assert_eq!(results[2]["data"]["rows"], json!([]));
}

#[test]
fn rename_errors_preserve_results_locations_and_prevent_later_files() {
  let fixture = Fixture::new();
  for (source, code) in [
    ("rename absent new", 1),
    ("rename `x raw` y", 1),
    ("rename `x raw` `x raw`", 1),
    ("rename `x raw` Y", 1),
    ("rename", 2),
    ("rename `x raw`", 2),
    ("rename `x raw` x, replace", 2),
    ("rename `x raw` x if y > 0", 2),
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
    assert_eq!(
      output.status.code(),
      Some(code),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(
      results[1]["error"]["type"],
      if code == 2 {
        "ParseError"
      } else {
        "ExecutionError"
      }
    );
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
    assert_eq!(output.status.code(), Some(code), "{source}");
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
  let output = fixture.batch(&["--json", "-c", "rename old new"]);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    lines(&output.stdout)[0]["error"]["message"]
      .as_str()
      .unwrap()
      .contains("active dataset")
  );
}

#[test]
fn rename_remaps_labels_but_retains_original_fit_names_and_failure_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y `x raw`");
  execute(&mut session, "label variable `x raw` 'Predictor'");
  execute(&mut session, "label define xs 1 'one' 2 'two'");
  execute(&mut session, "label values `x raw` xs");
  let before = session.state();
  let rows = execute(&mut session, "head 10");
  for source in [
    "rename absent new",
    "rename `x raw` y",
    "rename `x raw` `x raw`",
    "rename `x raw` Y",
  ] {
    assert!(session.execute(parse_command(source).unwrap()).is_err());
    assert_eq!(session.state(), before);
    assert_eq!(execute(&mut session, "head 10"), rows);
  }
  let result = execute(&mut session, "rename `x raw` x");
  let state = session.state();
  let value: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  assert_eq!(session.state(), state);
  assert_eq!(state.estimation, before.estimation);
  assert_eq!(
    state.labels.variable_labels,
    vec![("x".to_owned(), "Predictor".to_owned())]
  );
  assert_eq!(
    state.labels.attachments,
    vec![("x".to_owned(), "xs".to_owned())]
  );
  assert_eq!(state.labels.value_sets, before.labels.value_sets);
  assert!(value["data"]["dataset"]["label_metadata"].is_null());
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3]
  );
  execute(&mut session, "lincom `x raw`");
  assert_eq!(session.state(), state);
  assert!(
    session
      .execute(parse_command("predict unavailable").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), state);
  let refit = execute(&mut session, "regress y x");
  let value: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&refit).unwrap(),
  )
  .unwrap();
  close(&value["data"]["coefficients"][1]["value"], 2.1);
  execute(&mut session, "predict fitted");
}
