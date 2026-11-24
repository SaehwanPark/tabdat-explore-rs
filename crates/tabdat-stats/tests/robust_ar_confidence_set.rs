use tabdat_stats::{
  ArConfidenceSetShape, CovarianceType, EstimationSample, InstrumentalVariablesProblem, StatsError,
  robust_anderson_rubin_confidence_set, robust_anderson_rubin_test,
};

fn reference_problem(covariance: &str) -> InstrumentalVariablesProblem {
  let rows = include_str!("fixtures/robust_ar_ci_data.csv")
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .map(|value| value.parse::<f64>().ok())
        .collect::<Vec<_>>()
    })
    .filter(|row| {
      row[1..5].iter().all(Option::is_some) && (covariance != "cluster" || row[5].is_some())
    })
    .collect::<Vec<_>>();
  let clustered = covariance == "cluster";
  let total_observations = 80;
  let retained_indices = rows
    .iter()
    .map(|row| row[0].unwrap() as usize - 1)
    .collect::<Vec<_>>();
  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x".into()],
    instrument_names: vec!["z".into()],
    response: rows.iter().map(|row| row[1].unwrap()).collect(),
    exogenous: rows.iter().map(|row| vec![row[2].unwrap()]).collect(),
    endogenous: rows.iter().map(|row| vec![row[3].unwrap()]).collect(),
    instruments: rows.iter().map(|row| vec![row[4].unwrap()]).collect(),
    include_intercept: true,
    sample: Some(
      EstimationSample::new(
        retained_indices,
        total_observations,
        total_observations - rows.len(),
        None,
        clustered.then(|| rows.iter().map(|row| row[5].unwrap().to_string()).collect()),
      )
      .unwrap(),
    ),
  }
}

fn contains(set: &tabdat_stats::RobustAndersonRubinConfidenceSet, value: f64) -> bool {
  set.intervals.iter().any(|interval| {
    interval.lower.is_none_or(|lower| value >= lower)
      && interval.upper.is_none_or(|upper| value <= upper)
  })
}

fn close(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual:.16e} != {expected:.16e} (tol {tolerance:.2e})"
  );
}

fn orthogonal_problem(strength: f64, direct_z: f64) -> InstrumentalVariablesProblem {
  let mut problem = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x".into()],
    instrument_names: vec!["z".into()],
    response: Vec::new(),
    exogenous: Vec::new(),
    endogenous: Vec::new(),
    instruments: Vec::new(),
    include_intercept: true,
    sample: Some(EstimationSample::all_retained(40)),
  };
  for i in 1..=40 {
    let basis = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let w = basis(5.0);
    let z = basis(1.0);
    let x = strength * z + basis(3.0);
    problem
      .response
      .push(1.5 * x + 0.5 * w + direct_z * z + basis(4.0));
    problem.exogenous.push(vec![w]);
    problem.endogenous.push(vec![x]);
    problem.instruments.push(vec![z]);
  }
  problem
}

#[test]
fn weak_relevance_preserves_all_real_and_disjoint_robust_sets() {
  let all_real_problem = orthogonal_problem(0.1, 0.0);
  let all_real =
    robust_anderson_rubin_confidence_set(&all_real_problem, 95.0, &CovarianceType::RobustHc1)
      .unwrap();
  assert_eq!(all_real.shape, ArConfidenceSetShape::AllReal);
  for beta in [-100.0, 0.0, 1.5, 100.0] {
    assert!(contains(&all_real, beta));
    assert!(
      robust_anderson_rubin_test(&all_real_problem, &[beta], &CovarianceType::RobustHc1)
        .unwrap()
        .statistic
        <= all_real.critical_chi_square
    );
  }

  let disjoint_problem = orthogonal_problem(0.1, 2.0);
  let disjoint =
    robust_anderson_rubin_confidence_set(&disjoint_problem, 95.0, &CovarianceType::RobustHc1)
      .unwrap();
  assert_eq!(disjoint.shape, ArConfidenceSetShape::Disjoint);
  assert_eq!(disjoint.intervals.len(), 2);
  for beta in [-100.0, -8.0, 0.0, 8.0, 100.0] {
    let test =
      robust_anderson_rubin_test(&disjoint_problem, &[beta], &CovarianceType::RobustHc1).unwrap();
    assert_eq!(
      contains(&disjoint, beta),
      test.statistic <= disjoint.critical_chi_square
    );
  }
}

