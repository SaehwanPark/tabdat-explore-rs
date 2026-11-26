//! Bounded IID Gaussian sample-selection maximum likelihood.

#![allow(clippy::needless_range_loop)]

use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf};
use crate::matrix::choice_information_inverse;
use crate::{
  EstimationProblem, EstimationSample, HeckmanPredictionKind, HeckmanTwoStepResult, LogitEstimate,
  ProbitOptions, StatsError, fit_heckman_two_step, fit_least_squares,
};
use serde::{Deserialize, Serialize};

pub const MAX_HECKMAN_ML_PARAMETERS: usize = 66;
pub const MAX_HECKMAN_ML_WORK: usize = 200_000_000;
const NORMAL_95: f64 = 1.959963984540054;
const LOG_SQRT_2PI: f64 = 0.9189385332046727;
const SCORE_TOLERANCE: f64 = 1e-8;
const MAX_ML_EVALUATIONS: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeckmanMlOptions {
  pub max_iterations: usize,
}

impl Default for HeckmanMlOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeckmanMlResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub selection_outcome: String,
  pub selection_predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub selection_include_intercept: bool,
  pub outcome_coefficients: Vec<LogitEstimate>,
  pub selection_coefficients: Vec<LogitEstimate>,
  /// The ancillary parameter rows are `lnsigma` and `athrho`.
  pub ancillary_coefficients: Vec<LogitEstimate>,
  /// Full observed-information covariance ordered as outcome, selection,
  /// `lnsigma`, `athrho`.
  pub covariance: Vec<Vec<f64>>,
  pub covariance_parameter_names: Vec<String>,
  pub observation_count: usize,
  pub selected_count: usize,
  pub unselected_count: usize,
  pub selection_sample: Option<EstimationSample>,
  pub outcome_sample: Option<EstimationSample>,
  /// Indices into the retained selection sample, in outcome-stage order.
  pub selected_selection_indices: Vec<usize>,
  pub log_likelihood: f64,
  pub independent_log_likelihood: f64,
  pub independence_lr_chi_square: f64,
  pub independence_lr_p_value: f64,
  pub sigma: f64,
  pub sigma_standard_error: f64,
  pub sigma_confidence_lower: f64,
  pub sigma_confidence_upper: f64,
  pub rho: f64,
  pub rho_standard_error: f64,
  pub rho_confidence_lower: f64,
  pub rho_confidence_upper: f64,
  pub lambda: f64,
  pub lambda_standard_error: f64,
  pub lambda_confidence_lower: f64,
  pub lambda_confidence_upper: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_step: f64,
  pub excluded_selection_predictors: Vec<String>,
}

struct Evaluation {
  log_likelihood: f64,
  score: Vec<f64>,
}

struct Solution {
  point: Vec<f64>,
  evaluation: Evaluation,
  iterations: usize,
  score_bound: f64,
  step_bound: f64,
}

struct EvaluationBudget {
  observations: usize,
  parameters: usize,
  calls: usize,
  work: usize,
}

impl EvaluationBudget {
  fn charge(&mut self) -> Result<(), StatsError> {
    self.calls = self
      .calls
      .checked_add(1)
      .ok_or_else(|| failed("likelihood evaluation count overflowed"))?;
    self.work = self
      .observations
      .checked_mul(self.parameters)
      .and_then(|unit| unit.checked_mul(self.calls))
      .ok_or_else(|| failed("expanded work bound overflowed"))?;
    if self.calls > MAX_ML_EVALUATIONS || self.work > MAX_HECKMAN_ML_WORK {
      return Err(invalid(
        "joint likelihood exceeded the bounded evaluation work",
      ));
    }
    Ok(())
  }
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("heckman ML: {message}"))
}

fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("heckman ML: {message}"))
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
  a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn max_abs(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

fn norm(values: &[f64]) -> f64 {
  values.iter().fold(0.0_f64, |value, x| value.hypot(*x))
}

fn log_normal_cdf(value: f64) -> Result<f64, StatsError> {
  if !value.is_finite() {
    return Err(failed("normal-CDF argument is not finite"));
  }
  if value >= -8.0 {
    let probability = normal_cdf(value);
    if probability > 0.0 && probability.is_finite() {
      return Ok(probability.ln());
    }
    return Err(failed("normal CDF is outside the represented range"));
  }
  // Laplace's continued fraction evaluates Phi(-z)/phi(z) without first
  // underflowing Phi(-z). The tail is used by both the likelihood and score.
  let z = -value;
  let mut fraction = 0.0;
  for k in (1..=120).rev() {
    fraction = k as f64 / (z + fraction);
  }
  let mills = 1.0 / (z + fraction);
  let result = -0.5 * z * z - LOG_SQRT_2PI + mills.ln();
  if result.is_finite() {
    Ok(result)
  } else {
    Err(failed("log normal CDF is outside the represented range"))
  }
}

fn lower_inverse_mills(value: f64) -> Result<f64, StatsError> {
  let log_mills = -0.5 * value * value - LOG_SQRT_2PI - log_normal_cdf(value)?;
  let mills = log_mills.exp();
  if mills.is_finite() && mills > 0.0 {
    Ok(mills)
  } else {
    Err(failed(
      "inverse Mills ratio is outside the represented range",
    ))
  }
}

fn full_design(problem: &EstimationProblem) -> Vec<Vec<f64>> {
  problem
    .design_matrix
    .iter()
    .map(|row| {
      let mut values = Vec::with_capacity(problem.parameter_count());
      if problem.include_intercept {
        values.push(1.0);
      }
      values.extend_from_slice(row);
      values
    })
    .collect()
}

fn column_rms(matrix: &[Vec<f64>], column: usize) -> Result<f64, StatsError> {
  let rms = matrix
    .iter()
    .fold(0.0_f64, |value, row| value.hypot(row[column]))
    / (matrix.len() as f64).sqrt();
  if rms.is_finite() && rms > 0.0 {
    Ok(rms)
  } else {
    Err(failed("predictor norm is not finite and positive"))
  }
}

fn model_score(
  point: &[f64],
  parameter_scales: &[f64],
  selection: &EstimationProblem,
  outcome: &EstimationProblem,
  selected_rows: &[usize],
  selection_design: &[Vec<f64>],
  outcome_design: &[Vec<f64>],
) -> Result<Evaluation, StatsError> {
  let p = outcome.parameter_count();
  let q = selection.parameter_count();
  let expected = p + q + 2;
  if point.len() != expected || parameter_scales.len() != expected {
    return Err(StatsError::DimensionMismatch(
      "Heckman ML parameter vector has the wrong length".into(),
    ));
  }
  let theta: Vec<f64> = point
    .iter()
    .zip(parameter_scales)
    .map(|(value, scale)| value * scale)
    .collect();
  let beta = &theta[..p];
  let gamma = &theta[p..p + q];
  let log_sigma = theta[p + q];
  let atanh_rho = theta[p + q + 1];
  if !log_sigma.is_finite() || !(-8.0..=8.0).contains(&atanh_rho) {
    return Err(failed("scale/correlation parameter left its regular range"));
  }
  let sigma = log_sigma.exp();
  let rho = atanh_rho.tanh();
  let complement = (1.0 - rho * rho).sqrt();
  if !sigma.is_finite() || sigma <= 0.0 || !complement.is_finite() || complement <= 0.0 {
    return Err(failed("scale/correlation parameter is not interior"));
  }

  let mut log_likelihood = 0.0;
  let mut score = vec![0.0; expected];
  let mut outcome_row = 0;
  for i in 0..selection.observation_count() {
    let z_index = dot(&selection_design[i], gamma);
    if !z_index.is_finite() {
      return Err(failed("selection index overflowed"));
    }
    if selection.response[i] == 0.0 {
      let tail_index = -z_index;
      log_likelihood += log_normal_cdf(tail_index)?;
      let mills = lower_inverse_mills(tail_index)?;
      for j in 0..q {
        score[p + j] -= mills * selection_design[i][j];
      }
      continue;
    }
    if selection.response[i] != 1.0 || selected_rows.get(outcome_row) != Some(&i) {
      return Err(invalid("outcome rows must be the ordered selected subset"));
    }
    let x = &outcome_design[outcome_row];
    let residual = (outcome.response[outcome_row] - dot(x, beta)) / sigma;
    let conditional_index = (z_index + rho * residual) / complement;
    if !residual.is_finite() || !conditional_index.is_finite() {
      return Err(failed("conditional likelihood index overflowed"));
    }
    log_likelihood +=
      -log_sigma - 0.5 * residual * residual - LOG_SQRT_2PI + log_normal_cdf(conditional_index)?;
    let mills = lower_inverse_mills(conditional_index)?;
    let derivative_residual = -residual + mills * rho / complement;
    for j in 0..p {
      score[j] -= derivative_residual * x[j] / sigma;
    }
    for j in 0..q {
      score[p + j] += mills * selection_design[i][j] / complement;
    }
    score[p + q] += -1.0 - derivative_residual * residual;
    score[p + q + 1] += mills * (residual + rho * z_index) / complement;
    outcome_row += 1;
  }
  if outcome_row != outcome.observation_count() {
    return Err(invalid("outcome rows must match selected units"));
  }
  if !log_likelihood.is_finite() || score.iter().any(|value| !value.is_finite()) {
    return Err(failed("joint likelihood or score is not finite"));
  }
  Ok(Evaluation {
    log_likelihood,
    score,
  })
}

fn scaled_gradient(evaluation: &Evaluation, scales: &[f64], n: usize) -> Vec<f64> {
  evaluation
    .score
    .iter()
    .zip(scales)
    .map(|(score, scale)| -score * scale / n as f64)
    .collect()
}

fn minimize_bfgs(
  initial: Vec<f64>,
  scales: &[f64],
  n: usize,
  max_iterations: usize,
  mut evaluate: impl FnMut(&[f64]) -> Result<Evaluation, StatsError>,
) -> Result<Solution, StatsError> {
  let dim = initial.len();
  let mut point = initial;
  let mut evaluation = evaluate(&point)?;
  let mut gradient = scaled_gradient(&evaluation, scales, n);
  let mut inverse = vec![vec![0.0; dim]; dim];
  for i in 0..dim {
    inverse[i][i] = 1.0;
  }
  let mut step_bound: f64 = 0.0;

  for iteration in 1..=max_iterations {
    let score_bound = max_abs(&gradient);
    if score_bound <= SCORE_TOLERANCE && step_bound <= SCORE_TOLERANCE {
      return Ok(Solution {
        point,
        evaluation,
        iterations: iteration,
        score_bound,
        step_bound,
      });
    }
    let mut direction: Vec<f64> = inverse.iter().map(|row| -dot(row, &gradient)).collect();
    let mut slope = dot(&gradient, &direction);
    if !slope.is_finite() || slope >= 0.0 {
      inverse.fill(vec![0.0; dim]);
      for i in 0..dim {
        inverse[i][i] = 1.0;
      }
      direction = gradient.iter().map(|value| -value).collect();
      slope = -dot(&gradient, &gradient);
    }
    let current_objective = -evaluation.log_likelihood / n as f64;
    let mut alpha = 1.0;
    let mut accepted: Option<(Vec<f64>, Evaluation)> = None;
    for _ in 0..50 {
      let candidate: Vec<f64> = point
        .iter()
        .zip(&direction)
        .map(|(value, step)| value + alpha * step)
        .collect();
      if let Ok(next) = evaluate(&candidate) {
        let objective = -next.log_likelihood / n as f64;
        if objective <= current_objective + 1e-4 * alpha * slope {
          accepted = Some((candidate, next));
          break;
        }
      }
      alpha *= 0.5;
    }
    let (candidate, next_evaluation) = accepted.ok_or_else(|| failed("BFGS line search failed"))?;
    let next_gradient = scaled_gradient(&next_evaluation, scales, n);
    let displacement: Vec<f64> = candidate.iter().zip(&point).map(|(a, b)| a - b).collect();
    let gradient_change: Vec<f64> = next_gradient
      .iter()
      .zip(&gradient)
      .map(|(a, b)| a - b)
      .collect();
    let curvature = dot(&displacement, &gradient_change);
    let curvature_scale = norm(&displacement) * norm(&gradient_change);
    if curvature.is_finite() && curvature > 1e-12 * curvature_scale {
      let inverse_gradient_change: Vec<f64> = inverse
        .iter()
        .map(|row| dot(row, &gradient_change))
        .collect();
      let gradient_change_inverse_gradient = dot(&gradient_change, &inverse_gradient_change);
      let multiplier = (1.0 + gradient_change_inverse_gradient / curvature) / curvature;
      for i in 0..dim {
        for j in 0..dim {
          inverse[i][j] += multiplier * displacement[i] * displacement[j]
            - (displacement[i] * inverse_gradient_change[j]
              + inverse_gradient_change[i] * displacement[j])
              / curvature;
        }
      }
    } else {
      inverse.fill(vec![0.0; dim]);
      for i in 0..dim {
        inverse[i][i] = 1.0;
      }
    }
    step_bound = displacement
      .iter()
      .zip(&point)
      .map(|(step, value)| step.abs() / (1.0 + value.abs()))
      .fold(0.0, f64::max);
    point = candidate;
    evaluation = next_evaluation;
    gradient = next_gradient;
  }
  Err(failed(
    "iteration limit reached without strict joint score/step convergence",
  ))
}

fn observed_covariance(
  point: &[f64],
  scales: &[f64],
  mut evaluate: impl FnMut(&[f64]) -> Result<Evaluation, StatsError>,
) -> Result<Vec<Vec<f64>>, StatsError> {
  let dim = point.len();
  let h = 2e-5;
  let mut information = vec![vec![0.0; dim]; dim];
  for j in 0..dim {
    let delta = h * (1.0 + point[j].abs());
    let mut plus_point = point.to_vec();
    let mut minus_point = point.to_vec();
    plus_point[j] += delta;
    minus_point[j] -= delta;
    let plus = evaluate(&plus_point)?;
    let minus = evaluate(&minus_point)?;
    for i in 0..dim {
      let plus_score = plus.score[i] * scales[i];
      let minus_score = minus.score[i] * scales[i];
      information[i][j] = -(plus_score - minus_score) / (2.0 * delta);
    }
  }
  for i in 0..dim {
    for j in 0..i {
      let average = 0.5 * (information[i][j] + information[j][i]);
      information[i][j] = average;
      information[j][i] = average;
    }
  }
  let inverse_scaled = choice_information_inverse(&information)
    .map_err(|_| failed("joint observed information is not positive definite"))?;
  let covariance: Vec<Vec<f64>> = inverse_scaled
    .iter()
    .enumerate()
    .map(|(i, row)| {
      row
        .iter()
        .enumerate()
        .map(|(j, value)| value * scales[i] * scales[j])
        .collect()
    })
    .collect();
  if covariance.iter().flatten().any(|value| !value.is_finite()) {
    return Err(failed("joint observed covariance is not finite"));
  }
  Ok(covariance)
}

fn normal_estimate(name: String, value: f64, variance: f64) -> Result<LogitEstimate, StatsError> {
  let standard_error = variance.sqrt();
  if !standard_error.is_finite() || standard_error <= 0.0 {
    return Err(failed(
      "joint covariance has nonpositive parameter variance",
    ));
  }
  let statistic = value / standard_error;
  let p_value = chi_square_tail(statistic * statistic, 1);
  let estimate = LogitEstimate {
    name,
    value,
    standard_error,
    statistic,
    p_value,
    confidence_lower: value - NORMAL_95 * standard_error,
    confidence_upper: value + NORMAL_95 * standard_error,
  };
  if [
    estimate.value,
    estimate.statistic,
    estimate.p_value,
    estimate.confidence_lower,
    estimate.confidence_upper,
  ]
  .iter()
  .any(|value| !value.is_finite())
  {
    return Err(failed("coefficient inference is not finite"));
  }
  Ok(estimate)
}

fn parameter_names(problem: &EstimationProblem, equation: &str) -> Vec<String> {
  let mut names = Vec::with_capacity(problem.parameter_count());
  if problem.include_intercept {
    names.push(format!("{equation}:{}", problem.intercept_name));
  }
  names.extend(
    problem
      .predictor_names
      .iter()
      .map(|name| format!("{equation}:{name}")),
  );
  names
}

fn rms_y(outcome: &EstimationProblem) -> Result<f64, StatsError> {
  let mean = if outcome.include_intercept {
    outcome.response.iter().sum::<f64>() / outcome.response.len() as f64
  } else {
    0.0
  };
  let rms = outcome
    .response
    .iter()
    .fold(0.0_f64, |value, y| value.hypot(*y - mean))
    / (outcome.response.len() as f64).sqrt();
  if rms.is_finite() && rms > 0.0 {
    Ok(rms)
  } else {
    Err(failed("outcome scale is not finite and positive"))
  }
}

fn options_valid(options: &HeckmanMlOptions) -> bool {
  (1..=200).contains(&options.max_iterations)
}

/// Fit a joint IID Gaussian selection model. `selection` contains every
/// eligible unit and `outcome` contains exactly the ordered selected subset.
/// The existing two-step result supplies a deterministic interior initializer;
/// the returned covariance is the inverse full observed information.
pub fn fit_heckman_ml(
  selection: &EstimationProblem,
  outcome: &EstimationProblem,
  options: &HeckmanMlOptions,
) -> Result<HeckmanMlResult, StatsError> {
  if !options_valid(options) {
    return Err(invalid("iteration limit must be 1..=200"));
  }
  let start: HeckmanTwoStepResult =
    fit_heckman_two_step(selection, outcome, &ProbitOptions::default())?;
  if selection.include_intercept != outcome.include_intercept {
    return Err(invalid(
      "both equations must use the same intercept setting",
    ));
  }
  let n = selection.observation_count();
  let p = outcome.parameter_count();
  let q = selection.parameter_count();
  let dim = p
    .checked_add(q)
    .and_then(|value| value.checked_add(2))
    .ok_or_else(|| invalid("joint parameter count overflowed"))?;
  if dim > MAX_HECKMAN_ML_PARAMETERS {
    return Err(invalid("joint model exceeds 66 parameters"));
  }
  let work = n
    .checked_mul(dim)
    .and_then(|value| value.checked_mul(MAX_ML_EVALUATIONS))
    .ok_or_else(|| invalid("expanded work bound overflowed"))?;
  if work > MAX_HECKMAN_ML_WORK {
    return Err(invalid("joint model exceeds the bounded likelihood work"));
  }
  let selection_design = full_design(selection);
  let outcome_design = full_design(outcome);
  let selected_rows = start.selected_selection_indices.clone();
  let y_scale = rms_y(outcome)?;
  let mut parameter_scales = Vec::with_capacity(dim);
  if outcome.include_intercept {
    parameter_scales.push(y_scale);
  }
  for j in 0..outcome.predictor_names.len() {
    parameter_scales
      .push(y_scale / column_rms(&outcome_design, j + usize::from(outcome.include_intercept))?);
  }
  if selection.include_intercept {
    parameter_scales.push(1.0);
  }
  for j in 0..selection.predictor_names.len() {
    parameter_scales.push(
      1.0
        / column_rms(
          &selection_design,
          j + usize::from(selection.include_intercept),
        )?,
    );
  }
  parameter_scales.extend([1.0, 1.0]);
  if parameter_scales.len() != dim
    || parameter_scales
      .iter()
      .any(|scale| !scale.is_finite() || *scale <= 0.0)
  {
    return Err(failed("parameter scaling is invalid"));
  }

  let mut raw_start = Vec::with_capacity(dim);
  let beta_len = start.coefficients.len().saturating_sub(1);
  raw_start.extend(
    start.coefficients[..beta_len]
      .iter()
      .map(|coefficient| coefficient.value),
  );
  raw_start.extend(
    start
      .selection
      .coefficients
      .iter()
      .map(|coefficient| coefficient.value),
  );
  raw_start.push(start.sigma.ln());
  raw_start.push(start.rho.atanh());
  let initial: Vec<f64> = raw_start
    .iter()
    .zip(&parameter_scales)
    .map(|(value, scale)| value / scale)
    .collect();
  let mut budget = EvaluationBudget {
    observations: n,
    parameters: dim,
    calls: 0,
    work: 0,
  };
  let mut evaluate = |point: &[f64]| {
    budget.charge()?;
    model_score(
      point,
      &parameter_scales,
      selection,
      outcome,
      &selected_rows,
      &selection_design,
      &outcome_design,
    )
  };
  let solution = minimize_bfgs(
    initial,
    &parameter_scales,
    n,
    options.max_iterations,
    &mut evaluate,
  )?;
  let covariance = observed_covariance(&solution.point, &parameter_scales, &mut evaluate)?;
  let theta: Vec<f64> = solution
    .point
    .iter()
    .zip(&parameter_scales)
    .map(|(value, scale)| value * scale)
    .collect();
  let beta = &theta[..p];
  let gamma = &theta[p..p + q];
  let log_sigma = theta[p + q];
  let atanh_rho = theta[p + q + 1];
  let sigma = log_sigma.exp();
  let rho = atanh_rho.tanh();
  if !sigma.is_finite() || sigma <= 0.0 || !rho.is_finite() || rho.abs() >= 1.0 {
    return Err(failed(
      "converged scale/correlation is not regular interior",
    ));
  }

  let outcome_names = parameter_names(outcome, "outcome");
  let selection_names = parameter_names(selection, "selection");
  let mut covariance_parameter_names = outcome_names.clone();
  covariance_parameter_names.extend(selection_names.clone());
  covariance_parameter_names.push("lnsigma".into());
  covariance_parameter_names.push("athrho".into());
  let mut outcome_coefficients = Vec::with_capacity(p);
  for i in 0..p {
    let name = if outcome.include_intercept && i == 0 {
      outcome.intercept_name.clone()
    } else {
      outcome.predictor_names[i - usize::from(outcome.include_intercept)].clone()
    };
    outcome_coefficients.push(normal_estimate(name, beta[i], covariance[i][i])?);
  }
  let mut selection_coefficients = Vec::with_capacity(q);
  for j in 0..q {
    let i = p + j;
    let name = if selection.include_intercept && j == 0 {
      selection.intercept_name.clone()
    } else {
      selection.predictor_names[j - usize::from(selection.include_intercept)].clone()
    };
    selection_coefficients.push(normal_estimate(name, gamma[j], covariance[i][i])?);
  }
  let log_sigma_variance = covariance[p + q][p + q];
  let atanh_rho_variance = covariance[p + q + 1][p + q + 1];
  let ancillary_coefficients = vec![
    normal_estimate("lnsigma".into(), log_sigma, log_sigma_variance)?,
    normal_estimate("athrho".into(), atanh_rho, atanh_rho_variance)?,
  ];
  let sigma_standard_error = sigma * log_sigma_variance.sqrt();
  let sigma_confidence_lower = (log_sigma - NORMAL_95 * log_sigma_variance.sqrt()).exp();
  let sigma_confidence_upper = (log_sigma + NORMAL_95 * log_sigma_variance.sqrt()).exp();
  let rho_standard_error = (1.0 - rho * rho) * atanh_rho_variance.sqrt();
  let rho_confidence_lower = (atanh_rho - NORMAL_95 * atanh_rho_variance.sqrt()).tanh();
  let rho_confidence_upper = (atanh_rho + NORMAL_95 * atanh_rho_variance.sqrt()).tanh();
  let lambda = rho * sigma;
  let lambda_gradient = [lambda, sigma * (1.0 - rho * rho)];
  let lambda_variance = lambda_gradient[0] * lambda_gradient[0] * covariance[p + q][p + q]
    + 2.0 * lambda_gradient[0] * lambda_gradient[1] * covariance[p + q][p + q + 1]
    + lambda_gradient[1] * lambda_gradient[1] * covariance[p + q + 1][p + q + 1];
  let lambda_standard_error = lambda_variance.sqrt();
  if [
    sigma_standard_error,
    sigma_confidence_lower,
    sigma_confidence_upper,
    rho_standard_error,
    rho_confidence_lower,
    rho_confidence_upper,
    lambda,
    lambda_standard_error,
  ]
  .iter()
  .any(|value| !value.is_finite())
    || lambda_variance <= 0.0
  {
    return Err(failed(
      "transformed scale/correlation inference is not finite",
    ));
  }
  let lambda_confidence_lower = lambda - NORMAL_95 * lambda_standard_error;
  let lambda_confidence_upper = lambda + NORMAL_95 * lambda_standard_error;
  let least_squares_null = fit_least_squares(outcome)?;
  let null_sse = least_squares_null
    .fit_statistics
    .residual_sum_of_squares
    .ok_or_else(|| failed("independence null has no residual sum of squares"))?;
  if !null_sse.is_finite() || null_sse <= 0.0 {
    return Err(failed("independence null outcome variance is degenerate"));
  }
  let independent_log_likelihood = start.selection.log_likelihood
    - 0.5
      * outcome.observation_count() as f64
      * ((2.0 * std::f64::consts::PI).ln()
        + 1.0
        + (null_sse / outcome.observation_count() as f64).ln());
  let mut independence_lr_chi_square =
    2.0 * (solution.evaluation.log_likelihood - independent_log_likelihood);
  let lr_tolerance = 1e-8 * (1.0 + solution.evaluation.log_likelihood.abs());
  if independence_lr_chi_square < -lr_tolerance {
    return Err(failed("joint likelihood is below the nested rho=0 model"));
  }
  if independence_lr_chi_square < 0.0 {
    independence_lr_chi_square = 0.0;
  }
  let independence_lr_p_value = chi_square_tail(independence_lr_chi_square, 1);
  let mut excluded_selection_predictors: Vec<String> = selection
    .predictor_names
    .iter()
    .filter(|name| !outcome.predictor_names.contains(name))
    .cloned()
    .collect();
  excluded_selection_predictors.sort();

  Ok(HeckmanMlResult {
    outcome: outcome.outcome.clone(),
    predictor_names: outcome.predictor_names.clone(),
    selection_outcome: selection.outcome.clone(),
    selection_predictor_names: selection.predictor_names.clone(),
    include_intercept: outcome.include_intercept,
    selection_include_intercept: selection.include_intercept,
    outcome_coefficients,
    selection_coefficients,
    ancillary_coefficients,
    covariance,
    covariance_parameter_names,
    observation_count: n,
    selected_count: start.selected_count,
    unselected_count: start.unselected_count,
    selection_sample: selection.sample.clone(),
    outcome_sample: outcome.sample.clone(),
    selected_selection_indices: selected_rows,
    log_likelihood: solution.evaluation.log_likelihood,
    independent_log_likelihood,
    independence_lr_chi_square,
    independence_lr_p_value,
    sigma,
    sigma_standard_error,
    sigma_confidence_lower,
    sigma_confidence_upper,
    rho,
    rho_standard_error,
    rho_confidence_lower,
    rho_confidence_upper,
    lambda,
    lambda_standard_error,
    lambda_confidence_lower,
    lambda_confidence_upper,
    iterations: solution.iterations,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_step: solution.step_bound,
    excluded_selection_predictors,
  })
}

/// Point projection from a retained full-information maximum-likelihood fit.
pub fn heckman_ml_predict_row(
  result: &HeckmanMlResult,
  outcome_values: &[f64],
  selection_values: &[f64],
  kind: HeckmanPredictionKind,
) -> Result<f64, StatsError> {
  let requires_x = matches!(
    kind,
    HeckmanPredictionKind::Xb
      | HeckmanPredictionKind::ConditionalSelectedMean
      | HeckmanPredictionKind::ObservedContributionMean
  );
  let requires_z = kind != HeckmanPredictionKind::Xb;
  let beta = if requires_x {
    if outcome_values.len() != result.predictor_names.len()
      || outcome_values.iter().any(|value| !value.is_finite())
    {
      return Err(invalid(
        "prediction outcome design must match finite fitted columns",
      ));
    }
    let mut coefficients = Vec::with_capacity(result.outcome_coefficients.len());
    coefficients.extend(
      result
        .outcome_coefficients
        .iter()
        .map(|estimate| estimate.value),
    );
    let mut values =
      Vec::with_capacity(outcome_values.len() + usize::from(result.include_intercept));
    if result.include_intercept {
      values.push(1.0);
    }
    values.extend_from_slice(outcome_values);
    Some(dot(&values, &coefficients))
  } else {
    None
  };
  let z_index = if requires_z {
    if selection_values.len() != result.selection_predictor_names.len()
      || selection_values.iter().any(|value| !value.is_finite())
    {
      return Err(invalid(
        "prediction selection design must match finite fitted columns",
      ));
    }
    let mut coefficients = Vec::with_capacity(result.selection_coefficients.len());
    coefficients.extend(
      result
        .selection_coefficients
        .iter()
        .map(|estimate| estimate.value),
    );
    let mut values =
      Vec::with_capacity(selection_values.len() + usize::from(result.selection_include_intercept));
    if result.selection_include_intercept {
      values.push(1.0);
    }
    values.extend_from_slice(selection_values);
    dot(&values, &coefficients)
  } else {
    0.0
  };
  let mean = beta.unwrap_or(0.0);
  let value = match kind {
    HeckmanPredictionKind::Xb => mean,
    HeckmanPredictionKind::Pr => normal_cdf(z_index),
    HeckmanPredictionKind::Mills => lower_inverse_mills(z_index)?,
    HeckmanPredictionKind::ConditionalSelectedMean => {
      mean + result.lambda * lower_inverse_mills(z_index)?
    }
    HeckmanPredictionKind::ObservedContributionMean => {
      normal_cdf(z_index) * mean + result.lambda * normal_pdf(z_index)
    }
  };
  if value.is_finite() {
    Ok(value)
  } else {
    Err(failed(
      "prediction is outside the represented numerical range",
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn log_normal_cdf_and_inverse_mills_match_independent_r_tails() {
    for (argument, expected) in [
      (-40.0, -804.6084420137538),
      (-10.0, -53.23128515051247),
      (-8.000001, -35.013_445_281_283_15),
      (-8.0, -35.013_437_159_914_55),
      (-2.0, -3.7831843336820317),
      (0.0, -std::f64::consts::LN_2),
      (5.0, -2.866_516_129_637_636e-7),
    ] {
      let actual = log_normal_cdf(argument).unwrap();
      assert!(
        (actual - expected).abs() <= 2e-13 * expected.abs().max(1.0),
        "log Phi({argument}): {actual:.17e} != {expected:.17e}"
      );
    }
    assert!((lower_inverse_mills(-40.0).unwrap() - 40.02496884720726).abs() < 1e-10);
  }

  #[test]
  fn joint_score_matches_finite_differences_of_the_observed_likelihood() {
    let selection = EstimationProblem::new(
      "d",
      vec!["z".into()],
      vec![0.0, 1.0, 0.0, 1.0, 1.0, 0.0],
      vec![
        vec![-0.7],
        vec![0.2],
        vec![1.1],
        vec![-0.4],
        vec![0.8],
        vec![0.5],
      ],
    )
    .unwrap();
    let outcome = EstimationProblem::new(
      "y",
      vec!["x".into()],
      vec![0.6, 1.8, -0.3],
      vec![vec![0.2], vec![-0.4], vec![0.8]],
    )
    .unwrap();
    let selected_rows = vec![1, 3, 4];
    let selection_design = full_design(&selection);
    let outcome_design = full_design(&outcome);
    let scales = vec![1.0; 6];
    let point = vec![0.7, 0.3, -0.2, 0.15, 0.1_f64.ln(), 0.25_f64.atanh()];
    let evaluation = model_score(
      &point,
      &scales,
      &selection,
      &outcome,
      &selected_rows,
      &selection_design,
      &outcome_design,
    )
    .unwrap();
    for j in 0..point.len() {
      let step = 1e-6 * (1.0 + point[j].abs());
      let mut plus = point.clone();
      let mut minus = point.clone();
      plus[j] += step;
      minus[j] -= step;
      let plus = model_score(
        &plus,
        &scales,
        &selection,
        &outcome,
        &selected_rows,
        &selection_design,
        &outcome_design,
      )
      .unwrap()
      .log_likelihood;
      let minus = model_score(
        &minus,
        &scales,
        &selection,
        &outcome,
        &selected_rows,
        &selection_design,
        &outcome_design,
      )
      .unwrap()
      .log_likelihood;
      let numeric = (plus - minus) / (2.0 * step);
      assert!(
        (evaluation.score[j] - numeric).abs() <= 2e-7 * (1.0 + numeric.abs()),
        "score {j}: {} != {numeric}",
        evaluation.score[j]
      );
    }
  }

  #[test]
  fn boundary_correlation_parameters_fail_closed() {
    let selection = EstimationProblem::new(
      "d",
      vec!["z".into()],
      vec![0.0, 1.0],
      vec![vec![-0.5], vec![0.5]],
    )
    .unwrap();
    let outcome =
      EstimationProblem::new("y", vec!["x".into()], vec![1.0], vec![vec![0.25]]).unwrap();
    let selection_design = full_design(&selection);
    let outcome_design = full_design(&outcome);
    let mut parameters = vec![0.0; outcome.parameter_count() + selection.parameter_count() + 2];
    *parameters.last_mut().unwrap() = 8.000_001;
    let result = model_score(
      &parameters,
      &vec![1.0; parameters.len()],
      &selection,
      &outcome,
      &[1],
      &selection_design,
      &outcome_design,
    );
    let Err(error) = result else {
      panic!("a correlation outside the regular range must fail");
    };
    assert!(
      error
        .to_string()
        .contains("correlation parameter left its regular range")
    );
  }
}
