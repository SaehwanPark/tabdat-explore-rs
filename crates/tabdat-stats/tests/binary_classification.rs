use tabdat_stats::{StatsError, binary_classification};

#[test]
fn binary_classification_counts_and_percentages_use_a_positive_inclusive_cutoff() {
  let result =
    binary_classification(&[1.0, 1.0, 0.0, 0.0, 0.0], &[0.8, 0.5, 0.6, 0.1, 0.49], 0.5).unwrap();

  assert_eq!(result.cutoff, 0.5);
  assert_eq!(result.observation_count, 5);
  assert_eq!(result.actual_positive_count, 2);
  assert_eq!(result.actual_negative_count, 3);
  assert_eq!(result.predicted_positive_count, 3);
  assert_eq!(result.predicted_negative_count, 2);
  assert_eq!(
    (
      result.true_positive,
      result.false_positive,
      result.false_negative,
      result.true_negative
    ),
    (2, 1, 0, 2)
  );
  assert_eq!(result.sensitivity, Some(100.0));
  assert_eq!(result.specificity, Some(200.0 / 3.0));
  assert_eq!(result.positive_predictive_value, Some(200.0 / 3.0));
  assert_eq!(result.negative_predictive_value, Some(100.0));
  assert_eq!(result.false_positive_rate_given_negative, Some(100.0 / 3.0));
  assert_eq!(result.false_negative_rate_given_positive, Some(0.0));
  assert_eq!(result.false_positive_rate_given_positive, Some(100.0 / 3.0));
  assert_eq!(result.false_negative_rate_given_negative, Some(0.0));
  assert_eq!(result.correctly_classified, Some(80.0));
}

#[test]
fn binary_classification_keeps_zero_denominator_metrics_unavailable() {
  let result = binary_classification(&[1.0, 1.0], &[0.1, 0.4], 0.5).unwrap();
  assert_eq!(result.true_positive, 0);
  assert_eq!(result.false_negative, 2);
  assert_eq!(result.sensitivity, Some(0.0));
  assert_eq!(result.specificity, None);
  assert_eq!(result.positive_predictive_value, None);
  assert_eq!(result.negative_predictive_value, Some(0.0));
  assert_eq!(result.false_positive_rate_given_negative, None);
  assert_eq!(result.false_negative_rate_given_positive, Some(100.0));
  assert_eq!(result.false_positive_rate_given_positive, None);
  assert_eq!(result.false_negative_rate_given_negative, Some(100.0));
  assert_eq!(result.correctly_classified, Some(0.0));

  let no_events = binary_classification(&[0.0, 0.0], &[0.8, 0.2], 0.5).unwrap();
  assert_eq!(no_events.sensitivity, None);
  assert_eq!(no_events.specificity, Some(50.0));
  assert_eq!(no_events.false_negative_rate_given_positive, None);
}

#[test]
fn binary_classification_rejects_invalid_samples_and_cutoffs() {
  assert_eq!(
    binary_classification(&[], &[], 0.5),
    Err(StatsError::EmptySample)
  );
  assert!(matches!(
    binary_classification(&[0.0], &[], 0.5),
    Err(StatsError::DimensionMismatch(_))
  ));
  assert!(matches!(
    binary_classification(&[2.0], &[0.5], 0.5),
    Err(StatsError::InvalidEstimation(_))
  ));
  assert!(matches!(
    binary_classification(&[1.0], &[1.1], 0.5),
    Err(StatsError::InvalidEstimation(_))
  ));
  assert!(matches!(
    binary_classification(&[1.0], &[0.5], f64::NAN),
    Err(StatsError::InvalidEstimation(_))
  ));
  assert!(matches!(
    binary_classification(&[1.0], &[0.5], 1.01),
    Err(StatsError::InvalidEstimation(_))
  ));
}
