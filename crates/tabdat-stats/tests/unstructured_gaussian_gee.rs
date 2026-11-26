use tabdat_stats::{
  EstimationProblem, EstimationSample, UnstructuredGaussianGeeOptions,
  fit_unstructured_gaussian_gee,
};

fn synthetic(period_count: usize, group_count: usize) -> (EstimationProblem, Vec<usize>, Vec<f64>) {
  let mut response = Vec::new();
  let mut design = Vec::new();
  let mut keys = Vec::new();
  let mut retained = Vec::new();
  let mut periods = Vec::new();
  for group in 0..group_count {
    for period in 0..period_count {
      let x = (group as f64 * 0.17).sin() + period as f64 * 0.13;
      let seed = (group as u64 * 1_000_003 + period as u64 * 97_409 + 1) % 2_147_483_647;
      let uniform = (seed * 48_271 % 2_147_483_647) as f64 / 2_147_483_647.0;
      let noise = 0.5 * (uniform - 0.5);
      design.push(vec![x]);
      response.push(1.4 + 0.35 * x + noise);
      keys.push(format!("g{group:03}"));
      retained.push(group * period_count + period);
      periods.push(period);
    }
  }
  let n = response.len();
  let sample = EstimationSample::new(retained, n, 0, None, Some(keys)).unwrap();
  let problem = EstimationProblem::new("y", vec!["x".into()], response, design)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(sample);
  (
    problem,
    periods,
    (0..period_count)
      .map(|period| period as f64 + 2000.0)
      .collect(),
  )
}

#[test]
fn balanced_two_and_ten_period_panels_fit_with_symmetric_unit_diagonal_r() {
  for period_count in [2, 10] {
    let (problem, periods, time_values) = synthetic(period_count, 80);
    let fit = fit_unstructured_gaussian_gee(
      &problem,
      "id",
      "year",
      &time_values,
      &periods,
      &UnstructuredGaussianGeeOptions::default(),
    )
    .unwrap_or_else(|error| panic!("T={period_count}: {error}"));
    assert_eq!(fit.observation_count, 80 * period_count);
    assert_eq!(fit.group_count, 80);
    assert_eq!(fit.period_count, period_count);
    assert!(fit.phi.is_finite() && fit.phi > 0.0);
    assert!(fit.max_abs_scaled_score <= 1e-6);
    assert_eq!(fit.correlation_matrix.len(), period_count);
    for row in 0..period_count {
      assert_eq!(fit.correlation_matrix[row][row], 1.0);
      for column in 0..period_count {
        assert_eq!(
          fit.correlation_matrix[row][column],
          fit.correlation_matrix[column][row]
        );
      }
    }
    assert!(fit.working.covariance != fit.sandwich.covariance);
    assert_eq!(fit.fitted_means.len(), fit.observation_count);
    assert_eq!(
      fit.sample.retained_indices,
      (0..fit.observation_count).collect::<Vec<_>>()
    );
  }
}

#[test]
fn physical_row_permutation_preserves_fit_and_maps_fitted_values_to_that_order() {
  let (problem, periods, time_values) = synthetic(4, 100);
  let baseline = fit_unstructured_gaussian_gee(
    &problem,
    "id",
    "year",
    &time_values,
    &periods,
    &Default::default(),
  )
  .unwrap();
  let n = problem.response.len();
  let permutation = (0..n).rev().collect::<Vec<_>>();
  let response = permutation
    .iter()
    .map(|&index| problem.response[index])
    .collect::<Vec<_>>();
  let design = permutation
    .iter()
    .map(|&index| problem.design_matrix[index].clone())
    .collect::<Vec<_>>();
  let old_sample = problem.sample.as_ref().unwrap();
  let keys = permutation
    .iter()
    .map(|&index| old_sample.cluster_groups.as_ref().unwrap()[index].clone())
    .collect::<Vec<_>>();
  let sample = EstimationSample::new(permutation.clone(), n, 0, None, Some(keys)).unwrap();
  let permuted_problem = EstimationProblem::new("y", vec!["x".into()], response, design)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(sample);
  let permuted_periods = permutation
    .iter()
    .map(|&index| periods[index])
    .collect::<Vec<_>>();
  let permuted = fit_unstructured_gaussian_gee(
    &permuted_problem,
    "id",
    "year",
    &time_values,
    &permuted_periods,
    &Default::default(),
  )
  .unwrap();
  for (actual, expected) in permuted
    .working
    .coefficients
    .iter()
    .map(|coefficient| coefficient.value)
    .zip(
      baseline
        .working
        .coefficients
        .iter()
        .map(|coefficient| coefficient.value),
    )
  {
    assert!((actual - expected).abs() < 1e-10);
  }
  for (actual, expected) in permuted
    .correlation_matrix
    .iter()
    .flatten()
    .zip(baseline.correlation_matrix.iter().flatten())
  {
    assert!((actual - expected).abs() < 1e-10);
  }
  for (actual, source_index) in permuted.fitted_means.iter().zip(&permutation) {
    assert!((actual - baseline.fitted_means[*source_index]).abs() < 1e-10);
  }
}

#[test]
fn incomplete_duplicate_period_and_singular_r_fail_explicitly() {
  let (problem, mut periods, time_values) = synthetic(3, 40);
  let mut incomplete = periods.clone();
  incomplete[0] = 1;
  assert!(
    fit_unstructured_gaussian_gee(
      &problem,
      "id",
      "year",
      &time_values,
      &incomplete,
      &Default::default(),
    )
    .is_err()
  );

  periods.fill(0);
  assert!(
    fit_unstructured_gaussian_gee(
      &problem,
      "id",
      "year",
      &time_values,
      &periods,
      &Default::default(),
    )
    .is_err()
  );

  let group_count = 40;
  let period_count = 3;
  let mut response = Vec::new();
  let mut design = Vec::new();
  let mut keys = Vec::new();
  let mut indices = Vec::new();
  let mut time = Vec::new();
  for group in 0..group_count {
    for period in 0..period_count {
      let x = period as f64;
      response.push(3.0 + 0.4 * x + (group as f64 - 19.5));
      design.push(vec![x]);
      keys.push(format!("g{group:03}"));
      indices.push(period);
      time.push(group * period_count + period);
    }
  }
  let n = response.len();
  let sample = EstimationSample::new(time, n, 0, None, Some(keys)).unwrap();
  let singular_r = EstimationProblem::new("y", vec!["x".into()], response, design)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(sample);
  let error = fit_unstructured_gaussian_gee(
    &singular_r,
    "id",
    "year",
    &[0.0, 1.0, 2.0],
    &indices,
    &Default::default(),
  )
  .unwrap_err();
  assert!(error.to_string().contains("positive definite"), "{error}");
}
