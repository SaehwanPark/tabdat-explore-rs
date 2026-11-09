#![forbid(unsafe_code)]

//! End-to-end `regress` factor-design contract against an independent R control.
//!
//! The fixture is a small synthetic sample (not book data). Regenerate both
//! fixture files with `Rscript tools/capture_regress_factor_reference.R DIR`
//! then copy `input.csv` and `regress_factor_reference.json` here; the control
//! fits the same models with `lm` treatment contrasts and the output is
//! deterministic. Assertions cover the automatic and explicit reference,
//! text/integer/boolean factors, `noconstant`, retained design metadata,
//! current-data prediction with NULL policy, and failure paths.

mod common;
use common::TestDir;
use serde_json::Value;
use std::{collections::BTreeMap, fs};

const INPUT: &str = include_str!("fixtures/regress_factor/input.csv");
const REFERENCE: &str = include_str!("fixtures/regress_factor/reference.json");

fn run(dir: &TestDir, commands: &[&str]) -> std::process::Output {
  run_mode(dir, "--json", commands)
}

fn run_mode(dir: &TestDir, mode: &str, commands: &[&str]) -> std::process::Output {
  let mut process = common::tabdat_bin();
  process.current_dir(dir.path()).arg(mode);
  for command in commands {
    process.args(["-c", command]);
  }
  process.output().unwrap()
}

fn envelopes(output: &std::process::Output) -> Vec<Value> {
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

/// Reference rows for one case, in command parameter order.
fn case<'a>(reference: &'a Value, name: &str) -> Vec<BTreeMap<&'a str, &'a Value>> {
  let array = |key: &str| reference[key].as_array().unwrap();
  let cases = array("cases");
  (0..cases.len())
    .filter(|index| cases[*index].as_str() == Some(name))
    .map(|index| {
      [
        "parameters",
        "coefficients",
        "standard_errors",
        "p_values",
        "residual_degrees_of_freedom",
        "r_squared",
        "observations",
      ]
      .into_iter()
      .map(|key| (key, &array(key)[index]))
      .collect()
    })
    .collect()
}

fn close(actual: f64, expected: f64, context: &str) {
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{context}: {actual} != {expected} (tolerance {tolerance})"
  );
}

#[test]
fn factor_estimates_match_independent_r_reference() {
  let reference: Value = serde_json::from_str(REFERENCE).unwrap();
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  for (name, command) in [
    ("i.region", "regress y x i.region"),
    ("ib(West).region", "regress y x ib(West).region"),
    ("i.code", "regress y x i.code"),
    ("i.flag", "regress y x i.flag"),
    ("noconstant", "regress y x i.region, noconstant"),
  ] {
    let rows = case(&reference, name);
    let output = envelopes(&run(&dir, &["use factor.csv", command]));
    assert_eq!(output.len(), 2, "{name}");
    let data = &output[1]["data"];
    let coefficients = data["coefficients"].as_array().unwrap();
    assert_eq!(coefficients.len(), rows.len(), "{name}: parameter count");
    assert_eq!(
      data["observation_count"].as_u64().unwrap() as usize,
      rows[0]["observations"].as_u64().unwrap() as usize,
      "{name}: N"
    );
    assert_eq!(
      data["df_residual"].as_u64().unwrap() as usize,
      rows[0]["residual_degrees_of_freedom"].as_u64().unwrap() as usize,
      "{name}: residual df"
    );
    close(
      data["r_squared"].as_f64().unwrap(),
      rows[0]["r_squared"].as_f64().unwrap(),
      &format!("{name}: R-squared"),
    );
    for (index, row) in rows.iter().enumerate() {
      let coefficient = &coefficients[index];
      // R's `(Intercept)` is the installed command's `intercept`.
      let expected_name = match row["parameters"].as_str().unwrap() {
        "(Intercept)" => "intercept",
        "regionNorth" => "region_North",
        "regionWest" => "region_West",
        "regionEast" => "region_East",
        "flagTRUE" => "flag_true",
        "factor(code)10" => "code_10",
        other => other,
      };
      assert_eq!(
        coefficient["name"].as_str().unwrap(),
        expected_name,
        "{name}: parameter {index}"
      );
      for (field, key) in [
        ("value", "coefficients"),
        ("standard_error", "standard_errors"),
        ("p_value", "p_values"),
      ] {
        close(
          coefficient[field].as_f64().unwrap(),
          row[key].as_f64().unwrap(),
          &format!("{name}: {expected_name} {field}"),
        );
      }
    }
  }
}

