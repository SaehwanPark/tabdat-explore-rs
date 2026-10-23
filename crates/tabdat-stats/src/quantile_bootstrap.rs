#![forbid(unsafe_code)]

use crate::bootstrap::{draw_index, summarize_pairs};
use crate::quantile::{quantile_design, quantile_point_coefficients};
use crate::{BOOTSTRAP_RNG, BootstrapCoefficient, EstimationProblem, EstimationSample, StatsError};
use rand_xoshiro::{Xoshiro256PlusPlus, rand_core::SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileBootstrapResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub quantiles: Vec<f64>,
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
  StatsError::InvalidEstimation(format!("quantile pairs bootstrap: {}", message.into()))
}

/// Joint IID pairs bootstrap: each replication shares its N row draws across
/// all quantiles. Marginal type7 percentile intervals do not provide simultaneous
/// coverage. Failed/ambiguous fits abort, with no discards or redraws.
pub fn bootstrap_quantiles(
  problem: &EstimationProblem,
  quantiles: &[f64],
  replications: usize,
  seed: u64,
) -> Result<QuantileBootstrapResult, StatsError> {
  if quantiles.is_empty()
    || quantiles.len() > 19
    || quantiles
      .iter()
      .any(|q| !q.is_finite() || !(0.05..=0.95).contains(q))
    || quantiles.windows(2).any(|w| w[0] >= w[1])
  {
    return Err(invalid(
      "supply 1..19 strictly increasing finite quantiles from 0.05 to 0.95",
    ));
  }
  if !(2..=crate::MAX_BOOTSTRAP_REPLICATIONS).contains(&replications) {
    return Err(invalid("replications must be from 2 to 5000"));
  }
  let design = quantile_design(problem, quantiles[0])?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  // Count every quantile LP, including the original fits, in the work budget.
  let work = n
    .checked_mul(k * k)
    .and_then(|v| v.checked_mul(quantiles.len()))
    .and_then(|v| v.checked_mul(replications + 1));
  if work.is_none_or(|v| v > crate::MAX_BOOTSTRAP_WORK) {
    return Err(invalid(
      "N*K²*quantiles*(replications+1) must be at most 100000000",
    ));
  }
  let mut names = Vec::with_capacity(k * quantiles.len());
  let mut original = Vec::with_capacity(k * quantiles.len());
  for &q in quantiles {
    let beta =
      quantile_point_coefficients(&design, &problem.response, q, problem.include_intercept)
        .map_err(|e| invalid(format!("original quantile {q} failed: {e}")))?;
    original.extend(beta);
    names.extend(
      problem
        .parameter_names()
        .iter()
        .map(|name| format!("q{q}:{name}")),
    );
  }
  let mut estimates = vec![Vec::with_capacity(replications); original.len()];
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  for replicate in 0..replications {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for _ in 0..n {
      let index = draw_index(&mut rng, n);
      x.push(design[index].clone());
      y.push(problem.response[index]);
    }
    for (i, &q) in quantiles.iter().enumerate() {
      let beta =
        quantile_point_coefficients(&x, &y, q, problem.include_intercept).map_err(|e| {
          invalid(format!(
            "replicate {} quantile {q} failed: {e}; no draws were discarded or retried",
            replicate + 1
          ))
        })?;
      for (column, value) in estimates[i * k..(i + 1) * k].iter_mut().zip(beta) {
        column.push(value);
      }
    }
  }
  let (coefficients, covariance_matrix) = summarize_pairs(&names, &original, estimates)?;
  Ok(QuantileBootstrapResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    observation_count: n,
    quantiles: quantiles.to_vec(),
    replications,
    seed,
    rng: BOOTSTRAP_RNG.into(),
    confidence_level: 95.0,
    interval_method: "percentile_r_type7".into(),
    coefficients,
    covariance_parameter_names: names,
    covariance_matrix,
    sample: problem.sample.clone(),
  })
}
