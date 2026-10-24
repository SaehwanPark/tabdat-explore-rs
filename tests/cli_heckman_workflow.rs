//! Actual shipped-script boundary on an independent closed-form24-row control,
//! NOT replacement author data. Original200/130 source audit is separate.
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
      "tabdat-heckman-script-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("mma16p1tobit.csv"),
      include_str!("fixtures/heckman_script_data.csv"),
    )
    .unwrap();
    fs::write(
      root.join("heckman.td"),
      include_str!("../examples/mma16p1tobit/heckman.td"),
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, mode: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .args([mode, "-f", "heckman.td"])
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
fn records(output: &Output) -> Vec<Value> {
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn actual_heckman_script_matches_closed_form_r_inference_predictions_and_text_export() {
  let json = Fixture::new();
  let output = json.run("--json");
  assert!(
    output.status.success(),
    "{:?}\n{}",
    output.status,
    String::from_utf8_lossy(&output.stderr)
  );
  let events = records(&output);
  assert_eq!(events.len(), 8);
  assert!(events.iter().all(|r| r["schema_version"] == 1));
  assert_eq!(events[1]["result_type"], "HeckmanTwoStepResult");
  let fit = &events[1]["data"];
  assert_eq!(fit["method"], "twostep");
  assert_eq!(fit["observation_count"], 24);
  assert_eq!(fit["selected_count"], 12);
  assert_eq!(fit["unselected_count"], 12);
  assert_eq!(fit["scale_correlation_inference"], "points_only");
  assert!(fit["joint_log_likelihood"].is_null());
  assert!(
    fit["identification_warning"]
      .as_str()
      .unwrap()
      .contains("functional form")
  );
  assert_eq!(fit["excluded_selection_predictors"], serde_json::json!([]));
  assert!(fit["selection_max_abs_scaled_score"].as_f64().unwrap() <= 1e-10);
  assert!(
    fit["selection_max_abs_scaled_newton_step"]
      .as_f64()
      .unwrap()
      <= 1e-10
  );
  assert!(
    fit["max_abs_scaled_outcome_normal_equation"]
      .as_f64()
      .unwrap()
      <= 1e-10
  );
  let mut reference = BTreeMap::new();
  for line in include_str!("fixtures/heckman_script_reference.csv")
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
  for (stage, coefs, covariance) in [
    (
      "selection",
      &fit["selection_coefficients"],
      &fit["selection_covariance_matrix"],
    ),
    ("outcome", &fit["coefficients"], &fit["covariance_matrix"]),
  ] {
    for (i, b) in coefs.as_array().unwrap().iter().enumerate() {
      for field in [
        "value",
        "standard_error",
        "statistic",
        "p_value",
        "confidence_lower",
        "confidence_upper",
      ] {
        let expected = reference[&(format!("{stage}_{field}"), i + 1, 0)];
        let actual = number(&b[field]);
        close(actual, expected);
        if field == "p_value" {
          assert!(actual > 0.0 && expected > 0.0 && (actual / expected - 1.0).abs() <= 1e-8);
        }
      }
      for (j, v) in covariance[i].as_array().unwrap().iter().enumerate() {
        close(
          number(v),
          reference[&(format!("{stage}_covariance"), i + 1, j + 1)],
        );
      }
    }
  }
  for (name, key) in [
    ("n", "observation_count"),
    ("selected_count", "selected_count"),
    ("sigma", "sigma"),
    ("rho", "rho"),
    ("outcome_sse", "outcome_sse"),
    ("selection_log_likelihood", "selection_log_likelihood"),
  ] {
    close(number(&fit[key]), reference[&(name.into(), 0, 0)]);
  }
  close(number(&fit["coefficients"][0]["value"]), 3.0);
  close(number(&fit["coefficients"][1]["value"]), 2.0);
  close(number(&fit["coefficients"][2]["value"]), 0.5);
  close(number(&fit["sigma"]), 1.0);
  close(number(&fit["rho"]), 0.5);
  assert_eq!(
    fit["selection_sample"]["retained_indices"],
    serde_json::json!((0..24).collect::<Vec<_>>())
  );
  let selected = vec![0, 1, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21];
  assert_eq!(
    fit["selected_sample"]["retained_indices"],
    serde_json::json!(selected)
  );
  assert_eq!(fit["selection_sample"]["dropped_observations"], 0);
  assert_eq!(fit["selected_sample"]["dropped_observations"], 12);
  let csv = fs::read_to_string(json.0.join("heckman_predictions.csv")).unwrap();
  let rows: Vec<_> = csv.lines().map(cells).collect();
  assert_eq!(rows.len(), 25);
  assert_eq!(
    rows[0],
    vec![
      "source_row",
      "e",
      "lnwage",
      "ystar",
      "ytrunc",
      "ycens",
      "dy",
      "latent",
      "selection_probability",
      "inverse_mills",
      "conditional_selected_mean",
      "observed_contribution_mean"
    ]
  );
  for (i, (gold, input)) in include_str!("fixtures/heckman_script_predictions.csv")
    .lines()
    .skip(1)
    .zip(
      include_str!("fixtures/heckman_script_data.csv")
        .lines()
        .skip(1),
    )
    .enumerate()
  {
    let expected = cells(gold);
    let input = cells(input);
    let actual = &rows[i + 1];
    assert_eq!(actual.len(), 12);
    for j in 0..7 {
      if input[j].is_empty() {
        assert!(actual[j].is_empty());
      } else {
        assert_eq!(
          actual[j].parse::<f64>().unwrap(),
          input[j].parse::<f64>().unwrap()
        );
      }
    }
    for j in 0..5 {
      let value = actual[7 + j].parse::<f64>().unwrap();
      let reference = expected[1 + j].parse::<f64>().unwrap();
      if j == 1 {
        assert!((value - reference).abs() <= 1e-10);
      } else {
        close(value, reference);
      }
    }
  }
  let text = Fixture::new();
  let rendered = text.run("--text");
  assert!(
    rendered.status.success(),
    "{}",
    String::from_utf8_lossy(&rendered.stderr)
  );
  let stdout = String::from_utf8(rendered.stdout).unwrap();
  assert!(stdout.contains("WARNING: No excluded selection predictor"));
  assert!(stdout.contains("not joint ML"));
  assert!(stdout.contains("moment points only"));
  assert_eq!(
    fs::read(json.0.join("heckman_predictions.csv")).unwrap(),
    fs::read(text.0.join("heckman_predictions.csv")).unwrap()
  );
}

#[test]
fn actual_heckman_script_preserves_colliding_export_and_stops_on_missing_input_or_target() {
  let collision = Fixture::new();
  let input = fs::read(collision.0.join("mma16p1tobit.csv")).unwrap();
  fs::write(
    collision.0.join("heckman_predictions.csv"),
    b"owned collision sentinel\n",
  )
  .unwrap();
  assert!(!collision.run("--json").status.success());
  assert_eq!(
    fs::read(collision.0.join("heckman_predictions.csv")).unwrap(),
    b"owned collision sentinel\n"
  );
  assert_eq!(
    fs::read(collision.0.join("mma16p1tobit.csv")).unwrap(),
    input
  );
  let missing = Fixture::new();
  fs::remove_file(missing.0.join("mma16p1tobit.csv")).unwrap();
  assert!(!missing.run("--json").status.success());
  assert!(!missing.0.join("heckman_predictions.csv").exists());
  let target = Fixture::new();
  let existing = include_str!("fixtures/heckman_script_data.csv")
    .lines()
    .enumerate()
    .map(|(i, line)| format!("{line},{}\n", if i == 0 { "latent" } else { "0" }))
    .collect::<String>();
  fs::write(target.0.join("mma16p1tobit.csv"), &existing).unwrap();
  assert!(!target.run("--json").status.success());
  assert!(!target.0.join("heckman_predictions.csv").exists());
  assert_eq!(
    fs::read_to_string(target.0.join("mma16p1tobit.csv")).unwrap(),
    existing
  );
}
