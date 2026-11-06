//! Model-based Poisson fixed effects, conditioning on selected unit totals.
use crate::distributions::chi_square_tail;
use crate::matrix::{choice_information_inverse, lgamma, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, LogitEstimate, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalPoissonOptions {
  pub max_iterations: usize,
}
impl Default for ConditionalPoissonOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Fitted group normalization, conditional on its estimation-sample count total.
/// Anchoring avoids cancellation of a large raw index against a unit intercept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalPoissonEffect {
  pub key: String,
  pub anchor: Vec<f64>,
  pub log_normalizer: f64,
  pub observation_count: usize,
  pub count_sum: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalPoissonResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub joint_finite_observation_count: usize,
  pub candidate_group_count: usize,
  pub group_count: usize,
  pub zero_total_group_count: usize,
  pub zero_total_observation_count: usize,
  pub minimum_group_size: usize,
  pub maximum_group_size: usize,
  pub zero_count: usize,
  pub count_sum: f64,
  pub coefficients: Vec<LogitEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub conditional_log_likelihood: f64,
  pub null_conditional_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: f64,
  pub wald_statistic: f64,
  pub wald_p_value: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_means: Vec<f64>,
  pub effects: Vec<ConditionalPoissonEffect>,
  pub sample: EstimationSample,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("xtpoisson: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("xtpoisson: {message}"))
}
fn max_abs(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}
struct Group {
  key: String,
  rows: Vec<usize>,
  total: f64,
  constant: f64,
}
struct Evaluation {
  means: Vec<f64>,
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
  log_denominators: Vec<f64>,
}
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Group],
  beta: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = beta.len();
  let scale = groups.len() as f64;
  let mut means = vec![0.0; y.len()];
  let mut ll = 0.0;
  let mut score = vec![0.0; k];
  let mut information = vec![vec![0.0; k]; k];
  let mut log_denominators = Vec::with_capacity(groups.len());
  for group in groups {
    let eta = group
      .rows
      .iter()
      .map(|&i| x[i].iter().zip(beta).map(|(a, b)| a * b).sum::<f64>())
      .collect::<Vec<_>>();
    if eta.iter().any(|v| !v.is_finite()) {
      return Err(failed("nonfinite within-group index"));
    }
    let center = eta.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights = eta.iter().map(|v| (v - center).exp()).collect::<Vec<_>>();
    let sum = weights.iter().sum::<f64>();
    let log_sum = sum.ln();
    log_denominators.push(center + log_sum);
    let mean = (0..k)
      .map(|r| {
        group
          .rows
          .iter()
          .zip(&weights)
          .map(|(&i, w)| w / sum * x[i][r])
          .sum::<f64>()
      })
      .collect::<Vec<_>>();
    let mut group_ll = group.constant;
    for ((&i, &index), weight) in group.rows.iter().zip(&eta).zip(weights) {
      let mu = group.total * weight / sum;
      means[i] = mu;
      group_ll += y[i] * ((index - center) - log_sum);
      for r in 0..k {
        // Centering also protects the score from a rounded sum of residuals.
        score[r] += (y[i] - mu) * (x[i][r] - mean[r]) / scale;
        for s in 0..k {
          information[r][s] += mu * (x[i][r] - mean[r]) * (x[i][s] - mean[s]) / scale;
        }
      }
    }
    ll += group_ll;
  }
  if !ll.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .chain(&means)
      .chain(&log_denominators)
      .any(|v| !v.is_finite())
  {
    return Err(failed(
      "nonfinite conditional likelihood, score, information or means",
    ));
  }
  Ok(Evaluation {
    means,
    log_likelihood: ll,
    score,
    information,
    log_denominators,
  })
}

