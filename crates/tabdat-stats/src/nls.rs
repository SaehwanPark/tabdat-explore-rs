//! Unweighted explicit-formula NLS: guarded QR Gauss–Newton and conditional-mean
//! Gauss–Newton covariance. No likelihood, pseudoinverse or global-optimum claim.
#![allow(clippy::needless_range_loop)]

mod expression;
pub use expression::{NlsExpression, NlsModel, NlsRowEvaluation};

use crate::matrix::{
  add_compensated, choice_information_inverse, qr_decompose_with_response, solve_upper_triangular,
  student_t_pvalue, student_t_quantile, xtx_inverse_from_r,
};
use crate::{EstimationSample, StatsError};
use expression::Jet;
use serde::{Deserialize, Serialize};

pub const MAX_NLS_OBSERVATIONS: usize = 10_000;
pub const MAX_NLS_PARAMETERS: usize = 32;
pub const MAX_NLS_NODES: usize = 128;
pub const MAX_NLS_DEPTH: usize = 32;
pub const MAX_NLS_WORK: usize = 20_000_000;
pub const MAX_NLS_TOTAL_WORK: usize = 500_000_000;
const MAX_ITERATIONS: usize = 200;
const MAX_HALVINGS: usize = 60;
const CONVERGENCE: f64 = 1e-12;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NlsEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NlsResult {
  pub model: NlsModel,
  pub start_values: Vec<f64>,
  pub robust: bool,
  pub observation_count: usize,
  pub residual_degrees_of_freedom: usize,
  pub coefficients: Vec<NlsEstimate>,
  /// In model.parameter_names order. Bread is J'J, NOT the observed loss Hessian.
  pub covariance_matrix: Vec<Vec<f64>>,
  pub covariance_correction: f64,
  pub residual_sum_of_squares: f64,
  pub residual_variance: f64,
  pub root_mean_squared_error: f64,
  pub centered_r_squared: Option<f64>,
  pub iterations: usize,
  pub normalized_score: f64,
  pub relative_full_gauss_newton_step: f64,
  pub evaluation_work: usize,
  pub response_scale: f64,
  pub fitted_values: Vec<f64>,
  pub residuals: Vec<f64>,
  pub sample: EstimationSample,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("nl: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("nl: {message}"))
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut value, mut correction) = (0.0, 0.0);
  for next in values {
    add_compensated(&mut value, &mut correction, next);
  }
  value + correction
}

struct Budget {
  used: usize,
}
impl Budget {
  fn spend(&mut self, work: usize) -> Result<(), StatsError> {
    let next = self
      .used
      .checked_add(work)
      .filter(|v| *v <= MAX_NLS_TOTAL_WORK)
      .ok_or_else(|| failed("cumulative evaluation work limit exceeded"))?;
    self.used = next;
    Ok(())
  }
}

fn validate_sample(sample: &EstimationSample, n: usize) -> Result<(), StatsError> {
  if sample.total_observations > MAX_NLS_OBSERVATIONS
    || sample.retained_indices.len() != n
    || n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
    || sample.weights.is_some()
    || sample.cluster_groups.is_some()
    || sample
      .retained_indices
      .iter()
      .any(|i| *i >= sample.total_observations)
    || sample.retained_indices.windows(2).any(|w| w[0] >= w[1])
  {
    return Err(invalid(
      "sample must be ordered, unweighted, unclustered and within candidate bounds",
    ));
  }
  Ok(())
}

struct Evaluation {
  jets: Vec<Jet>,
  residuals: Vec<f64>,
  /// Normalized-response Jacobian columns, each divided by its own maximum.
  z: Vec<Vec<f64>>,
  column_scale: Vec<f64>,
  rss: f64,
}

fn evaluate(
  model: &NlsModel,
  theta: &[f64],
  response: &[f64],
  data: &[Vec<f64>],
  response_scale: f64,
) -> Result<Evaluation, StatsError> {
  let p = theta.len();
  let jets = data
    .iter()
    .map(|row| expression::jet(&model.expression, theta, row))
    .collect::<Result<Vec<_>, _>>()?;
  let residuals: Vec<_> = response
    .iter()
    .zip(&jets)
    .map(|(y, jet)| y / response_scale - jet.mean / response_scale)
    .collect();
  let rss = sum(residuals.iter().map(|r| r * r));
  if !rss.is_finite() || rss <= 0.0 {
    return Err(failed("zero or nonrepresentable residual variance"));
  }
  let column_scale: Vec<f64> = (0..p)
    .map(|j| {
      jets
        .iter()
        .map(|jet| (jet.gradient[j] / response_scale).abs())
        .fold(0.0, f64::max)
    })
    .collect();
  if column_scale.iter().any(|v| !v.is_finite() || *v == 0.0) {
    return Err(failed("zero or nonrepresentable Jacobian column"));
  }
  let z = jets
    .iter()
    .map(|jet| {
      (0..p)
        .map(|j| (jet.gradient[j] / response_scale) / column_scale[j])
        .collect()
    })
    .collect();
  Ok(Evaluation {
    jets,
    residuals,
    z,
    column_scale,
    rss,
  })
}

