mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma25p1treatment/nearest_matching.td"),
    )
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
fn nearest_matches_independent_reference_and_preserves_export() {
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
          .any(|h| h == "matched_treated")
    })
    .unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  assert_eq!(values[0], 261);
  assert_eq!(values[1], 135);
  assert!((values[2].as_f64().unwrap() - 2090.02298850575).abs() < 1e-6);
  assert_eq!(values[4], 0);
  assert_eq!(values[5], 8.0);
  assert!(values[6].is_null());
  let path = dir.path().join("nearest_pairs.csv");
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
fn exact_ties_average_all_controls_and_preserve_reuse() {
  let original = input();
  let mut doubled = original.clone();
  let mut id = 500;
  for line in original.lines().skip(1) {
    let mut fields: Vec<_> = line.split(',').collect();
    if fields[1] == "0" {
      id += 1;
      let new_id = id.to_string();
      let changed_outcome = (fields[9].parse::<u64>().unwrap() + 1000).to_string();
      fields[0] = &new_id;
      fields[9] = &changed_outcome;
      doubled.push_str(&fields.join(","));
      doubled.push('\n');
    }
  }
  let dir = TestDir::new();
  fs::write(dir.path().join("nswpsid.csv"), doubled).unwrap();
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
          .any(|h| h == "matched_treated")
    })
    .unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  assert_eq!(values[0], values[4]);
  let csv = fs::read_to_string(dir.path().join("nearest_pairs.csv")).unwrap();
  let lines: Vec<Vec<&str>> = csv
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect();
  let first = lines[0][0];
  let selected: Vec<_> = lines.iter().filter(|line| line[0] == first).collect();
  assert_eq!(selected.len(), 2);
  let left = selected[0][5].parse::<f64>().unwrap();
  let right = selected[1][5].parse::<f64>().unwrap();
  let matched = selected[0][9].parse::<f64>().unwrap();
  assert!((matched - (left + right) / 2.0).abs() < 1e-10);
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
  let mut extreme = String::from("source_row,treat,age,educ,black,hisp,marr,re74,re75,re78\n");
  for line in data.lines().skip(1) {
    let mut fields: Vec<_> = line.split(',').collect();
    fields[9] = if fields[1] == "1" {
      "1.7e308"
    } else {
      "-1.7e308"
    };
    extreme.push_str(&fields.join(","));
    extreme.push('\n');
  }
  for data in [duplicate, invalid, constant, extreme] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nswpsid.csv"), data).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("nearest_pairs.csv").exists());
  }
}

#[test]
fn rounded_probabilities_are_allowed_only_when_outside_common_range() {
  let script = fs::read_to_string(
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma25p1treatment/nearest_matching.td"),
  )
  .unwrap();
  let (_, after_score) = script.split_once("predict pscore, pr\n").unwrap();
  let valid = "source_row,treat,pscore,re78\n1,1,0.2,10\n2,1,0.3,20\n3,1,0.8,30\n4,0,0,2\n5,0,0.1,3\n6,0,0.4,4\n7,0,0.7,7\n8,0,1,9\n";
  let invalid = "source_row,treat,pscore,re78\n1,1,0,10\n2,1,0.3,20\n3,1,0.8,30\n4,0,0,2\n5,0,0.2,3\n6,0,0.4,4\n7,0,0.9,7\n";
  let invalid_high = "source_row,treat,pscore,re78\n1,1,0.2,10\n2,1,0.4,20\n3,1,1,30\n4,0,0.1,2\n5,0,0.3,3\n6,0,0.4,4\n7,0,1,7\n";
  for (input, succeeds) in [(valid, true), (invalid, false), (invalid_high, false)] {
    let dir = TestDir::new();
    fs::write(dir.path().join("scores.csv"), input).unwrap();
    fs::write(
      dir.path().join("boundary.td"),
      format!("use scores.csv\n{after_score}"),
    )
    .unwrap();
    let output = tabdat_bin()
      .current_dir(dir.path())
      .args(["--json", "-f", "boundary.td"])
      .output()
      .unwrap();
    assert_eq!(
      output.status.success(),
      succeeds,
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    if succeeds {
      let rows: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
      let boundary = rows
        .iter()
        .find(|v| {
          v["result_type"] == "TableResult"
            && v["data"]["headers"]
              .as_array()
              .unwrap()
              .iter()
              .any(|h| h == "rounded_boundary_scores")
        })
        .unwrap();
      assert_eq!(boundary["data"]["rows"][0][0], 2);
      assert_eq!(boundary["data"]["rows"][0][1], 2);
    } else {
      assert!(!dir.path().join("nearest_pairs.csv").exists());
    }
  }
}
