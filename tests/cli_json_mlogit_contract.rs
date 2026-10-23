use serde_json::Value;
use std::{
  fmt::Write,
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
      "tabdat-mlogit-cli-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = String::from("choice,x\n");
    for (g, counts) in [[1, 1, 6], [2, 4, 6], [4, 16, 6]].iter().enumerate() {
      for (j, &count) in counts.iter().enumerate() {
        for _ in 0..count {
          writeln!(csv, "{},{}", j + 1, g as i32 - 1).unwrap();
        }
      }
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    fs::write(root.join("model.td"),"use data.csv\nmlogit choice x, baseoutcome(1)\nestat margins\npredict p, pr outcome(1)\npredict base, xb outcome(1)\nexport predictions.csv\n").unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
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
fn script_json_and_text_report_joint_multinomial_inference_and_scalar_predictions() {
  let f = Fixture::new();
  let out = f.run(&["--json", "-f", "model.td"]);
  assert!(out.status.success(), "{out:?}");
  let records = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  let fit = &records
    .iter()
    .find(|r| r["result_type"] == "MultinomialLogitResult")
    .unwrap()["data"];
  assert_eq!(fit["estimator"], "mlogit");
  assert_eq!(fit["category_levels"], serde_json::json!([1, 2, 3]));
  assert_eq!(fit["category_counts"], serde_json::json!([7, 21, 18]));
  assert_eq!(fit["base_outcome"], 1);
  assert_eq!(fit["covariance"], "model_based_joint_fisher_information");
  assert_eq!(fit["covariance_correction"], 1.0);
  assert_eq!(fit["reference_distribution"], "asymptotic_standard_normal");
  assert_eq!(fit["likelihood_ratio_df"], 2);
  assert_eq!(fit["covariance_matrix"].as_array().unwrap().len(), 4);
  assert!((fit["log_likelihood"].as_f64().unwrap() + 42.0751828674327).abs() < 1e-10);
  let effects = &records
    .iter()
    .find(|r| r["result_type"] == "MultinomialMarginalEffectsResult")
    .unwrap()["data"];
  assert_eq!(effects["sample"], "retained_estimation_design");
  assert_eq!(
    effects["uncertainty"],
    "conditional_on_observed_design_delta_method"
  );
  assert_eq!(effects["effects"].as_array().unwrap().len(), 3);
  let sum = effects["effects"]
    .as_array()
    .unwrap()
    .iter()
    .map(|e| e["value"].as_f64().unwrap())
    .sum::<f64>();
  assert!(sum.abs() < 1e-14);
  assert_eq!(
    records
      .iter()
      .filter(|r| r["result_type"] == "PredictionResult")
      .count(),
    2
  );
  let exported = fs::read(f.0.join("predictions.csv")).unwrap();
  assert_eq!(
    String::from_utf8(exported.clone()).unwrap().lines().count(),
    47
  );
  let collision = f.run(&["--json", "-f", "model.td"]);
  assert!(!collision.status.success());
  assert_eq!(fs::read(f.0.join("predictions.csv")).unwrap(), exported);
  let text = f.run(&[
    "--text",
    "-c",
    "use data.csv",
    "-c",
    "mlogit choice x",
    "-c",
    "estat margins",
  ]);
  assert!(text.status.success());
  let text = String::from_utf8(text.stdout).unwrap();
  for fragment in [
    "Multinomial logit maximum likelihood",
    "base outcome = 1",
    "joint Fisher information",
    "Category\tVariable",
    "conditional on observed estimation X",
  ] {
    assert!(text.contains(fragment), "{fragment}: {text}");
  }
}

#[test]
fn invalid_category_prediction_and_intercept_only_conventions_are_explicit() {
  let f = Fixture::new();
  let invalid = f.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "mlogit choice x",
    "-c",
    "predict bad, pr outcome(9)",
  ]);
  assert!(!invalid.status.success());
  let diagnostic = format!(
    "{}{}",
    String::from_utf8_lossy(&invalid.stdout),
    String::from_utf8_lossy(&invalid.stderr)
  );
  assert!(diagnostic.contains("absent from the retained"));
  assert!(!diagnostic.contains("PredictionResult"));
  let out = f.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "mlogit choice",
    "-c",
    "estat margins",
    "-c",
    "predict p, pr outcome(2)",
  ]);
  assert!(out.status.success(), "{out:?}");
  let rows = String::from_utf8(out.stdout)
    .unwrap()
    .lines()
    .map(|l| serde_json::from_str::<Value>(l).unwrap())
    .collect::<Vec<_>>();
  let fit = &rows
    .iter()
    .find(|r| r["result_type"] == "MultinomialLogitResult")
    .unwrap()["data"];
  assert_eq!(fit["likelihood_ratio_df"], 0);
  assert!(fit["likelihood_ratio_p_value"].is_null());
  let effects = &rows
    .iter()
    .find(|r| r["result_type"] == "MultinomialMarginalEffectsResult")
    .unwrap()["data"];
  assert!(effects["effects"].as_array().unwrap().is_empty());
}
