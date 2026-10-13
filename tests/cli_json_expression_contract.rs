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
      "tabdat-json-expression-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,label\n1.5,2.0,A\n2.5,5.0,\n3.5,5.0,B\n4.5,9.0,A\n5.5,,B\n,8.0,C\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "label\n").unwrap();
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
fn close(actual: &Value, expected: f64) {
  assert!(
    (actual.as_f64().unwrap() - expected).abs() < 1e-10,
    "{actual} != {expected}"
  );
}

#[test]
fn json_transform_fit_predict_export_pipeline_matches_values_and_nested_stream() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "generate doubled = x * 2",
    "-c",
    "replace y = y + 1 if x > 2.5",
    "-c",
    "replace label = 'X' if label == null",
    "-c",
    "replace y = null if x == 1.5",
    "-c",
    "head 10",
    "-c",
    "regress y doubled",
    "-c",
    "predict fitted",
    "-c",
    "head 10",
    "-c",
    "export out.csv",
    "-c",
    "describe",
  ]);
  assert_success(&output);
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 11);
  for (index, result) in results.iter().enumerate().take(5).skip(1) {
    assert_eq!(result["schema_version"], 1);
    assert_eq!(
      result["result_type"],
      if index == 1 {
        "GenerateResult"
      } else {
        "ReplaceResult"
      }
    );
    assert_eq!(result["data"]["dataset"]["row_count"], 6);
    assert_eq!(result["data"]["dataset"]["path"], "data.csv");
    assert_eq!(
      result["data"]["dataset"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|column| column["name"].as_str().unwrap())
        .collect::<Vec<_>>(),
      ["x", "y", "label", "doubled"]
    );
    assert!(result["data"].get("affected_rows").is_none());
  }
  assert_eq!(
    results[5]["data"]["rows"],
    json!([
      [1.5, null, "A", 3.0],
      [2.5, 5.0, "X", 5.0],
      [3.5, 6.0, "B", 7.0],
      [4.5, 10.0, "A", 9.0],
      [5.5, null, "B", 11.0],
      [null, 8.0, "C", null]
    ])
  );
  assert_eq!(results[6]["data"]["observation_count"], 3);
  // Independent closed-form OLS on retained rows [1,2,3]: z=[5,7,9],
  // y=[5,6,10], Sxy/Sxx=10/8 and intercept=mean(y)-slope*mean(z).
  close(&results[6]["data"]["coefficients"][0]["value"], -1.75);
  close(&results[6]["data"]["coefficients"][1]["value"], 1.25);
  for (index, expected) in [2.0, 4.5, 7.0, 9.5, 12.0].iter().enumerate() {
    close(&results[8]["data"]["rows"][index][4], *expected);
  }
  assert!(results[8]["data"]["rows"][5][4].is_null());
  let reload = fixture.batch(&["--json", "-c", "use out.csv", "-c", "head 10"]);
  assert_success(&reload);
  assert_eq!(lines(&reload.stdout)[1]["data"], results[8]["data"]);
  fs::write(
    fixture.0.join("child.td"),
    "regress y doubled\npredict fitted\nhead 10\nexport out.csv, replace\ndescribe\n",
  )
  .unwrap();
  fs::write(fixture.0.join("middle.td"), "generate doubled = x * 2\nreplace y = y + 1 if x > 2.5\nreplace label = 'X' if label == null\nreplace y = null if x == 1.5\nhead 10\nrun child.td\n").unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun middle.td\n").unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_success(&nested);
  assert_eq!(lines(&nested.stdout), results);
  let mut session = Session::new();
  for source in [
    format!("use {}", fixture.0.join("data.csv").display()),
    "generate doubled = x * 2".to_owned(),
    "replace y = y + 1 if x > 2.5".to_owned(),
    "replace label = 'X' if label == null".to_owned(),
    "replace y = null if x == 1.5".to_owned(),
    "regress y doubled".to_owned(),
  ] {
    session.execute(parse_command(&source).unwrap()).unwrap();
  }
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    vec![1, 2, 3]
  );
}

