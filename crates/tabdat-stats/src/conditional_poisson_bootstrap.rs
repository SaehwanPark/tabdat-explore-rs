//! Whole-group pairs bootstrap of the retained conditional-Poisson FE sample.
use crate::bootstrap::{BOOTSTRAP_RNG, BootstrapCoefficient, summarize_pairs};
use crate::poisson_bootstrap::draw_clusters;
use crate::{
  ConditionalPoissonOptions, EstimationProblem, EstimationSample, StatsError,
  fit_conditional_poisson,
};
use rand_xoshiro::{Xoshiro256PlusPlus, rand_core::SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Read-only conditional-FE pairs-bootstrap report; no model is installed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalPoissonBootstrapResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub entity_variable: String,
  pub joint_finite_observation_count: usize,
  pub observation_count: usize,
  pub candidate_group_count: usize,
  pub zero_total_group_count: usize,
  pub zero_total_observation_count: usize,
  pub replications: usize,
  pub seed: u64,
  pub rng: String,
  pub confidence_level: f64,
  pub interval_method: String,
  pub cluster_labels: Vec<String>,
  pub cluster_sizes: Vec<usize>,
  /// Zero-based indices into first-seen positive-total groups, in draw order.
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
    "conditional Poisson FE cluster pairs bootstrap: {}",
    message.into()
  ))
}

/// Resample only groups in the original positive-total FE estimation sample.
/// A repeated group is a distinct copy, and failed refits abort without redraw.
/// Group order is first seen among retained complete cases; one continuous RNG
/// stream supplies every draw. This report does not install model state.
pub fn bootstrap_conditional_poisson_clustered(
  problem: &EstimationProblem,
  entity_variable: &str,
  replications: usize,
  seed: u64,
) -> Result<ConditionalPoissonBootstrapResult, StatsError> {
  if !(2..=500).contains(&replications) {
    return Err(invalid("replications must be from 2 to 500"));
  }
  let k = problem.predictor_names.len();
  if !(1..=16).contains(&k) {
    return Err(invalid("requires 1 to 16 slopes"));
  }
  let original = fit_conditional_poisson(
    problem,
    entity_variable,
    &ConditionalPoissonOptions::default(),
  )
  .map_err(|e| invalid(format!("original fit failed: {e}")))?;
  let input_sample = problem
    .sample
    .as_ref()
    .expect("original fit validated sample");
  let input_groups = input_sample
    .cluster_groups
    .as_ref()
    .expect("original fit validated groups");
  let mut retained = vec![false; input_sample.total_observations];
  for &row in &original.sample.retained_indices {
    retained[row] = true;
  }
  let mut lookup = HashMap::new();
  let mut labels = Vec::new();
  let mut clusters: Vec<Vec<usize>> = Vec::new();
  for (row, (&source_row, group)) in input_sample
    .retained_indices
    .iter()
    .zip(input_groups)
    .enumerate()
  {
    if retained[source_row] {
      let index = *lookup.entry(group.as_str()).or_insert_with(|| {
        labels.push(group.clone());
        clusters.push(Vec::new());
        clusters.len() - 1
      });
      clusters[index].push(row);
    }
  }
  let g = clusters.len();
  if !(2..=256).contains(&g) {
    return Err(invalid(
      "2 to 256 retained positive-total groups are required",
    ));
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
    .is_none_or(|work| work > 2_500_000_000)
  {
    return Err(invalid(
      "G * largest group size * replications * K^2 must be at most 2500000000",
    ));
  }
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
    let sample = EstimationSample::new((0..count).collect(), count, 0, None, Some(keys))?;
    let copied =
      EstimationProblem::new(&problem.outcome, problem.predictor_names.clone(), y, rows)?
        .with_intercept(false)
        .with_sample(sample);
    let fit = fit_conditional_poisson(
      &copied,
      entity_variable,
      &ConditionalPoissonOptions::default(),
    )
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
  Ok(ConditionalPoissonBootstrapResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    entity_variable: entity_variable.to_string(),
    joint_finite_observation_count: original.joint_finite_observation_count,
    observation_count: original.observation_count,
    candidate_group_count: original.candidate_group_count,
    zero_total_group_count: original.zero_total_group_count,
    zero_total_observation_count: original.zero_total_observation_count,
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
