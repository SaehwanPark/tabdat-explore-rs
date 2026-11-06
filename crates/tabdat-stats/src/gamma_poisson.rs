//! Shared-gamma group effects, not independent-row NB2 mixing.
use crate::binary::omnibus_wald;
use crate::distributions::chi_square_tail;
use crate::matrix::{add_compensated, lgamma, qr_decompose_with_response};
use crate::nb2::{CountCache, log_ratio_remainder, positive_inverse, softplus};
use crate::{EstimationProblem, EstimationSample, LogitEstimate, StatsError, fit_poisson};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GammaPoissonOptions {
  pub max_iterations: usize,
}
impl Default for GammaPoissonOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Model-based inverse full joint observed information. Alpha is a shared
/// multiplicative group variance, not a row-specific NB variance parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GammaPoissonResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub group_count: usize,
  pub zero_total_group_count: usize,
  pub singleton_group_count: usize,
  pub minimum_group_size: usize,
  pub maximum_group_size: usize,
  pub maximum_group_count_sum: f64,
  pub zero_count: usize,
  pub count_sum: f64,
  pub coefficients: Vec<LogitEstimate>,
  pub ln_alpha: LogitEstimate,
  pub alpha: f64,
  pub alpha_standard_error: f64,
  pub alpha_confidence_lower: f64,
  pub alpha_confidence_upper: f64,
  pub covariance_parameter_names: Vec<String>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub poisson_log_likelihood: f64,
  pub dispersion_likelihood_ratio: f64,
  pub dispersion_likelihood_ratio_p_value: f64,
  pub wald_statistic: f64,
  pub wald_df: usize,
  pub wald_p_value: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_means: Vec<f64>,
  pub sample: EstimationSample,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("xtpoisson re: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("xtpoisson re: {message}"))
}
fn max_abs(v: &[f64]) -> f64 {
  v.iter().map(|v| v.abs()).fold(0.0, f64::max)
}
fn times(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
  matrix
    .iter()
    .map(|row| row.iter().zip(vector).map(|(a, b)| a * b).sum())
    .collect()
}
struct Group {
  rows: Vec<usize>,
  total: usize,
  conditional_constant: f64,
}
struct Evaluation {
  means: Vec<f64>,
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
}
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Group],
  parameters: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = parameters.len() - 1;
  let a = parameters[k];
  if !a.is_finite() || a.abs() >= 12.0 {
    return Err(failed("ln(alpha) must be strictly inside (-12,12)"));
  }
  let r = (-a).exp();
  let cache = CountCache::new(groups.iter().map(|g| g.total).max().unwrap(), r);
  let means = x
    .iter()
    .map(|row| {
      row
        .iter()
        .zip(parameters)
        .map(|(v, b)| v * b)
        .sum::<f64>()
        .exp()
    })
    .collect::<Vec<_>>();
  if means.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(failed("positive marginal means are not representable"));
  }
  let mut score = vec![0.0; k + 1];
  let mut sc = vec![0.0; k + 1];
  let mut information = vec![vec![0.0; k + 1]; k + 1];
  let mut ic = information.clone();
  let (mut ll, mut lc) = (0.0, 0.0);
  let scale = groups.len() as f64;
  for group in groups {
    let s = group.total as f64;
    let m = group.rows.iter().map(|&i| means[i]).sum::<f64>();
    if !m.is_finite() {
      return Err(failed("nonfinite group marginal total"));
    }
    let total = r + m;
    let q = m / total;
    let log_ratio = softplus(m.ln() + a);
    let remainder = log_ratio_remainder(q, log_ratio);
    let eta_score = (s - m) * (r / total);
    let cross = q * eta_score;
    let weight = (r + s) / total;
    let mut center = (0..k)
      .map(|j| {
        group
          .rows
          .iter()
          .map(|&i| means[i] / m * x[i][j])
          .sum::<f64>()
      })
      .collect::<Vec<_>>();
    center[0] = x[group.rows[0]][0];
    // NB total plus conditional multinomial factorization keeps the same
    // group law while reusing stable integer-gamma dispersion recurrences.
    let mut conditional = group.conditional_constant;
    let mut cc = 0.0;
    for &i in &group.rows {
      let eta = x[i].iter().zip(parameters).map(|(v, b)| v * b).sum::<f64>();
      add_compensated(&mut conditional, &mut cc, y[i] * (eta - m.ln()));
      for j in 0..k {
        add_compensated(
          &mut score[j],
          &mut sc[j],
          y[i] * (x[i][j] - center[j]) / scale,
        );
        for l in 0..k {
          add_compensated(
            &mut information[j][l],
            &mut ic[j][l],
            weight * means[i] * (x[i][j] - center[j]) * (x[i][l] - center[l]) / scale,
          );
        }
      }
    }
    for j in 0..k {
      add_compensated(&mut score[j], &mut sc[j], center[j] * eta_score / scale);
      for l in 0..k {
        // Centered covariance plus rank-one total information avoids losing
        // intercept curvature by subtracting nearly equal large matrices.
        add_compensated(
          &mut information[j][l],
          &mut ic[j][l],
          weight * m * (r / total) * center[j] * center[l] / scale,
        );
      }
      add_compensated(
        &mut information[j][k],
        &mut ic[j][k],
        center[j] * cross / scale,
      );
      information[k][j] = information[j][k];
    }
    add_compensated(
      &mut score[k],
      &mut sc[k],
      (cache.score_sum[group.total] + r * remainder - s * q) / scale,
    );
    add_compensated(
      &mut information[k][k],
      &mut ic[k][k],
      (r * remainder + cross - cache.curvature_sum[group.total]) / scale,
    );
    add_compensated(
      &mut ll,
      &mut lc,
      conditional + cache.log_coefficient[group.total] - s * softplus(-a - m.ln()) - r * log_ratio,
    );
  }
  if !ll.is_finite()
    || score
      .iter()
      .chain(information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed(
      "nonfinite integrated likelihood, score or information",
    ));
  }
  Ok(Evaluation {
    means,
    log_likelihood: ll,
    score,
    information,
  })
}
struct CoreFit {
  evaluation: Evaluation,
  inverse: Vec<Vec<f64>>,
  parameters: Vec<f64>,
  iterations: usize,
  score: f64,
  step: f64,
}
fn optimize(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Group],
  start: Vec<f64>,
  options: &GammaPoissonOptions,
) -> Result<CoreFit, StatsError> {
  let mut parameters = start;
  for iteration in 1..=options.max_iterations {
    let current = evaluate(x, y, groups, &parameters)?;
    let inverse = positive_inverse(&current.information);
    let step = inverse.as_ref().map(|v| times(v, &current.score));
    let score = max_abs(&current.score);
    if let (Some(inverse), Some(step)) = (&inverse, &step)
      && score <= TOLERANCE
      && max_abs(step) <= TOLERANCE
    {
      return Ok(CoreFit {
        evaluation: current,
        inverse: inverse.clone(),
        parameters,
        iterations: iteration,
        score,
        step: max_abs(step),
      });
    }
    let mut accepted = None;
    // Damping is an intermediate search device only; convergence and the
    // reported covariance always use the full undamped observed information.
    for attempt in 0..20 {
      let damping = if attempt == 0 {
        0.0
      } else {
        10.0_f64.powi(attempt - 7)
      };
      let mut information = current.information.clone();
      for (i, row) in information.iter_mut().enumerate() {
        row[i] += damping;
      }
      let Some(inverse) = positive_inverse(&information) else {
        continue;
      };
      let step = times(&inverse, &current.score);
      let directional = step
        .iter()
        .zip(&current.score)
        .map(|(a, b)| a * b)
        .sum::<f64>();
      if !directional.is_finite() || directional <= 0.0 {
        continue;
      }
      let mut fraction = 1.0;
      for _ in 0..50 {
        let trial = parameters
          .iter()
          .zip(&step)
          .map(|(b, d)| b + fraction * d)
          .collect::<Vec<_>>();
        if let Ok(next) = evaluate(x, y, groups, &trial)
          && next.log_likelihood
            >= current.log_likelihood + groups.len() as f64 * 1e-4 * fraction * directional
              - 1e-12 * (1.0 + current.log_likelihood.abs())
        {
          accepted = Some(trial);
          break;
        }
        fraction *= 0.5;
      }
      if accepted.is_some() {
        break;
      }
    }
    parameters =
      accepted.ok_or_else(|| failed("no improving finite step; possible dispersion boundary"))?;
  }
  Err(failed(
    "iteration limit without strict score/undamped-step convergence; possible dispersion boundary",
  ))
}
fn estimate(name: String, value: f64, variance: f64) -> Result<LogitEstimate, StatsError> {
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
    return Err(failed("nonfinite inference or nonpositive variance"));
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

/// Joint finite cases and exact structural keys are supplied by the adapter.
/// Retains all groups, including all-zero groups and positive/zero singletons.
/// Boundary dispersion fails explicitly; this API never relabels pooled Poisson.
pub fn fit_gamma_poisson(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &GammaPoissonOptions,
) -> Result<GammaPoissonResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let p = problem.predictor_names.len();
  let k = p + 1;
  let dimension = k + 1;
  if !problem.include_intercept
    || problem.weights.is_some()
    || entity_variable.trim().is_empty()
    || options.max_iterations == 0
  {
    return Err(invalid(
      "requires an intercept, group name, no weights and positive iteration limit",
    ));
  }
  let mut names = problem.parameter_names();
  names.push("lnalpha".into());
  if !(1..=32).contains(&p)
    || names.iter().collect::<BTreeSet<_>>().len() != dimension
    || n > 40000
    || n.saturating_mul(dimension * dimension) > 20_000_000
  {
    return Err(invalid(
      "requires distinct parameter names, slopes1..32, rows<=40000 and rows*(slopes+2)^2<=20000000",
    ));
  }
  let sample = problem
    .sample
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  let keys = sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| invalid("requires sample-aligned group keys"))?;
  if sample.weights.is_some()
    || sample.total_observations > 40000
    || sample
      .total_observations
      .saturating_mul(dimension * dimension)
      > 20_000_000
    || keys.len() != n
    || sample.retained_indices.len() != n
    || n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .any(|&i| i >= sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != n
  {
    return Err(invalid(
      "invalid original sample mapping, weights or group-key alignment",
    ));
  }
  if problem
    .response
    .iter()
    .any(|v| *v < 0.0 || *v > 100000.0 || v.fract() != 0.0)
  {
    return Err(invalid("counts must be exact integers in 0..100000"));
  }
  let mut mapping: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
  for (i, key) in keys.iter().enumerate() {
    mapping.entry(key).or_default().push(i);
  }
  if mapping.len() <= dimension || mapping.len() > 10000 {
    return Err(invalid(
      "requires groups>joint parameter count and groups<=10000",
    ));
  }
  let mut groups = Vec::with_capacity(mapping.len());
  for rows in mapping.into_values() {
    let total = rows.iter().map(|&i| problem.response[i]).sum::<f64>();
    if total > 100000.0 {
      return Err(invalid("selected group count totals must be<=100000"));
    }
    let conditional_constant = lgamma(total + 1.0)
      - rows
        .iter()
        .map(|&i| lgamma(problem.response[i] + 1.0))
        .sum::<f64>();
    groups.push(Group {
      rows,
      total: total as usize,
      conditional_constant,
    });
  }
  let g = groups.len();
  let mut x = problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::once(1.0)
        .chain(row.iter().copied())
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let scales = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])) / (n as f64).sqrt())
    .collect::<Vec<_>>();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid("invalid predictor scale"));
  }
  for row in &mut x {
    for (v, s) in row.iter_mut().zip(&scales) {
      *v /= s;
    }
  }
  qr_decompose_with_response(&x, &vec![0.0; n])?;
  // Pooled initialization and the boundary null must use exactly this cohort,
  // but model-based Poisson correctly rejects a misleading cluster attachment.
  let mut pooled_problem = problem.clone();
  pooled_problem.sample.as_mut().unwrap().cluster_groups = None;
  let pooled = fit_poisson(&pooled_problem, false)?;
  let mut numerator = 0.0;
  let mut denominator = 0.0;
  for group in &groups {
    let m = group
      .rows
      .iter()
      .map(|&i| pooled.fitted_means[i])
      .sum::<f64>();
    numerator += (group.total as f64 - m).powi(2) - m;
    denominator += m * m;
  }
  let alpha_start = (numerator / denominator).clamp((-8.0_f64).exp(), 8.0_f64.exp());
  let mut start = pooled
    .coefficients
    .iter()
    .zip(&scales)
    .map(|(b, s)| b.value * s)
    .collect::<Vec<_>>();
  start.push(alpha_start.ln());
  let core = optimize(&x, &problem.response, &groups, start, options)?;
  let mut all_scales = scales;
  all_scales.push(1.0);
  let values = core
    .parameters
    .iter()
    .zip(&all_scales)
    .map(|(v, s)| v / s)
    .collect::<Vec<_>>();
  let covariance = core
    .inverse
    .iter()
    .enumerate()
    .map(|(i, row)| {
      row
        .iter()
        .enumerate()
        .map(|(j, v)| v / g as f64 / all_scales[i] / all_scales[j])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  if covariance.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("covariance not representable in original units"));
  }
  let mut estimates = names
    .iter()
    .enumerate()
    .map(|(j, name)| estimate(name.clone(), values[j], covariance[j][j]))
    .collect::<Result<Vec<_>, _>>()?;
  let ln_alpha = estimates.pop().unwrap();
  let alpha = ln_alpha.value.exp();
  let alpha_standard_error = alpha * ln_alpha.standard_error;
  let alpha_confidence_lower = ln_alpha.confidence_lower.exp();
  let alpha_confidence_upper = ln_alpha.confidence_upper.exp();
  if [
    alpha,
    alpha_standard_error,
    alpha_confidence_lower,
    alpha_confidence_upper,
  ]
  .iter()
  .any(|v| !v.is_finite() || *v <= 0.0)
  {
    return Err(failed("alpha uncertainty not representable"));
  }
  let coefficient_covariance = covariance[..k]
    .iter()
    .map(|row| row[..k].to_vec())
    .collect::<Vec<_>>();
  let wald = omnibus_wald(&values[..k], &coefficient_covariance, true, None);
  let (Some(wald_statistic), Some(wald_p_value)) = (wald.statistic, wald.p_value) else {
    return Err(failed("slope Wald covariance is not full rank"));
  };
  let dispersion_likelihood_ratio = 2.0 * (core.evaluation.log_likelihood - pooled.log_likelihood);
  if !dispersion_likelihood_ratio.is_finite() || dispersion_likelihood_ratio <= 1e-8 {
    return Err(failed(
      "interior dispersion is not distinguishable from Poisson boundary",
    ));
  }
  Ok(GammaPoissonResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    entity_variable: entity_variable.into(),
    observation_count: n,
    group_count: g,
    zero_total_group_count: groups.iter().filter(|g| g.total == 0).count(),
    singleton_group_count: groups.iter().filter(|g| g.rows.len() == 1).count(),
    minimum_group_size: groups.iter().map(|g| g.rows.len()).min().unwrap(),
    maximum_group_size: groups.iter().map(|g| g.rows.len()).max().unwrap(),
    maximum_group_count_sum: groups.iter().map(|g| g.total).max().unwrap() as f64,
    zero_count: problem.response.iter().filter(|&&y| y == 0.0).count(),
    count_sum: problem.response.iter().sum(),
    coefficients: estimates,
    ln_alpha,
    alpha,
    alpha_standard_error,
    alpha_confidence_lower,
    alpha_confidence_upper,
    covariance_parameter_names: names,
    covariance,
    log_likelihood: core.evaluation.log_likelihood,
    poisson_log_likelihood: pooled.log_likelihood,
    dispersion_likelihood_ratio,
    dispersion_likelihood_ratio_p_value: 0.5 * chi_square_tail(dispersion_likelihood_ratio, 1),
    wald_statistic,
    wald_df: p,
    wald_p_value,
    iterations: core.iterations,
    max_abs_scaled_score: core.score,
    max_abs_scaled_newton_step: core.step,
    fitted_means: core.evaluation.means,
    sample: sample.clone(),
  })
}
