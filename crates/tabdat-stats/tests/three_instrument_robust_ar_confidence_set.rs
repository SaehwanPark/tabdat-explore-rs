use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem, StatsError,
  robust_anderson_rubin_test, robust_multiple_instrument_anderson_rubin_confidence_set,
};

fn make_problem(fixture: &str, clustered: bool) -> InstrumentalVariablesProblem {
  let data = match fixture {
    "multiple" => include_str!("fixtures/robust_ar_three_ci_data.csv"),
    "all_real" => include_str!("fixtures/robust_ar_three_ci_all_real_data.csv"),
    "six_roots" => include_str!("fixtures/robust_ar_three_ci_six_roots_data.csv"),
    _ => unreachable!(),
  };
  let rows = data
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x".into()],
    instrument_names: vec!["z1".into(), "z2".into(), "z3".into()],
    response: rows.iter().map(|row| row[1]).collect(),
    exogenous: rows.iter().map(|row| vec![row[2]]).collect(),
    endogenous: rows.iter().map(|row| vec![row[6]]).collect(),
    instruments: rows
      .iter()
      .map(|row| vec![row[3], row[4], row[5]])
      .collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new(
        (0..rows.len()).collect(),
        rows.len(),
        0,
        None,
        clustered.then(|| rows.iter().map(|row| row[7].to_string()).collect()),
      )
      .unwrap(),
    ),
  }
}

fn close(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual:.16e} != {expected:.16e} (tol {tolerance:.2e})"
  );
}

fn reference_bound(value: &str) -> Option<f64> {
  if matches!(value, "-Inf" | "Inf" | "-inf" | "inf") {
    None
  } else {
    Some(value.parse().unwrap())
  }
}

fn contains(
  set: &tabdat_stats::RobustMultipleInstrumentAndersonRubinConfidenceSet,
  beta: f64,
) -> bool {
  set.intervals.iter().any(|interval| {
    interval.lower.is_none_or(|lower| beta >= lower)
      && interval.upper.is_none_or(|upper| beta <= upper)
  })
}

#[test]
fn three_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_three_ci_reference.csv");
  for (fixture, covariance_name, covariance) in [
    ("multiple", "hc1", CovarianceType::RobustHc1),
    (
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    ("all_real", "hc1", CovarianceType::RobustHc1),
    (
      "all_real",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    ("six_roots", "hc1", CovarianceType::RobustHc1),
    (
      "six_roots",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
  ] {
    let problem = make_problem(fixture, matches!(covariance, CovarianceType::Cluster(_)));
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap_or_else(|error| panic!("{fixture}/{covariance_name}: {error}"));
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(set.excluded_instruments, ["z1", "z2", "z3"]);
    assert_eq!(set.confidence_level, 95.0);
    assert_eq!(set.observation_count, 36);
    assert_eq!(set.auxiliary_parameter_count, 5);
    assert_eq!(set.auxiliary_residual_df, 31);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(12)
    );
    assert_eq!(set.degrees_of_freedom, 3);
    close(set.critical_chi_square, 7.814_727_903_251_179, 2e-12);

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    let shape = match expected_rows[0][2] {
      "multiple" => ArConfidenceSetShape::Multiple,
      "disjoint" => ArConfidenceSetShape::Disjoint,
      "all_real" => ArConfidenceSetShape::AllReal,
      "bounded" => ArConfidenceSetShape::Bounded,
      _ => unreachable!(),
    };
    assert_eq!(set.shape, shape);
    let expected_intervals = expected_rows
      .iter()
      .filter(|fields| fields[11] == "interval")
      .collect::<Vec<_>>();
    assert_eq!(set.intervals.len(), expected_intervals.len());
    for (actual, expected) in set.intervals.iter().zip(expected_intervals) {
      match (actual.lower, reference_bound(expected[13])) {
        (None, None) => {}
        (Some(actual), Some(expected)) => close(actual, expected, 3e-8),
        mismatch => panic!("lower endpoint mismatch: {mismatch:?}"),
      }
      match (actual.upper, reference_bound(expected[14])) {
        (None, None) => {}
        (Some(actual), Some(expected)) => close(actual, expected, 3e-8),
        mismatch => panic!("upper endpoint mismatch: {mismatch:?}"),
      }
    }
    close(
      set.covariance_correction,
      expected_rows[0][7].parse().unwrap(),
      2e-14,
    );
    for expected in expected_rows.iter().filter(|fields| fields[11] == "probe") {
      let beta = expected[15].parse::<f64>().unwrap();
      let statistic = expected[16].parse::<f64>().unwrap();
      let tested = robust_anderson_rubin_test(&problem, &[beta], &covariance).unwrap();
      close(tested.statistic, statistic, 2e-8);
      assert_eq!(
        contains(&set, beta),
        tested.statistic <= set.critical_chi_square
      );
    }
  }
}

#[test]
fn three_instrument_sets_are_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("multiple", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    row[0] *= -10.0;
    row[1] *= 0.25;
    row[2] *= 100.0;
  }
  let scaled = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  assert_eq!(baseline.shape, scaled.shape);
  assert_eq!(baseline.intervals.len(), scaled.intervals.len());
  for (left, right) in baseline.intervals.iter().zip(&scaled.intervals) {
    for (left, right) in [(left.lower, right.lower), (left.upper, right.upper)] {
      match (left, right) {
        (Some(left), Some(right)) => close(left, right, 2e-7),
        (None, None) => {}
        mismatch => panic!("endpoint mismatch after scaling: {mismatch:?}"),
      }
    }
  }
}

#[test]
fn five_instruments_and_rank_deficiency_fail_explicitly() {
  let mut too_many = make_problem("multiple", false);
  too_many.instrument_names.push("z4".into());
  too_many.instrument_names.push("z5".into());
  for row in &mut too_many.instruments {
    row.push(row[0] + row[1] + row[2]);
    row.push(row[0] - row[1] + row[2]);
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &too_many,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut singular = make_problem("multiple", false);
  for row in &mut singular.instruments {
    row[2] = row[0] + row[1];
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &singular,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut singular_score = make_problem("multiple", false);
  for index in 0..singular_score.response.len() {
    singular_score.response[index] = 2.0 * singular_score.endogenous[index][0]
      + singular_score.exogenous[index][0]
      + singular_score.instruments[index][0]
      - singular_score.instruments[index][1]
      + 0.4 * singular_score.instruments[index][2];
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &singular_score,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));
}
