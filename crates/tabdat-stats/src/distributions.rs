use crate::matrix::lgamma;

// Regularized upper incomplete gamma Q(df/2, statistic/2). Use a lower-tail
// series only where subtraction is well-conditioned, otherwise a continued
// fraction for the upper tail. Nonconvergence is unavailable, never a p-value.
pub(crate) fn chi_square_tail(statistic: f64, df: usize) -> f64 {
  if statistic.is_nan() || statistic < 0.0 || df == 0 {
    return f64::NAN;
  }
  if statistic == f64::INFINITY {
    return 0.0;
  }
  let (a, x) = (df as f64 / 2.0, statistic / 2.0);
  if x == 0.0 {
    return 1.0;
  }
  let prefactor = (a * x.ln() - x - lgamma(a)).exp();
  if x < a + 1.0 {
    let (mut term, mut sum) = (1.0 / a, 1.0 / a);
    for index in 1..=10000 {
      term *= x / (a + index as f64);
      sum += term;
      if term.abs() <= sum.abs() * 1e-14 {
        return (1.0 - prefactor * sum).clamp(0.0, 1.0);
      }
    }
  } else {
    let tiny: f64 = 1e-300;
    let mut b = x + 1.0 - a;
    let (mut c, mut d) = (1.0 / tiny, 1.0 / b);
    let mut h = d;
    for index in 1..=10000 {
      let i = index as f64;
      let an = -i * (i - a);
      b += 2.0;
      d = an * d + b;
      if d.abs() < tiny {
        d = tiny.copysign(d);
      }
      c = b + an / c;
      if c.abs() < tiny {
        c = tiny.copysign(c);
      }
      d = 1.0 / d;
      let delta = d * c;
      h *= delta;
      if (delta - 1.0).abs() <= 1e-14 {
        return (prefactor * h).clamp(0.0, 1.0);
      }
    }
  }
  f64::NAN
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn chi_square_tails_match_independent_r_pchisq() {
    // R 4.6.1: examples/cigarettes_iv/reference_overid.R.
    for (statistic, df, expected) in [
      (0.0, 1, 1.0),
      (0.1, 1, 0.7518296340458492),
      (3.841458820694124, 1, 0.05),
      (100.0, 1, 1.523970604832105e-23),
      // R normal tail for the fishing logit AME (representable, not zero).
      (34.711925154773_f64.powi(2), 1, 5.206395183737961e-264),
      (10.0, 2, 0.006737946999085467),
      (50.0, 3, 7.98917924495147e-11),
      (100.0, 10, 5.44970198292053e-17),
      (1000.0, 100, 2.306076738035398e-148),
      (1000.0, 1000, 0.4940528538292397),
    ] {
      let actual = chi_square_tail(statistic, df);
      assert!(
        (actual - expected).abs() <= expected * 1e-11 + f64::MIN_POSITIVE,
        "{statistic}, {df}: {actual} != {expected}"
      );
    }
    assert!(chi_square_tail(1.0, 0).is_nan());
    assert!(chi_square_tail(-1.0, 1).is_nan());
    assert_eq!(chi_square_tail(f64::INFINITY, 1), 0.0);
  }
}
