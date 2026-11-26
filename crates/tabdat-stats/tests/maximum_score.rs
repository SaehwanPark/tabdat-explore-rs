use tabdat_stats::{
  MAX_MAXIMUM_SCORE_OBSERVATIONS, MaximumScoreDirection, MaximumScoreRule, StatsError,
  fit_binary_maximum_score, maximum_score_predict,
};

#[test]
fn increasing_and_decreasing_thresholds_recover_perfect_classification() {
  let increasing = fit_binary_maximum_score(&[0.0, 0.0, 1.0, 1.0], &[0.0, 1.0, 2.0, 3.0]).unwrap();
  assert_eq!(increasing.correct_count, 4);
  assert_eq!(increasing.empirical_score, 1.0);
  assert_eq!(increasing.event_count, 2);
  assert_eq!(increasing.optimal_regions.len(), 1);
  assert!(matches!(
    increasing.selected_rule,
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Increasing,
      lower: 1.0,
      upper: 2.0,
      threshold: 1.5,
    }
  ));
  assert!((increasing.intercept.powi(2) + increasing.slope.powi(2) - 1.0).abs() < 1e-15);
  assert_eq!(increasing.predicted_classes, [false, false, true, true]);

  let decreasing = fit_binary_maximum_score(&[1.0, 1.0, 0.0, 0.0], &[0.0, 1.0, 2.0, 3.0]).unwrap();
  assert_eq!(decreasing.correct_count, 4);
  assert!(matches!(
    decreasing.selected_rule,
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Decreasing,
      lower: 1.0,
      upper: 2.0,
      threshold: 1.5,
    }
  ));
  assert_eq!(decreasing.predicted_classes, [true, true, false, false]);
}

#[test]
fn all_tied_regions_are_retained_and_selection_is_deterministic() {
  let result =
    fit_binary_maximum_score(&[0.0, 1.0, 0.0, 1.0, 0.0], &[0.0, 1.0, 2.0, 3.0, 4.0]).unwrap();

  assert_eq!(result.correct_count, 3);
  assert_eq!(result.empirical_score, 0.6);
  assert_eq!(result.optimal_regions.len(), 5);
  assert_eq!(
    result
      .optimal_regions
      .iter()
      .filter(|region| matches!(
        region.rule,
        MaximumScoreRule::Threshold {
          direction: MaximumScoreDirection::Increasing,
          ..
        }
      ))
      .count(),
    2
  );
  assert_eq!(
    result
      .optimal_regions
      .iter()
      .filter(|region| matches!(
        region.rule,
        MaximumScoreRule::Threshold {
          direction: MaximumScoreDirection::Decreasing,
          ..
        }
      ))
      .count(),
    2
  );
  assert!(matches!(
    result.optimal_regions[0].rule,
    MaximumScoreRule::Constant { class: false }
  ));
  assert_eq!(result.selected_rule, result.optimal_regions[0].rule);
  assert_eq!(result.classification.true_positive, 0);
  assert_eq!(result.classification.false_positive, 0);
  assert_eq!(result.classification.false_negative, 2);
  assert_eq!(result.classification.true_negative, 3);
}

#[test]
fn duplicate_predictors_are_never_split_and_constant_classifiers_are_candidates() {
  let duplicates = fit_binary_maximum_score(&[0.0, 1.0, 0.0, 1.0], &[0.0, 0.0, 2.0, 2.0]).unwrap();
  assert_eq!(duplicates.correct_count, 2);
  assert_eq!(duplicates.optimal_regions.len(), 4);
  assert_eq!(
    duplicates.predicted_classes[0],
    duplicates.predicted_classes[1]
  );
  assert_eq!(
    duplicates.predicted_classes[2],
    duplicates.predicted_classes[3]
  );

  let constant = fit_binary_maximum_score(&[0.0, 0.0], &[1.0, 2.0]).unwrap();
  assert_eq!(constant.correct_count, 2);
  assert_eq!(constant.intercept, -1.0);
  assert_eq!(constant.slope, 0.0);
  assert_eq!(constant.predicted_classes, [false, false]);
  assert!(matches!(
    constant.selected_rule,
    MaximumScoreRule::Constant { class: false }
  ));
}

#[test]
fn adjacent_float_thresholds_preserve_the_sample_split_and_prediction_rule() {
  let upper = f64::from_bits(1.0_f64.to_bits() + 1);
  let result = fit_binary_maximum_score(&[0.0, 1.0], &[1.0, upper]).unwrap();

  assert_eq!(result.correct_count, 2);
  assert_eq!(result.predicted_classes, [false, true]);
  assert!(!maximum_score_predict(&result.selected_rule, 1.0).unwrap());
  assert!(maximum_score_predict(&result.selected_rule, upper).unwrap());
}

#[test]
fn prediction_rejects_non_finite_values_and_retained_results_round_trip() {
  let result = fit_binary_maximum_score(&[0.0, 0.0, 1.0, 1.0], &[0.0, 1.0, 2.0, 3.0]).unwrap();
  assert!(matches!(
    maximum_score_predict(&result.selected_rule, f64::INFINITY),
    Err(StatsError::NonFiniteValue(_))
  ));

  let encoded = serde_json::to_string(&result).unwrap();
  let decoded: tabdat_stats::MaximumScoreResult = serde_json::from_str(&encoded).unwrap();
  assert_eq!(decoded, result);
}

#[test]
fn invalid_samples_fail_with_specific_categories() {
  assert_eq!(
    fit_binary_maximum_score(&[], &[]),
    Err(StatsError::EmptySample)
  );
  assert!(matches!(
    fit_binary_maximum_score(&[0.0], &[]),
    Err(StatsError::DimensionMismatch(_))
  ));
  assert!(matches!(
    fit_binary_maximum_score(&[2.0], &[1.0]),
    Err(StatsError::InvalidEstimation(_))
  ));
  assert!(matches!(
    fit_binary_maximum_score(&[f64::NAN], &[1.0]),
    Err(StatsError::NonFiniteValue(_))
  ));
  assert!(matches!(
    fit_binary_maximum_score(&[0.0, 1.0], &[1.0, f64::INFINITY]),
    Err(StatsError::NonFiniteValue(_))
  ));
  assert!(matches!(
    fit_binary_maximum_score(&[0.0, 1.0], &[1.0, 1.0]),
    Err(StatsError::InvalidEstimation(_))
  ));

  let too_many = vec![0.0; MAX_MAXIMUM_SCORE_OBSERVATIONS + 1];
  assert!(matches!(
    fit_binary_maximum_score(&too_many, &too_many),
    Err(StatsError::InsufficientObservations(_))
  ));
}
