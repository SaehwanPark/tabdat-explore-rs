mod common;
use common::{TestDir, tabdat_bin};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

const INTERIOR: &str = include_str!("fixtures/mma21_manual_gls_covariance/interior_inputs.csv");
const BOUNDARY: &str = include_str!("fixtures/mma21_manual_gls_covariance/boundary_inputs.csv");
const ROWS: &str = "panel_joint_residual_rows.csv";
const SUMMARY: &str = "panel_joint_residual_summary.csv";
fn run(dir: &TestDir, mode: &str) -> std::process::Output {
  tabdat_bin()
    .current_dir(dir.path())
    .args([mode, "-f"])
    .arg(
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma21p1panel/residual_summary.td"),
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
fn check(input: &str, reference: &str, predictions: &str, summary: &str, boundary: bool) {
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
  assert!(
    es.iter()
      .all(|e| e["schema_version"] == 1 && e.get("error").is_none())
  );
  let select = |kind: &str| {
    es.iter()
      .filter(|e| e["result_type"] == kind)
      .map(|e| &e["data"])
      .collect::<Vec<_>>()
  };
  let pooled = select("RegressionResult");
  let fe = select("FixedEffectsResult");
  let re = select("RandomEffectsResult");
  assert_eq!((pooled.len(), fe.len(), re.len()), (1, 1, 1));
  assert_eq!(pooled[0]["outcome"], "lnhr");
  assert_eq!(pooled[0]["predictors"], json!(["lnwg"]));
  assert_eq!(pooled[0]["covariance"], "nonrobust");
  assert_eq!(fe[0]["estimator"], "within_fixed_effects");
  assert_eq!(
    fe[0]["unit_effect_normalization"],
    "estimation_observation_weighted_zero_mean"
  );
  assert_eq!(
    re[0]["estimator"],
    "balanced_swamy_arora_random_effects_gls"
  );
  assert_eq!(re[0]["variance_component_method"], "balanced_swamy_arora");
  assert_eq!(re[0]["reference_distribution"], "normal");
  for fit in [pooled[0], fe[0], re[0]] {
    assert_eq!(
      fit["sample"],
      json!({"total_observations":160,"dropped_observations":0})
    );
  }
  let reference = rows(reference);
  assert_eq!(reference.len(), 64);
  for row in reference {
    let object = match row["result_type"] {
      "RegressionResult" => pooled[0],
      "FixedEffectsResult" => fe[0],
      "RandomEffectsResult" => re[0],
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
  let theta = re[0]["theta"].as_f64().unwrap();
  if boundary {
    assert_eq!(theta, 0.0);
    assert_eq!(re[0]["sigma_u_squared"], 0.0);
    assert!(re[0]["raw_sigma_u_squared"].as_f64().unwrap() < 0.0);
  } else {
    assert!(theta > 0.0 && theta < 1.0);
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
  assert!(
    serde_json::to_string(&es)
      .unwrap()
      .contains("descriptive, not a test or covariance correction")
  );
  let tables = select("TableResult");
  let retained: Vec<_> = tables
    .iter()
    .filter(|t| t["headers"] == json!(["retained_re_prediction_rows"]))
    .collect();
  assert_eq!(retained.len(), 1);
  assert_eq!(retained[0]["rows"], json!([[160]]));
  let exports = select("ExportResult");
  assert_eq!(exports.len(), 2);
  for (e, name, count) in [(exports[0], ROWS, 160), (exports[1], SUMMARY, 55)] {
    assert_eq!(e["path"], name);
    assert_eq!(e["dataset"]["path"], name);
    assert_eq!(e["dataset"]["row_count"], count);
  }
  let prediction = fs::read_to_string(dir.path().join(ROWS)).unwrap();
  let summary_output = fs::read_to_string(dir.path().join(SUMMARY)).unwrap();
  let actual = rows(&prediction);
  let expected = rows(predictions);
  assert_eq!((actual.len(), expected.len()), (160, 160));
  let mut persons = BTreeMap::<&str, Vec<&BTreeMap<&str, &str>>>::new();
  let mut nonzero_effect = false;
  for (a, b) in actual.iter().zip(expected) {
    assert_eq!(a.len(), 30);
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
    let val = |key: &str| a[key].parse::<f64>().unwrap();
    persons.entry(a["id"]).or_default().push(a);
    let unit = val("re_unit_effect");
    nonzero_effect |= unit.abs() > 1e-8;
    assert!((val("lnhr") - val("pooled_xb") - val("pooled_residual")).abs() <= 1e-9);
    assert!((val("fe_xb") + val("fe_u") - val("fe_xbu")).abs() <= 1e-9);
    assert!((val("lnhr") - val("fe_xbu") - val("fe_residual")).abs() <= 1e-9);
    assert!((val("lnhr") - val("population_xb") - val("population_residual")).abs() <= 1e-9);
    assert!((val("shrink") - (1.0 - (1.0 - theta).powi(2))).abs() <= 1e-9);
    assert!((unit - val("shrink") * val("mean_population_residual")).abs() <= 1e-9);
    assert!((val("population_xb") + unit - val("conditional_xbu")).abs() <= 1e-9);
    assert!((val("population_residual") - unit - val("re_e")).abs() <= 1e-9);
    for key in [
      "sigma_e_squared",
      "raw_sigma_u_squared",
      "sigma_u_squared",
      "theta",
    ] {
      assert!(
        (val(key) - re[0][key].as_f64().unwrap()).abs() <= 1e-9,
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
  assert_eq!(nonzero_effect, !boundary);
  for group in persons.values() {
    assert_eq!(group.len(), 10);
    let mean = group
      .iter()
      .map(|a| a["population_residual"].parse::<f64>().unwrap())
      .sum::<f64>()
      / 10.0;
    let fe_sum = group
      .iter()
      .map(|a| a["fe_residual"].parse::<f64>().unwrap())
      .sum::<f64>();
    assert!(fe_sum.abs() <= 1e-9);
    for a in group {
      assert!((a["mean_population_residual"].parse::<f64>().unwrap() - mean).abs() <= 1e-9);
    }
  }
  let actual = rows(&summary_output);
  let expected = rows(summary);
  assert_eq!((actual.len(), expected.len()), (55, 55));
  let header = json!([
    "scope", "year", "variable", "n", "mean", "std_dev", "minimum", "maximum"
  ]);
  let summary_tables: Vec<_> = tables.iter().filter(|t| t["headers"] == header).collect();
  assert_eq!(summary_tables.len(), 1);
  let table_rows = summary_tables[0]["rows"].as_array().unwrap();
  assert_eq!(table_rows.len(), 55);
  for (i, ((a, b), r)) in actual.iter().zip(expected).zip(table_rows).enumerate() {
    let scope = if i < 5 { "overall" } else { "year" };
    let variable = ["fe_residual", "lnhr", "lnwg", "pooled_residual", "re_e"][i % 5];
    assert_eq!(a["scope"], scope);
    assert_eq!(b["scope"], scope);
    assert_eq!(r[0], scope);
    assert_eq!(a["variable"], variable);
    assert_eq!(b["variable"], variable);
    assert_eq!(r[2], variable);
    if i < 5 {
      assert_eq!(a["year"], "");
      assert_eq!(b["year"], "");
      assert!(r[1].is_null());
      assert_eq!(a["n"].parse::<usize>().unwrap(), 160);
    } else {
      let year = (1979 + (i - 5) / 5) as f64;
      assert_eq!(a["year"].parse::<f64>().unwrap(), year);
      assert_eq!(b["year"].parse::<f64>().unwrap(), year);
      assert_eq!(r[1].as_f64().unwrap(), year);
      assert_eq!(a["n"].parse::<usize>().unwrap(), 16);
    }
    assert_eq!(
      a["n"].parse::<usize>().unwrap(),
      b["n"].parse::<usize>().unwrap()
    );
    assert_eq!(
      r[3].as_u64().unwrap() as usize,
      a["n"].parse::<usize>().unwrap()
    );
    for (j, key) in ["mean", "std_dev", "minimum", "maximum"].iter().enumerate() {
      let x = a[key].parse::<f64>().unwrap();
      let y = b[key].parse::<f64>().unwrap();
      assert!((x - y).abs() <= 1e-9, "{i} {key}: {x} != {y}");
      assert!((r[j + 4].as_f64().unwrap() - x).abs() <= 1e-9);
    }
  }
  fs::remove_file(dir.path().join(ROWS)).unwrap();
  fs::remove_file(dir.path().join(SUMMARY)).unwrap();
  let text = run(&dir, "--text");
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let stdout = String::from_utf8(text.stdout).unwrap();
  assert!(
    stdout.contains("descriptive, not a test") && stdout.contains("retained_re_prediction_rows")
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(ROWS)).unwrap(),
    prediction
  );
  assert_eq!(
    fs::read_to_string(dir.path().join(SUMMARY)).unwrap(),
    summary_output
  );
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn interior_joint_snapshot_matches_three_independent_estimators() {
  check(
    INTERIOR,
    include_str!("fixtures/mma21_residual_summary/interior_reference.csv"),
    include_str!("fixtures/mma21_residual_summary/interior_rows.csv"),
    include_str!("fixtures/mma21_residual_summary/interior_summary.csv"),
    false,
  );
}
#[test]
fn zero_component_joint_snapshot_matches_three_independent_estimators() {
  check(
    BOUNDARY,
    include_str!("fixtures/mma21_residual_summary/boundary_reference.csv"),
    include_str!("fixtures/mma21_residual_summary/boundary_rows.csv"),
    include_str!("fixtures/mma21_residual_summary/boundary_summary.csv"),
    true,
  );
}
#[test]
fn constant_year_is_a_valid_summary_not_a_correlation_failure() {
  let lines: Vec<_> = INTERIOR.lines().collect();
  let mut changed = vec![lines[0].to_owned()];
  for line in &lines[1..] {
    let mut c: Vec<_> = line.split(',').collect();
    if c[8] == "1979" {
      c[1] = "7";
      c[2] = "2";
    }
    changed.push(c.join(","));
  }
  let input = format!("{}\n", changed.join("\n"));
  let dir = TestDir::new();
  fs::write(dir.path().join("MOM.csv"), &input).unwrap();
  let result = run(&dir, "--json");
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  let summary = fs::read_to_string(dir.path().join(SUMMARY)).unwrap();
  let all = rows(&summary);
  for variable in ["lnhr", "lnwg"] {
    let r = all
      .iter()
      .find(|r| {
        r["scope"] == "year"
          && r["year"].parse::<f64>().unwrap() == 1979.0
          && r["variable"] == variable
      })
      .unwrap();
    assert_eq!(r["std_dev"].parse::<f64>().unwrap(), 0.0);
  }
  assert_eq!(
    fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
    input
  );
}
#[test]
fn invalid_sample_rank_and_component_fail_before_export() {
  let lines: Vec<_> = INTERIOR.lines().collect();
  let mut variants = vec![
    format!("{INTERIOR}{}\n", lines[1]),
    format!("{}\n", lines[..lines.len() - 1].join("\n")),
  ];
  for (index, value) in [(0, "1.5"), (1, ""), (2, "NaN"), (7, "1.5"), (8, "1978")] {
    let mut changed = lines.clone();
    let mut c: Vec<_> = changed[1].split(',').collect();
    c[index] = value;
    let line = c.join(",");
    changed[1] = &line;
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for kind in 0..4 {
    let mut changed = vec![lines[0].to_owned()];
    for line in &lines[1..] {
      let mut c: Vec<_> = line.split(',').map(str::to_owned).collect();
      let g: f64 = c[7].parse().unwrap();
      let x: f64 = c[2].parse().unwrap();
      match kind {
        0 => c[2] = "2".to_owned(),
        1 => c[1] = (3.0 + 2.0 * x + (g % 4.0 - 1.5) / 2.0).to_string(),
        2 => c[2] = (2.0 + (c[8].parse::<f64>().unwrap() - 1979.0) / 4.0).to_string(),
        _ => c[2] = c[7].clone(),
      };
      changed.push(c.join(","));
    }
    variants.push(format!("{}\n", changed.join("\n")));
  }
  for (i, input) in variants.iter().enumerate() {
    let dir = TestDir::new();
    fs::write(dir.path().join("MOM.csv"), input).unwrap();
    let result = run(&dir, "--json");
    assert!(!result.status.success(), "variant {i}");
    assert!(!result.stderr.is_empty());
    assert!(!dir.path().join(ROWS).exists() && !dir.path().join(SUMMARY).exists());
    assert_eq!(
      fs::read_to_string(dir.path().join("MOM.csv")).unwrap(),
      *input
    );
  }
}
#[test]
fn both_export_collisions_preserve_owner_and_partial_first_file() {
  for existing in [ROWS, SUMMARY] {
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
      assert!(!dir.path().join(SUMMARY).exists());
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