/// Joint finite cases and aligned structural group keys precede this call.
/// Zero-total groups are removed here; singleton positive groups remain but
/// contribute no slope information. No common intercept, weights or sandwich.
pub fn fit_conditional_poisson(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &ConditionalPoissonOptions,
) -> Result<ConditionalPoissonResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let k = problem.predictor_names.len();
  if problem.include_intercept || problem.weights.is_some() || entity_variable.trim().is_empty() {
    return Err(invalid(
      "requires a group name, no common intercept and no weights",
    ));
  }
  if !(1..=32).contains(&k)
    || n > 40000
    || n.saturating_mul(k * k) > 20_000_000
    || problem
      .predictor_names
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != k
  {
    return Err(invalid(
      "requires unique predictors K1..32, rows<=40000 and rows*K^2<=20000000",
    ));
  }
  let sample = problem
    .sample
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  if sample.weights.is_some() {
    return Err(invalid("weights are not supported"));
  }
  let keys = sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  if keys.len() != n
    || sample.total_observations > 40000
    || sample.total_observations.saturating_mul(k * k) > 20_000_000
    || n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
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
      "invalid original sample mapping or group-key alignment",
    ));
  }
  if problem
    .response
    .iter()
    .any(|v| *v < 0.0 || *v > 100000.0 || v.fract() != 0.0)
  {
    return Err(invalid("counts must be exact integers in 0..100000"));
  }
  let mut candidates: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
  for (i, key) in keys.iter().enumerate() {
    candidates.entry(key).or_default().push(i);
  }
  if candidates.len() > 10000 {
    return Err(invalid("requires at most10000 candidate groups"));
  }
  let candidate_group_count = candidates.len();
  let mut retained = vec![false; n];
  let mut positive = Vec::new();
  for (key, rows) in candidates {
    let total = rows.iter().map(|&i| problem.response[i]).sum::<f64>();
    if total > 0.0 {
      for &i in &rows {
        retained[i] = true;
      }
      positive.push((key, rows, total));
    }
  }
  let g = positive.len();
  if g <= k {
    return Err(invalid("requires positive-total groups G>K"));
  }
  let positions = retained
    .iter()
    .enumerate()
    .filter_map(|(i, &keep)| keep.then_some(i))
    .collect::<Vec<_>>();
  let mut mapping = vec![usize::MAX; n];
  for (i, &original) in positions.iter().enumerate() {
    mapping[original] = i;
  }
  let y = positions
    .iter()
    .map(|&i| problem.response[i])
    .collect::<Vec<_>>();
  let mut x = positions
    .iter()
    .map(|&i| problem.design_matrix[i].clone())
    .collect::<Vec<_>>();
  let mut anchors = Vec::with_capacity(g);
  let mut groups = Vec::with_capacity(g);
  for (key, original, total) in positive {
    let rows = original.iter().map(|&i| mapping[i]).collect::<Vec<_>>();
    let anchor = x[rows[0]].clone();
    for &i in &rows {
      for r in 0..k {
        x[i][r] -= anchor[r];
      }
    }
    let constant = lgamma(total + 1.0) - rows.iter().map(|&i| lgamma(y[i] + 1.0)).sum::<f64>();
    anchors.push(anchor);
    groups.push(Group {
      key: key.into(),
      rows,
      total,
      constant,
    });
  }
  if x.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite within-group predictor differences"));
  }
  let kept = y.len();
  let mut scales = Vec::with_capacity(k);
  for r in 0..k {
    let max = x.iter().map(|row| row[r].abs()).fold(0.0, f64::max);
    if max == 0.0 {
      return Err(invalid(&format!(
        "no within-group variation: {}",
        problem.predictor_names[r]
      )));
    }
    let scale =
      max * (x.iter().map(|row| (row[r] / max).powi(2)).sum::<f64>() / kept as f64).sqrt();
    if !scale.is_finite() || scale <= 0.0 {
      return Err(failed("undefined within-group predictor scale"));
    }
    scales.push(scale);
    for row in &mut x {
      row[r] /= scale;
    }
  }
  qr_decompose_with_response(&x, &vec![0.0; kept])?;
  let mut beta = vec![0.0; k];
  let null = evaluate(&x, &y, &groups, &beta)?.log_likelihood;
  let mut converged = None;
  for iteration in 0..options.max_iterations {
    let current = evaluate(&x, &y, &groups, &beta)?;
    let inverse =
      choice_information_inverse(&current.information).map_err(|e| failed(&e.to_string()))?;
    let step = inverse
      .iter()
      .map(|row| {
        row
          .iter()
          .zip(&current.score)
          .map(|(a, b)| a * b)
          .sum::<f64>()
      })
      .collect::<Vec<_>>();
    if step.iter().any(|v| !v.is_finite()) {
      return Err(failed("nonfinite Newton step"));
    }
    let score_bound = max_abs(&current.score);
    let step_bound = max_abs(&step);
    if score_bound <= TOLERANCE && step_bound <= TOLERANCE {
      converged = Some((current, inverse, iteration + 1, score_bound, step_bound));
      break;
    }
    let mut damping = 1.0;
    let mut accepted = None;
    for _ in 0..40 {
      let trial = beta
        .iter()
        .zip(&step)
        .map(|(b, s)| b + damping * s)
        .collect::<Vec<_>>();
      if let Ok(value) = evaluate(&x, &y, &groups, &trial)
        && value.log_likelihood
          >= current.log_likelihood - 1e-12 * (1.0 + current.log_likelihood.abs())
      {
        accepted = Some(trial);
        break;
      }
      damping *= 0.5;
    }
    beta = accepted.ok_or_else(|| failed("Newton line search did not converge"))?;
  }
  let (current, inverse, iterations, max_abs_scaled_score, max_abs_scaled_newton_step) = converged
    .ok_or_else(|| failed("iteration limit reached; possible separation or nonidentification"))?;
  let covariance = inverse
    .iter()
    .enumerate()
    .map(|(r, row)| {
      row
        .iter()
        .enumerate()
        .map(|(s, v)| v / g as f64 / scales[r] / scales[s])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let coefficients = (0..k)
    .map(|r| {
      let value = beta[r] / scales[r];
      let standard_error = covariance[r][r].sqrt();
      let statistic = value / standard_error;
      let p_value = chi_square_tail(statistic * statistic, 1);
      let confidence_lower = value - NORMAL_95 * standard_error;
      let confidence_upper = value + NORMAL_95 * standard_error;
      if standard_error <= 0.0
        || [
          value,
          standard_error,
          statistic,
          p_value,
          confidence_lower,
          confidence_upper,
        ]
        .iter()
        .any(|v| !v.is_finite())
      {
        return Err(failed("nonfinite normal inference or nonpositive variance"));
      }
      Ok(LogitEstimate {
        name: problem.predictor_names[r].clone(),
        value,
        standard_error,
        statistic,
        p_value,
        confidence_lower,
        confidence_upper,
      })
    })
    .collect::<Result<Vec<_>, StatsError>>()?;
  let wald_statistic = (0..k)
    .map(|r| {
      (0..k)
        .map(|s| beta[r] * current.information[r][s] * beta[s] * g as f64)
        .sum::<f64>()
    })
    .sum::<f64>();
  let likelihood_ratio = 2.0 * (current.log_likelihood - null);
  if !wald_statistic.is_finite()
    || wald_statistic < 0.0
    || !likelihood_ratio.is_finite()
    || likelihood_ratio < -1e-8
  {
    return Err(failed("invalid joint test statistic"));
  }
  let likelihood_ratio = likelihood_ratio.max(0.0);
  let effects = groups
    .iter()
    .zip(anchors)
    .zip(&current.log_denominators)
    .map(|((group, anchor), denom)| ConditionalPoissonEffect {
      key: group.key.clone(),
      anchor,
      log_normalizer: group.total.ln() - denom,
      observation_count: group.rows.len(),
      count_sum: group.total,
    })
    .collect();
  let retained_indices = positions
    .iter()
    .map(|&i| sample.retained_indices[i])
    .collect::<Vec<_>>();
  let final_sample = EstimationSample::new(
    retained_indices,
    sample.total_observations,
    sample.total_observations - kept,
    None,
    Some(positions.iter().map(|&i| keys[i].clone()).collect()),
  )?;
  Ok(ConditionalPoissonResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    entity_variable: entity_variable.into(),
    observation_count: kept,
    joint_finite_observation_count: n,
    candidate_group_count,
    group_count: g,
    zero_total_group_count: candidate_group_count - g,
    zero_total_observation_count: n - kept,
    minimum_group_size: groups.iter().map(|g| g.rows.len()).min().unwrap(),
    maximum_group_size: groups.iter().map(|g| g.rows.len()).max().unwrap(),
    zero_count: y.iter().filter(|&&v| v == 0.0).count(),
    count_sum: y.iter().sum(),
    coefficients,
    covariance,
    conditional_log_likelihood: current.log_likelihood,
    null_conditional_log_likelihood: null,
    likelihood_ratio,
    likelihood_ratio_df: k,
    likelihood_ratio_p_value: chi_square_tail(likelihood_ratio, k),
    wald_statistic,
    wald_p_value: chi_square_tail(wald_statistic, k),
    iterations,
    max_abs_scaled_score,
    max_abs_scaled_newton_step,
    fitted_means: current.means,
    effects,
    sample: final_sample,
  })
}
