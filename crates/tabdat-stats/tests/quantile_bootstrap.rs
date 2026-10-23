use tabdat_stats::{EstimationProblem, bootstrap_quantiles};

fn problem(case: &str) -> EstimationProblem {
  let rows: Vec<Vec<f64>> = include_str!("fixtures/quantile-bootstrap/inputs.csv")
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
fn joint_covariance_bias_and_percentiles_match_independent_common_draw_quantreg() {
  let mut comparisons = 0;
  for (case, seed) in [
    ("joint", u64::MAX),
    ("noconstant", 7),
    ("shifted", 7),
    ("rescaled", 7),
  ] {
    let p = problem(case);
    let fit = bootstrap_quantiles(&p, &[0.2, 0.5, 0.8], 40, seed).unwrap();
    assert_eq!(
      fit,
      bootstrap_quantiles(&p, &[0.2, 0.5, 0.8], 40, seed).unwrap()
    );
    // Adding quantiles consumes no extra RNG draws or changes marginal fits.
    let median = bootstrap_quantiles(&p, &[0.5], 40, seed).unwrap();
    let k = p.parameter_count();
    assert_eq!(fit.coefficients[k..2 * k], median.coefficients);
    for i in 0..k {
      for j in 0..k {
        assert_eq!(
          fit.covariance_matrix[k + i][k + j],
          median.covariance_matrix[i][j]
        );
      }
    }
    let value = serde_json::to_value(fit).unwrap();
    for line in include_str!("fixtures/quantile-bootstrap/reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = line.split(',').collect();
      if r[0] != case {
        continue;
      }
      let actual = value.pointer(r[1]).unwrap().as_f64().unwrap();
      let expected: f64 = r[2].parse().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-8,
        "{r:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 477);
}

#[test]
fn bounds_and_failed_original_or_replicate_are_explicit() {
  let p = problem("joint");
  for q in [
    vec![],
    vec![0.0],
    vec![f64::NAN],
    vec![0.5, 0.5],
    vec![0.8, 0.2],
    vec![0.5; 20],
  ] {
    assert!(bootstrap_quantiles(&p, &q, 40, 7).is_err());
  }
  for b in [0, 1, 5001, usize::MAX] {
    assert!(bootstrap_quantiles(&p, &[0.5], b, 7).is_err());
  }
  assert!(bootstrap_quantiles(&p.clone().with_weights(vec![1.; 120]), &[0.5], 40, 7).is_err());
  let small = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1., 2., 4., 8., 16.],
    vec![vec![0.], vec![1.], vec![2.], vec![3.], vec![4.]],
  )
  .unwrap();
  let error = bootstrap_quantiles(&small, &[0.5], 100, 1)
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  let mut rank = p.clone();
  rank.design_matrix = vec![vec![1., 1.]; 120];
  assert!(bootstrap_quantiles(&rank, &[0.5], 40, 7).is_err());
  let quantiles: Vec<_> = (1..=19).map(|i| i as f64 / 20.0).collect();
  assert!(bootstrap_quantiles(&p, &quantiles, 5000, 7).is_err());
}
