mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_manual_gls_covariance/boundary_inputs.csv");
const ROWS: &str = "panel_re_residual_rows.csv";
const MATRIX: &str = "panel_re_residual_correlations.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/random_effects_residual_correlations.td"),
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
fn check(input: &str, reference: &str, predictions: &str, correlations: &str, boundary: bool) {
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
  let fits = select("RandomEffectsResult");
  assert_eq!(fits.len(), 1);
  let fit = fits[0];
  assert_eq!(fit["estimator"], "balanced_swamy_arora_random_effects_gls");
  assert_eq!(fit["covariance"], "nonrobust");
  assert_eq!(fit["reference_distribution"], "normal");
  assert_eq!(
    fit["sample"],
    json!({"total_observations":160,"dropped_observations":0})
  );
  let reference = rows(reference);
  assert_eq!(reference.len(), 30);
  for row in reference {
    let actual = fit.pointer(row["path"]).unwrap().as_f64().unwrap();
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
  let theta = fit["theta"].as_f64().unwrap();
  if boundary {
    assert_eq!(theta, 0.0);
    assert_eq!(fit["sigma_u_squared"], 0.0);
    assert!(fit["raw_sigma_u_squared"].as_f64().unwrap() < 0.0);
  } else {
    assert!(theta > 0.0 && theta < 1.0);
    assert!(fit["sigma_u_squared"].as_f64().unwrap() > 0.0);
  }
  assert_eq!(
    es[es.len() - 6..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "TableResult",
      "ActivateResult",
      "PredictionResult",
      "TableResult",
      "ActivateResult",
      "ExportResult"
    ]
  );
  let tables = select("TableResult");
  let retained: Vec<_> = tables
    .iter()
    .filter(|t| t["headers"] == json!(["retained_re_prediction_rows"]))
    .collect();
  assert_eq!(retained.len(), 1);
  assert_eq!(retained[0]["rows"], json!([[160]]));
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("descriptive across people, not a test or covariance correction")
  );
  let prediction = fs::read_to_string(dir.path().join(ROWS)).unwrap();
  let matrix = fs::read_to_string(dir.path().join(MATRIX)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!((actual.len(), expected.len()), (160, 160));
  let mut persons = BTreeMap::<&str, Vec<f64>>::new();
  let mut nonzero_effect = false;
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 24);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let val = |key: &str| a[key].parse::<f64>().unwrap();
    let residual = val("population_residual");
    let unit = val("re_unit_effect");
    let shrink = val("shrink");
    persons.entry(a["id"]).or_default().push(residual);
    nonzero_effect |= unit.abs() > 1e-8;
    assert!((val("lnhr") - val("population_xb") - residual).abs() <= 1e-9);
    assert!((val("population_xb") + unit - val("conditional_xbu")).abs() <= 1e-9);
    assert!((residual - unit - val("re_e")).abs() <= 1e-9);
    assert!((shrink - (1.0 - (1.0 - theta).powi(2))).abs() <= 1e-9);
    assert!((unit - shrink * val("mean_population_residual")).abs() <= 1e-9);
    for key in [
      "sigma_e_squared",
      "raw_sigma_u_squared",
      "sigma_u_squared",
      "theta",
    ] {
      assert!(
        (val(key) - fit[key].as_f64().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
    for (key, value) in b {
      let x = val(key);
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
        assert_eq!(x, y, "{key}");
      } else {
        assert!((x - y).abs() <= 1e-9, "{key}: {x} != {y}");
      }
    }
  }
  assert_eq!(persons.len(), 16);
  assert!(persons.values().all(|r| r.len() == 10));
  assert_eq!(nonzero_effect, !boundary);
  if !boundary {
    assert!(((1.0 - (1.0 - theta).powi(2)) - theta).abs() > 1e-3);
  }
  let actual = rows(&matrix);
  let expected = rows(correlations);
  assert_eq!((actual.len(), expected.len()), (100, 100));
  let table: Vec<_> = tables
    .iter()
    .filter(|t| t["headers"] == json!(["year_a", "year_b", "people", "correlation"]))
    .collect();
  assert_eq!(table.len(), 1);
  let table_rows = table[0]["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 100);
  for (i, ((a, b), row)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    let x = (1979 + i / 10) as f64;
    let y = (1979 + i % 10) as f64;
    assert_eq!(a["year_a"].parse::<f64>().unwrap(), x);
    assert_eq!(a["year_b"].parse::<f64>().unwrap(), y);
    assert_eq!(a["people"].parse::<f64>().unwrap(), 16.0);
    for key in ["year_a", "year_b", "people", "correlation"] {
      assert!(
        (a[key].parse::<f64>().unwrap() - b[key].parse::<f64>().unwrap()).abs() <= 1e-9,
        "{key}"
      );
    }
    assert_eq!(row[0].as_f64().unwrap(), x);
    assert_eq!(row[1].as_f64().unwrap(), y);
    assert_eq!(row[2], 16);
    assert!((row[3].as_f64().unwrap() - a["correlation"].parse::<f64>().unwrap()).abs() <= 1e-9);
  }
  fs::remove_file(dir.path().join(ROWS)).unwrap();
  fs::remove_file(dir.path().join(MATRIX)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(
    stdout.contains("not a test or covariance correction")
      && stdout.contains("retained_re_prediction_rows")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(ROWS)).unwrap(),
    prediction
  );
  assert_eq!(fs::read_to_string(dir.path().join(MATRIX)).unwrap(), matrix);
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn interior_re_conditional_residual_correlations_match_plm_ranef() {
  check(
    INTERIOR,
    include_str!("fixtures/mma21_re_residual_correlations/interior_reference.csv"),
    include_str!("fixtures/mma21_re_residual_correlations/interior_rows.csv"),
    include_str!("fixtures/mma21_re_residual_correlations/interior_correlations.csv"),
    false,
  );
}
#[test]
fn zero_component_re_correlations_match_plm_ranef() {
  check(
    BOUNDARY,
    include_str!("fixtures/mma21_re_residual_correlations/boundary_reference.csv"),
    include_str!("fixtures/mma21_re_residual_correlations/boundary_rows.csv"),
    include_str!("fixtures/mma21_re_residual_correlations/boundary_correlations.csv"),
    true,
  );
}
#[test]
fn invalid_keys_samples_rank_and_constant_year_fail_before_export() {
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
  for kind in 0..5 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut cells: Vec<_> = line.split(',').map(str::to_owned).collect();
      let g: f64 = cells[7].parse().unwrap();
      let x: f64 = cells[2].parse().unwrap();
      match kind {
        0 => cells[2] = "2".to_owned(),
        1 => cells[1] = (3.0 + 2.0 * x + (g % 4.0 - 1.5) / 2.0).to_string(),
        2 => {
          let t: f64 = cells[8].parse::<f64>().unwrap() - 1979.0;
          let x = 2.0 + g / 8.0 + t / 4.0;
          cells[2] = x.to_string();
          cells[1] = (7.0 + x / 2.0 + (t * t - 9.0 * t) / 128.0).to_string();
        }
        3 => cells[2] = (2.0 + (cells[8].parse::<f64>().unwrap() - 1979.0) / 4.0).to_string(),
        _ => cells[2] = cells[7].clone(),
      }
      changed.push(cells.join(","));
    }
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for (i, input) in variants.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), input).unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success(), "variant {i}");
    assert!(!result.stderr.is_empty());
    if i == 6 {
      assert!(String::from_utf8_lossy(&result.stderr).contains(":49: sql failed"));
      assert!(
        String::from_utf8_lossy(&result.stdout)
          .contains("Every year requires positive finite cross-person e variance")
      );
    }
    assert!(!dir.path().join(ROWS).exists() && !dir.path().join(MATRIX).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn both_export_collisions_preserve_owners_and_explicit_partial_output() {
  for existing in [ROWS, MATRIX] {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), INTERIOR).unwrap();
    fs::write(dir.path().join(existing), "existing owner\n").unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success());
    assert_eq!(
      fs::read_to_string(dir.path().join(existing)).unwrap(),
      "existing owner\n"
    );
    if existing == ROWS {
      assert!(!dir.path().join(MATRIX).exists());
    } else {
      assert_eq!(
        rows(&fs::read_to_string(dir.path().join(ROWS)).unwrap()).len(),
        160
      );
    }
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      INTERIOR
    );
  }
}
