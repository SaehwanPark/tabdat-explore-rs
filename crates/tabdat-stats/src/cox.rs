//! Entry-zero Cox PH with Breslow ties, owned reference-baseline and efficient scores.
//! No intercept: the unspecified baseline absorbs every constant design direction.
#![allow(clippy::needless_range_loop)]

use crate::binary::{BinaryOmnibusWald, omnibus_wald};
use crate::distributions::chi_square_tail;
use crate::gaussian_mle::{Evaluation, Solution};
use crate::matrix::{add_compensated, qr_decompose_with_response};
use crate::{EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_COX_OBSERVATIONS: usize = 10_000;
pub const MAX_COX_PARAMETERS: usize = 64;
pub const MAX_COX_WORK: usize = 20_000_000;
const NORMAL_95: f64 = 1.959963984540054;

/// Unlike a location-model EstimationProblem, Cox admits zero predictors and
/// forbids an intercept. These owned values are the already retained finite sample.
#[derive(Debug, Clone, PartialEq)]
pub struct CoxProblem {
  pub time_variable: String,
  pub failure_variable: String,
  pub predictor_names: Vec<String>,
  pub time: Vec<f64>,
  pub events: Vec<f64>,
  pub design_matrix: Vec<Vec<f64>>,
  pub sample: Option<EstimationSample>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoxOptions {
  pub robust: bool,
  pub max_iterations: usize,
}
impl Default for CoxOptions {
  fn default() -> Self {
    Self {
      robust: false,
      max_iterations: 200,
    }
  }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}
/// Failure-time knots only; all same-time failures AND censors are at risk.
/// Natural-scale NULL means unrepresentable, not a missing fitted log baseline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxBaselineStep {
  pub time: f64,
  pub risk_count: usize,
  pub event_count: usize,
  pub log_increment: f64,
  pub increment: Option<f64>,
  pub log_cumulative_hazard: f64,
  pub cumulative_hazard: Option<f64>,
  pub survival: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxResult {
  pub time_variable: String,
  pub failure_variable: String,
  pub predictor_names: Vec<String>,
  pub robust: bool,
  pub observation_count: usize,
  pub failure_count: usize,
  pub censored_count: usize,
  pub distinct_exit_times: usize,
  pub coefficients: Vec<CoxEstimate>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub covariance_correction: f64,
  /// Baseline reference is ALL retained predictor means, including binary means.
  pub baseline_reference_values: Vec<f64>,
  pub baseline: Vec<CoxBaselineStep>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub omnibus_wald: BinaryOmnibusWald,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_xb: Vec<Option<f64>>,
  pub fitted_cumulative_hazard: Vec<f64>,
  pub fitted_survival: Vec<f64>,
  pub fitted_martingale: Vec<f64>,
  pub sample: Option<EstimationSample>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoxPredictionKind {
  Xb,
  CumulativeHazard,
  Survival,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("cox: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("cox: {message}"))
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
  let (mut value, mut correction) = (0.0, 0.0);
  for (x, y) in a.iter().zip(b) {
    add_compensated(&mut value, &mut correction, x * y);
  }
  value
}
fn finite_exp(log: f64) -> Option<f64> {
  let value = log.exp();
  value.is_finite().then_some(value)
}
fn survival(log_h: f64) -> f64 {
  if log_h > f64::MAX.ln() {
    0.0
  } else {
    (-log_h.exp()).exp()
  }
}
fn log_add(a: f64, b: f64) -> f64 {
  let m = a.max(b);
  m + ((a - m).exp() + (b - m).exp()).ln()
}
fn inference(name: String, value: f64, variance: f64) -> Result<CoxEstimate, StatsError> {
  if !value.is_finite() || !variance.is_finite() || variance < 0.0 {
    return Err(failed("invalid coefficient variance/units"));
  }
  let se = variance.sqrt();
  let z = value / se;
  let p = chi_square_tail(z * z, 1);
  let lo = value - NORMAL_95 * se;
  let hi = value + NORMAL_95 * se;
  let available = se > 0.0 && [z, p, lo, hi].iter().all(|v| v.is_finite());
  Ok(CoxEstimate {
    name,
    value,
    standard_error: se,
    statistic: available.then_some(z),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lo),
    confidence_upper: available.then_some(hi),
  })
}

struct RiskStep {
  time: f64,
  risk_count: usize,
  event_count: usize,
  log_sum: f64,
  mean: Vec<f64>,
}
struct ScoreStep {
  time: f64,
  mean: Vec<f64>,
  log_cumulative: f64,
  cumulative_mean: Vec<f64>,
}

// Reverse-time risk accumulation uses its OWN adaptive maximum at each risk set.
// A single global maximum would erase low-eta late risks after early exits.
fn evaluate(
  z: &[Vec<f64>],
  time: &[f64],
  events: &[f64],
  order: &[usize],
  theta: &[f64],
  retain: bool,
) -> Result<(Evaluation, Vec<RiskStep>), StatsError> {
  let k = theta.len();
  let eta: Vec<f64> = z.iter().map(|r| dot(r, theta)).collect();
  if theta.iter().chain(&eta).any(|v| !v.is_finite()) {
    return Err(failed("non-finite working index"));
  }
  let (mut ll, mut llc) = (0.0, 0.0);
  let mut score = vec![0.0; k];
  let mut sc = vec![0.0; k];
  let mut information = vec![vec![0.0; k]; k];
  let mut ic = information.clone();
  let (mut s0, mut s0c, mut maximum) = (0.0, 0.0, f64::NEG_INFINITY);
  let mut s1 = vec![0.0; k];
  let mut s1c = s1.clone();
  let mut s2 = information.clone();
  let mut s2c = s2.clone();
  let mut steps = Vec::new();
  let mut end = order.len();
  let mut risk_count = 0;
  while end > 0 {
    let t = time[order[end - 1]];
    let mut start = end - 1;
    while start > 0 && time[order[start - 1]] == t {
      start -= 1;
    }
    for &i in &order[start..end] {
      if eta[i] > maximum {
        let factor = (maximum - eta[i]).exp();
        s0 *= factor;
        s0c *= factor;
        for j in 0..k {
          s1[j] *= factor;
          s1c[j] *= factor;
          for l in 0..=j {
            s2[j][l] *= factor;
            s2c[j][l] *= factor;
          }
        }
        maximum = eta[i];
      }
      let w = (eta[i] - maximum).exp();
      add_compensated(&mut s0, &mut s0c, w);
      for j in 0..k {
        add_compensated(&mut s1[j], &mut s1c[j], w * z[i][j]);
        for l in 0..=j {
          add_compensated(&mut s2[j][l], &mut s2c[j][l], w * z[i][j] * z[i][l]);
        }
      }
      risk_count += 1;
    }
    let d = order[start..end]
      .iter()
      .filter(|&&i| events[i] == 1.0)
      .count();
    if d > 0 {
      let log_sum = maximum + s0.ln();
      let mean: Vec<f64> = s1.iter().map(|v| v / s0).collect();
      // Subtract the shared risk maximum before summing event likelihoods.
      add_compensated(&mut ll, &mut llc, -(d as f64) * s0.ln());
      for &i in &order[start..end] {
        if events[i] == 1.0 {
          add_compensated(&mut ll, &mut llc, eta[i] - maximum);
          for j in 0..k {
            add_compensated(&mut score[j], &mut sc[j], z[i][j] - mean[j]);
          }
        }
      }
      for j in 0..k {
        for l in 0..=j {
          add_compensated(
            &mut information[j][l],
            &mut ic[j][l],
            d as f64 * (s2[j][l] / s0 - mean[j] * mean[l]),
          );
        }
      }
      if retain {
        steps.push(RiskStep {
          time: t,
          risk_count,
          event_count: d,
          log_sum,
          mean,
        });
      }
    }
    end = start;
  }
  for j in 0..k {
    for l in 0..j {
      information[l][j] = information[j][l];
    }
  }
  if !ll.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed("non-finite partial likelihood/derivatives"));
  }
  steps.reverse();
  Ok((
    Evaluation {
      log_likelihood: ll,
      score,
      information,
    },
    steps,
  ))
}

fn baseline(steps: Vec<RiskStep>) -> Result<(Vec<CoxBaselineStep>, Vec<ScoreStep>), StatsError> {
  let mut cumulative = f64::NEG_INFINITY;
  let mut cumulative_mean = Vec::<f64>::new();
  let mut reported = Vec::new();
  let mut scoring = Vec::new();
  for step in steps {
    let increment = (step.event_count as f64).ln() - step.log_sum;
    let next = if reported.is_empty() {
      increment
    } else {
      log_add(cumulative, increment)
    };
    if !next.is_finite() {
      return Err(failed("unrepresentable log baseline"));
    }
    if cumulative_mean.is_empty() {
      cumulative_mean = vec![0.0; step.mean.len()];
    }
    let old_weight = (cumulative - next).exp();
    let new_weight = (increment - next).exp();
    for j in 0..step.mean.len() {
      cumulative_mean[j] = old_weight * cumulative_mean[j] + new_weight * step.mean[j];
    }
    cumulative = next;
    reported.push(CoxBaselineStep {
      time: step.time,
      risk_count: step.risk_count,
      event_count: step.event_count,
      log_increment: increment,
      increment: finite_exp(increment),
      log_cumulative_hazard: next,
      cumulative_hazard: finite_exp(next),
      survival: survival(next),
    });
    scoring.push(ScoreStep {
      time: step.time,
      mean: step.mean,
      log_cumulative: next,
      cumulative_mean: cumulative_mean.clone(),
    });
  }
  Ok((reported, scoring))
}

fn efficient_score(
  row: &[f64],
  eta: f64,
  time: f64,
  event: f64,
  steps: &[ScoreStep],
) -> Result<(Vec<f64>, f64), StatsError> {
  let end = steps.partition_point(|step| step.time <= time);
  if end == 0 {
    return Ok((vec![0.0; row.len()], 0.0));
  }
  let step = &steps[end - 1];
  let h = finite_exp(eta + step.log_cumulative)
    .ok_or_else(|| failed("unrepresentable fitted integrated hazard"))?;
  let score: Vec<f64> = row
    .iter()
    .enumerate()
    .map(|(j, x)| event * (x - step.mean[j]) - h * (x - step.cumulative_mean[j]))
    .collect();
  if score.iter().any(|v| !v.is_finite()) {
    return Err(failed("unrepresentable full efficient row score"));
  }
  Ok((score, h))
}

/// Fit Cox/Breslow with an unspecified baseline and no intercept. Missingness is
/// resolved by the caller before this explicit retained-sample support validation.
pub fn fit_cox(problem: &CoxProblem, options: &CoxOptions) -> Result<CoxResult, StatsError> {
  let n = problem.time.len();
  let k = problem.predictor_names.len();
  if n == 0
    || n > MAX_COX_OBSERVATIONS
    || k > MAX_COX_PARAMETERS
    || n <= k
    || n.checked_mul(k * k).is_none_or(|w| w > MAX_COX_WORK)
    || (options.robust && n < 2)
  {
    return Err(invalid(
      "requires N<=10000, K<=64, N>K, N*K^2<=20000000 and robust N>=2",
    ));
  }
  if options.max_iterations == 0
    || problem.time_variable.trim().is_empty()
    || problem.failure_variable.trim().is_empty()
    || problem.predictor_names.iter().any(|s| s.trim().is_empty())
    || problem
      .predictor_names
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != k
  {
    return Err(invalid(
      "nonempty unique names and a positive iteration limit are required",
    ));
  }
  if problem.events.len() != n
    || problem.design_matrix.len() != n
    || problem
      .design_matrix
      .iter()
      .any(|r| r.len() != k || r.iter().any(|v| !v.is_finite()))
    || problem.time.iter().any(|t| !t.is_finite() || *t <= 0.0)
    || problem.events.iter().any(|d| *d != 0.0 && *d != 1.0)
  {
    return Err(invalid(
      "retained finite X, positive times and same-sample exact binary events required",
    ));
  }
  let failures = problem.events.iter().filter(|d| **d == 1.0).count();
  if failures == 0 {
    return Err(invalid("requires at least one observed failure"));
  }
  if let Some(s) = &problem.sample {
    if s.weights.is_some() || s.cluster_groups.is_some() {
      return Err(invalid(
        "only unweighted independent-spell inference is implemented",
      ));
    }
    if s.retained_indices.len() != n
      || s.retained_indices.iter().collect::<BTreeSet<_>>().len() != n
      || s.total_observations < n
      || s.dropped_observations != s.total_observations - n
      || s
        .retained_indices
        .iter()
        .any(|i| *i >= s.total_observations)
    {
      return Err(invalid("inconsistent retained original sample identities"));
    }
  }
  let center: Vec<f64> = (0..k)
    .map(|j| {
      let (mut sum, mut c) = (0.0, 0.0);
      for row in &problem.design_matrix {
        add_compensated(&mut sum, &mut c, row[j] / n as f64);
      }
      sum
    })
    .collect();
  let root_n = (n as f64).sqrt();
  let scales: Vec<f64> = (0..k)
    .map(|j| {
      problem
        .design_matrix
        .iter()
        .fold(0.0_f64, |s, r| s.hypot((r[j] - center[j]) / root_n))
    })
    .collect();
  if center.iter().any(|v| !v.is_finite()) || scales.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(invalid(
      "constant or unrepresentable centered predictor scale; no Cox intercept",
    ));
  }
  let z: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|r| {
      r.iter()
        .zip(&center)
        .zip(&scales)
        .map(|((x, m), s)| (x - m) / s)
        .collect()
    })
    .collect();
  if k > 0 {
    qr_decompose_with_response(&z, &vec![0.0; n])?;
  }
  let mut order: Vec<usize> = (0..n).collect();
  order.sort_by(|&i, &j| problem.time[i].total_cmp(&problem.time[j]));
  let zero = vec![0.0; k];
  let null = evaluate(&z, &problem.time, &problem.events, &order, &zero, false)?
    .0
    .log_likelihood;
  let solution = if k == 0 {
    Solution {
      theta: Vec::new(),
      evaluation: evaluate(&z, &problem.time, &problem.events, &order, &zero, false)?.0,
      inverse: Vec::new(),
      iterations: 0,
      score_bound: 0.0,
      step_bound: 0.0,
    }
  } else {
    crate::gaussian_mle::maximize(zero, n, options.max_iterations, "cox", |theta| {
      evaluate(&z, &problem.time, &problem.events, &order, theta, false).map(|v| v.0)
    })?
  };
  let (_, risk_steps) = evaluate(
    &z,
    &problem.time,
    &problem.events,
    &order,
    &solution.theta,
    true,
  )?;
  let (reported, scoring) = baseline(risk_steps)?;
  let beta: Vec<f64> = solution
    .theta
    .iter()
    .zip(&scales)
    .map(|(b, s)| b / s)
    .collect();
  let eta: Vec<f64> = z.iter().map(|r| dot(r, &solution.theta)).collect();
  let correction = if options.robust {
    n as f64 / (n - 1) as f64
  } else {
    1.0
  };
  let mut covariance = solution.inverse.clone();
  let mut robust = vec![vec![0.0; k]; k];
  let mut robust_c = robust.clone();
  let mut fitted_h = Vec::with_capacity(n);
  for i in 0..n {
    let (score, h) = efficient_score(&z[i], eta[i], problem.time[i], problem.events[i], &scoring)?;
    fitted_h.push(h);
    if options.robust {
      let influence: Vec<f64> = solution.inverse.iter().map(|r| dot(r, &score)).collect();
      for j in 0..k {
        for l in 0..=j {
          add_compensated(
            &mut robust[j][l],
            &mut robust_c[j][l],
            influence[j] * influence[l],
          );
        }
      }
    }
  }
  if options.robust {
    for j in 0..k {
      for l in 0..=j {
        robust[j][l] *= correction;
        robust[l][j] = robust[j][l];
      }
    }
    covariance = robust;
  }
  for j in 0..k {
    for l in 0..=j {
      let value = covariance[j][l] / scales[j] / scales[l];
      covariance[j][l] = value;
      covariance[l][j] = value;
    }
  }
  if covariance.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("unrepresentable coefficient covariance"));
  }
  let coefficients = problem
    .predictor_names
    .iter()
    .enumerate()
    .map(|(j, name)| inference(name.clone(), beta[j], covariance[j][j]))
    .collect::<Result<Vec<_>, _>>()?;
  let ll = solution.evaluation.log_likelihood;
  if ll < null - 64.0 * f64::EPSILON * (1.0 + null.abs()) {
    return Err(failed("converged likelihood below zero-beta null"));
  }
  let lr = 2.0 * (ll - null).max(0.0);
  let lr_p = (!options.robust && k > 0).then(|| chi_square_tail(lr, k));
  let mut exits = problem.time.clone();
  exits.sort_by(f64::total_cmp);
  exits.dedup();
  Ok(CoxResult {
    time_variable: problem.time_variable.clone(),
    failure_variable: problem.failure_variable.clone(),
    predictor_names: problem.predictor_names.clone(),
    robust: options.robust,
    observation_count: n,
    failure_count: failures,
    censored_count: n - failures,
    distinct_exit_times: exits.len(),
    coefficients,
    omnibus_wald: omnibus_wald(&beta, &covariance, false, None),
    covariance_matrix: covariance,
    covariance_correction: correction,
    baseline_reference_values: center,
    baseline: reported,
    log_likelihood: ll,
    null_log_likelihood: null,
    likelihood_ratio: lr,
    likelihood_ratio_df: k,
    likelihood_ratio_p_value: lr_p,
    iterations: solution.iterations,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_xb: problem
      .design_matrix
      .iter()
      .map(|r| {
        let v = dot(r, &beta);
        v.is_finite().then_some(v)
      })
      .collect(),
    fitted_survival: fitted_h.iter().map(|h| (-h).exp()).collect(),
    fitted_martingale: problem
      .events
      .iter()
      .zip(&fitted_h)
      .map(|(d, h)| d - h)
      .collect(),
    fitted_cumulative_hazard: fitted_h,
    sample: problem.sample.clone(),
  })
}

