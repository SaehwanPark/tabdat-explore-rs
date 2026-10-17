//! Shared unweighted Bernoulli binary-response maximum likelihood: logit and
//! probit over one scaled Newton/QR core with family-specific link, observed
//! information, and marginal-effect conventions.

use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf, normal_quantile};
use crate::matrix::{qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Which binary response mechanism the maximum likelihood assumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryModel {
  Logit,
  Probit,
}

impl BinaryModel {
  fn label(self) -> &'static str {
    match self {
      Self::Logit => "logit",
      Self::Probit => "probit",
    }
  }
}

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

/// Explicit iteration bound; runtime commands use the default bound.
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

/// Explicit iteration bound; runtime commands use the default bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbitOptions {
  pub max_iterations: usize,
}

impl Default for ProbitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Unweighted independent-Bernoulli binary maximum likelihood result (logit
/// or probit). Covariance convention is family-specific: inverse Fisher
/// information for `Logit`, inverse observed information evaluated at the
/// converged coefficients for `Probit`, both without an N-K correction.
/// Coefficient/AME inference is asymptotic normal, never Student t.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryMleResult {
  pub model: BinaryModel,
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

/// Compatibility name for logit results before the shared binary core.
pub type BinaryLogitResult = BinaryMleResult;

/// Conditional-on-observed-design continuous average partial derivatives.
/// These are not discrete changes or chain-rule effects of original variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogitMarginalEffects {
  pub observation_count: usize,
  pub effects: Vec<LogitEstimate>,
}

fn invalid(model: BinaryModel, message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("{}: {message}", model.label()))
}

fn failed(model: BinaryModel, message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("{}: {message}", model.label()))
}

fn inference(
  model: BinaryModel,
  name: String,
  value: f64,
  standard_error: f64,
) -> Result<LogitEstimate, StatsError> {
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
      model,
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
  /// Fitted event probability per observation.
  probabilities: Vec<f64>,
  /// Stable p*(1-p) result payload; a probit step may use another diagonal.
  response_variances: Vec<f64>,
  /// Working diagonal for this iteration's information: the per-observation
  /// observed-information diagonal for probit's Newton step while every
  /// entry keeps positivity, the Fisher diagonal otherwise, and logit's
  /// stable p*(1-p) response variance.
  weights: Vec<f64>,
  /// Probit only: per-observation observed-information diagonal, reused for
  /// the converged covariance check.
  curvature_diagonal: Option<Vec<f64>>,
  /// Score residuals X'(these): logit y-p; probit (y-p)*phi/(p(1-p)).
  residuals: Vec<f64>,
  log_likelihood: f64,
}

fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  beta: &[f64],
  model: BinaryModel,
) -> Result<Evaluation, StatsError> {
  let mut probabilities = Vec::with_capacity(y.len());
  let mut response_variances = Vec::with_capacity(y.len());
  let mut residuals = Vec::with_capacity(y.len());
  let mut curvature_diagonal = match model {
    BinaryModel::Logit => None,
    BinaryModel::Probit => Some(Vec::with_capacity(y.len())),
  };
  let mut fisher_diagonal = match model {
    BinaryModel::Logit => None,
    BinaryModel::Probit => Some(Vec::with_capacity(y.len())),
  };
  let mut log_likelihood = 0.0;
  for (row, &outcome) in x.iter().zip(y) {
    let eta: f64 = row.iter().zip(beta).map(|(a, b)| a * b).sum();
    if !eta.is_finite() {
      return Err(failed(model, "linear index is non-finite"));
    }
    match model {
      BinaryModel::Logit => {
        let tail = (-eta.abs()).exp();
        let denominator = 1.0 + tail;
        let (p, q) = if eta >= 0.0 {
          (1.0 / denominator, tail / denominator)
        } else {
          (tail / denominator, 1.0 / denominator)
        };
        probabilities.push(p);
        residuals.push(if outcome == 1.0 { q } else { -p });
        // Neither log(p) at saturation nor eta-softplus(eta) cancellation.
        let signed_eta = if outcome == 1.0 { -eta } else { eta };
        log_likelihood -= signed_eta.max(0.0) + tail.ln_1p();
        response_variances.push(tail / denominator.powi(2));
      }
      BinaryModel::Probit => {
        // Complementary half-tails: p and q sum to exactly one, and a tiny
        // tail holds its own precision instead of 1 - p cancellation.
        let p = normal_cdf(eta);
        if !p.is_finite() {
          return Err(failed(model, "fitted probability is non-finite"));
        }
        let q = normal_cdf(-eta);
        if !q.is_finite() {
          return Err(failed(model, "fitted probability is non-finite"));
        }
        // Density-to-probability ratio a = phi/(p q); the density and a
        // saturated tail underflow together, so a vanishing variance gives
        // a vanishing score contribution with finite, non-NaN pieces.
        let variance = p * q;
        let pdf = normal_pdf(eta);
        let score_slope = if variance > 0.0 { pdf / variance } else { 0.0 };
        let fisher = pdf * score_slope;
        probabilities.push(p);
        response_variances.push(variance);
        residuals.push((outcome - p) * score_slope);
        curvature_diagonal
          .as_mut()
          .unwrap()
          .push(fisher + (outcome - p) * score_slope * (eta + (1.0 - 2.0 * p) * score_slope));
        fisher_diagonal.as_mut().unwrap().push(fisher);
        // p = 1 - q is exact, so the log density is ln1p(-q) for events and
        // ln(q) otherwise; both are negative, and accumulating them keeps
        // no log(p) at saturation and no 1 - p cancellation.
        log_likelihood += if outcome == 1.0 { (-q).ln_1p() } else { q.ln() };
      }
    }
  }
  if !log_likelihood.is_finite() {
    return Err(failed(model, "log likelihood is non-finite"));
  }
  // Logit scores the Fisher information. Probit takes Newton on the observed
  // information while every per-observation diagonal keeps positivity; a
  // negative diagonal is a legitimate nonconcavity of the observed
  // information, so that iteration falls back to Fisher scoring entirely.
  let weights = if let Some(curvature) = &curvature_diagonal {
    if curvature.iter().any(|&j| !j.is_finite() || j < 0.0) {
      fisher_diagonal.as_ref().unwrap().clone()
    } else {
      curvature.clone()
    }
  } else {
    response_variances.clone()
  };
  Ok(Evaluation {
    probabilities,
    response_variances,
    curvature_diagonal,
    residuals,
    weights,
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

/// Fit an unweighted independent-Bernoulli logit MLE using the default
/// iteration bound.
pub fn fit_binary_logit(problem: &EstimationProblem) -> Result<BinaryMleResult, StatsError> {
  fit_binary_logit_with_options(problem, &LogitOptions::default())
}

/// Fit an unweighted independent-Bernoulli probit MLE using the default
/// iteration bound. Covariance is inverse observed information evaluated at
/// the converged coefficients.
pub fn fit_binary_probit(problem: &EstimationProblem) -> Result<BinaryMleResult, StatsError> {
  fit_binary_probit_with_options(problem, &ProbitOptions::default())
}

/// Fit an unweighted independent-Bernoulli logit MLE with an explicit
/// iteration bound.
pub fn fit_binary_logit_with_options(
  problem: &EstimationProblem,
  options: &LogitOptions,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(problem, BinaryModel::Logit, options.max_iterations)
}

pub fn fit_binary_probit_with_options(
  problem: &EstimationProblem,
  options: &ProbitOptions,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(problem, BinaryModel::Probit, options.max_iterations)
}

/// Scaled Newton/QR with deterministic backtracking for both binary families.
/// A small likelihood change or score alone does not establish convergence
/// under separation: the *full* Newton step must also be small. Singular
/// information or exhaustion fails without publishing coefficients. This is
/// not an analytic separation classifier. Logit scores the Fisher
/// information; probit takes Newton on the observed information with a
/// Fisher-scoring fallback when a per-observation diagonal loses positivity.
fn fit_binary_mle(
  problem: &EstimationProblem,
  model: BinaryModel,
  max_iterations: usize,
) -> Result<BinaryMleResult, StatsError> {
  problem.validate()?;
  if max_iterations == 0 {
    return Err(invalid(model, "max_iterations must be positive"));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|s| s.weights.is_some() || s.cluster_groups.is_some())
  {
    return Err(invalid(
      model,
      "weights and clustered designs are not implemented",
    ));
  }
  if problem.response.iter().any(|v| *v != 0.0 && *v != 1.0) {
    return Err(invalid(
      model,
      "outcome must be exactly 0 or 1 on the estimation sample",
    ));
  }
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if n <= k {
    return Err(StatsError::InsufficientObservations(format!(
      "{} requires more observations than parameters",
      model.label()
    )));
  }
  let events = problem.response.iter().filter(|&&v| v == 1.0).count();
  if events == 0 || events == n {
    return Err(invalid(
      model,
      "this bounded workflow requires both outcome classes",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid(
      model,
      "parameter names must be distinct, including the intercept",
    ));
  }
  let mut x = full_design(problem);
  let scales: Vec<f64> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])))
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid(
      model,
      "predictor norms must be finite and positive",
    ));
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
    // The start inverts the family link at the event proportion, in
    // unit-column-norm coordinates; everything else starts at zero.
    beta[0] = match model {
      BinaryModel::Logit => (proportion / (1.0 - proportion)).ln() * scales[0],
      BinaryModel::Probit => normal_quantile(proportion) * scales[0],
    };
    events as f64 * proportion.ln() + (n - events) as f64 * (-proportion).ln_1p()
  } else {
    -(n as f64) * std::f64::consts::LN_2
  };
  for iteration in 1..=max_iterations {
    let current = evaluate(&x, &problem.response, &beta, model)?;
    let weighted: Vec<Vec<f64>> = x
      .iter()
      .zip(&current.weights)
      .map(|(row, w)| row.iter().map(|entry| entry * w.sqrt()).collect())
      .collect();
    let qr = qr_decompose_with_response(&weighted, &zero_response).map_err(|_| {
      failed(
        model,
        "information is singular or numerically unidentified (possible separation)",
      )
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
      return Err(failed(model, "Newton step or score is non-finite"));
    }
    let step_norm = maximum_absolute(&step);
    let score_norm = maximum_absolute(&score);
    // Absolute in unit-column-norm coordinates: another large coefficient
    // must not relax the bound on a still-diverging separation direction.
    if step_norm <= TOLERANCE && score_norm <= TOLERANCE {
      let scaled_covariance = match model {
        BinaryModel::Logit => xtx_inverse_from_r(&qr.r)?,
        BinaryModel::Probit => {
          let curvature = current.curvature_diagonal.as_ref().unwrap();
          if curvature.iter().any(|&j| !j.is_finite() || j < 0.0) {
            return Err(failed(
              model,
              "observed information has nonpositive curvature",
            ));
          }
          // This evaluation's step diagonal is the observed information, so
          // the converged QR is itself R'R = observed information.
          xtx_inverse_from_r(&qr.r)?
        }
      };
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
          model,
          "covariance cannot be represented in original predictor units",
        ));
      }
      let coefficients = (0..k)
        .map(|j| {
          inference(
            model,
            names[j].clone(),
            beta[j] / scales[j],
            covariance[j][j].sqrt(),
          )
        })
        .collect::<Result<Vec<_>, _>>()?;
      let likelihood_ratio = 2.0 * (current.log_likelihood - null_log_likelihood);
      if likelihood_ratio < -1e-10 * (1.0 + null_log_likelihood.abs()) {
        return Err(failed(
          model,
          "fitted likelihood is below the nested null model",
        ));
      }
      let likelihood_ratio = likelihood_ratio.max(0.0);
      let df = problem.predictor_names.len();
      let likelihood_ratio_p_value = (df > 0).then(|| chi_square_tail(likelihood_ratio, df));
      if likelihood_ratio_p_value.is_some_and(|p| !p.is_finite()) {
        return Err(failed(
          model,
          "likelihood-ratio chi-square inference is unavailable",
        ));
      }
      return Ok(BinaryMleResult {
        model,
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
        fitted_weights: current.response_variances,
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
      if let Ok(evaluation) = evaluate(&x, &problem.response, &candidate, model) {
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
    beta = accepted
      .ok_or_else(|| failed(model, "backtracking could not find a finite improving step"))?;
  }
  Err(failed(
    model,
    &format!(
      "did not converge within {} information evaluations; possible separation or ill-conditioning",
      max_iterations
    ),
  ))
}

/// Average continuous predictor-column effects over the *retained training X*.
/// Caller must provide the exact problem used for this fit. Uncertainty is the
/// normal delta method conditional on observed X, through the fitted family's
/// own covariance convention, including intercept/slopes' covariance. It does
/// not estimate the sampling variation of X's distribution.
pub fn binary_average_marginal_effects(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
) -> Result<LogitMarginalEffects, StatsError> {
  problem.validate()?;
  let model = result.model;
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
      model,
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
    return Err(failed(
      model,
      "marginal effects require finite fit quantities",
    ));
  }
  let x = full_design(problem);
  // (link derivative, link second derivative) mean structures.
  let (mean_link, mean_link_change) = match model {
    BinaryModel::Logit => {
      // Stored from the converged fit: stable p*(1-p) stays the link
      // derivative even when a probability rounds to exactly one.
      let mean_link = result.fitted_weights.iter().sum::<f64>() / n as f64;
      let mean_link_change: Vec<f64> = (0..k)
        .map(|j| {
          x.iter()
            .zip(&result.fitted_weights)
            .zip(&result.fitted_probabilities)
            .map(|((row, w), p)| row[j] * w * (1.0 - 2.0 * p))
            .sum::<f64>()
            / n as f64
        })
        .collect();
      (mean_link, mean_link_change)
    }
    BinaryModel::Probit => {
      // Density pieces recomputed from the reported coefficients and the
      // retained design, so saturation in the stored probabilities cannot
      // echo into the delta method.
      let etas: Vec<f64> = x
        .iter()
        .map(|row| {
          row
            .iter()
            .zip(&result.coefficients)
            .map(|(a, c)| a * c.value)
            .sum()
        })
        .collect();
      if etas.iter().any(|z| !z.is_finite()) {
        return Err(failed(
          model,
          "marginal effects require finite fit quantities",
        ));
      }
      let mean_link = etas.iter().map(|&z| normal_pdf(z)).sum::<f64>() / n as f64;
      let mean_link_change: Vec<f64> = (0..k)
        .map(|j| {
          x.iter()
            .zip(&etas)
            .map(|(row, z)| row[j] * (-(z * normal_pdf(*z))))
            .sum::<f64>()
            / n as f64
        })
        .collect();
      (mean_link, mean_link_change)
    }
  };
  let mut effects = Vec::new();
  for (column, name) in problem.predictor_names.iter().enumerate() {
    let index = column + usize::from(problem.include_intercept);
    let beta = result.coefficients[index].value;
    let mut gradient: Vec<f64> = mean_link_change.iter().map(|d| beta * d).collect();
    gradient[index] += mean_link;
    let variance: f64 = (0..k)
      .map(|i| {
        (0..k)
          .map(|j| gradient[i] * result.covariance[i][j] * gradient[j])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(
      model,
      name.clone(),
      beta * mean_link,
      variance.sqrt(),
    )?);
  }
  Ok(LogitMarginalEffects {
    observation_count: n,
    effects,
  })
}

/// Compatibility name for the logit variant of the shared binary core. It
/// only admits logit results, so prior callers stay name-checked.
pub fn logit_average_marginal_effects(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
) -> Result<LogitMarginalEffects, StatsError> {
  if result.model != BinaryModel::Logit {
    return Err(invalid(
      BinaryModel::Logit,
      "the logit compatibility alias requires a logit result",
    ));
  }
  binary_average_marginal_effects(problem, result)
}
