use tabdat_stats::{
  CovarianceMatrix, CovarianceType, PostEstimationModel, StatsError, student_t_pvalue,
  student_t_quantile,
};

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.16e} differs from expected {expected:.16e} by more than {tolerance:.3e}"
  );
}

fn model(degrees_of_freedom: usize) -> PostEstimationModel {
  let parameter_names = vec!["intercept".into(), "x1".into(), "x2".into()];
  let covariance = CovarianceMatrix::new(
    parameter_names.clone(),
    CovarianceType::NonRobust,
    vec![
      vec![1.0, 0.0, 0.0],
      vec![0.0, 2.0, 0.3],
      vec![0.0, 0.3, 1.0],
    ],
  )
  .expect("covariance matrix should be valid");
  PostEstimationModel {
    parameter_names,
    parameters: vec![5.0, 2.0, -1.0],
    covariance,
    degrees_of_freedom,
  }
}

#[test]
fn student_t_quantiles_match_independent_reference_values() {
  close(student_t_quantile(0.975, 3.0), 3.1824463052837055);
  close(student_t_quantile(0.975, 10.0), 2.2281388519649385);
  close(student_t_quantile(0.025, 3.0), -3.1824463052837055);
  close(student_t_pvalue(student_t_quantile(0.975, 3.0), 3.0), 0.05);
  let extreme_tail = student_t_pvalue(1e10, 3.0);
  assert!(extreme_tail > 0.0);
  assert!((extreme_tail / 2.205315581687166e-30 - 1.0).abs() < 1e-12);
  assert_eq!(student_t_quantile(0.5, 3.0), 0.0);
  assert_eq!(student_t_quantile(0.0, 3.0), f64::NEG_INFINITY);
  assert_eq!(student_t_quantile(1.0, 3.0), f64::INFINITY);
  assert!(student_t_quantile(0.975, 0.0).is_nan());
  assert!(student_t_quantile(1.1, 3.0).is_nan());
}

#[test]
fn affine_combination_inference_includes_constant_offset_and_covariance() {
  let result = model(3)
    .linear_combination_with_inference(&[0.0, 1.0, -1.0], 1.0, 95.0)
    .expect("combination inference should succeed");

  assert_eq!(result.degrees_of_freedom, 3);
  assert_eq!(result.confidence_level, 95.0);
  close(result.estimate, 4.0);
  close(result.standard_error, 1.5491933384829668);
  close(
    result.statistic.expect("positive standard error"),
    2.581988897471611,
  );
  close(result.p_value, 0.08163884393695156);
  close(result.ci_lower, -0.9302246162252503);
  close(result.ci_upper, 8.93022461622525);
}

#[test]
fn wald_test_normalizes_small_restriction_covariance_before_inversion() {
  let parameter_names = vec!["intercept".into(), "x1".into(), "x2".into()];
  let covariance = CovarianceMatrix::new(
    parameter_names.clone(),
    CovarianceType::NonRobust,
    vec![
      vec![1e-16, 0.0, 0.0],
      vec![0.0, 1e-16, 0.0],
      vec![0.0, 0.0, 2e-16],
    ],
  )
  .expect("small covariance matrix should be valid");
  let post_estimation = PostEstimationModel {
    parameter_names,
    parameters: vec![0.0, 2e-8, -1e-8],
    covariance,
    degrees_of_freedom: 3,
  };

  let result = post_estimation
    .test_linear_hypothesis(&[vec![0.0, 1.0, -1.0]], &[0.0])
    .expect("small but nonsingular restriction covariance should be invertible");

  close(result.chi2_statistic, 3.0);
  close(result.f_statistic, 3.0);
  assert_eq!(result.df_num, 1);
  assert_eq!(result.df_denom, 3);
}

#[test]
fn wald_test_preserves_representable_statistic_when_difference_squares_underflow() {
  let parameter_names = vec!["x".into()];
  let covariance = CovarianceMatrix::new(
    parameter_names.clone(),
    CovarianceType::NonRobust,
    vec![vec![1e-320]],
  )
  .expect("positive subnormal covariance should be valid");
  let post_estimation = PostEstimationModel {
    parameter_names,
    parameters: vec![1e-170],
    covariance,
    degrees_of_freedom: 3,
  };

  let result = post_estimation
    .test_linear_hypothesis(&[vec![1.0]], &[0.0])
    .expect("subnormal restriction covariance should remain usable");

  let expected = (1e-170 / 1e-320_f64.sqrt()).powi(2);
  assert!(result.chi2_statistic > 0.0);
  assert!((result.chi2_statistic / expected - 1.0).abs() < 1e-12);
}

#[test]
fn zero_variance_combination_has_nan_test_and_point_interval() {
  let result = model(3)
    .linear_combination_with_inference(&[0.0, 0.0, 0.0], 2.0, 95.0)
    .expect("constant combination inference should succeed");

  assert_eq!(result.estimate, 2.0);
  assert_eq!(result.standard_error, 0.0);
  assert_eq!(result.statistic, None);
  assert!(result.p_value.is_nan());
  assert_eq!(result.ci_lower, 2.0);
  assert_eq!(result.ci_upper, 2.0);
}

#[test]
fn inference_requires_positive_residual_degrees_of_freedom_and_valid_level() {
  assert_eq!(
    model(0)
      .linear_combination_with_inference(&[0.0, 1.0, 0.0], 0.0, 95.0)
      .unwrap_err(),
    StatsError::InsufficientObservations(
      "linear combination inference requires positive residual degrees of freedom".into()
    )
  );
  for confidence_level in [0.0, 100.0] {
    assert!(matches!(
      model(3).linear_combination_with_inference(&[0.0, 1.0, 0.0], 0.0, confidence_level),
      Err(StatsError::IncompatibleHypothesis(_))
    ));
  }
}
