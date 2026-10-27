use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-estimates-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(
      path.join("data.csv"),
      "y,x,z\n0,-2,0\n1,-1,1\n0,0,0\n1,1,1\n0,2,0\n1,3,1\n1,0.5,0\n0,1.5,1\n0,-0.5,1\n1,2.5,0\n",
    )
    .unwrap();
    fs::write(path.join("report.td"),"use data.csv\nregress y x, robust\nestimates store linear\nlogit y x\nestimates store binary\nprobit y x, robust\nestimates store probit\nestimates table probit linear binary\nestat margins\npredict prob, pr\nestimates drop linear\nestimates table binary probit\nstatus\nexport result.csv\n").unwrap();
    Self(path)
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
fn envelopes(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}

#[test]
fn json_script_preserves_requested_columns_inference_covariance_and_active_probit() {
  let fixture = Fixture::new();
  let output = fixture.run(&["--json", "-f", "report.td"]);
  assert!(output.status.success(), "{:?}", output);
  let rows = envelopes(&output.stdout);
  assert!(rows.iter().all(|r| r["schema_version"] == 1));
  let tables: Vec<_> = rows
    .iter()
    .filter(|r| r["result_type"] == "EstimateTableResult")
    .collect();
  assert_eq!(tables.len(), 2);
  let models = tables[0]["data"]["models"].as_array().unwrap();
  assert_eq!(
    models
      .iter()
      .map(|m| m["name"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["probit", "linear", "binary"]
  );
  assert_eq!(
    tables[0]["data"]["parameter_names"],
    serde_json::json!(["intercept", "x"])
  );
  for model in models {
    assert_eq!(model["observation_count"], 10);
    assert_eq!(
      model["sample"]["retained_indices"],
      serde_json::json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
    );
    assert_eq!(
      model["covariance_parameter_names"],
      serde_json::json!(["intercept", "x"])
    );
    let fit_type = match model["estimator"].as_str().unwrap() {
      "ols" => "RegressionResult",
      "logit" => "BinaryLogitResult",
      _ => "BinaryProbitResult",
    };
    let fit = &rows.iter().find(|r| r["result_type"] == fit_type).unwrap()["data"];
    assert_eq!(model["covariance_matrix"], fit["covariance_matrix"]);
    for (copy, original) in model["coefficients"]
      .as_array()
      .unwrap()
      .iter()
      .zip(fit["coefficients"].as_array().unwrap())
    {
      for key in ["name", "value", "standard_error", "statistic", "p_value"] {
        assert_eq!(copy[key], original[key]);
      }
    }
  }
  assert_eq!(models[1]["reference_distribution"], "student_t");
  assert_eq!(models[1]["inference_degrees_of_freedom"], 8);
  assert_eq!(
    models[0]["reference_distribution"],
    "asymptotic_standard_normal"
  );
  assert!(models[0]["inference_degrees_of_freedom"].is_null());
  assert!(
    rows
      .iter()
      .any(|r| r["result_type"] == "ProbitMarginalEffectsResult")
  );
  assert_eq!(
    rows
      .iter()
      .find(|r| r["result_type"] == "StatusResult")
      .unwrap()["data"]["estimation_family"],
    "probit"
  );
  assert_eq!(tables[1]["data"]["models"][1], models[0]);
  assert!(fixture.0.join("result.csv").exists());
}

#[test]
fn text_and_piped_scripts_show_native_table_conventions_and_same_predictions() {
  let text = Fixture::new();
  let output = text.run(&["--text", "-f", "report.td"]);
  assert!(output.status.success(), "{:?}", output);
  let rendered = String::from_utf8(output.stdout).unwrap();
  for phrase in [
    "Estimate\tprobit\tlinear\tbinary",
    "student_t(8)",
    "asymptotic_standard_normal",
    "x coefficient\t",
    "OLS R-squared",
    "Binary McFadden",
    "no refits",
    "not a cross-family",
    "Reports cannot restore fits",
  ] {
    assert!(rendered.contains(phrase), "{phrase}");
  }
  let piped = Fixture::new();
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&piped.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(b"run report.td\nexit\n")
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success(), "{:?}", output);
  assert!(String::from_utf8_lossy(&output.stdout).contains("Estimate\tprobit\tlinear\tbinary"));
  assert_eq!(
    fs::read(text.0.join("result.csv")).unwrap(),
    fs::read(piped.0.join("result.csv")).unwrap()
  );
}

#[test]
fn absent_terms_and_undefined_zero_variance_inference_are_na_not_zero_or_new_tests() {
  let fixture = Fixture::new();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "regress y x, noconstant",
    "-c",
    "estimates store one",
    "-c",
    "generate zero = 0",
    "-c",
    "regress zero z",
    "-c",
    "estimates store zero",
    "-c",
    "estimates table zero one",
  ]);
  assert!(output.status.success(), "{:?}", output);
  let rows = envelopes(&output.stdout);
  let table = &rows.last().unwrap()["data"];
  assert_eq!(
    table["parameter_names"],
    serde_json::json!(["intercept", "z", "x"])
  );
  let model = &table["models"][0];
  assert!(model["log_likelihood"].is_null());
  assert!(model["log_likelihood_convention"].is_null());
  for c in model["coefficients"].as_array().unwrap() {
    assert!(c["statistic"].is_null());
    assert!(c["p_value"].is_null());
  }
  let output = fixture.run(&[
    "--text",
    "-c",
    "use data.csv",
    "-c",
    "regress y x, noconstant",
    "-c",
    "estimates store one",
    "-c",
    "regress x z",
    "-c",
    "estimates store other",
    "-c",
    "estimates table other one",
  ]);
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("intercept coefficient\t"));
  assert!(text.contains("\tNA\n"));
  assert!(text.contains("Samples/outcomes can differ"));
}

