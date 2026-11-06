use tabdat_stats::{
  ConditionalPoissonOptions, EstimationProblem, EstimationSample, fit_conditional_poisson,
};
fn problem() -> EstimationProblem {
  EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0, 2.0, 2.0, 4.0, 3.0, 0.0, 0.0],
    vec![
      vec![0.0],
      vec![1.0],
      vec![0.0],
      vec![1.0],
      vec![7.0],
      vec![0.0],
      vec![1.0],
    ],
  )
  .unwrap()
  .with_intercept(false)
  .with_sample(
    EstimationSample::new(
      (0..7).collect(),
      8,
      1,
      None,
      Some(
        ["a", "a", "b", "b", "singleton", "zero", "zero"]
          .map(str::to_owned)
          .to_vec(),
      ),
    )
    .unwrap(),
  )
}
#[test]
fn closed_form_conditional_multinomial_information_exclusions_and_means() {
  let fit = fit_conditional_poisson(&problem(), "id", &Default::default()).unwrap();
  assert_eq!(
    (
      fit.observation_count,
      fit.group_count,
      fit.candidate_group_count
    ),
    (5, 3, 4)
  );
  assert_eq!(
    (fit.zero_total_group_count, fit.zero_total_observation_count),
    (1, 2)
  );
  assert_eq!(fit.sample.retained_indices, vec![0, 1, 2, 3, 4]);
  assert_eq!(fit.sample.dropped_observations, 3);
  assert!((fit.coefficients[0].value - 2.0_f64.ln()).abs() < 1e-10);
  assert!((fit.covariance[0][0] - 0.5).abs() < 1e-10);
  for (a, b) in fit.fitted_means.iter().zip([1., 2., 2., 4., 3.]) {
    assert!((a - b).abs() < 1e-10);
  }
  let ll = (4.0_f64 / 9.0).ln() + (240.0_f64 / 729.0).ln();
  assert!((fit.conditional_log_likelihood - ll).abs() < 1e-10);
  assert!(fit.max_abs_scaled_score <= 1e-10 && fit.max_abs_scaled_newton_step <= 1e-10);
}
#[test]
fn multiple_slopes_are_invariant_to_group_shifts_and_column_scaling() {
  let mut x = Vec::new();
  let mut y = Vec::new();
  let mut keys = Vec::new();
  for g in 0..12 {
    for t in 0..4 {
      x.push(vec![f64::from(t), f64::from((g * t + 3 * t * t) % 7)]);
      y.push(f64::from((g * 7 + t * 3) % 5 + 1));
      keys.push(g.to_string());
    }
  }
  let n = y.len();
  let p = EstimationProblem::new("y", vec!["x".into(), "z".into()], y, x)
    .unwrap()
    .with_intercept(false)
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap());
  let base = fit_conditional_poisson(&p, "id", &Default::default()).unwrap();
  let mut changed = p.clone();
  let factors = [8.0, 0.125];
  for (i, row) in changed.design_matrix.iter_mut().enumerate() {
    for j in 0..2 {
      row[j] = factors[j] * (row[j] + (i / 4) as f64 * 1024.0);
    }
  }
  let shifted = fit_conditional_poisson(&changed, "id", &Default::default()).unwrap();
  for j in 0..2 {
    assert!((base.coefficients[j].value - shifted.coefficients[j].value * factors[j]).abs() < 1e-9);
    for k in 0..2 {
      assert!(
        (base.covariance[j][k] - shifted.covariance[j][k] * factors[j] * factors[k]).abs() < 1e-9
      );
    }
  }
  for (a, b) in base.fitted_means.iter().zip(&shifted.fitted_means) {
    assert!((a - b).abs() < 1e-9);
  }
  assert!((base.conditional_log_likelihood - shifted.conditional_log_likelihood).abs() < 1e-8);
}

#[test]
fn failure_guards_do_not_return_separated_or_unidentified_fits() {
  let mut p = problem();
  p.response = [0., 3., 0., 6., 3., 0., 0.].to_vec();
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
  let mut p = problem();
  for row in &mut p.design_matrix {
    row[0] = 1.0;
  }
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
  assert!(
    fit_conditional_poisson(
      &problem(),
      "id",
      &ConditionalPoissonOptions { max_iterations: 0 }
    )
    .is_err()
  );
  let mut p = problem();
  p.response[0] = 0.5;
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
  let mut p = problem();
  p.sample.as_mut().unwrap().total_observations = 40001;
  p.sample.as_mut().unwrap().dropped_observations = 39994;
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
  let mut p = problem();
  p.response.fill(0.0);
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
  let mut p = problem();
  p.sample
    .as_mut()
    .unwrap()
    .cluster_groups
    .as_mut()
    .unwrap()
    .pop();
  assert!(fit_conditional_poisson(&p, "id", &Default::default()).is_err());
}
