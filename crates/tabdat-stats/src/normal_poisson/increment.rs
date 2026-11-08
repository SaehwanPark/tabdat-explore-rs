//! Stable posterior-ratio cumulants; isolated numerical controls before fitter use.
use super::{invalid, sum};
use crate::StatsError;

pub(super) fn expm1_minus_x(x: f64) -> f64 {
  if x.abs() >= 0.5 {
    return x.exp_m1() - x;
  }
  let mut term = x * x / 2.;
  let mut result = term;
  for n in 3..=24 {
    term *= x / n as f64;
    result += term;
    if term.abs() <= f64::EPSILON * result.abs() {
      break;
    }
  }
  result
}

pub(super) fn log1p_minus_x(x: f64) -> f64 {
  if x.abs() >= 0.125 {
    return x.ln_1p() - x;
  }
  let mut term = -x * x / 2.;
  let mut result = term;
  for n in 3..=40 {
    term *= -x * (n - 1) as f64 / n as f64;
    result += term;
    if term.abs() <= f64::EPSILON * result.abs() {
      break;
    }
  }
  result
}

// The caller supplies E[h] using stable score/remainder identities. Recomputing
// its linear part from quadrature would discard the tiny Newton gain we need.
pub(super) fn cumulant_correction(mean_h: f64, terms: &[(f64, f64)]) -> Result<f64, StatsError> {
  if !mean_h.is_finite()
    || terms.is_empty()
    || terms
      .iter()
      .any(|&(p, h)| !p.is_finite() || p < 0. || !h.is_finite())
    || (sum(terms.iter().map(|v| v.0)) - 1.).abs() > 8. * f64::EPSILON
  {
    return Err(invalid("invalid normalized posterior increment terms"));
  }
  let remainder = sum(
    terms
      .iter()
      .filter(|v| v.0 > 0.)
      .map(|&(p, h)| p * expm1_minus_x(h)),
  );
  let argument = mean_h + remainder;
  let correction = remainder + log1p_minus_x(argument);
  if argument <= -1. || !correction.is_finite() {
    return Err(invalid("nonfinite posterior likelihood increment"));
  }
  Ok(correction)
}