#[test]
fn robust_scalar_sets_match_independent_r_and_invert_the_joint_test() {
  // Base R 4.6.2 lm.fit plus explicit HC1/CR1 score sandwich and polyroot:
  // fixtures/robust_ar_ci_reference.R. The data contain nonconsecutive missing
  // outcome/instrument rows and an additional cluster-only exclusion.
  let reference = include_str!("fixtures/robust_ar_ci_reference.csv");
  for line in reference.lines().skip(1) {
    let fields = line.split(',').collect::<Vec<_>>();
    let clustered = fields[0] == "cluster";
    let level = fields[1].parse::<f64>().unwrap();
    let covariance = if clustered {
      CovarianceType::Cluster("g".into())
    } else {
      CovarianceType::RobustHc1
    };
    let problem = reference_problem(fields[0]);
    let before = problem.clone();
    let result = robust_anderson_rubin_confidence_set(&problem, level, &covariance).unwrap();
    assert_eq!(problem, before);
    assert_eq!(result.shape, ArConfidenceSetShape::Bounded);
    assert_eq!(result.endogenous, "x");
    assert_eq!(result.excluded_instrument, "z");
    assert_eq!(result.confidence_level, level);
    assert_eq!(
      result.observation_count,
      fields[2].parse::<usize>().unwrap()
    );
    assert_eq!(
      result.auxiliary_parameter_count,
      fields[3].parse::<usize>().unwrap()
    );
    assert_eq!(
      result.auxiliary_residual_df,
      result.observation_count - result.auxiliary_parameter_count
    );
    assert_eq!(result.covariance_type, covariance);
    assert_eq!(
      result.cluster_group_count,
      clustered.then(|| fields[4].parse().unwrap())
    );
    assert_eq!(result.degrees_of_freedom, 1);
    for (actual, field) in [
      (result.covariance_correction, 5),
      (result.critical_chi_square, 6),
      (result.intervals[0].lower.unwrap(), 8),
      (result.intervals[0].upper.unwrap(), 9),
    ] {
      close(actual, fields[field].parse().unwrap(), 1e-10);
    }
    assert_eq!(result.intervals.len(), 1);
    assert_eq!(
      problem.sample.as_ref().unwrap().retained_indices,
      (0..80)
        .filter(|index| ![1, 7].contains(index) && (!clustered || *index != 16))
        .collect::<Vec<_>>()
    );

    let lower_reference = fields[8].parse::<f64>().unwrap();
    let upper_reference = fields[9].parse::<f64>().unwrap();
    for (beta, expected_member, statistic_field) in [
      (fields[7].parse().unwrap(), true, 11),
      (lower_reference - 0.5, false, 13),
      (upper_reference + 0.5, false, 14),
    ] {
      assert_eq!(
        contains(&result, beta),
        expected_member,
        "{line}: beta={beta}"
      );
      let test = robust_anderson_rubin_test(&problem, &[beta], &covariance).unwrap();
      close(
        test.statistic,
        fields[statistic_field].parse().unwrap(),
        1e-9,
      );
      assert_eq!(
        contains(&result, beta),
        test.statistic <= result.critical_chi_square
      );
    }

    let (lower, upper) = (
      result.intervals[0].lower.unwrap(),
      result.intervals[0].upper.unwrap(),
    );
    for endpoint in [lower, upper] {
      assert!(contains(&result, endpoint));
      close(
        robust_anderson_rubin_test(&problem, &[endpoint], &covariance)
          .unwrap()
          .statistic,
        result.critical_chi_square,
        1e-9,
      );
    }

    let mut rescaled = problem.clone();
    for row in &mut rescaled.instruments {
      row[0] *= 1e5;
    }
    let changed = robust_anderson_rubin_confidence_set(&rescaled, level, &covariance).unwrap();
    close(changed.intervals[0].lower.unwrap(), lower, 1e-9);
    close(changed.intervals[0].upper.unwrap(), upper, 1e-9);
  }
}

