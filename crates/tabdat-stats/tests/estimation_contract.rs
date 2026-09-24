#![forbid(unsafe_code)]

use tabdat_stats::traits::Estimator;
use tabdat_stats::{
  EstimationProblem, EstimationSample, PostEstimationModel, PureLeastSquaresEstimator, StatsError,
  covariance_matrix, fit_least_squares, mean, predict_linear_response, sample_covariance,
  sample_variance,
};

#[test]
fn scalar_statistical_primitives_and_covariance_matrix() {
  let values = vec![1.0, 2.0, 3.0, 4.0];

  assert!((mean(&values).unwrap() - 2.5).abs() < 1e-12);
  assert!((sample_variance(&values).unwrap() - 1.6666666666666667).abs() < 1e-12);
  assert!((sample_covariance(&values, &values).unwrap() - 1.6666666666666667).abs() < 1e-12);

  let matrix = covariance_matrix(&[values.clone(), values.clone()]).unwrap();
  assert!((matrix[0][0] - 1.6666666666666667).abs() < 1e-12);
  assert!((matrix[0][1] - 1.6666666666666667).abs() < 1e-12);
  assert!((matrix[1][0] - 1.6666666666666667).abs() < 1e-12);
  assert!((matrix[1][1] - 1.6666666666666667).abs() < 1e-12);
}

#[test]
fn scalar_statistical_primitives_reject_invalid_inputs() {
  assert!(matches!(mean(&[]), Err(StatsError::EmptySample)));
  assert!(matches!(
    sample_variance(&[1.0]),
    Err(StatsError::DimensionMismatch(_))
  ));
  assert!(matches!(
    sample_covariance(&[1.0], &[1.0]),
    Err(StatsError::DimensionMismatch(_))
  ));
  assert!(matches!(
    sample_covariance(&[1.0, 2.0], &[1.0]),
    Err(StatsError::DimensionMismatch(_))
  ));
}

#[test]
fn fit_least_squares_returns_full_result_contract() {
  let problem = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0, 3.0, 5.0],
    vec![vec![0.0], vec![1.0], vec![2.0]],
  )
  .expect("valid problem");

  let result = fit_least_squares(&problem).expect("fit should succeed");

  assert_eq!(result.parameter_names, vec!["intercept", "x"]);
  assert_eq!(result.coefficients[0].name, "intercept");
  assert!((result.coefficients[0].value - 1.0).abs() < 1e-12);
  assert_eq!(result.coefficients[1].name, "x");
  assert!((result.coefficients[1].value - 2.0).abs() < 1e-12);

  assert_eq!(result.fitted_values.len(), 3);
  assert!((result.fitted_values[0] - 1.0).abs() < 1e-12);
  assert!((result.fitted_values[1] - 3.0).abs() < 1e-12);
  assert!((result.fitted_values[2] - 5.0).abs() < 1e-12);

  assert_eq!(result.residuals.len(), 3);
  assert!(result.residuals[0].abs() < 1e-12);
  assert!(result.residuals[1].abs() < 1e-12);
  assert!(result.residuals[2].abs() < 1e-12);

  assert_eq!(result.diagnostics.method, "least_squares");
  assert!(result.diagnostics.converged);
  assert!(result.diagnostics.objective_value.unwrap().abs() < 1e-12);
  assert_eq!(result.observation_count(), 3);
  assert_eq!(result.degrees_of_freedom(), 1);

  assert!((result.r_squared().unwrap() - 1.0).abs() < 1e-12);
  assert!((result.root_mse().unwrap() - 0.0).abs() < 1e-12);
}

#[test]
fn estimator_trait_dispatches_cleanly() {
  let problem = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![2.0, 4.0, 6.0],
    vec![vec![1.0], vec![2.0], vec![3.0]],
  )
  .expect("valid problem");

  let estimator = PureLeastSquaresEstimator;
  let result = estimator.fit(&problem).expect("trait fit should succeed");
  assert!((result.parameter_value("x").unwrap() - 2.0).abs() < 1e-12);
}

