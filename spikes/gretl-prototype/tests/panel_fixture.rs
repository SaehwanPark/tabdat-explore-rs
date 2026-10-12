use tabdat_gretl_spike::{PanelEstimator, PanelProblem, run_panel};
fn fixture() -> PanelProblem {
  let rows: Vec<Vec<f64>> = include_str!("../fixtures/panel.csv")
    .lines()
    .skip(1)
    .map(|s| s.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  PanelProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    periods: 8,
    keys: rows
      .iter()
      .map(|r| (r[0] as usize, r[1] as usize))
      .collect(),
    data: rows.iter().map(|r| r[2..].to_vec()).collect(),
  }
}
#[test]
fn panel_matches_python_and_r() {
  let problem = fixture();
  for (family, estimator) in [
    ("fe", PanelEstimator::FixedEffects),
    ("re", PanelEstimator::RandomEffects),
  ] {
    let result = run_panel(&problem, estimator).unwrap();
    assert_eq!(result.sample_rows, (0..192).collect::<Vec<_>>());
    assert_eq!(result.keys, problem.keys);
    let names: Vec<_> = result
      .coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect();
    assert_eq!(
      names,
      if family == "fe" {
        vec!["x1", "x2"]
      } else {
        vec!["intercept", "x1", "x2"]
      }
    );
    let values: Vec<_> = result
      .coefficients
      .iter()
      .map(|c| c.value)
      .chain(result.coefficients.iter().map(|c| c.standard_error))
      .chain(result.covariance.iter().flatten().copied())
      .collect();
    for line in include_str!("../fixtures/panel-reference.csv")
      .lines()
      .filter(|line| line.split(',').nth(1) == Some(family))
    {
      let reference: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|v| v.parse().unwrap())
        .collect();
      assert_eq!(values.len(), reference.len());
      for (i, (actual, expected)) in values.iter().zip(reference).enumerate() {
        assert!(
          (actual - expected).abs() <= 1e-9 + 1e-8 * expected.abs(),
          "{family} {i}: {actual} != {expected}"
        );
      }
    }
  }
}
#[test]
fn panel_invalid_and_native_failure_recover() {
  let base = fixture();
  let mut bad = base.clone();
  bad.keys[1] = (0, 0);
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  bad.keys.pop();
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  bad.predictors[1] = bad.predictors[0].clone();
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  bad.periods = 0;
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  bad.data[0].pop();
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  bad.data[0][0] = f64::NAN;
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = row[1];
  }
  assert!(run_panel(&bad, PanelEstimator::FixedEffects).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = 1.0;
  }
  for estimator in [PanelEstimator::FixedEffects, PanelEstimator::RandomEffects] {
    assert!(run_panel(&bad, estimator).is_err());
    run_panel(&base, estimator).unwrap();
  }
}
