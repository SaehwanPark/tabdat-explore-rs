use tabdat_stats::{
  CountProbabilityLaw as Law, count_point_cdf_interval, count_point_pmf_interval,
  count_point_probabilities,
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
fn pointwise_logit_wald_intervals_match_base_r_finite_differences() {
  let design = [1.0, -0.7, 1.2];
  let rows: Vec<_> = include_str!("fixtures/count_probability_interval_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .collect();
  assert_eq!(rows.len(), 44);
  for row in rows {
    let mean: f64 = row[1].parse().unwrap();
    let alpha: f64 = row[2].parse().unwrap_or(f64::NAN);
    let count: usize = row[3].parse().unwrap();
    let quantity = row[4];
    let expected_estimate: f64 = row[5].parse().unwrap();
    let expected_lower: f64 = row[6].parse().unwrap();
    let expected_upper: f64 = row[7].parse().unwrap();
    let (law, covariance) = match row[0] {
      "poisson" => (Law::Poisson, covariance_poisson()),
      "nb2" => (Law::Nb2 { alpha }, covariance_nb2()),
      other => panic!("unknown reference law {other}"),
    };
    let actual = if quantity == "pmf" {
      count_point_pmf_interval(law, mean, count, &design, &covariance, 95.0).unwrap()
    } else {
      count_point_cdf_interval(law, mean, count, &design, &covariance, 95.0).unwrap()
    };
    let estimate_tolerance = expected_estimate.abs() * 1e-10 + 1e-14;
    assert!(
      (actual.estimate - expected_estimate).abs() <= estimate_tolerance,
      "{} {quantity} mean={mean} alpha={alpha} k={count} estimate {} != {expected_estimate}",
      row[0],
      actual.estimate
    );
    for (bound, expected) in [("lower", expected_lower), ("upper", expected_upper)] {
      let actual = if bound == "lower" {
        actual.lower
      } else {
        actual.upper
      };
      let tolerance = expected.abs() * 1e-7 + 2e-13;
      assert!(
        (actual - expected).abs() <= tolerance,
        "{} {quantity} mean={mean} alpha={alpha} k={count} {bound} {actual} != {expected}",
        row[0]
      );
    }
    assert!(actual.lower.is_finite() && actual.lower >= 0.0);
    assert!(actual.upper.is_finite() && actual.upper <= 1.0);
    assert!(actual.lower <= actual.estimate + 2e-14);
    assert!(actual.estimate <= actual.upper + 2e-14);
  }
}

#[test]
fn probability_intervals_preserve_degenerate_limits_and_reject_invalid_inputs() {
  for (law, covariance) in [
    (Law::Poisson, vec![vec![1.0]]),
    (
      Law::Nb2 { alpha: 0.75 },
      vec![vec![1.0, 0.0], vec![0.0, 1.0]],
    ),
  ] {
    let design = [1.0];
    for (count, expected_pmf) in [(0, 1.0), (2, 0.0)] {
      let interval = count_point_pmf_interval(law, 0.0, count, &design, &covariance, 95.0).unwrap();
      assert_eq!(interval.estimate, expected_pmf);
      assert_eq!(interval.lower, expected_pmf);
      assert_eq!(interval.upper, expected_pmf);
    }
    let cdf = count_point_cdf_interval(law, 0.0, 2, &design, &covariance, 95.0).unwrap();
    assert_eq!((cdf.estimate, cdf.lower, cdf.upper), (1.0, 1.0, 1.0));
  }
  let point = count_point_probabilities(Law::Poisson, 2.4, 2).unwrap().0;
  let zero_covariance =
    count_point_pmf_interval(Law::Poisson, 2.4, 2, &[1.0], &[vec![0.0]], 95.0).unwrap();
  assert_eq!(
    (
      zero_covariance.estimate,
      zero_covariance.lower,
      zero_covariance.upper
    ),
    (point, point, point)
  );

  for level in [f64::NAN, 0.0, 100.0, -1.0, f64::EPSILON / 1024.0] {
    assert!(count_point_pmf_interval(Law::Poisson, 1.0, 0, &[1.0], &[vec![1.0]], level).is_err());
    assert!(count_point_cdf_interval(Law::Poisson, 1.0, 0, &[1.0], &[vec![1.0]], level).is_err());
  }
  assert!(count_point_pmf_interval(Law::Poisson, 1.0, 0, &[], &[], 95.0).is_err());
  assert!(count_point_cdf_interval(Law::Poisson, 1.0, 0, &[1.0], &[], 95.0).is_err());
  assert!(count_point_pmf_interval(Law::Poisson, 1.0, 0, &[1.0], &[vec![-1.0]], 95.0).is_err());
}
