mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

fn run(dir: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma25p1treatment/overlap_effect.td"),
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
fn overlap_effect_matches_independent_reference_and_preserves_export() {
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
          .any(|h| h == "sandwich_se")
    })
    .unwrap();
  let values = table["data"]["rows"][0].as_array().unwrap();
  assert_eq!(values[0], 500);
  // Base-R QR/score equations on the deterministic 500-person fixture.
  for (i, expected) in [
    (4, 1824.41936397932),
    (5, 1307.71069363619),
    (8, 258.519383672882),
    (9, 234.195678078936),
  ] {
    assert!((values[i].as_f64().unwrap() - expected).abs() < 1e-6);
  }
  let path = dir.path().join("overlap_effect_rows.csv");
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
fn invalid_identity_treatment_and_degenerate_outcome_fail_before_export() {
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
  for line in data.lines().skip(1) {
    let mut parts: Vec<_> = line.split(',').collect();
    parts[9] = "100";
    constant.push_str(&parts.join(","));
    constant.push('\n');
  }
  for data in [duplicate, invalid, constant] {
    let dir = TestDir::new();
    fs::write(dir.path().join("nswpsid.csv"), data).unwrap();
    assert!(!run(&dir, "--json").status.success());
    assert!(!dir.path().join("overlap_effect_rows.csv").exists());
  }
}
