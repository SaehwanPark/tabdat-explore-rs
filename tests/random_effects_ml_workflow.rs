mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

#[test]
fn ml_cli_reports_joint_information_likelihood_and_population_exports() {
  for boundary in [false, true] {
    let dir = TestDir::new();
    let mut csv = String::from("id,time,y,x,z\n");
    for id in 1..=8 {
      let xs: Vec<_> = (0..4)
        .map(|t| 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5))
        .collect();
      let mean = xs.iter().sum::<f64>() / 4.0;
      for t in 0..4 {
        let x = xs[t as usize];
        let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
        let y = if boundary {
          0.7 * (x - mean) + [-0.002, 0.002, 0.002, -0.002][t as usize]
        } else {
          1.0 + 0.7 * x - 0.3 * z
            + 0.3 * f64::from((id * id + 1) % 5)
            + 0.11 * f64::from((id * t + 3) % 7)
        };
        csv.push_str(&format!("{id},{t},{y},{x},{z}\n"));
      }
    }
    fs::write(dir.path().join("data.csv"), csv).unwrap();
    fs::write(dir.path().join("fit.td"),"use data.csv\nxtreg y x z, mle id(id) time(time)\npredict xb\npredict residual, residuals\nexport predictions.csv, replace\n").unwrap();
    let run = |mode| {
      tabdat_bin()
        .current_dir(dir.path())
        .args([mode, "-f", "fit.td"])
        .output()
        .unwrap()
    };
    let out = run("--json");
    assert!(
      out.status.success(),
      "{}",
      String::from_utf8_lossy(&out.stderr)
    );
    let events: Vec<Value> = String::from_utf8(out.stdout)
      .unwrap()
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let fit = &events
      .iter()
      .find(|e| e["result_type"] == "RandomEffectsResult")
      .unwrap()["data"];
    assert_eq!(fit["estimator"], "balanced_gaussian_random_effects_ml");
    assert_eq!(fit["covariance_convention"], "joint_observed_information");
    assert_eq!(fit["reference_distribution"], "normal");
    assert_eq!(fit["covariance"], "observed_information");
    assert_eq!(fit["ml"]["likelihood_ratio_p_value"].is_null(), boundary);
    assert_eq!(
      fit["ml"]["likelihood_ratio_unavailable_reason"].is_string(),
      boundary
    );
    let component_inference = &fit["ml"]["variance_component_inference"];
    assert_eq!(
      component_inference["method"],
      "joint_observed_information_log_wald"
    );
    assert!(component_inference["sigma_e_squared"]["standard_error"].is_number());
    assert!(component_inference["sigma_u_squared"]["standard_error"].is_number());
    assert!(fit.get("r_squared").is_none());
    let case = if boundary {
      "null_boundary"
    } else {
      "balanced"
    };
    let mut count = 0;
    for line in
      include_str!("../crates/tabdat-stats/tests/fixtures/random_effects_ml_reference.csv")
        .lines()
        .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != case
        || row[1].starts_with("/regression/fitted_values")
        || row[1].starts_with("/regression/residuals")
      {
        continue;
      }
      let path = if row[1].starts_with("/ml/") {
        row[1].to_owned()
      } else if let Some(p) = row[1].strip_prefix("/regression") {
        p.to_owned()
      } else if [
        "/sigma_e_squared",
        "/sigma_u_squared",
        "/group_count",
        "/group_size",
        "/log_likelihood",
      ]
      .contains(&row[1])
      {
        row[1].into()
      } else {
        format!("/ml{}", row[1])
      };
      let actual = fit
        .pointer(&path)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{path}"));
      let expected: f64 = row[2].parse().unwrap();
      let tolerance = if path.ends_with("p_value") {
        expected.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{path}: {actual} vs {expected}"
      );
      count += 1;
    }
    assert!(count >= 47);
    let json_csv = fs::read(dir.path().join("predictions.csv")).unwrap();
    for line in String::from_utf8(json_csv.clone()).unwrap().lines().skip(1) {
      let r: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
      let b = &fit["coefficients"];
      let xb = b[0]["value"].as_f64().unwrap()
        + b[1]["value"].as_f64().unwrap() * r[3]
        + b[2]["value"].as_f64().unwrap() * r[4];
      assert!((r[5] - xb).abs() < 1e-12);
      assert!((r[6] - (r[2] - xb)).abs() < 1e-12);
    }
    let text = run("--text");
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    for expected in [
      "random-effects ML",
      "Joint observed-information",
      "Marginal log likelihood",
      "slope LR chi-square(2)",
      "Variance-component inference: joint observed information",
      "sigma_u squared SE",
      "CI lower (95%)",
    ] {
      assert!(text.contains(expected), "{text}");
    }
    assert_eq!(text.contains("Slope LR reference unavailable"), boundary);
    assert!(!text.contains("No panel likelihood"));
    assert_eq!(
      fs::read(dir.path().join("predictions.csv")).unwrap(),
      json_csv
    );
  }
}

