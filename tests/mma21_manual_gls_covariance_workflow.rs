mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_manual_gls_covariance/boundary_inputs.csv");
const OUTPUT: &str = "panel_manual_gls_covariance_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/manual_gls_covariance.td"),
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
fn events(output: &std::process::Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect()
}
fn check(input: &str, reference: &str, predictions: &str, boundary: bool) {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), input).unwrap();
  let es = events(&run(&dir, "--json"));
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  assert_eq!(
    es[es.len() - 4..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "SummarizeResult",
      "LincomResult",
      "LincomResult",
      "ExportResult"
    ]
  );
  assert_eq!(select("PredictionResult").len(), 4);
  let canonical = select("RandomEffectsResult");
  assert_eq!(canonical.len(), 1);
  let fits = select("RegressionResult");
  let intervals = select("LincomResult");
  assert_eq!(fits.len(), 3);
  assert_eq!(intervals.len(), 6);
  let data: Vec<_> = (0..3)
    .map(|i| json!({"fit":fits[i],"lincom0":intervals[2*i],"lincom1":intervals[2*i+1]}))
    .collect();
  let reference = rows(reference);
  assert_eq!(reference.len(), 120);
  for row in reference {
    let object = match row["model"] {
      "canonical" => canonical[0],
      "iid" => &data[0],
      "hc1" => &data[1],
      "cluster" => &data[2],
      _ => panic!(),
    };
    let actual = object.pointer(row["path"]).unwrap().as_f64().unwrap();
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
  let theta = canonical[0]["theta"].as_f64().unwrap();
  let raw = canonical[0]["raw_sigma_u_squared"].as_f64().unwrap();
  if boundary {
    assert!(raw < 0.0);
    assert_eq!(theta, 0.0);
    assert_eq!(canonical[0]["sigma_u_squared"], 0.0);
  } else {
    assert!(raw > 0.0 && theta > 0.0 && theta < 1.0);
  }
  let tables = select("EstimateTableResult");
  assert_eq!(tables.len(), 1);
  let table = tables[0];
  assert_eq!(table["parameter_names"], json!(["intercept", "quasi_lnwg"]));
  let models = table["models"].as_array().unwrap();
  assert_eq!(models.len(), 3);
  let source = rows(input);
  for (i, (model, fit)) in models.iter().zip(&fits).enumerate() {
    assert_eq!(
      model["name"],
      [
        "p2_gls_iid_working",
        "p2_gls_hc1_working",
        "p2_gls_person_cluster"
      ][i]
    );
    for key in [
      "coefficients",
      "covariance_matrix",
      "covariance_parameter_names",
      "covariance",
      "observation_count",
      "df_residual",
      "outcome",
      "predictors",
      "include_intercept",
      "estimator",
    ] {
      assert_eq!(model[key], fit[key], "{key}");
    }
    assert_eq!(fit["covariance"], ["nonrobust", "robust", "cluster(id)"][i]);
    assert_eq!(
      fit["sample"],
      json!({"total_observations":160,"dropped_observations":0})
    );
    assert_eq!(
      model["sample"]["retained_indices"],
      json!((0..160).collect::<Vec<_>>())
    );
    assert_eq!(model["sample"]["weights"], Value::Null);
    assert_eq!(
      model["sample"]["cluster_groups"],
      if i == 2 {
        json!(source.iter().map(|r| r["id"]).collect::<Vec<_>>())
      } else {
        Value::Null
      }
    );
    assert_eq!(
      model["cluster_count"],
      if i == 2 { json!(16) } else { Value::Null }
    );
    assert_eq!(
      model["cluster_variable"],
      if i == 2 { json!("id") } else { Value::Null }
    );
    assert_eq!(model["reference_distribution"], "student_t");
    assert_eq!(
      model["inference_degrees_of_freedom"],
      if i == 2 { 15 } else { 158 }
    );
  }
  let text = serde_json::to_string(&es).unwrap();
  assert!(
    text.contains("normal inference") && text.contains("Student-t") && text.contains("(1-theta)")
  );
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!(actual.len(), 160);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 23);
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
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(
    stdout.contains("normal inference")
      && stdout.contains("p2_gls_person_cluster")
      && stdout.contains("CI lower (95%)")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
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
fn original_theta_interior_quasi_covariances_match_independent_plm_and_qr() {
  check(
    INTERIOR,
    include_str!("fixtures/mma21_manual_gls_covariance/interior_reference.csv"),
    include_str!("fixtures/mma21_manual_gls_covariance/interior_predictions.csv"),
    false,
  );
}
#[test]
fn negative_raw_unit_variance_is_preserved_with_theta_zero_and_valid_inference() {
  check(
    BOUNDARY,
    include_str!("fixtures/mma21_manual_gls_covariance/boundary_reference.csv"),
    include_str!("fixtures/mma21_manual_gls_covariance/boundary_predictions.csv"),
    true,
  );
}
#[test]
fn invalid_keys_samples_and_component_designs_fail_without_export_or_source_change() {
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