#[test]
fn fit_least_squares_rejects_saturated_models() {
  let problem = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0, 2.0],
    vec![vec![0.0], vec![1.0]],
  )
  .expect("valid problem");

  // With intercept, parameters = 2, observations = 2 -> df = 0
  let err = fit_least_squares(&problem).unwrap_err();
  assert!(matches!(
    err,
    StatsError::ZeroDegreesOfFreedom {
      observations: 2,
      parameters: 2
    }
  ));
}

#[test]
fn fit_least_squares_rejects_collinear_predictors() {
  let problem = EstimationProblem::new(
    "y",
    vec!["x1".into(), "x2".into()],
    vec![1.0, 2.0, 3.0, 4.0],
    vec![
      vec![1.0, 2.0],
      vec![2.0, 4.0],
      vec![3.0, 6.0],
      vec![4.0, 8.0],
    ],
  )
  .expect("valid problem");

  let err = fit_least_squares(&problem).unwrap_err();
  assert!(matches!(err, StatsError::SingularMatrix(_)));
}

#[test]
fn predict_linear_response_contract() {
  let design = vec![vec![1.0, 0.0], vec![1.0, 2.0], vec![1.0, 4.0]];
  let params = vec![10.0, 3.0];

  let preds = predict_linear_response(&design, &params).expect("prediction should succeed");
  assert_eq!(preds, vec![10.0, 16.0, 22.0]);

  // Dimension mismatch
  let err = predict_linear_response(&design, &[1.0]).unwrap_err();
  assert!(matches!(err, StatsError::DimensionMismatch(_)));
}

#[test]
fn post_estimation_hypothesis_testing_and_combinations() {
  let problem = EstimationProblem::new(
    "y",
    vec!["x1".into(), "x2".into()],
    vec![2.0, 5.0, 7.0, 9.0, 12.0],
    vec![
      vec![1.0, 2.0],
      vec![2.0, 1.0],
      vec![3.0, 4.0],
      vec![4.0, 3.0],
      vec![5.0, 5.0],
    ],
  )
  .expect("valid problem");

  let result = fit_least_squares(&problem).expect("fit should succeed");
  let post = PostEstimationModel::from_least_squares(&result);

  assert_eq!(post.parameter_names, vec!["intercept", "x1", "x2"]);
  let b_x1 = post.parameter("x1").expect("x1 parameter");
  assert!((b_x1 - result.parameter_value("x1").unwrap()).abs() < 1e-12);

  // Linear combination: x1 + x2 (weights: intercept=0, x1=1, x2=1)
  let lincom = post
    .linear_combination(&[0.0, 1.0, 1.0])
    .expect("lincom should succeed");
  let expected_combo = post.parameter("x1").unwrap() + post.parameter("x2").unwrap();
  assert!((lincom.estimate - expected_combo).abs() < 1e-12);
  assert!(lincom.standard_error > 0.0);
  assert!(lincom.statistic.is_some());

  // Hypothesis test: H0: x1 = 0, x2 = 0
  let r_matrix = vec![vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]];
  let r_vector = vec![0.0, 0.0];
  let test_result = post
    .test_linear_hypothesis(&r_matrix, &r_vector)
    .expect("test should succeed");
  assert_eq!(test_result.df_num, 2);
  assert_eq!(test_result.df_denom, 2);
  assert!(test_result.f_statistic > 0.0);
  assert!(test_result.chi2_statistic > 0.0);
}

#[test]
fn estimation_sample_tracking_invariants() {
  let mask = vec![true, false, true, true, false];
  let sample = EstimationSample::from_mask(&mask, None, None).expect("valid sample");

  assert_eq!(sample.total_observations, 5);
  assert_eq!(sample.retained_count(), 3);
  assert_eq!(sample.dropped_observations, 2);
  assert_eq!(sample.retained_indices, vec![0, 2, 3]);
  assert!(!sample.is_weighted());
  assert!(!sample.is_clustered());
}
