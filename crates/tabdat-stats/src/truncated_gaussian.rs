//! IID Gaussian regression conditional on Y*>lower, not censored Tobit.
//! Canonical eta=beta/sigma² and precision v>0 make this an exponential family:
//! log density is y*X*eta-v*y²/2 minus its conditional log partition. Its negative
//! Hessian is a sufficient-statistic covariance, unlike beta/logsigma away from
//! the solution. Reported inference uses the FULL observed beta/logsigma block.
#![allow(clippy::needless_range_loop)]
use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf};
use crate::gaussian_mle::{Evaluation, Solution};
use crate::matrix::{add_compensated, choice_information_inverse, qr_decompose_with_response};
use crate::tobit::{TobitEstimate, TobitScale, inference as normal_inference};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const NORMAL_95: f64 = 1.959963984540054;
pub const MAX_TRUNCATED_GAUSSIAN_OBSERVATIONS: usize = 10_000;
pub const MAX_TRUNCATED_GAUSSIAN_PARAMETERS: usize = 32;
pub const MAX_TRUNCATED_GAUSSIAN_WORK: usize = 10_000_000;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncatedGaussianOptions {
  pub max_iterations: usize,
}
impl Default for TruncatedGaussianOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
    }
  }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncatedGaussianPredictionKind {
  Latent,
  RetentionProbability,
  ConditionalMean,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TruncatedGaussianResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub lower_limit: f64,
  pub observation_count: usize,
  pub coefficients: Vec<TobitEstimate>,
  pub scale: TobitScale,
  /// Location order, then lnsigma; FULL observed information/correction1.
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub iterations: usize,
  /// RMS-X/effective-response canonical eta/precision coordinates, not raw beta.
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_latent_values: Vec<f64>,
  pub fitted_retention_probabilities: Vec<f64>,
  pub fitted_conditional_means: Vec<f64>,
  /// Retained-X sufficient summaries; effects never re-read changed data.
  pub mean_conditional_derivative: f64,
  pub mean_conditional_derivative_gradient: Vec<f64>,
  pub sample: Option<EstimationSample>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TruncatedGaussianMarginalEffects {
  pub observation_count: usize,
  pub lower_limit: f64,
  pub effects: Vec<TobitEstimate>,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("truncreg: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("truncreg: {message}"))
}
fn inference(name: String, value: f64, variance: f64) -> Result<TobitEstimate, StatsError> {
  normal_inference(name, value, variance)
    .map_err(|_| failed("normal inference is non-finite or has nonpositive variance"))
}
struct Tail {
  probability: f64,
  log_probability: f64,
  mills: f64,
  delta: f64,
  h: f64,
  hprime: f64,
}
fn tail(t: f64) -> Result<Tail, StatsError> {
  if t.is_nan() {
    return Err(failed("non-finite truncation index"));
  }
  let probability = normal_cdf(t);
  if !probability.is_finite() || probability <= 0.0 {
    return Err(failed(
      "retention probability/Mills ratio is outside the represented numerical range",
    ));
  }
  let log_probability = if t >= 0.0 {
    (-normal_cdf(-t)).ln_1p()
  } else {
    probability.ln()
  };
  let mills = normal_pdf(t) / probability;
  // Exact floating-point saturation is the ordinary Gaussian limit. Avoid
  // infinity*zero when a statistically inactive lower bound is extremely far away.
  let (delta, hprime) = if mills == 0.0 {
    (0.0, 0.0)
  } else {
    let delta = mills * (mills + t);
    (delta, delta * (2.0 * mills + t) - mills)
  };
  let h = 1.0 - delta;
  if !log_probability.is_finite()
    || !mills.is_finite()
    || !hprime.is_finite()
    || !h.is_finite()
    || h <= 0.0
    || h > 1.0
  {
    return Err(failed(
      "conditional moments are outside the represented numerical range",
    ));
  }
  Ok(Tail {
    probability,
    log_probability,
    mills,
    delta,
    h,
    hprime,
  })
}
fn conditional_mean(mu: f64, sigma: f64, lower: f64, t: f64, mills: f64) -> f64 {
  if t >= 0.0 {
    mu + sigma * mills
  } else {
    lower + sigma * (t + mills)
  }
}
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  lower: f64,
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = theta.len() - 1;
  let precision = theta[k];
  if theta.iter().any(|v| !v.is_finite()) || precision <= 0.0 {
    return Err(failed("canonical precision must be finite and positive"));
  }
  let sigma = 1.0 / precision.sqrt();
  let p = k + 1;
  let mut score = vec![0.0; p];
  let mut information = vec![vec![0.0; p]; p];
  let mut ll = 0.0;
  let mut correction = 0.0;
  for (row, &y) in x.iter().zip(y) {
    let mu = row.iter().zip(theta).map(|(a, b)| a * b).sum::<f64>() / precision;
    let t = (mu - lower) / sigma;
    let q = tail(t)?;
    let z = (y - mu) / sigma;
    let mean = conditional_mean(mu, sigma, lower, t, q.mills);
    let second = mu * mu
      + sigma * sigma
      + if q.mills == 0.0 {
        0.0
      } else {
        (mu + lower) * sigma * q.mills
      };
    let var = sigma * sigma * q.h;
    let cross = 2.0 * mu * var
      + if q.mills == 0.0 {
        0.0
      } else {
        sigma.powi(3) * (q.mills + t * q.delta)
      };
    let var_square = 4.0 * mu * mu * var
      + 2.0 * sigma.powi(4)
      + if q.mills == 0.0 {
        0.0
      } else {
        4.0 * mu * sigma.powi(3) * (q.mills + t * q.delta)
          - sigma.powi(4) * (t * q.mills + t * t * q.delta)
      };
    let contribution = 0.5 * precision.ln()
      - 0.5 * z * z
      - 0.5 * (2.0 * std::f64::consts::PI).ln()
      - q.log_probability;
    if [mu, mean, second, var, cross, var_square, contribution]
      .iter()
      .any(|v| !v.is_finite())
      || var_square <= 0.0
    {
      return Err(failed("non-finite conditional likelihood/moments"));
    }
    add_compensated(&mut ll, &mut correction, contribution);
    for i in 0..k {
      score[i] += row[i] * (y - mean);
      for j in 0..k {
        information[i][j] += row[i] * row[j] * var;
      }
      information[i][k] -= 0.5 * row[i] * cross;
      information[k][i] = information[i][k];
    }
    score[k] -= 0.5 * (y * y - second);
    information[k][k] += 0.25 * var_square;
  }
  if !ll.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed(
      "non-finite canonical likelihood, score or information",
    ));
  }
  Ok(Evaluation {
    log_likelihood: ll,
    score,
    information,
  })
}
fn optimize(
  x: &[Vec<f64>],
  y: &[f64],
  lower: f64,
  options: &TruncatedGaussianOptions,
) -> Result<Solution, StatsError> {
  let mut initial = vec![0.0; x[0].len() + 1];
  *initial.last_mut().unwrap() = 1.0;
  crate::gaussian_mle::maximize(
    initial,
    y.len(),
    options.max_iterations,
    "truncreg",
    |theta| evaluate(x, y, lower, theta),
  )
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
/// Fit on the explicit joint-finite sample. Every retained y must be STRICTLY
/// above the known bound; excluded/missing rows contribute no censoring mass.
pub fn fit_truncated_gaussian(
  problem: &EstimationProblem,
  lower_limit: f64,
  options: &TruncatedGaussianOptions,
) -> Result<TruncatedGaussianResult, StatsError> {
  problem.validate()?;
  if !lower_limit.is_finite() || options.max_iterations == 0 {
    return Err(invalid(
      "lower limit must be finite and iteration limit positive",
    ));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|s| s.weights.is_some() || s.cluster_groups.is_some())
  {
    return Err(invalid(
      "only unweighted IID model-based inference is implemented",
    ));
  }
  let n = problem.observation_count();
  let k = problem.parameter_count();
  let p = k + 1;
  if n > MAX_TRUNCATED_GAUSSIAN_OBSERVATIONS
    || k > MAX_TRUNCATED_GAUSSIAN_PARAMETERS
    || n <= p
    || n
      .checked_mul(p * p)
      .is_none_or(|v| v > MAX_TRUNCATED_GAUSSIAN_WORK)
  {
    return Err(invalid(
      "requires N<=10000, 1<=K<=32, N>K+1 and N*(K+1)^2<=10000000",
    ));
  }
  if let Some(sample) = &problem.sample {
    let unique: BTreeSet<_> = sample.retained_indices.iter().copied().collect();
    if sample.retained_indices.len() != n
      || unique.len() != n
      || sample.total_observations < n
      || sample.dropped_observations != sample.total_observations - n
      || sample
        .retained_indices
        .iter()
        .any(|i| *i >= sample.total_observations)
    {
      return Err(invalid(
        "sample identities must be unique original positions with consistent candidate/drop counts",
      ));
    }
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k || names.iter().any(|n| n == "lnsigma") {
    return Err(invalid(
      "parameter names must be unique and not reserved lnsigma",
    ));
  }
  if problem.response.iter().any(|y| *y <= lower_limit) {
    return Err(invalid(
      "all retained outcomes must be strictly above the truncation limit",
    ));
  }
  let original_x = full_design(problem);
  let scales: Vec<f64> = (0..k)
    .map(|j| original_x.iter().fold(0.0_f64, |a, row| a.hypot(row[j])) / (n as f64).sqrt())
    .collect();
  if scales.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(invalid("zero or unrepresentable predictor scale"));
  }
  let x: Vec<Vec<f64>> = original_x
    .iter()
    .map(|row| row.iter().zip(&scales).map(|(v, s)| v / s).collect())
    .collect();
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  // An inactive extreme lower bound must not destroy effective response units.
  // Centering is valid only with an intercept; otherwise zero location is fixed.
  let origin = if problem.include_intercept {
    problem
      .response
      .iter()
      .copied()
      .fold(f64::INFINITY, f64::min)
  } else {
    0.0
  };
  let centered: Vec<f64> = problem.response.iter().map(|y| y - origin).collect();
  let response_scale = centered.iter().map(|v| v.abs()).fold(0.0, f64::max);
  if !response_scale.is_finite() || response_scale <= 0.0 {
    return Err(invalid("degenerate or unrepresentable response scale"));
  }
  let y: Vec<f64> = centered.iter().map(|v| v / response_scale).collect();
  let lower = (lower_limit - origin) / response_scale;
  if !lower.is_finite() {
    return Err(invalid(
      "normalized truncation limit is outside the represented range",
    ));
  }
  let solution = optimize(&x, &y, lower, options)?;
  let precision = solution.theta[k];
  let sigma_scaled = 1.0 / precision.sqrt();
  let b: Vec<f64> = solution.theta[..k].iter().map(|v| v / precision).collect();
  // At the certified solution, evaluate the actual beta/logsigma OBSERVED
  // Hessian, not an OPG or a conditional-on-scale block. This also avoids relying
  // on a zero-score Jacobian approximation in the reported covariance.
  let mut info = vec![vec![0.0; p]; p];
  for (row, &y) in x.iter().zip(&y) {
    let mu = row.iter().zip(&b).map(|(v, b)| v * b).sum::<f64>();
    let t = (mu - lower) / sigma_scaled;
    let q = tail(t)?;
    let z = (y - mu) / sigma_scaled;
    let cross = (2.0 * z - q.mills + if q.mills == 0.0 { 0.0 } else { t * q.delta }) / sigma_scaled;
    for i in 0..k {
      for j in 0..k {
        info[i][j] += q.h * row[i] * row[j] / sigma_scaled.powi(2);
      }
      info[i][k] += row[i] * cross;
      info[k][i] = info[i][k];
    }
    info[k][k] += 2.0 * z * z
      + if q.mills == 0.0 {
        0.0
      } else {
        t * q.mills - t * t * q.delta
      };
  }
  let inverse = choice_information_inverse(&info)
    .map_err(|_| failed("nonpositive or degenerate full observed beta/logsigma information"))?;
  let factors: Vec<f64> = scales
    .iter()
    .map(|s| response_scale / s)
    .chain(std::iter::once(1.0))
    .collect();
  let covariance: Vec<Vec<f64>> = (0..p)
    .map(|i| {
      (0..p)
        .map(|j| inverse[i][j] * factors[i] * factors[j])
        .collect()
    })
    .collect();
  let mut beta: Vec<f64> = b.iter().zip(&factors).map(|(b, s)| b * s).collect();
  if problem.include_intercept {
    beta[0] += origin;
  }
  let sigma = response_scale * sigma_scaled;
  let log_sigma = sigma.ln();
  if !sigma.is_finite()
    || sigma <= 0.0
    || !log_sigma.is_finite()
    || beta
      .iter()
      .chain(covariance.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed("non-finite natural parameters or covariance"));
  }
  let coefficients = names
    .into_iter()
    .enumerate()
    .map(|(j, name)| inference(name, beta[j], covariance[j][j]))
    .collect::<Result<Vec<_>, _>>()?;
  let log_standard_error = covariance[k][k].sqrt();
  let scale = TobitScale {
    value: sigma,
    log_value: log_sigma,
    standard_error: sigma * log_standard_error,
    log_standard_error,
    confidence_lower: (log_sigma - NORMAL_95 * log_standard_error).exp(),
    confidence_upper: (log_sigma + NORMAL_95 * log_standard_error).exp(),
  };
  if [
    scale.log_standard_error,
    scale.standard_error,
    scale.confidence_lower,
    scale.confidence_upper,
  ]
  .iter()
  .any(|v| !v.is_finite() || *v <= 0.0)
  {
    return Err(failed("positive-scale log-Wald inference is unavailable"));
  }
  let null_x = vec![vec![1.0; usize::from(problem.include_intercept)]; n];
  let null = optimize(&null_x, &y, lower, options)?;
  let full_ll = solution.evaluation.log_likelihood;
  let null_ll = null.evaluation.log_likelihood;
  let difference = full_ll - null_ll;
  if difference < -64.0 * f64::EPSILON * (1.0 + full_ll.abs() + null_ll.abs()) {
    return Err(failed(
      "fitted likelihood is below its refitted restriction",
    ));
  }
  let lr = 2.0 * difference.max(0.0);
  let df = problem.predictor_names.len();
  let lr_p = (df > 0).then(|| chi_square_tail(lr, df));
  if lr_p.is_some_and(|v| !v.is_finite()) {
    return Err(failed("likelihood-ratio reference is non-finite"));
  }
  let mut result = TruncatedGaussianResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    lower_limit,
    observation_count: n,
    coefficients,
    scale,
    covariance,
    log_likelihood: full_ll - n as f64 * response_scale.ln(),
    null_log_likelihood: null_ll - n as f64 * response_scale.ln(),
    likelihood_ratio: lr,
    likelihood_ratio_df: df,
    likelihood_ratio_p_value: lr_p,
    iterations: solution.iterations,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_latent_values: Vec::with_capacity(n),
    fitted_retention_probabilities: Vec::with_capacity(n),
    fitted_conditional_means: Vec::with_capacity(n),
    mean_conditional_derivative: 0.0,
    mean_conditional_derivative_gradient: vec![0.0; p],
    sample: problem.sample.clone(),
  };
  for (row, full) in problem.design_matrix.iter().zip(&original_x) {
    let mu = truncated_gaussian_predict_row(&result, row, TruncatedGaussianPredictionKind::Latent)?;
    let t = (mu - lower_limit) / sigma;
    let q = tail(t)?;
    let mean = conditional_mean(mu, sigma, lower_limit, t, q.mills);
    if !mean.is_finite() || mean <= lower_limit {
      return Err(failed(
        "conditional mean is outside the represented support",
      ));
    }
    result.fitted_latent_values.push(mu);
    result.fitted_retention_probabilities.push(q.probability);
    result.fitted_conditional_means.push(mean);
    result.mean_conditional_derivative += q.h;
    for j in 0..k {
      result.mean_conditional_derivative_gradient[j] += q.hprime * full[j] / sigma;
    }
    result.mean_conditional_derivative_gradient[k] -=
      if q.mills == 0.0 { 0.0 } else { t * q.hprime };
  }
  result.mean_conditional_derivative /= n as f64;
  for g in &mut result.mean_conditional_derivative_gradient {
    *g /= n as f64;
  }
  if result
    .mean_conditional_derivative_gradient
    .iter()
    .any(|v| !v.is_finite())
  {
    return Err(failed("non-finite retained conditional-mean gradient"));
  }
  Ok(result)
}
/// Current X only. xb/pr do not require an unnecessary Mills division. pr means
/// underlying-population retention, not probability1 conditional on observation.
pub fn truncated_gaussian_predict_row(
  result: &TruncatedGaussianResult,
  predictors: &[f64],
  kind: TruncatedGaussianPredictionKind,
) -> Result<f64, StatsError> {
  if predictors.len() != result.predictor_names.len()
    || predictors.iter().any(|v| !v.is_finite())
    || result.coefficients.len() != predictors.len() + usize::from(result.include_intercept)
    || !result.scale.value.is_finite()
    || result.scale.value <= 0.0
    || !result.lower_limit.is_finite()
  {
    return Err(invalid(
      "prediction requires conformable finite predictors and a positive fitted scale",
    ));
  }
  let mu = std::iter::repeat_n(1.0, usize::from(result.include_intercept))
    .chain(predictors.iter().copied())
    .zip(&result.coefficients)
    .map(|(x, b)| x * b.value)
    .sum::<f64>();
  if !mu.is_finite() {
    return Err(failed("non-finite latent prediction"));
  }
  if kind == TruncatedGaussianPredictionKind::Latent {
    return Ok(mu);
  }
  let t = (mu - result.lower_limit) / result.scale.value;
  if kind == TruncatedGaussianPredictionKind::RetentionProbability {
    let p = normal_cdf(t);
    return if p.is_finite() {
      Ok(p)
    } else {
      Err(failed("non-finite retention probability"))
    };
  }
  let q = tail(t)?;
  let mean = conditional_mean(mu, result.scale.value, result.lower_limit, t, q.mills);
  if !mean.is_finite() || mean <= result.lower_limit {
    return Err(failed(
      "conditional mean is outside the represented support",
    ));
  }
  Ok(mean)
}
/// Retained-X continuous conditional-mean AMEs, full joint beta/logsigma delta.
pub fn truncated_gaussian_average_marginal_effects(
  result: &TruncatedGaussianResult,
) -> Result<TruncatedGaussianMarginalEffects, StatsError> {
  let k = result.coefficients.len();
  let p = k + 1;
  if result.observation_count == 0
    || k != result.predictor_names.len() + usize::from(result.include_intercept)
    || result.covariance.len() != p
    || result.covariance.iter().any(|r| r.len() != p)
    || result.mean_conditional_derivative_gradient.len() != p
    || !result.mean_conditional_derivative.is_finite()
    || !(0.0..=1.0).contains(&result.mean_conditional_derivative)
    || result
      .covariance
      .iter()
      .flatten()
      .chain(&result.mean_conditional_derivative_gradient)
      .any(|v| !v.is_finite())
  {
    return Err(invalid(
      "effects require finite conformable retained fit summaries/covariance",
    ));
  }
  let mut effects = Vec::new();
  for (column, name) in result.predictor_names.iter().enumerate() {
    let j = column + usize::from(result.include_intercept);
    let b = result.coefficients[j].value;
    let mut g: Vec<f64> = result
      .mean_conditional_derivative_gradient
      .iter()
      .map(|v| b * v)
      .collect();
    g[j] += result.mean_conditional_derivative;
    let variance = (0..p)
      .map(|a| {
        (0..p)
          .map(|b| g[a] * result.covariance[a][b] * g[b])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(
      name.clone(),
      b * result.mean_conditional_derivative,
      variance,
    )?);
  }
  Ok(TruncatedGaussianMarginalEffects {
    observation_count: result.observation_count,
    lower_limit: result.lower_limit,
    effects,
  })
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn canonical_score_and_sufficient_statistic_information_match_finite_differences() {
    let x = vec![
      vec![1.0, -1.5],
      vec![1.0, -0.5],
      vec![1.0, 0.3],
      vec![1.0, 1.1],
      vec![1.0, 2.0],
    ];
    let y = [-0.2, 0.1, 0.3, 1.3, 2.0];
    let theta = [0.2, -0.3, 1.4];
    let e = evaluate(&x, &y, -0.8, &theta).unwrap();
    for j in 0..3 {
      let mut plus = theta;
      let mut minus = theta;
      plus[j] += 1e-5;
      minus[j] -= 1e-5;
      let a = evaluate(&x, &y, -0.8, &plus).unwrap();
      let b = evaluate(&x, &y, -0.8, &minus).unwrap();
      assert!(((a.log_likelihood - b.log_likelihood) / 2e-5 - e.score[j]).abs() < 1e-8);
      for i in 0..3 {
        assert!((-(a.score[i] - b.score[i]) / 2e-5 - e.information[i][j]).abs() < 1e-8);
      }
    }
  }
}
