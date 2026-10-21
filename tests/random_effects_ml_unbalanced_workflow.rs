mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{collections::BTreeMap, fs};

#[test]
fn unequal_ml_reports_actual_counts_information_and_population_exports() {
  for case in ["unequal", "shifted_interleaved", "boundary"] {
    let dir = TestDir::new();
    let input: Vec<_> =
      include_str!("../crates/tabdat-stats/tests/fixtures/random_effects_ml_unbalanced_inputs.csv")
        .lines()
        .skip(1)
        .map(|s| s.split(',').collect::<Vec<_>>())
        .filter(|r| r[0] == case)
        .collect();
    let mut csv = String::from("id,time,y,x,z\n");
    for row in &input {
      csv.push_str(&row[1..].join(","));
      csv.push('\n');
    }
    fs::write(dir.path().join("data.csv"), csv).unwrap();
    fs::write(dir.path().join("fit.td"),"use data.csv\nxtreg y x z, mle id(id) time(time)\npredict ub_xb\npredict ub_residual, residuals\nexport predictions.csv, replace\n").unwrap();
    let run = |mode| {
      tabdat_bin()
        .current_dir(dir.path())
        .args([mode, "-f", "fit.td"])
        .output()
        .unwrap()
    };
    let output = run("--json");
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let events: Vec<Value> = String::from_utf8(output.stdout)
      .unwrap()
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let fit = &events
      .iter()
      .find(|e| e["result_type"] == "UnbalancedRandomEffectsMlResult")
      .unwrap()["data"];
    assert_eq!(fit["estimator"], "unbalanced_gaussian_random_effects_ml");
    assert_eq!(fit["balanced"], false);
    assert!(fit["group_size"].is_null() && fit["theta"].is_null());
    assert_eq!(fit["minimum_group_size"], 1);
    assert_eq!(fit["maximum_group_size"], 5);
    assert_eq!(fit["eta_reference_group_size"], 1);
    assert_eq!(fit["ml"]["unit_variance_boundary"], case == "boundary");
    assert_eq!(fit["reference_distribution"], "normal");
    let variance_inference = &fit["ml"]["variance_component_inference"];
    assert_eq!(
      variance_inference["method"],
      "joint_observed_information_log_wald"
    );
    assert_eq!(
      variance_inference["reference_distribution"],
      "asymptotic_standard_normal"
    );
    assert_eq!(variance_inference["confidence_level"], 95.0);
    assert_eq!(
      variance_inference["sigma_e_squared"]["estimate"],
      fit["sigma_e_squared"]
    );
    if case == "boundary" {
      assert!(variance_inference["sigma_u_squared"].is_null());
      assert!(
        variance_inference["sigma_u_squared_unavailable_reason"]
          .as_str()
          .unwrap()
          .contains("boundary")
      );
    } else {
      assert!(variance_inference["sigma_u_squared"]["standard_error"].is_number());
      assert!(
        variance_inference["sigma_u_squared"]["confidence_interval"][0]
          .as_f64()
          .unwrap()
          > 0.0
      );
    }
    let reference: BTreeMap<_, f64> = include_str!(
      "../crates/tabdat-stats/tests/fixtures/random_effects_ml_unbalanced_reference.csv"
    )
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect::<Vec<_>>())
    .filter(|r| r[0] == case)
    .map(|r| (r[1].to_owned(), r[2].parse().unwrap()))
    .collect();
    for (path, expected) in &reference {
      if path.starts_with("/gls/fitted_values/")
        || path.starts_with("/gls/residuals/")
        || path.starts_with("/population_")
        || path == "/group_size"
      {
        continue;
      }
      let path = if let Some(rest) = path.strip_prefix("/gls/coefficients/") {
        format!("/coefficients/{rest}")
      } else if let Some(rest) = path.strip_prefix("/gls/covariance/matrix/") {
        format!("/covariance_matrix/{rest}")
      } else if let Some(rest) = path.strip_prefix("/confidence_intervals/") {
        let (index, side) = rest.split_once('/').unwrap();
        format!(
          "/coefficients/{index}/{}",
          if side == "0" { "ci_lower" } else { "ci_upper" }
        )
      } else if path == "/theta" {
        "/theta_at_minimum_group_size".into()
      } else {
        path.clone()
      };
      let actual = fit
        .pointer(&path)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{path}"));
      let tol = if path.ends_with("p_value") {
        expected.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        (actual - expected).abs() <= tol,
        "{case} {path}: {actual} vs {expected}"
      );
    }
    let json_csv = fs::read(dir.path().join("predictions.csv")).unwrap();
    for (i, line) in String::from_utf8(json_csv.clone())
      .unwrap()
      .lines()
      .skip(1)
      .enumerate()
    {
      let row: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
      for j in 0..5 {
        assert!((row[j] - input[i][j + 1].parse::<f64>().unwrap()).abs() < 1e-12);
      }
      assert!((row[5] - reference[&format!("/population_fitted_values/{i}")]).abs() < 1e-9);
      assert!((row[6] - reference[&format!("/population_residuals/{i}")]).abs() < 1e-9);
    }
    let text = run("--text");
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    for phrase in [
      "Unbalanced one-way random-effects ML",
      "T = 1..5",
      "Unequal retained counts",
      "theta range",
      "Variance-component inference: joint observed information",
      "sigma_e squared SE",
    ] {
      assert!(text.contains(phrase), "{text}");
    }
    assert_eq!(
      text.contains("sigma_u squared uncertainty unavailable"),
      case == "boundary"
    );
    assert_eq!(text.contains("sigma_u squared SE"), case != "boundary");
    assert_eq!(
      fs::read(dir.path().join("predictions.csv")).unwrap(),
      json_csv
    );
  }
}
