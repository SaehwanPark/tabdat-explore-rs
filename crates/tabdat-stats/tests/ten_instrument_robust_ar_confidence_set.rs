use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem,
  robust_anderson_rubin_test, robust_multiple_instrument_anderson_rubin_confidence_set,
};

fn make_problem(fixture: &str, clustered: bool) -> InstrumentalVariablesProblem {
  let data = match fixture {
    "multiple" => include_str!("../../../examples/robust-ar/robust_ar_ten_ci_multiple_data.csv"),
    "secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_ten_ci_secondary_data.csv")
    }
    "eleven" => {
      include_str!("../../../examples/robust-ar/robust_ar_eleven_ci_multiple_data.csv")
    }
    "eleven-secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_eleven_ci_secondary_data.csv")
    }
    "twelve" => {
      include_str!("../../../examples/robust-ar/robust_ar_twelve_ci_multiple_data.csv")
    }
    "twelve-secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_twelve_ci_secondary_data.csv")
    }
    "thirteen" => {
      include_str!("../../../examples/robust-ar/robust_ar_thirteen_ci_multiple_data.csv")
    }
    "thirteen-secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_thirteen_ci_secondary_data.csv")
    }
    "fourteen" => {
      include_str!("../../../examples/robust-ar/robust_ar_fourteen_ci_multiple_data.csv")
    }
    "fourteen-secondary" => {
      include_str!("../../../examples/robust-ar/robust_ar_fourteen_ci_secondary_data.csv")
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
  let instrument_count = if fixture.starts_with("fourteen") {
    14
  } else if fixture.starts_with("thirteen") {
    13
  } else if fixture.starts_with("twelve") {
    12
  } else if fixture.starts_with("eleven") {
    11
  } else {
    10
  };
  let endogenous_index = 3 + instrument_count;
  let group_index = endogenous_index + 1;
  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x".into()],
    instrument_names: (1..=instrument_count)
      .map(|index| format!("z{index}"))
      .collect(),
    response: rows.iter().map(|row| row[1]).collect(),
    exogenous: rows.iter().map(|row| vec![row[2]]).collect(),
    endogenous: rows.iter().map(|row| vec![row[endogenous_index]]).collect(),
    instruments: rows
      .iter()
      .map(|row| row[3..endogenous_index].to_vec())
      .collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new(
        (0..rows.len()).collect(),
        rows.len(),
        0,
        None,
        clustered.then(|| {
          rows
            .iter()
            .map(|row| row[group_index].to_string())
            .collect()
        }),
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
fn ten_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_ten_ci_reference.csv");
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
      (1..=10)
        .map(|index| format!("z{index}"))
        .collect::<Vec<_>>()
    );
    assert_eq!(set.confidence_level, 95.0);
    assert_eq!(set.observation_count, 240);
    assert_eq!(set.auxiliary_parameter_count, 12);
    assert_eq!(set.auxiliary_residual_df, 228);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(24)
    );
    assert_eq!(set.degrees_of_freedom, 10);
    close(set.critical_chi_square, 18.307_038_053_275_146, 2e-12);

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "20"));
    for row in &expected_rows {
      let root_count = row[11].parse::<usize>().unwrap();
      assert!(root_count <= 20);
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
fn ten_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("multiple", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    for (value, scale) in row
      .iter_mut()
      .zip([-10.0, 0.25, 100.0, -0.5, 3.0, -2.0, 4.0, -1.5, 0.7, -0.8])
    {
      *value *= scale;
    }
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

#[test]
fn eleven_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_eleven_ci_reference.csv");
  for (fixture, reference_fixture, covariance_name, covariance) in [
    ("eleven", "multiple", "hc1", CovarianceType::RobustHc1),
    (
      "eleven",
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    (
      "eleven-secondary",
      "secondary",
      "hc1",
      CovarianceType::RobustHc1,
    ),
    (
      "eleven-secondary",
      "secondary",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
  ] {
    let problem = make_problem(fixture, covariance_name == "cluster");
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap();
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      (1..=11)
        .map(|index| format!("z{index}"))
        .collect::<Vec<_>>()
    );
    assert_eq!(set.observation_count, 240);
    assert_eq!(set.auxiliary_parameter_count, 13);
    assert_eq!(set.auxiliary_residual_df, 227);
    assert_eq!(set.degrees_of_freedom, 11);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(24)
    );

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == reference_fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "22"));
    assert!(expected_rows.iter().all(|fields| {
      let root_count = fields[11].parse::<usize>().unwrap();
      root_count <= 22 && root_count % 2 == 0
    }));
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
      set.critical_chi_square,
      expected_rows[0][10].parse().unwrap(),
      2e-12,
    );
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
fn eleven_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("eleven", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    for (value, scale) in row.iter_mut().zip([
      -10.0, 0.25, 100.0, -0.5, 3.0, -2.0, 4.0, -1.5, 0.7, -0.8, 2.5,
    ]) {
      *value *= scale;
    }
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

#[test]
fn twelve_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_twelve_ci_reference.csv");
  for (fixture, reference_fixture, covariance_name, covariance) in [
    ("twelve", "multiple", "hc1", CovarianceType::RobustHc1),
    (
      "twelve",
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    (
      "twelve-secondary",
      "secondary",
      "hc1",
      CovarianceType::RobustHc1,
    ),
    (
      "twelve-secondary",
      "secondary",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
  ] {
    let problem = make_problem(fixture, covariance_name == "cluster");
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap_or_else(|error| panic!("{fixture}/{covariance_name}: {error}"));
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      (1..=12)
        .map(|index| format!("z{index}"))
        .collect::<Vec<_>>()
    );
    assert_eq!(set.observation_count, 288);
    assert_eq!(set.auxiliary_parameter_count, 14);
    assert_eq!(set.auxiliary_residual_df, 274);
    assert_eq!(set.degrees_of_freedom, 12);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(24)
    );

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == reference_fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "24"));
    assert!(expected_rows.iter().all(|fields| {
      let root_count = fields[11].parse::<usize>().unwrap();
      root_count <= 24 && root_count % 2 == 0
    }));
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
      set.critical_chi_square,
      expected_rows[0][10].parse().unwrap(),
      2e-12,
    );
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
fn twelve_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("twelve", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    for (value, scale) in row.iter_mut().zip([
      -10.0, 0.25, 100.0, -0.5, 3.0, -2.0, 4.0, -1.5, 0.7, -0.8, 2.5, -0.25,
    ]) {
      *value *= scale;
    }
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

#[test]
fn thirteen_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_thirteen_ci_reference.csv");
  for (fixture, reference_fixture, covariance_name, covariance) in [
    ("thirteen", "multiple", "hc1", CovarianceType::RobustHc1),
    (
      "thirteen",
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    (
      "thirteen-secondary",
      "secondary",
      "hc1",
      CovarianceType::RobustHc1,
    ),
    (
      "thirteen-secondary",
      "secondary",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
  ] {
    let problem = make_problem(fixture, covariance_name == "cluster");
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap_or_else(|error| panic!("{fixture}/{covariance_name}: {error}"));
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      (1..=13)
        .map(|index| format!("z{index}"))
        .collect::<Vec<_>>()
    );
    assert_eq!(set.observation_count, 312);
    assert_eq!(set.auxiliary_parameter_count, 15);
    assert_eq!(set.auxiliary_residual_df, 297);
    assert_eq!(set.degrees_of_freedom, 13);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(24)
    );

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == reference_fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "26"));
    assert!(expected_rows.iter().all(|fields| {
      let root_count = fields[11].parse::<usize>().unwrap();
      root_count <= 26 && root_count % 2 == 0
    }));
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
      set.critical_chi_square,
      expected_rows[0][10].parse().unwrap(),
      2e-12,
    );
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
fn fourteen_instrument_hc1_and_cr1_sets_match_independent_reference() {
  let reference = include_str!("fixtures/robust_ar_fourteen_ci_reference.csv");
  for (fixture, reference_fixture, covariance_name, covariance) in [
    ("fourteen", "multiple", "hc1", CovarianceType::RobustHc1),
    (
      "fourteen",
      "multiple",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
    (
      "fourteen-secondary",
      "secondary",
      "hc1",
      CovarianceType::RobustHc1,
    ),
    (
      "fourteen-secondary",
      "secondary",
      "cluster",
      CovarianceType::Cluster("group".into()),
    ),
  ] {
    let problem = make_problem(fixture, covariance_name == "cluster");
    let before = problem.clone();
    let set = robust_multiple_instrument_anderson_rubin_confidence_set(&problem, 95.0, &covariance)
      .unwrap_or_else(|error| panic!("{fixture}/{covariance_name}: {error}"));
    assert_eq!(problem, before);
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      (1..=14)
        .map(|index| format!("z{index}"))
        .collect::<Vec<_>>()
    );
    assert_eq!(set.observation_count, 336);
    assert_eq!(set.auxiliary_parameter_count, 16);
    assert_eq!(set.auxiliary_residual_df, 320);
    assert_eq!(set.degrees_of_freedom, 14);
    assert_eq!(set.covariance_type, covariance);
    assert_eq!(
      set.cluster_group_count,
      (covariance_name == "cluster").then_some(24)
    );

    let expected_rows = reference
      .lines()
      .skip(1)
      .filter_map(|line| {
        let fields = line.split(',').collect::<Vec<_>>();
        (fields[0] == reference_fixture && fields[1] == covariance_name).then_some(fields)
      })
      .collect::<Vec<_>>();
    assert!(!expected_rows.is_empty());
    assert!(expected_rows.iter().all(|fields| fields[12] == "28"));
    assert!(expected_rows.iter().all(|fields| {
      let root_count = fields[11].parse::<usize>().unwrap();
      root_count <= 28 && root_count % 2 == 0
    }));
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
      set.critical_chi_square,
      expected_rows[0][10].parse().unwrap(),
      2e-12,
    );
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
fn thirteen_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("thirteen", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    for (value, scale) in row.iter_mut().zip([
      -10.0, 0.25, 100.0, -0.5, 3.0, -2.0, 4.0, -1.5, 0.7, -0.8, 2.5, -0.25, 1.75,
    ]) {
      *value *= scale;
    }
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

#[test]
fn fourteen_instrument_hc1_set_is_invariant_to_instrument_rescaling() {
  let mut problem = make_problem("fourteen", false);
  let baseline = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  for row in &mut problem.instruments {
    for (value, scale) in row.iter_mut().zip([
      -10.0, 0.25, 100.0, -0.5, 3.0, -2.0, 4.0, -1.5, 0.7, -0.8, 2.5, -0.25, 1.75, -0.9,
    ]) {
      *value *= scale;
    }
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

#[test]
fn fifteen_instrument_design_is_rejected_by_stats_api() {
  let mut problem = make_problem("fourteen", false);
  problem.instrument_names.push("row".into());
  for (index, row) in problem.instruments.iter_mut().enumerate() {
    row.push((index + 1) as f64);
  }
  let error = robust_multiple_instrument_anderson_rubin_confidence_set(
    &problem,
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap_err();
  assert!(
    error
      .to_string()
      .contains("two to fourteen excluded instruments")
  );
}