#[test]
fn robust_scalar_set_rejects_unsupported_or_undefined_designs() {
  let problem = reference_problem("hc1");
  assert!(
    robust_anderson_rubin_confidence_set(&problem, 95.0, &CovarianceType::NonRobust).is_err()
  );
  for level in [0.0, 100.0, -1.0, f64::NAN, f64::INFINITY] {
    assert!(
      robust_anderson_rubin_confidence_set(&problem, level, &CovarianceType::RobustHc1).is_err()
    );
  }
  assert!(
    robust_anderson_rubin_confidence_set(&problem, 99.99999999999999, &CovarianceType::RobustHc1)
      .is_err()
  );

  let mut multiple_instruments = problem.clone();
  multiple_instruments.instrument_names.push("z2".into());
  for (index, row) in multiple_instruments.instruments.iter_mut().enumerate() {
    row.push((index as f64 / 3.0).sin());
  }
  assert!(matches!(
    robust_anderson_rubin_confidence_set(&multiple_instruments, 95.0, &CovarianceType::RobustHc1),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut multiple_endogenous = problem.clone();
  multiple_endogenous.endogenous_names.push("x2".into());
  for (index, row) in multiple_endogenous.endogenous.iter_mut().enumerate() {
    row.push((index as f64 / 4.0).sin());
  }
  assert!(matches!(
    robust_anderson_rubin_confidence_set(&multiple_endogenous, 95.0, &CovarianceType::RobustHc1),
    Err(StatsError::IncompatibleHypothesis(_))
  ));

  let mut no_sample = problem.clone();
  no_sample.sample = None;
  assert!(
    robust_anderson_rubin_confidence_set(&no_sample, 95.0, &CovarianceType::Cluster("g".into()))
      .is_err()
  );
  let mut one_cluster = problem.clone();
  one_cluster.sample.as_mut().unwrap().cluster_groups =
    Some(vec!["same".into(); problem.response.len()]);
  assert!(
    robust_anderson_rubin_confidence_set(&one_cluster, 95.0, &CovarianceType::Cluster("g".into()))
      .is_err()
  );

  let mut rank_deficient = problem.clone();
  for (row, exogenous) in rank_deficient
    .instruments
    .iter_mut()
    .zip(&rank_deficient.exogenous)
  {
    row[0] = exogenous[0];
  }
  assert!(
    robust_anderson_rubin_confidence_set(&rank_deficient, 95.0, &CovarianceType::RobustHc1)
      .unwrap_err()
      .to_string()
      .contains("rank deficient")
  );

  let mut zero_df = problem.clone();
  zero_df.response.truncate(3);
  zero_df.exogenous.truncate(3);
  zero_df.endogenous.truncate(3);
  zero_df.instruments.truncate(3);
  zero_df.sample = Some(EstimationSample::all_retained(3));
  assert!(matches!(
    robust_anderson_rubin_confidence_set(&zero_df, 95.0, &CovarianceType::RobustHc1),
    Err(StatsError::ZeroDegreesOfFreedom {
      observations: 3,
      parameters: 3
    })
  ));

  let mut zero_outcome = problem;
  zero_outcome.response.fill(0.0);
  assert!(
    robust_anderson_rubin_confidence_set(&zero_outcome, 95.0, &CovarianceType::RobustHc1)
      .unwrap_err()
      .to_string()
      .contains("degenerate")
  );
}
