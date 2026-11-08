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
