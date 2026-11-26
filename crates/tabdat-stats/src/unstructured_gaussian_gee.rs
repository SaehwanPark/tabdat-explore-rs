//! Balanced Gaussian identity-link GEE with period-standardized unstructured R.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::matrix::{qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r};
use crate::{EstimationProblem, EstimationSample, GaussianGeeInference, StatsError};

const COEFFICIENT_TOLERANCE: f64 = 1e-6;
const SCORE_TOLERANCE: f64 = 1e-6;
const MAX_PERIODS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstructuredGaussianGeeOptions {
  pub max_iterations: usize,
}

impl Default for UnstructuredGaussianGeeOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Owned fit and inference for a balanced Gaussian unstructured GEE.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnstructuredGaussianGeeResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub time_variable: String,
  pub time_values: Vec<f64>,
  pub observation_count: usize,
  pub group_count: usize,
  pub period_count: usize,
  /// Scale estimated as Pearson RSS divided by N minus the parameter count.
  pub phi: f64,
  /// Symmetric working correlation with per-period standardized residual
  /// moments and unit diagonal. This differs from Stata's pooled denominator.
  pub correlation_matrix: Vec<Vec<f64>>,
  pub working: GaussianGeeInference,
  /// Uncorrected panel-cluster sandwich covariance, matching xtgee vce(robust).
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

fn residuals(x: &[Vec<f64>], y: &[f64], beta: &[f64]) -> Vec<f64> {
  x.iter()
    .zip(y)
    .map(|(row, y)| y - row.iter().zip(beta).map(|(x, b)| x * b).sum::<f64>())
    .collect()
}

fn max_scaled_difference(left: &[f64], right: &[f64]) -> f64 {
  left
    .iter()
    .zip(right)
    .map(|(left, right)| (left - right).abs() / (1.0 + left.abs()))
    .fold(0.0, f64::max)
}

fn pearson_correlation(
  residual: &[f64],
  groups: &[Vec<usize>],
  period_count: usize,
) -> Result<Vec<Vec<f64>>, StatsError> {
  let mut moment_covariance = vec![vec![0.0; period_count]; period_count];
  for first in 0..period_count {
    for second in first..period_count {
      let cross = groups
        .iter()
        .map(|group| residual[group[first]] * residual[group[second]])
        .sum::<f64>()
        / groups.len() as f64;
      if !cross.is_finite() {
        return Err(failed("nonfinite unstructured Pearson covariance moment"));
      }
      moment_covariance[first][second] = cross;
      moment_covariance[second][first] = cross;
    }
  }
  let standard_deviations = (0..period_count)
    .map(|period| moment_covariance[period][period].sqrt())
    .collect::<Vec<_>>();
  if standard_deviations
    .iter()
    .any(|value| !value.is_finite() || *value <= f64::MIN_POSITIVE)
  {
    return Err(failed(
      "nonfinite or numerically zero Pearson residual scale by period",
    ));
  }
  let mut correlation = vec![vec![0.0; period_count]; period_count];
  for (period, row) in correlation.iter_mut().enumerate() {
    row[period] = 1.0;
  }
  for first in 0..period_count {
    for second in first + 1..period_count {
      let value =
        moment_covariance[first][second] / standard_deviations[first] / standard_deviations[second];
      correlation[first][second] = value;
      correlation[second][first] = value;
    }
  }
  Ok(correlation)
}

fn cholesky(correlation: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  let size = correlation.len();
  let mut lower = vec![vec![0.0; size]; size];
  for row in 0..size {
    if correlation[row].len() != size {
      return Err(invalid("working correlation matrix must be square"));
    }
    for column in 0..=row {
      let value = correlation[row][column]
        - (0..column)
          .map(|inner| lower[row][inner] * lower[column][inner])
          .sum::<f64>();
      if row == column {
        if !value.is_finite() || value <= 1e-12 {
          return Err(failed(
            "unstructured Pearson working correlation is not numerically positive definite",
          ));
        }
        lower[row][column] = value.sqrt();
      } else {
        lower[row][column] = value / lower[column][column];
      }
    }
  }
  Ok(lower)
}

fn solve_lower(lower: &[Vec<f64>], rhs: &[f64]) -> Vec<f64> {
  let mut solution = vec![0.0; rhs.len()];
  for row in 0..rhs.len() {
    let correction = (0..row)
      .map(|column| lower[row][column] * solution[column])
      .sum::<f64>();
    solution[row] = (rhs[row] - correction) / lower[row][row];
  }
  solution
}

