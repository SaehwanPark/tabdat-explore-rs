#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-ttest-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x1,x2,grp\n1,10,group1\n2,12,group1\n3,15,group2\n4,18,group2\n5,20,group1\n6,25,group2\n,9,group1\n8,,\n").unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
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
fn close(actual: &Value, expected: &Value) {
  if expected.is_null() {
    assert!(actual.is_null());
    return;
  }
  let actual = actual.as_f64().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected}, tolerance {tolerance}"
  );
}

#[test]
fn json_four_ttest_forms_match_scipy_and_nested_stream_without_losing_model() {
  let fixture = Fixture::new();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "regress x2 x1",
    "-c",
    "ttest x1 == 3",
    "-c",
    "ttest x1 == x2",
    "-c",
    "ttest x1, by(grp)",
    "-c",
    "ttest x1, by(grp) welch",
    "-c",
    "predict fitted",
    "-c",
    "head 8",
  ]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 8);
  assert_eq!(
    results
      .iter()
      .map(|result| result["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "RegressionResult",
      "TtestResult",
      "TtestResult",
      "TtestResult",
      "TtestResult",
      "PredictionResult",
      "PreviewResult"
    ]
  );
  assert!(results.iter().all(|result| result["schema_version"] == 1));
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/ttest-terminal-reference.json")).unwrap();
  for (index, case) in reference["cases"].as_array().unwrap().iter().enumerate() {
    let data = &results[index + 2]["data"];
    assert_eq!(data["varname1"], "x1");
    assert_eq!(data["is_paired"], index == 1);
    assert_eq!(data["is_welch"], index == 3);
    assert_eq!(
      data["varname2"],
      if index == 1 { json!("x2") } else { Value::Null }
    );
    assert_eq!(
      data["by_variable"],
      if index >= 2 {
        json!("grp")
      } else {
        Value::Null
      }
    );
    assert_eq!(data["null_value"], if index == 0 { 3.0 } else { 0.0 });
    assert_eq!(data["ci_level"], 95.0);
    assert_eq!(data["reference_distribution"], "student_t");
    let keys = match index {
      0 => vec!["group1_stats", "difference_stats"],
      1 => vec!["group1_stats", "group2_stats", "difference_stats"],
      _ => vec![
        "group1_stats",
        "group2_stats",
        "combined_stats",
        "difference_stats",
      ],
    };
    for (key, expected) in keys.iter().zip(case["rows"].as_array().unwrap()) {
      let actual = &data[key];
      assert_eq!(actual["name"], expected["name"]);
      assert_eq!(actual["count"], expected["n"]);
      if !expected["n"].is_null() {
        assert!(actual["count"].as_u64().is_some());
      }
      assert!(actual.get("observations").is_none());
      for (field, value) in [
        "mean",
        "standard_error",
        "standard_deviation",
        "ci_lower",
        "ci_upper",
      ]
      .iter()
      .zip(expected["values"].as_array().unwrap())
      {
        close(&actual[field], value);
      }
    }
    if index == 0 {
      assert!(data["group2_stats"].is_null());
    }
    if index < 2 {
      assert!(data["combined_stats"].is_null());
    }
    close(&data["t_statistic"], &case["t"]);
    close(&data["degrees_of_freedom"], &case["df"]);
    for (field, expected) in ["p_left", "p_two", "p_right"]
      .iter()
      .zip(case["probabilities"].as_array().unwrap())
    {
      close(&data[field], expected);
    }
  }
  assert_eq!(results[6]["data"]["dataset"]["row_count"], 8);
  assert_eq!(results[7]["data"]["rows"].as_array().unwrap().len(), 8);
  assert!(results[7]["data"]["rows"][6][3].is_null());
  assert!(results[7]["data"]["rows"][7][3].is_number());
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nregress x2 x1\nrun child.td\npredict fitted\nhead 8\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("child.td"),
    "ttest x1 == 3\nttest x1 == x2\nrun grandchild.td\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("grandchild.td"),
    "ttest x1, by(grp)\nttest x1, by(grp) welch\n",
  )
  .unwrap();
  let nested = fixture.run(&["--json", "-f", "parent.td"]);
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  assert_eq!(lines(&nested.stdout), results);
}

#[test]
fn json_ttest_errors_keep_prior_envelopes_and_stop_at_child_location() {
  let fixture = Fixture::new();
  for command in [
    "ttest missing == 0",
    "ttest x1, by(absent)",
    "ttest grp == 0",
  ] {
    let output = fixture.run(&["--json", "-c", "use data.csv", "-c", command, "-c", "count"]);
    assert_eq!(output.status.code(), Some(1));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["result_type"], "LoadResult");
    assert_eq!(results[1]["error"]["type"], "ExecutionError");
  }
  let output = fixture.run(&["--json", "-c", "ttest x1 == 0"]);
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(lines(&output.stdout).len(), 1);
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  for (command, code) in [("ttest absent == 0", 1), ("ttest x1 ==", 2)] {
    fs::write(
      fixture.0.join("child.td"),
      format!("ttest x1 == 3\n{command}\nexport never.csv\n"),
    )
    .unwrap();
    let output = fixture.run(&["--json", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(code));
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 3);
    assert_eq!(results[1]["result_type"], "TtestResult");
    assert_eq!(results[2]["error"]["type"], "ScriptError");
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
}

#[test]
fn json_undefined_ttest_inference_is_null_not_fabricated_zero() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("constant.csv"), "x,y,grp\n2,1,A\n2,1,B\n").unwrap();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use constant.csv",
    "-c",
    "ttest x == 2",
    "-c",
    "ttest x == y",
    "-c",
    "ttest x, by(grp)",
    "-c",
    "ttest x, by(grp) welch",
  ]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 5);
  for result in &results[1..] {
    for field in ["t_statistic", "p_left", "p_two", "p_right"] {
      assert!(result["data"][field].is_null());
    }
  }
  assert_eq!(results[1]["data"]["group1_stats"]["standard_error"], 0.0);
  assert_eq!(results[1]["data"]["group1_stats"]["ci_lower"], 2.0);
  assert!(results[3]["data"]["difference_stats"]["count"].is_null());
  assert!(results[4]["data"]["difference_stats"]["count"].is_null());
  assert_eq!(results[3]["data"]["group1_stats"]["count"], 1);
  assert!(results[3]["data"]["group1_stats"]["standard_deviation"].is_null());
}
