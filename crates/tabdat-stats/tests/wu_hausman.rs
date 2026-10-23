use tabdat_stats::{
  CovarianceType, EstimationSample, InstrumentalVariablesProblem, StatsError,
  robust_endogeneity_test, wu_hausman_test,
};

fn problem(m: usize, control: bool, intercept: bool, overid: bool) -> InstrumentalVariablesProblem {
  let mut p = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: if control { vec!["w".into()] } else { vec![] },
    endogenous_names: if m == 2 {
      vec!["x1".into(), "x2".into()]
    } else {
      vec!["x1".into()]
    },
    instrument_names: if m == 2 {
      vec!["z1".into(), "z2".into()]
    } else {
      vec!["z1".into()]
    },
    response: vec![],
    exogenous: vec![],
    endogenous: vec![],
    instruments: vec![],
    include_intercept: intercept,
    sample: None,
  };
  if overid {
    p.instrument_names.push("z3".into());
  }
  for i in 1..=40 {
    let b = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let (w, z1, z2, z3, v1, v2) = (b(8.0), b(1.0), b(2.0), b(3.0), b(4.0), b(5.0));
    let (x1, x2) = (z1 + v1, z2 + v2);
    let mut y = 1.0 + 0.5 * w + 1.5 * x1 + 0.6 * v1 + b(6.0);
    let mut x = vec![x1];
    let mut z = vec![z1];
    if m == 2 {
      y -= 0.7 * x2 + 0.4 * v2;
      x.push(x2);
      z.push(z2);
    }
    if overid {
      z.push(z3);
    }
    p.response.push(y);
    p.exogenous.push(if control { vec![w] } else { vec![] });
    p.endogenous.push(x);
    p.instruments.push(z);
  }
  p
}

#[test]
fn joint_f_matches_independent_r_including_just_identification_and_zero_control_designs() {
  // R4.6.1 fixtures/wu_hausman_reference.R. DF is N-K-m, not first-stage or IV DF.
  for (m, control, intercept, overid, df, statistic, p_value) in [
    (
      2,
      true,
      true,
      false,
      34,
      4.419999999999995,
      0.01966530710521241,
    ),
    (
      2,
      true,
      false,
      false,
      35,
      1.516666666666664,
      0.233514555545233,
    ),
    (
      1,
      false,
      false,
      false,
      38,
      2.104615384615372,
      0.1550605456744226,
    ),
    (
      1,
      true,
      true,
      true,
      36,
      6.479999999999987,
      0.01533570687687769,
    ),
    (
      2,
      false,
      true,
      true,
      35,
      3.639999999999997,
      0.03662930259783138,
    ),
  ] {
    let p = problem(m, control, intercept, overid);
    let before = p.clone();
    let result = wu_hausman_test(&p).unwrap();
    assert_eq!(result.endogenous, p.endogenous_names);
    assert_eq!(result.excluded_instruments, p.instrument_names);
    assert_eq!(result.observation_count, 40);
    assert_eq!(result.numerator_df, m);
    assert_eq!(result.denominator_df, df);
    assert!((result.statistic - statistic).abs() < 1e-12, "{result:?}");
    assert!((result.p_value - p_value).abs() < 1e-12, "{result:?}");
    assert_eq!(p, before);
    // Units do not change the auxiliary spaces or the exogeneity hypothesis.
    for scale in [1e-8, 1e8] {
      let mut scaled = p.clone();
      for row in &mut scaled.endogenous {
        row[0] *= scale;
      }
      for row in &mut scaled.instruments {
        row[0] *= scale;
      }
      assert!((wu_hausman_test(&scaled).unwrap().statistic - statistic).abs() < 1e-12);
    }
  }
}

#[test]
fn robust_joint_wald_matches_base_r_auxiliary_scores_across_five_designs() {
  // R 4.6.1 fixtures/robust_endogeneity_reference.R; absolute 1e-11 accounts
  // for independent QR/bread arithmetic, not a different finite-sample correction.
  for (m, control, intercept, overid, hc1_w, hc1_p, cr1_w, cr1_p) in [
    (
      2,
      true,
      true,
      false,
      8.839999999999989,
      0.01203423228472384,
      6.998166248971174,
      0.0302250833605348,
    ),
    (
      2,
      true,
      false,
      false,
      2.099999999999999,
      0.3499377491111555,
      0.7875891691668562,
      0.674492593188149,
    ),
    (
      1,
      false,
      false,
      false,
      1.710000000000002,
      0.1909853704997109,
      0.7102632140040858,
      0.3993564450769161,
    ),
    (
      1,
      true,
      true,
      true,
      6.48,
      0.01090949836426928,
      5.168918870622576,
      0.02299452234977532,
    ),
    (
      2,
      false,
      true,
      true,
      5.503448275862056,
      0.06381773571939689,
      5.453154679692124,
      0.06544289557737284,
    ),
  ] {
    let mut p = problem(m, control, intercept, overid);
    p.sample = Some(
      EstimationSample::new(
        (2..42).collect(),
        42,
        2,
        None,
        Some((0..40).map(|i| (i / 4).to_string()).collect()),
      )
      .unwrap(),
    );
    for (mode, expected_w, expected_p, groups) in [
      (CovarianceType::RobustHc1, hc1_w, hc1_p, None),
      (
        CovarianceType::Cluster("unit".into()),
        cr1_w,
        cr1_p,
        Some(10),
      ),
    ] {
      let before = p.clone();
      let result = robust_endogeneity_test(&p, &mode).unwrap();
      let k = usize::from(intercept) + usize::from(control) + 2 * m;
      let correction = groups.map_or(40.0 / (40 - k) as f64, |g| {
        g as f64 / (g - 1) as f64 * 39.0 / (40 - k) as f64
      });
      assert_eq!(result.endogenous, p.endogenous_names);
      assert_eq!(result.excluded_instruments, p.instrument_names);
      assert_eq!(result.observation_count, 40);
      assert_eq!(result.auxiliary_parameter_count, k);
      assert_eq!(result.auxiliary_residual_df, 40 - k);
      assert_eq!(result.degrees_of_freedom, m);
      assert_eq!(result.covariance_type, mode);
      assert_eq!(result.cluster_group_count, groups);
      assert!((result.covariance_correction - correction).abs() < 1e-14);
      assert!((result.statistic - expected_w).abs() < 1e-11, "{result:?}");
      assert!((result.p_value - expected_p).abs() < 1e-11, "{result:?}");
      assert_eq!(p, before);
      for scale in [1e-8, 1e8] {
        let mut scaled = p.clone();
        for row in &mut scaled.endogenous {
          row[0] *= scale;
        }
        for row in &mut scaled.instruments {
          row[0] *= scale;
        }
        assert!(
          (robust_endogeneity_test(&scaled, &mode).unwrap().statistic - expected_w).abs() < 1e-11
        );
      }
    }
  }
}

