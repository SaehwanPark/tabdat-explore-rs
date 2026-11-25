mod common;

use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::{fs, process::Output};

struct Fixture(TestDir);

impl Fixture {
  fn new() -> Self {
    let dir = TestDir::new();
    fs::write(
      dir.path().join("binary_factor_data.csv"),
      include_bytes!("../examples/binary-factor-design/binary_factor_data.csv"),
    )
    .unwrap();
    fs::write(
      dir.path().join("analysis.td"),
      "use binary_factor_data.csv\nlogit y x i.region\nlincom x + region_North - region_West + 0.5\nestat margins\nestat gof, group(10)\nestat classification\nestat classification, cutoff(.35)\nestat classification, cutoff(1)\nestat margins, factor(region)\npredict index, xb\npredict probability, pr\nestimates store factors\nestimates table factors\nexport predictions.csv, replace\n",
    )
    .unwrap();
    Self(dir)
  }

  fn run(&self, mode: &str) -> Output {
    tabdat_bin()
      .current_dir(self.0.path())
      .args([mode, "-f", "analysis.td"])
      .output()
      .unwrap()
  }
}

fn envelopes(output: &Output) -> Vec<Value> {
  assert!(output.status.success(), "{output:?}");
  assert!(output.stderr.is_empty(), "{output:?}");
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn binary_factor_fit_diagnostic_prediction_and_report_run_in_cli_json_and_text() {
  let fixture = Fixture::new();
  let rows = envelopes(&fixture.run("--json"));
  let fit = rows
    .iter()
    .find(|row| row["result_type"] == "BinaryLogitResult")
    .expect("binary logit result");
  assert_eq!(fit["error"], Value::Null);
  assert_eq!(fit["data"]["observation_count"], 237);
  assert_eq!(fit["data"]["event_count"], 110);
  assert_eq!(fit["data"]["predictors"][1], "region_North");
  assert_eq!(fit["data"]["predictors"][2], "region_West");

  let lincom = rows
    .iter()
    .find(|row| row["result_type"] == "LincomResult")
    .expect("binary linear combination");
  assert_eq!(lincom["data"]["df_residual"], Value::Null);
  assert_eq!(lincom["data"]["ci_level"], 95.0);
  assert!(
    lincom["data"]["label"]
      .as_str()
      .unwrap()
      .contains("region_North")
  );
  assert!(
    lincom["data"]["label"]
      .as_str()
      .unwrap()
      .contains("region_West")
  );
  assert!(
    lincom["data"]["standard_error"]
      .as_f64()
      .unwrap()
      .is_finite()
  );
  assert!(lincom["data"]["statistic"].as_f64().unwrap().is_finite());

  let gof = rows
    .iter()
    .find(|row| row["result_type"] == "BinaryGoodnessOfFitResult")
    .expect("grouped binary diagnostic");
  assert_eq!(gof["data"]["group_count"], 10);

  let classifications = rows
    .iter()
    .filter(|row| row["result_type"] == "BinaryClassificationResult")
    .collect::<Vec<_>>();
  assert_eq!(classifications.len(), 3);
  assert_eq!(classifications[0]["data"]["estimator"], "logit");
  assert_eq!(
    classifications[0]["data"]["sample_role"],
    "retained_estimation_sample"
  );
  assert_eq!(classifications[0]["data"]["observation_count"], 237);
  assert_eq!(classifications[0]["data"]["cutoff"], 0.5);
  assert!(
    classifications[0]["data"]["confusion"]["true_positive"]
      .as_u64()
      .is_some()
  );
  assert_eq!(
    classifications[0]["data"]["metric_units"],
    "percent_0_to_100"
  );
  assert_eq!(classifications[0]["data"]["uncertainty"], "not_computed");
  assert_eq!(classifications[1]["data"]["cutoff"], 0.35);
  assert_eq!(classifications[2]["data"]["cutoff"], 1.0);
  assert_eq!(
    classifications[2]["data"]["confusion"]["predicted_positive_count"],
    0
  );
  assert_eq!(
    classifications[2]["data"]["metrics_percent"]["positive_predictive_value"],
    Value::Null
  );

  let margins = rows
    .iter()
    .find(|row| row["result_type"] == "LogitMarginalEffectsResult")
    .expect("continuous effects after a factor design");
  assert_eq!(margins["data"]["observation_count"], 237);
  assert_eq!(margins["data"]["effects"].as_array().unwrap().len(), 1);
  assert_eq!(margins["data"]["effects"][0]["name"], "x");

  let contrasts = rows
    .iter()
    .find(|row| row["result_type"] == "BinaryFactorContrastsResult")
    .expect("binary factor probability contrasts");
  assert_eq!(contrasts["data"]["factor_variable"], "region");
  assert_eq!(contrasts["data"]["baseline_level"], "East");
  assert_eq!(contrasts["data"]["observation_count"], 237);
  assert_eq!(contrasts["data"]["causal_interpretation"], false);
  assert_eq!(contrasts["data"]["contrasts"].as_array().unwrap().len(), 2);
  assert_eq!(contrasts["data"]["contrasts"][0]["level"], "North");

  let table = rows
    .iter()
    .find(|row| row["result_type"] == "EstimateTableResult")
    .expect("stored estimate table");
  let terms = table["data"]["models"][0]["linear_design_terms"]
    .as_array()
    .expect("retained factor metadata");
  assert_eq!(terms.len(), 2);
  assert_eq!(terms[0]["kind"], "continuous");
  assert_eq!(terms[1]["kind"], "factor");
  assert_eq!(terms[1]["reference"], "East");

  let predictions = fs::read_to_string(fixture.0.path().join("predictions.csv")).unwrap();
  assert_eq!(predictions.lines().count(), 241);
  let header = predictions.lines().next().unwrap();
  assert!(header.contains("index") && header.contains("probability"));
  assert!(predictions.lines().nth(5).unwrap().ends_with(",,"));
  assert!(predictions.lines().nth(12).unwrap().ends_with(",,"));

  let text = fixture.run("--text");
  assert!(text.status.success(), "{text:?}");
  assert!(text.stderr.is_empty(), "{text:?}");
  let text = String::from_utf8_lossy(&text.stdout);
  assert!(text.contains("region_North"));
  assert!(text.contains("East"));
  assert!(text.contains("Logit continuous average partial effects"));
  assert!(text.contains("Original continuous predictor derivatives only"));
  assert!(text.contains("Logit average factor probability contrasts"));
  assert!(text.contains("region: North vs East"));
  assert!(text.contains("Hosmer-Lemeshow grouped goodness-of-fit test"));
  assert!(text.contains("Logit retained-sample classification"));
  assert!(text.contains("Positive predictive value"));
  assert!(text.contains("cutoff = 0.35"));
  assert!(text.contains("cutoff = 1"));
  assert!(text.contains("Positive predictive value (TP / predicted positive)\tNA"));
  assert!(text.contains("Linear combination:"));
  assert!(text.contains("asymptotic standard-normal reference; residual df = NA"));
}

#[test]
fn probit_script_reports_continuous_margins_for_a_factor_design() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.path().join("analysis.td"),
    "use binary_factor_data.csv\nprobit y x i.region\nestat margins\n",
  )
  .unwrap();
  let rows = envelopes(&fixture.run("--json"));
  let margins = rows
    .iter()
    .find(|row| row["result_type"] == "ProbitMarginalEffectsResult")
    .expect("probit continuous effects after a factor design");
  assert_eq!(margins["data"]["observation_count"], 237);
  assert_eq!(margins["data"]["effects"].as_array().unwrap().len(), 1);
  assert_eq!(margins["data"]["effects"][0]["name"], "x");
}

