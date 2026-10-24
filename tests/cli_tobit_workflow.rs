use serde_json::Value;
use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-tobit-workflow-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    // Closed-form independent Gaussian control, not replacement textbook data:
    // balanced x=-1,0,1 and e=+/-.5, six replicates -> beta=(3,2), sigma=.5.
    let mut csv = String::from("source_row,e,lnwage,ystar,ytrunc,ycens,dy\n");
    let mut id = 0;
    for _ in 0..6 {
      for x in [-1.0, 0.0, 1.0] {
        for e in [-0.5, 0.5] {
          id += 1;
          let y = 3.0 + 2.0 * x + e;
          writeln!(csv, "{id},{e},{x},{y},{y},{y},1").unwrap();
        }
      }
    }
    fs::write(root.join("mma16p1tobit.csv"), csv).unwrap();
    fs::write(
      root.join("tobit.td"),
      include_str!("../examples/mma16p1tobit/tobit.td"),
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args([mode, "-f", "tobit.td"])
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn records(output: &std::process::Output) -> Vec<Value> {
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn value(v: &Value) -> f64 {
  v.as_f64().unwrap()
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9 + 1e-8 * b.abs(), "{a} != {b}");
}
fn phi(z: f64) -> f64 {
  (-z * z / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt()
}
fn p(z: f64) -> f64 {
  // Independent normal CDF constants (R4.6.1 pnorm); no statistics implementation
  // is called by this frontend test to validate its own output.
  match z as i32 {
    2 => 0.977_249_868_051_820_8,
    6 => 0.999_999_999_013_412_3,
    10 => 1.0,
    _ => panic!("unexpected control z={z}"),
  }
}

#[test]
fn actual_tobit_script_checks_closed_form_joint_inference_predictions_and_text_export() {
  let json_fixture = Fixture::new();
  let input = fs::read(json_fixture.0.join("mma16p1tobit.csv")).unwrap();
  let output = json_fixture.run("--json");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let records = records(&output);
  assert_eq!(records.len(), 7);
  let kinds: Vec<_> = records
    .iter()
    .map(|r| r["result_type"].as_str().unwrap())
    .collect();
  assert_eq!(
    kinds,
    [
      "LoadResult",
      "TobitResult",
      "TobitMarginalEffectsResult",
      "PredictionResult",
      "PredictionResult",
      "PredictionResult",
      "ExportResult"
    ]
  );
  assert!(
    records
      .iter()
      .all(|r| r["schema_version"] == 1 && r.get("error").is_none())
  );
  let fit = &records[1]["data"];
  let effect = &records[2]["data"]["effects"][0];
  assert_eq!(fit["observation_count"], 36);
  assert_eq!(fit["left_censored_count"], 0);
  assert_eq!(fit["uncensored_count"], 36);
  assert_eq!(fit["predictors"], serde_json::json!(["lnwage"]));
  assert_eq!(
    fit["covariance_parameter_names"],
    serde_json::json!(["intercept", "lnwage", "lnsigma"])
  );
  assert_eq!(
    fit["sample"]["retained_indices"],
    serde_json::json!((0..36).collect::<Vec<_>>())
  );
  close(value(&fit["coefficients"][0]["value"]), 3.0);
  close(value(&fit["coefficients"][1]["value"]), 2.0);
  close(value(&fit["scale"]["value"]), 0.5);
  let covariance = [0.25 / 36.0, 0.25 / 24.0, 1.0 / 72.0];
  for (i, variance) in covariance.iter().enumerate() {
    for j in 0..3 {
      close(
        value(&fit["covariance_matrix"][i][j]),
        if i == j { *variance } else { 0.0 },
      );
    }
  }
  let gaussian_constant = 0.5 * (1.0 + (2.0 * std::f64::consts::PI).ln());
  let ll = -36.0 * (gaussian_constant + 0.5_f64.ln());
  let null_sigma = (8.0_f64 / 3.0 + 0.25).sqrt();
  let null_ll = -36.0 * (gaussian_constant + null_sigma.ln());
  close(value(&fit["log_likelihood"]), ll);
  close(value(&fit["null_log_likelihood"]), null_ll);
  close(value(&fit["likelihood_ratio"]), 2.0 * (ll - null_ll));
  assert_eq!(fit["likelihood_ratio_df"], 1);
  assert!(value(&fit["likelihood_ratio_p_value"]) > 0.0);
  assert_eq!(
    fit["null_model"],
    "intercept_and_scale_refitted_same_sample_bound"
  );
  let mean_p = (p(2.0) + p(6.0) + p(10.0)) / 3.0;
  let mean_gradient = [
    (phi(2.0) + phi(6.0) + phi(10.0)) / 1.5,
    (-phi(2.0) + phi(10.0)) / 1.5,
    -(2.0 * phi(2.0) + 6.0 * phi(6.0) + 10.0 * phi(10.0)) / 3.0,
  ];
  close(value(&fit["mean_uncensored_probability"]), mean_p);
  for (i, g) in mean_gradient.iter().enumerate() {
    close(value(&fit["mean_probability_gradient"][i]), *g);
  }
  let gradient = [
    2.0 * mean_gradient[0],
    2.0 * mean_gradient[1] + mean_p,
    2.0 * mean_gradient[2],
  ];
  let variance: f64 = gradient
    .iter()
    .zip(covariance)
    .map(|(g, v)| g * g * v)
    .sum();
  close(value(&effect["value"]), 2.0 * mean_p);
  close(value(&effect["standard_error"]), variance.sqrt());
  assert_eq!(
    records[2]["data"]["uncertainty"],
    "conditional_on_observed_design_full_joint_beta_lnsigma_delta_method"
  );
  let csv = fs::read_to_string(json_fixture.0.join("tobit_predictions.csv")).unwrap();
  let mut lines = csv.lines();
  assert_eq!(
    lines.next().unwrap(),
    "source_row,e,lnwage,ystar,ytrunc,ycens,dy,latent,uncensored_probability,observed_mean"
  );
  for (i, line) in lines.enumerate() {
    let row: Vec<f64> = line.split(',').map(|x| x.parse().unwrap()).collect();
    assert_eq!(row.len(), 10);
    assert_eq!(row[0], (i + 1) as f64);
    let mu = 3.0 + 2.0 * row[2];
    let z = mu / 0.5;
    close(row[7], mu);
    close(row[8], p(z));
    close(row[9], mu * p(z) + 0.5 * phi(z));
  }
  assert_eq!(
    fs::read(json_fixture.0.join("mma16p1tobit.csv")).unwrap(),
    input
  );
  let text_fixture = Fixture::new();
  let text = text_fixture.run("--text");
  assert!(text.status.success());
  assert!(text.stderr.is_empty());
  let text = String::from_utf8(text.stdout).unwrap();
  for phrase in [
    "not Student t",
    "No sigma=0 Wald test",
    "Tobit continuous observed-censored-mean average partial effects",
    "retained estimation N = 36",
  ] {
    assert!(text.contains(phrase), "{phrase}");
  }
  assert_eq!(
    fs::read(text_fixture.0.join("tobit_predictions.csv")).unwrap(),
    csv.as_bytes()
  );
}

#[test]
fn actual_tobit_script_preserves_existing_export_and_stops_before_output_on_input_failure() {
  let f = Fixture::new();
  let original = fs::read(f.0.join("mma16p1tobit.csv")).unwrap();
  fs::write(
    f.0.join("tobit_predictions.csv"),
    b"owned collision sentinel\n",
  )
  .unwrap();
  let output = f.run("--json");
  assert!(!output.status.success());
  let stream = records(&output);
  assert_eq!(stream.len(), 7);
  assert!(stream.last().unwrap().get("error").is_some());
  assert_eq!(
    fs::read(f.0.join("tobit_predictions.csv")).unwrap(),
    b"owned collision sentinel\n"
  );
  assert_eq!(fs::read(f.0.join("mma16p1tobit.csv")).unwrap(), original);
  let missing = Fixture::new();
  fs::remove_file(missing.0.join("mma16p1tobit.csv")).unwrap();
  let output = missing.run("--json");
  assert!(!output.status.success());
  assert!(!missing.0.join("tobit_predictions.csv").exists());
  let stream = records(&output);
  assert_eq!(stream.len(), 1);
  assert!(stream[0].get("error").is_some());
}
