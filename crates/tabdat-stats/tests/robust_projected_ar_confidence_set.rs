use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem,
  robust_anderson_rubin_test, robust_projected_anderson_rubin_confidence_set,
};

fn make_problem(clustered: bool) -> InstrumentalVariablesProblem {
  let data = include_str!("../../../examples/robust-ar/projected_ar_data.csv");
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
  let groups = clustered.then(|| {
    rows
      .iter()
      .map(|row| row[7].to_string())
      .collect::<Vec<_>>()
  });
  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x1".into(), "x2".into()],
    instrument_names: vec!["z1".into(), "z2".into()],
    response: rows.iter().map(|row| row[1]).collect(),
    exogenous: rows.iter().map(|row| vec![row[2]]).collect(),
    endogenous: rows.iter().map(|row| vec![row[5], row[6]]).collect(),
    instruments: rows.iter().map(|row| vec![row[3], row[4]]).collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new((0..rows.len()).collect(), rows.len(), 0, None, groups).unwrap(),
    ),
  }
}

fn close(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual:.16e} != {expected:.16e} (tol {tolerance:.2e})"
  );
}

fn set_contains(result: &tabdat_stats::RobustProjectedArConfidenceSet, coefficient: f64) -> bool {
  result.intervals.iter().any(|interval| {
    interval.lower.is_none_or(|lower| coefficient >= lower)
      && interval.upper.is_none_or(|upper| coefficient <= upper)
  })
}

fn reference_rows(coefficient: &str, covariance: &str) -> Vec<Vec<&'static str>> {
  include_str!("fixtures/robust_ar_projected_reference.csv")
    .lines()
    .skip(1)
    .filter_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == coefficient && fields[1] == covariance).then_some(fields)
    })
    .collect()
}

fn reference_bound(value: &str) -> Option<f64> {
  match value {
    "-Inf" | "-inf" => None,
    "Inf" | "+Inf" | "inf" | "+inf" => None,
    value => Some(value.parse().unwrap()),
  }
}

#[test]
fn hc1_and_cr1_projections_match_independent_numeric_profile() {
  for (covariance_name, covariance) in [
    ("hc1", CovarianceType::RobustHc1),
    ("cluster", CovarianceType::Cluster("group".into())),
  ] {
    let problem = make_problem(matches!(covariance, CovarianceType::Cluster(_)));
    let before = problem.clone();
    for (coefficient, nuisance) in [("x1", "x2"), ("x2", "x1")] {
      let result =
        robust_projected_anderson_rubin_confidence_set(&problem, coefficient, 95.0, &covariance)
          .unwrap_or_else(|error| panic!("{coefficient}/{covariance_name}: {error}"));
      assert_eq!(problem, before);
      assert_eq!(result.coefficient, coefficient);
      assert_eq!(result.nuisance_endogenous, [nuisance]);
      assert_eq!(result.excluded_instruments, ["z1", "z2"]);
      assert_eq!(result.confidence_level, 95.0);
      assert_eq!(result.observation_count, 48);
      assert_eq!(result.auxiliary_parameter_count, 4);
      assert_eq!(result.auxiliary_residual_df, 44);
      assert_eq!(result.covariance_type, covariance);
      assert_eq!(
        result.cluster_group_count,
        (covariance_name == "cluster").then_some(12)
      );
      assert_eq!(result.degrees_of_freedom, 2);
      assert_eq!(result.shape, ArConfidenceSetShape::Bounded);
      assert_eq!(result.intervals.len(), 1);
      assert!(result.closure_conservative);
      assert!(!result.all_real_conservative_fallback);

      let expected_rows = reference_rows(coefficient, covariance_name);
      let interval = expected_rows
        .iter()
        .find(|fields| fields[11] == "interval")
        .unwrap();
      assert_eq!(result.shape, ArConfidenceSetShape::Bounded);
      close(
        result.covariance_correction,
        interval[7].parse().unwrap(),
        2e-14,
      );
      close(
        result.critical_chi_square,
        interval[10].parse().unwrap(),
        2e-12,
      );
      close(
        result.intervals[0].lower.unwrap(),
        reference_bound(interval[13]).unwrap(),
        2e-7,
      );
      close(
        result.intervals[0].upper.unwrap(),
        reference_bound(interval[14]).unwrap(),
        2e-7,
      );

      for fields in expected_rows.iter().filter(|fields| fields[11] == "probe") {
        let target = fields[15].parse::<f64>().unwrap();
        let nuisance = fields[16].parse::<f64>().unwrap();
        let null_values = if coefficient == "x1" {
          [target, nuisance]
        } else {
          [nuisance, target]
        };
        let tested = robust_anderson_rubin_test(&problem, &null_values, &covariance).unwrap();
        close(tested.statistic, fields[17].parse().unwrap(), 2e-7);
        assert_eq!(
          set_contains(&result, target),
          tested.statistic <= result.critical_chi_square,
          "{coefficient}/{covariance_name} at {target}"
        );
      }
    }
  }
}

