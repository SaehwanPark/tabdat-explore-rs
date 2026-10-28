mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

#[test]
fn exactly_identified_json_does_not_claim_an_overidentification_test() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/cluster_gmm/data.csv"),
  )
  .unwrap();
  let output = tabdat_bin()
    .current_dir(dir.path())
    .args([
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "ivregress gmm y, endog(x) iv(z1) noconstant cluster(id)",
    ])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let data = &values
    .iter()
    .find(|v| v["result_type"] == "ClusterGmmResult")
    .unwrap()["data"];
  assert_eq!(data["overidentification_df"], 0);
  for key in [
    "fixed_weight_j",
    "fixed_weight_j_p_value",
    "author_updated_j",
    "author_updated_j_p_value",
  ] {
    assert!(data[key].is_null(), "{key}");
  }
}

#[test]
fn clustered_gmm_script_json_text_and_failure_paths() {
  let dir = TestDir::new();
  let data = include_str!("../crates/tabdat-stats/tests/fixtures/cluster_gmm/data.csv");
  fs::write(dir.path().join("data.csv"), data).unwrap();
  let script = "use data.csv\nivregress gmm y, endog(x) iv(z1 z2) noconstant cluster(id)\npredict fitted, xb\npredict residual, residuals\nexport results.csv\n";
  fs::write(dir.path().join("run.td"), script).unwrap();
  let run = |dir: &TestDir, mode: &str| {
    tabdat_bin()
      .current_dir(dir.path())
      .args([mode, "-f", "run.td"])
      .output()
      .unwrap()
  };
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let values: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let result = values
    .iter()
    .find(|v| v["result_type"] == "ClusterGmmResult")
    .unwrap();
  let data = &result["data"];
  assert_eq!(data["observation_count"], 48);
  assert_eq!(data["cluster_count"], 16);
  assert_eq!(data["overidentification_df"], 1);
  assert_eq!(data["reference_distribution"], "asymptotic_standard_normal");
  assert!((data["coefficients"][0]["value"].as_f64().unwrap() - 1.67646962863864).abs() < 1e-11);
  assert!((data["fixed_weight_j"].as_f64().unwrap() - 0.64560965718317).abs() < 1e-11);
  assert!((data["author_updated_j"].as_f64().unwrap() - 0.643992021170902).abs() < 1e-11);
  let exported = fs::read(dir.path().join("results.csv")).unwrap();
  assert_eq!(String::from_utf8_lossy(&exported).lines().count(), 49);
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(fs::read(dir.path().join("results.csv")).unwrap(), exported);

  let text_dir = TestDir::new();
  fs::write(
    text_dir.path().join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/cluster_gmm/data.csv"),
  )
  .unwrap();
  fs::write(text_dir.path().join("run.td"), script).unwrap();
  let text = run(&text_dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(String::from_utf8_lossy(&text.stdout).contains("Author-updated SE"));
  assert_eq!(
    fs::read(text_dir.path().join("results.csv")).unwrap(),
    exported
  );
}