fn loss(
  model: &NlsModel,
  theta: &[f64],
  response: &[f64],
  data: &[Vec<f64>],
  response_scale: f64,
) -> Result<f64, StatsError> {
  let mut residuals = Vec::with_capacity(response.len());
  for (row, y) in data.iter().zip(response) {
    let mu = expression::value(&model.expression, theta, row)?;
    residuals.push(y / response_scale - mu / response_scale);
  }
  let rss = sum(residuals.iter().map(|r| r * r));
  if rss.is_finite() {
    Ok(rss)
  } else {
    Err(failed("nonrepresentable trial loss"))
  }
}

fn check_curvature(evaluation: &Evaluation, response_scale: f64) -> Result<(), StatsError> {
  let p = evaluation.column_scale.len();
  let n = evaluation.jets.len() as f64;
  let hessian: Vec<Vec<f64>> = (0..p)
    .map(|i| {
      (0..p)
        .map(|j| {
          sum(evaluation.jets.iter().enumerate().map(|(r, jet)| {
            evaluation.z[r][i] * evaluation.z[r][j]
              - ((evaluation.residuals[r] * (jet.hessian[i * p + j] / response_scale))
                / evaluation.column_scale[i])
                / evaluation.column_scale[j]
          }))
            / n
        })
        .collect()
    })
    .collect();
  // GN curvature is positive even at some stationary maxima. The observed
  // half-loss Hessian must independently pass the existing scaled SPD guard.
  choice_information_inverse(&hessian)
    .map_err(|_| failed("stationary point is not a numerically regular local minimum"))?;
  Ok(())
}

fn inference(
  name: String,
  value: f64,
  variance: f64,
  df: usize,
) -> Result<NlsEstimate, StatsError> {
  if !variance.is_finite() || variance < 0.0 {
    return Err(failed("invalid coefficient variance"));
  }
  let se = variance.sqrt();
  let statistic = value / se;
  let p = if se > 0.0 && statistic.is_finite() {
    student_t_pvalue(statistic, df as f64)
  } else {
    f64::NAN
  };
  let critical = student_t_quantile(0.975, df as f64);
  let lo = value - critical * se;
  let hi = value + critical * se;
  let available = se > 0.0 && [statistic, p, lo, hi].iter().all(|v| v.is_finite());
  Ok(NlsEstimate {
    name,
    value,
    standard_error: se,
    statistic: available.then_some(statistic),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lo),
    confidence_upper: available.then_some(hi),
  })
}

struct Solution {
  theta: Vec<f64>,
  evaluation: Evaluation,
  inverse_z: Vec<Vec<f64>>,
  iterations: usize,
  score: f64,
  step: f64,
  work: usize,
}

fn finish(
  model: &NlsModel,
  response: &[f64],
  response_scale: f64,
  sample: &EstimationSample,
  robust: bool,
  solution: Solution,
  start: &[f64],
) -> Result<NlsResult, StatsError> {
  let n = response.len();
  let p = solution.theta.len();
  let df = n - p;
  let evaluation = solution.evaluation;
  let rss = (evaluation.rss * response_scale) * response_scale;
  let variance = rss / df as f64;
  if !rss.is_finite() || !variance.is_finite() || variance <= 0.0 {
    return Err(failed("zero or nonrepresentable raw residual variance"));
  }
  let correction = if robust { n as f64 / df as f64 } else { 1.0 };
  let covariance_matrix = if robust {
    let influences: Vec<Vec<f64>> = evaluation
      .z
      .iter()
      .zip(&evaluation.residuals)
      .map(|(z, r)| {
        (0..p)
          .map(|i| {
            sum((0..p).map(|j| solution.inverse_z[i][j] * z[j] * r)) / evaluation.column_scale[i]
          })
          .collect()
      })
      .collect();
    (0..p)
      .map(|i| {
        (0..p)
          .map(|j| correction * sum(influences.iter().map(|u| u[i] * u[j])))
          .collect()
      })
      .collect::<Vec<Vec<f64>>>()
  } else {
    (0..p)
      .map(|i| {
        (0..p)
          .map(|j| {
            ((evaluation.rss / df as f64 * solution.inverse_z[i][j]) / evaluation.column_scale[i])
              / evaluation.column_scale[j]
          })
          .collect()
      })
      .collect()
  };
  if covariance_matrix.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("nonrepresentable covariance"));
  }
  let coefficients = model
    .parameter_names
    .iter()
    .enumerate()
    .map(|(i, name)| inference(name.clone(), solution.theta[i], covariance_matrix[i][i], df))
    .collect::<Result<Vec<_>, _>>()?;
  let mean_y = sum(response.iter().map(|y| y / response_scale)) / n as f64;
  let tss = sum(
    response
      .iter()
      .map(|y| (y / response_scale - mean_y).powi(2)),
  );
  let r_squared = (tss > 0.0).then(|| 1.0 - evaluation.rss / tss);
  if r_squared.is_some_and(|v| !v.is_finite()) {
    return Err(failed("nonrepresentable centered R squared"));
  }
  let fitted_values: Vec<_> = evaluation.jets.iter().map(|j| j.mean).collect();
  let residuals: Vec<_> = response
    .iter()
    .zip(&fitted_values)
    .map(|(y, mu)| y - mu)
    .collect();
  if residuals.iter().any(|v| !v.is_finite()) {
    return Err(failed("nonrepresentable raw residuals"));
  }
  Ok(NlsResult {
    model: model.clone(),
    start_values: start.to_vec(),
    robust,
    observation_count: n,
    residual_degrees_of_freedom: df,
    coefficients,
    covariance_matrix,
    covariance_correction: correction,
    residual_sum_of_squares: rss,
    residual_variance: variance,
    root_mean_squared_error: variance.sqrt(),
    centered_r_squared: r_squared,
    iterations: solution.iterations,
    normalized_score: solution.score,
    relative_full_gauss_newton_step: solution.step,
    evaluation_work: solution.work,
    response_scale,
    fitted_values,
    residuals,
    sample: sample.clone(),
  })
}

