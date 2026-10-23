//! Bounded unweighted multinomial logit with a fixed reference category.
//! All coefficient inference uses the full joint model-based Fisher information.

use crate::distributions::chi_square_tail;
use crate::matrix::{choice_information_inverse, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, LogitEstimate, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const NORMAL_95: f64 = 1.959963984540054;
const TOLERANCE: f64 = 1e-10;
const MAX_CODE: f64 = 9_007_199_254_740_991.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultinomialOptions {
  pub base_outcome: Option<i64>,
  pub max_iterations: usize,
}

impl Default for MultinomialOptions {
  fn default() -> Self {
    Self {
      base_outcome: None,
      max_iterations: 100,
    }
  }
}

/// A category-specific coefficient or continuous average derivative. Coefficient
/// vectors exclude the base category; effects include every category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialEstimate {
  pub category: i64,
  #[serde(flatten)]
  pub estimate: LogitEstimate,
}

/// Category-major nonbase coefficients, each block intercept then predictors.
/// Covariance rows/columns follow this exact coefficient vector. No N-K scaling,
/// robust/cluster covariance or finite-sample t/F reference is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialLogitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub category_levels: Vec<i64>,
  pub base_outcome: i64,
  pub category_counts: Vec<usize>,
  pub observation_count: usize,
  pub coefficients: Vec<MultinomialEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub mcfadden_r_squared: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  /// One row per retained observation, columns in category_levels order.
  pub fitted_probabilities: Vec<Vec<f64>>,
  pub sample: Option<EstimationSample>,
}

/// Conditional-on-retained-X continuous AMEs, not at-means effects or discrete
/// changes. Each normal95 interval uses the full joint coefficient covariance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialMarginalEffects {
  pub observation_count: usize,
  pub category_levels: Vec<i64>,
  pub base_outcome: i64,
  pub effects: Vec<MultinomialEstimate>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("mlogit: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("mlogit: {message}"))
}
fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

/// Numeric outcomes are integer codes, not ordinal scores. Exclude values outside
/// the exact f64 integer domain instead of conflating large adjacent integers.
pub fn multinomial_outcome_code(value: f64) -> Result<i64, StatsError> {
  if !value.is_finite() || value.fract() != 0.0 || value.abs() > MAX_CODE {
    return Err(invalid(
      "outcomes must be finite integer codes within +/- (2^53-1)",
    ));
  }
  Ok(value as i64)
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

// eta includes an exact zero for the base. Log probabilities are evaluated by
// log-sum-exp, not log of a probability that may have underflowed to zero.
fn softmax(eta: &[f64]) -> Result<(Vec<f64>, f64, f64), StatsError> {
  if eta.iter().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite linear index"));
  }
  let center = eta.iter().copied().fold(f64::NEG_INFINITY, f64::max);
  let mut weights = eta.iter().map(|v| (v - center).exp()).collect::<Vec<_>>();
  let sum = weights.iter().sum::<f64>();
  for p in &mut weights {
    *p /= sum;
  }
  Ok((weights, center, sum.ln()))
}

struct Evaluation {
  probabilities: Vec<Vec<f64>>,
  log_likelihood: f64,
  /// Per-observation score and information; scaling makes the rank/convergence
  /// bounds independent of N. The final covariance divides their inverse by N.
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
}

