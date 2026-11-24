use tabdat_stats::{
  CovarianceType, InstrumentalVariablesProblem, IvOptions, cluster_robust_overid_score_test,
  fit_two_stage_least_squares, robust_overid_score_test,
};

fn problem(m: usize, control: bool, intercept: bool, extra: bool) -> InstrumentalVariablesProblem {
  let mut p = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: if control { vec!["w".into()] } else { vec![] },
    endogenous_names: (1..=m).map(|j| format!("x{j}")).collect(),
    instrument_names: (1..=if extra { 4 } else { 3 })
      .map(|j| format!("z{j}"))
      .collect(),
    response: vec![],
    exogenous: vec![],
    endogenous: vec![],
    instruments: vec![],
    include_intercept: intercept,
    sample: None,
  };
  for i in 1..=40 {
    let b = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let (w, z1, z2, z3, z4) = (b(8.0), b(1.0), b(2.0), b(3.0), b(7.0));
    let x1 = 0.7 * z1 + 0.4 * z2 - 0.2 * z3 + b(4.0);
    let x2 = -0.3 * z1 + 1.2 * z2 + 0.5 * z3 + b(5.0);
    let mut y = 1.0 + 0.5 * w + 1.5 * x1 + 0.3 * z3 + (1.0 + 0.2 * w) * b(6.0);
    if m == 2 {
      y -= 0.7 * x2;
    }
    if extra {
      y += 0.2 * z4;
    }
    p.response.push(y);
    p.exogenous.push(if control { vec![w] } else { vec![] });
    p.endogenous
      .push(if m == 2 { vec![x1, x2] } else { vec![x1] });
    let mut z = vec![z1, z2, z3];
    if extra {
      z.push(z4);
    }
    p.instruments.push(z);
  }
  p
}

fn residuals(p: &InstrumentalVariablesProblem) -> Vec<f64> {
  fit_two_stage_least_squares(
    p,
    &IvOptions {
      covariance_type: CovarianceType::RobustHc1,
    },
  )
  .unwrap()
  .regression
  .residuals
}

#[test]
fn independent_r_svd_references_cover_complement_df_and_noconstant() {
  // Base R4.6.1 fixtures/robust_overid_score_reference.R uses SVD rather than
  // native pivoted Gram-Schmidt; 1e-11 absolute covers independent QR arithmetic.
  for (m, control, intercept, extra, expected, tail) in [
    (2, true, true, false, 1.976698041361932, 0.1597386938377605),
    (
      2,
      true,
      false,
      false,
      0.4868500290981139,
      0.4853358091691459,
    ),
    (2, true, true, true, 3.337814008438926, 0.1884529313718826),
    (2, true, false, true, 0.7636487091503886, 0.6826149408674895),
    (
      1,
      false,
      false,
      false,
      0.7466632366724965,
      0.6884368966875288,
    ),
  ] {
    let p = problem(m, control, intercept, extra);
    let before = p.clone();
    let u = residuals(&p);
    let result = robust_overid_score_test(&p, &u).unwrap();
    let k = usize::from(control) + usize::from(intercept) + m;
    let l = usize::from(control) + usize::from(intercept) + p.instrument_names.len();
    assert_eq!(result.endogenous, p.endogenous_names);
    assert_eq!(result.excluded_instruments, p.instrument_names);
    assert_eq!(result.observation_count, 40);
    assert_eq!(result.structural_parameter_count, k);
    assert_eq!(result.instrument_parameter_count, l);
    assert_eq!(result.degrees_of_freedom, l - k);
    assert_eq!(result.covariance_correction, 1.0);
    assert_eq!(result.cluster_variable, None);
    assert_eq!(result.cluster_group_count, None);
    assert!((result.statistic - expected).abs() < 1e-11, "{result:?}");
    assert!((result.p_value - tail).abs() < 1e-11, "{result:?}");
    assert_eq!(p, before);
    // Changing instrument coordinates or residual units must not select a
    // different moment hypothesis. Rotation also catches arbitrary-q selection.
    let mut rotated = p.clone();
    rotated.instrument_names.reverse();
    for row in &mut rotated.instruments {
      row[0] += 0.3 * row[1];
      row[2] -= 0.2 * row[1];
      row.reverse();
    }
    assert!((robust_overid_score_test(&rotated, &u).unwrap().statistic - expected).abs() < 1e-11);
    for scale in [1e-120, 1e120, -1.0] {
      let scaled = u.iter().map(|v| v * scale).collect::<Vec<_>>();
      assert!((robust_overid_score_test(&p, &scaled).unwrap().statistic - expected).abs() < 1e-11);
    }
    let mut scaled = p.clone();
    for row in &mut scaled.instruments {
      row[0] *= 1e-100;
    }
    for row in &mut scaled.endogenous {
      row[0] *= 1e100;
    }
    assert!((robust_overid_score_test(&scaled, &u).unwrap().statistic - expected).abs() < 1e-11);
  }
}

