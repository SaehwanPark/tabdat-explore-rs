#![forbid(unsafe_code)]

use crate::matrix::{add_compensated, qr_decompose_with_response, solve_upper_triangular};
use crate::{EstimationProblem, EstimationSample, StatsError, fit_least_squares};
use rand_xoshiro::{
  Xoshiro256PlusPlus,
  rand_core::{RngCore, SeedableRng},
};
use serde::{Deserialize, Serialize};

pub const BOOTSTRAP_RNG: &str = "xoshiro256++/splitmix64/u64-rejection-v1";
pub const MAX_BOOTSTRAP_REPLICATIONS: usize = 5000;
pub const MAX_BOOTSTRAP_WORK: usize = 100_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BootstrapCoefficient {
  pub name: String,
  pub estimate: f64,
  pub bootstrap_mean: f64,
  pub bias: f64,
  pub standard_error: f64,
  pub ci_lower: f64,
  pub ci_upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BootstrapOlsResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub replications: usize,
  pub seed: u64,
  pub rng: String,
  pub confidence_level: f64,
  pub interval_method: String,
  pub coefficients: Vec<BootstrapCoefficient>,
  pub covariance_parameter_names: Vec<String>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!("pairs bootstrap: {}", message.into()))
}

// A fixed-width rejection map avoids modulo bias and usize/platform-dependent
// sampling algorithms. Seed expansion is rand_xoshiro's documented SplitMix64.
fn draw_index(rng: &mut impl RngCore, bound: usize) -> usize {
  let bound = bound as u64;
  let zone = u64::MAX - u64::MAX % bound;
  loop {
    let value = rng.next_u64();
    if value < zone {
      return (value % bound) as usize;
    }
  }
}

fn average(values: &[f64]) -> f64 {
  let anchor = values[0];
  let (mut sum, mut correction) = (0.0, 0.0);
  for value in values {
    add_compensated(&mut sum, &mut correction, value - anchor);
  }
  anchor + sum / values.len() as f64
}

fn percentile(sorted: &[f64], probability: f64) -> f64 {
  let h = (sorted.len() - 1) as f64 * probability;
  let lo = h.floor() as usize;
  let hi = h.ceil() as usize;
  sorted[lo] + (sorted[hi] - sorted[lo]) * (h - lo as f64)
}