#[test]
fn discrete_changes_run_for_a_mixed_factor_design_in_cli_json_and_text() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.path().join("analysis.td"),
    "use binary_factor_data.csv\ngenerate treated = 0\nreplace treated = 1 if x > 0\nlogit y treated x i.region\nestat margins, discrete(treated)\n",
  )
  .unwrap();

  let rows = envelopes(&fixture.run("--json"));
  let change = rows
    .iter()
    .find(|row| row["result_type"] == "BinaryDiscreteChangeResult")
    .expect("discrete change after an additive factor fit");
  assert_eq!(change["error"], Value::Null);
  assert_eq!(change["data"]["predictor"], "treated");
  assert_eq!(change["data"]["from_value"], 0);
  assert_eq!(change["data"]["to_value"], 1);
  assert_eq!(change["data"]["observation_count"], 237);
  assert_eq!(
    change["data"]["effect"],
    "average_discrete_probability_change"
  );
  assert!(change["data"]["effect_estimate"]["standard_error"].is_number());

  let text = fixture.run("--text");
  assert!(text.status.success(), "{text:?}");
  assert!(text.stderr.is_empty(), "{text:?}");
  let text = String::from_utf8_lossy(&text.stdout);
  assert!(text.contains("Logit discrete average probability change"));
  assert!(text.contains("Predictor treated set from 0 to 1"));
  assert!(text.contains("other predictors stay at their observed retained-row values"));
}
