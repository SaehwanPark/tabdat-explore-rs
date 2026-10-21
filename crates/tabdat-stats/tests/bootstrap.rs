use tabdat_stats::{EstimationProblem, bootstrap_ols};
#[test]
fn seeded_bootstrap_matches_independent_draw_stream_r_lm_cov_and_type7_percentiles() {
  let rows: Vec<Vec<f64>> = include_str!("fixtures/bootstrap/inputs.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  let mut comparisons = 0;
  for (case, seed) in [("multiple", u64::MAX), ("noconstant", 7), ("shifted", 7)] {
    let p = EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      rows
        .iter()
        .map(|r| r[0] + if case == "shifted" { 100.0 } else { 0.0 })
        .collect(),
      rows.iter().map(|r| r[1..].to_vec()).collect(),
    )
    .unwrap()
    .with_intercept(case != "noconstant");
    let fit = bootstrap_ols(&p, 40, seed).unwrap();
    assert_eq!(fit, bootstrap_ols(&p, 40, seed).unwrap());
    assert_ne!(fit, bootstrap_ols(&p, 40, seed.wrapping_add(1)).unwrap());
    let value = serde_json::to_value(fit).unwrap();
    for line in include_str!("fixtures/bootstrap/reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = line.split(',').collect();
      if r[0] != case {
        continue;
      }
      let actual = value.pointer(r[1]).and_then(|v| v.as_f64()).unwrap();
      let expected: f64 = r[2].parse().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-9,
        "{r:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 76);
}

#[test]
fn bounds_weights_original_rank_and_failed_draws_are_explicit_without_retries() {
  let p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1., 4., 2., 8.],
    vec![vec![0.], vec![0.], vec![0.], vec![1.]],
  )
  .unwrap();
  let error = bootstrap_ols(&p, 100, 1).unwrap_err().to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  for reps in [0, 1, 5001, usize::MAX] {
    assert!(bootstrap_ols(&p, reps, 1).is_err());
  }
  assert!(bootstrap_ols(&p.clone().with_weights(vec![1.; 4]), 40, 1).is_err());
  let mut singular = p.clone();
  singular.design_matrix = vec![vec![0.]; 4];
  assert!(bootstrap_ols(&singular, 40, 1).is_err());
  let broad = EstimationProblem::new(
    "y",
    (0..20).map(|i| format!("x{i}")).collect(),
    vec![1.; 100],
    vec![vec![1.; 20]; 100],
  )
  .unwrap();
  assert!(
    bootstrap_ols(&broad, 5000, 1)
      .unwrap_err()
      .to_string()
      .contains("N * replications * K^2")
  );
}
