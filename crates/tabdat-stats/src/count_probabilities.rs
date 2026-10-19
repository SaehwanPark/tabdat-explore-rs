//! Descriptive observed/fitted count frequencies on an explicitly supplied sample.

use crate::distributions::chi_square_tail;
use crate::matrix::{add_compensated, beta_continued_fraction_bounded, lgamma};
use crate::{MAX_NB2_COUNT, MAX_POISSON_COUNT, StatsError};
use serde::{Deserialize, Serialize};

/// Largest admitted first count in the collapsed right-tail bin.
pub const MAX_COUNT_PROBABILITY_TAIL: usize = 100;

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

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("count probabilities: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("count probabilities: {message}"))
}
fn validate(law: CountProbabilityLaw, tail_from: usize) -> Result<(), StatsError> {
  if !(1..=MAX_COUNT_PROBABILITY_TAIL).contains(&tail_from) {
    return Err(invalid("tail cutoff must be an integer from 1 to 100"));
  }
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

// P(Y>=m) = P(Gamma(shape=m, rate=1)<=mu). Direct lower gamma series
// preserves a small Poisson tail that subtraction from a near-one CDF loses.
fn poisson_tail(mean: f64, m: usize) -> Result<f64, StatsError> {
  let a = m as f64;
  if mean >= a + 1.0 {
    return Ok(1.0 - chi_square_tail(2.0 * mean, 2 * m));
  }
  let prefactor = (a * mean.ln() - mean - lgamma(a)).exp();
  let (mut term, mut sum) = (1.0 / a, 1.0 / a);
  for j in 1..=10000 {
    term *= mean / (a + j as f64);
    sum += term;
    if term.abs() <= sum.abs() * 1e-15 {
      return Ok(prefactor * sum);
    }
  }
  Err(failed("Poisson survival series did not converge"))
}

// P(Y>=m) = I_x(m,r), x=mu/(mu+r), r=1/alpha. Compute both log
// coordinates, retaining a tiny 1-x even when x rounds to one. Integer m lets
// log B(m,r) use a finite product, avoiding cancellation of large log-gammas.
fn nb2_tail(mean: f64, r: f64, m: usize) -> Result<f64, StatsError> {
  let log_ratio = mean.ln() - r.ln();
  let (log_x, log_y) = (-softplus(-log_ratio), -softplus(log_ratio));
  let (mut product, mut correction) = (0.0, 0.0);
  for j in 0..m {
    add_compensated(&mut product, &mut correction, (r + j as f64).ln());
  }
  let log_beta = lgamma(m as f64) - product;
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
  let probability = (log_factor + fraction.ln() - a.ln()).exp();
  Ok(if complement {
    1.0 - probability
  } else {
    probability
  })
}

/// Conditional probabilities at 0..tail_from-1 and P(Y>=tail_from).
///
/// Requires a finite positive mean, cutoff 1..100 and the installed NB2 alpha
/// domain. Log PMFs preserve higher-count mass even if p0 underflows. Survival
/// uses direct gamma/beta tails, not 1-sum(PMF); no clipping or normalization.
pub fn count_bin_probabilities(
  law: CountProbabilityLaw,
  mean: f64,
  tail_from: usize,
) -> Result<Vec<f64>, StatsError> {
  validate(law, tail_from)?;
  if !mean.is_finite() || mean <= 0.0 {
    return Err(invalid("conditional means must be finite and positive"));
  }
  let log_mean = mean.ln();
  let (mut log_p, nb_denom) = match law {
    CountProbabilityLaw::Poisson => (-mean, None),
    CountProbabilityLaw::Nb2 { alpha } => {
      let r = 1.0 / alpha;
      let denom = softplus(log_mean - r.ln());
      (-r * denom, Some((r, denom)))
    }
  };
  let mut probabilities = Vec::with_capacity(tail_from + 1);
  for k in 0..tail_from {
    probabilities.push(log_p.exp());
    let next = (k + 1) as f64;
    log_p += log_mean - next.ln();
    if let Some((r, denom)) = nb_denom {
      log_p += (k as f64 / r).ln_1p() - denom;
    }
  }
  probabilities.push(match law {
    CountProbabilityLaw::Poisson => poisson_tail(mean, tail_from)?,
    CountProbabilityLaw::Nb2 { alpha } => nb2_tail(mean, 1.0 / alpha, tail_from)?,
  });
  if probabilities
    .iter()
    .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
    || (probabilities.iter().sum::<f64>() - 1.0).abs() > 1e-10
  {
    return Err(failed(
      "nonfinite, out-of-range or nonpartitioning probabilities",
    ));
  }
  Ok(probabilities)
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
  validate(law, tail_from)?;
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
