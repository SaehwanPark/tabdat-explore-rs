//! Single-choice McFadden logit on complete, balanced, explicitly keyed sets.
//! A choice set is one independent decision, not J independent Bernoulli rows.

use crate::distributions::chi_square_tail;
use crate::matrix::{choice_information_inverse, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, LogitEstimate, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Rows of `design` retain their input order. Each row belongs to exactly one
/// case; alternative codes identify the same available alternatives in every
/// case. The common intercept is not a conditional-likelihood parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalLogitProblem {
  pub design: EstimationProblem,
  pub group_variable: String,
  pub alternative_variable: String,
  pub case_rows: Vec<Vec<usize>>,
  pub alternative_codes: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalLogitOptions {
  pub max_iterations: usize,
}
impl Default for ConditionalLogitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalLogitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub group_variable: String,
  pub alternative_variable: String,
  pub alternative_levels: Vec<i64>,
  pub chosen_counts: Vec<usize>,
  /// Number of long-format alternative rows; inference is based on choice sets.
  pub observation_count: usize,
  pub choice_set_count: usize,
  pub choice_set_size: usize,
  pub candidate_choice_set_count: usize,
  pub excluded_choice_set_count: usize,
  pub coefficients: Vec<LogitEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: f64,
  pub mcfadden_r_squared: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  /// Same retained long-row order as the problem, not grouped or sorted output.
  pub fitted_probabilities: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("clogit: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("clogit: {message}"))
}
fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

pub(crate) fn alternative_order(codes: &[i64]) -> Result<Vec<usize>, StatsError> {
  if !(2..=16).contains(&codes.len())
    || codes
      .iter()
      .any(|v| v.unsigned_abs() > 9_007_199_254_740_991)
  {
    return Err(invalid(
      "requires 2..16 exact alternative codes within +/- (2^53-1)",
    ));
  }
  let mut order = (0..codes.len()).collect::<Vec<_>>();
  order.sort_by_key(|&i| codes[i]);
  if order.windows(2).any(|v| codes[v[0]] == codes[v[1]]) {
    return Err(invalid("duplicate alternative within a choice set"));
  }
  Ok(order)
}

pub(crate) struct Layout {
  pub(crate) cases: Vec<Vec<usize>>,
  pub(crate) chosen: Vec<usize>,
  pub(crate) levels: Vec<i64>,
  pub(crate) candidate_cases: usize,
}
pub(crate) fn validate(problem: &ConditionalLogitProblem) -> Result<Layout, StatsError> {
  let d = &problem.design;
  d.validate()?;
  let n = d.response.len();
  let k = d.predictor_names.len();
  let g = problem.case_rows.len();
  if d.include_intercept {
    return Err(invalid(
      "a common intercept cancels from the conditional likelihood",
    ));
  }
  if d.weights.is_some() || d.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("weights are not supported"));
  }
  if problem.group_variable.trim().is_empty() || problem.alternative_variable.trim().is_empty() {
    return Err(invalid(
      "group and alternative variable names must not be empty",
    ));
  }
  if d.predictor_names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid("predictor names must be unique"));
  }
  if g <= k
    || g > 10_000
    || n > 40_000
    || k > 32
    || n.saturating_mul(k.saturating_mul(k)) > 20_000_000
  {
    return Err(invalid(
      "requires independent choice sets G>K, G<=10000, rows<=40000, K<=32 and rows*K^2<=20000000",
    ));
  }
  if problem.alternative_codes.len() != n {
    return Err(invalid(
      "one alternative code is required for every retained row",
    ));
  }
  let mut seen = vec![false; n];
  let mut cases = Vec::with_capacity(g);
  let mut chosen = Vec::with_capacity(g);
  let mut levels = Vec::new();
  for input in &problem.case_rows {
    if input.iter().any(|&i| i >= n) {
      return Err(invalid("choice-set row index is out of bounds"));
    }
    let codes = input
      .iter()
      .map(|&i| problem.alternative_codes[i])
      .collect::<Vec<_>>();
    let order = alternative_order(&codes)?;
    let case = order.iter().map(|&i| input[i]).collect::<Vec<_>>();
    let current_levels = order.iter().map(|&i| codes[i]).collect::<Vec<_>>();
    if levels.is_empty() {
      levels = current_levels;
    } else if current_levels != levels {
      return Err(invalid(
        "every choice set must have the identical complete alternative universe",
      ));
    }
    let mut selected = None;
    for &i in &case {
      if seen[i] {
        return Err(invalid("a row belongs to more than one choice set"));
      }
      seen[i] = true;
      match d.response[i] {
        0.0 => {}
        1.0 if selected.is_none() => selected = Some(i),
        1.0 => {
          return Err(invalid(
            "exactly one chosen row is required per retained choice set",
          ));
        }
        _ => {
          return Err(invalid(
            "chosen outcome must be exactly 0 or 1 on every retained choice set",
          ));
        }
      }
    }
    chosen.push(
      selected
        .ok_or_else(|| invalid("exactly one chosen row is required per retained choice set"))?,
    );
    cases.push(case);
  }
  if seen.iter().any(|v| !*v) {
    return Err(invalid("choice sets must partition every retained row"));
  }
  let j = levels.len();
  let total = d.sample.as_ref().map_or(n, |s| s.total_observations);
  if total < n || !total.is_multiple_of(j) {
    return Err(invalid(
      "candidate row count must represent all retained and excluded complete balanced choice sets",
    ));
  }
  Ok(Layout {
    cases,
    chosen,
    levels,
    candidate_cases: total / j,
  })
}