#[test]
fn retained_design_reports_reference_levels_and_predicts_current_data() {
  let reference: Value = serde_json::from_str(REFERENCE).unwrap();
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  let output = envelopes(&run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      // Replace row 11's missing region with a level that was not estimated,
      // so both NULL cases are exercised in one current-data prediction.
      "replace region = 'South' if x == 11",
      "predict fitted",
      "select x region fitted",
      "head 12",
    ],
  ));
  let data = &output[1]["data"];
  assert_eq!(
    data["design_terms"],
    serde_json::json!([
      {"kind": "continuous", "name": "x"},
      {
        "kind": "factor",
        "variable": "region",
        "reference": "East",
        "levels": [
          {"kind": "text", "label": "East"},
          {"kind": "text", "label": "North"},
          {"kind": "text", "label": "West"}
        ]
      }
    ])
  );
  // Expanded parameter names are the fitted columns, not the source variables.
  assert_eq!(
    data["predictors"],
    serde_json::json!(["x", "region_North", "region_West"])
  );
  let predictions = reference["predictions"].as_array().unwrap();
  let rows = output[5]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), predictions.len());
  // Row 11 held the only missing region and now holds an unestimated level.
  assert!(rows[10][2].is_null(), "unseen level must predict NULL");
  for (index, row) in rows.iter().enumerate() {
    if index == 10 {
      continue;
    }
    close(
      row[2].as_f64().unwrap(),
      predictions[index].as_f64().unwrap(),
      &format!("prediction row {index}"),
    );
  }
}

#[test]
fn stored_factor_reports_keep_design_metadata_and_comparison_names() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  let commands = [
    "use factor.csv",
    "regress y x i.region",
    "estimates store factored",
    "regress y x",
    "estimates store plain",
    "estimates table factored plain",
  ];
  let output = envelopes(&run(&dir, &commands));
  assert_eq!(output.last().unwrap()["result_type"], "EstimateTableResult");
  let table = run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      "estimates store factored",
      "regress y x",
      "estimates store plain",
      "estimates table factored plain",
    ],
  );
  assert!(
    table.status.success(),
    "{}",
    String::from_utf8_lossy(&table.stderr)
  );
  let table = envelopes_failure(&table);
  let table = &table.last().unwrap()["data"];
  assert_eq!(
    table["parameter_names"],
    serde_json::json!(["intercept", "x", "region_North", "region_West"])
  );
  // The stored factor report keeps its design terms; the continuous-only report
  // keeps the established shape and omits the field entirely.
  assert_eq!(
    table["models"][0]["linear_design_terms"],
    serde_json::json!([
      {"kind": "continuous", "name": "x"},
      {
        "kind": "factor",
        "variable": "region",
        "reference": "East",
        "levels": [
          {"kind": "text", "label": "East"},
          {"kind": "text", "label": "North"},
          {"kind": "text", "label": "West"}
        ]
      }
    ])
  );
  assert!(table["models"][1].get("linear_design_terms").is_none());
  // Parameter rows come from the stored reports, not from a refit.
  let text = run_mode(
    &dir,
    "--text",
    &[
      "use factor.csv",
      "regress y x i.region",
      "estimates store factored",
      "estimates table factored",
    ],
  );
  assert!(
    text.status.success(),
    "{}",
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(text.contains("region_North coefficient"), "{text}");
  assert!(text.contains("region=East of 3"), "{text}");
}

#[test]
fn factor_failures_are_explicit_and_leave_the_prior_fit_cleared() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  for (command, expected) in [
    (
      "regress y x ib(Absent).region",
      "is not present in the estimation sample",
    ),
    (
      "regress y x ib(East).region, noconstant",
      "cannot use ib(level) with noconstant",
    ),
  ] {
    let output = run(&dir, &["use factor.csv", command]);
    assert_eq!(output.status.code(), Some(1), "{command}");
    let envelopes = envelopes_failure(&output);
    let message = envelopes.last().unwrap()["error"]["message"]
      .as_str()
      .unwrap();
    assert!(message.contains(expected), "{command}: {message}");
  }
  // A factor with one retained level cannot identify a contrast.
  fs::write(
    dir.path().join("one_level.csv"),
    "y,x,region\n1,1,A\n2,2,A\n",
  )
  .unwrap();
  let output = run(&dir, &["use one_level.csv", "regress y x i.region"]);
  assert_eq!(output.status.code(), Some(1));
  let envelopes = envelopes_failure(&output);
  assert!(
    envelopes.last().unwrap()["error"]["message"]
      .as_str()
      .unwrap()
      .contains("only one level")
  );
  // A failed attempt clears the cached model rather than leaving stale reports,
  // and the batch stops at the first failing command.
  let output = run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      "regress y x ib(Absent).region",
      "predict fitted",
    ],
  );
  assert_eq!(output.status.code(), Some(1));
  let envelopes = envelopes_failure(&output);
  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[1]["result_type"], "RegressionResult");
  assert_eq!(envelopes[2]["error"]["type"], "ExecutionError");
}

