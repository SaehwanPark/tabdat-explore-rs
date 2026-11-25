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
    fs::write(
      root.join("projected_ar_data.csv"),
      include_str!("../examples/robust-ar/projected_ar_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_six_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_six_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_six_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_six_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_seven_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_seven_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_seven_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_seven_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_eight_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_eight_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_eight_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_eight_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_nine_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_nine_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_nine_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_nine_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_ten_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_ten_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_ten_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_ten_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_eleven_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_eleven_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_eleven_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_eleven_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_twelve_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_twelve_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_twelve_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_twelve_ci_secondary_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_thirteen_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_thirteen_ci_multiple_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_thirteen_ci_secondary_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_thirteen_ci_secondary_data.csv"),
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
fn robust_projected_cli_workflows_report_reference_sets_and_closed_envelope() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, group_count, expected) in [
    (
      "projected_ci.td",
      "robust_hc1",
      1.09090909090909,
      Value::Null,
      [0.789041366121279, 1.65982451868902],
    ),
    (
      "projected_ci_clustered.td",
      "cluster(group)",
      1.16528925619835,
      serde_json::json!(12),
      [0.331931575649699, 2.27083168350681],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
      rows[2]["result_type"],
      "RobustProjectedAndersonRubinConfidenceSet"
    );
    let data = &rows[2]["data"];
    assert_eq!(
      data["test"],
      "anderson_rubin_projected_scalar_robust_confidence_set"
    );
    assert_eq!(data["covariance"], covariance);
    assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
    assert_eq!(
      data["coverage"],
      "at_least_confidence_level_by_closed_projection"
    );
    assert_eq!(data["projection"], "closed_conservative_envelope");
    assert_eq!(data["closure_conservative"], true);
    assert_eq!(data["all_real_conservative_fallback"], false);
    assert_eq!(data["degrees_of_freedom"], 2);
    assert_eq!(data["observation_count"], 48);
    assert_eq!(data["auxiliary_parameter_count"], 4);
    assert_eq!(data["auxiliary_residual_df"], 44);
    assert_eq!(data["cluster_group_count"], group_count);
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 5.99146454710798, 2e-12);
    assert_eq!(data["shape"], "bounded");
    close(&data["intervals"][0]["lower"], expected[0], 2e-7);
    close(&data["intervals"][0]["upper"], expected[1], 2e-7);
  }

  let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("examples/robust-ar")
    .join("projected_ci_clustered.td");
  let output = fixture.run("--text", script.to_str().unwrap());
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Projected Anderson-Rubin confidence set (CR1 cluster(group)"));
  assert!(text.contains("closed conservative envelope"));
}

#[test]
fn six_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "six_instrument_ci.td",
      "robust_hc1",
      1.05633802816901,
      Value::Null,
      [0.915364151803677, 1.53507912187761],
    ),
    (
      "six_instrument_ci_clustered.td",
      "cluster(group)",
      1.09301643192488,
      serde_json::json!(25),
      [0.951605392208714, 1.45797127952922],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
      rows[2]["result_type"],
      "RobustMultipleInstrumentAndersonRubinConfidenceSet"
    );
    let data = &rows[2]["data"];
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!(["z1", "z2", "z3", "z4", "z5", "z6"])
    );
    assert_eq!(data["degrees_of_freedom"], 6);
    assert_eq!(data["observation_count"], 150);
    assert_eq!(data["auxiliary_parameter_count"], 8);
    assert_eq!(data["auxiliary_residual_df"], 142);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 12.591587243744, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }

  fs::write(
    fixture.0.join("retained-fit.td"),
    "use robust_ar_six_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6) robust\nestat ar, ci\nreplace y = 999\ndrop z1\nestat ar, ci\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run("--json", "retained-fit.td"));
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[2]["data"], rows[5]["data"]);
}

#[test]
fn seven_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "seven_instrument_ci.td",
      "robust_hc1",
      1.05660377358491,
      Value::Null,
      [0.87058126469739, 1.50944638151829],
    ),
    (
      "seven_instrument_ci_clustered.td",
      "cluster(group)",
      1.09598031173093,
      serde_json::json!(24),
      [1.03741500489376, 1.47495371597571],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
      rows[2]["result_type"],
      "RobustMultipleInstrumentAndersonRubinConfidenceSet"
    );
    let data = &rows[2]["data"];
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!(["z1", "z2", "z3", "z4", "z5", "z6", "z7"])
    );
    assert_eq!(data["degrees_of_freedom"], 7);
    assert_eq!(data["observation_count"], 168);
    assert_eq!(data["auxiliary_parameter_count"], 9);
    assert_eq!(data["auxiliary_residual_df"], 159);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 14.0671404493402, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }
}

