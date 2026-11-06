//! Gaussian identity-link exchangeable GEE with pair-weighted Pearson moments.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::distributions::chi_square_tail;
use crate::matrix::{
  choice_information_inverse, qr_decompose_with_response, solve_upper_triangular,
  xtx_inverse_from_r,
};
use crate::{EstimationProblem, EstimationSample, LogitEstimate, StatsError, normal_cdf};

const NORMAL_95: f64 = 1.959963984540054;
const COEFFICIENT_TOLERANCE: f64 = 1e-12;
const SCORE_TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaussianGeeOptions {
  pub max_iterations: usize,
}
impl Default for GaussianGeeOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Normal and chi-square inference for one explicitly labeled covariance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaussianGeeInference {
  pub coefficients: Vec<LogitEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub wald_statistic: f64,
  pub wald_df: usize,
  pub wald_p_value: f64,
}

/// Owned marginal mean model and both uncorrected working/sandwich inferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaussianGeeResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub group_count: usize,
  pub minimum_group_size: usize,
  pub maximum_group_size: usize,
  pub singleton_group_count: usize,
  pub ordered_pair_count: usize,
  pub phi: f64,
  pub rho: f64,
  pub working: GaussianGeeInference,
  pub sandwich: GaussianGeeInference,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub fitted_means: Vec<f64>,
  pub sample: EstimationSample,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("xtgee: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("xtgee: {message}"))
}

// Applying the symmetric inverse square root preserves admissible negative rho;
// it is not a random-intercept variance-component transformation.
fn whiten_and_solve(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  scales: &[f64],
  phi: f64,
  rho: f64,
) -> Result<(Vec<f64>, Vec<Vec<f64>>), StatsError> {
  let k = scales.len();
  let max_size = groups.iter().map(Vec::len).max().unwrap_or(0);
  if !phi.is_finite()
    || phi <= 0.0
    || !rho.is_finite()
    || rho >= 1.0
    || 1.0 + (max_size - 1) as f64 * rho <= 0.0
  {
    return Err(failed(
      "nonpositive scale or non-positive-definite exchangeable correlation",
    ));
  }
  let multiplier = 1.0 / (phi * (1.0 - rho)).sqrt();
  let mut wx = vec![vec![0.0; k]; x.len()];
  let mut wy = vec![0.0; y.len()];
  for group in groups {
    let n = group.len() as f64;
    let theta = 1.0 - ((1.0 - rho) / (1.0 + (n - 1.0) * rho)).sqrt();
    let ym = group.iter().map(|&i| y[i]).sum::<f64>() / n;
    let xm = (0..k)
      .map(|j| group.iter().map(|&i| x[i][j]).sum::<f64>() / n)
      .collect::<Vec<_>>();
    for &i in group {
      wy[i] = multiplier * (y[i] - theta * ym);
      for j in 0..k {
        wx[i][j] = multiplier * (x[i][j] - theta * xm[j]) / scales[j];
      }
    }
  }
  if wy.iter().chain(wx.iter().flatten()).any(|v| !v.is_finite()) {
    return Err(failed("nonfinite whitened design or response"));
  }
  let qr = qr_decompose_with_response(&wx, &wy)?;
  let b = solve_upper_triangular(&qr.r, &qr.qty)?
    .iter()
    .zip(scales)
    .map(|(v, s)| v / s)
    .collect::<Vec<_>>();
  let normalized = xtx_inverse_from_r(&qr.r)?;
  let covariance = (0..k)
    .map(|i| {
      (0..k)
        .map(|j| normalized[i][j] / scales[i] / scales[j])
        .collect()
    })
    .collect::<Vec<Vec<f64>>>();
  if b
    .iter()
    .chain(covariance.iter().flatten())
    .any(|v| !v.is_finite())
  {
    return Err(failed("nonfinite coefficients or working covariance"));
  }
  Ok((b, covariance))
}

