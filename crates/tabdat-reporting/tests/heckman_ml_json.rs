use serde_json::Value;
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_reporting::{
  catalog::describe_command_result,
  json_results::{serialize_execution_result, supports_batch_command, supports_script_command},
};
use tabdat_stats::{EstimationProblem, EstimationSample, HeckmanMlOptions, fit_heckman_ml};

fn cells(line: &str) -> Vec<&str> {
  line
    .split(',')
    .map(|value| value.trim_matches('"'))
    .collect()
}

#[test]
fn ml_json_envelope_and_default_method_schema_are_explicit() {
  let command = parse_command("heckman y x, selectdep(d) select(z)").unwrap();
  assert!(supports_batch_command(&command));
  assert!(supports_script_command(&command));
  let twostep = parse_command("heckman y x, selectdep(d) select(z) twostep").unwrap();
  assert!(supports_batch_command(&twostep));
  assert!(supports_script_command(&twostep));

  let mut d = Vec::new();
  let mut z = Vec::new();
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut selected = Vec::new();
  for line in include_str!("../../tabdat-stats/tests/fixtures/heckman_data.csv")
    .lines()
    .skip(1)
  {
    let c = cells(line);
    if c[0] != "exclusion" {
      continue;
    }
    let flag = c[2].parse::<f64>().unwrap();
    let covariate = c[4].parse::<f64>().unwrap();
    let index = d.len();
    d.push(flag);
    z.push(vec![covariate, c[5].parse().unwrap()]);
    if flag == 1.0 {
      y.push(c[3].parse().unwrap());
      x.push(vec![covariate]);
      selected.push(index);
    }
  }
  let selection = EstimationProblem::new("d", vec!["x".into(), "z".into()], d, z)
    .unwrap()
    .with_sample(EstimationSample::new((0..360).collect(), 364, 4, None, None).unwrap());
  let outcome = EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_sample(
      EstimationSample::new(selected.clone(), 364, 364 - selected.len(), None, None).unwrap(),
    );
  let fit = fit_heckman_ml(&selection, &outcome, &HeckmanMlOptions::default()).unwrap();
  let encoded = serialize_execution_result(&ExecutionResult::HeckmanMl(Box::new(fit))).unwrap();
  let envelope: Value = serde_json::from_str(&encoded).unwrap();
  assert_eq!(envelope["result_type"], "HeckmanMlResult");
  let data = &envelope["data"];
  assert_eq!(data["estimator"], "heckman");
  assert_eq!(data["method"], "maximum_likelihood");
  assert_eq!(data["covariance"], "model_based_joint_observed_information");
  assert_eq!(data["confidence_level"], 95.0);
  assert_eq!(
    data["samples"]["selection"]["retained_indices"]
      .as_array()
      .unwrap()
      .len(),
    360
  );
  assert_eq!(
    data["samples"]["outcome"]["retained_indices"]
      .as_array()
      .unwrap()
      .len(),
    selected.len()
  );
  assert_eq!(data["optimizer"]["converged"], true);
  assert!(
    data["likelihood"]["independence_lr_chi_square"]
      .as_f64()
      .unwrap()
      > 0.0
  );
  assert_eq!(
    data["covariance_parameter_names"].as_array().unwrap().len(),
    data["covariance_matrix"].as_array().unwrap().len()
  );

  let schema = describe_command_result("heckman").unwrap();
  assert_eq!(
    schema.syntax,
    "heckman y x1 x2, selectdep(var) select(vars) [twostep] [noconstant]"
  );
  assert!(
    !schema
      .options
      .iter()
      .find(|option| option.name == "twostep")
      .unwrap()
      .required
  );
}
