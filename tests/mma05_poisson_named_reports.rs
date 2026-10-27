mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[test]
fn source_comparison_preserves_live_four_fits_and_fixed_offset() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("nls.csv"),
    "source_row,y,x\n1,1.1,-1\n2,0.8,0\n3,1.1,1\n",
  )
  .unwrap();
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p1mle/named_reports.td");
  let output = tabdat_bin()
    .current_dir(dir.path())
    .args(["--json", "-f"])
    .arg(script)
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rows: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let fits: Vec<_> = rows
    .iter()
    .filter(|r| r["result_type"] == "RegressionResult" || r["result_type"] == "PoissonResult")
    .collect();
  let table = rows
    .iter()
    .find(|r| r["result_type"] == "EstimateTableResult")
    .unwrap();
  let snapshots = table["data"]["models"].as_array().unwrap();
  assert_eq!(fits.len(), 4);
  assert_eq!(snapshots.len(), 4);
  for (fit, snapshot) in fits.iter().zip(snapshots) {
    assert_eq!(
      fit["data"]["covariance_matrix"],
      snapshot["covariance_matrix"]
    );
    assert_eq!(
      fit["data"]["observation_count"],
      snapshot["observation_count"]
    );
    for (coefficient, copy) in fit["data"]["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .zip(snapshot["coefficients"].as_array().unwrap())
    {
      for key in ["name", "value", "standard_error", "statistic", "p_value"] {
        assert_eq!(coefficient[key], copy[key]);
      }
    }
  }
  for snapshot in &snapshots[2..] {
    assert_eq!(snapshot["poisson_offset_variable"], "log_exposure");
    assert_eq!(snapshot["poisson_offset_coefficient"], 1.0);
    assert_eq!(
      snapshot["poisson_null_model"],
      "intercept_only_with_same_fixed_offset"
    );
    assert_eq!(snapshot["log_likelihood_convention"], "poisson_count_pmf");
    assert_eq!(
      snapshot["sample"]["retained_indices"],
      serde_json::json!([0, 1, 2])
    );
  }
  assert_eq!(snapshots[2]["poisson_covariance_correction"], 1.0);
  assert_eq!(snapshots[3]["poisson_covariance_correction"], 1.5);
}