fn moments(
  y: &[f64],
  x: &[Vec<f64>],
  groups: &[Vec<usize>],
  b: &[f64],
  pairs: usize,
) -> Result<(Vec<f64>, f64, f64), StatsError> {
  let residual = y
    .iter()
    .zip(x)
    .map(|(y, row)| y - row.iter().zip(b).map(|(x, b)| x * b).sum::<f64>())
    .collect::<Vec<_>>();
  let sumsq = residual.iter().map(|v| v * v).sum::<f64>();
  let phi = sumsq / y.len() as f64;
  let cross = groups
    .iter()
    .map(|g| {
      let sum = g.iter().map(|&i| residual[i]).sum::<f64>();
      sum * sum - g.iter().map(|&i| residual[i] * residual[i]).sum::<f64>()
    })
    .sum::<f64>();
  let rho = cross / (phi * pairs as f64);
  let response_scale = y.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
  if !phi.is_finite()
    || phi.sqrt() <= 128.0 * f64::EPSILON * response_scale
    || !rho.is_finite()
    || residual.iter().any(|v| !v.is_finite())
  {
    return Err(failed(
      "nonfinite or numerically zero residual scale or correlation",
    ));
  }
  Ok((residual, phi, rho))
}

fn inference(
  names: &[String],
  b: &[f64],
  covariance: Vec<Vec<f64>>,
) -> Result<GaussianGeeInference, StatsError> {
  let k = b.len();
  let se = (0..k).map(|i| covariance[i][i].sqrt()).collect::<Vec<_>>();
  if se.iter().any(|v| !v.is_finite() || *v <= 0.0)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(failed("nonpositive or nonfinite coefficient variance"));
  }
  let coefficients = (0..k)
    .map(|i| {
      let statistic = b[i] / se[i];
      LogitEstimate {
        name: names[i].clone(),
        value: b[i],
        standard_error: se[i],
        statistic,
        p_value: 2.0 * normal_cdf(-statistic.abs()),
        confidence_lower: b[i] - NORMAL_95 * se[i],
        confidence_upper: b[i] + NORMAL_95 * se[i],
      }
    })
    .collect::<Vec<_>>();
  // Standardizing the slope covariance keeps its Cholesky scale-independent.
  let corr = (1..k)
    .map(|i| (1..k).map(|j| covariance[i][j] / se[i] / se[j]).collect())
    .collect::<Vec<Vec<f64>>>();
  let inverse = choice_information_inverse(&corr)?;
  let z = (1..k).map(|i| b[i] / se[i]).collect::<Vec<_>>();
  let wald = (0..k - 1)
    .map(|i| (0..k - 1).map(|j| z[i] * inverse[i][j] * z[j]).sum::<f64>())
    .sum::<f64>();
  let p = chi_square_tail(wald, k - 1);
  if !wald.is_finite()
    || wald < 0.0
    || !p.is_finite()
    || coefficients.iter().any(|c| {
      !c.statistic.is_finite()
        || !c.p_value.is_finite()
        || !c.confidence_lower.is_finite()
        || !c.confidence_upper.is_finite()
    })
  {
    return Err(failed("nonfinite normal or Wald inference"));
  }
  Ok(GaussianGeeInference {
    coefficients,
    covariance,
    wald_statistic: wald,
    wald_df: k - 1,
    wald_p_value: p,
  })
}

