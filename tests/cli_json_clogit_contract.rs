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
    let p = std::env::temp_dir().join(format!(
      "tabdat-clogit-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    let mut csv = String::from("id,alternative,chosen,p,q\n");
    let mut id = 1;
    for (chosen, &count) in [2, 4, 6].iter().enumerate() {
      for _ in 0..count {
        for alt in 0..3 {
          writeln!(
            csv,
            "{id},{},{},{},{}",
            alt + 1,
            u8::from(alt == chosen),
            u8::from(alt == 1),
            u8::from(alt == 2)
          )
          .unwrap();
        }
        id += 1;
      }
    }
    fs::write(p.join("data.csv"), csv).unwrap();
    fs::write(p.join("model.td"),"use data.csv\nisid id alternative\nclogit chosen p q, group(id) alternative(alternative)\npredict baseline, pr\nreplace p = p*2\npredict shifted, pr\nexport predictions.csv\n").unwrap();
    Self(p)
  }
  fn run(&self, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
#[test]
fn script_json_and_terminal_report_decision_level_conditional_inference_and_current_probabilities()
{
  let f = Fixture::new();
  let out = f.run(&["--json", "-f", "model.td"]);
  assert!(out.status.success(), "{out:?}");
  let records = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  assert!(records.iter().all(|r| r["schema_version"] == 1));
  let fit = &records
    .iter()
    .find(|r| r["result_type"] == "ConditionalLogitResult")
    .unwrap()["data"];
  for (key, want) in [
    ("observation_count", 36),
    ("choice_set_count", 12),
    ("choice_set_size", 3),
    ("likelihood_ratio_df", 2),
  ] {
    assert_eq!(fit[key], want);
  }
  assert_eq!(fit["estimator"], "clogit");
  assert_eq!(fit["independent_unit"], "choice_set");
  assert_eq!(
    fit["covariance"],
    "model_based_joint_conditional_fisher_information"
  );
  assert_eq!(fit["covariance_correction"], 1.0);
  assert_eq!(fit["chosen_counts"], serde_json::json!([2, 4, 6]));
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["include_intercept"], false);
  assert_eq!(fit["confidence_level"], 95.0);
  assert!((fit["coefficients"][0]["value"].as_f64().unwrap() - 2.0_f64.ln()).abs() < 1e-9);
  assert!((fit["covariance_matrix"][0][1].as_f64().unwrap() - 0.5).abs() < 1e-9);
  assert_eq!(
    records
      .iter()
      .filter(|r| r["result_type"] == "PredictionResult")
      .count(),
    2
  );
  let exported = fs::read(f.0.join("predictions.csv")).unwrap();
  let contents = String::from_utf8(exported.clone()).unwrap();
  assert_eq!(contents.lines().count(), 37);
  for (index, line) in contents.lines().skip(1).enumerate() {
    let row = line.split(',').collect::<Vec<_>>();
    assert!((row[5].parse::<f64>().unwrap() - [1.0 / 6.0, 1.0 / 3.0, 0.5][index % 3]).abs() < 1e-9);
    assert!((row[6].parse::<f64>().unwrap() - [0.125, 0.5, 0.375][index % 3]).abs() < 1e-9);
  }
  let collision = f.run(&["--json", "-f", "model.td"]);
  assert!(!collision.status.success());
  assert_eq!(fs::read(f.0.join("predictions.csv")).unwrap(), exported);
  let text = f.run(&[
    "--text",
    "-c",
    "use data.csv",
    "-c",
    "clogit chosen p q, group(id) alternative(alternative)",
  ]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  for fragment in [
    "Single-choice conditional logit",
    "independent choice sets = 12",
    "alternative rows = 36",
    "equal-probability null",
    "without N-K/G-K correction",
  ] {
    assert!(text.contains(fragment), "{fragment}: {text}");
  }
  let unsupported = f.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "clogit chosen p q, group(id) alternative(alternative)",
    "-c",
    "estat margins",
  ]);
  assert!(!unsupported.status.success());
  let diagnostics = format!(
    "{}{}",
    String::from_utf8_lossy(&unsupported.stdout),
    String::from_utf8_lossy(&unsupported.stderr)
  );
  assert!(diagnostics.contains("point predictions only"));
}