#[test]
fn eight_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "eight_instrument_ci.td",
      "robust_hc1",
      1.05494505494505,
      Value::Null,
      [1.0887754541176, 1.40883145836894],
    ),
    (
      "eight_instrument_ci_clustered.td",
      "cluster(group)",
      1.09507883420927,
      serde_json::json!(24),
      [1.22474047234525, 1.32397269272157],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
      rows[2]["result_type"],
      "RobustMultipleInstrumentAndersonRubinConfidenceSet"
    );
    let data = &rows[2]["data"];
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!(["z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8"])
    );
    assert_eq!(data["degrees_of_freedom"], 8);
    assert_eq!(data["observation_count"], 192);
    assert_eq!(data["auxiliary_parameter_count"], 10);
    assert_eq!(data["auxiliary_residual_df"], 182);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 15.5073130558655, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/eight_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(8)"));
  assert!(text.contains("two through thirteen excluded instruments"));
}

#[test]
fn nine_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "nine_instrument_ci.td",
      "robust_hc1",
      1.05365853658537,
      Value::Null,
      [0.911362825704433, 1.28291406681336],
    ),
    (
      "nine_instrument_ci_clustered.td",
      "cluster(group)",
      1.09437963944857,
      serde_json::json!(24),
      [0.945792747726182, 1.20035149024702],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
    assert_eq!(rows.len(), 3);
    assert_eq!(
      rows[2]["result_type"],
      "RobustMultipleInstrumentAndersonRubinConfidenceSet"
    );
    let data = &rows[2]["data"];
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!(["z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9"])
    );
    assert_eq!(data["degrees_of_freedom"], 9);
    assert_eq!(data["observation_count"], 216);
    assert_eq!(data["auxiliary_parameter_count"], 11);
    assert_eq!(data["auxiliary_residual_df"], 205);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 16.9189776046204, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/nine_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(9)"));
  assert!(text.contains("two through thirteen excluded instruments"));
}

#[test]
fn ten_instrument_hc1_and_cr1_workflows_report_reference_sets_and_json_availability() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "ten_instrument_ci.td",
      "robust_hc1",
      1.05263157894737,
      Value::Null,
      [0.954530263639083, 1.303078784421],
    ),
    (
      "ten_instrument_ci_clustered.td",
      "cluster(group)",
      1.09382151029748,
      serde_json::json!(24),
      [0.967341667827175, 1.2132537558033],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
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
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!(["z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9", "z10"])
    );
    assert_eq!(data["degrees_of_freedom"], 10);
    assert_eq!(data["observation_count"], 240);
    assert_eq!(data["auxiliary_parameter_count"], 12);
    assert_eq!(data["auxiliary_residual_df"], 228);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 18.3070380532751, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/ten_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(10)"));
  assert!(text.contains("two through thirteen excluded instruments"));

  fs::write(
    fixture.0.join("ten-retained-fit.td"),
    "use robust_ar_ten_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10) robust\nestat ar, ci\nreplace y = 999\ndrop z1\nestat ar, ci\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run("--json", "ten-retained-fit.td"));
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[2]["data"], rows[5]["data"]);
}

#[test]
fn eleven_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, shape, lower, upper) in [
    (
      "eleven_instrument_ci.td",
      "robust_hc1",
      1.05726872246696,
      Value::Null,
      "bounded",
      Some(1.1070615225938),
      Some(1.36180085509442),
    ),
    (
      "eleven_instrument_ci_clustered.td",
      "cluster(group)",
      1.09864010725915,
      serde_json::json!(24),
      "bounded",
      Some(1.23563485161083),
      Some(1.29735877469493),
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
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
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!([
        "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9", "z10", "z11"
      ])
    );
    assert_eq!(data["degrees_of_freedom"], 11);
    assert_eq!(data["observation_count"], 240);
    assert_eq!(data["auxiliary_parameter_count"], 13);
    assert_eq!(data["auxiliary_residual_df"], 227);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], shape);
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 19.6751375726825, 2e-12);
    match (lower, upper) {
      (Some(lower), Some(upper)) => {
        close(&data["intervals"][0]["lower"], lower, 3e-8);
        close(&data["intervals"][0]["upper"], upper, 3e-8);
      }
      (None, None) => assert_eq!(data["intervals"].as_array().unwrap().len(), 0),
      mismatch => panic!("unexpected endpoint bounds: {mismatch:?}"),
    }
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/eleven_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(11)"));
  assert!(text.contains("two through thirteen excluded instruments"));
}

