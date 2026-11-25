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
      "tabdat-exposure-cli-{}-{}",
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
      .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/poisson-offset/exposure.td"))
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
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

fn result<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
  &rows.iter().find(|row| row["result_type"] == kind).unwrap()["data"]
}

#[test]
fn poisson_and_nb2_exposure_workflow_serializes_fit_effect_and_named_report_metadata() {
  let fixture = Fixture::new();
  let json = fixture.script("--json");
  assert!(json.status.success(), "{json:?}");
  let rows = envelopes(&json);
  assert_eq!(rows.len(), 15);
  assert!(rows.iter().all(|row| row["error"].is_null()));

  let poisson = result(&rows, "PoissonResult");
  assert_eq!(poisson["exposure_variable"], "exposure");
  assert_eq!(poisson["exposure_coefficient"], 1.0);
  assert_eq!(poisson["linear_index"], "x_beta_plus_log_exposure");
  assert_eq!(
    poisson["null_model"],
    "intercept_only_with_same_fixed_log_exposure"
  );
  assert!(poisson["offset_variable"].is_null());
  let overdispersion = result(&rows, "PoissonOverdispersionScoreTestResult");
  assert_eq!(overdispersion["exposure_variable"], "exposure");
  assert!(overdispersion["offset_variable"].is_null());
  let poisson_effect = result(&rows, "PoissonMarginalEffectsResult");
  assert_eq!(poisson_effect["exposure_variable"], "exposure");
  assert_eq!(poisson_effect["exposure_coefficient"], 1.0);
  assert_eq!(
    poisson_effect["exposure_treatment"],
    "fixed_at_retained_values_in_derivatives_and_delta_uncertainty"
  );
  let poisson_table = result(&rows, "EstimateTableResult");
  let poisson_model = poisson_table["models"]
    .as_array()
    .unwrap()
    .iter()
    .find(|model| model["estimator"] == "poisson")
    .unwrap();
  assert_eq!(poisson_model["poisson_exposure_variable"], "exposure");
  assert_eq!(poisson_model["poisson_exposure_coefficient"], 1.0);
  assert_eq!(
    poisson_model["poisson_null_model"],
    "intercept_only_with_same_fixed_log_exposure"
  );

  let nb2 = result(&rows, "NbregResult");
  assert_eq!(nb2["exposure_variable"], "exposure");
  assert_eq!(nb2["exposure_coefficient"], 1.0);
  assert_eq!(nb2["linear_index"], "x_beta_plus_log_exposure");
  assert_eq!(
    nb2["null_model"],
    "intercept_and_dispersion_reestimated_with_same_fixed_log_exposure"
  );
  let nb2_effect = result(&rows, "NbregMarginalEffectsResult");
  assert_eq!(nb2_effect["exposure_variable"], "exposure");
  assert_eq!(
    nb2_effect["exposure_treatment"],
    "fixed_at_retained_values_in_derivatives_and_delta_uncertainty"
  );
  let nb2_table = rows
    .iter()
    .filter(|row| row["result_type"] == "EstimateTableResult")
    .last()
    .unwrap();
  assert_eq!(
    nb2_table["data"]["models"][0]["nb2"]["exposure_variable"],
    "exposure"
  );
  assert_eq!(
    nb2_table["data"]["models"][0]["nb2"]["null_model"],
    "intercept_only_with_same_fixed_log_exposure_refit_alpha"
  );

  let text = fixture.script("--text");
  assert!(text.status.success(), "{text:?}");
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("Fixed exposure: exposure (log coefficient 1, not estimated)"));
  assert!(text.contains("Poisson exposure (coefficient 1)\texposure"));
  assert!(text.contains("NB2 exposure (coefficient 1)\texposure"));
}
