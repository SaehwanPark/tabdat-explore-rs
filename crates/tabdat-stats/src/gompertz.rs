//! Entry-zero single-spell Gompertz PH likelihood with unrestricted natural gamma.
//! The log hazard is affine in beta/gamma and integrated hazard is convex, so the
//! shared guarded concave Newton solver applies without a positive-shape boundary.

#![allow(clippy::needless_range_loop)]

use crate::binary::{BinaryOmnibusWald, omnibus_wald};
use crate::distributions::chi_square_tail;
use crate::gaussian_mle::{Evaluation, Solution};
use crate::matrix::{add_compensated, qr_decompose_with_response};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_GOMPERTZ_OBSERVATIONS: usize = 10_000;
pub const MAX_GOMPERTZ_PARAMETERS: usize = 64;
pub const MAX_GOMPERTZ_WORK: usize = 20_000_000;
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GompertzOptions {
  pub max_iterations: usize,
  pub robust: bool,
}
impl Default for GompertzOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
      robust: false,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GompertzEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

/// Natural gamma is unrestricted; its normal z tests gamma=0 (exponential), not
/// a positive-scale boundary. A negative gamma implies positive survival at infinity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GompertzShape {
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

pub type GompertzOmnibusWald = BinaryOmnibusWald;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GompertzResult {
  pub time_variable: String,
  pub failure_variable: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub robust: bool,
  pub observation_count: usize,
  pub failure_count: usize,
  pub censored_count: usize,
  pub coefficients: Vec<GompertzEstimate>,
  pub shape: GompertzShape,
  /// Location coefficients in fitted order, then NATURAL gamma, including all cross terms.
  pub covariance_matrix: Vec<Vec<f64>>,
  pub covariance_correction: f64,
  pub log_likelihood: f64,
  /// Stata event-log-time normalization; parameter-free on this exact retained sample.
  pub normalized_log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub null_normalized_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub omnibus_wald: GompertzOmnibusWald,
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
pub enum GompertzPredictionKind {
  Xb,
  Hazard,
  CumulativeHazard,
  Survival,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("gompertz: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("gompertz: {message}"))
}

fn inference(name: String, value: f64, variance: f64) -> Result<GompertzEstimate, StatsError> {
  if !value.is_finite() || !variance.is_finite() || variance < 0.0 {
    return Err(failed(
      "non-finite estimate or negative/non-finite variance",
    ));
  }
  let se = variance.sqrt();
  let z = value / se;
  let p = chi_square_tail(z * z, 1);
  let lo = value - NORMAL_95 * se;
  let hi = value + NORMAL_95 * se;
  let available = se > 0.0 && [z, p, lo, hi].iter().all(|v| v.is_finite());
  Ok(GompertzEstimate {
    name,
    value,
    standard_error: se,
    statistic: available.then_some(z),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lo),
    confidence_upper: available.then_some(hi),
  })
}

#[derive(Debug, Clone, Copy)]
struct TimeMoments {
  log_m0: f64,
  mean_u: f64,
  mean_u2: f64,
}

// For u in [0,1] with weight exp(z*u), compute log integral and first two
// normalized moments. The small-z Taylor integrals avoid cancellation at gamma0;
// sign-specific tails never form exp(large positive z). No shape clipping.
fn time_moments(z: f64) -> Result<TimeMoments, StatsError> {
  if !z.is_finite() {
    return Err(failed("unrepresentable shape-time product"));
  }
  let (log_m0, mean_u, mean_u2) = if z.abs() <= 1.0 {
    let mut sums = [0.0; 3];
    let mut corrections = [0.0; 3];
    let mut term = 1.0;
    // At |z|<=1 the remaining integral series is <e/23!, below double precision.
    for j in 0..=22 {
      for m in 0..3 {
        add_compensated(&mut sums[m], &mut corrections[m], term / (j + m + 1) as f64);
      }
      term *= z / (j + 1) as f64;
    }
    (sums[0].ln(), sums[1] / sums[0], sums[2] / sums[0])
  } else if z > 0.0 {
    let denominator = -(-z).exp_m1();
    (
      z + denominator.ln() - z.ln(),
      1.0 / denominator - 1.0 / z,
      (1.0 - 2.0 / z) / denominator + 2.0 / z / z,
    )
  } else {
    let denominator = -z.exp_m1();
    let ratio = z.exp() / denominator;
    (
      denominator.ln() - (-z).ln(),
      -ratio - 1.0 / z,
      2.0 / z / z - (1.0 - 2.0 / z) * ratio,
    )
  };
  if !log_m0.is_finite()
    || !mean_u.is_finite()
    || !mean_u2.is_finite()
    || !(0.0..=1.0).contains(&mean_u)
    || !(0.0..=1.0).contains(&mean_u2)
  {
    return Err(failed("unrepresentable integrated time moments"));
  }
  Ok(TimeMoments {
    log_m0,
    mean_u,
    mean_u2,
  })
}

fn log_cumulative(eta: f64, z: f64, log_t: f64, m: TimeMoments) -> Result<f64, StatsError> {
  // In the positive tail combine eta+z BEFORE the logarithmic terms: eta can
  // cancel z, and subtracting z back out of log(exprel(z)) loses log(z).
  let value = if z > 1.0 {
    (eta + z) + (log_t - z.ln()) + (-(-z).exp_m1()).ln()
  } else if z < -1.0 {
    eta + (log_t - (-z).ln()) + (-z.exp_m1()).ln()
  } else {
    eta + log_t + m.log_m0
  };
  if !value.is_finite() {
    return Err(failed("unrepresentable log cumulative hazard"));
  }
  Ok(value)
}

// X is RMS-scaled; tau=t/max(t) and q=gamma*max(t) are linear coordinates.
// The final Hessian remains the FULL natural beta/q information, not log gamma.
fn evaluate(
  x: &[Vec<f64>],
  log_time: &[f64],
  tau: &[f64],
  events: &[f64],
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let p = theta.len();
  let k = p - 1;
  if theta.iter().any(|v| !v.is_finite()) {
    return Err(failed("all parameters must be finite"));
  }
  let mut ll = 0.0;
  let mut llc = 0.0;
  let mut score = vec![0.0; p];
  let mut sc = vec![0.0; p];
  let mut information = vec![vec![0.0; p]; p];
  let mut ic = vec![vec![0.0; p]; p];
  for (((row, &log_t), &t), &d) in x.iter().zip(log_time).zip(tau).zip(events) {
    let eta: f64 = row.iter().zip(theta).map(|(v, b)| v * b).sum();
    let z = theta[k] * t;
    let m = time_moments(z)?;
    let log_hazard = eta + z;
    let h = log_cumulative(eta, z, log_t, m)?.exp();
    if !log_hazard.is_finite() || !h.is_finite() {
      return Err(failed("unrepresentable likelihood hazard"));
    }
    // Optimize the normalized objective so time-unit constants do not degrade
    // line-search precision. Its score/curvature equal the duration density's.
    add_compensated(&mut ll, &mut llc, d * (log_hazard + log_t) - h);
    for i in 0..k {
      add_compensated(&mut score[i], &mut sc[i], row[i] * (d - h));
      for j in 0..=i {
        add_compensated(&mut information[i][j], &mut ic[i][j], h * row[i] * row[j]);
      }
      add_compensated(
        &mut information[k][i],
        &mut ic[k][i],
        h * t * m.mean_u * row[i],
      );
    }
    add_compensated(&mut score[k], &mut sc[k], t * (d - h * m.mean_u));
    add_compensated(&mut information[k][k], &mut ic[k][k], h * t * t * m.mean_u2);
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
  tau: &[f64],
  events: &[f64],
  intercept: bool,
  options: &GompertzOptions,
) -> Result<Solution, StatsError> {
  let k = x[0].len();
  let mut theta = vec![0.0; k + 1];
  if intercept {
    let maximum = log_time.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let log_sum = maximum
      + log_time
        .iter()
        .map(|t| (t - maximum).exp())
        .sum::<f64>()
        .ln();
    theta[0] = (events.iter().sum::<f64>().ln() - log_sum) / x[0][0];
  }
  crate::gaussian_mle::maximize(
    theta,
    x.len(),
    options.max_iterations,
    "gompertz",
    |theta| evaluate(x, log_time, tau, events, theta),
  )
}

/// Fit positive exact times with the same retained sample's exact0/1 events.
/// Sample/weight/cluster/design support is checked before numerical work.
pub fn fit_gompertz(
  problem: &EstimationProblem,
  failure_variable: &str,
  events: &[f64],
  options: &GompertzOptions,
) -> Result<GompertzResult, StatsError> {
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
  if n > MAX_GOMPERTZ_OBSERVATIONS
    || p > MAX_GOMPERTZ_PARAMETERS
    || n <= p
    || n.checked_mul(p * p).is_none_or(|v| v > MAX_GOMPERTZ_WORK)
  {
    return Err(invalid(
      "requires N<=10000, P=K+gamma<=64, N>P, N*P^2<=20000000",
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
    || names.iter().any(|s| s.eq_ignore_ascii_case("gamma"))
  {
    return Err(invalid(
      "parameter names must be unique and not reserved gamma",
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
  let time_scale = problem.response.iter().copied().fold(0.0_f64, f64::max);
  let tau: Vec<f64> = problem.response.iter().map(|t| t / time_scale).collect();
  let solution = optimize(
    &x,
    &log_time,
    &tau,
    events,
    problem.include_intercept,
    options,
  )?;
  let gamma = solution.theta[k] / time_scale;
  let beta: Vec<f64> = solution.theta[..k]
    .iter()
    .zip(&scales)
    .map(|(b, s)| b / s)
    .collect();
  // Gamma is a LINEAR coordinate transform. Unlike a log-shape map, there is
  // no score-dependent Hessian correction and zero/negative gamma are regular.
  let mut covariance = solution.inverse.clone();
  let correction = if options.robust {
    n as f64 / (n - 1) as f64
  } else {
    1.0
  };
  if options.robust {
    let mut v = vec![vec![0.0; p]; p];
    let mut vc = vec![vec![0.0; p]; p];
    for (((row, &log_t), &t), &d) in x.iter().zip(&log_time).zip(&tau).zip(events) {
      let eta: f64 = row.iter().zip(&solution.theta).map(|(a, b)| a * b).sum();
      let z = solution.theta[k] * t;
      let m = time_moments(z)?;
      let h = log_cumulative(eta, z, log_t, m)?.exp();
      let mut score: Vec<f64> = row.iter().map(|x| x * (d - h)).collect();
      score.push(t * (d - h * m.mean_u));
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
  let mut divisor = scales.clone();
  divisor.push(time_scale);
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
  let coefficients = names
    .iter()
    .enumerate()
    .map(|(i, name)| inference(name.clone(), beta[i], covariance[i][i]))
    .collect::<Result<Vec<_>, _>>()?;
  let g = inference("gamma".into(), gamma, covariance[k][k])?;
  let shape = GompertzShape {
    value: g.value,
    standard_error: g.standard_error,
    statistic: g.statistic,
    p_value: g.p_value,
    confidence_lower: g.confidence_lower,
    confidence_upper: g.confidence_upper,
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
    &tau,
    events,
    problem.include_intercept,
    options,
  )?;
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
  let mut result = GompertzResult {
    time_variable: problem.outcome.clone(),
    failure_variable: failure_variable.into(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    robust: options.robust,
    observation_count: n,
    failure_count: failures,
    censored_count: n - failures,
    coefficients,
    shape,
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
    result.fitted_xb.push(gompertz_predict_row(
      &result,
      row,
      None,
      GompertzPredictionKind::Xb,
    )?);
    result.fitted_hazard.push(gompertz_predict_row(
      &result,
      row,
      Some(time),
      GompertzPredictionKind::Hazard,
    )?);
    result.fitted_cumulative_hazard.push(gompertz_predict_row(
      &result,
      row,
      Some(time),
      GompertzPredictionKind::CumulativeHazard,
    )?);
    result.fitted_survival.push(gompertz_predict_row(
      &result,
      row,
      Some(time),
      GompertzPredictionKind::Survival,
    )?);
  }
  Ok(result)
}

/// Current X/time points; no event is required. Time0 has its defined entry-zero
/// values, and negative gamma retains its positive infinite-tail survival limit.
pub fn gompertz_predict_row(
  result: &GompertzResult,
  row: &[f64],
  time: Option<f64>,
  kind: GompertzPredictionKind,
) -> Result<f64, StatsError> {
  if row.len() != result.predictor_names.len() || row.iter().any(|x| !x.is_finite()) {
    return Err(invalid(
      "prediction requires finite predictors in fitted order",
    ));
  }
  let skip = usize::from(result.include_intercept);
  if result.coefficients.len() != row.len() + skip || !result.shape.value.is_finite() {
    return Err(invalid("invalid retained Gompertz fit"));
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
  if kind == GompertzPredictionKind::Xb {
    return Ok(eta);
  }
  let t = time
    .filter(|t| t.is_finite() && *t >= 0.0)
    .ok_or_else(|| invalid("prediction time must be finite and nonnegative"))?;
  if t == 0.0 && kind != GompertzPredictionKind::Hazard {
    return Ok(if kind == GompertzPredictionKind::Survival {
      1.0
    } else {
      0.0
    });
  }
  let z = result.shape.value * t;
  let m = time_moments(z)?;
  let value = match kind {
    GompertzPredictionKind::Xb => unreachable!(),
    GompertzPredictionKind::Hazard => {
      let log_hazard = eta + z;
      if !log_hazard.is_finite() {
        return Err(failed("unrepresentable log hazard"));
      }
      log_hazard.exp()
    }
    GompertzPredictionKind::CumulativeHazard => log_cumulative(eta, z, t.ln(), m)?.exp(),
    GompertzPredictionKind::Survival => {
      let log_h = log_cumulative(eta, z, t.ln(), m)?;
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
