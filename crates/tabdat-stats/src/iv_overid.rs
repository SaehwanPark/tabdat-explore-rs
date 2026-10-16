use crate::matrix::lgamma;
use crate::{EstimationProblem, InstrumentalVariablesProblem, StatsError, fit_least_squares};
use serde::{Deserialize, Serialize};

// Regularized upper incomplete gamma Q(df/2, statistic/2). Use a lower-tail
// series only where subtraction is well-conditioned, otherwise a continued
// fraction for the upper tail. Nonconvergence is unavailable, never a p-value.
fn chi_square_tail(statistic: f64, df: usize) -> f64 {
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

/// Homoskedastic, asymptotic Sargan N*R² test, not Basmann or robust Hansen J.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SarganResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub degrees_of_freedom: usize,
  pub statistic: f64,
  pub p_value: f64,
}

/// Test overidentifying restrictions using retained 2SLS structural residuals.
/// The caller must supply residuals from this exact unweighted design/sample,
/// not auxiliary second-stage residuals. No 2SLS refit or sample selection occurs.
/// Inference assumes homoskedasticity, a correctly specified maintained model,
/// and standard strong-identification asymptotics. Failure to reject does not
/// prove instrument validity or resolve weak identification.
pub fn sargan_test(
  problem: &InstrumentalVariablesProblem,
  residuals: &[f64],
) -> Result<SarganResult, StatsError> {
  problem.validate()?;
  if problem.instrument_names.len() <= problem.endogenous_names.len() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan requires overidentification: more excluded instruments than endogenous regressors"
        .into(),
    ));
  }
  if residuals.len() != problem.response.len() {
    return Err(StatsError::DimensionMismatch(
      "Sargan residuals must match the retained estimation sample".into(),
    ));
  }
  if residuals.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "Sargan structural residuals must be finite".into(),
    ));
  }
  let rss = residuals.iter().map(|value| value * value).sum::<f64>();
  if !rss.is_finite() || rss <= 0.0 {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan structural residual variance is undefined or non-finite".into(),
    ));
  }
  let names = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect();
  let design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(w, z)| w.iter().chain(z).copied().collect())
    .collect();
  let auxiliary = fit_least_squares(
    &EstimationProblem::new(
      "sargan_structural_residual",
      names,
      residuals.to_vec(),
      design,
    )?
    .with_intercept(problem.include_intercept),
  )?;
  let projected_rss = auxiliary
    .fitted_values
    .iter()
    .map(|value| value * value)
    .sum::<f64>();
  if !projected_rss.is_finite() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan projected residual norm is non-finite".into(),
    ));
  }
  // Direct projection norm avoids cancellation when the statistic is tiny.
  // Uncentered u'u also defines noconstant N*R²; no N-K correction. Projection
  // cannot increase the norm in exact arithmetic; cap roundoff at N.
  let statistic = problem.response.len() as f64 * (projected_rss / rss).min(1.0);
  let df = problem.instrument_names.len() - problem.endogenous_names.len();
  let p_value = chi_square_tail(statistic, df);
  if !statistic.is_finite() || !p_value.is_finite() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan statistic or chi-square tail is unavailable".into(),
    ));
  }
  Ok(SarganResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: residuals.len(),
    degrees_of_freedom: df,
    statistic,
    p_value,
  })
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
