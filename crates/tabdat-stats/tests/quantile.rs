use tabdat_stats::{EstimationProblem, fit_quantile_iid};
fn problem(case: &str) -> EstimationProblem {
  let rows: Vec<Vec<f64>> = include_str!("fixtures/quantile/inputs.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "z".into()],
    rows
      .iter()
      .map(|r| r[0] + if case == "shifted" { 100.0 } else { 0.0 })
      .collect(),
    rows
      .iter()
      .map(|r| vec![r[1] * if case == "rescaled" { 100.0 } else { 1.0 }, r[2]])
      .collect(),
  )
  .unwrap()
  .with_intercept(case != "noconstant")
}
#[test]
fn point_sparsity_covariance_and_normal_inference_match_independent_quantreg() {
  let mut compared = 0;
  for (case, tau) in [
    ("q10", 0.1),
    ("median", 0.5),
    ("q90", 0.9),
    ("noconstant", 0.5),
    ("shifted", 0.5),
    ("rescaled", 0.5),
  ] {
    let p = problem(case);
    let result = fit_quantile_iid(&p, tau).unwrap();
    assert!(result.relative_duality_gap <= 1e-8 && result.max_score_violation <= 1e-7);
    let value = serde_json::to_value(result).unwrap();
    for l in include_str!("fixtures/quantile/reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = l.split(',').collect();
      if r[0] != case {
        continue;
      }
      let a = value.pointer(r[1]).unwrap().as_f64().unwrap();
      let e: f64 = r[2].parse().unwrap();
      let tolerance = if r[1].ends_with("p_value") {
        e.abs() * 1e-7 + 1e-300
      } else {
        1e-8
      };
      assert!(
        (a - e).abs() <= tolerance,
        "{r:?}: {a} != {e}; tolerance {tolerance}"
      );
      compared += 1;
    }
  }
  assert_eq!(compared, 185);
}
#[test]
fn unsupported_weights_quantiles_rank_nonisolated_and_zero_error_samples_fail() {
  let p = problem("median");
  for tau in [f64::NAN, 0., 1., 0.01, 0.99] {
    assert!(fit_quantile_iid(&p, tau).is_err());
  }
  assert!(fit_quantile_iid(&p.clone().with_weights(vec![1.; 120]), 0.5).is_err());
  let mut rank = p.clone();
  rank.design_matrix = vec![vec![1., 1.]; 120];
  assert!(fit_quantile_iid(&rank, 0.5).is_err());
  let mut constant = p.clone();
  constant.response = vec![1.; 120];
  assert!(fit_quantile_iid(&constant, 0.5).is_err());
  let face = EstimationProblem::new(
    "y",
    vec!["x".into()],
    (1..=80).map(|i| i as f64).collect(),
    (0..80).map(|i| vec![(i % 2) as f64]).collect(),
  )
  .unwrap();
  assert!(
    fit_quantile_iid(&face, 0.5)
      .unwrap_err()
      .to_string()
      .contains("not isolated")
  );
  let big = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.; 10001],
    vec![vec![1.]; 10001],
  )
  .unwrap();
  assert!(
    fit_quantile_iid(&big, 0.5)
      .unwrap_err()
      .to_string()
      .contains("N<=10000")
  );
}

#[test]
fn predictor_rescaling_keeps_inference_in_original_units() {
  let p = problem("median");
  let original = fit_quantile_iid(&p, 0.5).unwrap();
  let mut tiny = p.clone();
  for r in &mut tiny.design_matrix {
    r[0] *= 1e-14;
  }
  let fitted = fit_quantile_iid(&tiny, 0.5).unwrap();
  for (i, scale) in [1.0, 1e-14, 1.0].iter().enumerate() {
    assert!((fitted.coefficients[i].value * scale - original.coefficients[i].value).abs() < 1e-9);
    assert!(
      (fitted.coefficients[i].standard_error * scale - original.coefficients[i].standard_error)
        .abs()
        < 1e-9
    );
    for (j, other) in [1.0, 1e-14, 1.0].iter().enumerate() {
      assert!(
        (fitted.covariance_matrix[i][j] * scale * other - original.covariance_matrix[i][j]).abs()
          < 1e-9
      );
    }
  }
}
