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
      "use binary_factor_data.csv\nlogit y x i.region\nestat gof, group(10)\nestat margins, factor(region)\npredict index, xb\npredict probability, pr\nestimates store factors\nestimates table factors\nexport predictions.csv, replace\n",
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

  let gof = rows
    .iter()
    .find(|row| row["result_type"] == "BinaryGoodnessOfFitResult")
    .expect("grouped binary diagnostic");
  assert_eq!(gof["data"]["group_count"], 10);

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
  assert!(text.contains("Logit average factor probability contrasts"));
  assert!(text.contains("region: North vs East"));
  assert!(text.contains("Hosmer-Lemeshow grouped goodness-of-fit test"));
}
