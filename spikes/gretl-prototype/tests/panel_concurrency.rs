use tabdat_gretl_spike::{PanelEstimator, PanelProblem, run_panel};
#[test]
fn panel_calls_serialize_across_threads() {
  let data: Vec<Vec<f64>> = include_str!("../fixtures/panel.csv")
    .lines()
    .skip(1)
    .map(|r| r.split(',').skip(2).map(|v| v.parse().unwrap()).collect())
    .collect();
  let problem = PanelProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    keys: (0..192).map(|i| (i / 8, i % 8)).collect(),
    periods: 8,
    data,
  };
  let expected: Vec<_> = [PanelEstimator::FixedEffects, PanelEstimator::RandomEffects]
    .into_iter()
    .map(|e| run_panel(&problem, e).unwrap().coefficients[0].value)
    .collect();
  std::thread::scope(|scope| {
    for i in 0..4 {
      let problem = &problem;
      let expected = &expected;
      scope.spawn(move || {
        let index = i % 2;
        let estimator = if index == 0 {
          PanelEstimator::FixedEffects
        } else {
          PanelEstimator::RandomEffects
        };
        for _ in 0..8 {
          assert_eq!(
            run_panel(problem, estimator).unwrap().coefficients[0].value,
            expected[index]
          );
        }
      });
    }
  });
}
