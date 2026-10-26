mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

// Exact fit: mu=1, residuals (.1,-.2,.1), J=(-1,-x), x=(0,1,2).
// Thus B=M=[[3,3],[3,5]] and B^-1 M B^-1=[[5/6,-1/2],[-1/2,1/2]].
const INPUT: &str = "source_row,y,x\n1,1.1,0\n2,0.8,1\n3,1.1,2\n";

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p2nls/model_variance.td"))
    .output()
    .unwrap()
}

#[test]
fn model_variance_matches_closed_form_and_protects_export() {
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
  let fit = results
    .iter()
    .find(|v| v["result_type"] == "NlsResult")
    .unwrap();
  assert_eq!(
    fit["data"]["sample"]["retained_indices"],
    serde_json::json!([0, 1, 2])
  );
  for coefficient in fit["data"]["coefficients"].as_array().unwrap() {
    assert!(coefficient["value"].as_f64().unwrap().abs() < 1e-10);
  }
  let table = results
    .iter()
    .find(|v| v["data"]["headers"][7] == "cov_aa")
    .unwrap();
  let row = table["data"]["rows"][0].as_array().unwrap();
  for (i, expected) in [
    (1, 3.0),
    (2, 3.0),
    (3, 5.0),
    (4, 3.0),
    (5, 3.0),
    (6, 5.0),
    (7, 5.0 / 6.0),
    (8, -0.5),
    (9, 0.5),
    (10, (5.0_f64 / 6.0).sqrt()),
    (11, 0.5_f64.sqrt()),
  ] {
    assert!((row[i].as_f64().unwrap() - expected).abs() < 1e-10, "{i}");
  }
  assert_eq!(
    row[12],
    "variance-model plug-in, no HC correction; parameter order a,b"
  );
  let path = dir.path().join("nls_modelvar.csv");
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
    "source_row,y,x\n1,0,0\n2,1,1\n3,2,2\n",
    "source_row,y,x\n1,1,0\n2,2,0\n3,3,0\n",
    "source_row,y,x\n1,1,0\n1,2,1\n3,3,2\n",
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nls.csv"), input).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("nls_modelvar.csv").exists());
  }
}
