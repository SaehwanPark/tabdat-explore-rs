//! Entry-zero, single-spell Weibull PH likelihood with exact positive times.
//! Optimization is jointly concave in beta/natural shape, not beta/logshape.
//! Inference uses the FULL joint observed information, including shape cross terms.

#![allow(clippy::needless_range_loop)]

use crate::binary::{BinaryOmnibusWald, omnibus_wald};
use crate::distributions::chi_square_tail;
use crate::gaussian_mle::{Evaluation, Solution};
use crate::matrix::{add_compensated, choice_information_inverse, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_WEIBULL_OBSERVATIONS: usize = 10_000;
pub const MAX_WEIBULL_PARAMETERS: usize = 64;
pub const MAX_WEIBULL_WORK: usize = 20_000_000;
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeibullOptions {
  pub max_iterations: usize,
  pub robust: bool,
}
impl Default for WeibullOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
      robust: false,
    }
  }
}

/// Fixed normal95 inference; zero/unrepresentable inference is NULL, not p=0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeibullEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

/// Positive shape; log-scale normal test is shape=1, not shape=0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeibullShape {
  pub value: f64,
  pub log_value: f64,
  pub standard_error: f64,
  pub log_standard_error: f64,
  pub log_statistic: Option<f64>,
  pub log_p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

