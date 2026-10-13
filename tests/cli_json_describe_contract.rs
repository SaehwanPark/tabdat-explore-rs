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
      "tabdat-json-describe-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data schema.csv"),
      "x,y,label\n1.5,2,A\n2.5,5,B\n3.5,5,\n4.5,9,A\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "x,label\n").unwrap();
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
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn assert_success(output: &Output) {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
}

#[test]
fn describe_metadata_matches_load_and_canonical_nested_script_results() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use \"data schema.csv\"",
    "-c",
    "describe",
    "-c",
    "summarize x",
    "-c",
    "count",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 4);
  assert_eq!(results[1]["schema_version"], 1);
  assert_eq!(results[1]["result_type"], "DescribeResult");
  assert_eq!(results[1]["data"], results[0]["data"]);
  assert_eq!(
    results[1]["data"]["dataset"],
    json!({
      "path": "data schema.csv", "row_count": 4,
      "columns": [{"name":"x", "data_type":"DOUBLE"}, {"name":"y", "data_type":"BIGINT"}, {"name":"label", "data_type":"VARCHAR"}],
      "execution_mode": "eager", "lazy_engine": null, "label_metadata": null, "panel_metadata": null,
    })
  );
  fs::write(fixture.0.join("child.td"), "summarize x\ncount\n").unwrap();
  fs::write(fixture.0.join("middle.td"), "describe\nrun child.td\n").unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use \"data schema.csv\"\nrun middle.td\n",
  )
  .unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_success(&nested);
  assert_eq!(lines(&nested.stdout), results);
}

#[test]
fn describe_reports_current_schema_after_prediction_and_empty_relation_metadata() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use \"data schema.csv\"",
    "-c",
    "regress y x",
    "-c",
    "predict fitted",
    "-c",
    "describe",
    "-c",
    "lincom x",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 5);
  assert_eq!(results[3]["data"], results[2]["data"]);
  assert_eq!(results[3]["data"]["dataset"]["path"], "data schema.csv");
  assert_eq!(
    results[3]["data"]["dataset"]["columns"]
      .as_array()
      .unwrap()
      .iter()
      .map(|column| column["name"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["x", "y", "label", "fitted"]
  );
  assert_eq!(results[4]["result_type"], "LincomResult");
  let empty = fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "describe",
    "-c",
    "count",
  ]);
  assert_success(&empty);
  let results = lines(&empty.stdout);
  assert_eq!(results[1]["data"], results[0]["data"]);
  assert_eq!(results[1]["data"]["dataset"]["row_count"], 0);
  assert_eq!(results[2]["data"], json!({"row_count": 0}));
}

#[test]
fn describe_errors_keep_prior_results_child_locations_and_no_later_files() {
  let fixture = Fixture::new();
  let missing = fixture.batch(&["--json", "-c", "describe", "-c", "count"]);
  assert_eq!(missing.status.code(), Some(1));
  let results = lines(&missing.stdout);
  assert_eq!(results.len(), 1);
  assert_eq!(results[0]["error"]["type"], "ExecutionError");
  assert!(
    results[0]["error"]["message"]
      .as_str()
      .unwrap()
      .contains("active dataset")
  );
  let malformed = fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "describe extra",
    "-c",
    "count",
  ]);
  assert_eq!(malformed.status.code(), Some(2));
  let results = lines(&malformed.stdout);
  assert_eq!(results.len(), 2);
  assert_eq!(results[0]["result_type"], "LoadResult");
  assert_eq!(results[1]["error"]["type"], "ParseError");
  for (setup, command, code, line) in [
    ("", "describe", 1, 1),
    ("use empty.csv\n", "describe extra", 2, 2),
  ] {
    fs::write(
      fixture.0.join("child.td"),
      format!("{setup}{command}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(fixture.0.join("parent.td"), "run child.td\ncount\n").unwrap();
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(nested.status.code(), Some(code));
    let results = lines(&nested.stdout);
    assert_eq!(results.len(), if setup.is_empty() { 1 } else { 2 });
    let error = &results.last().unwrap()["error"];
    assert_eq!(error["type"], "ScriptError");
    assert_eq!(error["line"], line);
    assert_eq!(
      error["path"],
      fixture
        .0
        .join("child.td")
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .as_ref()
    );
    let plain = fixture.batch(&["-f", "parent.td"]);
    assert_eq!(plain.status.code(), nested.status.code());
    assert!(!fixture.0.join("never.csv").exists());
  }
}

#[test]
fn describe_serialization_preserves_all_existing_session_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let execute =
    |session: &mut Session, command: &str| session.execute(parse_command(command).unwrap());
  execute(
    &mut session,
    &format!("use \"{}\"", fixture.0.join("data schema.csv").display()),
  )
  .unwrap();
  execute(&mut session, "regress y x").unwrap();
  execute(&mut session, "label variable x \"Predictor label\"").unwrap();
  let before = session.state();
  let rows = format!("{:?}", execute(&mut session, "head 4").unwrap());
  let result = execute(&mut session, "describe").unwrap();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert!(parse_command("describe extra").is_err());
  assert_eq!(session.state(), before);
  assert_eq!(
    format!("{:?}", execute(&mut session, "head 4").unwrap()),
    rows
  );
}
