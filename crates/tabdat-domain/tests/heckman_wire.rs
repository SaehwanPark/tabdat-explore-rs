use tabdat_domain::{EstimationFamily, EstimationState, HeckmanCommand, ModelState};
use tabdat_stats::{EstimationProblem, EstimationSample, ProbitOptions};

#[test]
fn legacy_heckman_command_without_method_does_not_become_twostep() {
  let value = serde_json::json!({"outcome":"y","predictors":["x"],"selection_dependent":"d","selection_predictors":["z"],"robust":false,"cluster_variable":null,"include_intercept":true});
  let command: HeckmanCommand = serde_json::from_value(value).unwrap();
  assert!(!command.twostep);
  let mut explicit = command.clone();
  explicit.twostep = true;
  assert_eq!(
    serde_json::from_value::<HeckmanCommand>(serde_json::to_value(&explicit).unwrap()).unwrap(),
    explicit
  );
}

#[test]
fn legacy_placeholder_and_owned_twostep_state_remain_distinct_and_lossless() {
  let p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0, 3.0, 2.0, 5.0],
    vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]],
  )
  .unwrap();
  let ls = tabdat_stats::fit_least_squares(&p).unwrap();
  let legacy = EstimationState::Heckman(ModelState {
    model: tabdat_stats::PostEstimationModel::from_least_squares(&ls),
    sample: EstimationSample::all_retained(4),
    outcome: "y".into(),
    predictors: vec!["x".into()],
  });
  let mut d = Vec::new();
  let mut z = Vec::new();
  let mut y = Vec::new();
  let mut x = Vec::new();
  for line in include_str!("../../tabdat-stats/tests/fixtures/heckman_data.csv")
    .lines()
    .skip(1)
  {
    let c: Vec<_> = line.split(',').map(|v| v.trim_matches('"')).collect();
    if c[0] != "exclusion" {
      continue;
    }
    let flag = c[2].parse::<f64>().unwrap();
    let covariate = c[4].parse::<f64>().unwrap();
    d.push(flag);
    z.push(vec![covariate, c[5].parse().unwrap()]);
    if flag == 1.0 {
      y.push(c[3].parse().unwrap());
      x.push(vec![covariate]);
    }
  }
  let selection = EstimationProblem::new("d", vec!["x".into(), "z".into()], d, z).unwrap();
  let outcome = EstimationProblem::new("y", vec!["x".into()], y, x).unwrap();
  let owned = EstimationState::HeckmanTwoStepFit(Box::new(
    tabdat_stats::fit_heckman_two_step(&selection, &outcome, &ProbitOptions::default()).unwrap(),
  ));
  for (state, tag) in [(legacy, "heckman"), (owned, "heckman_two_step_fit")] {
    assert_eq!(state.family(), EstimationFamily::Heckman);
    assert!(state.linear().is_none());
    let value = serde_json::to_value(&state).unwrap();
    assert_eq!(value["family"], tag);
    assert_eq!(
      serde_json::from_value::<EstimationState>(value).unwrap(),
      state
    );
  }
}
