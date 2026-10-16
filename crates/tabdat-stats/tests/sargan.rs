use tabdat_stats::{
  InstrumentalVariablesProblem, IvOptions, fit_two_stage_least_squares, sargan_test,
};

fn problem() -> InstrumentalVariablesProblem {
  let mut p = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x1".into(), "x2".into()],
    instrument_names: vec!["z1".into(), "z2".into(), "z3".into()],
    response: vec![],
    exogenous: vec![],
    endogenous: vec![],
    instruments: vec![],
    include_intercept: true,
    sample: None,
  };
  for i in 1..=40 {
    let b = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let (w, z1, z2, z3, x1, x2) = (
      b(8.0),
      b(1.0),
      b(2.0),
      b(3.0),
      b(1.0) + b(4.0),
      b(2.0) + b(5.0),
    );
    p.response
      .push(1.0 + 0.5 * w + 1.5 * x1 - 0.7 * x2 + 0.3 * z3 + b(6.0));
    p.exogenous.push(vec![w]);
    p.endogenous.push(vec![x1, x2]);
    p.instruments.push(vec![z1, z2, z3]);
  }
  p
}

#[test]
fn multiple_endogenous_sargan_and_noconstant_match_independent_r() {
  for (intercept, extra, statistic, p_value) in [
    (true, false, 3.30275229357798, 0.06916390601014773),
    (false, false, 1.165048543689321, 0.2804209722293698),
    (true, true, 4.601769911504431, 0.1001701583294531),
    (false, true, 1.661341853035147, 0.4357568274125628),
  ] {
    let mut p = problem();
    p.include_intercept = intercept;
    if extra {
      p.instrument_names.push("z4".into());
      for (index, row) in p.instruments.iter_mut().enumerate() {
        let z4 = (2.0 * std::f64::consts::PI * 7.0 * (index + 1) as f64 / 40.0).cos();
        row.push(z4);
        p.response[index] += 0.2 * z4;
      }
    }
    let fit = fit_two_stage_least_squares(&p, &IvOptions::default()).unwrap();
    let original = p.clone();
    let result = sargan_test(&p, &fit.regression.residuals).unwrap();
    assert_eq!(result.observation_count, 40);
    assert_eq!(result.degrees_of_freedom, if extra { 2 } else { 1 });
    assert_eq!(result.endogenous, ["x1", "x2"]);
    assert_eq!(result.excluded_instruments, p.instrument_names);
    // R 4.6.1 fixtures/sargan_reference.R; uncentered N*R² for noconstant.
    assert!((result.statistic - statistic).abs() < 1e-12);
    assert!((result.p_value - p_value).abs() < 1e-12);
    let scaled = fit
      .regression
      .residuals
      .iter()
      .map(|u| u * 10.0)
      .collect::<Vec<_>>();
    assert!((sargan_test(&p, &scaled).unwrap().statistic - statistic).abs() < 1e-12);
    assert_eq!(p, original);
  }
}

#[test]
fn undefined_identification_variance_dimensions_and_rank_fail() {
  let p = problem();
  let residuals = fit_two_stage_least_squares(&p, &IvOptions::default())
    .unwrap()
    .regression
    .residuals;
  let mut just_identified = p.clone();
  just_identified.instrument_names.pop();
  for row in &mut just_identified.instruments {
    row.pop();
  }
  assert!(
    sargan_test(&just_identified, &residuals)
      .unwrap_err()
      .to_string()
      .contains("requires overidentification")
  );
  assert!(
    sargan_test(&p, &residuals[..39])
      .unwrap_err()
      .to_string()
      .contains("sample")
  );
  assert!(
    sargan_test(&p, &[0.0; 40])
      .unwrap_err()
      .to_string()
      .contains("variance")
  );
  let mut nonfinite = residuals.clone();
  nonfinite[0] = f64::NAN;
  assert!(sargan_test(&p, &nonfinite).is_err());
  assert!(
    sargan_test(&p, &[f64::MAX; 40])
      .unwrap_err()
      .to_string()
      .contains("non-finite")
  );
  let mut rank = p.clone();
  for row in &mut rank.instruments {
    row[2] = row[0];
  }
  assert!(sargan_test(&rank, &residuals).is_err());
  let mut saturated = p;
  saturated.response.truncate(5);
  saturated.exogenous.truncate(5);
  saturated.endogenous.truncate(5);
  saturated.instruments.truncate(5);
  assert!(
    sargan_test(&saturated, &residuals[..5])
      .unwrap_err()
      .to_string()
      .contains("degrees of freedom")
  );
}