// Split off the linear term so a fitter can sum it from its existing global
// Stein score, rather than cancelling first-order group contributions twice.
pub(super) fn poisson_group_increment(
  y: &[f64],
  mu: &[f64],
  delta_eta: &[f64],
  log_variance: f64,
  delta_t: f64,
) -> Result<(f64, f64), StatsError> {
  if y.is_empty()
    || y.len() != mu.len()
    || y.len() != delta_eta.len()
    || y
      .iter()
      .any(|v| !v.is_finite() || *v < 0. || v.fract() != 0.)
    || mu.iter().any(|v| !v.is_finite() || *v <= 0.)
    || delta_eta.iter().any(|v| !v.is_finite())
    || !delta_t.is_finite()
    || (log_variance + delta_t).abs() >= 8.
  {
    return Err(invalid("invalid Poisson posterior increment frame"));
  }
  let total = sum(y.iter().copied());
  let m = sum(mu.iter().copied());
  let post = super::posterior(total, m, log_variance)?;
  let variance = log_variance.exp();
  let delta_center = sum(mu.iter().zip(delta_eta).map(|(a, b)| a / m * b));
  let conditional = sum(y.iter().zip(delta_eta).map(|(a, b)| a * (b - delta_center)));
  let linear = conditional
    + delta_center * post.mean_u / variance
    + delta_t * (post.mean_u2 / (2. * variance) - 0.5);
  let nonlinear_mean = -post.mean_exp_u
    * sum(mu.iter().zip(delta_eta).map(|(a, b)| a * expm1_minus_x(*b)))
    - post.mean_u2 / (2. * variance) * expm1_minus_x(-delta_t);
  let y_delta = sum(y.iter().zip(delta_eta).map(|(a, b)| a * b));
  let mu_expm1 = sum(mu.iter().zip(delta_eta).map(|(a, b)| a * b.exp_m1()));
  let (center, curvature) = super::mode(total, m, variance)?;
  let scale = (2. / curvature).sqrt();
  let mut points = Vec::with_capacity(super::nodes::GH96.len());
  for &(z, weight) in &super::nodes::GH96 {
    let u = center + scale * z;
    if !u.is_finite() || u.abs() > 100. {
      return Err(invalid("increment quadrature random intercept exceeds100"));
    }
    let log_weight = weight.ln() + z * z + total * u - m * u.exp() - u * u / (2. * variance);
    let h =
      y_delta - u.exp() * mu_expm1 - delta_t / 2. - u * u / (2. * variance) * (-delta_t).exp_m1();
    points.push((log_weight, h));
  }
  let anchor = points.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
  let denominator = sum(points.iter().map(|v| (v.0 - anchor).exp()));
  let terms: Vec<_> = points
    .iter()
    .map(|v| ((v.0 - anchor).exp() / denominator, v.1))
    .collect();
  let correction = nonlinear_mean + cumulant_correction(linear + nonlinear_mean, &terms)?;
  if !linear.is_finite() || !correction.is_finite() {
    return Err(invalid("nonfinite Poisson posterior increment"));
  }
  Ok((linear, correction))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn remainders_match_independent_400_digit_decimal_at_actual_binary_inputs() {
    let data = include_str!("../../tests/fixtures/normal_poisson/increment/remainders.csv");
    let mut count = 0;
    for line in data.lines().skip(1) {
      let c: Vec<_> = line.split(',').collect();
      let x: f64 = c[1].parse().unwrap();
      let expected: f64 = c[2].parse().unwrap();
      let actual = match c[0] {
        "exp" => expm1_minus_x(x),
        "log" => log1p_minus_x(x),
        _ => panic!("unknown remainder"),
      };
      assert!(
        (actual - expected).abs() <= 4. * f64::EPSILON * expected.abs().max(f64::MIN_POSITIVE),
        "{} at{x}: {actual:.17e} versus {expected:.17e}",
        c[0]
      );
      count += 1;
    }
    assert_eq!(count, 44);
  }

  #[test]
  fn quadratic_gaussian_ratios_resolve_tiny_positive_negative_and_cancelled_changes() {
    let data = include_str!("../../tests/fixtures/normal_poisson/increment/gaussian.csv");
    let denominator = sum(super::super::nodes::GH96.iter().map(|v| v.1));
    let mut count = 0;
    for line in data.lines().skip(1) {
      let c: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
      let (a, b, constant, expected) = (c[0], c[1], c[2], c[3]);
      let terms: Vec<_> = super::super::nodes::GH96
        .iter()
        .map(|&(z, w)| {
          let u = 2_f64.sqrt() * z;
          (w / denominator, a * u + b * u * u + constant)
        })
        .collect();
      let mean_h = b + constant;
      let actual = mean_h + cumulant_correction(mean_h, &terms).unwrap();
      let tolerance = 1e-12 * expected.abs() + 1e-36;
      assert!(
        (actual - expected).abs() <= tolerance,
        "a={a},b={b},c={constant}: {actual:.17e} versus {expected:.17e}, tolerance{tolerance:.3e}"
      );
      if expected.abs() > tolerance {
        assert_eq!(actual.is_sign_positive(), expected.is_sign_positive());
      }
      count += 1;
    }
    assert_eq!(count, 12);
    assert_eq!(1_f64 + 5e-25, 1.); // A level comparison cannot resolve this gain.
  }

  #[test]
  fn poisson_group_ratios_match_18_independent_direct_integrals_and_complete_scores() {
    let input = include_str!("../../tests/fixtures/normal_poisson/increment/posterior_input.csv");
    let reference = include_str!("../../tests/fixtures/normal_poisson/increment/posterior.csv");
    let records: Vec<Vec<f64>> = input
      .lines()
      .skip(1)
      .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
      .collect();
    let mut count = 0;
    for line in reference.lines().skip(1) {
      let c: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
      let rows: Vec<_> = records.iter().filter(|r| r[0] == c[0]).collect();
      assert!(!rows.is_empty());
      for (i, row) in rows.iter().enumerate() {
        assert_eq!(row[1], (i + 1) as f64);
        assert_eq!(&row[5..], &rows[0][5..]);
      }
      let y: Vec<_> = rows.iter().map(|r| r[2]).collect();
      let mu: Vec<_> = rows.iter().map(|r| r[3]).collect();
      let delta: Vec<_> = rows.iter().map(|r| r[4]).collect();
      let (linear, correction) =
        poisson_group_increment(&y, &mu, &delta, rows[0][5], rows[0][6]).unwrap();
      for (name, actual, expected) in [
        ("linear", linear, c[1]),
        ("log ratio", linear + correction, c[2]),
      ] {
        let tolerance = 1e-9 * expected.abs() + 1e-24;
        assert!(
          (actual - expected).abs() <= tolerance,
          "case{} {name}: {actual:.17e} versus {expected:.17e}, tolerance{tolerance:.3e}",
          c[0]
        );
        assert_eq!(actual.is_sign_positive(), expected.is_sign_positive());
      }
      count += 1;
    }
    assert_eq!(count, 18);
    assert_eq!(records.len(), 54);
    assert_eq!(
      poisson_group_increment(&[0.], &[1.], &[0.], 0., 0.).unwrap(),
      (0., 0.)
    );
    for (y, mu, delta, dt) in [
      (vec![], vec![], vec![], 0.),
      (vec![1.], vec![], vec![0.], 0.),
      (vec![0.5], vec![1.], vec![0.], 0.),
      (vec![100001.], vec![1.], vec![0.], 0.),
      (vec![1.], vec![0.], vec![0.], 0.),
      (vec![1.], vec![1.], vec![f64::NAN], 0.),
      (vec![1.], vec![1.], vec![0.], 9.),
      (vec![1.], vec![1.], vec![0.], f64::INFINITY),
    ] {
      assert!(poisson_group_increment(&y, &mu, &delta, 0., dt).is_err());
    }
  }

  #[test]
  fn invalid_increment_inputs_fail_without_clipping_or_renormalization() {
    for (mean, terms) in [
      (0., vec![]),
      (0., vec![(0.5, 0.)]),
      (0., vec![(-1., 0.), (2., 0.)]),
      (f64::NAN, vec![(1., 0.)]),
      (0., vec![(1., f64::NAN)]),
      (0., vec![(f64::INFINITY, 0.)]),
      (-2., vec![(1., 0.)]),
      (0., vec![(1., 1000.)]),
    ] {
      assert!(cumulant_correction(mean, &terms).is_err());
    }
    assert_eq!(
      cumulant_correction(0., &[(0., 1000.), (1., 0.)]).unwrap(),
      0.
    );
  }
}
