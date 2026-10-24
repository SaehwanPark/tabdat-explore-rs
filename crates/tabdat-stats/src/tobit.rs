//! IID homoskedastic Gaussian MLE with a fixed lower censoring limit.
//!
//! Optimization uses the concave delta=beta/sigma, gamma=response_scale/sigma
//! parameterization, not a possibly indefinite beta/logsigma Newton iteration.
//! Inference transforms the FULL joint observed inverse back to beta/logsigma.

#![allow(clippy::needless_range_loop)]

use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf};
use crate::matrix::{add_compensated, choice_information_inverse, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TobitOptions {
  pub max_iterations: usize,
}
impl Default for TobitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
    }
  }
}

/// Asymptotic normal inference; there is no finite-sample Student-t correction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TobitEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: f64,
  pub p_value: f64,
  pub confidence_lower: f64,
  pub confidence_upper: f64,
}

/// Positive scale and log-Wald95 interval, not a test of the excluded sigma=0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TobitScale {
  pub value: f64,
  pub log_value: f64,
  pub standard_error: f64,
  pub log_standard_error: f64,
  pub confidence_lower: f64,
  pub confidence_upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TobitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub lower_limit: f64,
  pub observation_count: usize,
  pub left_censored_count: usize,
  pub uncensored_count: usize,
  pub coefficients: Vec<TobitEstimate>,
  pub scale: TobitScale,
  /// Coefficient order (intercept first when present), then lnsigma; correction1.
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_latent_values: Vec<f64>,
  pub fitted_uncensored_probabilities: Vec<f64>,
  pub fitted_observed_means: Vec<f64>,
  /// Retained-X sufficient summaries for continuous observed-mean delta effects;
  /// no current-data re-evaluation or implicit refit is needed for those effects.
  pub mean_uncensored_probability: f64,
  pub mean_probability_gradient: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TobitMarginalEffects {
  pub observation_count: usize,
  pub lower_limit: f64,
  pub effects: Vec<TobitEstimate>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("tobit: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("tobit: {message}"))
}

fn inference(name: String, value: f64, variance: f64) -> Result<TobitEstimate, StatsError> {
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
    return Err(failed(
      "normal inference is non-finite or has nonpositive variance",
    ));
  }
  Ok(TobitEstimate {
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
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
}

fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  lower: f64,
  response_scale: f64,
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = theta.len() - 1;
  let gamma = theta[k];
  if !gamma.is_finite() || gamma <= 0.0 {
    return Err(failed("scale coordinate must be finite and positive"));
  }
  let p = k + 1;
  let mut score = vec![0.0; p];
  let mut information = vec![vec![0.0; p]; p];
  let mut log_likelihood = 0.0;
  let mut correction = 0.0;
  for (row, &outcome) in x.iter().zip(y) {
    let index: f64 = row.iter().zip(theta).map(|(a, b)| a * b).sum();
    let censored = outcome <= lower;
    let value = if censored { lower } else { outcome } / response_scale;
    let residual = value * gamma - index;
    let (contribution, first, curvature) = if censored {
      let probability = normal_cdf(residual);
      if !probability.is_finite() || probability <= 0.0 {
        return Err(failed(
          "censor probability is outside the represented numerical range",
        ));
      }
      // Positive-tail log1p avoids rounding log(Phi) to zero at saturation.
      let log_probability = if residual >= 0.0 {
        (-normal_cdf(-residual)).ln_1p()
      } else {
        probability.ln()
      };
      let mills = normal_pdf(residual) / probability;
      (log_probability, mills, mills * (residual + mills))
    } else {
      let constant = 0.5 * (2.0 * std::f64::consts::PI).ln();
      (
        gamma.ln() - response_scale.ln() - 0.5 * residual * residual - constant,
        -residual,
        1.0,
      )
    };
    if !contribution.is_finite() || !first.is_finite() || !curvature.is_finite() || curvature < 0.0
    {
      return Err(failed(
        "likelihood derivatives are outside the represented numerical range",
      ));
    }
    add_compensated(&mut log_likelihood, &mut correction, contribution);
    let direction: Vec<f64> = row
      .iter()
      .map(|v| -v)
      .chain(std::iter::once(value))
      .collect();
    for i in 0..p {
      score[i] += first * direction[i];
      for j in 0..p {
        information[i][j] += curvature * direction[i] * direction[j];
      }
    }
    if !censored {
      score[k] += 1.0 / gamma;
      information[k][k] += 1.0 / gamma.powi(2);
    }
  }
  if !log_likelihood.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed("non-finite likelihood, score or information"));
  }
  Ok(Evaluation {
    log_likelihood,
    score,
    information,
  })
}

