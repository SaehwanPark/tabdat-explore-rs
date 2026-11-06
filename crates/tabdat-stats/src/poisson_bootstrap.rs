//! Bounded whole-cluster pairs bootstrap of unweighted pooled Poisson coefficients.

use crate::bootstrap::{BOOTSTRAP_RNG, BootstrapCoefficient, summarize_pairs};
use crate::{EstimationProblem, EstimationSample, StatsError, fit_poisson};
use rand_xoshiro::{
  Xoshiro256PlusPlus,
  rand_core::{RngCore, SeedableRng},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_REPLICATIONS: usize = 500;
const MAX_GROUPS: usize = 256;
const MAX_ROWS_PER_DRAW: usize = 40_000;
const MAX_PARAMETERS: usize = 16;
const MAX_WORK: usize = 2_500_000_000;

/// Auditable pairs-bootstrap report; this is not a fitted or predict-capable model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterBootstrapPoissonResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub observation_count: usize,
  pub replications: usize,
  pub seed: u64,
  pub rng: String,
  pub confidence_level: f64,
  pub interval_method: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster_variable: Option<String>,
  pub cluster_labels: Vec<String>,
  pub cluster_sizes: Vec<usize>,
  /// Per replicate, zero-based indices into first-seen complete-case cluster labels.
  pub replicate_cluster_indices: Vec<Vec<usize>>,
  pub replicate_observation_counts: Vec<usize>,
  /// Replicates in draw order; coefficients follow `covariance_parameter_names`.
  pub replicate_coefficients: Vec<Vec<f64>>,
  pub coefficients: Vec<BootstrapCoefficient>,
  pub covariance_parameter_names: Vec<String>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!(
    "Poisson cluster pairs bootstrap: {}",
    message.into()
  ))
}

fn draw_clusters(rng: &mut impl RngCore, groups: usize) -> Result<Vec<usize>, StatsError> {
  let bound = groups as u64;
  let zone = u64::MAX - u64::MAX % bound;
  let mut draws = Vec::with_capacity(groups);
  for _ in 0..groups + 1024 {
    let word = rng.next_u64();
    if word < zone {
      draws.push((word % bound) as usize);
      if draws.len() == groups {
        return Ok(draws);
      }
    }
  }
  Err(invalid("RNG rejection bound exhausted"))
}

