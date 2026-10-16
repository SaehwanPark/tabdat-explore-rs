use crate::distributions::chi_square_tail;
use crate::matrix::{qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Normal Wald inference, with a fixed 95% confidence interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogitEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: f64,
  pub p_value: f64,
  pub confidence_lower: f64,
  pub confidence_upper: f64,
}

/// The iteration bound is explicit; runtime commands use the default bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogitOptions {
  pub max_iterations: usize,
}

impl Default for LogitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Unweighted independent-Bernoulli logit maximum likelihood result.
/// Covariance is inverse Fisher information, without an N-K correction.
/// Coefficient/AME inference is asymptotic normal, never Student t.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryLogitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub event_count: usize,
  pub coefficients: Vec<LogitEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  /// None for an intercept-only model (zero restrictions).
  pub likelihood_ratio_p_value: Option<f64>,
  pub mcfadden_r_squared: f64,
  /// Information evaluations, including the final convergence check.
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_probabilities: Vec<f64>,
  /// Stable p*(1-p), even when a probability rounds to exactly one.
  pub fitted_weights: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

/// Conditional-on-observed-design continuous average partial derivatives.
/// These are not discrete changes or chain-rule effects of original variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogitMarginalEffects {
  pub observation_count: usize,
  pub effects: Vec<LogitEstimate>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("logit: {message}"))
}

fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("logit: {message}"))
}

fn inference(name: String, value: f64, standard_error: f64) -> Result<LogitEstimate, StatsError> {
  let statistic = value / standard_error;
  // Chi-square(1) survival is the two-sided standard-normal tail. Direct
  // survival evaluation preserves very small representable probabilities.
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
    return Err(failed(
      "normal inference is non-finite or has nonpositive variance",
    ));
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

struct Evaluation {
  probabilities: Vec<f64>,
  weights: Vec<f64>,
  residuals: Vec<f64>,
  log_likelihood: f64,
}

fn evaluate(x: &[Vec<f64>], y: &[f64], beta: &[f64]) -> Result<Evaluation, StatsError> {
  let mut probabilities = Vec::with_capacity(y.len());
  let mut weights = Vec::with_capacity(y.len());
  let mut residuals = Vec::with_capacity(y.len());
  let mut log_likelihood = 0.0;
  for (row, &outcome) in x.iter().zip(y) {
    let eta: f64 = row.iter().zip(beta).map(|(a, b)| a * b).sum();
    if !eta.is_finite() {
      return Err(failed("linear index is non-finite"));
    }
    let tail = (-eta.abs()).exp();
    let denominator = 1.0 + tail;
    let (p, q) = if eta >= 0.0 {
      (1.0 / denominator, tail / denominator)
    } else {
      (tail / denominator, 1.0 / denominator)
    };
    probabilities.push(p);
    weights.push(tail / denominator.powi(2));
    residuals.push(if outcome == 1.0 { q } else { -p });
    // Neither log(p) at saturation nor eta-softplus(eta) cancellation.
    let signed_eta = if outcome == 1.0 { -eta } else { eta };
    log_likelihood -= signed_eta.max(0.0) + tail.ln_1p();
  }
  if !log_likelihood.is_finite() {
    return Err(failed("log likelihood is non-finite"));
  }
  Ok(Evaluation {
    probabilities,
    weights,
    residuals,
    log_likelihood,
  })
}

fn full_design(problem: &EstimationProblem) -> Vec<Vec<f64>> {
  problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::repeat_n(1.0, usize::from(problem.include_intercept))
        .chain(row.iter().copied())
        .collect()
    })
    .collect()
}

fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

/// Fit a finite, numerically identified logit MLE using the default iteration bound.
pub fn fit_binary_logit(problem: &EstimationProblem) -> Result<BinaryLogitResult, StatsError> {
  fit_binary_logit_with_options(problem, &LogitOptions::default())
}