fn solve_upper_from_lower(lower: &[Vec<f64>], rhs: &[f64]) -> Vec<f64> {
  let mut solution = vec![0.0; rhs.len()];
  for row in (0..rhs.len()).rev() {
    let correction = ((row + 1)..rhs.len())
      .map(|column| lower[column][row] * solution[column])
      .sum::<f64>();
    solution[row] = (rhs[row] - correction) / lower[row][row];
  }
  solution
}

fn gls_solve(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  correlation: &[Vec<f64>],
  scales: &[f64],
) -> Result<(Vec<f64>, Vec<Vec<f64>>), StatsError> {
  let lower = cholesky(correlation)?;
  let k = scales.len();
  let mut whitened_x = Vec::with_capacity(x.len());
  let mut whitened_y = Vec::with_capacity(y.len());
  for group in groups {
    let mut prior_x = Vec::<Vec<f64>>::with_capacity(group.len());
    let mut prior_y = Vec::<f64>::with_capacity(group.len());
    for &index in group {
      let mut row = vec![0.0; k];
      for column in 0..k {
        let correction = (0..prior_x.len())
          .map(|period| lower[prior_x.len()][period] * prior_x[period][column])
          .sum::<f64>();
        row[column] =
          (x[index][column] / scales[column] - correction) / lower[prior_x.len()][prior_x.len()];
      }
      let correction = (0..prior_y.len())
        .map(|period| lower[prior_y.len()][period] * prior_y[period])
        .sum::<f64>();
      let response = (y[index] - correction) / lower[prior_y.len()][prior_y.len()];
      prior_x.push(row.clone());
      prior_y.push(response);
    }
    whitened_x.extend(prior_x);
    whitened_y.extend(prior_y);
  }
  if whitened_x
    .iter()
    .flatten()
    .chain(&whitened_y)
    .any(|value| !value.is_finite())
  {
    return Err(failed("nonfinite whitened design or response"));
  }
  let qr = qr_decompose_with_response(&whitened_x, &whitened_y)?;
  let beta = solve_upper_triangular(&qr.r, &qr.qty)?
    .iter()
    .zip(scales)
    .map(|(value, scale)| value / scale)
    .collect::<Vec<_>>();
  let normalized = xtx_inverse_from_r(&qr.r)?;
  let bread = (0..k)
    .map(|row| {
      (0..k)
        .map(|column| normalized[row][column] / scales[row] / scales[column])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  if beta
    .iter()
    .chain(bread.iter().flatten())
    .any(|value| !value.is_finite())
  {
    return Err(failed("nonfinite coefficients or GLS bread"));
  }
  Ok((beta, bread))
}

fn cluster_meat(
  x: &[Vec<f64>],
  residual: &[f64],
  groups: &[Vec<usize>],
  correlation: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, StatsError> {
  let lower = cholesky(correlation)?;
  let k = x[0].len();
  let mut meat = vec![vec![0.0; k]; k];
  for group in groups {
    let errors = group
      .iter()
      .map(|&index| residual[index])
      .collect::<Vec<_>>();
    let whitened = solve_lower(&lower, &errors);
    let correlated = solve_upper_from_lower(&lower, &whitened);
    let score = (0..k)
      .map(|column| {
        group
          .iter()
          .zip(&correlated)
          .map(|(&index, value)| x[index][column] * value)
          .sum::<f64>()
      })
      .collect::<Vec<_>>();
    for row in 0..k {
      for column in 0..k {
        meat[row][column] += score[row] * score[column];
      }
    }
  }
  if meat.iter().flatten().any(|value| !value.is_finite()) {
    return Err(failed("nonfinite panel-cluster score meat"));
  }
  Ok(meat)
}

/// Fit an unweighted, complete balanced Gaussian identity GEE.
/// `period_indices` aligns with the prepared problem; each value indexes the
/// ascending `time_values` vector and every group must contain each period once.
pub fn fit_unstructured_gaussian_gee(
  problem: &EstimationProblem,
  entity_variable: &str,
  time_variable: &str,
  time_values: &[f64],
  period_indices: &[usize],
  options: &UnstructuredGaussianGeeOptions,
) -> Result<UnstructuredGaussianGeeResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let slopes = problem.predictor_names.len();
  let k = slopes + 1;
  if !problem.include_intercept
    || problem.weights.is_some()
    || entity_variable.trim().is_empty()
    || time_variable.trim().is_empty()
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
      "requires intercept, unique K1..32 slopes, entity/time names, no weights and iteration bound1..100",
    ));
  }
  let period_count = time_values.len();
  if !(2..=MAX_PERIODS).contains(&period_count)
    || time_values.iter().any(|value| !value.is_finite())
    || time_values.windows(2).any(|pair| pair[0] >= pair[1])
    || period_indices.len() != n
    || period_indices.iter().any(|period| *period >= period_count)
  {
    return Err(invalid(
      "requires 2..10 finite ascending shared time values",
    ));
  }
  let sample = problem
    .sample
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned entity keys"))?;
  let keys = sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned entity keys"))?;
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
      "invalid sample mapping, weights, entity alignment or resource bound",
    ));
  }
  let mut grouped = BTreeMap::<&str, Vec<usize>>::new();
  for (index, key) in keys.iter().enumerate() {
    grouped.entry(key).or_default().push(index);
  }
  let mut groups = grouped.into_values().collect::<Vec<_>>();
  for group in &mut groups {
    group.sort_by_key(|&index| period_indices[index]);
    if group.len() != period_count
      || group
        .iter()
        .enumerate()
        .any(|(period, &index)| period_indices[index] != period)
    {
      return Err(invalid(
        "requires every entity to have each shared time value exactly once",
      ));
    }
  }
  let group_count = groups.len();
  if group_count > 10000 || group_count <= k {
    return Err(invalid(
      "requires at most10000 entities and G>parameter count",
    ));
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
    .map(|column| {
      x.iter()
        .map(|row| row[column].abs())
        .fold(0.0_f64, f64::max)
    })
    .collect::<Vec<_>>();
  if scales
    .iter()
    .any(|value| !value.is_finite() || *value == 0.0)
  {
    return Err(invalid("zero or nonfinite design column"));
  }

  let mut correlation = vec![vec![0.0; period_count]; period_count];
  for (period, row) in correlation.iter_mut().enumerate() {
    row[period] = 1.0;
  }
  let (mut beta, _) = gls_solve(&x, &problem.response, &groups, &correlation, &scales)?;
  let mut iterations = 0;
  let mut converged = false;
  while iterations < options.max_iterations {
    let residual = residuals(&x, &problem.response, &beta);
    correlation = pearson_correlation(&residual, &groups, period_count)?;
    let (next, _) = gls_solve(&x, &problem.response, &groups, &correlation, &scales)?;
    let change = max_scaled_difference(&next, &beta);
    beta = next;
    iterations += 1;
    if change <= COEFFICIENT_TOLERANCE {
      converged = true;
      break;
    }
  }
  if !converged {
    return Err(failed(
      "unstructured GLS iteration did not converge before iteration limit",
    ));
  }

  let residual = residuals(&x, &problem.response, &beta);
  let residual_ss = residual.iter().map(|value| value * value).sum::<f64>();
  let response_scale = problem
    .response
    .iter()
    .map(|value| value.abs())
    .fold(0.0_f64, f64::max);
  let phi = residual_ss / (n - k) as f64;
  if !residual_ss.is_finite()
    || residual_ss.sqrt() <= 128.0 * f64::EPSILON * response_scale
    || !phi.is_finite()
    || phi <= 0.0
  {
    return Err(failed("nonfinite or numerically zero Gaussian scale"));
  }
  correlation = pearson_correlation(&residual, &groups, period_count)?;
  let (score_step, bread) = gls_solve(&x, &problem.response, &groups, &correlation, &scales)?;
  let max_abs_scaled_score = max_scaled_difference(&score_step, &beta);
  if !max_abs_scaled_score.is_finite() || max_abs_scaled_score > SCORE_TOLERANCE {
    return Err(failed(
      "final unstructured inverse-bread-scaled score failed convergence guard",
    ));
  }
  let working_covariance = crate::matrix::scale(&bread, phi);
  let meat = cluster_meat(&x, &residual, &groups, &correlation)?;
  let sandwich_covariance = crate::matrix::sandwich(&bread, &meat)?;
  let names = problem.parameter_names();
  let working = crate::gaussian_gee::inference(&names, &beta, working_covariance)?;
  let sandwich = crate::gaussian_gee::inference(&names, &beta, sandwich_covariance)?;
  let fitted_means = x
    .iter()
    .map(|row| row.iter().zip(&beta).map(|(x, b)| x * b).sum::<f64>())
    .collect::<Vec<_>>();
  if fitted_means.iter().any(|value| !value.is_finite()) {
    return Err(failed("nonfinite fitted means"));
  }
  Ok(UnstructuredGaussianGeeResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    entity_variable: entity_variable.to_owned(),
    time_variable: time_variable.to_owned(),
    time_values: time_values.to_vec(),
    observation_count: n,
    group_count,
    period_count,
    phi,
    correlation_matrix: correlation,
    working,
    sandwich,
    iterations,
    max_abs_scaled_score,
    fitted_means,
    sample: sample.clone(),
  })
}