#[test]
fn twelve_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, expected) in [
    (
      "twelve_instrument_ci.td",
      "robust_hc1",
      1.05109489051095,
      Value::Null,
      [1.03010254615802, 1.28953391130388],
    ),
    (
      "twelve_instrument_ci_clustered.td",
      "cluster(group)",
      1.09298635353856,
      serde_json::json!(24),
      [1.14531210503888, 1.21665138888151],
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
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
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!([
        "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9", "z10", "z11", "z12"
      ])
    );
    assert_eq!(data["degrees_of_freedom"], 12);
    assert_eq!(data["observation_count"], 288);
    assert_eq!(data["auxiliary_parameter_count"], 14);
    assert_eq!(data["auxiliary_residual_df"], 274);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 21.0260698174831, 2e-12);
    close(&data["intervals"][0]["lower"], expected[0], 3e-8);
    close(&data["intervals"][0]["upper"], expected[1], 3e-8);
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/twelve_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(12)"));
  assert!(text.contains("two through thirteen excluded instruments"));

  fs::write(
    fixture.0.join("twelve-retained-fit.td"),
    "use robust_ar_twelve_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12) robust\nestat ar, ci\nreplace y = 999\ndrop z1\nestat ar, ci\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run("--json", "twelve-retained-fit.td"));
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[2]["data"], rows[5]["data"]);
}

#[test]
fn thirteen_instrument_hc1_and_cr1_workflows_report_reference_sets() {
  let fixture = Fixture::new();
  for (script_name, covariance, correction, groups, shape, interval) in [
    (
      "thirteen_instrument_ci.td",
      "robust_hc1",
      1.05050505050505,
      Value::Null,
      "bounded",
      Some([1.17583577145803, 1.25479744110509]),
    ),
    (
      "thirteen_instrument_ci_clustered.td",
      "cluster(group)",
      1.09266578831796,
      serde_json::json!(24),
      "empty",
      None,
    ),
  ] {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("examples/robust-ar")
      .join(script_name);
    let rows = json_lines(&fixture.run("--json", script.to_str().unwrap()));
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
    assert_eq!(data["covariance"], covariance);
    assert_eq!(
      data["excluded_instruments"],
      serde_json::json!([
        "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9", "z10", "z11", "z12", "z13"
      ])
    );
    assert_eq!(data["degrees_of_freedom"], 13);
    assert_eq!(data["observation_count"], 312);
    assert_eq!(data["auxiliary_parameter_count"], 15);
    assert_eq!(data["auxiliary_residual_df"], 297);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], shape);
    close(&data["covariance_correction"], correction, 2e-14);
    close(&data["critical_chi_square"], 22.3620324948269, 2e-12);
    match interval {
      Some([lower, upper]) => {
        assert_eq!(data["intervals"].as_array().unwrap().len(), 1);
        close(&data["intervals"][0]["lower"], lower, 3e-8);
        close(&data["intervals"][0]["upper"], upper, 3e-8);
      }
      None => assert!(data["intervals"].as_array().unwrap().is_empty()),
    }
  }

  let script =
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/robust-ar/thirteen_instrument_ci.td");
  let text = fixture.run("--text", script.to_str().unwrap());
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("asymptotic chi-square(13)"));
  assert!(text.contains("two through thirteen excluded instruments"));

  fs::write(
    fixture.0.join("thirteen-retained-fit.td"),
    "use robust_ar_thirteen_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13) robust\nestat ar, ci\nreplace y = 999\ndrop z1\nestat ar, ci\n",
  )
  .unwrap();
  let rows = json_lines(&fixture.run("--json", "thirteen-retained-fit.td"));
  assert_eq!(rows.len(), 6);
  assert_eq!(rows[2]["data"], rows[5]["data"]);
}

#[test]
fn fourteen_instrument_rejection_preserves_later_effects() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("fourteen-instruments.td"),
    "use robust_ar_thirteen_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 row) robust\nestat ar, ci\nexport should-not-exist.csv\n",
  )
  .unwrap();
  let output = fixture.run("--json", "fourteen-instruments.td");
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr)
      .contains("currently support at most thirteen excluded instruments")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
