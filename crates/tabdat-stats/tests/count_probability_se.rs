use tabdat_stats::{
  CountProbabilityLaw as Law, count_point_pmf_standard_error, count_point_probabilities,
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
fn pointwise_delta_se_matches_independent_base_r_finite_differences() {
  let design = [1.0, -0.7, 1.2];
  let rows: Vec<_> = include_str!("fixtures/count_probability_se_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .collect();
  assert_eq!(rows.len(), 2);
  for row in rows {
    let mean: f64 = row[1].parse().unwrap();
    let count: usize = row[3].parse().unwrap();
    let expected_probability: f64 = row[4].parse().unwrap();
    let expected_se: f64 = row[5].parse().unwrap();
    let (law, covariance) = match row[0] {
      "poisson" => (Law::Poisson, covariance_poisson()),
      "nb2" => (
        Law::Nb2 {
          alpha: row[2].parse().unwrap(),
        },
        covariance_nb2(),
      ),
      other => panic!("unknown reference law {other}"),
    };
    let (probability, _) = count_point_probabilities(law, mean, count).unwrap();
    let standard_error =
      count_point_pmf_standard_error(law, mean, count, &design, &covariance).unwrap();
    assert!((probability - expected_probability).abs() < 1e-14);
    assert!(
      (standard_error - expected_se).abs() <= expected_se * 1e-9 + 1e-13,
      "{} SE {standard_error} != {expected_se}",
      row[0]
    );
  }
}

#[test]
fn pointwise_delta_se_preserves_zero_mean_limit_and_rejects_invalid_covariance() {
  assert_eq!(
    count_point_pmf_standard_error(Law::Poisson, 0.0, 0, &[1.0], &[vec![1.0]]).unwrap(),
    0.0
  );
  assert_eq!(
    count_point_pmf_standard_error(
      Law::Nb2 { alpha: 0.75 },
      0.0,
      4,
      &[1.0],
      &[vec![1.0, 0.0], vec![0.0, 1.0]],
    )
    .unwrap(),
    0.0
  );
  assert!(count_point_pmf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[]).is_err());
  assert!(count_point_pmf_standard_error(Law::Poisson, 1.0, 0, &[], &[]).is_err());
  assert!(count_point_pmf_standard_error(Law::Poisson, 1.0, 0, &[f64::NAN], &[vec![1.0]]).is_err());
  assert!(
    count_point_pmf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[vec![f64::INFINITY]]).is_err()
  );
  assert!(count_point_pmf_standard_error(Law::Poisson, 1.0, 0, &[1.0], &[vec![-1.0]]).is_err());
  assert!(
    count_point_pmf_standard_error(
      Law::Nb2 { alpha: f64::NAN },
      1.0,
      0,
      &[1.0],
      &[vec![1.0, 0.0], vec![0.0, 1.0]]
    )
    .is_err()
  );
}

#[test]
fn pointwise_delta_se_keeps_representable_derivative_when_pmf_underflows() {
  let mean = 746.0;
  assert_eq!(
    count_point_probabilities(Law::Poisson, mean, 0).unwrap().0,
    0.0
  );
  let standard_error =
    count_point_pmf_standard_error(Law::Poisson, mean, 0, &[1.0], &[vec![1.0]]).unwrap();
  let expected = (-mean + mean.ln()).exp();
  assert!(standard_error > 0.0);
  assert_eq!(standard_error, expected);
}
