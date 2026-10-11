use tabdat_stats::{StatsError, independent_ttest, one_sample_ttest, paired_ttest};

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-9_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "expected {expected}, got {actual}"
  );
}

#[test]
fn one_sample_test_matches_scipy_reference() {
  let result = one_sample_ttest(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3.0).unwrap();

  assert_eq!(result.group1.observations, 6);
  close(result.group1.mean, 3.5);
  close(result.group1.standard_deviation, 1.870_828_693_386_970_7);
  close(result.group1.standard_error, 0.763_762_615_825_973_4);
  close(result.group1.ci_lower, 1.536_685_693_019_676);
  close(result.group1.ci_upper, 5.463_314_306_980_324);
  close(result.statistic, 0.654_653_670_707_977_1);
  close(result.degrees_of_freedom, 5.0);
  close(result.p_left, 0.729_197_719_603_439_8);
  close(result.p_two, 0.541_604_560_793_120_4);
  close(result.p_right, 0.270_802_280_396_560_2);
  assert_eq!(result.difference, result.group1);
  assert!(result.group2.is_none());
  assert!(result.combined.is_none());
}

#[test]
fn paired_test_matches_scipy_reference() {
  let result = paired_ttest(
    &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    &[10.0, 12.0, 15.0, 18.0, 20.0, 25.0],
  )
  .unwrap();

  let group2 = result.group2.as_ref().unwrap();
  close(result.group1.mean, 3.5);
  close(group2.mean, 16.666_666_666_666_668);
  close(result.difference.mean, -13.166_666_666_666_666);
  close(result.difference.standard_error, 1.492_574_211_815_874_2);
  close(result.difference.ci_lower, -17.003_450_823_899_74);
  close(result.difference.ci_upper, -9.329_882_509_433_59);
  close(result.statistic, -8.821_448_583_550_177);
  close(result.degrees_of_freedom, 5.0);
  close(result.p_left, 0.000_155_440_402_967_537_34);
  close(result.p_two, 0.000_310_880_805_935_109_8);
  close(result.p_right, 0.999_844_559_597_032_4);
  assert!(result.combined.is_none());
}

#[test]
fn pooled_and_welch_tests_match_scipy_references() {
  let first = [1.0, 2.0, 5.0];
  let second = [3.0, 4.0, 6.0];
  let pooled = independent_ttest(&first, &second, false).unwrap();
  let welch = independent_ttest(&first, &second, true).unwrap();

  close(pooled.group1.mean, 2.666_666_666_666_666_5);
  close(pooled.group2.as_ref().unwrap().mean, 4.333_333_333_333_333);
  close(pooled.combined.as_ref().unwrap().mean, 3.5);
  close(pooled.difference.mean, -1.666_666_666_666_666_5);
  close(pooled.difference.standard_error, 1.490_711_984_999_859_8);
  assert_eq!(pooled.difference.observations, 4);
  assert!(pooled.difference.standard_deviation.is_nan());
  close(pooled.statistic, -1.118_033_988_749_894_7);
  close(pooled.degrees_of_freedom, 4.0);
  close(pooled.p_left, 0.163_082_117_672_530_1);
  close(pooled.p_two, 0.326_164_235_345_060_2);
  close(pooled.p_right, 0.836_917_882_327_469_9);

  close(welch.difference.mean, pooled.difference.mean);
  close(
    welch.difference.standard_error,
    pooled.difference.standard_error,
  );
  close(welch.difference.ci_lower, -5.956_674_371_741_123);
  close(welch.difference.ci_upper, 2.623_341_038_407_79);
  assert_eq!(welch.difference.observations, 0);
  close(welch.statistic, -1.118_033_988_749_894_7);
  close(welch.degrees_of_freedom, 3.669_724_770_642_200_7);
  close(welch.p_left, 0.165_676_002_677_430_96);
  close(welch.p_two, 0.331_352_005_354_862);
  close(welch.p_right, 0.834_323_997_322_569);
}

#[test]
fn one_observation_and_zero_variance_preserve_undefined_inference() {
  let one = one_sample_ttest(&[4.0], 0.0).unwrap();
  assert_eq!(one.group1.observations, 1);
  assert_eq!(one.group1.mean, 4.0);
  assert!(one.group1.standard_deviation.is_nan());
  assert!(one.group1.standard_error.is_nan());
  assert!(one.group1.ci_lower.is_nan());
  assert!(one.statistic.is_nan());
  assert_eq!(one.degrees_of_freedom, 0.0);
  assert!(one.p_two.is_nan());

  let constant = paired_ttest(&[2.0, 2.0, 2.0], &[2.0, 2.0, 2.0]).unwrap();
  assert_eq!(constant.difference.mean, 0.0);
  assert_eq!(constant.difference.ci_lower, 0.0);
  assert_eq!(constant.difference.ci_upper, 0.0);
  assert_eq!(constant.degrees_of_freedom, 2.0);
  assert!(constant.statistic.is_nan());
  assert!(constant.p_left.is_nan());
  assert!(constant.p_two.is_nan());
  assert!(constant.p_right.is_nan());
}

#[test]
fn invalid_samples_are_rejected() {
  assert_eq!(
    one_sample_ttest(&[], 0.0).unwrap_err(),
    StatsError::EmptySample
  );
  assert_eq!(
    one_sample_ttest(&[1.0, f64::INFINITY], 0.0).unwrap_err(),
    StatsError::NonFiniteValue("one-sample observations".into())
  );
  assert!(matches!(
    paired_ttest(&[1.0, 2.0], &[1.0]),
    Err(StatsError::DimensionMismatch(_))
  ));
}
