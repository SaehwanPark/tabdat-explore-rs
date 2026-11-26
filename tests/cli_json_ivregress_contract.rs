#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-cli-json-iv-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = String::from("y,w,x1,x2,z1,z2\n");
    for index in 1..=60 {
      let i = index as f64;
      let w = (i / 7.0).sin() + i / 90.0;
      let z1 = (i / 5.0).cos() + i / 70.0;
      let z2 = (i / 11.0).sin() - (i / 13.0).cos() + i / 200.0;
      let v1 = (i / 3.0).sin() + (i / 9.0).cos();
      let v2 = (i / 4.0).cos() - (i / 6.0).sin();
      let x1 = 0.3 + 0.4 * w + 0.9 * z1 + 0.2 * z2 + v1;
      let x2 = -0.2 + 0.2 * w - 0.3 * z1 + 1.1 * z2 + v2;
      let y =
        1.2 + 0.5 * w + 1.7 * x1 - 0.6 * x2 + (i / 1.7).sin() + (i / 2.3).cos() + (i / 5.9).sin();
      writeln!(csv, "{y},{w},{x1},{x2},{z1},{z2}").unwrap();
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }

  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }

  fn batch(&self, mode: &str, commands: &[&str]) -> Output {
    let mut args = vec![mode];
    for command in commands {
      args.extend(["-c", command]);
    }
    self.run(&args)
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn confidence_fixture(strength: f64, direct_z1: f64, direct_z2: f64) -> Fixture {
  let fixture = Fixture::new();
  let mut csv = String::from("y,w,x1,z1,z2,group_id\n");
  for i in 1..=40 {
    let basis = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let w = basis(5.0);
    let z1 = basis(1.0);
    let z2 = basis(2.0);
    let x1 = strength * z1 + basis(3.0);
    let y = 1.5 * x1 + 0.5 * w + direct_z1 * z1 + direct_z2 * z2 + basis(4.0);
    writeln!(csv, "{y},{w},{x1},{z1},{z2},{}", (i - 1) / 4).unwrap();
  }
  fs::write(fixture.0.join("data.csv"), csv).unwrap();
  fixture
}

#[test]
fn wu_hausman_scripts_and_json_report_the_joint_null_and_stop_after_undefined_variance() {
  // y=.9*x1+.5*w+.6*v+u with orthogonal x1=z1+v; RSS_R=23.6, RSS_U=20.
  let fixture = confidence_fixture(1.0, -0.6, 0.0);
  fs::write(
    fixture.0.join("endogenous.td"),
    "use data.csv\nivregress 2sls y w, endog(x1) iv(z1 z2)\nestat endogenous\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run(&["--json", "-f", "endogenous.td"]));
  assert_eq!(rows[2]["result_type"], "WuHausmanResult");
  let data = &rows[2]["data"];
  assert_eq!(data["test"], "wu_hausman_augmented_regression_f");
  assert_eq!(data["covariance"], "homoskedastic");
  assert_eq!(data["weak_iv_robust"], false);
  assert_eq!(data["null_scope"], "all_endogenous_joint");
  assert_eq!(
    data["null_hypothesis"],
    "all_declared_endogenous_regressors_exogenous"
  );
  assert_eq!(data["endogenous"], json!(["x1"]));
  assert_eq!(data["observation_count"], 40);
  assert_eq!(data["numerator_df"], 1);
  assert_eq!(data["denominator_df"], 36);
  assert!((data["statistic"].as_f64().unwrap() - 6.48).abs() < 1e-12);
  assert!((data["p_value"].as_f64().unwrap() - 0.01533570687687769).abs() < 1e-12);
  let output = fixture.run(&["--text", "-f", "endogenous.td"]);
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  for expected in [
    "Wu-Hausman joint endogeneity test",
    "F(1, 36)",
    "conditional normality",
    "does not prove exogeneity",
  ] {
    assert!(text.contains(expected), "{text}");
  }
  for mode in ["robust", "cluster(w)"] {
    let fit = format!("ivregress 2sls y w, endog(x1) iv(z1 z2) {mode}");
    fs::write(
      fixture.0.join("robust-endogenous.td"),
      format!("use data.csv\n{fit}\nestat endogenous\nreplace y = 999\nestat endogenous\n"),
    )
    .unwrap();
    let rows = json_lines(&fixture.run(&["--json", "-f", "robust-endogenous.td"]));
    assert_eq!(rows[2], rows[4]);
    assert_eq!(rows[2]["result_type"], "RobustEndogeneityResult");
    let data = &rows[2]["data"];
    assert_eq!(data["test"], "endogeneity_augmented_regression_robust_wald");
    assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
    assert_eq!(data["weak_iv_robust"], false);
    assert_eq!(data["null_scope"], "all_endogenous_joint");
    assert_eq!(data["degrees_of_freedom"], 1);
    assert_eq!(data["auxiliary_parameter_count"], 4);
    assert_eq!(data["auxiliary_residual_df"], 36);
    assert_eq!(
      data["covariance"],
      if mode == "robust" {
        "robust_hc1"
      } else {
        "cluster(w)"
      }
    );
    assert!(data["statistic"].as_f64().unwrap().is_finite());
    assert!(data["p_value"].as_f64().unwrap() > 0.0);
    let output = fixture.run(&["--text", "-f", "robust-endogenous.td"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
      "Robust joint endogeneity",
      "asymptotic chi-square(1)",
      "auxiliary K = 4",
      "no few-cluster guarantee",
      "not weak-IV-robust",
    ] {
      assert!(text.contains(expected), "{text}");
    }
  }
  fs::write(
    fixture.0.join("data.csv"),
    "y,x1,z1,z2\n0,0,1,0\n0,0,0,1\n0,1,0,0\n0,0,0,0\n",
  )
  .unwrap();
  for option in ["", " robust", " cluster(z2)"] {
    let output = fixture.batch(
      "--json",
      &[
        "use data.csv",
        &format!("ivregress 2sls y, endog(x1) iv(z1 z2){option}"),
        "estat endogenous",
        "export not-written.csv",
      ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.0.join("not-written.csv").exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("auxiliary residual variance"));
  }
}

#[test]
fn sargan_scripts_and_json_label_convention_and_fail_before_file_effects() {
  let fixture = confidence_fixture(2.0, 0.0, 0.3);
  fs::write(
    fixture.0.join("overid.td"),
    "use data.csv\nivregress 2sls y w, endog(x1) iv(z1 z2)\nestat overid\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run(&["--json", "-f", "overid.td"]));
  assert_eq!(rows[2]["result_type"], "SarganResult");
  let data = &rows[2]["data"];
  assert_eq!(data["test"], "sargan_n_r_squared");
  assert_eq!(data["covariance"], "homoskedastic");
  assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
  assert_eq!(data["weak_iv_robust"], false);
  assert_eq!(
    data["null_hypothesis"],
    "overidentifying_restrictions_hold_given_maintained_model"
  );
  assert!(
    data["assumptions"]
      .as_str()
      .unwrap()
      .contains("strong_identification")
  );
  assert_eq!(data["observation_count"], 40);
  assert_eq!(data["degrees_of_freedom"], 1);
  assert!((data["statistic"].as_f64().unwrap() - 3.30275229357798).abs() < 1e-12);
  assert!((data["p_value"].as_f64().unwrap() - 0.06916390601014773).abs() < 1e-12);
  let output = fixture.run(&["--text", "-f", "overid.td"]);
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  for expected in [
    "Sargan overidentification test (homoskedastic N*R²)",
    "chi-square(1)",
    "Asymptotic",
    "Failure to reject does not prove",
  ] {
    assert!(text.contains(expected), "{text}");
  }
  for fit in [
    "ivregress 2sls y w, endog(x1) iv(z1)",
    "ivregress 2sls y w, endog(x1) iv(z1) robust",
  ] {
    let output = fixture.batch(
      "--json",
      &[
        "use data.csv",
        fit,
        "estat overid",
        "export not-written.csv",
      ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.0.join("not-written.csv").exists());
  }
  fs::write(fixture.0.join("robust-overid.td"),
    "use data.csv\nivregress 2sls y w, endog(x1) iv(z1 z2) robust\nestat overid\nreplace y = 999\nestat overid\n"
  ).unwrap();
  let rows = json_lines(&fixture.run(&["--json", "-f", "robust-overid.td"]));
  assert_eq!(rows[2], rows[4]);
  assert_eq!(rows[2]["result_type"], "RobustOveridScoreResult");
  let data = &rows[2]["data"];
  assert_eq!(
    data["test"],
    "wooldridge_residualized_instrument_overid_score"
  );
  assert_eq!(data["covariance"], "independent_row_uncentered_score_opg");
  assert_eq!(data["covariance_correction"], 1.0);
  assert!(data.get("cluster_variable").is_none());
  assert!(data.get("cluster_group_count").is_none());
  assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
  assert_eq!(data["weak_iv_robust"], false);
  assert_eq!(data["instrument_parameter_count"], 4);
  assert_eq!(data["structural_parameter_count"], 3);
  assert_eq!(data["degrees_of_freedom"], 1);
  // Base R4.6.1 fixtures/robust_overid_score_reference.R CLI-design block.
  assert!((data["statistic"].as_f64().unwrap() - 3.17180616740088).abs() < 1e-12);
  assert!((data["p_value"].as_f64().unwrap() - 0.07491955220648723).abs() < 1e-12);
  let text = fixture.run(&["--text", "-f", "robust-overid.td"]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  for expected in [
    "Wooldridge residualized instruments",
    "asymptotic chi-square(1)",
    "no regression DF adjustment",
    "Not Hansen J",
    "Independent rows",
  ] {
    assert!(text.contains(expected), "{text}");
  }

  fs::write(
    fixture.0.join("clustered-overid.td"),
    "use data.csv\nivregress 2sls y w, endog(x1) iv(z1 z2) cluster(group_id)\nestat overid\nreplace y = 999\nestat overid\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run(&["--json", "-f", "clustered-overid.td"]));
  assert_eq!(rows[2], rows[4]);
  assert_eq!(rows[2]["result_type"], "RobustOveridScoreResult");
  let data = &rows[2]["data"];
  assert_eq!(
    data["covariance"],
    "one_way_cluster_uncentered_group_score_opg_cr0"
  );
  assert_eq!(data["cluster_variable"], "group_id");
  assert_eq!(data["cluster_group_count"], 10);
  assert_eq!(data["covariance_correction"], 1.0);
  assert_eq!(data["degrees_of_freedom"], 1);
  assert!(data["statistic"].as_f64().unwrap().is_finite());
  assert!(data["p_value"].as_f64().unwrap() > 0.0);
  let text = fixture.run(&["--text", "-f", "clustered-overid.td"]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  for expected in [
    "cluster(group_id) CR0 grouped score covariance; G = 10",
    "many-group asymptotics required",
    "no few-cluster guarantee",
  ] {
    assert!(text.contains(expected), "{text}");
  }
  // A genuine zero structural variance fit cannot supply either test.
  fs::write(
    fixture.0.join("data.csv"),
    "y,x1,z1,z2\n0,1,1,0\n0,0,0,1\n0,0,0,0\n0,0,0,0\n",
  )
  .unwrap();
  for option in ["", " robust"] {
    let output = fixture.batch(
      "--json",
      &[
        "use data.csv",
        &format!("ivregress 2sls y, endog(x1) iv(z1 z2){option}"),
        "estat overid",
        "export not-written.csv",
      ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("structural residual variance"));
    assert!(!fixture.0.join("not-written.csv").exists());
  }
}

#[test]
fn scalar_ar_scripts_report_all_set_shapes_without_clipping_infinity() {
  for (strength, z1, z2, shape, count, text_fragment) in [
    (2.0, 0.0, 0.0, "bounded", 1, "Anderson-Rubin confidence set"),
    (0.01, 0.0, 0.0, "all_real", 1, "(-inf, +inf)"),
    (0.1, 2.0, 0.0, "disjoint", 2, " union "),
    (2.0, 0.0, 10.0, "empty", 0, "Empty set"),
  ] {
    let fixture = confidence_fixture(strength, z1, z2);
    fs::write(
      fixture.0.join("ci.td"),
      "use data.csv\nivregress 2sls y w, endog(x1) iv(z1 z2)\nestat ar, ci\n",
    )
    .unwrap();
    let results = json_lines(&fixture.run(&["--json", "-f", "ci.td"]));
    assert_eq!(
      results[1]["data"]["first_stage"]["scalar_ar_confidence_set_available"],
      true
    );
    assert_eq!(results[2]["result_type"], "AndersonRubinConfidenceSet");
    let ci = &results[2]["data"];
    assert_eq!(ci["shape"], shape);
    assert_eq!(ci["confidence_level"], 95.0);
    assert_eq!(ci["endogenous"], "x1");
    assert_eq!(ci["observation_count"], 40);
    assert_eq!(ci["numerator_df"], 2);
    assert_eq!(ci["denominator_df"], 36);
    assert_eq!(
      ci["endogenous_nuisance_policy"],
      "none_single_endogenous_only"
    );
    assert_eq!(ci["covariance"], "homoskedastic");
    assert_eq!(ci["weak_iv_robust"], true);
    close(&ci["critical_f"], 3.259446306144109);
    assert_eq!(ci["intervals"].as_array().unwrap().len(), count);
    match shape {
      "bounded" => {
        close(&ci["intervals"][0]["lower"], 1.282246370466847);
        close(&ci["intervals"][0]["upper"], 1.717753629533153);
      }
      "all_real" => {
        assert!(ci["intervals"][0]["lower"].is_null());
        assert!(ci["intervals"][0]["upper"].is_null());
      }
      "disjoint" => {
        assert!(ci["intervals"][0]["lower"].is_null());
        assert!(ci["intervals"][1]["upper"].is_null());
        close(&ci["intervals"][0]["upper"], -4.536179309204275);
        close(&ci["intervals"][1]["lower"], 5.198096652710294);
      }
      _ => (),
    }
    let text = fixture.run(&["--text", "-f", "ci.td"]);
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains(text_fragment));
    assert!(text.contains("Exactly one endogenous coefficient"));
  }
  let fixture = confidence_fixture(2.0, 0.0, 0.0);
  let results = json_lines(&fixture.batch(
    "--json",
    &[
      "use data.csv",
      "ivregress 2sls y w, endog(x1) iv(z1 z2) noconstant",
      "estat ar, ci level(90)",
    ],
  ));
  let ci = &results[2]["data"];
  assert_eq!(ci["confidence_level"], 90.0);
  close(&ci["critical_f"], 2.45201433032702);
  close(&ci["intervals"][0]["lower"], 1.314875969835299);
  close(&ci["intervals"][0]["upper"], 1.685124030164702);
  let fixture = Fixture::new();
  let output = fixture.batch(
    "--json",
    &[
      "use data.csv",
      "ivregress 2sls y w, endog(x1 x2) iv(z1 z2)",
      "estat ar, ci",
      "export should-not-exist.csv",
    ],
  );
  assert_eq!(output.status.code(), Some(1));
  assert!(!fixture.0.join("should-not-exist.csv").exists());
  assert!(String::from_utf8_lossy(&output.stderr).contains("exactly one endogenous"));
  fs::write(
    fixture.0.join("data.csv"),
    "y,x1,z1,z2\n0,1,1,0\n0,0,0,1\n1,0,0,0\n0,0,0,0\n",
  )
  .unwrap();
  let output = fixture.batch(
    "--json",
    &[
      "use data.csv",
      "ivregress 2sls y, endog(x1) iv(z1 z2) noconstant",
      "estat ar, ci level(1e-16)",
      "export numerical-not-written.csv",
    ],
  );
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).contains("probability resolution"));
  assert!(!fixture.0.join("numerical-not-written.csv").exists());
}

#[test]
fn projected_ar_json_identifies_nuisance_policy_and_conservative_coverage() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("projected.td"),
    "use data.csv\nivregress 2sls y w, endog(x1 x2) iv(z1 z2)\nestat ar, ci coefficient(x1) level(90)\n",
  )
  .unwrap();
  let results = json_lines(&fixture.run(&["--json", "-f", "projected.td"]));
  assert_eq!(
    results[2]["result_type"],
    "ProjectedAndersonRubinConfidenceSet"
  );
  let data = &results[2]["data"];
  assert_eq!(
    data["test"],
    "anderson_rubin_projected_scalar_confidence_set"
  );
  assert_eq!(data["coverage"], "at_least_confidence_level_by_projection");
  assert_eq!(data["coefficient"], "x1");
  assert_eq!(data["nuisance_endogenous"], json!(["x2"]));
  assert_eq!(data["confidence_level"], 90.0);
  assert_eq!(data["numerator_df"], 2);
  assert_eq!(data["denominator_df"], 56);
  assert_eq!(data["all_real_conservative_fallback"], false);
  assert_eq!(data["weak_iv_robust"], true);
  assert_eq!(data["covariance"], "homoskedastic");
  assert_eq!(data["shape"], "bounded");
  let output = fixture.run(&["--text", "-f", "projected.td"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  for expected in [
    "Projected Anderson-Rubin confidence set",
    "at least nominal coverage",
    "Nuisance endogenous coefficients left unrestricted: [\"x2\"]",
    "at-least-nominal coverage",
  ] {
    assert!(text.contains(expected), "{text}");
  }
}

fn json_lines(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  std::str::from_utf8(&output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

fn close(actual: &Value, expected: f64) {
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() < 1e-8,
    "expected {expected:.12}, got {actual:.12}"
  );
}

#[test]
fn cli_json_and_text_report_iv_estimates_and_label_first_stage_limits() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "ivregress 2sls y w, endog(x1 x2) iv(z1 z2) robust",
    "lincom x1",
    "predict fitted, xb",
  ];
  let results = json_lines(&fixture.batch("--json", &commands));
  assert_eq!(
    results
      .iter()
      .map(|result| result["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "RegressionResult",
      "LincomResult",
      "PredictionResult"
    ]
  );
  let regression = &results[1]["data"];
  assert_eq!(regression["estimator"], "2sls");
  assert_eq!(regression["covariance"], "robust");
  assert_eq!(regression["predictors"], json!(["w", "x1", "x2"]));
  assert_eq!(
    regression["sample"],
    json!({"total_observations": 60, "dropped_observations": 0})
  );
  assert_eq!(
    regression["first_stage"]["weak_iv_robust_inference_available"],
    false
  );
  assert_eq!(
    regression["first_stage"]["coefficient_inference"],
    "conventional_t_not_weak_iv_robust"
  );
  assert_eq!(
    regression["first_stage"]["joint_ar_postestimation_available"],
    true
  );
  let first_stage = regression["first_stage"]["rows"].as_array().unwrap();
  assert_eq!(first_stage.len(), 2);
  assert_eq!(first_stage[0]["endogenous"], "x1");
  assert_eq!(first_stage[1]["endogenous"], "x2");
  close(&first_stage[0]["f_statistic"], 56.6351625385778);
  close(&first_stage[0]["partial_r_squared"], 0.6691682374068);
  close(&first_stage[0]["shea_partial_r_squared"], 0.46058071111269);
  close(&first_stage[1]["shea_partial_r_squared"], 0.29097041265132);
  close(&regression["coefficients"][2]["value"], 1.43920781503306);
  close(
    &regression["coefficients"][2]["standard_error"],
    0.17393281558833,
  );
  close(&results[2]["data"]["estimate"], 1.43920781503306);

  let text = fixture.batch("--text", &commands[..2]);
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("First stage:"));
  assert!(text.contains("Shea partial R-squared"));
  assert!(text.contains("F(2, 56)"));
  assert!(text.contains("coefficient p-values use conventional t inference"));
  assert!(text.contains("relevance diagnostics only"));
}

#[test]
fn joint_ar_runs_in_scripts_and_json_with_explicit_inference_scope() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("ar.td"),
    "use data.csv\nivregress 2sls y w, endog(x1 x2) iv(z1 z2)\nestat ar, null(1.7 -0.6)\n",
  )
  .unwrap();
  let results = json_lines(&fixture.run(&["--json", "-f", "ar.td"]));
  assert_eq!(
    results[1]["data"]["first_stage"]["joint_ar_postestimation_available"],
    true
  );
  let ar = &results[2];
  assert_eq!(ar["result_type"], "AndersonRubinResult");
  let data = &ar["data"];
  assert_eq!(data["test"], "anderson_rubin_joint");
  assert_eq!(data["covariance"], "homoskedastic");
  assert_eq!(data["null_scope"], "all_endogenous_joint");
  assert_eq!(data["weak_iv_robust"], true);
  assert_eq!(data["endogenous"], json!(["x1", "x2"]));
  assert_eq!(data["null_values"], json!([1.7, -0.6]));
  assert_eq!(data["observation_count"], 60);
  assert_eq!(data["numerator_df"], 2);
  assert_eq!(data["denominator_df"], 56);
  close(&data["statistic"], 5.422_721_819_930_05);
  close(&data["p_value"], 0.00703483761854517);
  assert!(
    data["assumptions"]
      .as_str()
      .unwrap()
      .contains("valid_instruments")
  );
  let text = fixture.run(&["--text", "-f", "ar.td"]);
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("Anderson-Rubin joint test"));
  assert!(text.contains("F(2, 56)"));
  assert!(text.contains("unrestricted nuisance parameters"));
  assert!(text.contains("No confidence set or subvector inference"));

  for option in ["", "robust"] {
    let null = "0";
    let output = fixture.batch(
      "--json",
      &[
        "use data.csv",
        &format!("ivregress 2sls y w, endog(x1 x2) iv(z1 z2) {option}"),
        &format!("estat ar, null({null})"),
        "export should-not-exist.csv",
      ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.0.join("should-not-exist.csv").exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("estat ar"));
  }
}

#[test]
fn robust_ar_script_reports_auxiliary_chi_square_and_retained_sample() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_data.csv"),
  )
  .unwrap();
  for (option, n, correction, statistic, p) in [
    (
      "robust",
      70,
      1.07692307692308,
      3.53464562620592,
      0.316296420698153,
    ),
    (
      "cluster(g)",
      69,
      1.15909090909091,
      35.2116876251244,
      1.099029699071e-7,
    ),
  ] {
    fs::write(fixture.0.join("robust-ar.td"), format!(
      "use data.csv\nivregress 2sls y w, endog(x1 x2) iv(z1 z2 z3) {option}\nestat ar, null(1.7 -0.6)\nreplace y = 999\ndrop z1\nestat ar, null(1.7 -0.6)\n"
    )).unwrap();
    let rows = json_lines(&fixture.run(&["--json", "-f", "robust-ar.td"]));
    let r = &rows[2];
    assert_eq!(r["result_type"], "RobustAndersonRubinResult");
    assert_eq!(
      rows[1]["data"]["first_stage"]["joint_ar_postestimation_available"],
      true
    );
    assert_eq!(
      rows[1]["data"]["first_stage"]["scalar_ar_confidence_set_available"],
      false
    );
    assert_eq!(r["data"], rows[5]["data"]);
    let d = &r["data"];
    assert_eq!(d["test"], "anderson_rubin_joint_robust_wald");
    assert_eq!(
      d["covariance"],
      if option == "robust" {
        "robust_hc1"
      } else {
        "cluster(g)"
      }
    );
    assert!(d.get("covariance_type").is_none());
    assert_eq!(d["reference_distribution"], "asymptotic_chi_square");
    assert_eq!(d["weak_iv_robust"], true);
    assert_eq!(d["null_scope"], "all_endogenous_joint");
    assert_eq!(d["endogenous"], json!(["x1", "x2"]));
    assert_eq!(d["null_values"], json!([1.7, -0.6]));
    assert_eq!(d["observation_count"], n);
    assert_eq!(d["auxiliary_parameter_count"], 5);
    assert_eq!(d["auxiliary_residual_df"], n - 5);
    assert_eq!(d["degrees_of_freedom"], 3);
    assert!(d.get("denominator_df").is_none());
    close(&d["covariance_correction"], correction);
    close(&d["statistic"], statistic);
    assert!((d["p_value"].as_f64().unwrap() - p).abs() < 1e-10 * p);
    assert_eq!(
      d["cluster_group_count"],
      if option == "robust" {
        json!(null)
      } else {
        json!(12)
      }
    );
    let text = fixture.run(&["--text", "-f", "robust-ar.td"]);
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("asymptotic chi-square(3)"));
    assert!(text.contains("No finite-sample or few-cluster guarantee"));
    assert!(text.contains("unrestricted nuisance parameters"));
  }
  // A rank-limited joint restriction must fail without dropping instruments or
  // running the later file effect, while preserving earlier ordered outputs.
  let output = fixture.batch(
    "--json",
    &[
      "use data.csv",
      "sql SELECT *, source_row % 3 AS few FROM active into few_groups",
      "use few_groups",
      "ivregress 2sls y w, endog(x1 x2) iv(z1 z2 z3) cluster(few)",
      "estat ar, null(1.7 -0.6)",
      "export should-not-exist.csv",
    ],
  );
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr).contains("insufficient_cluster_rank"),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rows = String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str::<Value>(line).unwrap())
    .collect::<Vec<_>>();
  assert_eq!(rows.len(), 5);
  assert_eq!(rows[3]["result_type"], "RegressionResult");
  assert_eq!(rows[4]["error"]["type"], "ExecutionError");
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}