/// Shared rank-aware coefficient-only chi-square contract, without family refit.
pub type WeibullOmnibusWald = BinaryOmnibusWald;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeibullResult {
  pub time_variable: String,
  pub failure_variable: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub robust: bool,
  pub observation_count: usize,
  pub failure_count: usize,
  pub censored_count: usize,
  pub coefficients: Vec<WeibullEstimate>,
  pub shape: WeibullShape,
  /// PH beta order, then lnshape; FULL observed inverse or row-score sandwich.
  pub covariance_matrix: Vec<Vec<f64>>,
  pub covariance_correction: f64,
  /// Ordinary duration density, not Stata's event-log-time normalization.
  pub log_likelihood: f64,
  pub normalized_log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub null_normalized_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub omnibus_wald: WeibullOmnibusWald,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_xb: Vec<f64>,
  pub fitted_hazard: Vec<f64>,
  pub fitted_cumulative_hazard: Vec<f64>,
  pub fitted_survival: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeibullPredictionKind {
  Xb,
  Hazard,
  CumulativeHazard,
  Survival,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("weibull: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("weibull: {message}"))
}

fn inference(name: String, value: f64, variance: f64) -> Result<WeibullEstimate, StatsError> {
  if !value.is_finite() || !variance.is_finite() || variance < 0.0 {
    return Err(failed(
      "non-finite coefficient or negative/non-finite variance",
    ));
  }
  let se = variance.sqrt();
  let z = value / se;
  let p = chi_square_tail(z * z, 1);
  let lo = value - NORMAL_95 * se;
  let hi = value + NORMAL_95 * se;
  let available = se > 0.0 && [z, p, lo, hi].iter().all(|v| v.is_finite());
  Ok(WeibullEstimate {
    name,
    value,
    standard_error: se,
    statistic: available.then_some(z),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lo),
    confidence_upper: available.then_some(hi),
  })
}

// Columns are RMS-scaled X and log(time); the final coordinate gamma=a*log_scale
// is positive. No Hessian damping or shape clipping enters reported inference.
fn evaluate(
  x: &[Vec<f64>],
  log_time: &[f64],
  events: &[f64],
  log_scale: f64,
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let p = theta.len();
  let k = p - 1;
  let gamma = theta[k];
  if gamma <= 0.0 || theta.iter().any(|v| !v.is_finite()) {
    return Err(failed(
      "natural shape must be positive and all parameters finite",
    ));
  }
  let mut ll = 0.0;
  let mut llc = 0.0;
  let mut score = vec![0.0; p];
  let mut sc = vec![0.0; p];
  let mut information = vec![vec![0.0; p]; p];
  let mut ic = vec![vec![0.0; p]; p];
  for ((row, &t), &d) in x.iter().zip(log_time).zip(events) {
    let l = t / log_scale;
    let eta: f64 = row.iter().zip(theta).map(|(v, b)| v * b).sum();
    let log_h = eta + gamma * l;
    let h = log_h.exp();
    if !log_h.is_finite() || !h.is_finite() {
      return Err(failed(
        "unrepresentable integrated hazard during optimization",
      ));
    }
    add_compensated(
      &mut ll,
      &mut llc,
      d * (gamma.ln() - log_scale.ln() + log_h) - h,
    );
    let mut z = row.clone();
    z.push(l);
    for i in 0..p {
      add_compensated(&mut score[i], &mut sc[i], z[i] * (d - h));
      for j in 0..=i {
        add_compensated(&mut information[i][j], &mut ic[i][j], h * z[i] * z[j]);
      }
    }
    add_compensated(&mut score[k], &mut sc[k], d / gamma);
    add_compensated(&mut information[k][k], &mut ic[k][k], d / gamma / gamma);
  }
  for i in 0..p {
    for j in 0..i {
      information[j][i] = information[i][j];
    }
  }
  if !ll.is_finite()
    || score.iter().any(|v| !v.is_finite())
    || information.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(failed(
      "unrepresentable likelihood, score or observed information",
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
  log_time: &[f64],
  events: &[f64],
  log_scale: f64,
  intercept: bool,
  options: &WeibullOptions,
) -> Result<Solution, StatsError> {
  let k = x[0].len();
  // This start bounds a*log(t), even for extreme finite time units. The
  // conditional intercept enforces sum H=sum d without summing raw durations.
  let a = 1.0 / log_time.iter().map(|t| t.abs()).fold(1.0, f64::max);
  let mut theta = vec![0.0; k + 1];
  theta[k] = a * log_scale;
  if intercept {
    let m = log_time
      .iter()
      .map(|t| a * t)
      .fold(f64::NEG_INFINITY, f64::max);
    let log_sum = m + log_time.iter().map(|t| (a * t - m).exp()).sum::<f64>().ln();
    theta[0] = (events.iter().sum::<f64>().ln() - log_sum) / x[0][0];
  }
  crate::gaussian_mle::maximize(theta, x.len(), options.max_iterations, "weibull", |theta| {
    evaluate(x, log_time, events, log_scale, theta)
  })
}

/// Fit an entry-zero, single-spell Weibull PH model. The problem response is
/// observed time, and `events` is the same retained sample's exact0/1 indicator.
/// Weight/cluster/sample validation happens before any optimization or effects.
pub fn fit_weibull(
  problem: &EstimationProblem,
  failure_variable: &str,
  events: &[f64],
  options: &WeibullOptions,
) -> Result<WeibullResult, StatsError> {
  problem.validate()?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  let p = k + 1;
  if options.max_iterations == 0 || failure_variable.trim().is_empty() {
    return Err(invalid(
      "iteration limit must be positive and failure name nonempty",
    ));
  }
  if events.len() != n
    || events.iter().any(|d| *d != 0.0 && *d != 1.0)
    || problem.response.iter().any(|t| *t <= 0.0)
  {
    return Err(invalid(
      "retained time must be positive and same-sample events exact0/1",
    ));
  }
  let failures = events.iter().filter(|d| **d == 1.0).count();
  if failures == 0 {
    return Err(invalid("requires at least one observed failure"));
  }
  if n > MAX_WEIBULL_OBSERVATIONS
    || p > MAX_WEIBULL_PARAMETERS
    || n <= p
    || n.checked_mul(p * p).is_none_or(|v| v > MAX_WEIBULL_WORK)
  {
    return Err(invalid(
      "requires N<=10000, P=K+shape<=64, N>P, N*P^2<=20000000",
    ));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|s| s.weights.is_some() || s.cluster_groups.is_some())
  {
    return Err(invalid(
      "only unweighted independent-spell model-based or robust inference is implemented",
    ));
  }
  if let Some(s) = &problem.sample
    && (s.retained_indices.len() != n
      || s.retained_indices.iter().collect::<BTreeSet<_>>().len() != n
      || s.total_observations < n
      || s.dropped_observations != s.total_observations - n
      || s
        .retained_indices
        .iter()
        .any(|i| *i >= s.total_observations))
  {
    return Err(invalid(
      "sample identities must be unique original positions with consistent totals",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k
    || names.iter().any(|s| s.eq_ignore_ascii_case("lnshape"))
  {
    return Err(invalid(
      "parameter names must be unique and not reserved lnshape",
    ));
  }
  let original_x: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|row| {
      let mut r = Vec::with_capacity(k);
      if problem.include_intercept {
        r.push(1.0);
      }
      r.extend(row);
      r
    })
    .collect();
  let root_n = (n as f64).sqrt();
  let scales: Vec<f64> = (0..k)
    .map(|j| {
      original_x
        .iter()
        .fold(0.0_f64, |s, row| s.hypot(row[j] / root_n))
    })
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid("zero or unrepresentable predictor scale"));
  }
  let x: Vec<Vec<f64>> = original_x
    .iter()
    .map(|r| r.iter().zip(&scales).map(|(v, s)| v / s).collect())
    .collect();
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  let log_time: Vec<f64> = problem.response.iter().map(|t| t.ln()).collect();
  let log_scale = log_time
    .iter()
    .fold(0.0_f64, |s, t| s.hypot(t / root_n))
    .max(1.0);
  let solution = optimize(
    &x,
    &log_time,
    events,
    log_scale,
    problem.include_intercept,
    options,
  )?;
  let gamma = solution.theta[k];
  let shape = gamma / log_scale;
  let beta: Vec<f64> = solution.theta[..k]
    .iter()
    .zip(&scales)
    .map(|(b, s)| b / s)
    .collect();
  // At finite solver precision use the ACTUAL beta/logshape observed Hessian,
  // including its -gamma*score_gamma term, not a zero-score-only approximation.
  let mut info = solution.evaluation.information.clone();
  for i in 0..k {
    info[i][k] *= gamma;
    info[k][i] *= gamma;
  }
  info[k][k] = gamma * gamma * info[k][k] - gamma * solution.evaluation.score[k];
  let average: Vec<Vec<f64>> = info
    .iter()
    .map(|r| r.iter().map(|v| v / n as f64).collect())
    .collect();
  let mut covariance = choice_information_inverse(&average)
    .map_err(|_| failed("singular joint beta/logshape observed information"))?;
  for row in &mut covariance {
    for v in row {
      *v /= n as f64;
    }
  }
  let correction = if options.robust {
    n as f64 / (n - 1) as f64
  } else {
    1.0
  };
  if options.robust {
    let mut v = vec![vec![0.0; p]; p];
    let mut vc = vec![vec![0.0; p]; p];
    for ((row, &t), &d) in x.iter().zip(&log_time).zip(events) {
      let eta: f64 = row.iter().zip(&solution.theta).map(|(x, b)| x * b).sum();
      let h = (eta + shape * t).exp();
      let mut score: Vec<f64> = row.iter().map(|x| x * (d - h)).collect();
      score.push(d + (d - h) * shape * t);
      let u: Vec<f64> = covariance
        .iter()
        .map(|r| r.iter().zip(&score).map(|(a, b)| a * b).sum())
        .collect();
      for i in 0..p {
        for j in 0..=i {
          add_compensated(&mut v[i][j], &mut vc[i][j], u[i] * u[j]);
        }
      }
    }
    for i in 0..p {
      for j in 0..=i {
        v[i][j] *= correction;
        v[j][i] = v[i][j];
      }
    }
    covariance = v;
  }
  // Undo X scaling; q=logshape was already the covariance's final coordinate.
  let mut divisor = scales.clone();
  divisor.push(1.0);
  for i in 0..p {
    for j in 0..p {
      let scale = divisor[i] * divisor[j];
      if !scale.is_finite() || scale <= 0.0 {
        return Err(failed("unrepresentable joint covariance units"));
      }
      covariance[i][j] /= scale;
    }
  }
  if covariance.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("unrepresentable full joint covariance"));
  }
  let coefficients: Vec<WeibullEstimate> = names
    .iter()
    .enumerate()
    .map(|(i, name)| inference(name.clone(), beta[i], covariance[i][i]))
    .collect::<Result<_, _>>()?;
  let qi = inference("lnshape".into(), shape.ln(), covariance[k][k])?;
  let slo = qi.confidence_lower.map(f64::exp);
  let shi = qi.confidence_upper.map(f64::exp);
  let positive_ci = slo
    .zip(shi)
    .is_some_and(|(l, h)| l > 0.0 && l.is_finite() && h.is_finite());
  let shape_se = shape * qi.standard_error;
  if !shape.is_finite() || shape <= 0.0 || !shape_se.is_finite() {
    return Err(failed(
      "unrepresentable positive shape or shape standard error",
    ));
  }
  let shape_result = WeibullShape {
    value: shape,
    log_value: shape.ln(),
    standard_error: shape_se,
    log_standard_error: qi.standard_error,
    log_statistic: qi.statistic,
    log_p_value: qi.p_value,
    confidence_lower: if positive_ci { slo } else { None },
    confidence_upper: if positive_ci { shi } else { None },
  };
  let null_x = vec![
    if problem.include_intercept {
      vec![1.0]
    } else {
      vec![]
    };
    n
  ];
  let null = optimize(
    &null_x,
    &log_time,
    events,
    log_scale,
    problem.include_intercept,
    options,
  )?;
  // Optimize the normalized objective: its omitted constant is parameter-free,
  // and removes time-unit shifts from Newton line-search precision and LR.
  let normalized = solution.evaluation.log_likelihood;
  let null_normalized = null.evaluation.log_likelihood;
  let lr = 2.0 * (normalized - null_normalized);
  if lr < -64.0 * f64::EPSILON * (1.0 + normalized.abs() + null_normalized.abs()) || !lr.is_finite()
  {
    return Err(failed("full likelihood is below the same-sample null"));
  }
  let lr = lr.max(0.0);
  let df = k - usize::from(problem.include_intercept);
  let normalization: f64 = events.iter().zip(&log_time).map(|(d, t)| d * t).sum();
  let omnibus = omnibus_wald(&beta, &covariance, problem.include_intercept, None);
  let mut result = WeibullResult {
    time_variable: problem.outcome.clone(),
    failure_variable: failure_variable.into(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    robust: options.robust,
    observation_count: n,
    failure_count: failures,
    censored_count: n - failures,
    coefficients,
    shape: shape_result,
    covariance_matrix: covariance,
    covariance_correction: correction,
    log_likelihood: normalized - normalization,
    normalized_log_likelihood: normalized,
    null_log_likelihood: null_normalized - normalization,
    null_normalized_log_likelihood: null_normalized,
    likelihood_ratio: lr,
    likelihood_ratio_df: df,
    likelihood_ratio_p_value: (!options.robust && df > 0).then(|| chi_square_tail(lr, df)),
    omnibus_wald: omnibus,
    iterations: solution.iterations,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_xb: Vec::with_capacity(n),
    fitted_hazard: Vec::with_capacity(n),
    fitted_cumulative_hazard: Vec::with_capacity(n),
    fitted_survival: Vec::with_capacity(n),
    sample: problem.sample.clone(),
  };
  for (row, &time) in problem.design_matrix.iter().zip(&problem.response) {
    result.fitted_xb.push(weibull_predict_row(
      &result,
      row,
      None,
      WeibullPredictionKind::Xb,
    )?);
    result.fitted_hazard.push(weibull_predict_row(
      &result,
      row,
      Some(time),
      WeibullPredictionKind::Hazard,
    )?);
    result.fitted_cumulative_hazard.push(weibull_predict_row(
      &result,
      row,
      Some(time),
      WeibullPredictionKind::CumulativeHazard,
    )?);
    result.fitted_survival.push(weibull_predict_row(
      &result,
      row,
      Some(time),
      WeibullPredictionKind::Survival,
    )?);
  }
  Ok(result)
}

/// Current-X PH predictions. xb needs no time/event; other kinds require only
/// current positive time and X. The caller handles missing rows before calling.
pub fn weibull_predict_row(
  result: &WeibullResult,
  row: &[f64],
  time: Option<f64>,
  kind: WeibullPredictionKind,
) -> Result<f64, StatsError> {
  if row.len() != result.predictor_names.len() || row.iter().any(|x| !x.is_finite()) {
    return Err(invalid(
      "prediction requires finite predictors in fitted order",
    ));
  }
  let skip = usize::from(result.include_intercept);
  if result.coefficients.len() != row.len() + skip
    || !result.shape.value.is_finite()
    || result.shape.value <= 0.0
  {
    return Err(invalid("invalid retained Weibull fit"));
  }
  let eta = if skip == 1 {
    result.coefficients[0].value
  } else {
    0.0
  } + row
    .iter()
    .zip(&result.coefficients[skip..])
    .map(|(x, b)| x * b.value)
    .sum::<f64>();
  if !eta.is_finite() {
    return Err(failed("unrepresentable PH linear prediction"));
  }
  if kind == WeibullPredictionKind::Xb {
    return Ok(eta);
  }
  let t = time
    .filter(|t| t.is_finite() && *t > 0.0)
    .ok_or_else(|| invalid("prediction time must be finite and strictly positive"))?;
  let a = result.shape.value;
  let log_h = eta + a * t.ln();
  if !log_h.is_finite() {
    return Err(failed("unrepresentable log cumulative hazard"));
  }
  let value = match kind {
    WeibullPredictionKind::Xb => unreachable!(),
    WeibullPredictionKind::Hazard => (a.ln() + eta + (a - 1.0) * t.ln()).exp(),
    WeibullPredictionKind::CumulativeHazard => log_h.exp(),
    // exp(-H) rounds to zero long before H itself overflows. A genuine
    // represented probability endpoint is not an integrated-hazard error.
    WeibullPredictionKind::Survival => {
      if log_h > f64::MAX.ln() {
        0.0
      } else {
        (-log_h.exp()).exp()
      }
    }
  };
  if !value.is_finite() {
    return Err(failed("unrepresentable current-time prediction"));
  }
  Ok(value)
}

#[cfg(test)]
mod tests;