fn probabilities(indices: &[f64]) -> Result<(Vec<f64>, f64, f64), StatsError> {
  if indices.iter().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite within-choice-set linear index"));
  }
  let center = indices.iter().copied().fold(f64::NEG_INFINITY, f64::max);
  let mut weights = indices
    .iter()
    .map(|v| (v - center).exp())
    .collect::<Vec<_>>();
  let sum = weights.iter().sum::<f64>();
  for p in &mut weights {
    *p /= sum;
  }
  Ok((weights, center, sum.ln()))
}

struct Evaluation {
  probabilities: Vec<f64>,
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
}
fn evaluate(x: &[Vec<f64>], layout: &Layout, beta: &[f64]) -> Result<Evaluation, StatsError> {
  let k = beta.len();
  let g = layout.cases.len() as f64;
  let mut fitted = vec![0.0; x.len()];
  let mut ll = 0.0;
  let mut score = vec![0.0; k];
  let mut information = vec![vec![0.0; k]; k];
  for (case, &selected) in layout.cases.iter().zip(&layout.chosen) {
    let eta = case
      .iter()
      .map(|&i| x[i].iter().zip(beta).map(|(v, b)| v * b).sum::<f64>())
      .collect::<Vec<_>>();
    let (p, center, log_sum) = probabilities(&eta)?;
    let chosen_position = case.iter().position(|&i| i == selected).unwrap();
    ll += (eta[chosen_position] - center) - log_sum;
    let mean = (0..k)
      .map(|r| case.iter().zip(&p).map(|(&i, p)| p * x[i][r]).sum::<f64>())
      .collect::<Vec<_>>();
    for (&i, &probability) in case.iter().zip(&p) {
      fitted[i] = probability;
      for r in 0..k {
        // A sum over nonchosen tails avoids x_chosen-E[x] cancellation when
        // the selected probability rounds to one near separation.
        score[r] += probability * (x[selected][r] - x[i][r]) / g;
        for s in 0..k {
          information[r][s] += probability * (x[i][r] - mean[r]) * (x[i][s] - mean[s]) / g;
        }
      }
    }
  }
  if !ll.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed("nonfinite likelihood, score or information"));
  }
  Ok(Evaluation {
    probabilities: fitted,
    log_likelihood: ll,
    score,
    information,
  })
}

fn inference(name: String, value: f64, variance: f64) -> Result<LogitEstimate, StatsError> {
  let standard_error = variance.sqrt();
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
    name,
    value,
    standard_error,
    statistic,
    p_value,
    confidence_lower,
    confidence_upper,
  })
}

