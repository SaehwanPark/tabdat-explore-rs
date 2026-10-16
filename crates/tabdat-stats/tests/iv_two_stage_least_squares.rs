use tabdat_stats::{
  CovarianceType, EstimationSample, InstrumentalVariablesProblem, IvOptions, StatsError,
  anderson_rubin_test, fit_two_stage_least_squares,
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

  for (diagnostic, expected) in classical
    .first_stage
    .iter()
    .zip([0.46058071111269, 0.29097041265132])
  {
    close(diagnostic.shea_partial_r_squared.unwrap(), expected);
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
  for ((classical, robust), clustered) in classical
    .first_stage
    .iter()
    .zip(&robust.first_stage)
    .zip(&clustered.first_stage)
  {
    assert_eq!(
      classical.shea_partial_r_squared,
      robust.shea_partial_r_squared
    );
    assert_eq!(
      classical.shea_partial_r_squared,
      clustered.shea_partial_r_squared
    );
  }
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
fn shea_handles_no_intercept_and_is_independent_of_the_outcome() {
  let mut problem = reference_problem();
  problem.include_intercept = false;
  let fit = fit_two_stage_least_squares(&problem, &IvOptions::default()).unwrap();
  for (diagnostic, expected) in fit
    .first_stage
    .iter()
    .zip([0.63244977946811, 0.41428783098388])
  {
    close(diagnostic.shea_partial_r_squared.unwrap(), expected);
  }
  // Constant/zero outcomes must not turn a design diagnostic into 0/0.
  for outcome in [0.0, 3.0] {
    problem.response.fill(outcome);
    let changed = fit_two_stage_least_squares(&problem, &IvOptions::default()).unwrap();
    for (before, after) in fit.first_stage.iter().zip(changed.first_stage) {
      close(
        after.shea_partial_r_squared.unwrap(),
        before.shea_partial_r_squared.unwrap(),
      );
    }
  }
}

#[test]
fn single_endogenous_shea_equals_ordinary_partial_r_squared() {
  let mut problem = reference_problem();
  problem.endogenous_names.pop();
  for row in &mut problem.endogenous {
    row.pop();
  }
  // Two excluded instruments for one endogenous regressor: overidentified design.
  for intercept in [true, false] {
    problem.include_intercept = intercept;
    let fit = fit_two_stage_least_squares(&problem, &IvOptions::default()).unwrap();
    let diagnostic = &fit.first_stage[0];
    close(
      diagnostic.shea_partial_r_squared.unwrap(),
      diagnostic.partial_r_squared.unwrap(),
    );
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
fn anderson_rubin_matches_independent_r_auxiliary_regressions() {
  let mut problem = reference_problem();
  // R 4.6.1: fixtures/anderson_rubin_reference.R, stats::lm/anova.
  for (null, expected_f, expected_p) in [
    ([1.7, -0.6], 5.4227218199300502, 0.00703483761854517),
    ([0.0, 0.0], 48.5717213299665, 5.84129841873989e-13),
  ] {
    let result = anderson_rubin_test(&problem, &null).unwrap();
    assert_eq!(result.endogenous, ["x1", "x2"]);
    assert_eq!(result.null_values, null);
    assert_eq!(result.observation_count, 60);
    assert_eq!((result.numerator_df, result.denominator_df), (2, 56));
    close(result.statistic, expected_f);
    assert!((result.p_value - expected_p).abs() < expected_p * 1e-8 + 1e-16);
  }
  problem.include_intercept = false;
  let result = anderson_rubin_test(&problem, &[1.7, -0.6]).unwrap();
  close(result.statistic, 5.98024398000216806);
  close(result.p_value, 0.00438858080705134);
  assert_eq!(result.denominator_df, 57);
  problem.exogenous_names.clear();
  for row in &mut problem.exogenous {
    row.clear();
  }
  let result = anderson_rubin_test(&problem, &[1.7, -0.6]).unwrap();
  close(result.statistic, 1.445372093898745);
  close(result.p_value, 0.244019559181223);
  assert_eq!(result.denominator_df, 58);
}

#[test]
fn anderson_rubin_does_not_divide_by_first_stage_relevance() {
  let mut problem = reference_problem();
  let baseline = anderson_rubin_test(&problem, &[0.0, 0.0]).unwrap();
  for strength in [1e-8, 0.0] {
    for row in &mut problem.endogenous {
      for value in row {
        *value *= strength;
      }
    }
    let result = anderson_rubin_test(&problem, &[0.0, 0.0]).unwrap();
    assert_eq!(result, baseline);
  }
}

#[test]
fn anderson_rubin_rejects_invalid_nulls_and_undefined_auxiliary_variance() {
  let mut problem = reference_problem();
  assert!(anderson_rubin_test(&problem, &[0.0]).is_err());
  assert!(anderson_rubin_test(&problem, &[f64::NAN, 0.0]).is_err());
  assert!(anderson_rubin_test(&problem, &[f64::MAX, f64::MAX]).is_err());
  for row in &mut problem.instruments {
    row[1] = row[0];
  }
  assert!(matches!(
    anderson_rubin_test(&problem, &[0.0, 0.0]),
    Err(StatsError::SingularMatrix(_))
  ));
  problem = reference_problem();
  problem.sample = None;
  problem.response.truncate(5);
  problem.exogenous.truncate(5);
  problem.endogenous.truncate(5);
  problem.instruments.truncate(5);
  problem.instrument_names.push("z3".into());
  for (index, row) in problem.instruments.iter_mut().enumerate() {
    row.push((index as f64).powi(2));
  }
  assert!(matches!(
    anderson_rubin_test(&problem, &[0.0, 0.0]),
    Err(StatsError::ZeroDegreesOfFreedom { .. })
  ));
  problem = reference_problem();
  problem.response.fill(0.0);
  assert!(matches!(
    anderson_rubin_test(&problem, &[0.0, 0.0]),
    Err(StatsError::IncompatibleHypothesis(_))
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
