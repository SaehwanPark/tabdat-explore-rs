//! Descriptive observed/fitted count frequencies on an explicitly supplied sample.

use crate::matrix::{add_compensated, beta_continued_fraction_bounded, lgamma};
use crate::{MAX_NB2_COUNT, MAX_POISSON_COUNT, StatsError};
use serde::{Deserialize, Serialize};

/// Largest admitted first count in the collapsed right-tail bin.
pub const MAX_COUNT_PROBABILITY_TAIL: usize = 100;
/// Largest exact point-count cutoff shared by Poisson and NB2 predictions.
pub const MAX_COUNT_POINT_CUTOFF: usize = 100_000;

/// A plug-in distributional assumption, not a consequence of robust covariance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "family", rename_all = "snake_case")]
pub enum CountProbabilityLaw {
  Poisson,
  Nb2 { alpha: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountProbabilityBin {
  pub lower_bound: usize,
  /// Inclusive upper bound; absent for the final >= tail_from bin.
  pub upper_bound: Option<usize>,
  pub observed_count: usize,
  pub observed_fraction: f64,
  pub fitted_fraction: f64,
  pub expected_count: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountProbabilityTable {
  pub observation_count: usize,
  pub tail_from: usize,
  pub bins: Vec<CountProbabilityBin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CountPointProbabilityInterval {
  pub estimate: f64,
  pub lower: f64,
  pub upper: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CountPredictiveInterval {
  pub lower: usize,
  pub upper: usize,
  /// CDF-prefix terms used by the bounded quantile search.
  pub work_terms: usize,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("count probabilities: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("count probabilities: {message}"))
}
fn failed_for(quantity: &str, message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("count probabilities: {quantity} {message}"))
}
fn validate(
  law: CountProbabilityLaw,
  tail_from: usize,
  maximum_tail: usize,
  cutoff_error: &str,
) -> Result<(), StatsError> {
  if !(1..=maximum_tail).contains(&tail_from) {
    return Err(invalid(cutoff_error));
  }
  validate_law(law)
}

fn validate_law(law: CountProbabilityLaw) -> Result<(), StatsError> {
  if let CountProbabilityLaw::Nb2 { alpha } = law
    && (!alpha.is_finite() || alpha <= 0.0 || alpha.ln().abs() >= 12.0)
  {
    return Err(invalid(
      "NB2 alpha must have log(alpha) strictly in (-12,12)",
    ));
  }
  Ok(())
}
fn softplus(x: f64) -> f64 {
  if x > 0.0 {
    x + (-x).exp().ln_1p()
  } else {
    x.exp().ln_1p()
  }
}

// These bounded integer shapes use compensated finite sums rather than
// subtracting large log-gamma values in a small survival tail.
fn log_factorial(n: usize) -> f64 {
  let (mut sum, mut correction) = (0.0, 0.0);
  for j in 1..=n {
    add_compensated(&mut sum, &mut correction, (j as f64).ln());
  }
  sum
}

fn log1pmx(x: f64) -> f64 {
  if x.abs() >= 0.01 {
    return x.ln_1p() - x;
  }
  // The direct difference loses the quadratic residual when x is small.
  let (mut sum, mut correction) = (0.0, 0.0);
  let mut power = x * x;
  for n in 2..=100 {
    let term = if n % 2 == 0 {
      -power / n as f64
    } else {
      power / n as f64
    };
    add_compensated(&mut sum, &mut correction, term);
    if term.abs() <= sum.abs() * 1e-16 {
      break;
    }
    power *= x;
  }
  sum
}

fn poisson_log_prefactor(mean: f64, shape: f64) -> f64 {
  if shape < 64.0 {
    return shape * mean.ln() - mean - lgamma(shape);
  }
  // Near the mode, this rearrangement avoids subtracting O(shape*ln(shape)) terms.
  let delta = (mean - shape) / shape;
  let inverse = 1.0 / shape;
  let inverse_squared = inverse * inverse;
  let correction = inverse
    * (1.0 / 12.0
      + inverse_squared
        * (-1.0 / 360.0
          + inverse_squared
            * (1.0 / 1260.0
              + inverse_squared * (-1.0 / 1680.0 + inverse_squared * (1.0 / 1188.0)))));
  shape * log1pmx(delta) + 0.5 * (shape.ln() - std::f64::consts::TAU.ln()) - correction
}

// P(Y>=m) = P(Gamma(shape=m, rate=1)<=mu). Evaluate small tails directly.
// Complement the finite PMF sum only when mean>=m+1, where that CDF is safely
// away from one; this never manufactures a tiny tail by cancellation.
fn poisson_tail(mean: f64, m: usize, prefix: &[f64]) -> Result<f64, StatsError> {
  let a = m as f64;
  if mean >= a + 1.0 {
    let (mut sum, mut correction) = (0.0, 0.0);
    for p in prefix {
      add_compensated(&mut sum, &mut correction, *p);
    }
    return Ok(1.0 - sum);
  }
  let prefactor = poisson_log_prefactor(mean, a).exp();
  let (mut term, mut sum, mut correction) = (1.0 / a, 1.0 / a, 0.0);
  for j in 1..=10000 {
    term *= mean / (a + j as f64);
    add_compensated(&mut sum, &mut correction, term);
    if term.abs() <= sum.abs() * 1e-15 {
      return Ok(prefactor * sum);
    }
  }
  Err(failed("Poisson survival series did not converge"))
}

// Log P(Y>=m) for the small Poisson tail. The CDF caller selects this series
// only when the opposite tail is below one half, so its terms decrease.
fn poisson_log_small_tail(mean: f64, m: usize) -> Result<f64, StatsError> {
  if mean <= 0.0 || !mean.is_finite() || mean >= m as f64 + 1.0 {
    return Err(failed(
      "Poisson direct survival series requires a small tail",
    ));
  }
  let shape = m as f64;
  let log_prefactor = poisson_log_prefactor(mean, shape);
  let (mut term, mut sum, mut correction) = (1.0 / shape, 1.0 / shape, 0.0);
  for j in 1..=10000 {
    term *= mean / (shape + j as f64);
    add_compensated(&mut sum, &mut correction, term);
    if term <= sum * 1e-15 {
      let value = log_prefactor + sum.ln();
      if value.is_finite() && value <= 0.0 {
        return Ok(value);
      }
      return Err(failed("Poisson log survival is invalid"));
    }
  }
  Err(failed("Poisson survival series did not converge"))
}

// P(Y>=m) = I_x(m,r), x=mu/(mu+r), r=1/alpha. Compute both log
// coordinates, retaining a tiny 1-x even when x rounds to one. Integer m lets
// log B(m,r) use a finite product, avoiding cancellation of large log-gammas.
fn nb2_log_tail(mean: f64, r: f64, m: usize) -> Result<f64, StatsError> {
  let log_ratio = mean.ln() - r.ln();
  let (log_x, log_y) = (-softplus(-log_ratio), -softplus(log_ratio));
  let (mut product, mut correction) = (0.0, 0.0);
  for j in 0..m {
    add_compensated(&mut product, &mut correction, (r + j as f64).ln());
  }
  let log_beta = log_factorial(m - 1) - product;
  let log_factor = m as f64 * log_x + r * log_y - log_beta;
  let x = log_x.exp();
  let (a, b, coordinate, complement) = if x < (m as f64 + 1.0) / (m as f64 + r + 2.0) {
    (m as f64, r, x, false)
  } else {
    (r, m as f64, log_y.exp(), true)
  };
  let (fraction, converged) = beta_continued_fraction_bounded(a, b, coordinate, 10000);
  if !converged || !fraction.is_finite() || fraction <= 0.0 {
    return Err(failed("NB2 survival continued fraction did not converge"));
  }
  let log_probability = log_factor + fraction.ln() - a.ln();
  let probability = log_probability.exp();
  if !log_probability.is_finite() {
    return Err(failed("NB2 log survival probability is invalid"));
  }
  if complement && probability > 0.5 {
    // The beta convergence switch alone does not prevent a near-one CDF when
    // r is small. Sum its finite NB prefix excluding p0's unit term, retaining
    // log(CDF) accurately, then use expm1 rather than subtracting rounded CDF.
    let (mut sum, mut correction) = (0.0, 0.0);
    let mut term = r * x;
    for k in 1..m {
      add_compensated(&mut sum, &mut correction, term);
      term *= (r + k as f64) / (k + 1) as f64 * x;
    }
    let log_cdf = r * log_y + sum.ln_1p();
    if !log_cdf.is_finite() || log_cdf > 0.0 {
      return Err(failed("NB2 CDF complement is invalid"));
    }
    let log_survival = if log_cdf == 0.0 {
      f64::NEG_INFINITY
    } else if log_cdf > -std::f64::consts::LN_2 {
      (-log_cdf.exp_m1()).ln()
    } else {
      (-log_cdf.exp()).ln_1p()
    };
    return Ok(log_survival);
  }
  let log_tail = if complement {
    if probability == 1.0 {
      f64::NEG_INFINITY
    } else if probability > 0.5 {
      (-(probability.ln()).exp_m1()).ln()
    } else {
      (-probability).ln_1p()
    }
  } else {
    log_probability
  };
  if log_tail <= 0.0 {
    Ok(log_tail)
  } else {
    Err(failed("NB2 log survival probability is invalid"))
  }
}

fn nb2_tail(mean: f64, r: f64, m: usize) -> Result<f64, StatsError> {
  Ok(nb2_log_tail(mean, r, m)?.exp())
}

fn log_one_minus_exp(log_value: f64) -> Result<f64, StatsError> {
  if !log_value.is_finite() || log_value > 0.0 {
    return Err(failed("log probability complement is invalid"));
  }
  if log_value == 0.0 {
    return Ok(f64::NEG_INFINITY);
  }
  Ok(if log_value > -std::f64::consts::LN_2 {
    (-log_value.exp_m1()).ln()
  } else {
    (-log_value.exp()).ln_1p()
  })
}

fn add_log_positive_compensated(
  log_value: f64,
  scale: &mut f64,
  sum: &mut f64,
  correction: &mut f64,
) -> Result<(), StatsError> {
  if !log_value.is_finite() {
    return Err(failed("count CDF log-PMF term is not finite"));
  }
  if !scale.is_finite() || log_value > *scale {
    if scale.is_finite() {
      let rescale = (*scale - log_value).exp();
      *sum *= rescale;
      *correction *= rescale;
    }
    *scale = log_value;
    add_compensated(sum, correction, 1.0);
  } else {
    add_compensated(sum, correction, (log_value - *scale).exp());
  }
  Ok(())
}

fn count_log_prefix_probability(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
) -> Result<f64, StatsError> {
  if mean <= 0.0 || !mean.is_finite() {
    return Err(invalid("conditional means must be finite and positive"));
  }
  let log_mean = mean.ln();
  let (mut log_probability, log_probability_step) = match law {
    CountProbabilityLaw::Poisson => (-mean, None),
    CountProbabilityLaw::Nb2 { alpha } => {
      let shape = 1.0 / alpha;
      let log_shape = shape.ln();
      let log_mu_fraction = -softplus(log_shape - log_mean);
      (
        -shape * softplus(log_mean - log_shape),
        Some((shape, log_mu_fraction)),
      )
    }
  };
  let mut log_probability_correction = 0.0;
  let (mut scale, mut sum, mut sum_correction) = (f64::NEG_INFINITY, 0.0, 0.0);
  for k in 0..=count {
    add_log_positive_compensated(log_probability, &mut scale, &mut sum, &mut sum_correction)?;
    if k < count {
      let next = (k + 1) as f64;
      let step = if let Some((shape, log_mu_fraction)) = log_probability_step {
        ((shape - 1.0) / next).ln_1p() + log_mu_fraction
      } else {
        log_mean - next.ln()
      };
      add_compensated(&mut log_probability, &mut log_probability_correction, step);
    }
  }
  let result = scale + sum.ln();
  if !result.is_finite() || result > 64.0 * f64::EPSILON {
    return Err(failed("count CDF log probability is invalid"));
  }
  // A complete prefix can exceed one by a few rounding units only. Preserve
  // that endpoint as log(1) so the caller selects a directly evaluated tail.
  Ok(result.min(0.0))
}

fn count_zero_log_survival(law: CountProbabilityLaw, mean: f64) -> Result<f64, StatsError> {
  match law {
    CountProbabilityLaw::Poisson => Ok((-(-mean).exp_m1()).ln()),
    CountProbabilityLaw::Nb2 { alpha } => {
      let log_mean = mean.ln();
      let log_alpha = alpha.ln();
      let log_product = log_alpha + log_mean;
      if log_product < -36.0 {
        // The first-order log complement is log(mu); the omitted relative
        // correction is below double precision at this cutoff.
        return Ok(log_mean);
      }
      let log_a = softplus(log_product).ln() - log_alpha;
      if log_a < -36.0 {
        return Ok(log_a);
      }
      if log_a > 36.0 {
        return Ok(0.0);
      }
      let a = log_a.exp();
      Ok((-(-a).exp_m1()).ln())
    }
  }
}

fn count_log_probability_pair(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  cdf: bool,
) -> Result<(f64, f64), StatsError> {
  if !cdf {
    let Some(log_probability) = count_log_point_probability(law, mean, count)? else {
      return Err(invalid(
        "zero-mean probabilities have no logit representation",
      ));
    };
    let log_complement = if count == 0 {
      count_zero_log_survival(law, mean)?
    } else {
      log_one_minus_exp(log_probability)?
    };
    if log_complement == f64::NEG_INFINITY {
      return Err(failed("count PMF complement is not representable"));
    }
    return Ok((log_probability, log_complement));
  }

  if matches!(law, CountProbabilityLaw::Poisson) && mean < (count + 1) as f64 + 1.0 {
    let log_survival = poisson_log_small_tail(mean, count + 1)?;
    return Ok((log_one_minus_exp(log_survival)?, log_survival));
  }

  let log_cdf = count_log_prefix_probability(law, mean, count)?;
  if log_cdf <= -std::f64::consts::LN_2 {
    return Ok((log_cdf, log_one_minus_exp(log_cdf)?));
  }
  let log_survival = match law {
    CountProbabilityLaw::Poisson => poisson_log_small_tail(mean, count + 1)?,
    CountProbabilityLaw::Nb2 { alpha } => nb2_log_tail(mean, 1.0 / alpha, count + 1)?,
  };
  if !log_survival.is_finite() || log_survival > -std::f64::consts::LN_2 {
    return Err(failed("count CDF survival log probability is invalid"));
  }
  Ok((log_one_minus_exp(log_survival)?, log_survival))
}

fn count_predictive_tail_probability(level: f64) -> Result<f64, StatsError> {
  if !level.is_finite() || !(0.0..100.0).contains(&level) {
    return Err(invalid(
      "predictive interval level must be finite and strictly between 0 and 100",
    ));
  }
  let tail = (100.0 - level) / 200.0;
  if !tail.is_finite() || tail <= 0.0 || tail >= 0.5 {
    return Err(invalid(
      "predictive interval level is too close to 0 or 100 to represent equal tails",
    ));
  }
  Ok(tail)
}

fn count_predictive_cdf_pair(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  work_terms: &mut usize,
) -> Result<(f64, f64), StatsError> {
  *work_terms = work_terms
    .checked_add(count + 1)
    .ok_or_else(|| failed("predictive interval work count overflowed"))?;
  count_log_probability_pair(law, mean, count, true)
}

fn count_predictive_quantile(
  law: CountProbabilityLaw,
  mean: f64,
  log_tail: f64,
  find_upper: bool,
  work_terms: &mut usize,
) -> Result<usize, StatsError> {
  let reached = |log_cdf: f64, log_survival: f64| {
    if find_upper {
      log_survival <= log_tail
    } else {
      log_cdf >= log_tail
    }
  };
  let (log_cdf_zero, log_survival_zero) = count_predictive_cdf_pair(law, mean, 0, work_terms)?;
  if reached(log_cdf_zero, log_survival_zero) {
    return Ok(0);
  }

  let mut lower = 0usize;
  let mut high = 1usize;
  loop {
    let (log_cdf, log_survival) = count_predictive_cdf_pair(law, mean, high, work_terms)?;
    if reached(log_cdf, log_survival) {
      break;
    }
    lower = high;
    if high == MAX_COUNT_POINT_CUTOFF {
      return Err(failed_for(
        "predictive interval",
        "count endpoint exceeds the supported cutoff 100000",
      ));
    }
    high = high.saturating_mul(2).min(MAX_COUNT_POINT_CUTOFF);
  }

  while lower + 1 < high {
    let middle = lower + (high - lower) / 2;
    let (log_cdf, log_survival) = count_predictive_cdf_pair(law, mean, middle, work_terms)?;
    if reached(log_cdf, log_survival) {
      high = middle;
    } else {
      lower = middle;
    }
  }
  Ok(high)
}

/// Equal-tail predictive interval for a future count at a fixed current-row
/// Poisson/NB2 mean. Model parameters are treated as fixed; `level` controls
/// only count-outcome variation. Bounds are the first integer quantiles whose
/// lower CDF and upper survival satisfy the two tail probabilities. Quantile
/// search is bounded to 0..100000 and reports its CDF-prefix work for runtime
/// accounting. A zero mean returns the point mass [0, 0].
pub fn count_point_predictive_interval(
  law: CountProbabilityLaw,
  mean: f64,
  level: f64,
) -> Result<CountPredictiveInterval, StatsError> {
  validate_law(law)?;
  let tail = count_predictive_tail_probability(level)?;
  if !mean.is_finite() || mean < 0.0 {
    return Err(invalid("conditional means must be finite and nonnegative"));
  }
  if mean == 0.0 {
    return Ok(CountPredictiveInterval {
      lower: 0,
      upper: 0,
      work_terms: 0,
    });
  }
  let log_tail = tail.ln();
  let mut work_terms = 0usize;
  let lower = count_predictive_quantile(law, mean, log_tail, false, &mut work_terms)?;
  let upper = count_predictive_quantile(law, mean, log_tail, true, &mut work_terms)?;
  if lower > upper {
    return Err(failed("predictive interval endpoints are not ordered"));
  }
  Ok(CountPredictiveInterval {
    lower,
    upper,
    work_terms,
  })
}

fn inverse_logit(value: f64) -> f64 {
  if value >= 0.0 {
    1.0 / (1.0 + (-value).exp())
  } else {
    let exponential = value.exp();
    exponential / (1.0 + exponential)
  }
}

fn count_probability_interval_from_logs(
  estimate: f64,
  log_probability: f64,
  log_complement: f64,
  log_standard_error: Option<f64>,
  level: f64,
  quantity: &str,
) -> Result<CountPointProbabilityInterval, StatsError> {
  validate_count_interval_level(level)?;
  if !estimate.is_finite() {
    return Err(failed_for(quantity, "point estimate is not finite"));
  }
  let Some(log_standard_error) = log_standard_error else {
    return Ok(CountPointProbabilityInterval {
      estimate,
      lower: estimate,
      upper: estimate,
    });
  };
  let logit = log_probability - log_complement;
  let logit_standard_error = (log_standard_error - log_probability - log_complement).exp();
  let critical = crate::normal_quantile(0.5 + level / 200.0);
  if !logit.is_finite()
    || !logit_standard_error.is_finite()
    || logit_standard_error == 0.0
    || !critical.is_finite()
    || critical <= 0.0
  {
    return Err(failed_for(
      quantity,
      "logit interval gradient is not representable",
    ));
  }
  let half_width = critical * logit_standard_error;
  let lower_logit = logit - half_width;
  let upper_logit = logit + half_width;
  if !half_width.is_finite() || !lower_logit.is_finite() || !upper_logit.is_finite() {
    return Err(failed_for(
      quantity,
      "logit interval endpoints are not finite",
    ));
  }
  let lower = inverse_logit(lower_logit);
  let upper = inverse_logit(upper_logit);
  if !lower.is_finite() || !upper.is_finite() || lower < 0.0 || lower > upper || upper > 1.0 {
    return Err(failed_for(quantity, "logit interval endpoints are invalid"));
  }
  Ok(CountPointProbabilityInterval {
    estimate,
    lower,
    upper,
  })
}

fn validate_count_interval_level(level: f64) -> Result<(), StatsError> {
  if !level.is_finite() || !(0.0..100.0).contains(&level) {
    return Err(invalid(
      "interval level must be finite and strictly between 0 and 100",
    ));
  }
  let upper_probability = 0.5 + level / 200.0;
  if upper_probability <= 0.5 || upper_probability >= 1.0 {
    return Err(invalid(
      "interval level is too close to 0 or 100 to represent a finite normal critical value",
    ));
  }
  Ok(())
}

/// Conditional probabilities at 0..tail_from-1 and P(Y>=tail_from).
///
/// Requires a finite positive mean, cutoff 1..100 and the installed NB2 alpha
/// domain. Log PMFs preserve higher-count mass even if p0 underflows. Survival
/// evaluates small gamma/beta tails directly, complementing only well-conditioned
/// probabilities; no clipping or normalization.
pub fn count_bin_probabilities(
  law: CountProbabilityLaw,
  mean: f64,
  tail_from: usize,
) -> Result<Vec<f64>, StatsError> {
  validate(
    law,
    tail_from,
    MAX_COUNT_PROBABILITY_TAIL,
    "tail cutoff must be an integer from 1 to 100",
  )?;
  calculate_count_bin_probabilities(law, mean, tail_from)
}

fn calculate_count_bin_probabilities(
  law: CountProbabilityLaw,
  mean: f64,
  tail_from: usize,
) -> Result<Vec<f64>, StatsError> {
  if !mean.is_finite() || mean <= 0.0 {
    return Err(invalid("conditional means must be finite and positive"));
  }
  let log_mean = mean.ln();
  let (mut log_p, nb_denom) = match law {
    CountProbabilityLaw::Poisson => (-mean, None),
    CountProbabilityLaw::Nb2 { alpha } => {
      let r = 1.0 / alpha;
      let log_zero_denom = softplus(log_mean - r.ln());
      let log_mu_fraction = -softplus(r.ln() - log_mean);
      (-r * log_zero_denom, Some((r, log_mu_fraction)))
    }
  };
  let mut log_p_correction = 0.0;
  let mut probabilities = Vec::with_capacity(tail_from + 1);
  for k in 0..tail_from {
    probabilities.push(log_p.exp());
    let next = (k + 1) as f64;
    if let Some((r, log_mu_fraction)) = nb_denom {
      let relative_count_step = (r - 1.0) / next;
      add_compensated(
        &mut log_p,
        &mut log_p_correction,
        relative_count_step.ln_1p() + log_mu_fraction,
      );
    } else {
      add_compensated(&mut log_p, &mut log_p_correction, log_mean - next.ln());
    }
  }
  let tail = match law {
    CountProbabilityLaw::Poisson => poisson_tail(mean, tail_from, &probabilities)?,
    CountProbabilityLaw::Nb2 { alpha } => nb2_tail(mean, 1.0 / alpha, tail_from)?,
  };
  probabilities.push(tail);
  let (mut partition, mut partition_correction) = (0.0, 0.0);
  for &probability in &probabilities {
    add_compensated(&mut partition, &mut partition_correction, probability);
  }
  if probabilities
    .iter()
    .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
    || (partition - 1.0).abs() > 1e-10
  {
    return Err(failed(
      "nonfinite, out-of-range or nonpartitioning probabilities",
    ));
  }
  Ok(probabilities)
}

/// P(Y=count) and P(Y<=count), for exact counts 0..100000 and finite mean>=0.
/// Zero means include the degenerate limit reached by exponential underflow.
/// Use a direct prefix for small CDFs and complement direct survival for large
/// CDFs, avoiding subtraction of a rounded near-one survival.
pub fn count_point_probabilities(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
) -> Result<(f64, f64), StatsError> {
  let cutoff = count
    .checked_add(1)
    .filter(|&cutoff| cutoff <= MAX_COUNT_POINT_CUTOFF + 1)
    .ok_or_else(|| invalid("count must be an integer from 0 to 100000"))?;
  validate(
    law,
    cutoff,
    MAX_COUNT_POINT_CUTOFF + 1,
    "count must be an integer from 0 to 100000",
  )?;
  if mean == 0.0 {
    return Ok((if count == 0 { 1.0 } else { 0.0 }, 1.0));
  }
  let bins = calculate_count_bin_probabilities(law, mean, cutoff)?;
  let tail = bins[cutoff];
  let cumulative = if tail > 0.5 {
    let (mut sum, mut correction) = (0.0, 0.0);
    for &p in &bins[..cutoff] {
      add_compensated(&mut sum, &mut correction, p);
    }
    sum
  } else {
    1.0 - tail
  };
  Ok((bins[count], cumulative))
}

fn count_log_point_probability(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
) -> Result<Option<f64>, StatsError> {
  let cutoff = count
    .checked_add(1)
    .filter(|&cutoff| cutoff <= MAX_COUNT_POINT_CUTOFF + 1)
    .ok_or_else(|| invalid("count must be an integer from 0 to 100000"))?;
  validate(
    law,
    cutoff,
    MAX_COUNT_POINT_CUTOFF + 1,
    "count must be an integer from 0 to 100000",
  )?;
  if !mean.is_finite() || mean < 0.0 {
    return Err(invalid("conditional means must be finite and nonnegative"));
  }
  if mean == 0.0 {
    return Ok(None);
  }

  let log_mean = mean.ln();
  if matches!(law, CountProbabilityLaw::Poisson) {
    // Around the Poisson mode, use the stable Stirling rearrangement already
    // used by the tail calculation. Away from it, a short recurrence retains
    // finite log probabilities even when exp(log_p) underflows.
    let shape = count as f64 + 1.0;
    let delta = (mean - shape) / shape;
    if shape >= 64.0 && (-0.5..=0.5).contains(&delta) {
      return Ok(Some(poisson_log_prefactor(mean, shape) - log_mean));
    }
    let (mut log_p, mut correction) = (-mean, 0.0);
    for j in 1..=count {
      add_compensated(&mut log_p, &mut correction, log_mean - (j as f64).ln());
    }
    return Ok(Some(log_p));
  }

  let CountProbabilityLaw::Nb2 { alpha } = law else {
    unreachable!();
  };
  let shape = 1.0 / alpha;
  let log_shape = shape.ln();
  let log_mu_fraction = -softplus(log_shape - log_mean);
  let (mut log_p, mut correction) = (-shape * softplus(log_mean - log_shape), 0.0);
  for j in 0..count {
    let next = (j + 1) as f64;
    let relative_count_step = (shape - 1.0) / next;
    add_compensated(
      &mut log_p,
      &mut correction,
      relative_count_step.ln_1p() + log_mu_fraction,
    );
  }
  Ok(Some(log_p))
}

fn count_delta_log_standard_error(
  covariance: &[Vec<f64>],
  log_gradient_magnitudes: &[Option<f64>],
  gradient_signs: &[f64],
  quantity: &str,
) -> Result<Option<f64>, StatsError> {
  let parameter_count = covariance.len();
  if log_gradient_magnitudes.len() != parameter_count
    || gradient_signs.len() != parameter_count
    || covariance.iter().any(|row| row.len() != parameter_count)
  {
    return Err(StatsError::DimensionMismatch(format!(
      "count {quantity} covariance and gradient dimensions must match"
    )));
  }
  if covariance.iter().flatten().any(|value| !value.is_finite())
    || log_gradient_magnitudes
      .iter()
      .flatten()
      .any(|value| !value.is_finite())
    || gradient_signs.iter().any(|value| !value.is_finite())
  {
    return Err(failed_for(
      quantity,
      "gradient and covariance must be finite",
    ));
  }
  let Some(max_log_gradient) = log_gradient_magnitudes
    .iter()
    .flatten()
    .copied()
    .reduce(f64::max)
  else {
    return Ok(None);
  };

  // Scaling preserves a representable delta SE even when its probability
  // underflows, and bounds intermediate products for large finite inputs.
  let max_covariance = covariance
    .iter()
    .flatten()
    .fold(0.0_f64, |maximum, value| maximum.max(value.abs()));
  if max_covariance == 0.0 {
    return Ok(None);
  }
  let log_max_covariance = max_covariance.ln();
  let scaled_gradient: Vec<_> = log_gradient_magnitudes
    .iter()
    .zip(gradient_signs)
    .map(|(value, sign)| {
      value.map_or(0.0, |log_magnitude| {
        sign * (log_magnitude - max_log_gradient).exp()
      })
    })
    .collect();

  let (mut variance, mut variance_correction) = (0.0, 0.0);
  let (mut magnitude, mut magnitude_correction) = (0.0, 0.0);
  for i in 0..parameter_count {
    for j in 0..parameter_count {
      let term = scaled_gradient[i] * (covariance[i][j] / max_covariance) * scaled_gradient[j];
      if !term.is_finite() {
        return Err(failed_for(quantity, "delta variance is not finite"));
      }
      add_compensated(&mut variance, &mut variance_correction, term);
      add_compensated(&mut magnitude, &mut magnitude_correction, term.abs());
    }
  }
  if variance < 0.0 {
    let roundoff_bound = 64.0 * f64::EPSILON * magnitude;
    if variance >= -roundoff_bound {
      variance = 0.0;
    } else {
      return Err(failed_for(quantity, "delta variance is negative"));
    }
  }
  if variance == 0.0 {
    return Ok(None);
  }
  let log_standard_error = max_log_gradient + 0.5 * log_max_covariance + 0.5 * variance.ln();
  if !log_standard_error.is_finite() {
    return Err(failed_for(
      quantity,
      "log delta standard error is not finite",
    ));
  }
  Ok(Some(log_standard_error))
}

fn add_log_signed_compensated(
  log_magnitude: f64,
  sign: f64,
  scale: &mut f64,
  sum: &mut f64,
  correction: &mut f64,
) -> Result<(), StatsError> {
  if !log_magnitude.is_finite() || !sign.is_finite() {
    return Err(failed("NB2 CDF score term is not finite"));
  }
  if !scale.is_finite() || log_magnitude > *scale {
    if scale.is_finite() {
      let rescale = (*scale - log_magnitude).exp();
      *sum *= rescale;
      *correction *= rescale;
    }
    *scale = log_magnitude;
    add_compensated(sum, correction, sign);
  } else {
    add_compensated(sum, correction, sign * (log_magnitude - *scale).exp());
  }
  Ok(())
}

fn count_nb2_cdf_log_alpha_gradient(
  mean: f64,
  count: usize,
  alpha: f64,
) -> Result<Option<(f64, f64)>, StatsError> {
  let shape = 1.0 / alpha;
  let tail = nb2_tail(mean, shape, count + 1)?;
  if !tail.is_finite() || !(0.0..=1.0).contains(&tail) {
    return Err(failed("NB2 CDF survival probability is invalid"));
  }
  if tail == 0.0 {
    return Ok(None);
  }
  if tail < 0.5 {
    // Prefix score terms nearly cancel when the CDF is close to one. Differentiate
    // log survival instead; this uses the directly evaluated small tail.
    let step = 1e-5_f64;
    let tail_plus = nb2_tail(mean, shape * (-step).exp(), count + 1)?;
    let tail_minus = nb2_tail(mean, shape * step.exp(), count + 1)?;
    if !tail_plus.is_finite() || !tail_minus.is_finite() || tail_plus <= 0.0 || tail_minus <= 0.0 {
      return Err(failed("NB2 CDF survival derivative is not representable"));
    }
    let log_survival_slope = (tail_plus.ln() - tail_minus.ln()) / (2.0 * step);
    if !log_survival_slope.is_finite() {
      return Err(failed("NB2 CDF alpha gradient is not finite"));
    }
    if log_survival_slope == 0.0 {
      return Ok(None);
    }
    let log_gradient_magnitude = tail.ln() + log_survival_slope.abs().ln();
    if !log_gradient_magnitude.is_finite() {
      return Err(failed("NB2 CDF alpha gradient is not finite"));
    }
    return Ok(Some((log_gradient_magnitude, -log_survival_slope.signum())));
  }

  let log_shape = shape.ln();
  let log_mean = mean.ln();
  let log_ratio = log_mean - log_shape;
  let mean_fraction = (-softplus(-log_ratio)).exp();
  let shape_fraction = (-softplus(log_ratio)).exp();
  let zero_score = if log_ratio < 0.01_f64.ln() {
    let ratio = log_ratio.exp();
    shape * (log1pmx(ratio) + ratio * ratio / (1.0 + ratio))
  } else {
    shape * (softplus(log_ratio) - mean_fraction)
  };
  let log_mu_fraction = -softplus(-log_ratio);
  let (mut log_probability, mut log_probability_correction) = (-shape * softplus(log_ratio), 0.0);
  let (mut score, mut score_correction) = (zero_score, 0.0);
  let (mut scale, mut sum, mut sum_correction) = (f64::NEG_INFINITY, 0.0, 0.0);
  for j in 0..=count {
    if score != 0.0 {
      add_log_signed_compensated(
        log_probability + score.abs().ln(),
        score.signum(),
        &mut scale,
        &mut sum,
        &mut sum_correction,
      )?;
    }
    if j < count {
      let next = (j + 1) as f64;
      let relative_count_step = (shape - 1.0) / next;
      add_compensated(
        &mut log_probability,
        &mut log_probability_correction,
        relative_count_step.ln_1p() + log_mu_fraction,
      );
      let score_step = shape_fraction - shape / (shape + j as f64);
      add_compensated(&mut score, &mut score_correction, score_step);
    }
  }
  if !sum.is_finite() {
    return Err(failed("NB2 CDF alpha gradient is not finite"));
  }
  if scale == f64::NEG_INFINITY || sum == 0.0 {
    return Ok(None);
  }
  let log_gradient_magnitude = scale + sum.abs().ln();
  if !log_gradient_magnitude.is_finite() {
    return Err(failed("NB2 CDF alpha gradient is not finite"));
  }
  Ok(Some((log_gradient_magnitude, sum.signum())))
}

fn count_point_pmf_log_standard_error(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
) -> Result<Option<f64>, StatsError> {
  if design.is_empty() {
    return Err(invalid(
      "PMF design must contain at least one fitted coefficient",
    ));
  }
  let nuisance = if matches!(law, CountProbabilityLaw::Nb2 { .. }) {
    1
  } else {
    0
  };
  let parameter_count = design
    .len()
    .checked_add(nuisance)
    .ok_or_else(|| StatsError::DimensionMismatch("count PMF parameter count overflow".into()))?;
  if covariance.len() != parameter_count
    || covariance.iter().any(|row| row.len() != parameter_count)
  {
    return Err(StatsError::DimensionMismatch(format!(
      "count PMF covariance must be {parameter_count} by {parameter_count}"
    )));
  }
  if design.iter().any(|value| !value.is_finite())
    || covariance.iter().flatten().any(|value| !value.is_finite())
  {
    return Err(StatsError::IncompatibleHypothesis(
      "count PMF design and covariance must be finite".into(),
    ));
  }

  let Some(log_probability) = count_log_point_probability(law, mean, count)? else {
    // The installed zero-mean point-mass limit has a zero parameter gradient.
    return Ok(None);
  };

  let (eta_score, log_alpha_score) = match law {
    CountProbabilityLaw::Poisson => (count as f64 - mean, None),
    CountProbabilityLaw::Nb2 { alpha } => {
      let shape = 1.0 / alpha;
      let log_ratio = mean.ln() - shape.ln();
      let mean_fraction = (-softplus(-log_ratio)).exp();
      let shape_fraction = (-softplus(log_ratio)).exp();
      let (mut harmonic, mut harmonic_correction) = (0.0, 0.0);
      for j in 0..count {
        add_compensated(
          &mut harmonic,
          &mut harmonic_correction,
          1.0 / (shape + j as f64),
        );
      }
      // Differentiate the NB2 log PMF in eta=ln(mu) and ln(alpha). The finite
      // harmonic sum is psi(shape+k)-psi(shape), with no external special fn.
      let count_f = count as f64;
      let eta = count_f * shape_fraction - shape * mean_fraction;
      let log_shape = shape * harmonic - shape * softplus(log_ratio) + shape * mean_fraction
        - count_f * shape_fraction;
      (eta, Some(-log_shape))
    }
  };

  let mut log_gradient_magnitudes = Vec::with_capacity(parameter_count);
  let mut gradient_signs = Vec::with_capacity(parameter_count);
  for value in design {
    log_gradient_magnitudes.push(if eta_score == 0.0 || *value == 0.0 {
      None
    } else {
      Some(log_probability + eta_score.abs().ln() + value.abs().ln())
    });
    gradient_signs.push(eta_score.signum() * value.signum());
  }
  if let Some(score) = log_alpha_score {
    log_gradient_magnitudes.push(if score == 0.0 {
      None
    } else {
      Some(log_probability + score.abs().ln())
    });
    gradient_signs.push(score.signum());
  }
  count_delta_log_standard_error(covariance, &log_gradient_magnitudes, &gradient_signs, "PMF")
}

/// Pointwise delta-method SE for a current-row PMF, conditional on its design
/// and fixed offset/exposure. `design` and the square `covariance` use fitted
/// beta order, including an intercept when fitted; NB2 appends lnalpha. A zero
/// mean uses the installed point-mass limit and returns zero. Invalid laws,
/// means, counts, covariance dimensions, nonfinite inputs, or a materially
/// negative delta variance return an error.
pub fn count_point_pmf_standard_error(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
) -> Result<f64, StatsError> {
  let Some(log_standard_error) =
    count_point_pmf_log_standard_error(law, mean, count, design, covariance)?
  else {
    return Ok(0.0);
  };
  let standard_error = log_standard_error.exp();
  if !standard_error.is_finite() {
    return Err(failed_for("PMF", "delta standard error is not finite"));
  }
  Ok(standard_error)
}

fn count_point_cdf_log_standard_error(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
) -> Result<Option<f64>, StatsError> {
  if design.is_empty() {
    return Err(invalid(
      "CDF design must contain at least one fitted coefficient",
    ));
  }
  let nuisance = usize::from(matches!(law, CountProbabilityLaw::Nb2 { .. }));
  let parameter_count = design
    .len()
    .checked_add(nuisance)
    .ok_or_else(|| StatsError::DimensionMismatch("count CDF parameter count overflow".into()))?;
  if covariance.len() != parameter_count
    || covariance.iter().any(|row| row.len() != parameter_count)
  {
    return Err(StatsError::DimensionMismatch(format!(
      "count CDF covariance must be {parameter_count} by {parameter_count}"
    )));
  }
  if design.iter().any(|value| !value.is_finite())
    || covariance.iter().flatten().any(|value| !value.is_finite())
  {
    return Err(StatsError::IncompatibleHypothesis(
      "count CDF design and covariance must be finite".into(),
    ));
  }

  let Some(log_probability) = count_log_point_probability(law, mean, count)? else {
    return Ok(None);
  };
  let log_mean = mean.ln();
  let (log_eta_gradient, log_alpha_gradient) = match law {
    CountProbabilityLaw::Poisson => (log_mean + log_probability, None),
    CountProbabilityLaw::Nb2 { alpha } => {
      let shape = 1.0 / alpha;
      let log_shape = shape.ln();
      let log_shape_plus_count = if count == 0 {
        log_shape
      } else {
        log_shape + softplus((count as f64).ln() - log_shape)
      };
      let log_shape_plus_mean = log_shape + softplus(log_mean - log_shape);
      (
        log_mean + log_shape_plus_count - log_shape_plus_mean + log_probability,
        count_nb2_cdf_log_alpha_gradient(mean, count, alpha)?,
      )
    }
  };
  if !log_eta_gradient.is_finite() {
    return Err(failed("CDF eta gradient is not finite"));
  }

  let mut log_gradient_magnitudes = Vec::with_capacity(parameter_count);
  let mut gradient_signs = Vec::with_capacity(parameter_count);
  for value in design {
    log_gradient_magnitudes.push(if *value == 0.0 {
      None
    } else {
      Some(log_eta_gradient + value.abs().ln())
    });
    gradient_signs.push(-value.signum());
  }
  if let Some((log_magnitude, sign)) = log_alpha_gradient {
    log_gradient_magnitudes.push(Some(log_magnitude));
    gradient_signs.push(sign);
  } else if nuisance == 1 {
    log_gradient_magnitudes.push(None);
    gradient_signs.push(0.0);
  }
  count_delta_log_standard_error(covariance, &log_gradient_magnitudes, &gradient_signs, "CDF")
}

/// Pointwise delta-method SE for a current-row CDF, conditional on its design
/// and fixed offset/exposure. `design` and covariance use beta order, including
/// an intercept when fitted; NB2 appends lnalpha. A zero mean uses the installed
/// point-mass limit and returns zero. Invalid laws, means, counts, covariance
/// dimensions, nonfinite inputs, unrepresentable gradients or a materially
/// negative delta variance return an error.
pub fn count_point_cdf_standard_error(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
) -> Result<f64, StatsError> {
  let Some(log_standard_error) =
    count_point_cdf_log_standard_error(law, mean, count, design, covariance)?
  else {
    return Ok(0.0);
  };
  let standard_error = log_standard_error.exp();
  if !standard_error.is_finite() {
    return Err(failed_for("CDF", "delta standard error is not finite"));
  }
  Ok(standard_error)
}

fn count_point_probability_interval(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
  level: f64,
  cdf: bool,
) -> Result<CountPointProbabilityInterval, StatsError> {
  validate_count_interval_level(level)?;
  let (pmf, cumulative) = count_point_probabilities(law, mean, count)?;
  let estimate = if cdf { cumulative } else { pmf };
  let log_standard_error = if cdf {
    count_point_cdf_log_standard_error(law, mean, count, design, covariance)?
  } else {
    count_point_pmf_log_standard_error(law, mean, count, design, covariance)?
  };
  if mean == 0.0 || log_standard_error.is_none() {
    return Ok(CountPointProbabilityInterval {
      estimate,
      lower: estimate,
      upper: estimate,
    });
  }
  let (log_probability, log_complement) = count_log_probability_pair(law, mean, count, cdf)?;
  count_probability_interval_from_logs(
    estimate,
    log_probability,
    log_complement,
    log_standard_error,
    level,
    if cdf { "CDF" } else { "PMF" },
  )
}

/// Pointwise two-sided logit-Wald interval for a current-row PMF. The interval
/// uses asymptotic normal critical values, the selected covariance, fixed
/// predictors/offsets/exposure, and NB2's joint beta/lnalpha block. Zero means
/// and zero delta variance return the point estimate at both endpoints. The
/// level must be finite and strictly between 0 and 100.
pub fn count_point_pmf_interval(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
  level: f64,
) -> Result<CountPointProbabilityInterval, StatsError> {
  count_point_probability_interval(law, mean, count, design, covariance, level, false)
}

/// Pointwise two-sided logit-Wald interval for a current-row CDF, using the
/// selected covariance and asymptotic normal critical values. The endpoints
/// remain in [0,1]; levels and zero-variance limits match the PMF interval.
pub fn count_point_cdf_interval(
  law: CountProbabilityLaw,
  mean: f64,
  count: usize,
  design: &[f64],
  covariance: &[Vec<f64>],
  level: f64,
) -> Result<CountPointProbabilityInterval, StatsError> {
  count_point_probability_interval(law, mean, count, design, covariance, level, true)
}

/// Descriptive frequencies: average each individual's conditional probability,
/// not a probability at the average fitted mean. No uncertainty or test follows
/// from these bins, and covariance selection does not establish the fitted law.
pub fn count_probability_table(
  law: CountProbabilityLaw,
  response: &[f64],
  means: &[f64],
  tail_from: usize,
) -> Result<CountProbabilityTable, StatsError> {
  validate(
    law,
    tail_from,
    MAX_COUNT_PROBABILITY_TAIL,
    "tail cutoff must be an integer from 1 to 100",
  )?;
  if response.is_empty() {
    return Err(StatsError::EmptySample);
  }
  if response.len() != means.len() {
    return Err(invalid("outcomes and fitted means must align"));
  }
  let maximum = match law {
    CountProbabilityLaw::Poisson => MAX_POISSON_COUNT,
    CountProbabilityLaw::Nb2 { .. } => MAX_NB2_COUNT,
  };
  let mut counts = vec![0; tail_from + 1];
  let mut sums = vec![0.0; tail_from + 1];
  let mut corrections = vec![0.0; tail_from + 1];
  for (&y, &mean) in response.iter().zip(means) {
    if !y.is_finite() || y < 0.0 || y > maximum || y.fract() != 0.0 {
      return Err(invalid(
        "outcomes must be exact counts in the fitted family domain",
      ));
    }
    let index = if y >= tail_from as f64 {
      tail_from
    } else {
      y as usize
    };
    counts[index] += 1;
    for (i, probability) in count_bin_probabilities(law, mean, tail_from)?
      .into_iter()
      .enumerate()
    {
      add_compensated(&mut sums[i], &mut corrections[i], probability);
    }
  }
  let n = response.len();
  Ok(CountProbabilityTable {
    observation_count: n,
    tail_from,
    bins: counts
      .into_iter()
      .zip(sums)
      .enumerate()
      .map(|(i, (count, sum))| CountProbabilityBin {
        lower_bound: i,
        upper_bound: (i < tail_from).then_some(i),
        observed_count: count,
        observed_fraction: count as f64 / n as f64,
        fitted_fraction: sum / n as f64,
        expected_count: sum,
      })
      .collect(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bounded_beta_fraction_distinguishes_exhaustion_from_convergence() {
    let (_, converged) = beta_continued_fraction_bounded(10.0, 0.3, 0.5, 1);
    assert!(!converged);
    let (value, converged) = beta_continued_fraction_bounded(10.0, 0.3, 0.5, 10000);
    assert!(converged && value.is_finite() && value > 0.0);
  }
}