#[test]
fn three_instrument_robust_ar_confidence_sets_report_json_and_text() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_three_ci_data.csv"),
  )
  .unwrap();
  for (option, covariance, group_count) in [
    ("robust", "robust_hc1", serde_json::Value::Null),
    ("cluster(group)", "cluster(group)", json!(12)),
  ] {
    fs::write(
      fixture.0.join("robust-ar-ci.td"),
      format!(
        "use data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3) {option}\nestat ar, ci\nreplace y = 999\ndrop z1\nestat ar, ci\n"
      ),
    )
    .unwrap();
    let rows = json_lines(&fixture.run(&["--json", "-f", "robust-ar-ci.td"]));
    assert_eq!(
      rows[1]["data"]["first_stage"]["scalar_ar_confidence_set_available"],
      true
    );
    assert_eq!(
      rows[2]["result_type"],
      "RobustMultipleInstrumentAndersonRubinConfidenceSet"
    );
    assert_eq!(rows[2]["data"], rows[5]["data"]);
    let data = &rows[2]["data"];
    assert_eq!(
      data["test"],
      "anderson_rubin_multiple_instrument_robust_confidence_set"
    );
    assert_eq!(data["endogenous"], "x");
    assert_eq!(data["excluded_instruments"], json!(["z1", "z2", "z3"]));
    assert_eq!(data["covariance"], covariance);
    assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
    assert_eq!(data["degrees_of_freedom"], 3);
    assert_eq!(data["observation_count"], 36);
    assert_eq!(data["auxiliary_parameter_count"], 5);
    assert_eq!(data["auxiliary_residual_df"], 31);
    assert_eq!(data["cluster_group_count"], group_count);
    assert_eq!(data["weak_iv_robust"], true);
    close(&data["critical_chi_square"], 7.814_727_903_251_179);
    let text = fixture.run(&["--text", "-f", "robust-ar-ci.td"]);
    assert!(
      text.status.success(),
      "{}",
      String::from_utf8_lossy(&text.stderr)
    );
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("asymptotic chi-square(3)"));
    assert!(text.contains("two through twenty-one excluded instruments"));
  }
}