fn clustered_problem(
  m: usize,
  control: bool,
  intercept: bool,
  extra: bool,
) -> InstrumentalVariablesProblem {
  let mut problem = problem(m, control, intercept, extra);
  let group_shocks = [-0.9, -0.5, -0.2, 0.6, 1.1, 0.8, 0.1, -0.7, -0.4, 0.1];
  let n = problem.response.len();
  for (index, response) in problem.response.iter_mut().enumerate() {
    *response += 0.25 * group_shocks[index / 4];
  }
  problem.sample = Some(
    tabdat_stats::sample::EstimationSample::new(
      (0..n).collect(),
      n,
      0,
      None,
      Some((0..n).map(|index| format!("g{}", index / 4)).collect()),
    )
    .unwrap(),
  );
  problem
}

#[test]
fn clustered_grouped_scores_match_independent_r_cr0_references() {
  // Base R4.6.2 fixture in robust_overid_score_reference.R uses a separate
  // SVD and grouped score crossproduct. Tolerance covers QR/SVD arithmetic.
  for (m, control, intercept, extra, expected, tail) in [
    (2, true, true, false, 0.5767918743903001, 0.4475725583094755),
    (2, true, true, true, 0.9078062533814221, 0.6351442587920135),
    (1, false, true, false, 2.94293227537329, 0.2295886297663257),
  ] {
    let p = clustered_problem(m, control, intercept, extra);
    let before = p.clone();
    let fit = fit_two_stage_least_squares(
      &p,
      &IvOptions {
        covariance_type: CovarianceType::Cluster("group_id".into()),
      },
    )
    .unwrap();
    let result =
      cluster_robust_overid_score_test(&p, &fit.regression.residuals, "group_id").unwrap();
    assert_eq!(result.observation_count, 40);
    assert_eq!(result.degrees_of_freedom, p.instrument_names.len() - m);
    assert_eq!(result.covariance_correction, 1.0);
    assert_eq!(result.cluster_variable.as_deref(), Some("group_id"));
    assert_eq!(result.cluster_group_count, Some(10));
    assert!((result.statistic - expected).abs() < 1e-11, "{result:?}");
    assert!((result.p_value - tail).abs() < 1e-11, "{result:?}");
    assert_eq!(p, before);

    // Basis rotation, residual units and cluster labels cannot change the test.
    let mut rotated = p.clone();
    rotated.instrument_names.reverse();
    for row in &mut rotated.instruments {
      row[0] += 0.3 * row[1];
      row[2] -= 0.2 * row[1];
      row.reverse();
    }
    let rotated_residuals = fit_two_stage_least_squares(
      &rotated,
      &IvOptions {
        covariance_type: CovarianceType::Cluster("group_id".into()),
      },
    )
    .unwrap()
    .regression
    .residuals;
    assert!(
      (cluster_robust_overid_score_test(&rotated, &rotated_residuals, "group_id")
        .unwrap()
        .statistic
        - expected)
        .abs()
        < 1e-11
    );
    for scale in [1e-120, 1e120, -1.0] {
      let scaled = fit
        .regression
        .residuals
        .iter()
        .map(|value| value * scale)
        .collect::<Vec<_>>();
      assert!(
        (cluster_robust_overid_score_test(&p, &scaled, "group_id")
          .unwrap()
          .statistic
          - expected)
          .abs()
          < 1e-11
      );
    }
    let mut relabeled = p.clone();
    for group in relabeled
      .sample
      .as_mut()
      .unwrap()
      .cluster_groups
      .as_mut()
      .unwrap()
    {
      *group = format!("renamed-{}", group.trim_start_matches('g'));
    }
    assert!(
      (cluster_robust_overid_score_test(&relabeled, &fit.regression.residuals, "renamed")
        .unwrap()
        .statistic
        - expected)
        .abs()
        < 1e-11
    );
  }
}

