mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

const INPUT: &str =
  "source_row,treat,re75,re78\n1,0,1,2\n2,0,2,5\n3,0,4,5\n4,1,5,8\n5,1,7,11\n6,1,4,10\n";

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma25p1treatment/paired.td"))
    .output()
    .unwrap()
}

#[test]
fn paired_comparisons_use_person_clusters_and_preserve_export() {
  let dir = TestDir::new();
  fs::write(dir.path().join("nswpsid.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let results: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let intervals: Vec<_> = results
    .iter()
    .filter(|v| v["result_type"] == "LincomResult")
    .collect();
  assert_eq!(intervals.len(), 2);
  for (v, estimate, df) in intervals
    .iter()
    .zip([13.0 / 3.0, 8.0 / 3.0])
    .zip([2, 5])
    .map(|((v, e), df)| (v, e, df))
  {
    assert!((v["data"]["estimate"].as_f64().unwrap() - estimate).abs() < 1e-10);
    assert_eq!(v["data"]["df_residual"], df);
  }
  let table = results
    .iter()
    .find(|v| v["result_type"] == "EstimateTableResult")
    .unwrap();
  let models = table["data"]["models"].as_array().unwrap();
  assert_eq!(models.len(), 2);
  for (model, clusters) in models.iter().zip([3, 6]) {
    assert_eq!(model["covariance"], "cluster(source_row)");
    assert_eq!(model["cluster_count"], clusters);
  }
  assert_eq!(
    models[0]["sample"]["retained_indices"],
    serde_json::json!([6, 7, 8, 9, 10, 11])
  );
  let path = dir.path().join("paired_predictions.csv");
  let saved = fs::read(&path).unwrap();
  assert!(!run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), saved);
  fs::remove_file(&path).unwrap();
  assert!(run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), saved);
}

#[test]
fn bad_person_ids_groups_and_missing_earnings_stop_before_export() {
  for input in [
    INPUT.replace("2,0,2,5", "1,0,2,5"),
    INPUT.replace("1,0,1,2", "1,2,1,2"),
    INPUT.replace("1,0,1,2", "1,0,,2"),
    INPUT
      .replace("4,1,5,8", "4,0,5,8")
      .replace("5,1,7,11", "5,0,7,11"),
    INPUT.replace("1,0,1,2\n2,0,2,5", "2,0,2,5\n1,0,1,2"),
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nswpsid.csv"), input).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("paired_predictions.csv").exists());
  }
}
