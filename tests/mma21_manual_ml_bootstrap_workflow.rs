mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_manual_gls_covariance/boundary_inputs.csv");
const OUTPUT: &str = "panel_manual_ml_bootstrap_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/manual_ml_bootstrap.td"),
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
fn check(input: &str, reference: &str, predictions: &str, boundary: bool) {
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
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let canonical = select("RandomEffectsResult");
  let boot = select("ClusterBootstrapOlsResult");
  let fit = select("RegressionResult");
  assert_eq!((canonical.len(), boot.len(), fit.len()), (1, 1, 1));
  assert_eq!(rows(reference).len(), 22);
  for row in rows(reference) {
    let object = match row["model"] {
      "canonical" => canonical[0],
      "bootstrap" => boot[0],
      _ => panic!(),
    };
    let actual = object.pointer(row["path"]).unwrap().as_f64().unwrap();
    let expected: f64 = row["value"].parse().unwrap();
    assert!(
      actual.is_finite() && (actual - expected).abs() <= 1e-9,
      "{row:?}: {actual} != {expected}"
    );
  }
  let theta = canonical[0]["theta"].as_f64().unwrap();
  if boundary {
    assert_eq!(theta, 0.0);
    assert_eq!(canonical[0]["sigma_u_squared"], 0.0);
  } else {
    assert!((theta - 0.702385318200663).abs() < 1e-9);
    assert!((theta - 0.720_342_426_153_654).abs() > 0.01);
  }
  assert_eq!(boot[0]["seed"], "10001");
  assert_eq!(boot[0]["rng"], "xoshiro256++/splitmix64/u64-rejection-v1");
  assert_eq!(boot[0]["interval_method"], "percentile_r_type7");
  assert_eq!(boot[0]["cluster_count"], 16);
  assert_eq!(boot[0]["cluster_sizes"], json!(vec![10; 16]));
  assert_eq!(
    boot[0]["replicate_observation_counts"],
    json!(vec![160; 500])
  );
  assert_eq!(boot[0]["cluster_order"], "first_seen_complete_case");
  assert_eq!(
    boot[0]["sample"],
    json!({"total_observations":160,"dropped_observations":0})
  );
  for i in 0..2 {
    assert!(
      (boot[0]["coefficients"][i]["estimate"].as_f64().unwrap()
        - fit[0]["coefficients"][i]["value"].as_f64().unwrap())
      .abs()
        <= 1e-9
    );
  }
  assert_eq!(
    es[es.len() - 5..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "TableResult",
      "PredictionResult",
      "PredictionResult",
      "SummarizeResult",
      "ExportResult"
    ]
  );
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("not any bootstrap replicate")
  );
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!((actual.len(), expected.len()), (160, 160));
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 22);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for key in ["theta", "sigma_e_squared", "sigma_u_squared"] {
      let reconstructed: f64 = a[key].parse().unwrap();
      assert!(
        (reconstructed - canonical[0][key].as_f64().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
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
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(
    stdout.contains("frozen original ML theta") && stdout.contains("not any bootstrap replicate")
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
fn interior_fixed_ml_bootstrap_matches_independent_likelihood_and_qr() {
  check(
    INTERIOR,
    include_str!("fixtures/mma21_manual_ml_bootstrap/interior_reference.csv"),
    include_str!("fixtures/mma21_manual_ml_bootstrap/interior_predictions.csv"),
    false,
  );
}
#[test]
fn boundary_fixed_ml_bootstrap_matches_independent_likelihood_and_qr() {
  check(
    BOUNDARY,
    include_str!("fixtures/mma21_manual_ml_bootstrap/boundary_reference.csv"),
    include_str!("fixtures/mma21_manual_ml_bootstrap/boundary_predictions.csv"),
    true,
  );
}
#[test]
fn invalid_inputs_and_failed_draw_do_not_export_or_modify_source() {
  let lines: Vec<_> = INTERIOR.lines().collect();
  let mut variants = vec![
    format!("{INTERIOR}{}\n", lines[1]),
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
  for kind in 0..2 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
      let g: f64 = cells[7].parse().unwrap();
      let x: f64 = cells[2].parse().unwrap();
      if kind == 0 {
        cells[2] = "2".to_owned();
      } else {
        cells[1] = (3.0 + 2.0 * x + (g % 4.0 - 1.5) / 2.0).to_string();
      }
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
  // Seed10001 draws only the first person in replicate 1; that person's x is constant.
  let mut rank = vec![lines[0].to_owned()];
  for line in &lines[1..] {
    let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
    if cells[7].parse::<usize>().unwrap() > 3 {
      continue;
    }
    cells[0] = rank.len().to_string();
    if cells[7] == "1" {
      cells[2] = "3.25".to_owned();
    }
    rank.push(cells.join(","));
  }
  let input = format!("{}\n", rank.join("\n"));
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), &input).unwrap();
  let result = run(&dir, "--json");
  assert!(!result.status.success());
  let error = String::from_utf8_lossy(&result.stderr);
  assert!(
    error.contains("replicate 1 failed") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  assert!(!dir.path().join(OUTPUT).exists());
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