#[test]
fn discovery_advertises_exact_actions_and_located_registry_errors_stop_scripts() {
  let fixture = Fixture::new();
  let output = fixture.run(&["--json", "--describe-command", "estimates"]);
  assert!(output.status.success());
  let schema: Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(
    schema["data"]["syntax"],
    "estimates store NAME | estimates table NAME [NAME ...] | estimates drop NAME [NAME ...]"
  );
  assert_eq!(schema["data"]["options"], serde_json::json!([]));
  let output = fixture.run(&["--json", "--help-topic", "estimates"]);
  assert!(output.status.success());
  let help: Value = serde_json::from_slice(&output.stdout).unwrap();
  assert!(
    help["data"]["text"]
      .as_str()
      .unwrap()
      .contains("not restorable model caches")
  );
  fs::write(
    fixture.0.join("bad.td"),
    "use data.csv\nregress y x\nestimates store a\nestimates store a\nexport after.csv\n",
  )
  .unwrap();
  let output = fixture.run(&["--json", "-f", "bad.td"]);
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).contains("bad.td:4:"));
  assert!(!fixture.0.join("after.csv").exists());
  let records = envelopes(&output.stdout);
  let error = &records.last().unwrap()["error"];
  assert_eq!(error["type"], "ScriptError");
  assert!(
    error["message"]
      .as_str()
      .unwrap()
      .contains("already stored")
  );
}

#[test]
fn nls_named_report_has_formula_covariance_and_visible_standard_errors() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("nls_report.td"), "use data.csv\nnl y = a+b*x, params(a b) start(1 0) robust\nestimates store nlsfit\nregress y x\nestimates store olsfit\nestimates table nlsfit olsfit\n").unwrap();
  let output = fixture.run(&["--json", "-f", "nls_report.td"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rows = envelopes(&output.stdout);
  let table = rows
    .iter()
    .find(|v| v["result_type"] == "EstimateTableResult")
    .unwrap();
  let models = table["data"]["models"].as_array().unwrap();
  assert_eq!(models.len(), 2);
  assert_eq!(models[0]["estimator"], "nls");
  assert_eq!(
    models[0]["nls_model"]["parameter_names"],
    serde_json::json!(["a", "b"])
  );
  assert_eq!(
    models[0]["nls_model"]["predictor_names"],
    serde_json::json!(["x"])
  );
  assert_eq!(models[0]["reference_distribution"], "approximate_student_t");
  assert_eq!(models[0]["covariance"], "hc1_gauss_newton");
  assert!(models[0]["log_likelihood"].is_null());
  assert!(models[1].get("nls_model").is_none());
  assert!(models[1].get("nls_centered_r_squared").is_none());
  assert!(models[1].get("nls_covariance_correction").is_none());
  assert_eq!(models[0]["nls_covariance_correction"], 1.25);
  let output = fixture.run(&["--text", "-f", "nls_report.td"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("NLS mean\t(a+(b*x))\tNA"), "{text}");
  assert!(text.contains("a standard error\t"), "{text}");
  assert!(text.contains("NLS centered R-squared\t"), "{text}");
  assert!(
    text.contains("NLS covariance correction\t1.250000\tNA"),
    "{text}"
  );
  assert!(text.contains("approximate_student_t(8)"), "{text}");
}

#[test]
fn poisson_named_reports_show_offset_correction_and_preserve_active_fit() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("poisson_report.td"), "use data.csv\npoisson y x, offset(z)\nestimates store model\npoisson y x, offset(z) robust\nestimates store hc\npoisson y x, offset(z) cluster(z)\nestimates store clustered\nestimates table model hc clustered\nestat margins\npredict predicted, n\n").unwrap();
  let output = fixture.run(&["--json", "-f", "poisson_report.td"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rows = envelopes(&output.stdout);
  let table = rows
    .iter()
    .find(|v| v["result_type"] == "EstimateTableResult")
    .unwrap();
  let models = table["data"]["models"].as_array().unwrap();
  assert_eq!(models.len(), 3);
  for model in models {
    assert_eq!(model["estimator"], "poisson");
    assert_eq!(model["poisson_offset_variable"], "z");
    assert_eq!(model["log_likelihood_convention"], "poisson_count_pmf");
    assert_eq!(
      model["reference_distribution"],
      "asymptotic_standard_normal"
    );
    assert!(model["mcfadden_r_squared"].is_null());
    assert!(model["poisson_mcfadden_r_squared"].is_number());
  }
  assert_eq!(models[0]["poisson_covariance_correction"], 1.0);
  assert_eq!(models[1]["poisson_covariance_correction"], 10.0 / 9.0);
  assert_eq!(models[2]["poisson_covariance_correction"], 2.0);
  assert_eq!(models[2]["cluster_variable"], "z");
  assert_eq!(models[2]["cluster_count"], 2);
  assert!(
    rows
      .iter()
      .any(|r| r["result_type"] == "PoissonMarginalEffectsResult")
  );
  assert!(rows.iter().any(|r| r["result_type"] == "PredictionResult"));
  let output = fixture.run(&["--text", "-f", "poisson_report.td"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(
    text.contains("Poisson offset (coefficient 1)\tz\tz\tz"),
    "{text}"
  );
  assert!(
    text.contains("Poisson covariance correction\t1.000000\t1.111111\t2.000000"),
    "{text}"
  );
  assert!(text.contains("Poisson McFadden pseudo-R-squared"), "{text}");
}