struct Solution {
  theta: Vec<f64>,
  evaluation: Evaluation,
  inverse: Vec<Vec<f64>>,
  iterations: usize,
  score_bound: f64,
  step_bound: f64,
}

fn optimize(
  x: &[Vec<f64>],
  y: &[f64],
  lower: f64,
  response_scale: f64,
  options: &TobitOptions,
) -> Result<Solution, StatsError> {
  let p = x[0].len() + 1;
  let n = y.len() as f64;
  let mut theta = vec![0.0; p];
  theta[p - 1] = 1.0;
  for iteration in 1..=options.max_iterations {
    let e = evaluate(x, y, lower, response_scale, &theta)?;
    let average: Vec<Vec<f64>> = e
      .information
      .iter()
      .map(|row| row.iter().map(|v| v / n).collect())
      .collect();
    // Same positive-information check used by existing RMS-scaled choice ML:
    // no damping, ridge or pseudoinverse is allowed into the final covariance.
    let inverse = choice_information_inverse(&average)
      .map_err(|_| failed("singular or numerically degenerate joint information"))?;
    let score: Vec<f64> = e.score.iter().map(|v| v / n).collect();
    let step: Vec<f64> = inverse
      .iter()
      .map(|row| row.iter().zip(&score).map(|(a, b)| a * b).sum())
      .collect();
    let score_bound = score.iter().map(|v| v.abs()).fold(0.0, f64::max);
    let step_bound = step
      .iter()
      .zip(&theta)
      .map(|(d, t)| d.abs() / (1.0 + t.abs()))
      .fold(0.0, f64::max);
    if score_bound <= TOLERANCE && step_bound <= TOLERANCE {
      return Ok(Solution {
        theta,
        evaluation: e,
        inverse: inverse
          .iter()
          .map(|row| row.iter().map(|v| v / n).collect())
          .collect(),
        iterations: iteration,
        score_bound,
        step_bound,
      });
    }
    let mut alpha = 1.0;
    let mut accepted = None;
    for _ in 0..60 {
      let candidate: Vec<f64> = theta
        .iter()
        .zip(&step)
        .map(|(t, d)| t + alpha * d)
        .collect();
      if let Ok(next) = evaluate(x, y, lower, response_scale, &candidate)
        && next.log_likelihood
          >= e.log_likelihood - 32.0 * f64::EPSILON * (1.0 + e.log_likelihood.abs())
      {
        accepted = Some(candidate);
        break;
      }
      alpha *= 0.5;
    }
    theta = accepted.ok_or_else(|| failed("positive-scale Newton line search failed"))?;
  }
  Err(failed(
    "iteration limit reached without score/step convergence; no finite regular fit established",
  ))
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

/// Fit on the already-selected joint finite sample. Rows at/below the limit
/// contribute a probability at the LIMIT, not a density at their recorded value.
/// All-uncensored data are valid Gaussian MLE; all-censored data are unidentified.
pub fn fit_tobit(
  problem: &EstimationProblem,
  lower_limit: f64,
  options: &TobitOptions,
) -> Result<TobitResult, StatsError> {
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
  if n > 10_000 || k > 32 || n <= p || n.checked_mul(p * p).is_none_or(|work| work > 10_000_000) {
    return Err(invalid(
      "requires N<=10000, 1<=K<=32, N>K+1 and N*(K+1)^2<=10000000",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid("parameter names must be unique"));
  }
  let left_censored_count = problem
    .response
    .iter()
    .filter(|y| **y <= lower_limit)
    .count();
  if left_censored_count == n {
    return Err(invalid(
      "all observations are left-censored; no finite regular fit is identified",
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
    .map(|row| row.iter().zip(&scales).map(|(a, s)| a / s).collect())
    .collect();
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  let response_scale = problem
    .response
    .iter()
    .map(|y| y.abs())
    .fold(lower_limit.abs(), f64::max);
  if !response_scale.is_finite() || response_scale <= 0.0 {
    return Err(invalid("degenerate response scale"));
  }
  let solution = optimize(&x, &problem.response, lower_limit, response_scale, options)?;
  let gamma = solution.theta[k];
  let sigma = response_scale / gamma;
  let log_sigma = sigma.ln();
  let beta: Vec<f64> = (0..k)
    .map(|j| sigma * solution.theta[j] / scales[j])
    .collect();
  // FULL Jacobian: beta depends on the scale coordinate as well as delta.
  // Omitting beta/gamma cross terms would silently corrupt coefficient/AME SEs.
  let mut jacobian = vec![vec![0.0; p]; p];
  for j in 0..k {
    jacobian[j][j] = sigma / scales[j];
    jacobian[j][k] = -beta[j] / gamma;
  }
  jacobian[k][k] = -1.0 / gamma;
  let covariance: Vec<Vec<f64>> = (0..p)
    .map(|i| {
      (0..p)
        .map(|j| {
          (0..p)
            .map(|a| {
              (0..p)
                .map(|b| jacobian[i][a] * solution.inverse[a][b] * jacobian[j][b])
                .sum::<f64>()
            })
            .sum()
        })
        .collect()
    })
    .collect();
  if !sigma.is_finite()
    || sigma <= 0.0
    || !log_sigma.is_finite()
    || covariance.iter().flatten().any(|v| !v.is_finite())
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
  if log_standard_error <= 0.0
    || [
      log_standard_error,
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
  let null = optimize(
    &null_x,
    &problem.response,
    lower_limit,
    response_scale,
    options,
  )?;
  let difference = solution.evaluation.log_likelihood - null.evaluation.log_likelihood;
  let rounding = 64.0
    * f64::EPSILON
    * (1.0 + solution.evaluation.log_likelihood.abs() + null.evaluation.log_likelihood.abs());
  if difference < -rounding {
    return Err(failed(
      "fitted likelihood is below its refitted restriction",
    ));
  }
  let likelihood_ratio = 2.0 * difference.max(0.0);
  let likelihood_ratio_df = problem.predictor_names.len();
  let likelihood_ratio_p_value =
    (likelihood_ratio_df > 0).then(|| chi_square_tail(likelihood_ratio, likelihood_ratio_df));
  if likelihood_ratio_p_value.is_some_and(|v| !v.is_finite()) {
    return Err(failed("likelihood-ratio reference is non-finite"));
  }
  let mut result = TobitResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    lower_limit,
    observation_count: n,
    left_censored_count,
    uncensored_count: n - left_censored_count,
    coefficients,
    scale,
    covariance,
    log_likelihood: solution.evaluation.log_likelihood,
    null_log_likelihood: null.evaluation.log_likelihood,
    likelihood_ratio,
    likelihood_ratio_df,
    likelihood_ratio_p_value,
    iterations: solution.iterations,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_latent_values: Vec::with_capacity(n),
    fitted_uncensored_probabilities: Vec::with_capacity(n),
    fitted_observed_means: Vec::with_capacity(n),
    mean_uncensored_probability: 0.0,
    mean_probability_gradient: vec![0.0; p],
    sample: problem.sample.clone(),
  };
  for (row, full) in problem.design_matrix.iter().zip(&original_x) {
    let (latent, probability, mean) = tobit_predict_row(&result, row)?;
    result.fitted_latent_values.push(latent);
    result.fitted_uncensored_probabilities.push(probability);
    result.fitted_observed_means.push(mean);
    result.mean_uncensored_probability += probability;
    let z = (latent - lower_limit) / sigma;
    let density = normal_pdf(z);
    for (j, &value) in full.iter().enumerate() {
      result.mean_probability_gradient[j] += density * value / sigma;
    }
    result.mean_probability_gradient[k] -= z * density;
  }
  // Divide once: accumulating p/N can round a saturated mean above one.
  result.mean_uncensored_probability /= n as f64;
  for gradient in &mut result.mean_probability_gradient {
    *gradient /= n as f64;
  }
  if result
    .mean_probability_gradient
    .iter()
    .any(|v| !v.is_finite())
  {
    return Err(failed("retained probability gradient is non-finite"));
  }
  Ok(result)
}

/// Current-row (latent mean, P(Y*>lower|X), E[max(lower,Y*)|X]).
/// The observed outcome is deliberately not an input; missing-row handling and
/// atomic publication belong to the data/execution boundary.
pub fn tobit_predict_row(
  result: &TobitResult,
  predictors: &[f64],
) -> Result<(f64, f64, f64), StatsError> {
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
  let latent: f64 = std::iter::repeat_n(1.0, usize::from(result.include_intercept))
    .chain(predictors.iter().copied())
    .zip(&result.coefficients)
    .map(|(x, b)| x * b.value)
    .sum();
  let z = (latent - result.lower_limit) / result.scale.value;
  let probability = normal_cdf(z);
  let mean = result.lower_limit + result.scale.value * (z * probability + normal_pdf(z));
  if [latent, z, probability, mean]
    .iter()
    .any(|v| !v.is_finite())
    || mean < result.lower_limit
  {
    return Err(failed(
      "prediction is non-finite or outside the represented censor-mean range",
    ));
  }
  Ok((latent, probability, mean))
}

/// Continuous derivatives of the observed censored mean, averaged over retained
/// X. The delta gradient uses the full beta/lnsigma covariance and the stored
/// retained-X probability-gradient summaries, not the current dataset.
pub fn tobit_average_marginal_effects(
  result: &TobitResult,
) -> Result<TobitMarginalEffects, StatsError> {
  let k = result.coefficients.len();
  let p = k + 1;
  if result.observation_count == 0
    || result.mean_probability_gradient.len() != p
    || result.covariance.len() != p
    || result.covariance.iter().any(|row| row.len() != p)
    || result
      .covariance
      .iter()
      .flatten()
      .chain(&result.mean_probability_gradient)
      .any(|v| !v.is_finite())
    || !result.mean_uncensored_probability.is_finite()
    || !(0.0..=1.0).contains(&result.mean_uncensored_probability)
    || k != result.predictor_names.len() + usize::from(result.include_intercept)
  {
    return Err(invalid(
      "effects require finite conformable retained fit summaries/covariance",
    ));
  }
  let mut effects = Vec::new();
  for (column, name) in result.predictor_names.iter().enumerate() {
    let j = column + usize::from(result.include_intercept);
    let beta = result.coefficients[j].value;
    let mut gradient: Vec<f64> = result
      .mean_probability_gradient
      .iter()
      .map(|g| beta * g)
      .collect();
    gradient[j] += result.mean_uncensored_probability;
    let variance = (0..p)
      .map(|a| {
        (0..p)
          .map(|b| gradient[a] * result.covariance[a][b] * gradient[b])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(
      name.clone(),
      beta * result.mean_uncensored_probability,
      variance,
    )?);
  }
  Ok(TobitMarginalEffects {
    observation_count: result.observation_count,
    lower_limit: result.lower_limit,
    effects,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn concave_coordinate_full_score_and_information_match_finite_differences() {
    let x = vec![
      vec![1.0, -1.5],
      vec![1.0, -0.5],
      vec![1.0, 0.3],
      vec![1.0, 1.1],
      vec![1.0, 2.0],
    ];
    let y = [-2.0, -0.8, 0.2, 1.5, 2.7];
    let theta = [0.2, -0.3, 1.4];
    let e = evaluate(&x, &y, -0.8, 2.7, &theta).unwrap();
    for j in 0..3 {
      let h = 1e-5;
      let mut plus = theta;
      let mut minus = theta;
      plus[j] += h;
      minus[j] -= h;
      let a = evaluate(&x, &y, -0.8, 2.7, &plus).unwrap();
      let b = evaluate(&x, &y, -0.8, 2.7, &minus).unwrap();
      let gradient = (a.log_likelihood - b.log_likelihood) / (2.0 * h);
      assert!((gradient - e.score[j]).abs() < 1e-8);
      for i in 0..3 {
        let information = -(a.score[i] - b.score[i]) / (2.0 * h);
        assert!((information - e.information[i][j]).abs() < 1e-8);
      }
    }
  }
}
