use tabdat_stats::{
  CountProbabilityLaw as Law, count_point_cdf_standard_error, count_point_probabilities,
};

fn covariance_poisson() -> Vec<Vec<f64>> {
  vec![
    vec![0.04, 0.005, -0.002],
    vec![0.005, 0.01, 0.001],
    vec![-0.002, 0.001, 0.03],
  ]
}

fn covariance_nb2() -> Vec<Vec<f64>> {
  vec![
    vec![0.04, 0.005, -0.002, 0.001],
    vec![0.005, 0.01, 0.001, -0.001],
    vec![-0.002, 0.001, 0.03, 0.002],
    vec![0.001, -0.001, 0.002, 0.02],
  ]
}

#[test]
fn pointwise_cdf_delta_se_matches_independent_base_r_finite_differences() {
  let design = [1.0, -0.7, 1.2];
  let rows: Vec<_> = include_str!("fixtures/count_cdf_se_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .collect();
  assert_eq!(rows.len(), 15);
  for row in rows {
    let mean: f64 = row[1].parse().unwrap();
    let alpha: f64 = row[2].parse().unwrap_or(f64::NAN);
    let count: usize = row[3].parse().unwrap();
    let expected_probability: f64 = row[4].parse().unwrap();
    let expected_se: f64 = row[5].parse().unwrap();
    let (law, covariance) = match row[0] {
      "poisson" => (Law::Poisson, covariance_poisson()),
      "nb2" => (Law::Nb2 { alpha }, covariance_nb2()),
      other => panic!("unknown reference law {other}"),
    };
    let (_, probability) = count_point_probabilities(law, mean, count).unwrap();
    let standard_error =
      count_point_cdf_standard_error(law, mean, count, &design, &covariance).unwrap();
    assert!(
      (probability - expected_probability).abs() <= expected_probability.abs() * 1e-13 + 1e-14,
      "{} alpha={alpha} k={count} CDF {probability} != {expected_probability}",
      row[0]
    );
    let (relative_tolerance, absolute_tolerance) = if expected_se < 1e-10 {
      (1e-7, 0.0)
    } else {
      (1e-8, 2e-13)
    };
    assert!(
      (standard_error - expected_se).abs() <= expected_se * relative_tolerance + absolute_tolerance,
      "{} alpha={alpha} k={count} SE {standard_error} != {expected_se}",
      row[0]
    );
  }
}

#[test]
fn pointwise_cdf_delta_se_preserves_limits_and_rejects_invalid_inputs() {
  assert_eq!(
    count_point_cdf_standard_error(Law::Poisson, 0.0, 0, &[1.0], &[vec![1.0]]).unwrap(),
    0.0
  );
  assert!(count_point_cdf_standard_error(Law::Poisson, 1.0, 0, &[], &[]).is_err());
  assert!(count_point_cdf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[]).is_err());
  assert!(
    count_point_cdf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[vec![f64::INFINITY]]).is_err()
  );
  assert!(count_point_cdf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[vec![-1.0]]).is_err());
  assert!(
    count_point_cdf_standard_error(
      Law::Nb2 { alpha: f64::NAN },
      1.0,
      0,
      &[1.0],
      &[vec![1.0, 0.0], vec![0.0, 1.0]],
    )
    .is_err()
  );
}