#[test]
fn robust_endogeneity_rejects_degenerate_innovations_variance_and_cluster_rank() {
  let p = problem(2, true, true, false);
  assert!(robust_endogeneity_test(&p, &CovarianceType::NonRobust).is_err());
  assert!(robust_endogeneity_test(&p, &CovarianceType::Cluster("unit".into())).is_err());
  let mut bad = p.clone();
  for (x, z) in bad.endogenous.iter_mut().zip(&bad.instruments) {
    x[0] = z[0];
  }
  assert!(
    robust_endogeneity_test(&bad, &CovarianceType::RobustHc1)
      .unwrap_err()
      .to_string()
      .contains("first-stage innovation")
  );
  bad = p.clone();
  bad.response = bad.endogenous.iter().map(|x| x[0]).collect();
  assert!(
    robust_endogeneity_test(&bad, &CovarianceType::RobustHc1)
      .unwrap_err()
      .to_string()
      .contains("residual variance")
  );
  bad = p;
  let mut sample = EstimationSample::all_retained(40);
  sample.cluster_groups = Some((0..40).map(|i| (i / 20).to_string()).collect());
  bad.sample = Some(sample);
  assert!(
    robust_endogeneity_test(&bad, &CovarianceType::Cluster("unit".into()))
      .unwrap_err()
      .to_string()
      .contains("Wald")
  );
}

#[test]
fn null_and_internal_name_collisions_preserve_the_intended_joint_test() {
  let mut p = problem(2, true, true, false);
  p.exogenous_names[0] = "__wu_innovation_0".into();
  p.outcome = "__wu_innovation_1".into();
  assert!((wu_hausman_test(&p).unwrap().statistic - 4.42).abs() < 1e-12);
  for (index, y) in p.response.iter_mut().enumerate() {
    let b = |k: f64| (2.0 * std::f64::consts::PI * k * (index + 1) as f64 / 40.0).cos();
    *y -= 0.6 * b(4.0) - 0.4 * b(5.0);
  }
  let result = wu_hausman_test(&p).unwrap();
  assert!(result.statistic < 1e-12);
  assert!(result.p_value > 1.0 - 1e-10);
}

#[test]
fn innovation_rank_df_and_auxiliary_variance_failures_are_explicit() {
  let mut perfect = problem(1, true, true, true);
  for (row, z) in perfect.endogenous.iter_mut().zip(&perfect.instruments) {
    row[0] = 1e8 * z[0];
  }
  assert!(
    wu_hausman_test(&perfect)
      .unwrap_err()
      .to_string()
      .contains("first-stage innovation")
  );
  let mut shared = problem(2, true, true, false);
  for (index, row) in shared.endogenous.iter_mut().enumerate() {
    let v = (2.0 * std::f64::consts::PI * 4.0 * (index + 1) as f64 / 40.0).cos();
    row[1] = shared.instruments[index][1] + 2.0 * v;
  }
  assert!(wu_hausman_test(&shared).is_err());
  let mut variance = problem(1, true, true, true);
  variance.response = variance.endogenous.iter().map(|row| row[0]).collect();
  assert!(
    wu_hausman_test(&variance)
      .unwrap_err()
      .to_string()
      .contains("residual variance")
  );
  variance.response = vec![1e-20; 40];
  assert!(
    wu_hausman_test(&variance)
      .unwrap_err()
      .to_string()
      .contains("residual variance")
  );
  let mut df = problem(2, true, true, false);
  df.response.truncate(6);
  df.exogenous.truncate(6);
  df.endogenous.truncate(6);
  df.instruments.truncate(6);
  assert!(matches!(
    wu_hausman_test(&df),
    Err(StatsError::ZeroDegreesOfFreedom {
      observations: 6,
      parameters: 6
    })
  ));
  let mut rank = problem(2, true, true, false);
  for row in &mut rank.instruments {
    row[1] = row[0];
  }
  assert!(wu_hausman_test(&rank).is_err());
  let mut nonfinite = problem(1, true, true, true);
  nonfinite.response[0] = f64::NAN;
  assert!(matches!(
    wu_hausman_test(&nonfinite),
    Err(StatsError::NonFiniteValue(_))
  ));
}
