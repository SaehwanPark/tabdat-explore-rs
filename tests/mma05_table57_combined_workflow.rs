mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

const INPUT: &str = "source_row,y,x\n1,2.2,-1\n2,1.6,0\n3,2.2,1\n";

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma05table57/comparison.td"))
    .output()
    .unwrap()
}

#[test]
fn combined_reports_and_export_are_consistent() {
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
  let tables: Vec<_> = results
    .iter()
    .filter(|v| v["result_type"] == "EstimateTableResult")
    .collect();
  assert_eq!(tables.len(), 2);
  for (table, mode) in tables.iter().zip(["working", "robust"]) {
    let models = table["data"]["models"].as_array().unwrap();
    assert_eq!(models.len(), 5);
    for (model, name) in models.iter().zip(["ols", "exp", "nls", "wnls", "fgnls"]) {
      assert_eq!(model["name"], format!("{name}_{mode}"));
      assert_eq!(
        model["sample"]["retained_indices"],
        serde_json::json!([0, 1, 2])
      );
    }
    assert_eq!(models[1]["poisson_offset_variable"], "log_exposure");
    assert_eq!(models[2]["nls_model"]["outcome"], "y");
    assert_eq!(models[3]["nls_model"]["outcome"], "wnls_y");
    assert_eq!(models[4]["nls_model"]["outcome"], "fgnls_y");
  }
  let path = dir.path().join("table57_predictions.csv");
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
fn invalid_samples_stop_before_export() {
  for input in [
    "source_row,y,x\n1,0,-1\n2,1,0\n3,2,1\n",
    "source_row,y,x\n1,1,0\n2,2,0\n3,3,0\n",
    "source_row,y,x\n1,1,0\n1,2,1\n3,3,2\n",
    "source_row,y,x\n2,1,0\n1,2,1\n3,3,2\n",
    "source_row,y,x\n1,1,0\n2,1,1\n3,1,2\n",
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nls.csv"), input).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("table57_predictions.csv").exists());
  }
}
