use serde_json::Value;
use std::{
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
      "tabdat-offset-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("poisson_offset.csv"),
      include_bytes!("../examples/poisson-offset/poisson_offset.csv"),
    )
    .unwrap();
    Self(root)
  }
  fn script(&self, flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([flag, "-f"])
      .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/poisson-offset/workflow.td"))
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(out: &Output) -> Vec<Value> {
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect()
}

#[test]
fn actual_offset_script_reports_fixed_semantics_and_matches_r_and_text_export() {
  let json_fixture = Fixture::new();
  let text_fixture = Fixture::new();
  let out = json_fixture.script("--json");
  assert!(out.status.success(), "{out:?}");
  let results = envelopes(&out);
  assert_eq!(results.len(), 9);
  assert!(
    results
      .iter()
      .all(|r| r["error"].is_null() && r["schema_version"] == 1)
  );
  let fit = &results
    .iter()
    .find(|r| r["result_type"] == "PoissonResult")
    .unwrap()["data"];
  assert_eq!(fit["offset_variable"], "log_time");
  assert_eq!(fit["offset_coefficient"], 1.0);
  assert_eq!(fit["linear_index"], "x_beta_plus_offset");
  assert_eq!(fit["null_model"], "intercept_only_with_same_fixed_offset");
  assert_eq!(fit["observation_count"], 36);
  assert_eq!(fit["df_residual"], 33);
  assert_eq!(
    fit["sample"]["retained_indices"],
    serde_json::json!((0..36).collect::<Vec<_>>())
  );
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["intercept", "x", "z"])
  );
  assert!(fit["likelihood_ratio_p_value"].is_null());
  let effect = &results
    .iter()
    .find(|r| r["result_type"] == "PoissonMarginalEffectsResult")
    .unwrap()["data"];
  assert_eq!(effect["offset_variable"], "log_time");
  assert_eq!(
    effect["offset_treatment"],
    "fixed_at_retained_values_in_derivatives_and_delta_uncertainty"
  );
  for line in include_str!("../crates/tabdat-stats/tests/fixtures/poisson_offset_reference.csv")
    .lines()
    .skip(1)
  {
    let r: Vec<_> = line.split(',').collect();
    if r[0] != "multiple_robust" {
      continue;
    }
    let i: usize = r[2].parse().unwrap();
    let j: usize = r[3].parse().unwrap();
    let actual = if let Some(field) = r[1].strip_prefix("coefficient_") {
      &fit["coefficients"][i][field]
    } else if let Some(field) = r[1].strip_prefix("effect_") {
      &effect["effects"][i][field]
    } else if r[1] == "covariance" {
      &fit["covariance_matrix"][i][j]
    } else if r[1] == "wald_statistic" {
      &fit["omnibus_wald"]["statistic"]
    } else if r[1] == "wald_p_value" {
      &fit["omnibus_wald"]["p_value"]
    } else if matches!(r[1], "mean" | "xb") {
      continue;
    } else {
      &fit[r[1]]
    };
    if r[4] == "NA" {
      assert!(actual.is_null());
      continue;
    }
    let expected: f64 = r[4].parse().unwrap();
    let actual = actual.as_f64().unwrap();
    let tolerance = if r[1].ends_with("p_value") {
      expected.abs() * 1e-9
    } else {
      1e-9
    };
    assert!(
      (actual - expected).abs() <= tolerance,
      "{r:?}: {actual} != {expected}"
    );
  }
  let text = text_fixture.script("--text");
  assert!(text.status.success(), "{text:?}");
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("Fixed offset: log_time (coefficient 1, not estimated)"));
  assert!(text.contains("intercept-only with same fixed offset"));
  assert_eq!(
    fs::read(json_fixture.0.join("poisson_offset_predictions.csv")).unwrap(),
    fs::read(text_fixture.0.join("poisson_offset_predictions.csv")).unwrap()
  );
}

#[test]
fn unknown_offset_stops_script_before_output_file() {
  let f = Fixture::new();
  fs::write(
    f.0.join("bad.td"),
    "use poisson_offset.csv\npoisson y x, offset(absent)\nexport should_not_exist.csv\n",
  )
  .unwrap();
  let out = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&f.0)
    .args(["--json", "-f", "bad.td"])
    .output()
    .unwrap();
  assert!(!out.status.success());
  let results = envelopes(&out);
  assert_eq!(results.len(), 2);
  assert!(!results[1]["error"].is_null());
  assert!(!f.0.join("should_not_exist.csv").exists());
}