/// Scaled Newton/QR with deterministic backtracking. A small likelihood change
/// or score alone does not establish convergence under separation: the *full*
/// Newton step must also be small. Singular information or exhaustion fails
/// without publishing coefficients. This is not an analytic separation classifier.
pub fn fit_binary_logit_with_options(
  problem: &EstimationProblem,
  options: &LogitOptions,
) -> Result<BinaryLogitResult, StatsError> {
  problem.validate()?;
  if options.max_iterations == 0 {
    return Err(invalid("max_iterations must be positive"));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|s| s.weights.is_some() || s.cluster_groups.is_some())
  {
    return Err(invalid("weights and clustered designs are not implemented"));
  }
  if problem.response.iter().any(|v| *v != 0.0 && *v != 1.0) {
    return Err(invalid(
      "outcome must be exactly 0 or 1 on the estimation sample",
    ));
  }
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if n <= k {
    return Err(StatsError::InsufficientObservations(
      "logit requires more observations than parameters".into(),
    ));
  }
  let events = problem.response.iter().filter(|&&v| v == 1.0).count();
  if events == 0 || events == n {
    return Err(invalid(
      "this bounded workflow requires both outcome classes",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid(
      "parameter names must be distinct, including the intercept",
    ));
  }
  let mut x = full_design(problem);
  let scales: Vec<f64> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])))
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid("predictor norms must be finite and positive"));
  }
  for row in &mut x {
    for (entry, scale) in row.iter_mut().zip(&scales) {
      *entry /= scale;
    }
  }
  let zero_response = vec![0.0; n];
  // Reject a deficient original design separately from vanishing information.
  qr_decompose_with_response(&x, &zero_response)?;
  let mut beta = vec![0.0; k];
  let proportion = events as f64 / n as f64;
  let null_log_likelihood = if problem.include_intercept {
    beta[0] = (proportion / (1.0 - proportion)).ln() * scales[0];
    events as f64 * proportion.ln() + (n - events) as f64 * (-proportion).ln_1p()
  } else {
    -(n as f64) * std::f64::consts::LN_2
  };
  for iteration in 1..=options.max_iterations {
    let current = evaluate(&x, &problem.response, &beta)?;
    let weighted: Vec<Vec<f64>> = x
      .iter()
      .zip(&current.weights)
      .map(|(row, w)| row.iter().map(|entry| entry * w.sqrt()).collect())
      .collect();
    let qr = qr_decompose_with_response(&weighted, &zero_response).map_err(|_| {
      failed("information is singular or numerically unidentified (possible separation)")
    })?;
    let score: Vec<f64> = (0..k)
      .map(|j| {
        x.iter()
          .zip(&current.residuals)
          .map(|(row, residual)| row[j] * residual)
          .sum()
      })
      .collect();
    // R'R step = score, without forming the normal matrix or dividing a
    // working response by tiny/zero Bernoulli standard deviations.
    let mut intermediate = vec![0.0; k];
    for i in 0..k {
      let preceding: f64 = (0..i).map(|j| qr.r[j][i] * intermediate[j]).sum();
      intermediate[i] = (score[i] - preceding) / qr.r[i][i];
    }
    let step = solve_upper_triangular(&qr.r, &intermediate)?;
    if step.iter().chain(&score).any(|v| !v.is_finite()) {
      return Err(failed("Newton step or score is non-finite"));
    }
    let step_norm = maximum_absolute(&step);
    let score_norm = maximum_absolute(&score);
    if step_norm <= TOLERANCE * (1.0 + maximum_absolute(&beta)) && score_norm <= TOLERANCE {
      let scaled_covariance = xtx_inverse_from_r(&qr.r)?;
      let covariance: Vec<Vec<f64>> = scaled_covariance
        .iter()
        .enumerate()
        .map(|(i, row)| {
          row
            .iter()
            .enumerate()
            .map(|(j, v)| v / scales[i] / scales[j])
            .collect()
        })
        .collect();
      if covariance.iter().flatten().any(|v| !v.is_finite()) {
        return Err(failed(
          "covariance cannot be represented in original predictor units",
        ));
      }
      let coefficients = (0..k)
        .map(|j| {
          inference(
            names[j].clone(),
            beta[j] / scales[j],
            covariance[j][j].sqrt(),
          )
        })
        .collect::<Result<Vec<_>, _>>()?;
      let likelihood_ratio = 2.0 * (current.log_likelihood - null_log_likelihood);
      if likelihood_ratio < -1e-10 * (1.0 + null_log_likelihood.abs()) {
        return Err(failed("fitted likelihood is below the nested null model"));
      }
      let likelihood_ratio = likelihood_ratio.max(0.0);
      let df = problem.predictor_names.len();
      let likelihood_ratio_p_value = (df > 0).then(|| chi_square_tail(likelihood_ratio, df));
      if likelihood_ratio_p_value.is_some_and(|p| !p.is_finite()) {
        return Err(failed(
          "likelihood-ratio chi-square inference is unavailable",
        ));
      }
      return Ok(BinaryLogitResult {
        outcome: problem.outcome.clone(),
        predictor_names: problem.predictor_names.clone(),
        include_intercept: problem.include_intercept,
        observation_count: n,
        event_count: events,
        coefficients,
        covariance,
        log_likelihood: current.log_likelihood,
        null_log_likelihood,
        likelihood_ratio,
        likelihood_ratio_df: df,
        likelihood_ratio_p_value,
        mcfadden_r_squared: -likelihood_ratio / (2.0 * null_log_likelihood),
        iterations: iteration,
        max_abs_scaled_score: score_norm,
        max_abs_scaled_newton_step: step_norm,
        fitted_probabilities: current.probabilities,
        fitted_weights: current.weights,
        sample: problem.sample.clone(),
      });
    }
    let directional: f64 = step.iter().zip(&score).map(|(a, b)| a * b).sum();
    let mut fraction = 1.0;
    let mut accepted = None;
    for _ in 0..50 {
      let candidate: Vec<f64> = beta
        .iter()
        .zip(&step)
        .map(|(b, d)| b + fraction * d)
        .collect();
      if let Ok(evaluation) = evaluate(&x, &problem.response, &candidate) {
        // Allow likelihood accumulation roundoff, not a convergence shortcut.
        let slack = 1e-12 * (1.0 + current.log_likelihood.abs());
        if evaluation.log_likelihood
          >= current.log_likelihood + 1e-4 * fraction * directional - slack
        {
          accepted = Some(candidate);
          break;
        }
      }
      fraction *= 0.5;
    }
    beta = accepted.ok_or_else(|| failed("backtracking could not find a finite improving step"))?;
  }
  Err(failed(&format!(
    "did not converge within {} information evaluations; possible separation or ill-conditioning",
    options.max_iterations
  )))
}