#[test]
fn factor_designs_support_vif_lincom_test_and_cluster_inference() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  let output = envelopes(&run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region, cluster(code)",
      // Retained design columns, not source variables, drive post-estimation.
      "estat vif",
      "lincom region_North - region_West",
      "test x",
    ],
  ));
  assert_eq!(output.len(), 5);
  assert_eq!(output[2]["result_type"], "TableResult");
  // VIF is reported over the retained expanded columns.
  let vif = &output[2]["data"]["rows"];
  assert_eq!(vif.as_array().unwrap().len(), 4);
  assert_eq!(vif[0][0], "x");
  assert_eq!(vif[1][0], "region_North");
  assert_eq!(vif[2][0], "region_West");
  assert_eq!(output[3]["result_type"], "LincomResult");
  assert!(output[3]["data"]["estimate"].is_number());
  assert_eq!(output[4]["result_type"], "TestResult");
  // Cluster inference uses the requested grouping on the factor design.
  assert_eq!(output[1]["data"]["covariance"], "cluster(code)");
  assert_eq!(
    output[1]["data"]["covariance_parameter_names"],
    serde_json::json!(["intercept", "x", "region_North", "region_West"])
  );
}

#[test]
fn weighted_and_gls_factor_fits_keep_the_same_expanded_design() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  for command in [
    "regress y x i.region, wls(w)",
    "regress y x i.region, gls(w)",
    "regress y x i.region, robust",
  ] {
    let output = envelopes(&run(
      &dir,
      &[
        "use factor.csv",
        "generate w = 1",
        command,
        "predict fitted",
        "count",
      ],
    ));
    assert_eq!(
      output[2]["data"]["predictors"],
      serde_json::json!(["x", "region_North", "region_West"]),
      "{command}"
    );
    assert_eq!(output[3]["result_type"], "PredictionResult", "{command}");
  }
}

#[test]
fn noconstant_omits_the_reference_field_instead_of_naming_an_empty_level() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  let output = envelopes(&run(
    &dir,
    &["use factor.csv", "regress y x i.region, noconstant"],
  ));
  let data = &output[1]["data"];
  assert_eq!(
    data["predictors"],
    serde_json::json!(["x", "region_East", "region_North", "region_West"])
  );
  // `reference` absent means "no level omitted"; it is never an empty string,
  // which would collide with a literal empty-string level.
  assert_eq!(
    data["design_terms"][1],
    serde_json::json!({
      "kind": "factor",
      "variable": "region",
      "levels": [
        {"kind": "text", "label": "East"},
        {"kind": "text", "label": "North"},
        {"kind": "text", "label": "West"}
      ]
    })
  );
  let text = run_mode(
    &dir,
    "--text",
    &["use factor.csv", "regress y x i.region, noconstant"],
  );
  let text = String::from_utf8(text.stdout).unwrap();
  assert!(
    text.contains("none (noconstant: every level estimated)"),
    "{text}"
  );
}

#[test]
fn factor_terms_stay_out_of_other_models_and_bootstrap() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  for command in [
    "summarize i.region",
    "logit flag i.region",
    "probit flag i.region",
    "poisson code i.region",
    "bootstrap y x i.region, seed(7)",
    "qreg y x i.region",
    "xtreg y x i.region, fe id(code)",
  ] {
    let output = run(&dir, &["use factor.csv", command]);
    assert!(!output.status.success(), "{command} must stay unsupported");
    // Parser rejections carry no execution envelope; execution refusals do.
    if output.status.code() == Some(1) {
      let envelopes = envelopes_failure(&output);
      assert!(
        envelopes.last().unwrap().get("error").is_some(),
        "{command}"
      );
    }
  }
}

#[test]
fn factor_prediction_is_null_safe_without_mutating_the_fit() {
  let dir = TestDir::new();
  fs::write(dir.path().join("factor.csv"), INPUT).unwrap();
  let output = envelopes(&run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      "predict fitted",
      "predict again",
      "status",
    ],
  ));
  assert_eq!(output[2]["result_type"], "PredictionResult");
  assert_eq!(output[3]["result_type"], "PredictionResult");
  let status = &output[4]["data"]["linear_model"];
  assert_eq!(status["outcome"], "y");
  assert_eq!(
    status["predictors"],
    serde_json::json!(["x", "region_North", "region_West"])
  );
  // Re-predicting from the same retained fit is deterministic.
  let first = envelopes(&run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      "predict a",
      "select x a",
      "head 12",
    ],
  ));
  let second = envelopes(&run(
    &dir,
    &[
      "use factor.csv",
      "regress y x i.region",
      "predict b",
      "select x b",
      "head 12",
    ],
  ));
  let rows = |value: &Value| {
    value["data"]["rows"]
      .as_array()
      .unwrap()
      .iter()
      .map(|row| row[1].clone())
      .collect::<Vec<_>>()
  };
  assert_eq!(rows(&first[4]), rows(&second[4]));
}

fn envelopes_failure(output: &std::process::Output) -> Vec<Value> {
  String::from_utf8(output.stdout.clone())
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
