use tabdat_stats::{CountProbabilityLaw as Law, count_point_probabilities};

#[test]
fn pmf_and_cdf_match_independent_r_including_tiny_cdfs_and_zero_mean_limits() {
  let mut comparisons = 0;
  for line in include_str!("fixtures/count_point_reference.csv")
    .lines()
    .skip(1)
  {
    let r: Vec<_> = line.split(',').collect();
    let law = if r[0] == "poisson" {
      Law::Poisson
    } else {
      Law::Nb2 {
        alpha: r[1].parse().unwrap(),
      }
    };
    let (pmf, cdf) =
      count_point_probabilities(law, r[2].parse().unwrap(), r[3].parse().unwrap()).unwrap();
    for (actual, expected) in [pmf, cdf].into_iter().zip(&r[4..]) {
      let expected: f64 = expected.parse().unwrap();
      assert!((0.0..=1.0).contains(&actual));
      let tolerance = expected.abs() * 1e-10 + f64::MIN_POSITIVE;
      assert!(
        (actual - expected).abs() <= tolerance,
        "{r:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 288);
}

#[test]
fn extended_cutoffs_match_independent_r_through_the_nb2_count_limit() {
  let mut comparisons = 0;
  for line in include_str!("fixtures/count_point_large_reference.csv")
    .lines()
    .skip(1)
  {
    let row: Vec<_> = line.split(',').collect();
    let law = if row[0] == "poisson" {
      Law::Poisson
    } else {
      Law::Nb2 {
        alpha: row[1].parse().unwrap(),
      }
    };
    let expected_pmf: f64 = row[4].parse().unwrap();
    let expected_cdf: f64 = row[5].parse().unwrap();
    let (pmf, cdf) =
      count_point_probabilities(law, row[2].parse().unwrap(), row[3].parse().unwrap()).unwrap();
    for (actual, expected) in [(pmf, expected_pmf), (cdf, expected_cdf)] {
      assert!(
        actual.is_finite()
          && (actual - expected).abs() <= expected.abs() * 1e-10 + f64::MIN_POSITIVE,
        "{row:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 20);
}

#[test]
fn invalid_domains_are_rejected_even_at_zero_mean() {
  for mean in [-1.0, f64::NAN, f64::INFINITY] {
    assert!(count_point_probabilities(Law::Poisson, mean, 0).is_err());
  }
  for count in [100001, usize::MAX] {
    assert!(count_point_probabilities(Law::Poisson, 0.0, count).is_err());
  }
  assert!(count_point_probabilities(Law::Nb2 { alpha: 0.0 }, 0.0, 0).is_err());
}