#[test]
fn clustered_score_requires_retained_groups_and_sufficient_group_rank() {
  let p = clustered_problem(1, false, true, false);
  let u = residuals(&p);
  assert!(
    robust_overid_score_test(&p, &u)
      .unwrap_err()
      .to_string()
      .contains("retained cluster identifiers require the clustered score test")
  );
  let mut missing = p.clone();
  missing.sample = None;
  assert!(
    cluster_robust_overid_score_test(&missing, &u, "group_id")
      .unwrap_err()
      .to_string()
      .contains("retained cluster identifiers")
  );

  let mut insufficient = p.clone();
  insufficient.sample.as_mut().unwrap().cluster_groups =
    Some((0..40).map(|index| format!("g{}", index % 2)).collect());
  assert!(
    cluster_robust_overid_score_test(&insufficient, &u, "group_id")
      .unwrap_err()
      .to_string()
      .contains("more clusters than test degrees of freedom")
  );

  let mut mismatched = p;
  mismatched.sample.as_mut().unwrap().cluster_groups = Some(vec!["g0".into(); 39]);
  assert!(
    cluster_robust_overid_score_test(&mismatched, &u, "group_id")
      .unwrap_err()
      .to_string()
      .contains("cluster identifier count")
  );
}

#[test]
fn orthogonal_null_moments_give_a_zero_score_without_cancellation() {
  let mut p = problem(2, true, true, true);
  for (index, y) in p.response.iter_mut().enumerate() {
    let error = (2.0 * std::f64::consts::PI * 6.0 * (index + 1) as f64 / 40.0).cos();
    *y = 1.0 + 0.5 * p.exogenous[index][0] + 1.5 * p.endogenous[index][0]
      - 0.7 * p.endogenous[index][1]
      + error;
  }
  let test = robust_overid_score_test(&p, &residuals(&p)).unwrap();
  assert_eq!(test.degrees_of_freedom, 2);
  assert!(test.statistic < 1e-20, "{test:?}");
  assert!(test.p_value > 1.0 - 1e-12, "{test:?}");
}

#[test]
fn identification_residual_dimensions_finiteness_and_score_rank_fail_explicitly() {
  let p = problem(2, true, true, false);
  let u = residuals(&p);
  let mut just = p.clone();
  just.instrument_names.pop();
  for row in &mut just.instruments {
    row.pop();
  }
  assert!(
    robust_overid_score_test(&just, &u)
      .unwrap_err()
      .to_string()
      .contains("requires overidentification")
  );
  assert!(
    robust_overid_score_test(&p, &u[..39])
      .unwrap_err()
      .to_string()
      .contains("sample")
  );
  assert!(
    robust_overid_score_test(&p, &[0.0; 40])
      .unwrap_err()
      .to_string()
      .contains("variance")
  );
  let mut bad = u.clone();
  bad[0] = f64::NAN;
  assert!(robust_overid_score_test(&p, &bad).is_err());
  let mut rank = p.clone();
  for row in &mut rank.instruments {
    row[2] = row[0];
  }
  assert!(robust_overid_score_test(&rank, &u).is_err());
  let mut unidentified = p.clone();
  for row in &mut unidentified.endogenous {
    row[1] = row[0];
  }
  assert!(robust_overid_score_test(&unidentified, &u).is_err());
  let mut saturated = p;
  saturated.response.truncate(5);
  saturated.exogenous.truncate(5);
  saturated.endogenous.truncate(5);
  saturated.instruments.truncate(5);
  assert!(robust_overid_score_test(&saturated, &u[..5]).is_err());
  let q2 = problem(2, true, true, true);
  // Both products have support on only one row, so the q=2 score space is rank1.
  let mut sparse = vec![0.0; 40];
  sparse[0] = 1.0;
  assert!(robust_overid_score_test(&q2, &sparse).is_err());
}
