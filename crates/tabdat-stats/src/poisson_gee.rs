//! Balanced Poisson/log-link exchangeable GEE, fixed working scale1.
#![forbid(unsafe_code)]

use crate::matrix::{
  add_compensated, qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r,
};
use crate::{EstimationProblem, EstimationSample, GaussianGeeInference, StatsError, fit_poisson};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Normal/chi-square inference; the result labels the covariance correction.
pub type PoissonGeeInference = GaussianGeeInference;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoissonGeeOptions {
  pub max_iterations: usize,
}
impl Default for PoissonGeeOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Owned marginal mean model; neither covariance defines a joint count likelihood.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoissonGeeResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub group_count: usize,
  pub group_size: usize,
  pub zero_total_group_count: usize,
  pub zero_count: usize,
  pub count_sum: f64,
  pub rho: f64,
  /// Pearson sum of squares / N normalizes rho, not the working covariance.
  pub pearson_normalizer: f64,
  pub working_scale: f64,
  pub sandwich_correction: f64,
  pub working: PoissonGeeInference,
  pub sandwich: PoissonGeeInference,
  pub iterations: usize,
  /// Score divided coordinatewise by one plus absolute whitened row contributions.
  pub max_abs_scaled_score: f64,
  /// Expected-information step in RMS coefficient coordinates, relative to 1+|beta|.
  pub max_abs_scaled_newton_step: f64,
  pub fitted_means: Vec<f64>,
  pub sample: EstimationSample,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("xtgee poisson: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("xtgee poisson: {message}"))
}
fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
  let (mut value, mut correction) = (0.0, 0.0);
  for x in values {
    add_compensated(&mut value, &mut correction, x);
  }
  value
}
const GUARD: f64 = 128.0 * f64::EPSILON;
struct Equations {
  mu: Vec<f64>,
  rho: f64,
  normalizer: f64,
  white_design: Vec<Vec<f64>>,
  white_residual: Vec<f64>,
  group_scores: Vec<Vec<f64>>,
  group_bounds: Vec<Vec<f64>>,
  scaled_score: f64,
}
// Symmetric whitening is also valid for negative exchangeable correlations.
// Its Gram is expected GEE sensitivity, not the observed score derivative.
fn equations(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  beta: &[f64],
) -> Result<Equations, StatsError> {
  let n = y.len();
  let k = beta.len();
  let t = groups[0].len() as f64;
  let mut mu = Vec::with_capacity(n);
  let mut e = Vec::with_capacity(n);
  let mut noise = Vec::with_capacity(n);
  for (row, y) in x.iter().zip(y) {
    let eta = sum(row.iter().zip(beta).map(|(x, b)| x * b));
    if !eta.is_finite() || !(-40.0..=40.0).contains(&eta) {
      return Err(failed("linear index escaped supported [-40,40] range"));
    }
    let m = eta.exp();
    mu.push(m);
    e.push((y - m) / m.sqrt());
    noise.push((y / m.sqrt() + m.sqrt()).powi(2));
  }
  let ss = sum(e.iter().map(|v| v * v));
  if !ss.is_finite() || ss <= GUARD.powi(2) * sum(noise) {
    return Err(failed("zero or roundoff-dominated Pearson scale"));
  }
  let sg = sum(groups.iter().map(|g| sum(g.iter().map(|&i| e[i])).powi(2)));
  let within = sum(groups.iter().map(|g| {
    let mean = sum(g.iter().map(|&i| e[i])) / t;
    sum(g.iter().map(|&i| (e[i] - mean).powi(2)))
  }));
  // These nonnegative sums avoid cancelling near either SPD boundary.
  let lower = sg / ss;
  let upper = t * within / ((t - 1.0) * ss);
  let rho = (lower - 1.0) / (t - 1.0);
  if !lower.is_finite()
    || !upper.is_finite()
    || lower <= GUARD
    || upper <= GUARD
    || !rho.is_finite()
    || rho >= 1.0
    || 1.0 + (t - 1.0) * rho <= 0.0
  {
    return Err(failed(
      "degenerate or non-positive-definite exchangeable correlation",
    ));
  }
  let mut wx = vec![vec![0.0; k]; n];
  let mut we = vec![0.0; n];
  let mut scores = Vec::with_capacity(groups.len());
  let mut bounds = Vec::with_capacity(groups.len());
  for g in groups {
    let em = sum(g.iter().map(|&i| e[i])) / t;
    let zm = (0..k)
      .map(|j| sum(g.iter().map(|&i| mu[i].sqrt() * x[i][j])) / t)
      .collect::<Vec<_>>();
    for &i in g {
      we[i] = (e[i] - em) / upper.sqrt() + em / lower.sqrt();
      for j in 0..k {
        wx[i][j] = (mu[i].sqrt() * x[i][j] - zm[j]) / upper.sqrt() + zm[j] / lower.sqrt();
      }
    }
    scores.push(
      (0..k)
        .map(|j| sum(g.iter().map(|&i| wx[i][j] * we[i])))
        .collect::<Vec<_>>(),
    );
    bounds.push(
      (0..k)
        .map(|j| sum(g.iter().map(|&i| (wx[i][j] * we[i]).abs())))
        .collect::<Vec<_>>(),
    );
  }
  if wx
    .iter()
    .flatten()
    .chain(&we)
    .chain(scores.iter().flatten())
    .chain(bounds.iter().flatten())
    .any(|v| !v.is_finite())
  {
    return Err(failed("nonfinite whitened equations"));
  }
  let scaled_score = (0..k)
    .map(|j| sum(scores.iter().map(|s| s[j])).abs() / (1.0 + sum(bounds.iter().map(|s| s[j]))))
    .fold(0.0, f64::max);
  Ok(Equations {
    mu,
    rho,
    normalizer: ss / n as f64,
    white_design: wx,
    white_residual: we,
    group_scores: scores,
    group_bounds: bounds,
    scaled_score,
  })
}
fn covariance_scaled_to_raw(covariance: &[Vec<f64>], scales: &[f64]) -> Vec<Vec<f64>> {
  covariance
    .iter()
    .enumerate()
    .map(|(i, row)| {
      row
        .iter()
        .enumerate()
        .map(|(j, v)| v / scales[i] / scales[j])
        .collect()
    })
    .collect()
}
fn group_sandwich(
  eq: &Equations,
  inverse: &[Vec<f64>],
  scales: &[f64],
) -> Result<Vec<Vec<f64>>, StatsError> {
  let k = scales.len();
  let g = eq.group_scores.len();
  let correction = g as f64 / (g - 1) as f64;
  let influences = eq
    .group_scores
    .iter()
    .map(|u| {
      (0..k)
        .map(|i| sum((0..k).map(|j| inverse[i][j] * u[j])) / scales[i])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let bounds = eq
    .group_bounds
    .iter()
    .map(|u| {
      (0..k)
        .map(|i| sum((0..k).map(|j| inverse[i][j].abs() * u[j])) / scales[i])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let mut covariance = vec![vec![0.0; k]; k];
  for i in 0..k {
    for j in 0..=i {
      let value = correction * sum(influences.iter().map(|v| v[i] * v[j]));
      covariance[i][j] = value;
      covariance[j][i] = value;
    }
    let propagation = correction * sum(bounds.iter().map(|v| v[i] * v[i]));
    if !propagation.is_finite()
      || !covariance[i][i].is_finite()
      || covariance[i][i] <= GUARD.powi(2) * propagation
    {
      return Err(failed("zero or roundoff-dominated group sandwich variance"));
    }
  }
  Ok(covariance)
}

/// Fit sample-aligned complete cases with balanced groups T>=2, G>K, an
/// intercept and no weights. Callers handle missingness before this boundary.
/// Both fixed-scale working and G/(G−1) group-sandwich normal inference return.
pub fn fit_poisson_gee(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &PoissonGeeOptions,
) -> Result<PoissonGeeResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let slopes = problem.predictor_names.len();
  let k = slopes + 1;
  if !problem.include_intercept
    || problem.weights.is_some()
    || entity_variable.trim().is_empty()
    || !(1..=100).contains(&options.max_iterations)
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
      .any(|s| s == &problem.intercept_name)
  {
    return Err(invalid(
      "requires intercept, unique K1..32 slopes, group name, no weights and iteration bound1..100",
    ));
  }
  if problem
    .response
    .iter()
    .any(|v| !v.is_finite() || *v < 0.0 || *v > 100000.0 || v.fract() != 0.0)
    || !problem.response.iter().any(|v| *v > 0.0)
  {
    return Err(invalid(
      "requires integer counts0..100000 and positive total count",
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
    || sample.total_observations > 40000
    || sample.total_observations.saturating_mul(k * k) > 20_000_000
    || n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
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
  if g <= k || g > 10000 {
    return Err(invalid(
      "requires at most10000 groups and G>parameter count",
    ));
  }
  let t = groups[0].len();
  if t < 2 || groups.iter().any(|group| group.len() != t) {
    return Err(invalid(
      "requires balanced selected groups of common size T>=2",
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
    .map(|j| {
      let max = x.iter().map(|r| r[j].abs()).fold(0.0, f64::max);
      max * (sum(x.iter().map(|r| (r[j] / max).powi(2))) / n as f64).sqrt()
    })
    .collect::<Vec<_>>();
  if scales.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(invalid("zero or nonfinite RMS design scale"));
  }
  let scaled = x
    .iter()
    .map(|r| {
      r.iter()
        .zip(&scales)
        .map(|(v, s)| v / s)
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  // The independent-row starter rejects cluster metadata. Detach only that
  // metadata on an owned copy; GEE keeps the original sample and all groups.
  let mut start_problem = problem.clone();
  start_problem.sample.as_mut().unwrap().cluster_groups = None;
  let start = fit_poisson(&start_problem, false)?;
  let mut beta = start
    .coefficients
    .iter()
    .zip(&scales)
    .map(|(b, s)| b.value * s)
    .collect::<Vec<_>>();
  for iterations in 0..=options.max_iterations {
    let eq = equations(&scaled, &problem.response, &groups, &beta)?;
    let qr = qr_decompose_with_response(&eq.white_design, &eq.white_residual)?;
    let delta = solve_upper_triangular(&qr.r, &qr.qty)?;
    let step = delta
      .iter()
      .zip(&beta)
      .map(|(d, b)| d.abs() / (1.0 + b.abs()))
      .fold(0.0, f64::max);
    if !step.is_finite() {
      return Err(failed("nonfinite coefficient step"));
    }
    if step <= 1e-12 && eq.scaled_score <= 1e-10 {
      let inverse = xtx_inverse_from_r(&qr.r)?;
      let working = covariance_scaled_to_raw(&inverse, &scales);
      let sandwich = group_sandwich(&eq, &inverse, &scales)?;
      let b = beta
        .iter()
        .zip(&scales)
        .map(|(b, s)| b / s)
        .collect::<Vec<_>>();
      let names = problem.parameter_names();
      let working = crate::gaussian_gee::inference(&names, &b, working)?;
      let sandwich = crate::gaussian_gee::inference(&names, &b, sandwich)?;
      return Ok(PoissonGeeResult {
        outcome: problem.outcome.clone(),
        predictor_names: problem.predictor_names.clone(),
        entity_variable: entity_variable.into(),
        observation_count: n,
        group_count: g,
        group_size: t,
        zero_total_group_count: groups
          .iter()
          .filter(|group| group.iter().all(|&i| problem.response[i] == 0.0))
          .count(),
        zero_count: problem.response.iter().filter(|&&y| y == 0.0).count(),
        count_sum: sum(problem.response.iter().copied()),
        rho: eq.rho,
        pearson_normalizer: eq.normalizer,
        working_scale: 1.0,
        sandwich_correction: g as f64 / (g - 1) as f64,
        working,
        sandwich,
        iterations,
        max_abs_scaled_score: eq.scaled_score,
        max_abs_scaled_newton_step: step,
        fitted_means: eq.mu,
        sample: sample.clone(),
      });
    }
    if iterations == options.max_iterations {
      break;
    }
    for (b, d) in beta.iter_mut().zip(delta) {
      *b += d;
    }
  }
  Err(failed(
    "full Fisher scoring did not converge within iteration bound",
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn near_spd_boundaries_are_rejected_without_clipping() {
    let groups = (0..4)
      .map(|g| (0..5).map(|t| g * 5 + t).collect::<Vec<_>>())
      .collect::<Vec<_>>();
    let upper_x = (0..20)
      .map(|i| vec![1.0, (i % 5 + 1) as f64])
      .collect::<Vec<_>>();
    let upper_y = (0..20).map(|i| (i / 5 + 1) as f64).collect::<Vec<_>>();
    let error = equations(&upper_x, &upper_y, &groups, &[2.0_f64.ln(), 1e-8])
      .err()
      .unwrap();
    assert!(error.to_string().contains("correlation"), "{error}");
    assert!(equations(&upper_x, &upper_y, &groups, &[2.0_f64.ln(), 1e-5]).is_ok());
    let lower_x = (0..20)
      .map(|i| vec![1.0, (i / 5 + 1) as f64])
      .collect::<Vec<_>>();
    let lower_y = (0..20).map(|i| (i % 5 + 1) as f64).collect::<Vec<_>>();
    let error = equations(&lower_x, &lower_y, &groups, &[3.0_f64.ln(), 1e-9])
      .err()
      .unwrap();
    assert!(error.to_string().contains("correlation"), "{error}");
    let interior = equations(&lower_x, &lower_y, &groups, &[3.0_f64.ln(), 1e-5]).unwrap();
    assert!(interior.rho < 0.0);
  }

  #[test]
  fn pearson_roundoff_and_index_escape_are_distinct_failures() {
    let groups = vec![vec![0, 1], vec![2, 3], vec![4, 5]];
    let x = vec![vec![1.0, 0.0]; 6];
    let error = equations(&x, &[1.0; 6], &groups, &[1e-15, 0.0])
      .err()
      .unwrap();
    assert!(error.to_string().contains("Pearson scale"), "{error}");
    for b in [-41.0, 41.0, f64::INFINITY, f64::NAN] {
      let error = equations(&x, &[1.0; 6], &groups, &[b, 0.0]).err().unwrap();
      assert!(error.to_string().contains("linear index"), "{error}");
    }
  }

  #[test]
  fn sandwich_propagation_rejects_positive_but_roundoff_sized_variance() {
    let mut eq = Equations {
      mu: vec![],
      rho: 0.0,
      normalizer: 1.0,
      white_design: vec![],
      white_residual: vec![],
      group_scores: vec![vec![1e-20, 1e-20]; 4],
      group_bounds: vec![vec![1.0, 1.0]; 4],
      scaled_score: 0.0,
    };
    let inverse = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    let error = group_sandwich(&eq, &inverse, &[1.0, 1.0]).unwrap_err();
    assert!(error.to_string().contains("sandwich variance"), "{error}");
    eq.group_scores = vec![vec![1e-4, 1e-4]; 4];
    assert!(group_sandwich(&eq, &inverse, &[1.0, 1.0]).is_ok());
  }
}
