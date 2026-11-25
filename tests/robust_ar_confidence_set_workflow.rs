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
      "tabdat-robust-ar-ci-{}-{}",
      std::process::id(),
      NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("robust_ar_ci_data.csv"),
      include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_ci_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_multi_ci_data.csv"),
      include_str!("../crates/tabdat-stats/tests/fixtures/robust_ar_multi_ci_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("robust_ar_nine_ci_multiple_data.csv"),
      include_str!("../examples/robust-ar/robust_ar_nine_ci_multiple_data.csv"),
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
fn robust_scalar_ci_scripts_report_independent_sets_and_reuse_the_fit_sample() {
  // Endpoints and corrections come from base R 4.6.2 in
  // crates/tabdat-stats/tests/fixtures/robust_ar_ci_reference.R.
  for (script, covariance, n, correction, groups, lower, upper) in [
    (
      include_str!("../examples/robust-ar/scalar_ci.td"),
      "robust_hc1",
      78,
      1.04,
      Value::Null,
      1.36445194119086,
      1.96845342740648,
    ),
    (
      include_str!("../examples/robust-ar/scalar_ci_clustered.td"),
      "cluster(g)",
      77,
      1.14114114114114,
      Value::from(10),
      1.20537141708336,
      2.13637942939771,
    ),
  ] {
    let fixture = Fixture::new();
    let script_path =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(if covariance == "robust_hc1" {
        "examples/robust-ar/scalar_ci.td"
      } else {
        "examples/robust-ar/scalar_ci_clustered.td"
      });
    let output = fixture.run("--json", script_path.to_str().unwrap());
    let rows = json_lines(&output);
    assert_eq!(rows.len(), 6);
    assert_eq!(
      rows[1]["data"]["first_stage"]["scalar_ar_confidence_set_available"],
      true
    );
    assert_eq!(rows[2]["result_type"], "RobustAndersonRubinConfidenceSet");
    assert_eq!(rows[2]["data"], rows[5]["data"]);
    let data = &rows[2]["data"];
    assert_eq!(data["test"], "anderson_rubin_scalar_robust_confidence_set");
    assert_eq!(data["covariance"], covariance);
    assert_eq!(data["reference_distribution"], "asymptotic_chi_square");
    assert_eq!(data["weak_iv_robust"], true);
    assert_eq!(
      data["endogenous_nuisance_policy"],
      "none_single_endogenous_only"
    );
    assert_eq!(data["endogenous"], "x");
    assert_eq!(data["excluded_instrument"], "z");
    assert_eq!(data["confidence_level"], 95.0);
    assert_eq!(data["observation_count"], n);
    assert_eq!(data["auxiliary_parameter_count"], 3);
    assert_eq!(data["auxiliary_residual_df"], n - 3);
    assert_eq!(data["degrees_of_freedom"], 1);
    assert_eq!(data["cluster_group_count"], groups);
    assert_eq!(data["shape"], "bounded");
    close(&data["covariance_correction"], correction, 1e-12);
    close(&data["critical_chi_square"], 3.84145882069413, 1e-12);
    close(&data["intervals"][0]["lower"], lower, 1e-10);
    close(&data["intervals"][0]["upper"], upper, 1e-10);

    let output = fixture.run("--text", script_path.to_str().unwrap());
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
      "Robust Anderson-Rubin confidence set",
      "asymptotic chi-square(1)",
      "unrestricted nuisance parameters",
      "No finite-sample or few-cluster guarantee",
    ] {
      assert!(text.contains(expected), "{text}");
    }
    assert_eq!(script.lines().count(), 6);
  }
}

#[test]
fn robust_ar_ci_rejects_eighteen_instruments_before_later_effects() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("robust_ar_ten_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_ten_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("robust_ar_seventeen_ci_multiple_data.csv"),
    include_str!("../examples/robust-ar/robust_ar_seventeen_ci_multiple_data.csv"),
  )
  .unwrap();
  fs::write(
    fixture.0.join("multiple.td"),
    "use robust_ar_seventeen_ci_multiple_data.csv\nivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 z14 z15 z16 z17 row) robust\nestat ar, ci\nexport should-not-exist.csv\n",
  )
  .unwrap();
  let output = fixture.run("--json", "multiple.td");
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8_lossy(&output.stderr)
      .contains("currently support at most seventeen excluded instruments")
  );
  assert!(!fixture.0.join("should-not-exist.csv").exists());
}