#[test]
fn false_null_predicates_empty_data_and_quoted_columns_are_reported_without_fake_counts() {
  let fixture = Fixture::new();
  let baseline = fixture.batch(&["--json", "-c", "use data.csv", "-c", "head 10"]);
  assert_success(&baseline);
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "replace y = 99 if x < 0",
    "-c",
    "replace y = 99 if null",
    "-c",
    "head 10",
  ]);
  assert_success(&output);
  assert_eq!(
    lines(&output.stdout)[3]["data"],
    lines(&baseline.stdout)[1]["data"]
  );
  let empty = fixture.batch(&[
    "--json",
    "-c",
    "use empty.csv",
    "-c",
    "generate `倍 数` = 0",
    "-c",
    "replace `倍 数` = 1",
    "-c",
    "describe",
    "-c",
    "save empty.parquet",
  ]);
  assert_success(&empty);
  let results = lines(&empty.stdout);
  for index in [1, 2, 3, 4] {
    assert_eq!(results[index]["data"]["dataset"]["row_count"], 0);
  }
  assert_eq!(results[2]["data"]["dataset"]["columns"][1]["name"], "倍 数");
}

#[test]
fn invalid_and_unsupported_transforms_keep_results_locations_and_later_files_untouched() {
  let fixture = Fixture::new();
  for source in [
    "generate x = 1",
    "generate bad = absent + 1",
    "generate bad = 'text'",
    "generate bad = null",
    "generate bad = log(x)",
    "replace absent = 1",
    "replace label = 3",
    "replace y = 0 if x",
    "replace y = log(x)",
    "replace y = absent",
    "generate bad",
    "replace y",
  ] {
    let code = if source == "generate bad" || source == "replace y" {
      2
    } else {
      1
    };
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert_eq!(output.status.code(), Some(code), "{source}");
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
    let nested = fixture.batch(&["--json", "-f", "parent.td"]);
    assert_eq!(nested.status.code(), Some(code), "{source}");
    let results = lines(&nested.stdout);
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
  for source in ["generate new = 1", "replace y = 1"] {
    let output = fixture.batch(&["--json", "-c", source]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
      lines(&output.stdout)[0]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("active dataset")
    );
  }
}

#[test]
fn transform_failure_preserves_state_and_success_retains_model_with_existing_label_rules() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let execute =
    |session: &mut Session, source: &str| session.execute(parse_command(source).unwrap());
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  )
  .unwrap();
  execute(&mut session, "regress y x").unwrap();
  execute(&mut session, "label variable y 'Outcome'").unwrap();
  execute(&mut session, "label define ys 2 'two' 5 'five'").unwrap();
  execute(&mut session, "label values y ys").unwrap();
  let state = session.state();
  let rows = format!("{:?}", execute(&mut session, "head 10").unwrap());
  for source in [
    "generate x = 1",
    "generate bad = log(x)",
    "replace label = 3",
    "replace y = 0 if x",
    "replace y = absent",
  ] {
    assert!(execute(&mut session, source).is_err());
    assert_eq!(session.state(), state);
    assert_eq!(
      format!("{:?}", execute(&mut session, "head 10").unwrap()),
      rows
    );
  }
  let result = execute(&mut session, "generate doubled = x * 2").unwrap();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert_eq!(
    session.last_regression(),
    state
      .estimation
      .as_ref()
      .and_then(|state| state.linear())
      .map(|state| &state.result)
  );
  let model = session.last_regression().cloned();
  let result = execute(&mut session, "replace y = y + 1 if x > 2.5").unwrap();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert_eq!(session.last_regression(), model.as_ref());
  let labels = session.active_label_metadata().unwrap();
  assert!(labels.attachments.iter().all(|(name, _)| name != "y"));
  assert_eq!(labels.variable_labels, state.labels.variable_labels);
  assert_eq!(labels.value_sets, state.labels.value_sets);
}