fn evaluate(
  x: &[Vec<f64>],
  y: &[usize],
  category_count: usize,
  nonbase: &[usize],
  beta: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = x[0].len();
  let p = beta.len();
  let n = x.len() as f64;
  let mut score = vec![0.0; p];
  let mut information = vec![vec![0.0; p]; p];
  let mut probabilities = Vec::with_capacity(x.len());
  let mut log_likelihood = 0.0;
  for (row, &outcome) in x.iter().zip(y) {
    let mut eta = vec![0.0; category_count];
    for (a, &category) in nonbase.iter().enumerate() {
      eta[category] = row
        .iter()
        .zip(&beta[a * k..(a + 1) * k])
        .map(|(x, b)| x * b)
        .sum();
    }
    let (probs, center, log_sum) = softmax(&eta)?;
    log_likelihood += (eta[outcome] - center) - log_sum;
    for (a, &category) in nonbase.iter().enumerate() {
      let residual = if outcome == category {
        probs
          .iter()
          .enumerate()
          .filter(|(j, _)| *j != category)
          .map(|(_, p)| p)
          .sum()
      } else {
        -probs[category]
      };
      for s in 0..k {
        score[a * k + s] += row[s] * residual / n;
      }
      for (b, &other) in nonbase.iter().enumerate().take(a + 1) {
        let weight = if category == other {
          probs[category]
            * probs
              .iter()
              .enumerate()
              .filter(|(j, _)| *j != category)
              .map(|(_, p)| p)
              .sum::<f64>()
        } else {
          -probs[category] * probs[other]
        };
        for s in 0..k {
          for t in 0..k {
            information[a * k + s][b * k + t] += weight * (row[s] * row[t]) / n;
          }
        }
      }
    }
    probabilities.push(probs);
  }
  // The category-block lower triangle was accumulated; diagonal blocks already
  // contain both predictor triangles. Mirror only off-diagonal category blocks.
  for a in 0..nonbase.len() {
    for b in 0..a {
      for s in 0..k {
        for t in 0..k {
          information[b * k + t][a * k + s] = information[a * k + s][b * k + t];
        }
      }
    }
  }
  if !log_likelihood.is_finite()
    || score.iter().any(|v| !v.is_finite())
    || information.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(failed("nonfinite likelihood, score or information"));
  }
  Ok(Evaluation {
    probabilities,
    log_likelihood,
    score,
    information,
  })
}

