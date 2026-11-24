use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem,
  robust_anderson_rubin_test, robust_multiple_instrument_anderson_rubin_confidence_set,
};

fn make_problem(fixture: &str, clustered: bool) -> InstrumentalVariablesProblem {
  let data = match fixture {
    "multiple" => include_str!("../../../examples/robust-ar/robust_ar_six_ci_multiple_data.csv"),
    "secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_six_ci_secondary_data.csv")
    }
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
    instrument_names: vec![
      "z1".into(),
      "z2".into(),
      "z3".into(),
      "z4".into(),
      "z5".into(),
      "z6".into(),
    ],
    response: rows.iter().map(|row| row[1]).collect(),
    exogenous: rows.iter().map(|row| vec![row[2]]).collect(),
    endogenous: rows.iter().map(|row| vec![row[9]]).collect(),
    instruments: rows
      .iter()
      .map(|row| vec![row[3], row[4], row[5], row[6], row[7], row[8]])
      .collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new(
        (0..rows.len()).collect(),
        rows.len(),
        0,
        None,
        clustered.then(|| rows.iter().map(|row| row[10].to_string()).collect()),
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
fn six_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_six_ci_reference.csv");
  for (fixture, covariance_name, covariance) in [
    ("multiple", "hc1", CovarianceType::RobustHc1),
    (
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    ("secondary", "hc1", CovarianceType::RobustHc1),
    (
      "secondary",
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
    assert_eq!(
      set.excluded_instruments,
      ["z1", "z2", "z3", "z4", "z5", "z6"]
    );
    assert_eq!(set.confidence_level, 95.0);
    assert_eq!(set.observation_count, 150);
    assert_eq!(set.auxiliary_parameter_count, 8);
    assert_eq!(set.auxiliary_residual_df, 142);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(25)
    );
    assert_eq!(set.degrees_of_freedom, 6);
    close(set.critical_chi_square, 12.591_587_243_744, 2e-12);

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "12"));
    for row in &expected_rows {
      let root_count = row[11].parse::<usize>().unwrap();
      assert!(root_count <= 12);
      assert_eq!(root_count % 2, 0);
    }
    let shape = match expected_rows[0][2] {
      "multiple" => ArConfidenceSetShape::Multiple,
      "disjoint" => ArConfidenceSetShape::Disjoint,
      "all_real" => ArConfidenceSetShape::AllReal,
      "bounded" => ArConfidenceSetShape::Bounded,
      "empty" => ArConfidenceSetShape::Empty,
      _ => unreachable!(),
    };
    assert_eq!(set.shape, shape);
    let expected_intervals = expected_rows
      .iter()
      .filter(|fields| fields[13] == "interval")
      .collect::<Vec<_>>();
    assert_eq!(set.intervals.len(), expected_intervals.len());
    for (actual, expected) in set.intervals.iter().zip(expected_intervals) {
      match (actual.lower, reference_bound(expected[15])) {
        (None, None) => {}
        (Some(actual), Some(expected)) => close(actual, expected, 3e-8),
        mismatch => panic!("lower endpoint mismatch: {mismatch:?}"),
      }
      match (actual.upper, reference_bound(expected[16])) {
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
    for expected in expected_rows.iter().filter(|fields| fields[13] == "probe") {
      let beta = expected[17].parse::<f64>().unwrap();
      let statistic = expected[18].parse::<f64>().unwrap();
      let tested = robust_anderson_rubin_test(&problem, &[beta], &covariance).unwrap();
      close(tested.statistic, statistic, 3e-8);
      assert_eq!(
        contains(&set, beta),
        tested.statistic <= set.critical_chi_square
      );
    }
  }
}

#[test]
fn six_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
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
    row[3] *= -0.5;
    row[4] *= 3.0;
    row[5] *= -2.0;
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
        (Some(left), Some(right)) => close(left, right, 3e-7),
        (None, None) => {}
        mismatch => panic!("endpoint mismatch after scaling: {mismatch:?}"),
      }
    }
  }
}
