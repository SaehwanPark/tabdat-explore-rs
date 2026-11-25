use tabdat_stats::{CountProbabilityLaw as Law, count_point_predictive_interval};

#[test]
fn discrete_equal_tail_intervals_match_base_r_quantiles_and_coverage() {
  let rows: Vec<_> = include_str!("fixtures/count_predictive_interval_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .collect();
  assert_eq!(rows.len(), 87);
  for row in rows {
    let mean: f64 = row[1].parse().unwrap();
    let alpha: f64 = row[2].parse().unwrap_or(f64::NAN);
    let level: f64 = row[3].parse().unwrap();
    let expected_lower: usize = row[4].parse().unwrap();
    let expected_upper: usize = row[5].parse().unwrap();
    let expected_lower_previous_cdf: f64 = row[6].parse().unwrap();
    let expected_lower_cdf: f64 = row[7].parse().unwrap();
    let expected_upper_previous_survival: f64 = row[8].parse().unwrap();
    let expected_upper_survival: f64 = row[9].parse().unwrap();
    let expected_coverage: f64 = row[10].parse().unwrap();
    let law = match row[0] {
      "poisson" => Law::Poisson,
      "nb2" => Law::Nb2 { alpha },
      other => panic!("unknown reference law {other}"),
    };
    let actual = count_point_predictive_interval(law, mean, level).unwrap_or_else(|error| {
      panic!(
        "{} mean={mean} alpha={alpha} level={level}: {error}",
        row[0]
      )
    });
    let tail = (100.0 - level) / 200.0;
    assert_eq!(actual.lower, expected_lower, "{row:?}");
    assert_eq!(actual.upper, expected_upper, "{row:?}");
    assert!(actual.lower <= actual.upper, "{row:?}");
    assert!(actual.work_terms > 0 || mean == 0.0, "{row:?}");
    assert!(
      expected_lower_previous_cdf < tail || expected_lower == 0,
      "{row:?}"
    );
    assert!(expected_lower_cdf >= tail, "{row:?}");
    assert!(expected_upper_survival <= tail, "{row:?}");
    assert!(
      expected_upper_previous_survival > tail || expected_upper == 0,
      "{row:?}"
    );
    assert!(expected_coverage + 2e-14 >= level / 100.0, "{row:?}");
  }
}

#[test]
fn predictive_intervals_preserve_zero_limits_and_reject_invalid_domains() {
  let zero = count_point_predictive_interval(Law::Poisson, 0.0, 95.0).unwrap();
  assert_eq!((zero.lower, zero.upper, zero.work_terms), (0, 0, 0));
  for level in [f64::NAN, 0.0, 100.0, -1.0, f64::EPSILON / 1024.0] {
    assert!(count_point_predictive_interval(Law::Poisson, 2.4, level).is_err());
  }
  assert!(count_point_predictive_interval(Law::Poisson, f64::NAN, 95.0).is_err());
  assert!(count_point_predictive_interval(Law::Poisson, -1.0, 95.0).is_err());
  assert!(count_point_predictive_interval(Law::Nb2 { alpha: 0.0 }, 2.4, 95.0).is_err());
  assert!(count_point_predictive_interval(Law::Poisson, 1e8, 95.0).is_err());

  let near_100 = f64::from_bits(100.0_f64.to_bits() - 1);
  let interval = count_point_predictive_interval(Law::Poisson, 2.4, near_100).unwrap();
  assert_eq!(interval.lower, 0);
  assert!(interval.upper > 0);
}