/// Fit explicit parameters of a correctly specified conditional mean. Classical
/// covariance additionally assumes homoskedasticity; HC1 uses GN, not observed-loss
/// bread. Sample identity is supplied separately because data columns are not model
/// parameters and no automatic intercept is inserted.
pub fn fit_nls(
  model: &NlsModel,
  response: &[f64],
  data: &[Vec<f64>],
  start: &[f64],
  sample: &EstimationSample,
  robust: bool,
) -> Result<NlsResult, StatsError> {
  let nodes = model.validate()?;
  let n = response.len();
  let p = model.parameter_names.len();
  validate_sample(sample, n)?;
  if n <= p || n > MAX_NLS_OBSERVATIONS || n * p * p * nodes > MAX_NLS_WORK {
    return Err(invalid(
      "requires N>P and observation/parameter/expression work within bounds",
    ));
  }
  if data.len() != n || response.iter().any(|v| !v.is_finite()) {
    return Err(invalid("response/data dimensions or finite values invalid"));
  }
  for row in data {
    model.validate_row(start, row)?;
  }
  let response_scale = response.iter().map(|y| y.abs()).fold(0.0, f64::max);
  let response_scale = if response_scale == 0.0 {
    1.0
  } else {
    response_scale
  };
  let value_work = n * nodes;
  let derivative_work = value_work * (1 + p + p * p);
  let mut budget = Budget { used: 0 };
  let mut theta = start.to_vec();
  for iterations in 0..=MAX_ITERATIONS {
    budget.spend(derivative_work)?;
    let evaluation = evaluate(model, &theta, response, data, response_scale)?;
    let qr = qr_decompose_with_response(&evaluation.z, &evaluation.residuals)?;
    let normalized_step = solve_upper_triangular(&qr.r, &qr.qty)?;
    let step: Vec<_> = normalized_step
      .iter()
      .zip(&evaluation.column_scale)
      .map(|(a, b)| a / b)
      .collect();
    let step_bound = step
      .iter()
      .zip(&theta)
      .map(|(a, b)| a.abs() / (1.0 + b.abs()))
      .fold(0.0, f64::max);
    let score = (0..p)
      .map(|j| {
        sum(
          evaluation
            .z
            .iter()
            .zip(&evaluation.residuals)
            .map(|(z, r)| z[j] * r),
        )
        .abs()
          / evaluation.rss.sqrt()
          / sum(evaluation.z.iter().map(|z| z[j] * z[j])).sqrt()
      })
      .fold(0.0, f64::max);
    if step.iter().any(|v| !v.is_finite()) || !score.is_finite() || !step_bound.is_finite() {
      return Err(failed("nonrepresentable score or full GN step"));
    }
    if score <= CONVERGENCE && step_bound <= CONVERGENCE {
      check_curvature(&evaluation, response_scale)?;
      let inverse_z = xtx_inverse_from_r(&qr.r)?;
      let solution = Solution {
        theta,
        evaluation,
        inverse_z,
        iterations,
        score,
        step: step_bound,
        work: budget.used,
      };
      return finish(
        model,
        response,
        response_scale,
        sample,
        robust,
        solution,
        start,
      );
    }
    if iterations == MAX_ITERATIONS {
      break;
    }
    let allowance = 32.0 * f64::EPSILON * (1.0 + evaluation.rss);
    let mut accepted = None;
    let mut alpha = 1.0;
    for _ in 0..MAX_HALVINGS {
      let candidate: Vec<_> = theta
        .iter()
        .zip(&step)
        .map(|(a, b)| a + alpha * b)
        .collect();
      if candidate.iter().all(|v| v.is_finite()) {
        budget.spend(value_work)?;
        if let Ok(rss) = loss(model, &candidate, response, data, response_scale)
          && rss <= evaluation.rss + allowance
        {
          accepted = Some(candidate);
          break;
        }
      }
      alpha *= 0.5;
    }
    theta =
      accepted.ok_or_else(|| failed("backtracking could not find a finite acceptable step"))?;
  }
  Err(failed(
    "iteration limit reached without score and full-step convergence",
  ))
}

#[cfg(test)]
mod tests;
