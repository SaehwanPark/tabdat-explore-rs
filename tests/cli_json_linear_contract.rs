#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-linear-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y\n0,1\n1,2\n2,1.5\n3,4\n4,3\n5,5\n6,\n,8\n",
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn close(actual: &Value, expected: &Value) {
  let actual = actual.as_f64().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected} (tolerance {tolerance})"
  );
}

#[test]
fn ols_working_likelihood_and_selected_covariance_have_explicit_reporting_contracts() {
  let f = Fixture::new();
  // Base R 4.6.1 lm/logLik/vcov; robust covariance is HC1 with N/(N-K).
  for (fit, covariance) in [
    (
      "regress y x",
      [
        [0.338605442176871, -0.09234693877551027],
        [-0.09234693877551027, 0.0369387755102041],
      ],
    ),
    (
      "regress y x, robust",
      [
        [0.11748299319727887, -0.022979591836734696],
        [-0.022979591836734696, 0.016824489795918364],
      ],
    ),
  ] {
    let output = f.run(&["--json", "-c", "use data.csv", "-c", fit]);
    assert!(output.status.success());
    let rows = lines(&output.stdout);
    let data = &rows[1]["data"];
    assert!((data["log_likelihood"].as_f64().unwrap() + 5.988358159193272).abs() < 1e-10);
    assert_eq!(
      data["log_likelihood_convention"],
      "gaussian_working_model_ml_variance_rss_over_n"
    );
    assert_eq!(
      data["covariance_parameter_names"],
      json!(["intercept", "x"])
    );
    for (i, row) in covariance.iter().enumerate() {
      for (j, expected) in row.iter().enumerate() {
        assert!((data["covariance_matrix"][i][j].as_f64().unwrap() - expected).abs() < 1e-10);
      }
    }
    let text = f.run(&["--text", "-c", "use data.csv", "-c", fit]);
    assert!(text.status.success());
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(text.contains("ML variance RSS/N"));
    assert!(text.contains("not Bernoulli likelihood or robust/cross-family LR inference"));
  }
  for fit in [
    "regress y x, wls(w)",
    "regress y x, gls(w)",
    "ivregress 2sls y, endog(x) iv(z)",
  ] {
    let output = f.run(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "generate w = 1",
      "-c",
      "generate z = x",
      "-c",
      fit,
    ]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output.stdout);
    assert_eq!(rows[3]["data"]["log_likelihood"], Value::Null);
    assert_eq!(rows[3]["data"]["log_likelihood_convention"], Value::Null);
  }
  fs::write(f.0.join("zero.csv"), "x,y\n0,0\n1,0\n2,0\n").unwrap();
  let output = f.run(&["--json", "-c", "use zero.csv", "-c", "regress y x"]);
  assert!(output.status.success());
  let rows = lines(&output.stdout);
  assert_eq!(rows[1]["data"]["log_likelihood"], Value::Null);
  assert_eq!(rows[1]["data"]["log_likelihood_convention"], Value::Null);
}

#[test]
fn json_linear_workflow_matches_independent_reference_without_full_model_arrays() {
  let fixture = Fixture::new();
  // statsmodels/SciPy provenance and regeneration tool are recorded in the shared fixture.
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/linear-terminal-reference.json")).unwrap();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "regress y x",
    "-c",
    "lincom intercept + x",
    "-c",
    "test x",
    "-c",
    "predict fitted",
    "-c",
    "head 8",
  ]);
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let envelopes = lines(&output.stdout);
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  fs::write(
    fixture.0.join("child.td"),
    "regress y x\nrun grandchild.td\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("grandchild.td"),
    "lincom intercept + x\ntest x\npredict fitted\nhead 8\n",
  )
  .unwrap();
  let nested = fixture.run(&["--json", "-f", "parent.td"]);
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  let mut streamed = lines(&nested.stdout);
  assert_eq!(streamed.len(), envelopes.len());
  // Scripts resolve file paths relative to their source; all statistical and
  // preview payloads must otherwise match batch execution exactly.
  for index in [0, 4] {
    streamed[index]["data"]["dataset"]["path"] =
      envelopes[index]["data"]["dataset"]["path"].clone();
  }
  assert_eq!(streamed, envelopes);
  assert_eq!(
    envelopes
      .iter()
      .map(|value| value["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "RegressionResult",
      "LincomResult",
      "TestResult",
      "PredictionResult",
      "PreviewResult"
    ]
  );
  assert!(envelopes.iter().all(|value| value["schema_version"] == 1));
  let model = &envelopes[1]["data"];
  assert_eq!(model["outcome"], "y");
  assert_eq!(model["predictors"], json!(["x"]));
  assert_eq!(model["estimator"], "ols");
  assert_eq!(model["covariance"], "nonrobust");
  assert_eq!(model["include_intercept"], true);
  assert_eq!(model["observation_count"], reference["n"]);
  assert_eq!(model["df_residual"], reference["df_residual"]);
  assert_eq!(
    model["sample"],
    json!({"total_observations": 8, "dropped_observations": 2})
  );
  for key in ["r_squared", "adjusted_r_squared", "root_mse"] {
    close(&model[key], &reference[key]);
  }
  assert_eq!(model["coefficients"].as_array().unwrap().len(), 2);
  for (index, coefficient) in model["coefficients"].as_array().unwrap().iter().enumerate() {
    assert_eq!(coefficient["name"], reference["coefficient_names"][index]);
    for (field, key) in [
      ("value", "coefficients"),
      ("standard_error", "standard_errors"),
      ("statistic", "statistics"),
      ("p_value", "p_values"),
    ] {
      close(&coefficient[field], &reference[key][index]);
    }
  }
  for field in [
    "least_squares",
    "fitted_values",
    "residuals",
    "retained_indices",
  ] {
    assert!(model.get(field).is_none());
  }
  let lincom = &envelopes[2]["data"];
  assert_eq!(lincom["label"], "(intercept + x)");
  assert_eq!(lincom["df_residual"], 4);
  assert_eq!(lincom["ci_level"], 95.0);
  for (index, key) in [
    "estimate",
    "standard_error",
    "statistic",
    "p_value",
    "ci_lower",
    "ci_upper",
  ]
  .iter()
  .enumerate()
  {
    close(&lincom[key], &reference["lincom"][index]);
  }
  let test = &envelopes[3]["data"];
  assert_eq!(test["df"], 1);
  assert_eq!(test["df_residual"], 4);
  assert_eq!(test["is_chi2"], false);
  assert_eq!(test["constraints"].as_array().unwrap().len(), 1);
  close(&test["statistic"], &reference["test"][0]);
  close(&test["p_value"], &reference["test"][1]);
  assert_eq!(envelopes[4]["data"]["dataset"]["row_count"], 8);
  assert_eq!(
    envelopes[4]["data"]["dataset"]["columns"][2]["name"],
    "fitted"
  );
  assert_eq!(envelopes[4]["data"]["dataset"]["path"], "data.csv");
  for (index, row) in envelopes[5]["data"]["rows"]
    .as_array()
    .unwrap()
    .iter()
    .enumerate()
  {
    if reference["predictions"][index].is_null() {
      assert!(row[2].is_null());
    } else {
      close(&row[2], &reference["predictions"][index]);
    }
  }
}

