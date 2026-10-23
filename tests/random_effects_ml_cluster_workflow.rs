mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

fn reported_value(fit: &Value, path: &str) -> f64 {
  let native = if let Some(suffix) = path.strip_prefix("/gls/coefficients/") {
    format!("/coefficients/{suffix}")
  } else if let Some(suffix) = path.strip_prefix("/gls/covariance/matrix/") {
    format!("/covariance_matrix/{suffix}")
  } else if let Some(suffix) = path.strip_prefix("/confidence_intervals/") {
    let (index, bound) = suffix.split_once('/').unwrap();
    format!(
      "/coefficients/{index}/{}",
      if bound == "0" { "ci_lower" } else { "ci_upper" }
    )
  } else if path == "/theta" && fit["theta"].is_null() {
    "/theta_at_minimum_group_size".into()
  } else {
    path.into()
  };
  fit
    .pointer(&native)
    .and_then(Value::as_f64)
    .unwrap_or_else(|| panic!("{path}: {fit}"))
}

#[test]
fn interior_ml_cluster_scripts_match_r_report_full_joint_inference_and_preserve_exports() {
  let input =
    include_str!("../crates/tabdat-stats/tests/fixtures/random_effects_ml_cluster_inputs.csv");
  let reference =
    include_str!("../crates/tabdat-stats/tests/fixtures/random_effects_ml_cluster_reference.csv");
  let mut checked = 0;
  for case in [
    "balanced",
    "unequal",
    "shifted_interleaved",
    "null_boundary",
  ] {
    let dir = TestDir::new();
    let mut csv = String::from("source_row,id,time,y,x,z\n");
    for line in input.lines().skip(1) {
      let (name, row) = line.split_once(',').unwrap();
      if name == case {
        csv.push_str(row);
        csv.push('\n');
      }
    }
    fs::write(dir.path().join("data.csv"), &csv).unwrap();
    let execute = |mode: &str, option: &str| {
      fs::write(dir.path().join("fit.td"),format!("use data.csv\nxtreg y x z, mle id(id) time(time) {option}\npredict population_xb\npredict population_residual, residuals\nexport predictions.csv, replace\n")).unwrap();
      tabdat_bin()
        .current_dir(dir.path())
        .args([mode, "-f", "fit.td"])
        .output()
        .unwrap()
    };
    let output = execute("--json", "cluster(id)");
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let events: Vec<Value> = String::from_utf8_lossy(&output.stdout)
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    let event = &events[1];
    let fit = &event["data"];
    assert_eq!(
      event["result_type"],
      if case == "unequal" || case == "shifted_interleaved" {
        "UnbalancedRandomEffectsMlResult"
      } else {
        "RandomEffectsResult"
      }
    );
    assert_eq!(fit["covariance"], "cluster(id)");
    assert_eq!(
      fit["covariance_convention"],
      "unit_grouped_joint_ml_scores_observed_bread"
    );
    assert_eq!(fit["reference_distribution"], "normal");
    assert_eq!(fit["wald_reference"], "chi_square");
    assert_eq!(fit["cluster"]["group_count"], 8);
    assert_eq!(
      fit["cluster"]["correction_convention"],
      "gaussian_marginal_ml_full_joint_scores_g_over_g_minus_1"
    );
    assert_eq!(
      fit["ml"]["variance_component_inference"]["method"],
      "unit_cluster_joint_score_log_wald"
    );
    assert_eq!(fit["sample"]["dropped_observations"], 1);
    assert!(fit["ml"]["likelihood_ratio_p_value"].is_null());
    assert!(fit["ml"].get("unit_variance_test").is_none());
    assert!(
      fit["ml"]["unit_variance_test_unavailable_reason"]
        .as_str()
        .unwrap()
        .contains("unit-cluster")
    );
    for row in reference
      .lines()
      .skip(1)
      .map(|s| s.split(',').collect::<Vec<_>>())
      .filter(|r| r[0] == case)
    {
      let actual = reported_value(fit, row[1]);
      let expected = row[2].parse::<f64>().unwrap();
      let tolerance = if row[1].ends_with("p_value") {
        1e-7 * expected.abs().max(1e-300)
      } else {
        1e-8 * expected.abs().max(1.)
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{case} {}: {actual} vs {expected}",
        row[1]
      );
      checked += 1;
    }
    let exported = fs::read(dir.path().join("predictions.csv")).unwrap();
    let text = execute("--text", "robust");
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    assert_eq!(
      fs::read(dir.path().join("predictions.csv")).unwrap(),
      exported
    );
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("Full joint ML unit-score sandwich"));
    assert!(text.contains("multiplier G/(G-1)"));
    assert!(text.contains("no few-cluster guarantee"));
    assert!(text.contains("Unit-variance LR reference unavailable"));
    assert!(!text.contains("plm Arellano"));
    assert!(!text.contains("slope LR chi-square"));
    let alias = execute("--json", "robust");
    assert!(alias.status.success());
    let alias: Vec<Value> = String::from_utf8_lossy(&alias.stdout)
      .lines()
      .map(|s| serde_json::from_str(s).unwrap())
      .collect();
    assert_eq!(alias[1], *event);
    // Mean predictions depend only on the fitted mean model, not the selected covariance.
    let model = execute("--json", "");
    assert!(model.status.success());
    assert_eq!(
      fs::read(dir.path().join("predictions.csv")).unwrap(),
      exported
    );
    fs::write(
      dir.path().join("bad.td"),
      "use data.csv\nxtreg y x z, mle id(id) time(time) cluster(time)\nexport must-not-exist.csv\n",
    )
    .unwrap();
    let bad = tabdat_bin()
      .current_dir(dir.path())
      .args(["--json", "-f", "bad.td"])
      .output()
      .unwrap();
    assert_eq!(bad.status.code(), Some(1));
    assert!(
      String::from_utf8_lossy(&bad.stderr).contains("only cluster(id)"),
      "{}",
      String::from_utf8_lossy(&bad.stderr)
    );
    assert!(!dir.path().join("must-not-exist.csv").exists());
  }
  assert_eq!(checked, 292);
}
