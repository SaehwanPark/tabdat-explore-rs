mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, process::Output};

const INPUT: &str = include_str!("fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("fixtures/gompertz/parameter_names.txt");
const POINTS: &str = include_str!("fixtures/gompertz/predictions.csv");
fn setup() -> TestDir {
  let d = TestDir::new();
  fs::write(d.path().join("input.csv"), INPUT).unwrap();
  d
}
fn script(robust: bool, no_intercept: bool, change: &str) -> String {
  let names = NAMES
    .lines()
    .skip(1)
    .filter(|n| *n != "gamma")
    .collect::<Vec<_>>()
    .join(" ");
  format!(
    "use input.csv\ngenerate rr_ui = reprate * ui\ngenerate dr_ui = disrate * ui\n{change}\nstreg spell {names}, failure(censor1) dist(gompertz) {} {}\ndrop censor1\npredict prediction_index\npredict prediction_hazard, hazard\npredict prediction_cumulative, cumhaz\npredict prediction_survival, survival\nexport points.csv\n",
    if robust { "robust" } else { "" },
    if no_intercept { "noconstant" } else { "" }
  )
}
fn run(d: &TestDir, mode: &str) -> Output {
  tabdat_bin()
    .current_dir(d.path())
    .args([mode, "-f", "fit.td"])
    .output()
    .unwrap()
}
fn success(out: &Output) {
  assert!(
    out.status.success(),
    "stdout={}\nstderr={}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
}
fn fit(out: &Output) -> Value {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .find(|e| e["result_type"] == "GompertzResult")
    .unwrap()["data"]
    .clone()
}
fn close(actual: f64, expected: f64, label: &str) {
  let tolerance = if label.ends_with("p_value") && expected > 0.0 {
    1e-300 + 1e-8 * expected.abs()
  } else {
    1e-9 + 1e-8 * expected.abs()
  };
  assert!(
    (actual - expected).abs() <= tolerance,
    "{label}: {actual:.17e} != {expected:.17e}"
  );
}

#[test]
fn gompertz_cli_full_joint_inference_metadata_and_current_points_match_r_in_both_modes() {
  for (robust, no_intercept, reference) in [
    (
      false,
      false,
      include_str!("fixtures/gompertz/model_based.csv"),
    ),
    (true, false, include_str!("fixtures/gompertz/robust.csv")),
    (
      false,
      true,
      include_str!("fixtures/gompertz/no_intercept/model_based.csv"),
    ),
    (
      true,
      true,
      include_str!("fixtures/gompertz/no_intercept/robust.csv"),
    ),
  ] {
    let d = setup();
    fs::write(d.path().join("fit.td"), script(robust, no_intercept, "")).unwrap();
    let json = run(&d, "--json");
    success(&json);
    let data = fit(&json);
    let mut gates = 0;
    for line in reference.lines().skip(1) {
      let (path, expected) = line.split_once(',').unwrap();
      close(
        data
          .pointer(path)
          .unwrap_or_else(|| panic!("missing {path}"))
          .as_f64()
          .unwrap(),
        expected.parse().unwrap(),
        path,
      );
      gates += 1;
    }
    let k = if no_intercept { 40 } else { 41 };
    assert_eq!(
      gates,
      6 * k + (k + 1) * (k + 1) + 6 + if robust { 13 } else { 14 }
    );
    assert_eq!(data["estimator"], "streg_gompertz");
    assert_eq!(
      data["covariance_parameter_order"],
      "ph_coefficients_then_natural_gamma"
    );
    assert_eq!(
      data["covariance_parameter_names"]
        .as_array()
        .unwrap()
        .last()
        .unwrap(),
      "gamma"
    );
    assert_eq!(
      data["covariance_parameter_names"].as_array().unwrap().len(),
      k + 1
    );
    assert_eq!(data["reference_distribution"], "asymptotic_standard_normal");
    assert_eq!(
      data["negative_gamma_implies_positive_survival_mass_at_infinity"],
      false
    );
    assert!(data["shape"].get("log_value").is_none());
    assert_eq!(
      data["likelihood_ratio_reference_distribution"].is_null(),
      robust
    );
    assert_eq!(
      data["sample"]["retained_indices"].as_array().unwrap(),
      &(0..400).map(Value::from).collect::<Vec<_>>()
    );
    for name in [
      "fitted_xb",
      "fitted_hazard",
      "fitted_cumulative_hazard",
      "fitted_survival",
      "predictor_names",
    ] {
      assert!(data.get(name).is_none());
    }
    let bytes = fs::read(d.path().join("points.csv")).unwrap();
    if !no_intercept {
      let csv = String::from_utf8(bytes.clone()).unwrap();
      let mut lines = csv.lines();
      let headers = lines.next().unwrap().split(',').collect::<Vec<_>>();
      let mut gold = POINTS.lines();
      let ref_headers = gold.next().unwrap().split(',').collect::<Vec<_>>();
      let mut count = 0;
      for (line, expected) in lines.zip(gold) {
        let values = line
          .split(',')
          .map(|v| v.parse::<f64>().unwrap())
          .collect::<Vec<_>>();
        let expected = expected
          .split(',')
          .map(|v| v.parse::<f64>().unwrap())
          .collect::<Vec<_>>();
        for (actual, reference) in [
          ("prediction_index", "xb"),
          ("prediction_hazard", "hazard"),
          ("prediction_cumulative", "cumulative_hazard"),
          ("prediction_survival", "net_survival"),
        ] {
          close(
            values[headers.iter().position(|h| *h == actual).unwrap()],
            expected[ref_headers.iter().position(|h| *h == reference).unwrap()],
            actual,
          );
        }
        count += 1;
      }
      assert_eq!(count, 400);
    }
    fs::remove_file(d.path().join("points.csv")).unwrap();
    let text = run(&d, "--text");
    success(&text);
    let report = String::from_utf8_lossy(&text.stdout);
    for expected in [
      "Gompertz proportional-hazards maximum likelihood",
      "natural-gamma covariance",
      "z(gamma=0)",
      "natural normal95 interval",
      "not identified causal cure",
      "NOT all-cause joblessness survival",
    ] {
      assert!(report.contains(expected), "missing {expected}");
    }
    assert_eq!(report.contains("robust LR reference withheld"), robust);
    assert_eq!(fs::read(d.path().join("points.csv")).unwrap(), bytes);
    assert_eq!(
      fs::read_to_string(d.path().join("input.csv")).unwrap(),
      INPUT
    );
  }
}

#[test]
fn gompertz_cli_negative_gamma_is_reported_naturally_and_help_bounds_are_discoverable() {
  let d = setup();
  fs::write(
    d.path().join("fit.td"),
    script(
      true,
      false,
      "replace spell = 0.1 + spell/100 if censor1 == 1",
    ),
  )
  .unwrap();
  let output = run(&d, "--json");
  success(&output);
  let data = fit(&output);
  assert!(data["shape"]["value"].as_f64().unwrap() < 0.0);
  assert_eq!(
    data["negative_gamma_implies_positive_survival_mass_at_infinity"],
    true
  );
  assert_eq!(
    data["shape_interval"],
    "natural_gamma_normal_wald_95_zero_tests_exponential_within_family"
  );
  let mut text = String::new();
  for topic in ["streg", "predict"] {
    let help = tabdat_bin()
      .args(["--json", "--help-topic", topic])
      .output()
      .unwrap();
    success(&help);
    let envelope: Value = serde_json::from_slice(&help.stdout).unwrap();
    assert_eq!(envelope["result_type"], "HelpTopicResult");
    text.push_str(envelope["data"]["text"].as_str().unwrap());
  }
  for expected in [
    "dist(gompertz)",
    "Gamma<0",
    "H0/S1",
    "at least one failure",
    "no point",
  ] {
    assert!(text.contains(expected), "missing {expected}");
  }
  let schema = tabdat_bin()
    .args(["--json", "--describe-command", "streg"])
    .output()
    .unwrap();
  success(&schema);
  let envelope: Value = serde_json::from_slice(&schema.stdout).unwrap();
  let args = envelope["data"]["arguments"].as_array().unwrap();
  assert!(
    args.iter().find(|a| a["name"] == "variables").unwrap()["required"]
      .as_bool()
      .unwrap()
  );
}

#[test]
fn gompertz_cli_exact_decimal_failures_stop_before_predictions_without_touching_input() {
  for value in ["1.000000000000000001", "0.999999999999999999"] {
    for mode in ["--json", "--text"] {
      let d = setup();
      let change = format!(
        "sql \"\"\"SELECT * EXCLUDE(censor1),CASE WHEN censor1=1 THEN CAST('{value}' AS DECIMAL(38,18)) ELSE CAST(censor1 AS DECIMAL(38,18)) END AS censor1 FROM active\"\"\" into decimal_events"
      );
      fs::write(d.path().join("fit.td"), script(true, false, &change)).unwrap();
      let out = run(&d, mode);
      assert_eq!(out.status.code(), Some(1));
      assert!(!String::from_utf8_lossy(&out.stdout).contains("\"result_type\":\"GompertzResult\""));
      assert!(!d.path().join("points.csv").exists());
      assert_eq!(
        fs::read_to_string(d.path().join("input.csv")).unwrap(),
        INPUT
      );
    }
  }
}