#[test]
fn nested_json_linear_errors_keep_prior_results_and_child_location() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("child.td"),
    "regress y x\nlincom x\ntest absent\npredict never\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let output = fixture.run(&["--json", "-f", "parent.td"]);
  assert_eq!(output.status.code(), Some(1));
  let envelopes = lines(&output.stdout);
  assert_eq!(envelopes.len(), 4);
  assert_eq!(envelopes[1]["result_type"], "RegressionResult");
  assert_eq!(envelopes[2]["result_type"], "LincomResult");
  assert_eq!(envelopes[3]["error"]["type"], "ScriptError");
  assert_eq!(envelopes[3]["error"]["line"], 3);
  assert_eq!(
    envelopes[3]["error"]["path"],
    fixture
      .0
      .join("child.td")
      .canonicalize()
      .unwrap()
      .to_string_lossy()
      .as_ref()
  );
}

#[test]
fn json_model_and_prediction_failures_never_emit_false_success() {
  let fixture = Fixture::new();
  for command in ["predict fitted", "lincom x", "test x", "regress y absent"] {
    let output = fixture.run(&["--json", "-c", "use data.csv", "-c", command]);
    assert_eq!(output.status.code(), Some(1));
    let envelopes = lines(&output.stdout);
    assert_eq!(envelopes.len(), 2);
    assert_eq!(envelopes[1]["error"]["type"], "ExecutionError");
  }
  for command in [
    "predict x",
    "predict probability, pr",
    "predict spillover, spatial_lag",
    "predict posterior, posterior_predictive std interval level(90)",
    "predict posterior, posterior_predictive saving(draws.parquet)",
  ] {
    let output = fixture.run(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "regress y x",
      "-c",
      command,
    ]);
    assert_eq!(output.status.code(), Some(1));
    let envelopes = lines(&output.stdout);
    assert_eq!(envelopes.len(), 3);
    assert_eq!(envelopes[2]["error"]["type"], "ExecutionError");
    assert!(!fixture.0.join("draws.parquet").exists());
  }
}

#[test]
fn linear_json_null_policy_preserves_zero_and_does_not_mutate_model_state() {
  let fixture = Fixture::new();
  let mut session = tabdat_execution::Session::new();
  session
    .execute(
      tabdat_language::parse_command(&format!("use {}", fixture.0.join("data.csv").display()))
        .unwrap(),
    )
    .unwrap();
  let mut result = session
    .execute(tabdat_language::parse_command("regress y x").unwrap())
    .unwrap();
  let state = session.last_regression().cloned();
  if let tabdat_domain::ExecutionResult::Regression(model) = &mut result {
    model.coefficients[0].value = f64::INFINITY;
    model.coefficients[0].standard_error = Some(f64::NAN);
    model.coefficients[0].statistic = None;
    model.coefficients[0].p_value = Some(0.0);
    model.root_mse = Some(f64::INFINITY);
    model.least_squares.sample = None;
  } else {
    panic!("expected regression result");
  }
  let envelope: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  assert!(envelope["data"]["sample"].is_null());
  assert!(envelope["data"]["root_mse"].is_null());
  let coefficient = &envelope["data"]["coefficients"][0];
  assert!(coefficient["value"].is_null());
  assert!(coefficient["standard_error"].is_null());
  assert!(coefficient["statistic"].is_null());
  assert_eq!(coefficient["p_value"], 0.0);
  assert_eq!(session.last_regression(), state.as_ref());
  let result = tabdat_domain::ExecutionResult::Lincom(tabdat_domain::LincomResult {
    label: "constant".to_owned(),
    estimate: 5.0,
    standard_error: 0.0,
    statistic: f64::NAN,
    p_value: f64::NAN,
    ci_lower: 5.0,
    ci_upper: 5.0,
    ci_level: 95.0,
    df_residual: Some(4),
  });
  let envelope: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  assert_eq!(envelope["data"]["standard_error"], 0.0);
  assert!(envelope["data"]["statistic"].is_null());
  assert!(envelope["data"]["p_value"].is_null());
}
