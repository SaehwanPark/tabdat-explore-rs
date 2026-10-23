use tabdat_stats::{
  CovarianceType, InstrumentalVariablesProblem, IvOptions, fit_two_stage_least_squares,
  robust_overid_score_test,
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