/// Average continuous predictor-column effects over the *retained training X*.
/// Caller must provide the exact problem used for this fit. Uncertainty is the
/// normal delta method conditional on observed X, including intercept/slopes'
/// covariance. It does not estimate the sampling variation of X's distribution.
pub fn logit_average_marginal_effects(
  problem: &EstimationProblem,
  result: &BinaryLogitResult,
) -> Result<LogitMarginalEffects, StatsError> {
  problem.validate()?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if result.outcome != problem.outcome
    || result.predictor_names != problem.predictor_names
    || result.include_intercept != problem.include_intercept
    || result.sample != problem.sample
    || result.observation_count != n
    || result
      .coefficients
      .iter()
      .map(|c| c.name.clone())
      .collect::<Vec<_>>()
      != problem.parameter_names()
    || result.fitted_probabilities.len() != n
    || result.fitted_weights.len() != n
    || result.covariance.len() != k
    || result.covariance.iter().any(|row| row.len() != k)
  {
    return Err(invalid(
      "marginal effects require the exact retained fit/design/sample",
    ));
  }
  if result
    .fitted_probabilities
    .iter()
    .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
    || result
      .fitted_weights
      .iter()
      .any(|w| !w.is_finite() || !(0.0..=0.25).contains(w))
    || result.covariance.iter().flatten().any(|v| !v.is_finite())
    || result.coefficients.iter().any(|c| !c.value.is_finite())
  {
    return Err(failed("marginal effects require finite fit quantities"));
  }
  let x = full_design(problem);
  let mean_weight = result.fitted_weights.iter().sum::<f64>() / n as f64;
  let mean_derivative: Vec<f64> = (0..k)
    .map(|j| {
      x.iter()
        .zip(&result.fitted_weights)
        .zip(&result.fitted_probabilities)
        .map(|((row, w), p)| row[j] * w * (1.0 - 2.0 * p))
        .sum::<f64>()
        / n as f64
    })
    .collect();
  let mut effects = Vec::new();
  for (column, name) in problem.predictor_names.iter().enumerate() {
    let index = column + usize::from(problem.include_intercept);
    let beta = result.coefficients[index].value;
    let mut gradient: Vec<f64> = mean_derivative.iter().map(|d| beta * d).collect();
    gradient[index] += mean_weight;
    let variance: f64 = (0..k)
      .map(|i| {
        (0..k)
          .map(|j| gradient[i] * result.covariance[i][j] * gradient[j])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(
      name.clone(),
      beta * mean_weight,
      variance.sqrt(),
    )?);
  }
  Ok(LogitMarginalEffects {
    observation_count: n,
    effects,
  })
}
