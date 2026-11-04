mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INPUT: &str = include_str!("fixtures/mma21_fe_level_bootstrap/heterogeneous_inputs.csv");
const OUTPUT: &str = "panel_manual_fe_covariance_predictions.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/mma21p1panel/manual_fe_covariance.td"),
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
      assert_eq!(values.len(), header.len());
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
#[test]
fn manual_auxiliary_covariances_match_qr_and_preserve_named_samples_and_predictions() {
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), INPUT).unwrap();
  let es = events(&run(&dir, "--json"));
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  assert_eq!(
    es[es.len() - 3..]
      .iter()
      .map(|e| e["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["SummarizeResult", "LincomResult", "ExportResult"]
  );
  let canonical = select("FixedEffectsResult");
  assert_eq!(canonical.len(), 1);
  let fits = select("RegressionResult");
  let intervals = select("LincomResult");
  assert_eq!(fits.len(), 3);
  assert_eq!(intervals.len(), 3);
  let data: Vec<_> = fits
    .iter()
    .zip(&intervals)
    .map(|(f, l)| json!({"fit":f,"lincom":l}))
    .collect();
  let reference = rows(include_str!(
    "fixtures/mma21_manual_fe_covariance/reference.csv"
  ));
  assert_eq!(reference.len(), 68);
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
  let tables = select("EstimateTableResult");
  assert_eq!(tables.len(), 1);
  let table = tables[0];
  assert_eq!(
    table["parameter_names"],
    json!(["intercept", "within_lnwg"])
  );
  let models = table["models"].as_array().unwrap();
  assert_eq!(models.len(), 3);
  let source = rows(INPUT);
  for (i, (model, fit)) in models.iter().zip(&fits).enumerate() {
    assert_eq!(
      model["name"],
      [
        "p2_fe_iid_working",
        "p2_fe_hc1_working",
        "p2_fe_person_cluster"
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
    assert_eq!(
      fit["sample"],
      json!({"total_observations":160,"dropped_observations":0})
    );
    assert_eq!(fit["covariance"], ["nonrobust", "robust", "cluster(id)"][i]);
    assert_eq!(model["reference_distribution"], "student_t");
    assert_eq!(
      model["inference_degrees_of_freedom"],
      if i == 2 { 15 } else { 158 }
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
    assert!(fit["coefficients"][0]["value"].as_f64().unwrap().abs() <= 1e-12);
  }
  assert!(
    fits[2]["coefficients"][0]["standard_error"]
      .as_f64()
      .unwrap()
      <= 1e-12
  );
  for value in fits[2]["covariance_matrix"][0].as_array().unwrap() {
    assert!(value.as_f64().unwrap().abs() <= 1e-12);
  }
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("floating-point artifacts")
  );
  let prediction = fs::read_to_string(dir.path().join(OUTPUT)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(include_str!(
    "fixtures/mma21_manual_fe_covariance/predictions.csv"
  ));
  assert_eq!(actual.len(), 160);
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 15);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    for (key, value) in b {
      let x: f64 = a[key].parse().unwrap();
      let y: f64 = value.parse().unwrap();
      if key.starts_with("within_") || key.starts_with("mean_") || key.starts_with("aux_") {
        assert!((x - y).abs() <= 1e-9, "{key}: {x} != {y}");
      } else {
        assert_eq!(x, y, "raw {key}");
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
    stdout.contains("must not be interpreted")
      && stdout.contains("95% CI lower")
      && stdout.contains("p2_fe_person_cluster")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
  fs::write(dir.path().join(OUTPUT), "existing owner\n").unwrap();
  assert!(!run(&dir, "--json").status.success());
  assert_eq!(
    fs::read_to_string(dir.path().join(OUTPUT)).unwrap(),
    "existing owner\n"
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    INPUT
  );
}
#[test]
fn valid_vanishing_cluster_slope_scores_do_not_require_invertible_joint_covariance() {
  let dir = TestDir::new();
  let mut input = String::from("source_row,lnhr,lnwg,kids,ageh,agesq,disab,id,year\n");
  let mut row = 0;
  for t in (0..10).rev() {
    for g in 1..=16 {
      row += 1;
      let x = g as f64 / 4.0 + t as f64 / 4.0;
      let z = t as f64 - 4.5;
      let y = 3.0 + g as f64 / 8.0 + 0.5 * x + (z * z - 8.25) / 16.0;
      input.push_str(&format!("{row},{y},{x},0,30,900,0,{g},{}\n", 1979 + t));
    }
  }
  fs::write(dir.path().join("MOM.csv"), &input).unwrap();
  let es = events(&run(&dir, "--json"));
  let fits: Vec<_> = es
    .iter()
    .filter(|e| e["result_type"] == "RegressionResult")
    .map(|e| &e["data"])
    .collect();
  assert_eq!(fits.len(), 3);
  assert!(
    fits[0]["coefficients"][1]["standard_error"]
      .as_f64()
      .unwrap()
      > 0.01
  );
  assert!((fits[2]["coefficients"][1]["value"].as_f64().unwrap() - 0.5).abs() <= 1e-12);
  assert!(
    fits[2]["coefficients"][1]["standard_error"]
      .as_f64()
      .unwrap()
      <= 1e-12
  );
  assert!(dir.path().join(OUTPUT).exists());
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn invalid_keys_samples_and_unidentified_within_design_do_not_export() {
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
  let mut constant = vec![lines[0].to_owned()];
  for line in &lines[1..] {
    let mut cells: Vec<_> = line.split(',').collect();
    cells[2] = cells[7];
    constant.push(cells.join(","));
  }
  variants.push(format!("{}\n", constant.join("\n")));
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
