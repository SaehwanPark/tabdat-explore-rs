use tabdat_stats::f_distribution_survival_probability;

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-12_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.16e} differs from expected {expected:.16e} by more than {tolerance:.3e}"
  );
}

#[test]
fn f_distribution_survival_probability_matches_scipy_reference_values() {
  close(
    f_distribution_survival_probability(182.12068965516917, 2.0, 3.0),
    0.000738336718915947,
  );
  close(
    f_distribution_survival_probability(6.256742705570278, 1.0, 3.0),
    0.08760247783654902,
  );
  let extreme_tail = f_distribution_survival_probability(1e20, 2.0, 3.0);
  assert!(extreme_tail > 0.0);
  assert!((extreme_tail / 1.8371173070873807e-30 - 1.0).abs() < 1e-9);
}

#[test]
fn f_distribution_survival_probability_handles_boundaries() {
  assert_eq!(f_distribution_survival_probability(0.0, 2.0, 3.0), 1.0);
  assert_eq!(f_distribution_survival_probability(-1.0, 2.0, 3.0), 1.0);
  assert_eq!(
    f_distribution_survival_probability(f64::INFINITY, 2.0, 3.0),
    0.0
  );
  assert!(f_distribution_survival_probability(f64::NAN, 2.0, 3.0).is_nan());
  assert!(f_distribution_survival_probability(1.0, 0.0, 3.0).is_nan());
  assert!(f_distribution_survival_probability(1.0, 1.0, 0.0).is_nan());
}

#[test]
fn f_distribution_survival_probability_preserves_unit_tail_when_x_underflows() {
  let probability = f_distribution_survival_probability(1e308, 2.0, 1e-20);
  assert!(probability > 1.0 - 1e-12, "got {probability}");

  let tiny_df_probability = f_distribution_survival_probability(1.0, 1.0, 1e-320);
  assert!(
    tiny_df_probability > 1.0 - 1e-12,
    "got {tiny_df_probability}"
  );

  let min_positive_df = f64::from_bits(1);
  assert_eq!(
    f_distribution_survival_probability(1.0, 1.0, min_positive_df),
    1.0
  );
  assert_eq!(
    f_distribution_survival_probability(1.0, min_positive_df, min_positive_df),
    0.5
  );
}
