mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

const INPUT: &str = include_str!("fixtures/mma24_individual_poisson/input.csv");
const REFERENCE: &str = include_str!("fixtures/mma24_individual_poisson/reference.csv");
const PREDICTIONS: &str = include_str!("fixtures/mma24_individual_poisson/predictions.csv");
const OUTPUT: &str = "individual_poisson_predictions.csv";

fn example(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/mma24p2poiscluster")
    .join(name)
}
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(example("individual_poisson.td"))
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
  let tol = if p { b.abs() * 1e-8 } else { 1e-9 };
  assert!((a - b).abs() <= tol, "{a} != {b}, tolerance {tol}");
}

#[test]
fn synthetic_glm_score_delta_prediction_and_text_match_r() {
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam_individual.csv"), INPUT).unwrap();
  let output = run(&dir, "--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
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
  let fits = select("PoissonResult");
  let effects = select("PoissonMarginalEffectsResult");
  assert_eq!((fits.len(), effects.len()), (3, 3));
  for (fit, cov) in fits.iter().zip([
    "model_based_poisson_information",
    "robust_sandwich_n_over_n_minus_1",
    "cluster_sandwich_g_over_g_minus_1",
  ]) {
    assert_eq!(fit["covariance"], cov);
    assert_eq!(fit["observation_count"], 240);
    assert_eq!(
      fit["predictors"],
      json!([
        "lnhhexp",
        "insurance",
        "sex",
        "age",
        "married",
        "illdays",
        "actdays",
        "injury",
        "illness",
        "educ"
      ])
    );
    assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
    assert_eq!(fit["converged"], true);
  }
  assert_eq!(fits[2]["cluster"]["group_count"], 21);
  let reference = rows(REFERENCE);
  assert_eq!(reference.len(), 778);
  for r in reference {
    let mode = match r["mode"] {
      "classical" => 0,
      "robust" => 1,
      "cluster" => 2,
      _ => panic!(),
    };
    let (q, i, j) = (
      r["quantity"],
      r["i"].parse::<usize>().unwrap(),
      r["j"].parse::<usize>().unwrap(),
    );
    let value = if let Some(name) = q.strip_prefix("coefficient_") {
      &fits[mode]["coefficients"][i][name]
    } else if let Some(name) = q.strip_prefix("effect_") {
      &effects[mode]["effects"][i][name]
    } else if q == "covariance" {
      &fits[mode]["covariance_matrix"][i][j]
    } else if let Some(name) = q.strip_prefix("wald_") {
      &fits[mode]["omnibus_wald"][name]
    } else {
      &fits[mode][q]
    };
    close(
      value.as_f64().unwrap(),
      r["value"].parse().unwrap(),
      q.ends_with("p_value"),
    );
  }
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let (a, b) = (rows(&prediction), rows(PREDICTIONS));
  assert_eq!((a.len(), b.len()), (240, 240));
  for (a, b) in a.iter().zip(b.iter()) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in b.keys() {
      let av: f64 = a[key].parse().unwrap();
      let bv: f64 = b[key].parse().unwrap();
      if *key == "pooled_mu" || *key == "pooled_xb" {
        close(av, bv, false)
      } else {
        assert_eq!(av, bv, "{key}")
      }
    }
  }
  assert!(!a.iter().any(|r| r["source_row"] == "44"));
  assert!(a.iter().any(|r| r["pharvis"] == "0"));
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
      .contains("not a survey design, FE/RE, or causal analysis")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam_individual.csv")).unwrap(),
    INPUT
  );
}

#[test]
fn invalid_data_and_singular_design_fail_without_export_or_source_replacement() {
  let lines: Vec<_> = INPUT.lines().collect();
  let mut variants = vec![format!("{INPUT}{}\n", lines[1])];
  for (col, value) in [
    (0, "-1"),
    (0, "1.00000000000000001"),
    (1, "-1"),
    (1, "1.00000000000000001"),
    (1, "9007199254740992"),
    (2, "NaN"),
    (3, ""),
    (12, ""),
    (12, "1.00000000000000001"),
  ] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    rows[1][col] = value;
    variants.push(
      rows
        .iter()
        .map(|r| r.join(","))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n",
    );
  }
  for col in [1, 5, 12] {
    let mut rows: Vec<Vec<&str>> = lines.iter().map(|s| s.split(',').collect()).collect();
    for row in rows.iter_mut().skip(1) {
      if !row[1].is_empty() {
        row[col] = if col == 1 { "0" } else { "1" };
      }
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
    fs::write(dir.path().join("vietnam_individual.csv"), input).unwrap();
    let out = run(&dir, "--json");
    assert!(
      !out.status.success(),
      "invalid case {i} succeeded: {}",
      String::from_utf8_lossy(&out.stdout)
    );
    assert!(!dir.path().join(OUTPUT).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("vietnam_individual.csv")).unwrap(),
      *input
    );
  }
}

#[test]
fn separated_poisson_boundary_fails_without_export() {
  let mut rows: Vec<Vec<&str>> = INPUT.lines().map(|s| s.split(',').collect()).collect();
  for row in rows.iter_mut().skip(1) {
    if !row[1].is_empty() {
      row[1] = if row[11] == "0" { "0" } else { "1" };
    }
  }
  let input = rows
    .iter()
    .map(|row| row.join(","))
    .collect::<Vec<_>>()
    .join("\n")
    + "\n";
  let dir = TestDir::new();
  fs::write(dir.path().join("vietnam_individual.csv"), &input).unwrap();
  let output = run(&dir, "--json");
  assert!(!output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("possible boundary fit"));
  assert!(!dir.path().join(OUTPUT).exists());
  assert_eq!(
    fs::read_to_string(dir.path().join("vietnam_individual.csv")).unwrap(),
    input
  );
}

#[test]
fn owners_survive_bad_source_and_export_collision() {
  let dir = TestDir::new();
  let source = dir.path().join("bad.asc");
  let csv = dir.path().join("vietnam_individual.csv");
  fs::write(&source, "fake").unwrap();
  fs::write(&csv, INPUT).unwrap();
  let out = Command::new("python3")
    .arg(example("prepare.py"))
    .arg(&source)
    .arg(&csv)
    .output()
    .unwrap();
  assert!(!out.status.success());
  assert_eq!(fs::read_to_string(&csv).unwrap(), INPUT);
  assert_eq!(fs::read_to_string(&source).unwrap(), "fake");
  fs::write(dir.path().join(OUTPUT), "owner").unwrap();
  let out = run(&dir, "--json");
  assert!(!out.status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "owner"
  );
  assert_eq!(fs::read_to_string(&csv).unwrap(), INPUT);
}
