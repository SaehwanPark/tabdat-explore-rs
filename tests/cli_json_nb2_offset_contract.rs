use serde_json::{Value, json};
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
      "tabdat-nb-offset-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("nb2_offset.csv"),
      include_bytes!("../examples/nb2-offset/nb2_offset.csv"),
    )
    .unwrap();
    Self(root)
  }
  fn script(&self, flag: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([flag, "-f"])
      .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/nb2-offset/workflow.td"))
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
fn actual_nb2_offset_script_matches_joint_r_reference_in_text_and_json() {
  let f = Fixture::new();
  let text_fixture = Fixture::new();
  let out = f.script("--json");
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
    .find(|r| r["result_type"] == "NbregResult")
    .unwrap()["data"];
  let effect = &results
    .iter()
    .find(|r| r["result_type"] == "NbregMarginalEffectsResult")
    .unwrap()["data"];
  assert_eq!(fit["offset_variable"], "log_time");
  assert_eq!(fit["offset_coefficient"], 1.0);
  assert_eq!(fit["linear_index"], "x_beta_plus_offset");
  assert_eq!(
    fit["null_model"],
    "intercept_and_dispersion_reestimated_with_same_fixed_offset"
  );
  assert_eq!(fit["mean_parameter_count"], 3);
  assert_eq!(fit["parameter_count"], 4);
  assert_eq!(fit["df_residual"], 32);
  assert_eq!(
    fit["sample"]["retained_indices"],
    json!((0..36).collect::<Vec<_>>())
  );
  assert_eq!(
    fit["covariance_parameter_names"],
    json!(["intercept", "x", "z", "lnalpha"])
  );
  assert_eq!(effect["offset_variable"], "log_time");
  assert_eq!(
    effect["offset_treatment"],
    "fixed_at_retained_values_in_derivatives_and_delta_uncertainty"
  );
  assert_eq!(
    effect["covariance_role"],
    "beta_block_of_full_joint_beta_lnalpha_covariance"
  );
  let exported = fs::read_to_string(f.0.join("nb2_offset_predictions.csv")).unwrap();
  let cells: Vec<Vec<_>> = exported
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect())
    .collect();
  assert_eq!(cells.len(), 36);
  for line in include_str!("../crates/tabdat-stats/tests/fixtures/nb2_offset_reference.csv")
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
      fit["coefficients"][i][field].clone()
    } else if let Some(field) = r[1].strip_prefix("effect_") {
      effect["effects"][i][field].clone()
    } else if fit["dispersion"].get(r[1]).is_some() {
      fit["dispersion"][r[1]].clone()
    } else {
      match r[1] {
        "covariance" => fit["covariance_matrix"][i][j].clone(),
        "wald_statistic" => fit["omnibus_wald"]["statistic"].clone(),
        "wald_p_value" => fit["omnibus_wald"]["p_value"].clone(),
        "mean" => json!(cells[i][6].parse::<f64>().unwrap()),
        "xb" => json!(cells[i][7].parse::<f64>().unwrap()),
        _ => fit[r[1]].clone(),
      }
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
  assert!(
    text.contains("intercept-only with same fixed offset, with nuisance dispersion reestimated")
  );
  assert_eq!(
    exported.as_bytes(),
    fs::read(text_fixture.0.join("nb2_offset_predictions.csv")).unwrap()
  );
}

#[test]
fn nb2_offset_discovery_is_explicit_and_unknown_offset_stops_before_export() {
  let f = Fixture::new();
  let out = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .args(["--json", "--describe-command", "nbreg"])
    .output()
    .unwrap();
  assert!(out.status.success());
  let r = envelopes(&out);
  let d = &r[0]["data"];
  assert_eq!(
    d["syntax"],
    "nbreg y [x1 x2] [, offset(var) robust cluster(var) noconstant]"
  );
  assert!(
    d["options"]
      .as_array()
      .unwrap()
      .iter()
      .any(|o| o["name"] == "offset")
  );
  let out = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .args(["--json", "--help-topic", "nbreg"])
    .output()
    .unwrap();
  assert!(out.status.success());
  let r = envelopes(&out);
  let help = r[0]["data"]["text"].as_str().unwrap();
  assert!(help.contains("offset(log_time)"));
  assert!(help.contains("same fixed offset"));
  fs::write(
    f.0.join("bad.td"),
    "use nb2_offset.csv\nnbreg y x, offset(absent)\nexport should_not_exist.csv\n",
  )
  .unwrap();
  let out = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&f.0)
    .args(["--json", "-f", "bad.td"])
    .output()
    .unwrap();
  assert!(!out.status.success());
  let r = envelopes(&out);
  assert_eq!(r.len(), 2);
  assert!(!r[1]["error"].is_null());
  assert!(!f.0.join("should_not_exist.csv").exists());
}
