use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem, StatsError,
  robust_anderson_rubin_test, robust_multiple_instrument_anderson_rubin_confidence_set,
};

fn make_problem(fixture: &str, clustered: bool) -> InstrumentalVariablesProblem {
  let data = match fixture {
    "multiple" => include_str!("fixtures/robust_ar_multi_ci_data.csv"),
    "all_real" => include_str!("fixtures/robust_ar_multi_ci_all_real_data.csv"),
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
    instrument_names: vec!["z1".into(), "z2".into()],
    response: rows.iter().map(|row| row[1]).collect(),
    exogenous: rows.iter().map(|row| vec![row[2]]).collect(),
    endogenous: rows.iter().map(|row| vec![row[5]]).collect(),
    instruments: rows.iter().map(|row| vec![row[3], row[4]]).collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new(
        (0..rows.len()).collect(),
        rows.len(),
        0,
        None,
        clustered.then(|| rows.iter().map(|row| row[6].to_string()).collect()),
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
fn two_instrument_hc1_and_cr1_sets_match_independent_reference_and_joint_test() {
  let reference = include_str!("fixtures/robust_ar_multi_ci_reference.csv");
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
  ] {
    let problem = make_problem(fixture, matches!(covariance, CovarianceType::Cluster(_)));
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap();
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(set.excluded_instruments, ["z1", "z2"]);
    assert_eq!(set.confidence_level, 95.0);
    assert_eq!(set.observation_count, 20);
    assert_eq!(set.auxiliary_parameter_count, 4);
    assert_eq!(set.auxiliary_residual_df, 16);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(10)
    );
    assert_eq!(set.degrees_of_freedom, 2);
    close(set.critical_chi_square, 5.991_464_547_107_98, 1e-14);

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
      _ => unreachable!(),
    };
    assert_eq!(set.shape, shape);
    let expected_intervals = expected_rows
      .iter()
      .filter(|fields| fields[11] == "interval")
      .collect::<Vec<_>>();
    assert_eq!(set.intervals.len(), expected_intervals.len());
    for (actual, expected) in set.intervals.iter().zip(expected_intervals) {
      let expected_lower = reference_bound(expected[13]);
      let expected_upper = reference_bound(expected[14]);
      match (actual.lower, expected_lower) {
        (None, None) => {}
        (Some(actual), Some(expected)) => close(actual, expected, 2e-10),
        mismatch => panic!("lower endpoint mismatch: {mismatch:?}"),
      }
      match (actual.upper, expected_upper) {
        (None, None) => {}
        (Some(actual), Some(expected)) => close(actual, expected, 2e-10),
        mismatch => panic!("upper endpoint mismatch: {mismatch:?}"),
      }
    }
    close(
      set.covariance_correction,
      expected_rows[0][7].parse().unwrap(),
      1e-14,
    );
    for expected in expected_rows.iter().filter(|fields| fields[11] == "probe") {
      let beta = expected[15].parse::<f64>().unwrap();
      let statistic = expected[16].parse::<f64>().unwrap();
      let tested = robust_anderson_rubin_test(&problem, &[beta], &covariance).unwrap();
      close(tested.statistic, statistic, 2e-9);
      assert_eq!(
        contains(&set, beta),
        tested.statistic <= set.critical_chi_square
      );
    }
  }
}

#[test]
fn two_instrument_sets_are_invariant_to_instrument_rescaling() {
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
    match (left.lower, right.lower) {
      (Some(left), Some(right)) => close(left, right, 2e-9),
      (None, None) => {}
      mismatch => panic!("lower endpoint mismatch after scaling: {mismatch:?}"),
    }
    match (left.upper, right.upper) {
      (Some(left), Some(right)) => close(left, right, 2e-9),
      (None, None) => {}
      mismatch => panic!("upper endpoint mismatch after scaling: {mismatch:?}"),
    }
  }
}

#[test]
fn unsupported_dimensions_and_rank_fail_explicitly() {
  let mut problem = make_problem("multiple", false);
  problem.instrument_names.pop();
  for row in &mut problem.instruments {
    row.pop();
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &problem,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut problem = make_problem("multiple", false);
  problem.instrument_names.push("z3".into());
  for row in &mut problem.instruments {
    row.push(row[0] + row[1]);
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &problem,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut problem = make_problem("multiple", false);
  for row in &mut problem.instruments {
    row[1] = row[0];
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &problem,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));
}

#[test]
fn no_intercept_and_singular_auxiliary_score_covariance_are_handled_explicitly() {
  let mut no_intercept = make_problem("multiple", false);
  no_intercept.include_intercept = false;
  let set = robust_multiple_instrument_anderson_rubin_confidence_set(
    &no_intercept,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for beta in [-10.0, 0.0, 0.5, 10.0] {
    let statistic =
      robust_anderson_rubin_test(&no_intercept, &[beta], &CovarianceType::RobustHc1).unwrap();
    assert_eq!(
      contains(&set, beta),
      statistic.statistic <= set.critical_chi_square
    );
  }

  let mut singular = make_problem("multiple", false);
  for index in 0..singular.response.len() {
    singular.response[index] = singular.exogenous[index][0] + singular.instruments[index][0]
      - singular.instruments[index][1];
    singular.endogenous[index][0] = 2.0 * singular.exogenous[index][0]
      + 0.5 * singular.instruments[index][0]
      + 0.2 * singular.instruments[index][1];
  }
  let singular_result = robust_multiple_instrument_anderson_rubin_confidence_set(
    &singular,
    95.0,
    &CovarianceType::RobustHc1,
  );
  assert!(
    matches!(singular_result, Err(StatsError::IncompatibleHypothesis(_))),
    "unexpected singular-design result: {singular_result:?}"
  );

  let mut singular_at_beta = make_problem("multiple", false);
  for index in 0..singular_at_beta.response.len() {
    singular_at_beta.response[index] = 2.0 * singular_at_beta.endogenous[index][0]
      + singular_at_beta.exogenous[index][0]
      + singular_at_beta.instruments[index][0]
      - singular_at_beta.instruments[index][1];
  }
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &singular_at_beta,
      95.0,
      &CovarianceType::RobustHc1,
    ),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut one_cluster = make_problem("multiple", true);
  one_cluster.sample.as_mut().unwrap().cluster_groups =
    Some(vec!["only".into(); one_cluster.response.len()]);
  assert!(matches!(
    robust_multiple_instrument_anderson_rubin_confidence_set(
      &one_cluster,
      95.0,
      &CovarianceType::Cluster("group".into()),
    ),
    Err(StatsError::InsufficientObservations(_))
  ));
}
