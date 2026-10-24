use serde_json::Value;
use std::{
  collections::BTreeMap,
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
      "tabdat-truncreg-script-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("mma16p1tobit.csv"),
      include_str!("fixtures/truncreg_script_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("truncreg.td"),
      include_str!("../examples/mma16p1tobit/truncreg.td"),
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .args([mode, "-f", "truncreg.td"])
      .current_dir(&self.0)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn cells(line: &str) -> Vec<&str> {
  line.split(',').map(|v| v.trim_matches('"')).collect()
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.16e} != {b:.16e}"
  );
}
fn number(v: &Value) -> f64 {
  v.as_f64().unwrap()
}
#[test]
fn actual_source_intended_script_matches_certified_affine_r_control_and_text_export() {
  let json = Fixture::new();
  let output = json.run("--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let events: Vec<Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str(l).unwrap())
    .collect();
  assert_eq!(events.len(), 7);
  assert!(events.iter().all(|r| r["schema_version"] == 1));
  assert_eq!(events[1]["result_type"], "TruncatedGaussianResult");
  assert_eq!(
    events[2]["result_type"],
    "TruncatedGaussianMarginalEffectsResult"
  );
  let f = &events[1]["data"];
  let effects = &events[2]["data"];
  assert_eq!(f["estimator"], "truncreg");
  assert_eq!(f["include_intercept"], true);
  assert_eq!(f["lower_limit"], 0.0);
  assert_eq!(f["observation_count"], 240);
  assert_eq!(f["sample"]["total_observations"], 242);
  assert_eq!(f["sample"]["dropped_observations"], 2);
  assert_eq!(
    f["sample"]["retained_indices"],
    serde_json::json!((0..240).collect::<Vec<_>>())
  );
  assert_eq!(
    f["covariance_parameter_names"],
    serde_json::json!(["intercept", "lnwage", "lnsigma"])
  );
  assert_eq!(f["covariance_correction"], 1.0);
  assert_eq!(f["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(
    f["null_model"],
    "intercept_and_scale_refitted_same_sample_bound"
  );
  assert_eq!(
    f["log_likelihood_convention"],
    "gaussian_density_divided_by_population_retention_probability"
  );
  assert_eq!(effects["sample"], "retained_estimation_design");
  assert_eq!(
    effects["uncertainty"],
    "conditional_on_observed_design_full_joint_beta_lnsigma_delta_method"
  );
  for q in ["max_abs_scaled_score", "max_abs_scaled_newton_step"] {
    assert!(number(&f[q]) <= 1e-10);
  }
  let mut reference = BTreeMap::new();
  for line in include_str!("fixtures/truncreg_script_reference.csv")
    .lines()
    .skip(1)
  {
    let c = cells(line);
    assert!(
      reference
        .insert(
          (
            c[0].to_string(),
            c[1].parse::<usize>().unwrap(),
            c[2].parse::<usize>().unwrap()
          ),
          c[3].parse::<f64>().unwrap()
        )
        .is_none()
    );
  }
  assert_eq!(reference.len(), 49);
  let mut compared = 0;
  for ((q, i, j), expected) in reference {
    if [
      "scaled_score",
      "scaled_step",
      "null_scaled_score",
      "null_scaled_step",
    ]
    .contains(&q.as_str())
    {
      assert!(expected.abs() <= 1e-10);
      continue;
    }
    let actual = if let Some(field) = q.strip_prefix("coefficient_") {
      number(&f["coefficients"][i - 1][field])
    } else if let Some(field) = q.strip_prefix("ame_") {
      number(&effects["effects"][i - 1][field])
    } else {
      match q.as_str() {
        "joint_covariance" => number(&f["covariance_matrix"][i - 1][j - 1]),
        "mean_conditional_derivative_gradient" => number(&f[q.as_str()][i - 1]),
        "n" => number(&f["observation_count"]),
        "intercept" => 1.0,
        "sigma" => number(&f["scale"]["value"]),
        "log_sigma" => number(&f["scale"]["log_value"]),
        "log_sigma_standard_error" => number(&f["scale"]["log_standard_error"]),
        "sigma_standard_error" => number(&f["scale"]["standard_error"]),
        "sigma_confidence_lower" => number(&f["scale"]["confidence_lower"]),
        "sigma_confidence_upper" => number(&f["scale"]["confidence_upper"]),
        _ => number(&f[q.as_str()]),
      }
    };
    close(actual, expected);
    if q.ends_with("p_value") {
      assert!(actual > 0.0 && expected > 0.0 && (actual / expected - 1.0).abs() <= 1e-8);
    }
    compared += 1;
  }
  assert_eq!(compared, 45);
  let exported = fs::read_to_string(json.0.join("truncreg_predictions.csv")).unwrap();
  let native: Vec<_> = exported.lines().skip(1).map(cells).collect();
  let gold: Vec<_> = include_str!("fixtures/truncreg_script_predictions.csv")
    .lines()
    .skip(1)
    .map(cells)
    .collect();
  assert_eq!(native.len(), 242);
  assert_eq!(gold.len(), 242);
  for (row, want) in native.iter().zip(&gold) {
    assert_eq!(row.len(), 10);
    for j in 0..10 {
      if want[j].is_empty() {
        assert!(row[j].is_empty());
      } else {
        let a = row[j].parse::<f64>().unwrap();
        let b = want[j].parse::<f64>().unwrap();
        if j == 8 {
          assert!((a - b).abs() <= 1e-10);
        } else {
          close(a, b);
        }
      }
    }
  }
  let text = Fixture::new();
  let output = text.run("--text");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let rendered = String::from_utf8(output.stdout).unwrap();
  assert!(rendered.contains("Lower-truncated Gaussian maximum likelihood"));
  assert!(rendered.contains("not censoring"));
  assert!(rendered.contains("E[Y*|Y*>lower,X]"));
  assert_eq!(
    fs::read(json.0.join("truncreg_predictions.csv")).unwrap(),
    fs::read(text.0.join("truncreg_predictions.csv")).unwrap()
  );
}
#[test]
fn actual_script_source_target_and_export_failures_preserve_owned_files() {
  let existing = Fixture::new();
  fs::write(existing.0.join("truncreg_predictions.csv"), b"sentinel\n").unwrap();
  let before = fs::read(existing.0.join("mma16p1tobit.csv")).unwrap();
  assert!(!existing.run("--json").status.success());
  assert_eq!(
    fs::read(existing.0.join("truncreg_predictions.csv")).unwrap(),
    b"sentinel\n"
  );
  assert_eq!(
    fs::read(existing.0.join("mma16p1tobit.csv")).unwrap(),
    before
  );
  let missing = Fixture::new();
  fs::remove_file(missing.0.join("mma16p1tobit.csv")).unwrap();
  assert!(!missing.run("--json").status.success());
  assert!(!missing.0.join("truncreg_predictions.csv").exists());
  let target = Fixture::new();
  let content = include_str!("fixtures/truncreg_script_data.csv")
    .lines()
    .map(|line| {
      format!(
        "{line},{}\n",
        if line.starts_with("source_row") {
          "latent"
        } else {
          "777"
        }
      )
    })
    .collect::<String>();
  fs::write(target.0.join("mma16p1tobit.csv"), content.as_bytes()).unwrap();
  assert!(!target.run("--json").status.success());
  assert!(!target.0.join("truncreg_predictions.csv").exists());
  assert_eq!(
    fs::read(target.0.join("mma16p1tobit.csv")).unwrap(),
    content.as_bytes()
  );
}