/// Sample G whole groups with replacement and refit the intercept and all slopes
/// on each copied sample. Uses one continuous xoshiro256++ stream, in the same
/// first-seen cluster order as the OLS pairs bootstrap. Any failed fit aborts;
/// draws are never discarded or retried. The report does not install fit state.
pub fn bootstrap_poisson_clustered(
  problem: &EstimationProblem,
  replications: usize,
  seed: u64,
) -> Result<ClusterBootstrapPoissonResult, StatsError> {
  problem.validate()?;
  if !(2..=MAX_REPLICATIONS).contains(&replications) {
    return Err(invalid("replications must be from 2 to 500"));
  }
  if !problem.include_intercept {
    return Err(invalid("an intercept is required"));
  }
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("only unweighted pairs are supported"));
  }
  let k = problem.parameter_count();
  if k > MAX_PARAMETERS {
    return Err(invalid("at most 16 coefficients are supported"));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|s| s.cluster_groups.as_ref())
    .ok_or_else(|| invalid("whole-cluster pairs require retained cluster identifiers"))?;
  if groups.len() != problem.observation_count() {
    return Err(invalid(
      "cluster identifier count must match retained observations",
    ));
  }
  let mut lookup = HashMap::new();
  let mut labels = Vec::new();
  let mut clusters: Vec<Vec<usize>> = Vec::new();
  for (row, group) in groups.iter().enumerate() {
    let index = *lookup.entry(group.as_str()).or_insert_with(|| {
      labels.push(group.clone());
      clusters.push(Vec::new());
      clusters.len() - 1
    });
    clusters[index].push(row);
  }
  let g = clusters.len();
  if !(2..=MAX_GROUPS).contains(&g) {
    return Err(invalid("2 to 256 retained clusters are required"));
  }
  let max_rows = g
    .checked_mul(clusters.iter().map(Vec::len).max().unwrap())
    .ok_or_else(|| invalid("maximum copied sample size overflowed"))?;
  if max_rows > MAX_ROWS_PER_DRAW {
    return Err(invalid("G * largest cluster size must be at most 40000"));
  }
  if max_rows
    .checked_mul(replications)
    .and_then(|n| n.checked_mul(k))
    .and_then(|n| n.checked_mul(k))
    .is_none_or(|work| work > MAX_WORK)
  {
    return Err(invalid(
      "G * largest cluster size * replications * K^2 must be at most 2500000000",
    ));
  }

  // The groups select copies, but the pooled likelihood is not a clustered fit.
  let mut original_problem = problem.clone();
  if let Some(sample) = &mut original_problem.sample {
    sample.cluster_groups = None;
  }
  let original = fit_poisson(&original_problem, false)?;
  let names = original
    .coefficients
    .iter()
    .map(|c| c.name.clone())
    .collect::<Vec<_>>();
  let point = original
    .coefficients
    .iter()
    .map(|c| c.value)
    .collect::<Vec<_>>();
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  let mut replicate_cluster_indices = Vec::with_capacity(replications);
  let mut replicate_observation_counts = Vec::with_capacity(replications);
  let mut replicate_coefficients = Vec::with_capacity(replications);
  let mut by_parameter = vec![Vec::with_capacity(replications); k];
  for replicate in 0..replications {
    let mut rows = Vec::new();
    let mut y = Vec::new();
    let draws = draw_clusters(&mut rng, g)
      .map_err(|e| invalid(format!("replicate {} failed: {e}", replicate + 1)))?;
    for &index in &draws {
      for &row in &clusters[index] {
        rows.push(problem.design_matrix[row].clone());
        y.push(problem.response[row]);
      }
    }
    let count = y.len();
    let copied =
      EstimationProblem::new(&problem.outcome, problem.predictor_names.clone(), y, rows)?
        .with_intercept_name(&problem.intercept_name);
    let fit = fit_poisson(&copied, false)
      .map_err(|e| invalid(format!("replicate {} failed: {e}", replicate + 1)))?;
    let values = fit.coefficients.iter().map(|c| c.value).collect::<Vec<_>>();
    for (column, &value) in by_parameter.iter_mut().zip(&values) {
      column.push(value);
    }
    replicate_cluster_indices.push(draws);
    replicate_observation_counts.push(count);
    replicate_coefficients.push(values);
  }
  let (coefficients, covariance_matrix) = summarize_pairs(&names, &point, by_parameter)?;
  Ok(ClusterBootstrapPoissonResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    observation_count: problem.observation_count(),
    replications,
    seed,
    rng: BOOTSTRAP_RNG.to_string(),
    confidence_level: 0.95,
    interval_method: "type7_percentile_95".to_string(),
    cluster_variable: None,
    cluster_labels: labels,
    cluster_sizes: clusters.iter().map(Vec::len).collect(),
    replicate_cluster_indices,
    replicate_observation_counts,
    replicate_coefficients,
    coefficients,
    covariance_parameter_names: names,
    covariance_matrix,
    sample: problem.sample.clone(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Rejected;
  impl RngCore for Rejected {
    fn next_u32(&mut self) -> u32 {
      u32::MAX
    }
    fn next_u64(&mut self) -> u64 {
      u64::MAX
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
      dest.fill(0xff);
    }
  }

  #[test]
  fn repeated_rng_rejection_is_bounded() {
    assert!(
      draw_clusters(&mut Rejected, 4)
        .unwrap_err()
        .to_string()
        .contains("RNG rejection bound exhausted")
    );
  }

  struct OneRejected(usize);
  impl RngCore for OneRejected {
    fn next_u32(&mut self) -> u32 {
      self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
      self.0 += 1;
      if self.0 == 1 { u64::MAX } else { 9 }
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
      dest.fill(0);
    }
  }

  #[test]
  fn rejected_word_is_not_counted_as_a_draw() {
    let mut rng = OneRejected(0);
    assert_eq!(draw_clusters(&mut rng, 10).unwrap(), vec![9; 10]);
    assert_eq!(rng.0, 11);
  }
}
