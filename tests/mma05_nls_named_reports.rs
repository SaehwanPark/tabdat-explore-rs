mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[test]
fn source_example_compares_both_covariances_without_refitting_the_reports() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("nls.csv"),
    "source_row,y,x\n1,1.1,-1\n2,0.8,0\n3,1.1,1\n",
  )
  .unwrap();
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p2nls/named_reports.td");
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
    .filter(|r| r["result_type"] == "NlsResult")
    .collect();
  let table = rows
    .iter()
    .find(|r| r["result_type"] == "EstimateTableResult")
    .unwrap();
  let snapshots = table["data"]["models"].as_array().unwrap();
  assert_eq!(fits.len(), 2);
  assert_eq!(snapshots.len(), 2);
  for (fit, snapshot) in fits.iter().zip(snapshots) {
    assert_eq!(fit["data"]["model"], snapshot["nls_model"]);
    assert_eq!(fit["data"]["sample"], snapshot["sample"]);
    assert_eq!(
      fit["data"]["covariance_matrix"],
      snapshot["covariance_matrix"]
    );
    assert_eq!(
      fit["data"]["covariance_correction"],
      snapshot["nls_covariance_correction"]
    );
    assert_eq!(
      snapshot["sample"]["retained_indices"],
      serde_json::json!([0, 1, 2])
    );
  }
  assert_eq!(
    snapshots[0]["covariance"],
    "working_homoskedastic_gauss_newton"
  );
  assert_eq!(snapshots[1]["covariance"], "hc1_gauss_newton");
}
