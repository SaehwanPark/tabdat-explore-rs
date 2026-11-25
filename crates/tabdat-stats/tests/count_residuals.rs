use tabdat_stats::{CountProbabilityLaw, CountResidualKind, count_diagnostic_residual};

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 2e-12 + 2e-10 * expected.abs(),
    "actual={actual:.17e}, expected={expected:.17e}"
  );
}

#[test]
fn poisson_pearson_and_deviance_residuals_use_the_poisson_variance_and_unit_deviance() {
  let law = CountProbabilityLaw::Poisson;
  let observed = 5_i128;
  let mean = 2.0;
  close(
    count_diagnostic_residual(law, observed, mean, CountResidualKind::Pearson)
      .unwrap()
      .unwrap(),
    (observed as f64 - mean) / mean.sqrt(),
  );
  close(
    count_diagnostic_residual(law, observed, mean, CountResidualKind::Deviance)
      .unwrap()
      .unwrap(),
    (2.0 * (observed as f64 * (observed as f64 / mean).ln() - (observed as f64 - mean))).sqrt(),
  );
  close(
    count_diagnostic_residual(law, 0, mean, CountResidualKind::Deviance)
      .unwrap()
      .unwrap(),
    -(2.0 * mean).sqrt(),
  );
}

#[test]
fn nb2_pearson_and_deviance_residuals_use_the_retained_dispersion() {
  let alpha = 0.4;
  let law = CountProbabilityLaw::Nb2 { alpha };
  let observed = 5_i128;
  let mean = 2.0;
  let variance = mean + alpha * mean * mean;
  let inverse_alpha = 1.0 / alpha;
  close(
    count_diagnostic_residual(law, observed, mean, CountResidualKind::Pearson)
      .unwrap()
      .unwrap(),
    (observed as f64 - mean) / variance.sqrt(),
  );
  close(
    count_diagnostic_residual(law, observed, mean, CountResidualKind::Deviance)
      .unwrap()
      .unwrap(),
    (2.0
      * (observed as f64 * (observed as f64 / mean).ln()
        - (observed as f64 + inverse_alpha)
          * ((1.0 + alpha * observed as f64) / (1.0 + alpha * mean)).ln()))
    .sqrt(),
  );
  close(
    count_diagnostic_residual(law, 0, mean, CountResidualKind::Deviance)
      .unwrap()
      .unwrap(),
    -(2.0 * inverse_alpha * (1.0 + alpha * mean).ln()).sqrt(),
  );
}

#[test]
fn residual_boundaries_and_invalid_count_inputs_are_explicit() {
  for law in [
    CountProbabilityLaw::Poisson,
    CountProbabilityLaw::Nb2 { alpha: 0.4 },
  ] {
    assert_eq!(
      count_diagnostic_residual(law, 0, 0.0, CountResidualKind::Pearson).unwrap(),
      None
    );
    assert_eq!(
      count_diagnostic_residual(law, 0, 0.0, CountResidualKind::Deviance).unwrap(),
      Some(0.0)
    );
    assert_eq!(
      count_diagnostic_residual(law, 1, 0.0, CountResidualKind::Deviance).unwrap(),
      None
    );
    assert!(count_diagnostic_residual(law, -1, 1.0, CountResidualKind::Pearson).is_err());
    assert!(count_diagnostic_residual(law, 0, f64::NAN, CountResidualKind::Pearson).is_err());
  }
  assert!(
    count_diagnostic_residual(
      CountProbabilityLaw::Nb2 { alpha: 0.0 },
      0,
      1.0,
      CountResidualKind::Pearson
    )
    .is_err()
  );
}

#[test]
fn deviance_residuals_are_stable_when_the_observation_is_near_the_mean() {
  let observed = 10_000_i128;
  let mean = 10_000.000001;
  for law in [
    CountProbabilityLaw::Poisson,
    CountProbabilityLaw::Nb2 { alpha: 0.4 },
  ] {
    let pearson = count_diagnostic_residual(law, observed, mean, CountResidualKind::Pearson)
      .unwrap()
      .unwrap();
    let deviance = count_diagnostic_residual(law, observed, mean, CountResidualKind::Deviance)
      .unwrap()
      .unwrap();
    close(deviance, pearson);
  }
}

#[test]
fn deviance_residuals_remain_finite_when_the_unit_deviance_exceeds_f64() {
  let mean = f64::MAX;
  let expected = -(2.0_f64.sqrt() * mean.sqrt());
  assert!(expected.is_finite());
  close(
    count_diagnostic_residual(
      CountProbabilityLaw::Poisson,
      0,
      mean,
      CountResidualKind::Deviance,
    )
    .unwrap()
    .unwrap(),
    expected,
  );
  close(
    count_diagnostic_residual(
      CountProbabilityLaw::Poisson,
      1,
      mean,
      CountResidualKind::Deviance,
    )
    .unwrap()
    .unwrap(),
    expected,
  );
}