fn information_inverse(information: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, StatsError> {
  choice_information_inverse(information).map_err(|error| match error {
    StatsError::EstimationFailed(message) => failed(&message),
    error => error,
  })
}

fn inference(
  category: i64,
  name: String,
  value: f64,
  variance: f64,
) -> Result<MultinomialEstimate, StatsError> {
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
  Ok(MultinomialEstimate {
    category,
    estimate: LogitEstimate {
      name,
      value,
      standard_error,
      statistic,
      p_value,
      confidence_lower,
      confidence_upper,
    },
  })
}

/// Fit an unweighted independent-observation MNL with fixed category reference.
/// Bounds: N<=10000, 2..16 levels, K<=32, P=(J-1)K<=128, N*P^2<=20million.
/// Failure to converge or identify the full joint information is an error.
pub fn fit_multinomial_logit(
  problem: &EstimationProblem,
  options: &MultinomialOptions,
) -> Result<MultinomialLogitResult, StatsError> {
  problem.validate()?;
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("weights are not supported"));
  }
  let codes = problem
    .response
    .iter()
    .map(|&v| multinomial_outcome_code(v))
    .collect::<Result<Vec<_>, _>>()?;
  let category_levels = codes
    .iter()
    .copied()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
  let j = category_levels.len();
  let n = codes.len();
  let k = problem.parameter_count();
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid(
      "parameter names must be unique within each category",
    ));
  }
  let p = j.saturating_sub(1) * k;
  if !(2..=16).contains(&j)
    || n > 10_000
    || k > 32
    || p > 128
    || n.saturating_mul(p.saturating_mul(p)) > 20_000_000
  {
    return Err(invalid(
      "requires 2..16 categories, N<=10000, K<=32, P<=128 and N*P^2<=20000000",
    ));
  }
  if n <= p {
    return Err(invalid("requires N greater than the joint parameter count"));
  }
  let base_outcome = options.base_outcome.unwrap_or(category_levels[0]);
  let base = category_levels
    .binary_search(&base_outcome)
    .map_err(|_| invalid("base outcome is absent from the retained sample"))?;
  let nonbase = (0..j).filter(|c| *c != base).collect::<Vec<_>>();
  let y = codes
    .iter()
    .map(|c| category_levels.binary_search(c).unwrap())
    .collect::<Vec<_>>();
  let mut category_counts = vec![0usize; j];
  for &c in &y {
    category_counts[c] += 1;
  }
  let mut x = full_design(problem);
  let mut scales = Vec::with_capacity(k);
  for s in 0..k {
    let max = x.iter().map(|r| r[s].abs()).fold(0.0, f64::max);
    if max == 0.0 {
      return Err(invalid("zero predictor column"));
    }
    let scale = max * (x.iter().map(|r| (r[s] / max).powi(2)).sum::<f64>() / n as f64).sqrt();
    if !scale.is_finite() || scale <= 0.0 {
      return Err(invalid("undefined predictor scale"));
    }
    scales.push(scale);
    for row in &mut x {
      row[s] /= scale;
    }
  }
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  let mut beta = vec![0.0; p];
  if problem.include_intercept {
    for (a, &c) in nonbase.iter().enumerate() {
      beta[a * k] = (category_counts[c] as f64 / category_counts[base] as f64).ln();
    }
  }
  let mut converged = None;
  for iteration in 0..options.max_iterations {
    let current = evaluate(&x, &y, j, &nonbase, &beta)?;
    let inverse = information_inverse(&current.information)?;
    let step = inverse
      .iter()
      .map(|row| row.iter().zip(&current.score).map(|(a, b)| a * b).sum())
      .collect::<Vec<f64>>();
    let score_bound = maximum_absolute(&current.score);
    let step_bound = maximum_absolute(&step);
    if step.iter().any(|v| !v.is_finite()) || !score_bound.is_finite() || !step_bound.is_finite() {
      return Err(failed("nonfinite Newton step"));
    }
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
      if let Ok(trial) = evaluate(&x, &y, j, &nonbase, &candidate)
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
  let values = beta
    .iter()
    .enumerate()
    .map(|(a, b)| b / scales[a % k])
    .collect::<Vec<_>>();
  let covariance = inverse
    .iter()
    .enumerate()
    .map(|(a, row)| {
      row
        .iter()
        .enumerate()
        .map(|(b, v)| (v / n as f64) / scales[a % k] / scales[b % k])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let mut coefficients = Vec::with_capacity(p);
  for (a, &c) in nonbase.iter().enumerate() {
    for s in 0..k {
      coefficients.push(inference(
        category_levels[c],
        names[s].clone(),
        values[a * k + s],
        covariance[a * k + s][a * k + s],
      )?);
    }
  }
  let null_log_likelihood = if problem.include_intercept {
    category_counts
      .iter()
      .map(|&count| count as f64 * (count as f64 / n as f64).ln())
      .sum()
  } else {
    -(n as f64) * (j as f64).ln()
  };
  let likelihood_ratio = (2.0 * (current.log_likelihood - null_log_likelihood)).max(0.0);
  let likelihood_ratio_df = (j - 1) * (k - usize::from(problem.include_intercept));
  let likelihood_ratio_p_value =
    (likelihood_ratio_df > 0).then(|| chi_square_tail(likelihood_ratio, likelihood_ratio_df));
  Ok(MultinomialLogitResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    category_levels,
    base_outcome,
    category_counts,
    observation_count: n,
    coefficients,
    covariance,
    log_likelihood: current.log_likelihood,
    null_log_likelihood,
    likelihood_ratio,
    likelihood_ratio_df,
    likelihood_ratio_p_value,
    mcfadden_r_squared: 1.0 - current.log_likelihood / null_log_likelihood,
    iterations,
    max_abs_scaled_score,
    max_abs_scaled_newton_step,
    fitted_probabilities: current.probabilities,
    sample: problem.sample.clone(),
  })
}

fn result_layout(result: &MultinomialLogitResult) -> Result<(usize, Vec<usize>), StatsError> {
  let k = result.predictor_names.len() + usize::from(result.include_intercept);
  let levels = &result.category_levels;
  if k == 0
    || !(2..=16).contains(&levels.len())
    || levels.windows(2).any(|v| v[0] >= v[1])
    || !levels.contains(&result.base_outcome)
  {
    return Err(invalid("invalid retained category/design metadata"));
  }
  let nonbase = (0..levels.len())
    .filter(|&c| levels[c] != result.base_outcome)
    .collect::<Vec<_>>();
  if result.coefficients.len() != k * nonbase.len()
    || result
      .coefficients
      .iter()
      .any(|c| !c.estimate.value.is_finite())
  {
    return Err(invalid("invalid retained coefficient dimensions/values"));
  }
  for (a, &c) in nonbase.iter().enumerate() {
    if result.coefficients[a * k..(a + 1) * k]
      .iter()
      .any(|v| v.category != levels[c])
    {
      return Err(invalid("invalid retained coefficient category order"));
    }
  }
  Ok((k, nonbase))
}

/// Predict all category probabilities and reference-relative indices for one
/// current row of predictor columns. Current outcomes are deliberately unused.
pub fn multinomial_predict_row(
  result: &MultinomialLogitResult,
  predictors: &[f64],
) -> Result<(Vec<f64>, Vec<f64>), StatsError> {
  let (k, nonbase) = result_layout(result)?;
  if predictors.len() != result.predictor_names.len() || predictors.iter().any(|v| !v.is_finite()) {
    return Err(invalid(
      "prediction requires finite current predictors in retained order",
    ));
  }
  let row = std::iter::repeat_n(1.0, usize::from(result.include_intercept))
    .chain(predictors.iter().copied())
    .collect::<Vec<_>>();
  let mut eta = vec![0.0; result.category_levels.len()];
  for (a, &c) in nonbase.iter().enumerate() {
    eta[c] = row
      .iter()
      .zip(&result.coefficients[a * k..(a + 1) * k])
      .map(|(x, b)| x * b.estimate.value)
      .sum();
  }
  Ok((softmax(&eta)?.0, eta))
}

/// Retained-design continuous AMEs and analytic joint-covariance delta inference.
/// The caller supplies the exact fit problem; no empirical-X variance is added.
pub fn multinomial_marginal_effects(
  problem: &EstimationProblem,
  result: &MultinomialLogitResult,
) -> Result<MultinomialMarginalEffects, StatsError> {
  problem.validate()?;
  let (k, nonbase) = result_layout(result)?;
  let n = problem.response.len();
  let j = result.category_levels.len();
  let p = result.coefficients.len();
  if problem.weights.is_some()
    || problem.sample.as_ref().is_some_and(|s| s.weights.is_some())
    || problem.predictor_names != result.predictor_names
    || problem.include_intercept != result.include_intercept
    || problem.outcome != result.outcome
    || n != result.observation_count
    || result.covariance.len() != p
    || result
      .covariance
      .iter()
      .any(|row| row.len() != p || row.iter().any(|v| !v.is_finite()))
  {
    return Err(invalid(
      "margins require the retained fit design and joint covariance",
    ));
  }
  let mut beta = vec![vec![0.0; k]; j];
  for (a, &c) in nonbase.iter().enumerate() {
    for (s, value) in beta[c].iter_mut().enumerate() {
      *value = result.coefficients[a * k + s].estimate.value;
    }
  }
  let rows = full_design(problem);
  let mut effects = Vec::new();
  for (c, category_beta) in beta.iter().enumerate() {
    for (r, &category_slope) in category_beta
      .iter()
      .enumerate()
      .skip(usize::from(problem.include_intercept))
    {
      let mut value = 0.0;
      let mut gradient = vec![0.0; p];
      for (raw, row) in problem.design_matrix.iter().zip(&rows) {
        let (probs, _) = multinomial_predict_row(result, raw)?;
        let average_slope = (0..j).map(|l| probs[l] * beta[l][r]).sum::<f64>();
        let difference = category_slope - average_slope;
        value += probs[c] * difference / n as f64;
        for (a, &category) in nonbase.iter().enumerate() {
          for s in 0..k {
            let category_contrast = f64::from(c == category) - probs[category];
            let slope_indicator = f64::from(s == r);
            gradient[a * k + s] += probs[c]
              * (category_contrast * (slope_indicator + row[s] * difference)
                - row[s] * probs[category] * (beta[category][r] - average_slope))
              / n as f64;
          }
        }
      }
      let variance = gradient
        .iter()
        .enumerate()
        .map(|(a, g)| {
          g * result.covariance[a]
            .iter()
            .zip(&gradient)
            .map(|(v, h)| v * h)
            .sum::<f64>()
        })
        .sum();
      effects.push(inference(
        result.category_levels[c],
        problem.predictor_names[r - usize::from(problem.include_intercept)].clone(),
        value,
        variance,
      )?);
    }
  }
  Ok(MultinomialMarginalEffects {
    observation_count: n,
    category_levels: result.category_levels.clone(),
    base_outcome: result.base_outcome,
    effects,
  })
}