/// Seeded IID pairs bootstrap of unweighted OLS coefficients on the supplied
/// complete-finite sample. Every resample draws N whole (y,X) records. A failed
/// replicate aborts without redraw/discard; no conditional-on-success inference.
/// Percentile95 intervals use R type7 interpolation, not analytic t/normal tails.
pub fn bootstrap_ols(
  problem: &EstimationProblem,
  replications: usize,
  seed: u64,
) -> Result<BootstrapOlsResult, StatsError> {
  problem.validate()?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if !(2..=MAX_BOOTSTRAP_REPLICATIONS).contains(&replications) {
    return Err(invalid("replications must be from 2 to 5000"));
  }
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("only unweighted IID pairs are supported"));
  }
  let work = n
    .checked_mul(replications)
    .and_then(|v| v.checked_mul(k))
    .and_then(|v| v.checked_mul(k));
  if work.is_none_or(|v| v > MAX_BOOTSTRAP_WORK) {
    return Err(invalid("N * replications * K^2 must be at most 100000000"));
  }
  let original = fit_least_squares(problem)?;
  if original.coefficients.iter().any(|b| !b.value.is_finite()) {
    return Err(invalid("original fit produced nonfinite coefficients"));
  }
  let design: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|row| {
      let mut values = Vec::with_capacity(k);
      if problem.include_intercept {
        values.push(1.0);
      }
      values.extend_from_slice(row);
      values
    })
    .collect();
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  let mut estimates = vec![Vec::with_capacity(replications); k];
  for replicate in 0..replications {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for _ in 0..n {
      let index = draw_index(&mut rng, n);
      x.push(design[index].clone());
      y.push(problem.response[index]);
    }
    // Only the point fit is needed. A full-rank draw with zero residual variance
    // still has valid coefficients; analytic replicate SEs are irrelevant here.
    let beta = qr_decompose_with_response(&x, &y)
      .and_then(|qr| solve_upper_triangular(&qr.r, &qr.qty))
      .map_err(|e| {
        invalid(format!(
          "replicate {} failed: {e}; no draws were discarded or retried",
          replicate + 1
        ))
      })?;
    if beta.iter().any(|v| !v.is_finite()) {
      return Err(invalid(format!(
        "replicate {} produced nonfinite coefficients; no draws were discarded or retried",
        replicate + 1
      )));
    }
    for (column, value) in estimates.iter_mut().zip(beta) {
      column.push(value);
    }
  }
  let means: Vec<_> = estimates.iter().map(|v| average(v)).collect();
  let mut covariance = vec![vec![0.0; k]; k];
  for i in 0..k {
    for j in 0..=i {
      let (mut sum, mut correction) = (0.0, 0.0);
      for (&left, &right) in estimates[i].iter().zip(&estimates[j]) {
        add_compensated(
          &mut sum,
          &mut correction,
          (left - means[i]) * (right - means[j]),
        );
      }
      covariance[i][j] = sum / (replications - 1) as f64;
      covariance[j][i] = covariance[i][j];
    }
  }
  let coefficients: Vec<_> = estimates
    .iter_mut()
    .enumerate()
    .map(|(j, values)| {
      values.sort_by(f64::total_cmp);
      BootstrapCoefficient {
        name: original.parameter_names[j].clone(),
        estimate: original.coefficients[j].value,
        bootstrap_mean: means[j],
        bias: means[j] - original.coefficients[j].value,
        standard_error: covariance[j][j].sqrt(),
        ci_lower: percentile(values, 0.025),
        ci_upper: percentile(values, 0.975),
      }
    })
    .collect();
  if covariance.iter().flatten().any(|v| !v.is_finite())
    || coefficients.iter().any(|b| {
      [
        b.estimate,
        b.bootstrap_mean,
        b.bias,
        b.standard_error,
        b.ci_lower,
        b.ci_upper,
      ]
      .iter()
      .any(|v| !v.is_finite())
    })
  {
    return Err(invalid("coefficient summaries overflowed"));
  }
  Ok(BootstrapOlsResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    observation_count: n,
    replications,
    seed,
    rng: BOOTSTRAP_RNG.into(),
    confidence_level: 95.0,
    interval_method: "percentile_r_type7".into(),
    coefficients,
    covariance_parameter_names: original.parameter_names,
    covariance_matrix: covariance,
    sample: problem.sample.clone(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn every_draw_matches_independent_python_reference() {
    let mut compared = 0;
    for (case, seed) in [("multiple", u64::MAX), ("noconstant", 7), ("shifted", 7)] {
      let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
      for line in include_str!("../tests/fixtures/bootstrap/draws.csv")
        .lines()
        .skip(1)
      {
        let r: Vec<_> = line.split(',').collect();
        if r[0] != case {
          continue;
        }
        for expected in r[4].split(':') {
          assert_eq!(draw_index(&mut rng, 12), expected.parse::<usize>().unwrap());
          compared += 1;
        }
      }
    }
    assert_eq!(compared, 1440);
  }
}

#[cfg(test)]
mod rejection_test {
  use super::*;
  struct Fixed(std::vec::IntoIter<u64>);
  impl RngCore for Fixed {
    fn next_u32(&mut self) -> u32 {
      unreachable!("contract uses next_u64")
    }
    fn next_u64(&mut self) -> u64 {
      self.0.next().unwrap()
    }
    fn fill_bytes(&mut self, _: &mut [u8]) {
      unreachable!("contract uses next_u64")
    }
  }
  #[test]
  fn rejects_the_incomplete_upper_residue_block() {
    let zone = u64::MAX - u64::MAX % 10;
    let mut rng = Fixed(vec![u64::MAX, zone, 9].into_iter());
    assert_eq!(draw_index(&mut rng, 10), 9);
    assert!(rng.0.next().is_none());
  }
}