/// Fit jointly finite, sample-aligned rows; callers select missing cases first.
/// Requires an intercept, explicit group keys and no weights. Both working and
/// uncorrected unit-sandwich normal inference are always returned.
pub fn fit_gaussian_gee(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &GaussianGeeOptions,
) -> Result<GaussianGeeResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let slopes = problem.predictor_names.len();
  let k = slopes + 1;
  if !problem.include_intercept
    || problem.weights.is_some()
    || entity_variable.trim().is_empty()
    || options.max_iterations == 0
    || options.max_iterations > 100
    || !(1..=32).contains(&slopes)
    || problem
      .predictor_names
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != slopes
    || problem
      .predictor_names
      .iter()
      .any(|name| name == &problem.intercept_name)
  {
    return Err(invalid(
      "requires intercept, unique K1..32 slopes, group name, no weights and iteration bound1..100",
    ));
  }
  let sample = problem
    .sample
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  let keys = sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  if sample.weights.is_some()
    || keys.len() != n
    || n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
    || sample.total_observations > 40000
    || sample.total_observations.saturating_mul(k * k) > 20_000_000
    || sample.retained_indices.len() != n
    || sample
      .retained_indices
      .iter()
      .any(|&i| i >= sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != n
  {
    return Err(invalid(
      "invalid sample mapping, weights, group alignment or resource bound",
    ));
  }
  let mut map = BTreeMap::<&str, Vec<usize>>::new();
  for (i, key) in keys.iter().enumerate() {
    map.entry(key).or_default().push(i);
  }
  let groups = map.into_values().collect::<Vec<_>>();
  let g = groups.len();
  if g > 10000 || g <= k {
    return Err(invalid(
      "requires at most10000 groups and G>parameter count",
    ));
  }
  let min_size = groups.iter().map(Vec::len).min().unwrap();
  let max_size = groups.iter().map(Vec::len).max().unwrap();
  let pairs = groups
    .iter()
    .map(|group| group.len() * (group.len() - 1))
    .sum::<usize>();
  if pairs == 0 {
    return Err(invalid("requires at least one within-group pair"));
  }
  let x = problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::once(1.0)
        .chain(row.iter().copied())
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let scales = (0..k)
    .map(|j| x.iter().map(|row| row[j].abs()).fold(0.0, f64::max))
    .collect::<Vec<_>>();
  if scales.iter().any(|v| !v.is_finite() || *v == 0.0) {
    return Err(invalid("zero or nonfinite design column"));
  }
  let y = &problem.response;
  let (mut b, _) = whiten_and_solve(&x, y, &groups, &scales, 1.0, 0.0)?;
  let mut iterations = 0;
  let mut converged = false;
  while iterations < options.max_iterations {
    let (_, phi, rho) = moments(y, &x, &groups, &b, pairs)?;
    let (next, _) = whiten_and_solve(&x, y, &groups, &scales, phi, rho)?;
    iterations += 1;
    let relative = next
      .iter()
      .zip(&b)
      .map(|(a, b)| (a - b).abs() / (1.0 + b.abs()))
      .fold(0.0, f64::max);
    b = next;
    if relative <= COEFFICIENT_TOLERANCE {
      converged = true;
      break;
    }
  }
  if !converged || iterations >= options.max_iterations {
    return Err(failed(
      "coefficient iteration did not converge strictly before iteration limit",
    ));
  }
  let (residual, phi, rho) = moments(y, &x, &groups, &b, pairs)?;
  let (step_fit, working) = whiten_and_solve(&x, y, &groups, &scales, phi, rho)?;
  let max_scaled_score = step_fit
    .iter()
    .zip(&b)
    .map(|(a, b)| (a - b).abs() / (1.0 + b.abs()))
    .fold(0.0, f64::max);
  if !max_scaled_score.is_finite() || max_scaled_score > SCORE_TOLERANCE {
    return Err(failed(
      "final inverse-bread-scaled score failed convergence guard",
    ));
  }
  let mut meat = vec![vec![0.0; k]; k];
  for group in &groups {
    let size = group.len() as f64;
    let adjustment = rho / (1.0 + (size - 1.0) * rho);
    let sum = group.iter().map(|&i| residual[i]).sum::<f64>();
    let score = (0..k)
      .map(|j| {
        group
          .iter()
          .map(|&i| x[i][j] * (residual[i] - adjustment * sum))
          .sum::<f64>()
          / (phi * (1.0 - rho))
      })
      .collect::<Vec<_>>();
    for i in 0..k {
      for j in 0..k {
        meat[i][j] += score[i] * score[j];
      }
    }
  }
  let sandwich = crate::matrix::sandwich(&working, &meat)?;
  let names = problem.parameter_names();
  let working = inference(&names, &b, working)?;
  let sandwich = inference(&names, &b, sandwich)?;
  let fitted_means = x
    .iter()
    .map(|row| row.iter().zip(&b).map(|(x, b)| x * b).sum::<f64>())
    .collect::<Vec<_>>();
  if fitted_means.iter().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite fitted means"));
  }
  Ok(GaussianGeeResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    entity_variable: entity_variable.into(),
    observation_count: n,
    group_count: g,
    minimum_group_size: min_size,
    maximum_group_size: max_size,
    singleton_group_count: groups.iter().filter(|group| group.len() == 1).count(),
    ordered_pair_count: pairs,
    phi,
    rho,
    working,
    sandwich,
    iterations,
    max_abs_scaled_score: max_scaled_score,
    fitted_means,
    sample: sample.clone(),
  })
}
