use tabdat_stats::{CountProbabilityLaw as Law, count_bin_probabilities, count_probability_table};

fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && (a - b).abs() <= b.abs() * 1e-10 + f64::MIN_POSITIVE,
    "{a} != {b}"
  );
}

#[test]
fn geometric_and_poisson_survival_laws_preserve_small_tails_and_underflowed_zero_bins() {
  for cutoff in [1, 10, 100] {
    let p = count_bin_probabilities(Law::Nb2 { alpha: 1. }, 1., cutoff).unwrap();
    for (k, value) in p[..cutoff].iter().enumerate() {
      close(*value, 2_f64.powi(-(k as i32 + 1)));
    }
    close(p[cutoff], 2_f64.powi(-(cutoff as i32)));
    for mean in [1e-300, 1e-8, 0.1, 3., 1000., 1e6] {
      let p = count_bin_probabilities(Law::Nb2 { alpha: 1. }, mean, cutoff).unwrap();
      close(
        p[cutoff],
        (cutoff as f64 * (mean.ln() - mean.ln_1p())).exp(),
      );
    }
  }
  let p = count_bin_probabilities(Law::Poisson, 1., 1).unwrap();
  close(p[0], (-1_f64).exp());
  close(p[1], -(-1_f64).exp_m1());
  let p = count_bin_probabilities(Law::Poisson, 1e-8, 1).unwrap();
  close(p[1], -(-1e-8_f64).exp_m1());
  let p = count_bin_probabilities(Law::Poisson, 1000., 100).unwrap();
  assert_eq!(p[0], 0.);
  assert!(p[99] > 0. && p[99] < 1e-290);
  assert_eq!(p[100], 1.);
}

#[test]
fn aggregate_uses_individual_probabilities_and_exact_tail_frequencies() {
  let y = [0., 1., 2., 77.];
  let mu = [0.1, 2., 3., 4.];
  for law in [Law::Poisson, Law::Nb2 { alpha: 0.69 }] {
    let table = count_probability_table(law, &y, &mu, 2).unwrap();
    assert_eq!(table.observation_count, 4);
    assert_eq!(table.tail_from, 2);
    assert_eq!(
      table
        .bins
        .iter()
        .map(|b| b.observed_count)
        .collect::<Vec<_>>(),
      [1, 1, 2]
    );
    for (i, bin) in table.bins.iter().enumerate() {
      let mean = mu
        .iter()
        .map(|m| count_bin_probabilities(law, *m, 2).unwrap()[i])
        .sum::<f64>()
        / 4.;
      close(bin.fitted_fraction, mean);
      close(bin.expected_count, 4. * mean);
      assert_eq!(bin.lower_bound, i);
      assert_eq!(bin.upper_bound, (i < 2).then_some(i));
    }
    assert!(
      (table.bins[0].fitted_fraction - count_bin_probabilities(law, 2.275, 2).unwrap()[0]).abs()
        > 0.01
    );
    close(table.bins.iter().map(|b| b.fitted_fraction).sum(), 1.);
    close(table.bins.iter().map(|b| b.observed_fraction).sum(), 1.);
  }
}

#[test]
fn invalid_laws_means_cutoffs_samples_and_count_domains_fail_without_mutation() {
  for mean in [f64::NAN, f64::INFINITY, 0., -1.] {
    assert!(count_bin_probabilities(Law::Poisson, mean, 10).is_err());
  }
  for alpha in [
    f64::NAN,
    f64::INFINITY,
    0.,
    -1.,
    (-12_f64).exp(),
    12_f64.exp(),
  ] {
    assert!(count_bin_probabilities(Law::Nb2 { alpha }, 1., 10).is_err());
  }
  for cutoff in [0, 101, usize::MAX] {
    assert!(count_bin_probabilities(Law::Poisson, 1., cutoff).is_err());
  }
  for y in [f64::NAN, f64::INFINITY, -1., 0.5, 9007199254740992.] {
    assert!(count_probability_table(Law::Poisson, &[y], &[1.], 10).is_err());
  }
  assert!(count_probability_table(Law::Nb2 { alpha: 1. }, &[100001.], &[1.], 10).is_err());
  assert!(count_probability_table(Law::Poisson, &[], &[], 10).is_err());
  assert!(count_probability_table(Law::Poisson, &[1.], &[1., 2.], 10).is_err());
}