#[test]
fn projection_is_invariant_to_instrument_rescaling() {
  let problem = make_problem(false);
  let baseline = robust_projected_anderson_rubin_confidence_set(
    &problem,
    "x1",
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  let mut rescaled = problem;
  for row in &mut rescaled.instruments {
    row[0] *= -100.0;
    row[1] *= 0.025;
  }
  let actual = robust_projected_anderson_rubin_confidence_set(
    &rescaled,
    "x1",
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  close(
    actual.intervals[0].lower.unwrap(),
    baseline.intervals[0].lower.unwrap(),
    2e-7,
  );
  close(
    actual.intervals[0].upper.unwrap(),
    baseline.intervals[0].upper.unwrap(),
    2e-7,
  );
}

#[test]
fn unsupported_dimensions_and_two_cluster_covariance_fail() {
  let problem = make_problem(false);
  assert!(
    robust_projected_anderson_rubin_confidence_set(
      &problem,
      "x1",
      95.0,
      &CovarianceType::NonRobust,
    )
    .is_err()
  );
  assert!(
    robust_projected_anderson_rubin_confidence_set(
      &problem,
      "missing",
      95.0,
      &CovarianceType::RobustHc1,
    )
    .is_err()
  );
  let mut three_instruments = problem.clone();
  three_instruments.instrument_names.push("z3".into());
  assert!(
    robust_projected_anderson_rubin_confidence_set(
      &three_instruments,
      "x1",
      95.0,
      &CovarianceType::RobustHc1,
    )
    .is_err()
  );

  let mut two_groups = make_problem(true);
  two_groups.sample.as_mut().unwrap().cluster_groups =
    Some((0..48).map(|row| (row / 24).to_string()).collect());
  assert!(
    robust_projected_anderson_rubin_confidence_set(
      &two_groups,
      "x1",
      95.0,
      &CovarianceType::Cluster("group".into()),
    )
    .is_err()
  );
}

#[test]
fn score_covariance_singular_for_every_null_is_an_error() {
  let mut problem = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x1".into(), "x2".into()],
    instrument_names: vec!["z1".into(), "z2".into()],
    response: Vec::new(),
    exogenous: Vec::new(),
    endogenous: Vec::new(),
    instruments: Vec::new(),
    include_intercept: true,
    sample: None,
  };
  for index in 0..8 {
    let w = if index < 4 { 1.0 } else { -1.0 };
    let z1 = if index % 4 < 2 { 1.0 } else { -1.0 };
    let z2 = if index % 2 == 0 { 1.0 } else { -1.0 };
    problem.exogenous.push(vec![w]);
    problem.instruments.push(vec![z1, z2]);
    problem.endogenous.push(vec![
      0.125 * w + 0.5 * z1 + 0.25 * z2,
      0.25 * w - 0.25 * z1 + 0.5 * z2,
    ]);
    problem
      .response
      .push(0.25 + 0.5 * w + 0.125 * z1 + 0.25 * z2);
  }
  assert!(
    robust_projected_anderson_rubin_confidence_set(
      &problem,
      "x1",
      95.0,
      &CovarianceType::RobustHc1,
    )
    .is_err()
  );
}

#[test]
fn candidate_score_covariance_singularity_uses_the_marked_all_real_fallback() {
  let mut problem = make_problem(false);
  for row in 0..problem.response.len() {
    problem.response[row] =
      0.8 + 0.3 * problem.exogenous[row][0] + 1.1 * problem.endogenous[row][0]
        - 0.55 * problem.endogenous[row][1];
  }
  let result = robust_projected_anderson_rubin_confidence_set(
    &problem,
    "x1",
    95.0,
    &CovarianceType::RobustHc1,
  )
  .unwrap();
  assert_eq!(result.shape, ArConfidenceSetShape::AllReal);
  assert!(result.closure_conservative);
  assert!(result.all_real_conservative_fallback);
}
