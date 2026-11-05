mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};
const INPUT: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_manual_gls_covariance/boundary_inputs.csv");
const OUTPUT: &str = "panel_iid_hausman_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/iid_hausman.td"))
    .output()
    .unwrap()
}
fn rows(text: &str) -> Vec<BTreeMap<&str, &str>> {
  let mut lines = text.lines();
  let header: Vec<_> = lines.next().unwrap().split(',').collect();
  lines
    .map(|line| {
      let v: Vec<_> = line.split(',').collect();
      assert_eq!(v.len(), header.len());
      header.iter().copied().zip(v).collect()
    })
    .collect()
}
fn check(input: &str, reference: &str, predictions: &str) {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), input).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let es: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  let reference = rows(reference);
  assert_eq!(reference.len(), 39);
  for row in reference {
    let matches: Vec<_> = es
      .iter()
      .filter(|e| e["result_type"] == row["result_type"])
      .collect();
    assert_eq!(matches.len(), 1);
    let actual = matches[0]["data"]
      .pointer(row["path"])
      .unwrap()
      .as_f64()
      .unwrap();
    let expected: f64 = row["value"].parse().unwrap();
    let tolerance = if row["path"].ends_with("p_value") {
      expected.abs() * 1e-8
    } else {
      1e-9
    };
    assert!(
      actual.is_finite() && (actual - expected).abs() <= tolerance,
      "{row:?}: {actual} != {expected}"
    );
  }
  assert_eq!(
    es[es.len() - 4..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "SummarizeResult",
      "LincomResult",
      "TestResult",
      "ExportResult"
    ]
  );
  let fit = &es
    .iter()
    .find(|e| e["result_type"] == "RegressionResult")
    .unwrap()["data"];
  assert_eq!(fit["covariance"], "nonrobust");
  assert_eq!(fit["predictors"], json!(["quasi_lnwg", "within_lnwg"]));
  assert_eq!(
    fit["sample"],
    json!({"total_observations":160,"dropped_observations":0})
  );
  let test = &es
    .iter()
    .find(|e| e["result_type"] == "TestResult")
    .unwrap()["data"];
  assert_eq!(test["is_chi2"], false);
  assert_eq!(test["df_residual"], 157);
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("within-minus-Between")
  );
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!(actual.len(), 160);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 18);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (key, value) in b {
      let x: f64 = a[key].parse().unwrap();
      let y: f64 = value.parse().unwrap();
      if [
        "source_row",
        "lnhr",
        "lnwg",
        "kids",
        "ageh",
        "agesq",
        "disab",
        "id",
        "year",
      ]
      .contains(&key)
      {
        assert_eq!(x, y, "raw {key}");
      } else {
        assert!((x - y).abs() <= 1e-9, "{key}: {x} != {y}");
      }
    }
  }
  fs::remove_file(dir.path().join(OUTPUT)).unwrap();
  let output = run(&dir, "--text");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(
    text.contains("within-minus-Between")
      && text.contains("F(1,N-3)")
      && text.contains("CI lower (95%)")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  fs::write(dir.path().join(OUTPUT), "existing owner\n").unwrap();
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "existing owner\n"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn ordinary_auxiliary_diagnostic_matches_independent_interior_qr() {
  check(
    INPUT,
    include_str!("fixtures/mma21_iid_hausman/interior_reference.csv"),
    include_str!("fixtures/mma21_iid_hausman/interior_predictions.csv"),
  );
}
#[test]
fn null_contrast_at_theta_zero_retains_positive_ordinary_covariance() {
  check(
    BOUNDARY,
    include_str!("fixtures/mma21_iid_hausman/boundary_reference.csv"),
    include_str!("fixtures/mma21_iid_hausman/boundary_predictions.csv"),
  );
}
#[test]
fn invalid_structure_sample_and_components_do_not_export() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![
    format!("{INPUT}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (index, value) in [(2, ""), (7, "1.5")] {
    let mut changed = lines.clone();
    let mut cells: Vec<_> = changed[1].split(',').collect();
    cells[index] = value;
    let line = cells.join(",");
    changed[1] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for kind in 0..3 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
      let g: f64 = cells[7].parse().unwrap();
      let year: f64 = cells[8].parse().unwrap();
      let x: f64 = cells[2].parse().unwrap();
      match kind {
        0 => cells[2] = cells[7].clone(),
        1 => cells[2] = (2.0 + (year - 1979.0) / 4.0).to_string(),
        _ => cells[1] = (3.0 + 2.0 * x + (g % 4.0 - 1.5) / 2.0).to_string(),
      };
      changed.push(cells.join(","));
    }
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for input in variants {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), &input).unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success());
    assert!(!result.stderr.is_empty());
    assert!(!dir.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      input
    );
  }
}
