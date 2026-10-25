use tabdat_domain::{Command, EstimationFamily, EstimationState, TruncregCommand};
use tabdat_stats::{EstimationProblem, TruncatedGaussianOptions, fit_truncated_gaussian};
#[test]
fn serialized_unsupported_inference_fields_cannot_silently_become_iid() {
  for (name, value) in [
    ("robust", serde_json::json!(true)),
    ("cluster_variable", serde_json::json!("id")),
    ("weights", serde_json::json!([1, 1])),
  ] {
    let mut request = serde_json::json!({"outcome":"y","predictors":["x"],"lower_limit":"0","include_intercept":true});
    request[name] = value;
    assert!(
      serde_json::from_value::<TruncregCommand>(request).is_err(),
      "{name}"
    );
  }
}

#[test]
fn owned_truncated_state_and_command_have_distinct_lossless_wire_tags() {
  let command = Command::Truncreg {
    command: TruncregCommand {
      outcome: "y".into(),
      predictors: vec!["x".into()],
      lower_limit: "-0.6".into(),
      include_intercept: true,
    },
  };
  let value = serde_json::to_value(&command).unwrap();
  assert_eq!(serde_json::from_value::<Command>(value).unwrap(), command);
  let mut y = Vec::new();
  let mut x = Vec::new();
  for line in include_str!("../../tabdat-stats/tests/fixtures/truncated_gaussian_data.csv")
    .lines()
    .skip(1)
  {
    let c: Vec<_> = line.split(',').collect();
    if c[0] == "1" {
      y.push(c[2].parse().unwrap());
      x.push(vec![c[3].parse().unwrap()]);
    }
  }
  let p = EstimationProblem::new("y", vec!["x".into()], y, x).unwrap();
  let state = EstimationState::TruncatedGaussianFit(Box::new(
    fit_truncated_gaussian(&p, -0.6, &TruncatedGaussianOptions::default()).unwrap(),
  ));
  assert_eq!(state.family(), EstimationFamily::TruncatedGaussian);
  assert!(state.linear().is_none());
  let value = serde_json::to_value(&state).unwrap();
  assert_eq!(value["family"], "truncated_gaussian_fit");
  assert_eq!(
    serde_json::from_value::<EstimationState>(value).unwrap(),
    state
  );
}
