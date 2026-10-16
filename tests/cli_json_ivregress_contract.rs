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
  let mut csv = String::from("y,w,x1,z1,z2\n");
  for i in 1..=40 {
    let basis = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let w = basis(5.0);
    let z1 = basis(1.0);
    let z2 = basis(2.0);
    let x1 = strength * z1 + basis(3.0);
    let y = 1.5 * x1 + 0.5 * w + direct_z1 * z1 + direct_z2 * z2 + basis(4.0);
    writeln!(csv, "{y},{w},{x1},{z1},{z2}").unwrap();
  }
  fs::write(fixture.0.join("data.csv"), csv).unwrap();
  fixture
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
    false
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
    let null = if option.is_empty() { "0" } else { "0 0" };
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
