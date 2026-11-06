//! Whole-group pairs bootstrap of shared-gamma Poisson random effects.
use crate::bootstrap::{BOOTSTRAP_RNG, BootstrapCoefficient, summarize_pairs};
use crate::poisson_bootstrap::draw_clusters;
use crate::{
  EstimationProblem, EstimationSample, GammaPoissonOptions, StatsError, fit_gamma_poisson,
};
use rand_xoshiro::{Xoshiro256PlusPlus, rand_core::SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Read-only joint coefficient/dispersion bootstrap report, not a fitted model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GammaPoissonBootstrapResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub zero_total_group_count: usize,
  pub singleton_group_count: usize,
  pub replications: usize,
  pub seed: u64,
  pub rng: String,
  pub confidence_level: f64,
  pub interval_method: String,
  pub cluster_labels: Vec<String>,
  pub cluster_sizes: Vec<usize>,
  /// Zero-based first-seen complete-case group indices in draw order.
  pub replicate_cluster_indices: Vec<Vec<usize>>,
  pub replicate_observation_counts: Vec<usize>,
  /// Replicates in draw order; coefficients follow `covariance_parameter_names`.
  pub replicate_coefficients: Vec<Vec<f64>>,
  pub coefficients: Vec<BootstrapCoefficient>,
  pub covariance_parameter_names: Vec<String>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub sample: EstimationSample,
}
fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!(
    "shared-gamma Poisson RE cluster pairs bootstrap: {}",
    message.into()
  ))
}

/// Sample original whole groups, retaining all zero-total and singleton groups.
/// Each repeated copy has an independent shared-gamma effect; all regression
/// parameters and ln(alpha) are refit. Any failed copy aborts without redraw.
/// One continuous RNG stream uses first-seen complete-case group order.
pub fn bootstrap_gamma_poisson_clustered(
  problem: &EstimationProblem,
  entity_variable: &str,
  replications: usize,
  seed: u64,
) -> Result<GammaPoissonBootstrapResult, StatsError> {
  if !(2..=500).contains(&replications) {
    return Err(invalid("replications must be from 2 to 500"));
  }
  let k = problem
    .predictor_names
    .len()
    .checked_add(2)
    .ok_or_else(|| invalid("parameter count overflowed"))?;
  if !(3..=16).contains(&k) {
    return Err(invalid(
      "requires 1 to 14 slopes and at most 16 joint parameters",
    ));
  }
  let original = fit_gamma_poisson(problem, entity_variable, &GammaPoissonOptions::default())
    .map_err(|e| invalid(format!("original fit failed: {e}")))?;
  let sample = problem
    .sample
    .as_ref()
    .expect("original fit validated sample");
  let keys = sample
    .cluster_groups
    .as_ref()
    .expect("original fit validated groups");
  let mut lookup = HashMap::new();
  let mut labels = Vec::new();
  let mut clusters: Vec<Vec<usize>> = Vec::new();
  for (row, group) in keys.iter().enumerate() {
    let index = *lookup.entry(group.as_str()).or_insert_with(|| {
      labels.push(group.clone());
      clusters.push(Vec::new());
      clusters.len() - 1
    });
    clusters[index].push(row);
  }
  let g = clusters.len();
  if !(2..=256).contains(&g) {
    return Err(invalid("2 to 256 retained groups are required"));
  }
  let max_rows = g
    .checked_mul(clusters.iter().map(Vec::len).max().unwrap())
    .ok_or_else(|| invalid("maximum copied sample size overflowed"))?;
  if max_rows > 40_000 {
    return Err(invalid("G * largest group size must be at most 40000"));
  }
  if max_rows
    .checked_mul(replications)
    .and_then(|n| n.checked_mul(k))
    .and_then(|n| n.checked_mul(k))
    .is_none_or(|work| work > 3_000_000_000)
  {
    return Err(invalid(
      "G * largest group size * replications * joint K^2 must be at most 3000000000",
    ));
  }
  let names = original.covariance_parameter_names.clone();
  let point = original
    .coefficients
    .iter()
    .map(|c| c.value)
    .chain(std::iter::once(original.ln_alpha.value))
    .collect::<Vec<_>>();
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  let mut replicate_cluster_indices = Vec::with_capacity(replications);
  let mut replicate_observation_counts = Vec::with_capacity(replications);
  let mut replicate_coefficients = Vec::with_capacity(replications);
  let mut by_parameter = vec![Vec::with_capacity(replications); k];
  for replicate in 0..replications {
    let draws = draw_clusters(&mut rng, g)
      .map_err(|e| invalid(format!("replicate {} failed: {e}", replicate + 1)))?;
    let mut rows = Vec::new();
    let mut y = Vec::new();
    let mut keys = Vec::new();
    for (copy, &index) in draws.iter().enumerate() {
      let copy_key = format!("{copy:03}");
      for &row in &clusters[index] {
        rows.push(problem.design_matrix[row].clone());
        y.push(problem.response[row]);
        keys.push(copy_key.clone());
      }
    }
    let count = y.len();
    let copied_sample = EstimationSample::new((0..count).collect(), count, 0, None, Some(keys))?;
    let copied =
      EstimationProblem::new(&problem.outcome, problem.predictor_names.clone(), y, rows)?
        .with_intercept_name(&problem.intercept_name)
        .with_sample(copied_sample);
    let fit = fit_gamma_poisson(&copied, entity_variable, &GammaPoissonOptions::default())
      .map_err(|e| invalid(format!("replicate {} failed: {e}", replicate + 1)))?;
    let values = fit
      .coefficients
      .iter()
      .map(|c| c.value)
      .chain(std::iter::once(fit.ln_alpha.value))
      .collect::<Vec<_>>();
    for (column, &value) in by_parameter.iter_mut().zip(&values) {
      column.push(value);
    }
    replicate_cluster_indices.push(draws);
    replicate_observation_counts.push(count);
    replicate_coefficients.push(values);
  }
  let (coefficients, covariance_matrix) = summarize_pairs(&names, &point, by_parameter)?;
  Ok(GammaPoissonBootstrapResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    entity_variable: entity_variable.to_string(),
    observation_count: original.observation_count,
    zero_total_group_count: original.zero_total_group_count,
    singleton_group_count: original.singleton_group_count,
    replications,
    seed,
    rng: BOOTSTRAP_RNG.to_string(),
    confidence_level: 0.95,
    interval_method: "type7_percentile_95".to_string(),
    cluster_labels: labels,
    cluster_sizes: clusters.iter().map(Vec::len).collect(),
    replicate_cluster_indices,
    replicate_observation_counts,
    replicate_coefficients,
    coefficients,
    covariance_parameter_names: names,
    covariance_matrix,
    sample: original.sample,
  })
}
