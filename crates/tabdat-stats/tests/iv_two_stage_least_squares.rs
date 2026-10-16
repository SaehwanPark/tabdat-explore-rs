use tabdat_stats::{
  CovarianceType, EstimationSample, InstrumentalVariablesProblem, IvOptions, StatsError,
  fit_two_stage_least_squares,
};

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() < 1e-8,
    "expected {expected:.12}, got {actual:.12}"
  );
}

fn reference_problem() -> InstrumentalVariablesProblem {
  let n = 60;
  let mut exogenous = Vec::with_capacity(n);
  let mut endogenous = Vec::with_capacity(n);
  let mut instruments = Vec::with_capacity(n);
  let mut response = Vec::with_capacity(n);
  let mut cluster_groups = Vec::with_capacity(n);

  for index in 1..=n {
    let i = index as f64;
    let w = (i / 7.0).sin() + i / 90.0;
    let z1 = (i / 5.0).cos() + i / 70.0;
    let z2 = (i / 11.0).sin() - (i / 13.0).cos() + i / 200.0;
    let v1 = (i / 3.0).sin() + (i / 9.0).cos();
    let v2 = (i / 4.0).cos() - (i / 6.0).sin();
    let x1 = 0.3 + 0.4 * w + 0.9 * z1 + 0.2 * z2 + v1;
    let x2 = -0.2 + 0.2 * w - 0.3 * z1 + 1.1 * z2 + v2;
    let y =
      1.2 + 0.5 * w + 1.7 * x1 - 0.6 * x2 + (i / 1.7).sin() + (i / 2.3).cos() + (i / 5.9).sin();
    exogenous.push(vec![w]);
    endogenous.push(vec![x1, x2]);
    instruments.push(vec![z1, z2]);
    response.push(y);
    cluster_groups.push(format!("g{}", (index - 1) / 5));
  }

  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x1".into(), "x2".into()],
    instrument_names: vec!["z1".into(), "z2".into()],
    response,
    exogenous,
    endogenous,
    instruments,
    include_intercept: true,
    sample: Some(
      EstimationSample::new((0..n).collect(), n, 0, None, Some(cluster_groups)).unwrap(),
    ),
  }
}

#[test]
fn two_endogenous_regressors_match_independent_r_reference() {
  // Reference: tests/fixtures/iv_two_stage_least_squares_reference.R (R 4.6.1).
  let problem = reference_problem();
  let classical = fit_two_stage_least_squares(&problem, &IvOptions::default()).unwrap();
  assert_eq!(
    classical.regression.parameter_names,
    ["intercept", "w", "x1", "x2"]
  );
  assert_eq!(classical.regression.fit_statistics.observation_count, 60);
  assert_eq!(classical.regression.fit_statistics.degrees_of_freedom, 56);
  for (coefficient, expected) in classical.regression.coefficients.iter().zip([
    1.79978251178494,
    0.49573876410341,
    1.43920781503306,
    -1.39990592983552,
  ]) {
    close(coefficient.value, expected);
  }
  for (coefficient, expected) in classical.regression.coefficients.iter().zip([
    0.33823320989461,
    0.29268754864960,
    0.20893192110474,
    0.27176608904457,
  ]) {
    close(coefficient.standard_error.unwrap(), expected);
  }
  assert_eq!(classical.first_stage.len(), 2);
  for (diagnostic, (expected_f, expected_partial_r2)) in classical.first_stage.iter().zip([
    (56.6351625385778, 0.6691682374068),
    (20.50542024171924, 0.42274492499052),
  ]) {
    assert_eq!(diagnostic.numerator_df, 2);
    assert_eq!(diagnostic.denominator_df, 56);
    close(diagnostic.f_statistic.unwrap(), expected_f);
    close(diagnostic.partial_r_squared.unwrap(), expected_partial_r2);
  }

  let robust = fit_two_stage_least_squares(
    &problem,
    &IvOptions {
      covariance_type: CovarianceType::RobustHc1,
    },
  )
  .unwrap();
  for (coefficient, expected) in robust.regression.coefficients.iter().zip([
    0.34043925118168,
    0.26737827214256,
    0.17393281558833,
    0.26258134979964,
  ]) {
    close(coefficient.standard_error.unwrap(), expected);
  }

  let clustered = fit_two_stage_least_squares(
    &problem,
    &IvOptions {
      covariance_type: CovarianceType::Cluster("group".into()),
    },
  )
  .unwrap();
  for (coefficient, expected) in clustered.regression.coefficients.iter().zip([
    0.57024986793111,
    0.44391982639610,
    0.22706712234890,
    0.43337204386643,
  ]) {
    close(coefficient.standard_error.unwrap(), expected);
  }
}

#[test]
fn two_stage_least_squares_rejects_underidentification_and_rank_failure() {
  let mut underidentified = reference_problem();
  underidentified.instrument_names.pop();
  for row in &mut underidentified.instruments {
    row.pop();
  }
  assert!(matches!(
    fit_two_stage_least_squares(&underidentified, &IvOptions::default()),
    Err(StatsError::Underidentified {
      endogenous_regressors: 2,
      excluded_instruments: 1
    })
  ));

  let mut rank_deficient = reference_problem();
  for row in &mut rank_deficient.instruments {
    row[1] = row[0];
  }
  assert!(matches!(
    fit_two_stage_least_squares(&rank_deficient, &IvOptions::default()),
    Err(StatsError::SingularMatrix(_))
  ));
}

#[test]
fn clustered_two_stage_least_squares_requires_two_sample_clusters() {
  let mut problem = reference_problem();
  let sample = problem.sample.as_mut().unwrap();
  sample.cluster_groups = Some(vec!["one".to_owned(); sample.retained_count()]);
  assert!(matches!(
    fit_two_stage_least_squares(
      &problem,
      &IvOptions {
        covariance_type: CovarianceType::Cluster("group".into()),
      }
    ),
    Err(StatsError::InsufficientObservations(_))
  ));
}