/// Model-based conditional MLE, with per-decision rather than per-row score and
/// information scaling. Final covariance is the inverse information sum over G
/// independent decisions, without N-K or G-K correction.
pub fn fit_conditional_logit(
  problem: &ConditionalLogitProblem,
  options: &ConditionalLogitOptions,
) -> Result<ConditionalLogitResult, StatsError> {
  let layout = validate(problem)?;
  let d = &problem.design;
  let n = d.response.len();
  let k = d.predictor_names.len();
  let g = layout.cases.len();
  let mut x = d.design_matrix.clone();
  for case in &layout.cases {
    let baseline = x[case[0]].clone();
    for &i in case {
      for (r, &base) in baseline.iter().enumerate() {
        x[i][r] -= base;
      }
    }
  }
  if x.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite within-choice-set predictor differences"));
  }
  let mut scales = Vec::with_capacity(k);
  for r in 0..k {
    let max = x.iter().map(|row| row[r].abs()).fold(0.0, f64::max);
    if max == 0.0 {
      return Err(invalid(&format!(
        "no within-choice-set variation: {}",
        d.predictor_names[r]
      )));
    }
    let scale = max * (x.iter().map(|row| (row[r] / max).powi(2)).sum::<f64>() / n as f64).sqrt();
    if !scale.is_finite() || scale <= 0.0 {
      return Err(failed("undefined within-choice-set predictor scale"));
    }
    scales.push(scale);
    for row in &mut x {
      row[r] /= scale;
    }
  }
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  let mut beta = vec![0.0; k];
  let mut converged = None;
  for iteration in 0..options.max_iterations {
    let current = evaluate(&x, &layout, &beta)?;
    let inverse =
      choice_information_inverse(&current.information).map_err(|e| failed(&e.to_string()))?;
    let step = inverse
      .iter()
      .map(|row| row.iter().zip(&current.score).map(|(a, b)| a * b).sum())
      .collect::<Vec<f64>>();
    if step.iter().any(|v| !v.is_finite()) {
      return Err(failed("nonfinite Newton step"));
    }
    let score_bound = maximum_absolute(&current.score);
    let step_bound = maximum_absolute(&step);
    if score_bound <= TOLERANCE && step_bound <= TOLERANCE {
      converged = Some((current, inverse, iteration + 1, score_bound, step_bound));
      break;
    }
    let mut damping = 1.0;
    let mut accepted = None;
    for _ in 0..40 {
      let candidate = beta
        .iter()
        .zip(&step)
        .map(|(b, s)| b + damping * s)
        .collect::<Vec<_>>();
      if let Ok(trial) = evaluate(&x, &layout, &candidate)
        && trial.log_likelihood
          >= current.log_likelihood - 1e-12 * (1.0 + current.log_likelihood.abs())
      {
        accepted = Some(candidate);
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
        .map(|(s, v)| (v / g as f64) / scales[r] / scales[s])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let coefficients = (0..k)
    .map(|r| {
      inference(
        d.predictor_names[r].clone(),
        beta[r] / scales[r],
        covariance[r][r],
      )
    })
    .collect::<Result<Vec<_>, _>>()?;
  let mut chosen_counts = vec![0; layout.levels.len()];
  for &i in &layout.chosen {
    chosen_counts[layout
      .levels
      .binary_search(&problem.alternative_codes[i])
      .unwrap()] += 1;
  }
  let null_log_likelihood = -(g as f64) * (layout.levels.len() as f64).ln();
  let likelihood_ratio = (2.0 * (current.log_likelihood - null_log_likelihood)).max(0.0);
  Ok(ConditionalLogitResult {
    outcome: d.outcome.clone(),
    predictor_names: d.predictor_names.clone(),
    group_variable: problem.group_variable.clone(),
    alternative_variable: problem.alternative_variable.clone(),
    choice_set_size: layout.levels.len(),
    alternative_levels: layout.levels,
    chosen_counts,
    observation_count: n,
    choice_set_count: g,
    candidate_choice_set_count: layout.candidate_cases,
    excluded_choice_set_count: layout.candidate_cases - g,
    coefficients,
    covariance,
    log_likelihood: current.log_likelihood,
    null_log_likelihood,
    likelihood_ratio,
    likelihood_ratio_df: k,
    likelihood_ratio_p_value: chi_square_tail(likelihood_ratio, k),
    mcfadden_r_squared: 1.0 - current.log_likelihood / null_log_likelihood,
    iterations,
    max_abs_scaled_score,
    max_abs_scaled_newton_step,
    fitted_probabilities: current.probabilities,
    sample: d.sample.clone(),
  })
}

/// Probabilities for one complete current choice set, returned in input-row
/// order. No outcome is required. Missing predictors are a whole-set policy at
/// the execution boundary, never a smaller-denominator renormalization here.
pub fn conditional_logit_predict_set(
  result: &ConditionalLogitResult,
  rows: &[Vec<f64>],
  codes: &[i64],
) -> Result<Vec<f64>, StatsError> {
  let k = result.predictor_names.len();
  let order = alternative_order(codes)?;
  if order.iter().map(|&i| codes[i]).collect::<Vec<_>>() != result.alternative_levels
    || rows.len() != codes.len()
  {
    return Err(invalid(
      "prediction requires the complete retained alternative universe",
    ));
  }
  if k == 0
    || result.coefficients.len() != k
    || result.coefficients.iter().any(|c| !c.value.is_finite())
    || rows
      .iter()
      .any(|row| row.len() != k || row.iter().any(|v| !v.is_finite()))
  {
    return Err(invalid(
      "prediction requires finite current predictors in retained order",
    ));
  }
  let baseline = &rows[order[0]];
  let eta = rows
    .iter()
    .map(|row| {
      row
        .iter()
        .zip(baseline)
        .zip(&result.coefficients)
        .map(|((x, base), b)| (x - base) * b.value)
        .sum()
    })
    .collect::<Vec<f64>>();
  Ok(probabilities(&eta)?.0)
}
