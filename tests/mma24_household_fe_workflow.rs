mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma24_household_fe/input.csv");
const REFERENCE: &str = include_str!("fixtures/mma24_household_fe/reference.csv");
const PREDICTIONS: &str = include_str!("fixtures/mma24_household_fe/predictions.csv");
const OUTPUT: &str = "household_fe_predictions.csv";

fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma24p1olscluster/household_fe.td"),
    )
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').collect();
      assert_eq!(header.len(), values.len());
      header.iter().copied().zip(values).collect()
    })
    .collect()
}
fn close(a: f64, b: f64, p: bool) {
  assert!(a.is_finite() && b.is_finite());
  let tolerance = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!(
    (a - b).abs() <= tolerance,
    "{a} != {b}, tolerance {tolerance}"
  );
}

#[test]
fn unbalanced_singleton_fit_and_predictions_match_full_dummy_qr() {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), INPUT).unwrap();
  let out = run(&dir, "--json");
  assert!(
    out.status.success(),
    "{}",
    String::from_utf8_lossy(&out.stderr)
  );
  let events: Vec<Value> = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert!(
    events
      .iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let select = |kind: &str| {
    events
      .iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let fits = select("FixedEffectsResult");
  let lincom = select("LincomResult");
  let test = select("TestResult");
  assert_eq!((fits.len(), lincom.len(), test.len()), (1, 2, 1));
  let fit = fits[0];
  assert_eq!(fit["observation_count"], 35);
  assert_eq!(fit["group_count"], 7);
  assert_eq!(fit["singleton_groups"], 1);
  assert_eq!(fit["df_residual"], 22);
  assert_eq!(fit["entity_variable"], "commune");
  assert_eq!(fit["time_variable"], "source_row");
  assert_eq!(
    fit["unit_effect_normalization"],
    "estimation_observation_weighted_zero_mean"
  );
  assert_eq!(
    fit["normalized_level"]["parameter_names"],
    json!(["_cons", "lnhhexp", "age", "sex", "hhsize", "farm", "educ"])
  );
  assert_eq!(lincom[0]["label"], "_cons");
  assert_eq!(lincom[1]["label"], "lnhhexp");
  assert_eq!(test[0]["df"], 2);
  assert_eq!(test[0]["df_residual"], 22);
  let reference = rows(REFERENCE);
  assert_eq!(reference.len(), 114);
  for r in reference {
    let obj = match r["kind"] {
      "fit" => fit,
      "common" => lincom[0],
      "slope" => lincom[1],
      "joint" => test[0],
      _ => panic!(),
    };
    close(
      obj.pointer(r["path"]).unwrap().as_f64().unwrap(),
      r["value"].parse().unwrap(),
      r["path"].ends_with("p_value"),
    );
  }
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let (a, b) = (rows(&prediction), rows(PREDICTIONS));
  assert_eq!((a.len(), b.len()), (35, 35));
  let mut sum_u = 0.0;
  for (a, b) in a.iter().zip(b.iter()) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if ["fe_xb", "fe_xbu", "fe_u", "fe_residual"].contains(key) {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}")
      }
    }
    sum_u += a["fe_u"].parse::<f64>().unwrap();
  }
  assert!((sum_u / 35.0).abs() <= 1e-12);
  assert!(
    a.iter()
      .any(|r| r["source_row"] == "5" && r["lnexp12m"].parse::<f64>().unwrap() == 0.0)
  );
  assert!(!a.iter().any(|r| r["source_row"] == "4"));
  let single = a.iter().find(|r| r["source_row"] == "36").unwrap();
  assert!(single["fe_residual"].parse::<f64>().unwrap().abs() <= 1e-12);
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  assert!(
    String::from_utf8(text.stdout)
      .unwrap()
      .contains("source_row is a unique key, not time")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn invalid_keys_nonfinite_rank_and_single_cluster_do_not_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![format!("{INPUT}{}\n", lines[1])];
  for (column, value) in [
    (0, "-1"),
    (0, "1.00000000000000001"),
    (8, ""),
    (8, "1.00000000000000001"),
    (1, "NaN"),
    (2, "NaN"),
  ] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    rows[1][column] = value;
    variants.push(
      rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for column in [6, 8] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    for row in rows.iter_mut().skip(1) {
      row[column] = "1";
    }
    variants.push(
      rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for (i, input) in variants.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("vietnam.csv"), input).unwrap();
    let out = run(&dir, "--json");
    assert!(
      !out.status.success(),
      "invalid case {i} succeeded: {}",
      String::from_utf8_lossy(&out.stdout)
    );
    assert!(!dir.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
      *input
    );
  }
}

#[test]
fn export_collision_keeps_owner_and_source() {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam.csv"), INPUT).unwrap();
  fs::write(dir.path().join(OUTPUT), "owner").unwrap();
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "owner"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam.csv")).unwrap(),
    INPUT
  );
}
