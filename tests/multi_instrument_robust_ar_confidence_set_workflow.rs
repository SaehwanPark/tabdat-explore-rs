#![forbid(unsafe_code)]

use serde_json::Value;
use std::{
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
      "tabdat-multi-ar-ci-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("robust_ar_multi_ci_data.csv"),
      include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_multi_ci_all_real_data.csv"),
      include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci_all_real_data.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn run(&self, mode: &str, script: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([mode, "-f", script])
      .output()
      .unwrap()
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn json_lines(output: &Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

fn close(actual: &Value, expected: f64, tolerance: f64) {
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual:.16e} != {expected:.16e}"
  );
}

#[test]
fn two_instrument_hc1_script_reports_multiple_components_and_reuses_retained_fit() {
  let fixture = Fixture::new();
  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/two_instrument_ci.td");
  let output = fixture.run("--json", script.to_str().unwrap());
  let rows = json_lines(&output);
  assert_eq!(rows.len(), 6);
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
  assert_eq!(data["covariance"], "robust_hc1");
  assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
  assert_eq!(data["weak_iv_robust"], true);
  assert_eq!(data["degrees_of_freedom"], 2);
  assert_eq!(data["observation_count"], 20);
  assert_eq!(data["auxiliary_parameter_count"], 4);
  assert_eq!(data["auxiliary_residual_df"], 16);
  assert_eq!(data["cluster_group_count"], Value::Null);
  assert_eq!(data["shape"], "multiple");
  assert_eq!(
    data["excluded_instruments"],
    serde_json::json!(["z1", "z2"])
  );
  assert_eq!(data["intervals"].as_array().unwrap().len(), 3);
  assert_eq!(data["intervals"][0]["lower"], Value::Null);
  assert_eq!(data["intervals"][2]["upper"], Value::Null);
  close(&data["covariance_correction"], 1.25, 1e-14);
  close(&data["critical_chi_square"], 5.99146454710798, 1e-14);
  close(&data["intervals"][0]["upper"], 0.0628690191316964, 2e-10);

  let output = fixture.run("--text", script.to_str().unwrap());
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  for expected in [
    "Robust Anderson-Rubin confidence set",
    "asymptotic chi-square(2)",
    "instruments = [\"z1\", \"z2\"]",
    "unrestricted nuisance parameters",
    "No finite-sample or few-cluster guarantee",
  ] {
    assert!(text.contains(expected), "missing {expected:?} in {text}");
  }
}

#[test]
fn all_real_two_instrument_cli_workflow_reports_the_unbounded_set() {
  let fixture = Fixture::new();
  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar/two_instrument_ci_all_real.td");
  let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
  assert_eq!(rows.len(), 3);
  let first_stage = &rows[1]["data"]["first_stage"]["rows"][0];
  assert!(
    first_stage["partial_r_squared"].as_f64().unwrap() > 0.0,
    "all-real fixture must retain nonzero first-stage relevance: {first_stage}"
  );
  assert!(
    first_stage["f_statistic"].as_f64().unwrap() > 0.0,
    "all-real fixture must retain a nonzero first-stage statistic: {first_stage}"
  );
  assert_eq!(
    rows[2]["result_type"],
    "RobustMultipleInstrumentAndersonRubinConfidenceSet"
  );
  let data = &rows[2]["data"];
  assert_eq!(data["shape"], "all_real");
  assert_eq!(
    data["intervals"],
    serde_json::json!([{"lower": null, "upper": null}])
  );
}

#[test]
fn two_instrument_cluster_script_reports_cr1_correction_and_unbounded_set() {
  let fixture = Fixture::new();
  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar/two_instrument_ci_clustered.td");
  let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
  assert_eq!(rows.len(), 6);
  let data = &rows[2]["data"];
  assert_eq!(data["covariance"], "cluster(group)");
  assert_eq!(data["cluster_group_count"], 10);
  assert_eq!(data["shape"], "disjoint");
  assert_eq!(data["intervals"].as_array().unwrap().len(), 2);
  assert_eq!(data["intervals"][0]["lower"], Value::Null);
  assert_eq!(data["intervals"][1]["upper"], Value::Null);
  close(&data["covariance_correction"], 1.31944444444444, 1e-13);
  close(&data["intervals"][0]["upper"], 0.154198964682854, 2e-10);
  close(&data["intervals"][1]["lower"], 0.573634185761531, 2e-10);
}

#[test]
fn more_than_four_robust_excluded_instruments_stop_before_later_script_effects() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("robust_ar_four_ci_multiple_data.csv"),
    include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_four_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("unsupported.td"),
    "use robust_ar_four_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 row) robust\nestat ar, ci\nexport should-not-exist.csv\n",
  )
  .unwrap();
  let output = fixture.run("--json", "unsupported.td");
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr)
      .contains("currently support at most four excluded instruments")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
