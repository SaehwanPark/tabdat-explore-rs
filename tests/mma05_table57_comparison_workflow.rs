mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

const INPUT: &str = "source_row,y,x\n1,0.3,-1\n2,0.8,0\n3,0.5,1\n4,1.8,2\n5,2.2,3\n6,4,4\n";

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p1mle/comparison.td"))
    .output()
    .unwrap()
}

#[test]
fn four_fits_match_independent_reference_and_protect_export() {
  let dir = TestDir::new();
  fs::write(dir.path().join("nls.csv"), INPUT).unwrap();
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
  let fits: Vec<&Value> = results
    .iter()
    .filter(|v| v["result_type"] == "RegressionResult" || v["result_type"] == "PoissonResult")
    .collect();
  assert_eq!(fits.len(), 4);
  // R QR OLS and profiled exponential likelihood with both covariance modes.
  let expected = [
    (
      [0.5714285714285714, 0.6857142857142857],
      [0.32955339806603534, 0.14498416521560314],
    ),
    (
      [0.5714285714285714, 0.6857142857142857],
      [0.29053618205080906, 0.15157482348234033],
    ),
    (
      [0.5745177920466601, -0.4825999205518615],
      [0.5430534367074646, 0.23873776264449362],
    ),
    (
      [0.5745177920466601, -0.4825999205518615],
      [0.18268118357128549, 0.04832905588939825],
    ),
  ];
  for ((fit, (beta, se)), covariance) in fits.iter().zip(expected).zip([
    "nonrobust",
    "robust",
    "model_based_poisson_information",
    "robust_sandwich_n_over_n_minus_1",
  ]) {
    assert_eq!(fit["data"]["covariance"], covariance);
    assert_eq!(fit["data"]["coefficients"].as_array().unwrap().len(), 2);
    assert_eq!(fit["data"]["observation_count"], 6);
    for (i, coefficient) in fit["data"]["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .enumerate()
    {
      assert!((coefficient["value"].as_f64().unwrap() - beta[i]).abs() < 1e-10);
      assert!((coefficient["standard_error"].as_f64().unwrap() - se[i]).abs() < 1e-10);
    }
  }
  let path = dir.path().join("comparison_predictions.csv");
  let original = fs::read(&path).unwrap();
  assert!(!run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
  fs::remove_file(&path).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn invalid_sample_stops_before_export() {
  for input in [
    "source_row,y,x\n1,0,-1\n2,1,0\n3,2,1\n",
    "source_row,y,x\n1,1,0\n2,2,0\n3,3,0\n",
    "source_row,y,x\n1,1,0\n1,2,1\n3,3,2\n",
    "source_row,y,x\n1,1,0\n2,1,1\n3,1,2\n",
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nls.csv"), input).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("comparison_predictions.csv").exists());
  }
}
