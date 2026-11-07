mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
};

const EXPORT: &str = "native_conditional_fe_bootstrap_predictions.csv";
const RAW: &str = include_str!("fixtures/mma23_conditional_fe/source.asc");
fn root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn prepare(dir: &TestDir, raw: &str) {
  fs::write(dir.path().join("source.asc"), raw).unwrap();
  let output = Command::new("sh")
    .arg(root().join("examples/mma23p1pannonlin/prepare.sh"))
    .arg(dir.path().join("source.asc"))
    .arg(dir.path().join("patents.csv"))
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
}
fn run(dir: &TestDir, mode: &str, script: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(root().join("examples/mma23p1pannonlin").join(script))
    .output()
    .unwrap()
}
fn events(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= 1e-9,
    "{actual} != {expected}"
  );
}
fn report(events: &[Value]) -> &Value {
  let reports: Vec<_> = events
    .iter()
    .filter(|e| e["result_type"] == "ConditionalPoissonBootstrapResult")
    .collect();
  assert_eq!(reports.len(), 1);
  &reports[0]["data"]
}
fn check_control(events: &[Value]) {
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let d = report(events);
  let c = &d["coefficients"][0];
  assert_eq!(d["cluster_count"], 10);
  assert_eq!(d["replications"], 500);
  assert_eq!(d["zero_total_group_count"], 2);
  assert_eq!(d["zero_total_observation_count"], 10);
  assert_eq!(d["seed"], "10001");
  assert_eq!(d["interval_method"], "type7_percentile_95");
  assert_eq!(d["covariance_parameter_names"], serde_json::json!(["logr"]));
  let replicates = d["replicate_coefficients"].as_array().unwrap();
  assert_eq!(replicates.len(), 500);
  for (index, line) in include_str!("fixtures/mma23_native_conditional_fe_bootstrap/replicates.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let cells: Vec<_> = line.split(',').collect();
    assert_eq!(cells[0].parse::<usize>().unwrap(), index + 1);
    close(
      replicates[index][0].as_f64().unwrap(),
      cells[1].parse().unwrap(),
    );
  }
  let mut count = 0;
  for line in include_str!("fixtures/mma23_native_conditional_fe_bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let (field, value) = line.split_once(',').unwrap();
    let expected = value.parse().unwrap();
    let actual = match field {
      "candidate_rows" => d["joint_finite_observation_count"].as_f64().unwrap(),
      "candidate_firms" => d["candidate_group_count"].as_f64().unwrap(),
      "retained_rows" => d["observation_count"].as_f64().unwrap(),
      "firms_per_draw" => d["cluster_count"].as_f64().unwrap(),
      "replications" => d["replications"].as_f64().unwrap(),
      "bootstrap_se" => c["standard_error"].as_f64().unwrap(),
      name => c[name].as_f64().unwrap(),
    };
    close(actual, expected);
    count += 1;
  }
  assert_eq!(count, 11);
  close(
    d["covariance_matrix"][0][0].as_f64().unwrap(),
    c["standard_error"].as_f64().unwrap().powi(2),
  );
  assert!(
    d["replicate_cluster_indices"]
      .as_array()
      .unwrap()
      .iter()
      .all(|draw| draw.as_array().unwrap().len() == 10)
  );
  assert!(
    d["replicate_observation_counts"]
      .as_array()
      .unwrap()
      .iter()
      .all(|n| *n == 50)
  );
}

#[test]
fn all_r_refits_reports_and_original_owned_predictions_survive_bootstrap() {
  let json = TestDir::new();
  prepare(&json, RAW);
  check_control(&events(&run(
    &json,
    "--json",
    "native_conditional_fe_bootstrap.td",
  )));
  let after = fs::read(json.path().join(EXPORT)).unwrap();
  let original = TestDir::new();
  prepare(&original, RAW);
  events(&run(&original, "--json", "native_conditional_fe.td"));
  assert_eq!(
    after,
    fs::read(
      original
        .path()
        .join("native_conditional_fe_predictions.csv")
    )
    .unwrap()
  );
  let text = TestDir::new();
  prepare(&text, RAW);
  let output = run(&text, "--text", "native_conditional_fe_bootstrap.td");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert_eq!(after, fs::read(text.path().join(EXPORT)).unwrap());
  let collision = run(&json, "--json", "native_conditional_fe_bootstrap.td");
  assert!(!collision.status.success());
  assert_eq!(after, fs::read(json.path().join(EXPORT)).unwrap());
}

#[test]
fn permutation_and_missing_nonmodel_metadata_keep_frozen_refits() {
  let dir = TestDir::new();
  prepare(&dir, RAW);
  let csv = fs::read_to_string(dir.path().join("patents.csv")).unwrap();
  let mut lines = csv.lines();
  let header = lines.next().unwrap();
  let mut records: Vec<_> = lines
    .map(|line| {
      let mut cells: Vec<_> = line.split(',').collect();
      cells[3] = "";
      cells.join(",")
    })
    .collect();
  records.reverse();
  fs::write(
    dir.path().join("patents.csv"),
    format!("{header}\n{}\n", records.join("\n")),
  )
  .unwrap();
  check_control(&events(&run(
    &dir,
    "--json",
    "native_conditional_fe_bootstrap.td",
  )));
}

#[test]
fn actual_script_bootstraps_more_than_256_positive_firms_without_dropping_draws() {
  let raw = (0..257)
    .map(|group| {
      format!(
        "{} 11 1 1 30 0 1 2 3 4 0 1 2 3 4 1 2 3 4 5 1 2 3 4 5\n",
        2001 + group
      )
    })
    .collect::<String>();
  let dir = TestDir::new();
  prepare(&dir, &raw);
  let output = events(&run(&dir, "--json", "native_conditional_fe_bootstrap.td"));
  let d = report(&output);
  assert_eq!(d["cluster_count"], 257);
  assert_eq!(d["observation_count"], 1285);
  assert_eq!(d["replications"], 500);
  assert_eq!(d["zero_total_group_count"], 0);
  let point = d["coefficients"][0]["estimate"].as_f64().unwrap();
  for row in d["replicate_coefficients"].as_array().unwrap() {
    close(row[0].as_f64().unwrap(), point);
  }
  assert_eq!(
    d["replicate_cluster_indices"].as_array().unwrap().len(),
    500
  );
  for draw in d["replicate_cluster_indices"].as_array().unwrap() {
    assert_eq!(draw.as_array().unwrap().len(), 257);
    assert!(
      draw
        .as_array()
        .unwrap()
        .iter()
        .all(|index| index.as_u64().unwrap() < 257)
    );
  }
}
