mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma25p1treatment/comparison.td"))
    .output()
    .unwrap()
}

fn input() -> String {
  let mut data = String::from("source_row,treat,age,educ,black,hisp,marr,re74,re75,re78\n");
  for i in 0..40 {
    let treat = i % 2;
    let age = 19 + (i * 13 % 31);
    let educ = 8 + (i * 7 % 10);
    let black = (i / 3) % 2;
    let hisp = (i / 5) % 2;
    let marr = (i / 7) % 2;
    let re74 = (i * 37 % 101) * 100;
    let re75 = (i * 53 % 127) * 100;
    let re78 = 1000 + (i * 43 % 137) * 100 + 200 * treat;
    data.push_str(&format!(
      "{},{treat},{age},{educ},{black},{hisp},{marr},{re74},{re75},{re78}\n",
      i + 1
    ));
  }
  data
}

#[test]
fn reports_and_predictions_preserve_export() {
  let dir = TestDir::new();
  fs::write(dir.path().join("nswpsid.csv"), input()).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let rows: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect();
  let tables: Vec<_> = rows
    .iter()
    .filter(|v| v["result_type"] == "EstimateTableResult")
    .collect();
  assert_eq!(tables.len(), 1);
  let models = tables[0]["data"]["models"].as_array().unwrap();
  assert_eq!(models.len(), 2);
  for (m, name) in models.iter().zip(["unadjusted", "adjusted"]) {
    assert_eq!(m["name"], name);
    assert_eq!(m["covariance"], "robust");
    assert_eq!(
      m["sample"]["retained_indices"].as_array().unwrap().len(),
      40
    );
  }
  let path = dir.path().join("treatment_predictions.csv");
  let original = fs::read(&path).unwrap();
  assert!(!run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
  fs::remove_file(&path).unwrap();
  assert!(run(&dir, "--text").status.success());
  assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn invalid_group_and_identity_fail_before_export() {
  for data in [
    input().replace(",1,", ",2,"),
    input().replace("2,1,", "1,1,"),
  ] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nswpsid.csv"), data).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("treatment_predictions.csv").exists());
  }
}
