use crate::matrix::lgamma;

/// Standard normal CDF, Phi(x).
///
/// Uses the exact chi-square identity Q(1/2, x^2/2) = `2*Phi(-|x|)`, evaluated
/// by the independently validated chi-square tail above, split by sign so no
/// branch subtracts near one: the small-tail ratio stays multiplicative.
/// Phi(NaN) is NaN; saturated |x| yield exactly 1 or 0 when the tail
/// underflows, mirroring the represented normal distribution.
pub fn normal_cdf(x: f64) -> f64 {
  if !x.is_finite() && !x.is_nan() {
    // +-infinity: the tail statistic squares to infinity and underflows.
    return if x > 0.0 { 1.0 } else { 0.0 };
  }
  if x.is_nan() {
    return f64::NAN;
  }
  let reflected_tail = 0.5 * chi_square_tail(x * x, 1);
  if !reflected_tail.is_finite() {
    return f64::NAN;
  }
  if x >= 0.0 {
    1.0 - reflected_tail
  } else {
    reflected_tail
  }
}

/// Standard normal density, phi(x). Underflow to exactly zero at |x| past the
/// representable exponent range, like any double-precision evaluation.
// The constant keeps its written 1/sqrt(2*pi) provenance digits as data.
#[allow(clippy::excessive_precision)]
pub fn normal_pdf(x: f64) -> f64 {
  if x.is_nan() {
    return f64::NAN;
  }
  if !x.is_finite() {
    return 0.0;
  }
  0.398942280401432677939946059934 * (-0.5 * x * x).exp()
}

/// Inverse standard normal CDF for strictly interior probabilities, by
/// guarded bisection. Only used for deterministic starting values in binary
/// fits, so iteration count is awkward but exactness is provable: the
/// bracketing property of the monotone CDF is the only assumption.
pub fn normal_quantile(probability: f64) -> f64 {
  if !(0.0 < probability && probability < 1.0) {
    return f64::NAN;
  }
  let mut lower = -80.0_f64;
  let mut upper = 80.0_f64;
  // Bisection until the bracket cannot shrink further at double resolution.
  for _ in 0..400 {
    let middle = 0.5 * (lower + upper);
    if middle <= lower || middle >= upper {
      break;
    }
    if normal_cdf(middle) < probability {
      lower = middle;
    } else {
      upper = middle;
    }
  }
  // Report the endpoint whose CDF rounds onto the target; monotonicity makes
  // either representative within one representable-mapping ulp of the truth.
  let mid = 0.5 * (lower + upper);
  if !mid.is_finite() {
    return f64::NAN;
  }
  mid
}

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

  // Reference values preserve R's %.17g spellings verbatim, so provenance
  // beats shortest representation here; 1/sqrt(2) is deliberate.
  #[allow(clippy::approx_constant, clippy::excessive_precision)]
  #[test]
  fn normal_cdf_pdf_and_quantile_match_independent_r() {
    // R 4.6.1 pnorm/dnorm/qnorm references, plus a saturated extreme.
    for (x, cdf, pdf) in [
      (
        -4.062713386470777,
        2.4252786221911947e-05,
        0.00010393357513447421,
      ),
      (
        -2.153975795300000,
        0.015621035075294422,
        0.039213100304504256,
      ),
      (-0.5, 0.30853753872598694, 0.35206532676429952),
      (0.0, 0.5, 0.3989422804014327),
      (0.7071067811865476, 0.76024993890652326, 0.31069656037692778),
      (1.1943604331481332, 0.88383149638145997, 0.19550155310062106),
      (3.46730123, 0.99973714380403811, 0.00097797402950106256),
      (5.0, 0.99999971334842808, 1.4867195147342977e-06),
    ] {
      let at_cdf = normal_cdf(x);
      assert!(
        (at_cdf - cdf).abs() <= cdf.abs() * 1e-11 + f64::MIN_POSITIVE,
        "cdf {x}: {at_cdf} != {cdf}"
      );
      let at_pdf = normal_pdf(x);
      assert!(
        (at_pdf - pdf).abs() <= pdf.abs() * 1e-11,
        "pdf {x}: {at_pdf} != {pdf}"
      );
    }
    // The chi-square identity saturation and symmetric half-tail invariants.
    assert_eq!(normal_cdf(f64::INFINITY), 1.0);
    assert_eq!(normal_cdf(f64::NEG_INFINITY), 0.0);
    assert_eq!(normal_cdf(0.0), 0.5);
    assert!(normal_cdf(f64::NAN).is_nan());
    assert_eq!(normal_pdf(f64::INFINITY), 0.0);
    // Quantile: inverse consistency and direct R values on this design's
    // event proportion (452/630), which seeds the binary fits.
    let proportion = 452.0 / 630.0;
    let quantile = normal_quantile(proportion);
    assert!(
      (quantile - 0.57531339332650666).abs() < 1e-12,
      "quantile {quantile} for R qnorm(452/630)"
    );
    for (p, value) in [
      (0.02425, -1.9729610513118845),
      (0.5, 0.0_f64),
      (0.8413447460685429, 1.0),
      (0.975, 1.9599639845400534),
      (0.999, 3.0902323061678132),
    ] {
      let actual = normal_quantile(p);
      if value.abs() < 1e-8 {
        // At the flat point zero the representable answer is the plateau
        // edge, not R's symbolic zero; pin only its scale.
        assert!(actual.abs() < 1e-15, "quantile {p}: {actual} != {value}");
      } else {
        assert!(
          (actual - value).abs() <= (value.abs() * 1e-11).max(f64::MIN_POSITIVE),
          "quantile {p}: {actual} != {value}"
        );
      }
      // Round trip: the quantile's CDF must map back onto the probability.
      let back = normal_cdf(actual);
      assert!(
        (back - p).abs() <= (p * 1e-14).max(1e-17),
        "round trip {p}: {actual} -> {back}"
      );
    }
    // Past interior probabilities the tail-mirrored CDF quantizes results
    // near one in ulp(1) steps, so the bisection crossing resolves only to
    // that width in x space; pin the R point within that local resolution.
    let extreme_probability = 0.99999971334842808_f64;
    let extreme = normal_quantile(extreme_probability);
    let local_width = 2.0 * f64::EPSILON / normal_pdf(5.0);
    assert!(
      (extreme - 5.0000000000448512).abs() <= local_width,
      "extreme quantile {extreme} vs R 5.0000000000448512, width {local_width}"
    );
    assert!(
      (normal_cdf(extreme) - extreme_probability).abs() <= extreme_probability * 1e-14,
      "extreme round trip {extreme}"
    );
    assert!(normal_quantile(0.0).is_nan());
    assert!(normal_quantile(1.0).is_nan());
    assert!(normal_quantile(-0.5).is_nan());
  }
}
