use serde_json::Value;
use std::io::Write as IoWrite;
use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-logit-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = String::from("source_row,y,x1,x2\n");
    for i in 1..=40 {
      let x1 = ((i * 7) % 17) as f64 / 5.0 - 1.6;
      let x2 = ((i * 11) % 19) as f64 / 7.0 - 9.0 / 7.0;
      let y = u8::from((((i * 13) % 37) as f64) < 18.0 + 4.0 * x1 - 3.0 * x2);
      writeln!(csv, "{i},{y},{x1},{x2}").unwrap();
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
    for c in commands {
      args.extend(["-c", c]);
    }
    self.run(&args)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect()
}
fn close(v: &Value, expected: f64) {
  let actual = v.as_f64().unwrap();
  assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn script_json_exposes_explicit_logit_inference_predictions_and_retained_margins() {
  let f = Fixture::new();
  fs::write(f.0.join("analysis.td"),"use data.csv\nlogit y x1 x2\nstatUS\nestat margins\npredict p, pr\nreplace x1 = x1 + 10\nestat margins\nexport output.csv\n").unwrap();
  let rows = envelopes(&f.run(&["--json", "-f", "analysis.td"]));
  assert_eq!(
    rows
      .iter()
      .map(|r| r["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "BinaryLogitResult",
      "StatusResult",
      "LogitMarginalEffectsResult",
      "PredictionResult",
      "ReplaceResult",
      "LogitMarginalEffectsResult",
      "ExportResult"
    ]
  );
  assert!(rows.iter().all(|r| r["schema_version"] == 1));
  let fit = &rows[1]["data"];
  assert_eq!(fit["covariance"], "model_based_fisher_information");
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(
    fit["likelihood_ratio_reference_distribution"],
    "asymptotic_chi_square"
  );
  assert_eq!(fit["convergence_coordinate_system"], "unit_column_norm");
  assert_eq!(fit["converged"], true);
  assert_eq!(fit["confidence_level"], 95.0);
  assert_eq!(fit["observation_count"], 40);
  assert_eq!(fit["event_count"], 22);
  assert_eq!(
    fit["sample"]["retained_indices"],
    serde_json::json!((0..40).collect::<Vec<_>>())
  );
  close(&fit["coefficients"][1]["value"], 0.6009973792306854);
  close(&fit["coefficients"][1]["p_value"], 0.11031090942408736);
  close(&fit["covariance_matrix"][1][2], 0.0386698118821906);
  close(&fit["log_likelihood"], -24.51495924658521);
  assert!(fit.get("fitted_weights").is_none());
  assert!(fit.get("fitted_probabilities").is_none());
  assert_eq!(rows[2]["data"]["estimation_family"], "logit");
  let ame = &rows[3]["data"];
  assert_eq!(
    ame["effect"],
    "average_continuous_predictor_column_derivative"
  );
  assert_eq!(
    ame["uncertainty"],
    "conditional_on_observed_design_delta_method"
  );
  assert_eq!(ame["sample"], "retained_estimation_design");
  close(&ame["effects"][0]["value"], 0.12738548021377377);
  close(&ame["effects"][0]["standard_error"], 0.06969557366837566);
  assert_eq!(rows[3], rows[6]);
  let csv = fs::read_to_string(f.0.join("output.csv")).unwrap();
  assert_eq!(csv.lines().count(), 41);
  assert!(csv.lines().next().unwrap().ends_with(",p"));
}

#[test]
fn readable_and_piped_sessions_distinguish_z_inference_from_linear_t_and_discrete_changes() {
  let f = Fixture::new();
  let output = f.batch(
    "--text",
    &[
      "use data.csv",
      "logit y x1",
      "estat margins",
      "predict p, pr",
    ],
  );
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  for expected in [
    "Binary logit maximum likelihood",
    "inverse Fisher",
    "asymptotic standard-normal",
    "continuous average partial effects",
    "not discrete changes",
    "conditional on observed estimation X",
    "CI lower (95%)",
    "Predicted",
  ] {
    assert!(text.contains(expected), "missing {expected}: {text}");
  }
  assert!(!text.contains("residual df"));
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&f.0)
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(b"use data.csv\nlogit y x1\nestat margins\npredict p, pr\nexit\n")
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Binary logit maximum likelihood"));
  assert!(text.contains("continuous average partial effects"));
  assert!(text.contains("Predicted"));
}

#[test]
fn unsupported_covariance_and_separation_stop_scripts_before_file_side_effects() {
  let f = Fixture::new();
  for (name, fit) in [
    ("robust", "logit y x1, robust"),
    ("cluster", "logit y x1, cluster(source_row)"),
    (
      "separated",
      "sql \"\"\"SELECT source_row,x1,CASE WHEN x1 > 0 THEN 1 ELSE 0 END AS y FROM active\"\"\" into separated\nlogit y x1",
    ),
  ] {
    fs::write(
      f.0.join("failure.td"),
      format!("use data.csv\n{fit}\nexport {name}.csv\n"),
    )
    .unwrap();
    let output = f.run(&["--json", "-f", "failure.td"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(!f.0.join(format!("{name}.csv")).exists());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
      text
        .lines()
        .all(|l| serde_json::from_str::<Value>(l).unwrap()["result_type"] != "BinaryLogitResult")
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("logit"));
  }
}

#[test]
fn batch_json_noconstant_and_missing_model_errors_are_not_preflight_rejections() {
  let f = Fixture::new();
  let rows = envelopes(&f.batch(
    "--json",
    &["use data.csv", "logit y x1, noconstant", "estat margins"],
  ));
  assert_eq!(rows[1]["data"]["include_intercept"], false);
  close(
    &rows[1]["data"]["coefficients"][0]["value"],
    0.728903097698189,
  );
  close(&rows[1]["data"]["null_log_likelihood"], -27.725887222397784);
  close(&rows[2]["data"]["effects"][0]["value"], 0.1627034993552953);
  let failure = f.batch("--json", &["use data.csv", "estat margins"]);
  assert_eq!(failure.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&failure.stderr).contains("estat margins requires a prior logit fit")
  );
  assert!(!String::from_utf8_lossy(&failure.stderr).contains("JsonResultUnsupported"));
}
