use tabdat_stats::{StatsError, variance_inflation_factors};

fn assert_close(actual: f64, expected: f64) {
  let allowed = 1e-12_f64.max(1e-12 * expected.abs());
  assert!(
    (actual - expected).abs() <= allowed,
    "actual {actual} differs from expected {expected} by more than {allowed}"
  );
}

#[test]
fn variance_inflation_factors_match_closed_form_for_two_predictors() {
  let predictor_names = vec!["x1".to_owned(), "x2".to_owned()];
  let design_matrix = vec![
    vec![-2.0, -3.0],
    vec![-1.0, -4.0],
    vec![0.0, 2.0],
    vec![1.0, 0.0],
    vec![2.0, 5.0],
  ];

  let values = variance_inflation_factors(&predictor_names, &design_matrix, true)
    .expect("full-rank auxiliary regressions should succeed");
  assert_eq!(values.len(), 2);
  assert_close(values[0].unwrap(), 27.0 / 7.0);
  assert_close(values[1].unwrap(), 27.0 / 7.0);
}

#[test]
fn variance_inflation_factors_preserve_infinity_for_exact_auxiliary_dependence() {
  let predictor_names = vec!["x1".to_owned(), "x2".to_owned()];
  let design_matrix = vec![
    vec![-2.0, -3.0],
    vec![-1.0, -1.0],
    vec![0.0, 1.0],
    vec![1.0, 3.0],
    vec![2.0, 5.0],
  ];

  let values = variance_inflation_factors(&predictor_names, &design_matrix, true)
    .expect("each auxiliary model should have a full-rank design");
  assert!(values.into_iter().all(|value| value.unwrap().is_infinite()));
}

#[test]
fn variance_inflation_factors_return_none_for_undefined_auxiliary_r_squared() {
  let predictor_names = vec!["constant".to_owned()];
  let design_matrix = vec![vec![1.0], vec![1.0], vec![1.0]];

  assert_eq!(
    variance_inflation_factors(&predictor_names, &design_matrix, true)
      .expect("intercept-only auxiliary fit should succeed"),
    vec![None]
  );
}

#[test]
fn variance_inflation_factors_reject_invalid_design_dimensions() {
  let predictor_names = vec!["x1".to_owned(), "x2".to_owned()];
  assert_eq!(
    variance_inflation_factors(&predictor_names, &[vec![1.0]], true).unwrap_err(),
    StatsError::DimensionMismatch("VIF design row 0 has 1 columns, expected 2".to_owned())
  );
  assert_eq!(
    variance_inflation_factors(&[], &[vec![]], true).unwrap_err(),
    StatsError::InvalidParameterName("VIF requires at least one predictor".to_owned())
  );
}
