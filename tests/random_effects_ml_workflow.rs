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
      let path = if let Some(p) = row[1].strip_prefix("/regression") {
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