#[test]
fn zero_unit_variance_ml_reports_constrained_information_and_exports_lm_predictions() {
  for case in ["pooled", "shifted_interleaved", "both_boundary"] {
    let dir = TestDir::new();
    let mut rows = Vec::new();
    for id in 1..=8 {
      for t in 0..4 {
        let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
        let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
        let y = (if case == "both_boundary" {
          0.0
        } else {
          0.7 * x
        }) + [-0.2, 0.2, 0.2, -0.2][t as usize]
          + if case == "shifted_interleaved" {
            100.0
          } else {
            0.0
          };
        rows.push((id, t, y, x, z));
      }
    }
    if case == "shifted_interleaved" {
      rows.sort_by_key(|r| (-r.1, -r.0));
    }
    let mut csv = String::from("id,time,y,x,z\n");
    for r in &rows {
      csv.push_str(&format!("{},{},{},{},{}\n", r.0, r.1, r.2, r.3, r.4));
    }
    fs::write(dir.path().join("data.csv"), csv).unwrap();
    fs::write(dir.path().join("fit.td"),"use data.csv\nxtreg y x z, mle id(id) time(time)\npredict xb\npredict residual, residuals\nexport predictions.csv, replace\n").unwrap();
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
      .find(|e| e["result_type"] == "RandomEffectsResult")
      .unwrap()["data"];
    assert_eq!(fit["ml"]["unit_variance_boundary"], true);
    assert_eq!(fit["covariance"], "observed_information");
    assert_eq!(
      fit["covariance_convention"],
      "constrained_observed_information"
    );
    assert_eq!(fit["reference_distribution"], "normal");
    assert_eq!(
      fit["ml"]["joint_parameter_names"],
      serde_json::json!(["intercept", "x", "z", "log_sigma_e_squared"])
    );
    assert!(fit["ml"]["likelihood_ratio_p_value"].is_null());
    assert!(
      fit["ml"]["likelihood_ratio_unavailable_reason"]
        .as_str()
        .unwrap()
        .contains("full-model")
    );
    assert!(fit["ml"]["variance_component_inference"]["sigma_u_squared"].is_null());
    assert!(
      fit["ml"]["variance_component_inference"]["sigma_u_squared_unavailable_reason"]
        .as_str()
        .unwrap()
        .contains("boundary")
    );
    let expected: std::collections::BTreeMap<_, f64> = include_str!(
      "../crates/tabdat-stats/tests/fixtures/random_effects_ml_boundary_reference.csv"
    )
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .filter(|row| row[0] == case)
    .map(|row| (row[1].to_owned(), row[2].parse().unwrap()))
    .collect();
    for (path, expected) in &expected {
      if path.starts_with("/gls/fitted_values/") || path.starts_with("/gls/residuals/") {
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
      } else {
        path.to_owned()
      };
      let actual = fit
        .pointer(&path)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{path}"));
      let tolerance = if path.ends_with("p_value") {
        expected.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        (actual - expected).abs() <= tolerance,
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
      let values: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
      assert_eq!(values[0], f64::from(rows[i].0));
      assert_eq!(values[1], f64::from(rows[i].1));
      assert!((values[5] - expected[&format!("/gls/fitted_values/{i}")]).abs() < 1e-9);
      assert!((values[6] - expected[&format!("/gls/residuals/{i}")]).abs() < 1e-9);
    }
    let text = run("--text");
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    for phrase in [
      "zero unit variance boundary",
      "Constrained observed-information",
      "No regular unit-variance SE or test",
      "sigma_u squared uncertainty unavailable",
      "Slope LR reference unavailable",
    ] {
      assert!(text.contains(phrase), "{text}");
    }
    assert_eq!(
      fs::read(dir.path().join("predictions.csv")).unwrap(),
      json_csv
    );
  }
}
