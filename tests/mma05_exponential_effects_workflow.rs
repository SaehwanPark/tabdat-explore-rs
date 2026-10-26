mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05p4margeffects/workflow.td"))
    .output()
    .unwrap()
}

#[test]
fn effects_workflow_matches_profile_reference_and_protects_export() {
  let dir = TestDir::new();
  fs::write(
    dir.path().join("nls.csv"),
    "source_row,y,x\n1,0.3,-1\n2,0.8,0\n3,0.5,1\n4,1.8,2\n5,2.2,3\n6,4,4\n",
  )
  .unwrap();
  let result = run(&dir, "--json");
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  let results: Vec<Value> = String::from_utf8(result.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  let fit = results
    .iter()
    .find(|v| v["result_type"] == "PoissonResult")
    .unwrap();
  let coefficients = fit["data"]["coefficients"].as_array().unwrap();
  // Base R profile likelihood and root of the exponential score.
  for (row, expected) in coefficients
    .iter()
    .zip([0.5745177920466601, -0.4825999205518615])
  {
    assert!((row["value"].as_f64().unwrap() - expected).abs() < 1e-10);
  }
  let summary = results
    .iter()
    .find(|v| v["data"]["headers"][1] == "average_derivative")
    .unwrap();
  assert!((summary["data"]["rows"][0][1].as_f64().unwrap() - 0.7701924350951259).abs() < 1e-10);
  let path = dir.path().join("exponential_effects.csv");
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
