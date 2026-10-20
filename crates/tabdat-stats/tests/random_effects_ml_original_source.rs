use tabdat_stats::{EstimationProblem, EstimationSample, fit_random_effects_ml_interior};
#[test]
#[ignore = "manual original MOM source probe; verify documented hash and set TABDAT_REML_SOURCE"]
fn original_mom_ml_matches_stationary_r_likelihood_and_joint_information() {
  let path = std::env::var("TABDAT_REML_SOURCE").unwrap();
  let text = std::fs::read_to_string(path).unwrap();
  let v: Vec<f64> = text
    .split_whitespace()
    .map(|s| s.parse().unwrap())
    .collect();
  assert_eq!(v.len(), 5320 * 8);
  for (i, row) in v.as_chunks::<8>().0.iter().enumerate() {
    assert!(row.iter().all(|v| v.is_finite()));
    assert_eq!(row[6], (i / 10 + 1) as f64);
    assert_eq!(row[7], (i % 10 + 1979) as f64);
  }
  let p = EstimationProblem::new(
    "lnhr",
    vec!["lnwg".into()],
    v.as_chunks::<8>().0.iter().map(|r| r[0]).collect(),
    v.as_chunks::<8>().0.iter().map(|r| vec![r[1]]).collect(),
  )
  .unwrap()
  .with_sample(
    EstimationSample::new(
      (0..5320).collect(),
      5320,
      0,
      None,
      Some(
        v.as_chunks::<8>()
          .0
          .iter()
          .map(|r| r[6].to_string())
          .collect(),
      ),
    )
    .unwrap(),
  );
  let r = fit_random_effects_ml_interior(&p, "id").unwrap();
  assert!(
    (r.log_likelihood - (-266.91149706378894)).abs() < 1e-9,
    "{:?}",
    r.log_likelihood
  );
  assert!((r.regression.coefficients[1].value - 0.11954740235722944).abs() < 1e-9);
  assert!(
    (r.regression.coefficients[1].standard_error.unwrap() - 0.013748574924515727).abs() < 1e-9
  );
  assert!((r.regression.coefficients[0].value - 7.345479144924143).abs() < 1e-9);
  assert!((r.null_log_likelihood - (-304.979_813_611_042_2)).abs() < 1e-9);
  assert!((r.likelihood_ratio - 76.13663309450647).abs() < 1e-9);
  assert!((r.sigma_e_squared - 0.0542504082187426).abs() < 1e-9);
  assert!((r.sigma_u_squared - 0.026300722026628514).abs() < 1e-9);
  println!(
    "MOM: beta={:?}; SE={:?}; LL={}; null LL={}; LR={}; eta={}",
    r.regression
      .coefficients
      .iter()
      .map(|c| c.value)
      .collect::<Vec<_>>(),
    r.regression
      .coefficients
      .iter()
      .map(|c| c.standard_error)
      .collect::<Vec<_>>(),
    r.log_likelihood,
    r.null_log_likelihood,
    r.likelihood_ratio,
    r.eta
  );
}