/// Evaluate CURRENT raw predictors and time against the immutable reference step
/// baseline. Step holds after the last failure are estimates, not tail validation.
pub fn cox_predict_row(
  result: &CoxResult,
  predictors: &[f64],
  time: Option<f64>,
  kind: CoxPredictionKind,
) -> Result<f64, StatsError> {
  if predictors.len() != result.coefficients.len() || predictors.iter().any(|v| !v.is_finite()) {
    return Err(invalid(
      "prediction requires every finite current predictor",
    ));
  }
  let beta: Vec<f64> = result.coefficients.iter().map(|c| c.value).collect();
  if kind == CoxPredictionKind::Xb {
    let eta = dot(predictors, &beta);
    return if eta.is_finite() {
      Ok(eta)
    } else {
      Err(failed("unrepresentable current PH index"))
    };
  }
  let t = time
    .filter(|t| t.is_finite() && *t >= 0.0)
    .ok_or_else(|| invalid("current prediction time must be finite and nonnegative"))?;
  let end = result.baseline.partition_point(|b| b.time <= t);
  if end == 0 {
    return Ok(if kind == CoxPredictionKind::Survival {
      1.0
    } else {
      0.0
    });
  }
  let centered: Vec<f64> = predictors
    .iter()
    .zip(&result.baseline_reference_values)
    .map(|(x, m)| x - m)
    .collect();
  let eta = dot(&centered, &beta);
  let log_h = result.baseline[end - 1].log_cumulative_hazard + eta;
  if !eta.is_finite() || !log_h.is_finite() {
    return Err(failed(
      "unrepresentable current centered PH index/log hazard",
    ));
  }
  if kind == CoxPredictionKind::Survival {
    Ok(survival(log_h))
  } else {
    finite_exp(log_h).ok_or_else(|| failed("unrepresentable current cumulative hazard"))
  }
}

#[cfg(test)]
mod tests;
