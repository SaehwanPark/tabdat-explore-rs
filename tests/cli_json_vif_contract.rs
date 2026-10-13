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
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(reference: &Value) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-vif-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = reference["columns"]
      .as_array()
      .unwrap()
      .iter()
      .map(|cell| cell.as_str().unwrap())
      .collect::<Vec<_>>()
      .join(",")
      + "\n";
    for row in reference["rows"].as_array().unwrap() {
      csv += &row
        .as_array()
        .unwrap()
        .iter()
        .map(|cell| match cell {
          Value::Null => String::new(),
          Value::String(text) => text.clone(),
          _ => cell.to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
      csv += "\n";
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    fs::write(root.join("other.csv"), "other\n100\n200\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn shell(&self, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(input.as_bytes())
      .unwrap();
    child.wait_with_output().unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn reference() -> Value {
  serde_json::from_str(include_str!("fixtures/vif-reporting-reference.json")).unwrap()
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
fn close(actual: &Value, expected: &Value) {
  let expected = expected.as_f64().unwrap();
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() <= 1e-10_f64.max(1e-9 * expected.abs()),
    "{actual} != {expected}"
  );
}

#[test]
fn json_vif_matches_independent_reference_and_actual_parent_sample_membership() {
  let reference = reference();
  let fixture = Fixture::new(&reference);
  for case in reference["cases"].as_array().unwrap() {
    let command = case["command"].as_str().unwrap();
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      command,
      "-c",
      "estat vif",
      "-c",
      "count",
    ]);
    assert_success(&output);
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 4);
    let table = &results[2];
    assert_eq!(table["schema_version"], 1);
    assert_eq!(table["result_type"], "TableResult");
    assert_eq!(table["data"]["headers"], json!(["Variable", "VIF"]));
    let rows = table["data"]["rows"].as_array().unwrap();
    let predictors = case["predictors"].as_array().unwrap();
    assert_eq!(rows.len(), predictors.len() + 1);
    for (index, name) in predictors.iter().enumerate() {
      assert_eq!(rows[index][0], *name);
      close(&rows[index][1], &case["vifs"][index]);
    }
    assert_eq!(rows.last().unwrap()[0], "mean_vif");
    close(&rows.last().unwrap()[1], &case["mean_vif"]);
    assert_eq!(
      results[1]["data"]["observation_count"],
      case["retained_indices"].as_array().unwrap().len()
    );
    assert_eq!(
      results[1]["data"]["include_intercept"],
      case["include_intercept"]
    );
    assert_eq!(results[3]["data"], json!({"row_count": 9}));
    let mut session = Session::new();
    session
      .execute(parse_command(&format!("use {}", fixture.0.join("data.csv").display())).unwrap())
      .unwrap();
    session.execute(parse_command(command).unwrap()).unwrap();
    assert_eq!(
      serde_json::to_value(
        &session
          .last_regression()
          .unwrap()
          .sample
          .as_ref()
          .unwrap()
          .retained_indices
      )
      .unwrap(),
      case["retained_indices"]
    );
  }
}

#[test]
fn nested_json_and_terminal_keep_supplied_vif_order_and_retained_design() {
  let fixture = Fixture::new(&reference());
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "regress y x2 x1 z",
    "-c",
    "estat vif",
    "-c",
    "use other.csv",
    "-c",
    "estat vif",
    "-c",
    "count",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results[2], results[4]);
  assert_eq!(results[5]["data"], json!({"row_count": 2}));
  fs::write(
    fixture.0.join("child.td"),
    "estat vif\nuse other.csv\nestat vif\ncount\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("middle.td"),
    "regress y x2 x1 z\nrun child.td\n",
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun middle.td\n").unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_success(&nested);
  assert_eq!(lines(&nested.stdout), results);
  let terminal = fixture
    .shell("use data.csv\nregress y x2 x1 z\nestat vif\nestat residuals\nestat vif\ncount\nexit\n");
  assert!(terminal.status.success());
  assert_eq!(
    String::from_utf8_lossy(&terminal.stderr)
      .matches("Error:")
      .count(),
    1
  );
  let text = String::from_utf8(terminal.stdout).unwrap();
  assert_eq!(text.matches("Variable\tVIF\n").count(), 2);
  assert!(text.ends_with("9 rows\n"));
}

#[test]
fn vif_errors_preserve_prior_streams_locations_and_stop_later_effects() {
  let fixture = Fixture::new(&reference());
  let no_data = fixture.batch(&["--json", "-c", "estat vif"]);
  assert_eq!(no_data.status.code(), Some(1));
  assert!(
    lines(&no_data.stdout)[0]["error"]["message"]
      .as_str()
      .unwrap()
      .contains("active dataset")
  );
  for command in ["estat residuals", "sql select 1"] {
    let unsupported =
      fixture.batch(&["--json", "-c", "use data.csv", "-c", command, "-c", "count"]);
    assert_eq!(unsupported.status.code(), Some(2));
    let results = lines(&unsupported.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[1]["error"]["type"], "JsonResultUnsupported");
    assert!(
      results[1]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("supported only")
    );
  }
  for (setup, command, code) in [
    ("", "estat vif", 1),
    ("regress y x1, noconstant\n", "estat vif", 1),
    ("regress y x2 x1 z\n", "estat residuals", 1),
    ("regress y x2 x1 z\n", "sql select 1", 1),
  ] {
    fs::write(
      fixture.0.join("child.td"),
      format!("{command}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("parent.td"),
      format!("use data.csv\n{setup}run child.td\ncount\n"),
    )
    .unwrap();
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(nested.status.code(), Some(code), "{command}");
    let results = lines(&nested.stdout);
    assert_eq!(results.len(), if setup.is_empty() { 2 } else { 3 });
    let error = &results.last().unwrap()["error"];
    assert_eq!(error["type"], "ScriptError");
    assert_eq!(error["line"], 1);
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
    assert!(!fixture.0.join("never.csv").exists());
  }
}

#[test]
fn vif_serialization_success_and_auxiliary_failure_do_not_change_data_or_model() {
  let fixture = Fixture::new(&reference());
  let mut session = Session::new();
  let execute =
    |session: &mut Session, command: &str| session.execute(parse_command(command).unwrap());
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  )
  .unwrap();
  for regression in ["regress y x2 x1 z", "regress y x1, noconstant"] {
    execute(&mut session, regression).unwrap();
    let dataset = session.active_dataset().cloned();
    let model = session.last_regression().cloned();
    let before = format!("{:?}", execute(&mut session, "head 9").unwrap());
    let result = execute(&mut session, "estat vif");
    if regression.contains("noconstant") {
      assert!(result.is_err());
    } else {
      let result = result.unwrap();
      assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
    }
    assert_eq!(session.active_dataset(), dataset.as_ref());
    assert_eq!(session.last_regression(), model.as_ref());
    assert_eq!(
      format!("{:?}", execute(&mut session, "head 9").unwrap()),
      before
    );
  }
}
