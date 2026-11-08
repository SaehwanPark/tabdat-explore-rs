//! Adaptive shared-normal-intercept integration; native fit integration is WIP.
mod nodes;
use crate::{StatsError, matrix::add_compensated};
use nodes::{GH80, GH96};

#[derive(Debug, Clone)]
struct Posterior {
  log_integral: f64,
  mean_exp_u: f64,
  mean_u2: f64,
  var_exp_u: f64,
  cov_exp_u_u2: f64,
  var_u2: f64,
  mode: f64,
  curvature: f64,
  max_relative_difference: f64,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("normal Poisson RE: {message}"))
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut total, mut correction) = (0., 0.);
  for value in values {
    add_compensated(&mut total, &mut correction, value);
  }
  total
}
fn mode(total: f64, mean_sum: f64, variance: f64) -> Result<(f64, f64), StatsError> {
  let score = |u: f64| total - mean_sum * u.exp() - u / variance;
  let (mut lo, mut hi) = (-60., 60.);
  if score(lo) <= 0. || score(hi) >= 0. {
    return Err(invalid("posterior mode is not bracketed"));
  }
  for _ in 0..55 {
    let mid = (lo + hi) / 2.;
    if score(mid) > 0. {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  let center = (lo + hi) / 2.;
  let lambda = mean_sum * center.exp();
  if score(center).abs() > 1e-10 * (1. + total + lambda + (center / variance).abs()) {
    return Err(invalid("posterior mode root check failed"));
  }
  Ok((center, lambda + 1. / variance))
}
fn integrate_rule(
  total: f64,
  mean_sum: f64,
  variance: f64,
  center: f64,
  curvature: f64,
  rule: &[(f64, f64)],
) -> Result<[f64; 6], StatsError> {
  let scale = (2. / curvature).sqrt();
  let mut values = Vec::with_capacity(rule.len());
  for &(z, weight) in rule {
    let u = center + scale * z;
    if !u.is_finite() || u.abs() > 100. {
      return Err(invalid("quadrature random intercept exceeds100"));
    }
    let exp_u = u.exp();
    let log_weight = weight.ln() + z * z + total * u - mean_sum * exp_u - u * u / (2. * variance);
    if !log_weight.is_finite() {
      return Err(invalid("nonfinite quadrature integrand"));
    }
    values.push((log_weight, exp_u, u * u));
  }
  let anchor = values.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
  let weights: Vec<_> = values.iter().map(|v| (v.0 - anchor).exp()).collect();
  let denominator = sum(weights.iter().copied());
  let mean_exp = sum(values.iter().zip(&weights).map(|(v, w)| w * v.1)) / denominator;
  let mean_u2 = sum(values.iter().zip(&weights).map(|(v, w)| w * v.2)) / denominator;
  // Centered products retain small posterior covariances at high count totals.
  let var_exp = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp).powi(2)),
  ) / denominator;
  let cov = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp) * (v.2 - mean_u2)),
  ) / denominator;
  let var_u2 = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.2 - mean_u2).powi(2)),
  ) / denominator;
  let log_integral =
    scale.ln() + anchor + denominator.ln() - (2. * std::f64::consts::PI * variance).ln() / 2.;
  let result = [log_integral, mean_exp, mean_u2, var_exp, cov, var_u2];
  if result.iter().any(|v| !v.is_finite()) {
    return Err(invalid("nonfinite posterior moments"));
  }
  Ok(result)
}
fn posterior(total: f64, mean_sum: f64, log_variance: f64) -> Result<Posterior, StatsError> {
  if !total.is_finite()
    || total < 0.
    || total > 100000.
    || total.fract() != 0.
    || !mean_sum.is_finite()
    || mean_sum <= 0.
    || !log_variance.is_finite()
    || log_variance.abs() >= 8.
  {
    return Err(invalid("invalid integrated group parameters"));
  }
  let variance = log_variance.exp();
  let (mode, curvature) = mode(total, mean_sum, variance)?;
  let high = integrate_rule(total, mean_sum, variance, mode, curvature, &GH96)?;
  let low = integrate_rule(total, mean_sum, variance, mode, curvature, &GH80)?;
  // Check intensity and prior-score units, not tiny unscaled exp(u) moments:
  // regression/information multiply these by M and inverse variance.
  let units = [
    1.,
    mean_sum,
    1. / variance,
    mean_sum.powi(2),
    mean_sum / variance,
    1. / variance.powi(2),
  ];
  let discrepancy = high
    .iter()
    .zip(low)
    .zip(units)
    .map(|((a, b), scale)| ((a - b) * scale).abs() / (1. + (a * scale).abs()))
    .fold(0., f64::max);
  if discrepancy > 1e-9 {
    return Err(invalid("80/96-node quadrature disagreement exceeds1e-9"));
  }
  Ok(Posterior {
    log_integral: high[0],
    mean_exp_u: high[1],
    mean_u2: high[2],
    var_exp_u: high[3],
    cov_exp_u_u2: high[4],
    var_u2: high[5],
    mode,
    curvature,
    max_relative_difference: discrepancy,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn fixed_rules_have_positive_tail_weights_and_independent_gaussian_moments() {
    for rule in [&GH80[..], &GH96[..]] {
      assert!(
        rule
          .iter()
          .all(|(z, w)| z.is_finite() && w.is_finite() && *w > 0.)
      );
      for pair in rule.windows(2) {
        assert!(pair[0].0 < pair[1].0);
      }
      let mut exact = std::f64::consts::PI.sqrt();
      for power in 0..=12 {
        if power > 0 {
          exact *= (2 * power - 1) as f64 / 2.;
        }
        let actual = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power)));
        assert!(
          (actual / exact - 1.).abs() < 2e-13,
          "power{power}: {actual} vs {exact}"
        );
        let odd = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power + 1)));
        assert!(odd.abs() < 2e-12 * (1. + exact), "odd power{power}: {odd}");
      }
    }
  }
  #[test]
  fn posterior_moments_match_direct_r_improper_integration() {
    let fixture = include_str!("../tests/fixtures/normal_poisson/posterior.csv");
    let cases: Vec<_> = fixture.lines().skip(1).collect();
    assert_eq!(cases.len(), 6);
    for line in cases {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      assert_eq!(v.len(), 9);
      let p = posterior(v[0], v[1], v[2]).unwrap();
      let actual = [
        p.log_integral,
        p.mean_exp_u,
        p.mean_u2,
        p.var_exp_u,
        p.cov_exp_u_u2,
        p.var_u2,
      ];
      for (&a, &b) in actual.iter().zip(&v[3..]) {
        assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs(), "{v:?}: {a} vs {b}");
      }
      assert!(p.max_relative_difference <= 1e-9);
      let variance = v[2].exp();
      let score = v[0] - v[1] * p.mode.exp() - p.mode / variance;
      assert!(score.abs() <= 1e-10 * (1. + v[0] + v[1] * p.mode.exp() + (p.mode / variance).abs()));
      assert_eq!(p.curvature, v[1] * p.mode.exp() + 1. / variance);
    }
  }
  #[test]
  fn high_variance_control_start_rejects_actual_intensity_moment_disagreement() {
    let source = include_str!("../../../tests/fixtures/mma23_gamma_re_bootstrap/source.asc");
    let mut rejected = 0;
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
      let tokens: Vec<_> = line.split_whitespace().collect();
      let total = tokens[20..25]
        .iter()
        .map(|v| v.parse::<f64>().unwrap())
        .sum();
      // Independent R Poisson GLM start on the existing 200-record control.
      let m = tokens[10..15]
        .iter()
        .map(|v| (1.2479608264839344 + 0.5673452307402028 * v.parse::<f64>().unwrap()).exp())
        .sum();
      if let Err(error) = posterior(total, m, 2.) {
        assert!(
          error.to_string().contains("quadrature disagreement"),
          "{error}"
        );
        rejected += 1;
      }
    }
    assert!(
      rejected > 0,
      "high-variance start must expose actual 80/96 disagreement"
    );
  }
  #[test]
  fn invalid_parameters_and_unbracketed_modes_are_errors() {
    for (s, m, t) in [
      (0., 0., 0.),
      (-1., 1., 0.),
      (1.5, 1., 0.),
      (100001., 1., 0.),
      (1., 1., 8.),
      (1., 1., -8.),
      (f64::NAN, 1., 0.),
      (1., f64::INFINITY, 0.),
      (100000., 1e-100, 0.),
    ] {
      assert!(posterior(s, m, t).is_err(), "{s},{m},{t}");
    }
  }
}
