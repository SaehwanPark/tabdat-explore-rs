mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma25p1treatment/stratified.td"))
    .output()
    .unwrap()
}

fn input() -> String {
  let mut state = 0x7a6d_25c1_u64;
  let mut draw = || {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
  };
  let mut text = String::from("source_row,treat,age,educ,black,hisp,marr,re74,re75,re78\n");
  for i in 1..=500 {
    let treat = draw() % 2;
    let age = 20 + draw() % 36;
    let educ = 8 + draw() % 11;
    let black = draw() % 2;
    let hisp = draw() % 2;
    let marr = draw() % 2;
    let re74 = if draw() % 5 == 0 {
      0
    } else {
      100 + draw() % 40000
    };
    let re75 = draw() % 40000;
    let re78 = draw() % 50000;
    text.push_str(&format!(
      "{i},{treat},{age},{educ},{black},{hisp},{marr},{re74},{re75},{re78}\n"
    ));
  }
  text
}

#[test]
fn stratified_comparison_matches_reference_and_preserves_export() {
  let dir = TestDir::new();
  fs::write(dir.path().join("nswpsid.csv"), input()).unwrap();
  let output = run(&dir, "--json");
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
  let table = rows
    .iter()
    .find(|v| {
      v["result_type"] == "TableResult"
        && v["data"]["headers"]
          .as_array()
          .unwrap()
          .iter()
          .any(|h| h == "treated_weighted_difference")
    })
    .unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  assert_eq!(values[1], 261);
  assert!((values[4].as_f64().unwrap() - 1961.35463993248).abs() < 1e-6);
  assert!(values[5].is_null());
  let cells = rows
    .iter()
    .find(|v| {
      v["result_type"] == "TableResult"
        && v["data"]["headers"]
          .as_array()
          .unwrap()
          .iter()
          .any(|h| h == "difference")
    })
    .unwrap();
  assert_eq!(cells["data"]["rows"].as_array().unwrap().len(), 10);
  let path = dir.path().join("stratified_records.csv");
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
fn zero_contrasts_remain_valid_and_are_not_dropped() {
  let mut constant = String::from("source_row,treat,age,educ,black,hisp,marr,re74,re75,re78\n");
  for line in input().lines().skip(1) {
    let mut parts: Vec<_> = line.split(',').collect();
    parts[9] = "100";
    constant.push_str(&parts.join(","));
    constant.push('\n');
  }
  let dir = TestDir::new();
  fs::write(dir.path().join("nswpsid.csv"), constant).unwrap();
  let output = run(&dir, "--json");
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
  let table = rows
    .iter()
    .find(|v| {
      v["result_type"] == "TableResult"
        && v["data"]["headers"]
          .as_array()
          .unwrap()
          .iter()
          .any(|h| h == "treated_weighted_difference")
    })
    .unwrap();
  assert_eq!(table["data"]["rows"][0][1], 261);
  assert_eq!(table["data"]["rows"][0][4], 0.0);
}

#[test]
fn invalid_identity_treatment_and_rank_fail_before_export() {
  let data = input();
  let duplicate = data.replacen("\n2,", "\n1,", 1);
  let mut lines = data.lines();
  let header = lines.next().unwrap();
  let first = lines.next().unwrap();
  let mut fields: Vec<_> = first.split(',').collect();
  fields[1] = "2";
  let invalid = format!(
    "{header}\n{}\n{}\n",
    fields.join(","),
    lines.collect::<Vec<_>>().join("\n")
  );
  let mut constant = String::from("source_row,treat,age,educ,black,hisp,marr,re74,re75,re78\n");
  for i in 1..=30 {
    constant.push_str(&format!("{i},{},30,12,0,0,0,10,12,20\n", i % 2));
  }
  for data in [duplicate, invalid, constant] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nswpsid.csv"), data).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("stratified_records.csv").exists());
  }
}