#[test]
fn four_instrument_robust_ar_confidence_sets_report_json_text_and_twenty_two_dimension_limit() {
  let fixture = Fixture::new();
  for name in ["multiple", "four_roots", "all_real"] {
    fs::write(
      fixture.0.join(format!("robust_ar_four_ci_{name}_data.csv")),
      match name {
        "multiple" => {
          include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_multiple_data.csv")
        }
        "four_roots" => include_str!(
          "../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_four_roots_data.csv"
        ),
        "all_real" => {
          include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_all_real_data.csv")
        }
        _ => unreachable!(),
      },
    )
    .unwrap();
  }
  fs::write(
    fixture.0.join("robust_ar_five_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_five_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("robust_ar_five_ci_empty_cluster_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_five_ci_empty_cluster_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("robust_ar_six_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_six_ci_multiple_data.csv"),
  )
  .unwrap();

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/four_instrument_ci.td");
  let rows = json_lines(&fixture.run(&["--json", "-f", script.to_str().unwrap()]));
  assert_eq!(rows.len(), 3);
  assert_eq!(
    rows[1]["data"]["first_stage"]["scalar_ar_confidence_set_available"],
    true
  );
  assert_eq!(
    rows[2]["result_type"],
    "RobustMultipleInstrumentAndersonRubinConfidenceSet"
  );
  let data = &rows[2]["data"];
  assert_eq!(
    data["excluded_instruments"],
    json!(["z1", "z2", "z3", "z4"])
  );
  assert_eq!(data["degrees_of_freedom"], 4);
  assert_eq!(data["auxiliary_parameter_count"], 6);
  assert_eq!(data["auxiliary_residual_df"], 42);
  assert_eq!(data["shape"], "bounded");
  close(&data["critical_chi_square"], 9.48772903678115);
  close(&data["intervals"][0]["lower"], 0.838181194785723);
  close(&data["intervals"][0]["upper"], 1.84531003737431);

  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar/four_instrument_ci_clustered.td");
  let rows = json_lines(&fixture.run(&["--json", "-f", script.to_str().unwrap()]));
  let clustered = &rows[2]["data"];
  assert_eq!(clustered["covariance"], "cluster(group)");
  assert_eq!(clustered["cluster_group_count"], 12);
  assert_eq!(clustered["shape"], "disjoint");
  assert_eq!(clustered["intervals"].as_array().unwrap().len(), 2);
  close(&clustered["covariance_correction"], 1.22077922077922);
  close(&clustered["intervals"][0]["lower"], -4.08201124973978);
  close(&clustered["intervals"][0]["upper"], 0.25737090071055);
  close(&clustered["intervals"][1]["lower"], 1.34492724853664);
  close(&clustered["intervals"][1]["upper"], 2.72206249093528);

  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar/four_instrument_ci_all_real.td");
  let rows = json_lines(&fixture.run(&["--json", "-f", script.to_str().unwrap()]));
  assert_eq!(rows[2]["data"]["shape"], "empty");
  assert_eq!(rows[2]["data"]["intervals"], json!([]));

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/five_instrument_ci.td");
  let rows = json_lines(&fixture.run(&["--json", "-f", script.to_str().unwrap()]));
  let five = &rows[2]["data"];
  assert_eq!(
    five["excluded_instruments"],
    json!(["z1", "z2", "z3", "z4", "z5"])
  );
  assert_eq!(five["degrees_of_freedom"], 5);
  assert_eq!(five["observation_count"], 100);
  assert_eq!(five["auxiliary_parameter_count"], 7);
  assert_eq!(five["auxiliary_residual_df"], 93);
  assert_eq!(five["shape"], "bounded");
  close(&five["critical_chi_square"], 11.0704976935164);
  close(&five["intervals"][0]["lower"], 1.06421221080996);
  close(&five["intervals"][0]["upper"], 1.39773070439002);

  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar/five_instrument_ci_clustered.td");
  let rows = json_lines(&fixture.run(&["--json", "-f", script.to_str().unwrap()]));
  let clustered = &rows[2]["data"];
  assert_eq!(clustered["covariance"], "cluster(group)");
  assert_eq!(clustered["cluster_group_count"], 20);
  assert_eq!(clustered["degrees_of_freedom"], 5);
  assert_eq!(clustered["shape"], "bounded");
  close(&clustered["covariance_correction"], 1.12054329371817);
  close(&clustered["intervals"][0]["lower"], 1.16236801651081);
  close(&clustered["intervals"][0]["upper"], 1.39363336463301);

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/four_instrument_ci.td");
  let text = fixture.run(&["--text", "-f", script.to_str().unwrap()]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(4)"));
  assert!(text.contains("two through twenty-one excluded instruments"));

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/five_instrument_ci.td");
  let text = fixture.run(&["--text", "-f", script.to_str().unwrap()]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(5)"));
  assert!(text.contains("two through twenty-one excluded instruments"));

  fs::write(
    fixture.0.join("robust_ar_ten_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_ten_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("robust_ar_fourteen_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_fourteen_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("robust_ar_twenty_one_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_twenty_one_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("twenty-two-instruments.td"),
    "use robust_ar_twenty_one_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 z14 z15 z16 z17 z18 z19 z20 z21 row) robust\nestat ar, ci\nexport should-not-exist.csv\n",
  )
  .unwrap();
  let output = fixture.run(&["--json", "-f", "twenty-two-instruments.td"]);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr).contains("at most twenty-one excluded instruments")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
