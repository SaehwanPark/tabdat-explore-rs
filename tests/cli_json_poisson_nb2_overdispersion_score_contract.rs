mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Output};

struct Fixture(TestDir);

impl Fixture {
  fn new() -> Self {
    let dir = TestDir::new();
    fs::write(
      dir.path().join("score_reference.csv"),
      include_bytes!("../examples/nb2-overdispersion-score/score_reference.csv"),
    )
    .unwrap();
    Self(dir)
  }

  fn run(&self, flag: &str, script_name: &str) -> Output {
    tabdat_bin()
      .current_dir(self.0.path())
      .args([flag, "-f"])
      .arg(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
          .join("examples/nb2-overdispersion-score")
          .join(script_name),
      )
      .output()
      .unwrap()
  }
}

fn envelopes(output: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn executable_model_row_robust_and_cluster_scripts_emit_v1_score_results() {
  for (script, expected_covariance, independent_unit) in [
    (
      "model_based.td",
      "poisson_information_nb2_score",
      "observation",
    ),
    (
      "robust.td",
      "robust_row_nb2_score_n_over_n_minus_1",
      "observation",
    ),
    (
      "clustered.td",
      "cluster_nb2_score_g_over_g_minus_1",
      "cluster",
    ),
  ] {
    let fixture = Fixture::new();
    let output = fixture.run("--json", script);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let rows = envelopes(&output);
    assert_eq!(rows.len(), 3);
    assert!(
      rows
        .iter()
        .all(|row| row["schema_version"] == 1 && row["error"].is_null())
    );
    let event = &rows[2];
    assert_eq!(event["result_type"], "PoissonOverdispersionScoreTestResult");
    let data = &event["data"];
    assert_eq!(data["estimator"], "poisson_nb2_overdispersion_score");
    assert_eq!(data["independent_unit"], independent_unit);
    assert_eq!(data["covariance"], expected_covariance);
    assert_eq!(data["observation_count"], 180);
    assert_eq!(data["offset_variable"], "log_exposure");
    assert_eq!(data["reference_distribution"], "asymptotic_standard_normal");
    assert!(data["score"].as_f64().unwrap().is_finite());
    assert!(data["score_variance"].as_f64().unwrap() > 0.0);
    assert!(data["p_value"].as_f64().unwrap().is_finite());
    if independent_unit == "cluster" {
      assert_eq!(data["cluster"]["group_count"], 30);
    } else {
      assert!(data["cluster"].is_null());
    }
  }
}

#[test]
fn executable_text_report_keeps_offset_and_inference_limits_visible() {
  let fixture = Fixture::new();
  let output = fixture.run("--text", "model_based.td");
  assert!(output.status.success(), "{output:?}");
  let text = String::from_utf8_lossy(&output.stdout);
  for expected in [
    "Poisson one-sided NB2 overdispersion score test",
    "one-sided p =",
    "model-based Poisson information",
    "Correct Poisson conditional means are required",
    "Fixed offset log_exposure is included in retained fitted means",
  ] {
    assert!(text.contains(expected), "missing {expected:?}: {text}");
  }
}
