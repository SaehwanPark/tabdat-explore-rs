//! Bounded integrated normal random-intercept Poisson, not mean-one gamma RE.
mod increment;
mod nodes;
mod refined_nodes;
use crate::{
  EstimationProblem, EstimationSample, LogitEstimate, StatsError, fit_poisson,
  matrix::add_compensated,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalPoissonOptions {
  pub max_iterations: usize,
}
impl Default for NormalPoissonOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Model-based inverse full joint observed information; population means integrate
/// the mean-zero normal log effect, not posterior firm effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalPoissonResult {
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
  pub ln_variance: LogitEstimate,
  pub sigma: f64,
  pub sigma_standard_error: f64,
  pub sigma_confidence_lower: f64,
  pub sigma_confidence_upper: f64,
  pub covariance_parameter_names: Vec<String>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub poisson_log_likelihood: f64,
  pub variance_likelihood_ratio: f64,
  pub variance_likelihood_ratio_p_value: f64,
  pub wald_statistic: f64,
  pub wald_df: usize,
  pub wald_p_value: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub max_relative_quadrature_difference: f64,
  pub fitted_means: Vec<f64>,
  pub sample: EstimationSample,
}
use nodes::{GH80, GH96};

#[derive(Clone, Copy)]
struct Quadrature {
  low: &'static [(f64, f64)],
  high: &'static [(f64, f64)],
}
const ORDINARY: Quadrature = Quadrature {
  low: &GH80,
  high: &GH96,
};
const REFINED: Quadrature = Quadrature {
  low: &refined_nodes::GH176,
  high: &refined_nodes::GH208,
};
pub(crate) const COPIED_QUADRATURE_NODES: [usize; 2] = [REFINED.low.len(), REFINED.high.len()];

#[derive(Debug, Clone)]
struct Posterior {
  log_integral: f64,
  mean_exp_u: f64,
  mean_u2: f64,
  #[cfg(test)]
  var_exp_u: f64,
  cov_exp_u_u2: f64,
  var_u2: f64,
  mean_u: f64,
  cov_exp_u_u: f64,
  #[cfg(test)]
  mode: f64,
  #[cfg(test)]
  curvature: f64,
  max_relative_difference: f64,
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("normal Poisson RE: {message}"))
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut total, mut correction) = (0., 0.);
  for value in values {
    add_compensated(&mut total, &mut correction, value);
  }
  total
}
fn mode(total: f64, mean_sum: f64, variance: f64) -> Result<(f64, f64), StatsError> {
  let score = |u: f64| total - mean_sum * u.exp() - u / variance;
  let (mut lo, mut hi) = (-60., 60.);
  if score(lo) <= 0. || score(hi) >= 0. {
    return Err(invalid("posterior mode is not bracketed"));
  }
  for _ in 0..55 {
    let mid = (lo + hi) / 2.;
    if score(mid) > 0. {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  let center = (lo + hi) / 2.;
  let lambda = mean_sum * center.exp();
  if score(center).abs() > 1e-10 * (1. + total + lambda + (center / variance).abs()) {
    return Err(invalid("posterior mode root check failed"));
  }
  Ok((center, lambda + 1. / variance))
}
fn integrate_rule(
  total: f64,
  mean_sum: f64,
  variance: f64,
  center: f64,
  curvature: f64,
  rule: &[(f64, f64)],
) -> Result<[f64; 8], StatsError> {
  let scale = (2. / curvature).sqrt();
  let mut values = Vec::with_capacity(rule.len());
  for &(z, weight) in rule {
    let u = center + scale * z;
    if !u.is_finite() || u.abs() > 100. {
      return Err(invalid("quadrature random intercept exceeds100"));
    }
    let exp_u = u.exp();
    let log_weight = weight.ln() + z * z + total * u - mean_sum * exp_u - u * u / (2. * variance);
    if !log_weight.is_finite() {
      return Err(invalid("nonfinite quadrature integrand"));
    }
    values.push((log_weight, exp_u, u * u, u));
  }
  let anchor = values.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
  let weights: Vec<_> = values.iter().map(|v| (v.0 - anchor).exp()).collect();
  let denominator = sum(weights.iter().copied());
  let mean_exp = sum(values.iter().zip(&weights).map(|(v, w)| w * v.1)) / denominator;
  let mean_u2 = sum(values.iter().zip(&weights).map(|(v, w)| w * v.2)) / denominator;
  let mean_u = sum(values.iter().zip(&weights).map(|(v, w)| w * v.3)) / denominator;
  let cov_u = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp) * (v.3 - mean_u)),
  ) / denominator;
  // Centered products retain small posterior covariances at high count totals.
  let var_exp = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp).powi(2)),
  ) / denominator;
  let cov = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.1 - mean_exp) * (v.2 - mean_u2)),
  ) / denominator;
  let var_u2 = sum(
    values
      .iter()
      .zip(&weights)
      .map(|(v, w)| w * (v.2 - mean_u2).powi(2)),
  ) / denominator;
  let log_integral =
    scale.ln() + anchor + denominator.ln() - (2. * std::f64::consts::PI * variance).ln() / 2.;
  let result = [
    log_integral,
    mean_exp,
    mean_u2,
    var_exp,
    cov,
    var_u2,
    mean_u,
    cov_u,
  ];
  if result.iter().any(|v| !v.is_finite()) {
    return Err(invalid("nonfinite posterior moments"));
  }
  Ok(result)
}
#[cfg(test)]
fn posterior(total: f64, mean_sum: f64, log_variance: f64) -> Result<Posterior, StatsError> {
  posterior_with_rule(total, mean_sum, log_variance, ORDINARY)
}
fn posterior_with_rule(
  total: f64,
  mean_sum: f64,
  log_variance: f64,
  quadrature: Quadrature,
) -> Result<Posterior, StatsError> {
  if !total.is_finite()
    || total < 0.
    || total > 100000.
    || total.fract() != 0.
    || !mean_sum.is_finite()
    || mean_sum <= 0.
    || !log_variance.is_finite()
    || log_variance.abs() >= 8.
  {
    return Err(invalid("invalid integrated group parameters"));
  }
  let variance = log_variance.exp();
  let (mode, curvature) = mode(total, mean_sum, variance)?;
  let high = integrate_rule(total, mean_sum, variance, mode, curvature, quadrature.high)?;
  let low = integrate_rule(total, mean_sum, variance, mode, curvature, quadrature.low)?;
  // Check intensity and prior-score units, not tiny unscaled exp(u) moments:
  // regression/information multiply these by M and inverse variance.
  let units = [
    1.,
    mean_sum,
    1. / variance,
    mean_sum.powi(2),
    mean_sum / variance,
    1. / variance.powi(2),
    1. / variance,
    mean_sum / variance,
  ];
  let discrepancy = high
    .iter()
    .zip(low)
    .zip(units)
    .map(|((a, b), scale)| ((a - b) * scale).abs() / (1. + (a * scale).abs()))
    .fold(0., f64::max);
  if discrepancy > 1e-9 {
    return Err(invalid(&format!(
      "{}/{}-node quadrature disagreement exceeds1e-9",
      quadrature.low.len(),
      quadrature.high.len()
    )));
  }
  Ok(Posterior {
    log_integral: high[0],
    mean_exp_u: high[1],
    mean_u2: high[2],
    #[cfg(test)]
    var_exp_u: high[3],
    cov_exp_u_u2: high[4],
    var_u2: high[5],
    mean_u: high[6],
    cov_exp_u_u: high[7],
    #[cfg(test)]
    mode,
    #[cfg(test)]
    curvature,
    max_relative_difference: discrepancy,
  })
}

#[derive(Debug)]
struct Evaluation {
  log_likelihood: f64,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
  max_quadrature_difference: f64,
}

#[cfg(test)]
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  parameters: &[f64],
) -> Result<Evaluation, StatsError> {
  evaluate_with_rule(x, y, groups, parameters, ORDINARY)
}
fn evaluate_with_rule(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  parameters: &[f64],
  quadrature: Quadrature,
) -> Result<Evaluation, StatsError> {
  let k = parameters
    .len()
    .checked_sub(1)
    .ok_or_else(|| invalid("empty parameters"))?;
  if !(2..=15).contains(&k)
    || x.len() != y.len()
    || y.is_empty()
    || groups.is_empty()
    || x
      .iter()
      .any(|row| row.len() != k || row[0] != 1. || row.iter().any(|v| !v.is_finite()))
    || y
      .iter()
      .any(|v| !v.is_finite() || *v < 0. || *v > 100000. || v.fract() != 0.)
  {
    return Err(invalid("inconsistent complete-case group design"));
  }
  let t = parameters[k];
  if !t.is_finite() || t.abs() >= 8. {
    return Err(invalid("log variance must be strictly inside (-8,8)"));
  }
  let variance = t.exp();
  let mut seen = vec![false; y.len()];
  for group in groups {
    if group.is_empty() {
      return Err(invalid("empty group"));
    }
    for &row in group {
      if row >= y.len() || seen[row] {
        return Err(invalid("group frame must partition rows exactly"));
      }
      seen[row] = true;
    }
  }
  if seen.iter().any(|v| !*v) {
    return Err(invalid("group frame omitted rows"));
  }
  let eta: Vec<_> = x
    .iter()
    .map(|row| sum(row.iter().zip(parameters).map(|(a, b)| a * b)))
    .collect();
  if eta.iter().any(|v| !v.is_finite() || *v < -40. || *v > 40.) {
    return Err(invalid("fixed linear predictor exceeds [-40,40]"));
  }
  if eta
    .iter()
    .any(|v| !(-700. ..=700.).contains(&(v + variance / 2.)))
  {
    return Err(invalid("population log mean exceeds [-700,700]"));
  }
  let mu: Vec<_> = eta.iter().map(|v| v.exp()).collect();
  let mut score = vec![0.; k + 1];
  let mut sc = score.clone();
  let mut information = vec![vec![0.; k + 1]; k + 1];
  let mut ic = information.clone();
  let (mut ll, mut lc, mut max_difference) = (0., 0., 0.0_f64);
  for group in groups {
    let total = sum(group.iter().map(|&i| y[i]));
    let m = sum(group.iter().map(|&i| mu[i]));
    let post = posterior_with_rule(total, m, t, quadrature)?;
    max_difference = max_difference.max(post.max_relative_difference);
    let mut center: Vec<_> = (0..k)
      .map(|j| sum(group.iter().map(|&i| mu[i] / m * x[i][j])))
      .collect();
    center[0] = 1.;
    let lambda_mean = m * post.mean_exp_u;
    // Stein identities stabilize count-total score/curvature. In particular,
    // Cov(lambda,u)/v replaces E(lambda)-Var(lambda), avoiding large-count subtraction.
    let eta_score = post.mean_u / variance;
    let eta_information = m * post.cov_exp_u_u / variance;
    for j in 0..k {
      let conditional_score = sum(group.iter().map(|&i| y[i] * (x[i][j] - center[j])));
      add_compensated(
        &mut score[j],
        &mut sc[j],
        conditional_score + center[j] * eta_score,
      );
      for l in j..k {
        let within = sum(
          group
            .iter()
            .map(|&i| mu[i] / m * (x[i][j] - center[j]) * (x[i][l] - center[l])),
        );
        add_compensated(
          &mut information[j][l],
          &mut ic[j][l],
          lambda_mean * within + eta_information * center[j] * center[l],
        );
      }
      add_compensated(
        &mut information[j][k],
        &mut ic[j][k],
        center[j] * m * post.cov_exp_u_u2 / (2. * variance),
      );
    }
    add_compensated(
      &mut score[k],
      &mut sc[k],
      post.mean_u2 / (2. * variance) - 0.5,
    );
    add_compensated(
      &mut information[k][k],
      &mut ic[k][k],
      post.mean_u2 / (2. * variance) - post.var_u2 / (4. * variance * variance),
    );
    let constant = sum(
      group
        .iter()
        .map(|&i| y[i] * eta[i] - crate::matrix::lgamma(y[i] + 1.)),
    );
    add_compensated(&mut ll, &mut lc, constant + post.log_integral);
  }
  let g = groups.len() as f64;
  for value in &mut score {
    *value /= g;
  }
  for row in &mut information {
    for value in row {
      *value /= g;
    }
  }
  for j in 0..information.len() {
    let (earlier, remaining) = information.split_at_mut(j);
    for (l, row) in earlier.iter().enumerate() {
      remaining[0][l] = row[j];
    }
  }
  if !ll.is_finite()
    || score.iter().any(|v| !v.is_finite())
    || information.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite integrated likelihood derivatives"));
  }
  Ok(Evaluation {
    log_likelihood: ll,
    score,
    information,
    max_quadrature_difference: max_difference,
  })
}

#[derive(Debug)]
struct CoreFit {
  evaluation: Evaluation,
  parameters: Vec<f64>,
  inverse: Vec<Vec<f64>>,
  iterations: usize,
  score_norm: f64,
  relative_step: f64,
}
fn scaled_inverse(information: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
  let scales: Vec<_> = information
    .iter()
    .enumerate()
    .map(|(i, row)| row[i].sqrt())
    .collect();
  if scales.iter().any(|v| !v.is_finite() || *v <= 0.) {
    return None;
  }
  let normalized: Vec<Vec<_>> = information
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
  // Existing SPD inversion checks dimensionless pivots above1e-12 after whitening;
  // neither step nor reported covariance uses a ridge or absolute eigenvalues.
  crate::nb2::positive_inverse(&normalized).map(|inverse| {
    inverse
      .iter()
      .enumerate()
      .map(|(i, row)| {
        row
          .iter()
          .enumerate()
          .map(|(j, v)| v / scales[i] / scales[j])
          .collect()
      })
      .collect()
  })
}
#[cfg(test)]
fn optimize(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  parameters: Vec<f64>,
  max_iterations: usize,
) -> Result<CoreFit, StatsError> {
  optimize_with_rule(x, y, groups, parameters, max_iterations, ORDINARY)
}
fn optimize_with_rule(
  x: &[Vec<f64>],
  y: &[f64],
  groups: &[Vec<usize>],
  mut parameters: Vec<f64>,
  max_iterations: usize,
  quadrature: Quadrature,
) -> Result<CoreFit, StatsError> {
  if max_iterations == 0 || max_iterations > 100 {
    return Err(invalid("iteration limit must be1..100"));
  }
  for iteration in 1..=max_iterations {
    let current = evaluate_with_rule(x, y, groups, &parameters, quadrature)?;
    let inverse = scaled_inverse(&current.information)
      .ok_or_else(|| invalid("joint undamped information is not positive definite"))?;
    let step: Vec<_> = inverse
      .iter()
      .map(|row| sum(row.iter().zip(&current.score).map(|(a, b)| a * b)))
      .collect();
    let score_norm = current.score.iter().map(|v| v.abs()).fold(0., f64::max);
    let relative_step = step
      .iter()
      .zip(&parameters)
      .map(|(d, b)| d.abs() / (1. + b.abs()))
      .fold(0., f64::max);
    if score_norm <= 1e-10 && relative_step <= 1e-12 {
      return Ok(CoreFit {
        evaluation: current,
        parameters,
        inverse,
        iterations: iteration,
        score_norm,
        relative_step,
      });
    }
    let directional = sum(step.iter().zip(&current.score).map(|(a, b)| a * b));
    if !directional.is_finite() || directional <= 0. {
      return Err(invalid("Newton direction is not finite and ascending"));
    }
    let mut fraction = 1.;
    let mut accepted = None;
    for _ in 0..32 {
      let trial: Vec<_> = parameters
        .iter()
        .zip(&step)
        .map(|(b, d)| b + fraction * d)
        .collect();
      if trial != parameters
        && let Ok(next) = evaluate_with_rule(x, y, groups, &trial, quadrature)
      {
        let nondecreasing = match increment::stable_likelihood_increment(
          x,
          y,
          groups,
          &parameters,
          &trial,
          &current.score,
          quadrature,
        ) {
          Ok(Some(change)) => change >= 0.,
          Ok(None) => next.log_likelihood >= current.log_likelihood,
          Err(_) => false,
        };
        if nondecreasing {
          accepted = Some(trial);
          break;
        }
      }
      fraction *= 0.5;
    }
    parameters =
      accepted.ok_or_else(|| invalid("no finite nondecreasing Newton step within32 trials"))?;
  }
  Err(invalid(
    "iteration limit without strict joint score/step convergence",
  ))
}

fn estimate(name: String, value: f64, variance: f64) -> Result<LogitEstimate, StatsError> {
  let se = variance.sqrt();
  let statistic = value / se;
  let critical = 1.959963984540054;
  let p_value = crate::distributions::chi_square_tail(statistic * statistic, 1);
  let lower = value - critical * se;
  let upper = value + critical * se;
  if se <= 0.
    || [value, se, statistic, p_value, lower, upper]
      .iter()
      .any(|v| !v.is_finite())
  {
    return Err(invalid("coefficient inference is not representable"));
  }
  Ok(LogitEstimate {
    name,
    value,
    standard_error: se,
    statistic,
    p_value,
    confidence_lower: lower,
    confidence_upper: upper,
  })
}

/// Joint finite rows and exact raw-key validation are supplied by the adapter.
/// All zero-total/singleton groups remain; boundary or numerical failure is not
/// relabeled as pooled Poisson. Returned covariance includes variance cross terms.
pub fn fit_normal_poisson(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &NormalPoissonOptions,
) -> Result<NormalPoissonResult, StatsError> {
  fit_with_rule(problem, entity_variable, options, ORDINARY, None)
}
// One deterministic original-normal initializer, not an error-triggered retry.
// Each copied frame rescales the raw coefficients into its own RMS units.
pub(crate) fn fit_normal_poisson_copied(
  problem: &EstimationProblem,
  entity_variable: &str,
  original_raw_parameters: &[f64],
) -> Result<NormalPoissonResult, StatsError> {
  fit_with_rule(
    problem,
    entity_variable,
    &NormalPoissonOptions::default(),
    REFINED,
    Some(original_raw_parameters),
  )
}

fn fit_with_rule(
  problem: &EstimationProblem,
  entity_variable: &str,
  options: &NormalPoissonOptions,
  quadrature: Quadrature,
  initial_raw_parameters: Option<&[f64]>,
) -> Result<NormalPoissonResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let p = problem.predictor_names.len();
  let k = p + 1;
  let dimension = k + 1;
  let mut names = problem.parameter_names();
  names.push("lnsigma2".into());
  if !problem.include_intercept
    || problem.weights.is_some()
    || entity_variable.trim().is_empty()
    || !(1..=14).contains(&p)
    || names.iter().collect::<BTreeSet<_>>().len() != dimension
    || !(1..=100).contains(&options.max_iterations)
  {
    return Err(invalid(
      "requires intercept,1..14 unique slopes/group name, no weights and1..100 iterations",
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
  if sample.total_observations > 40000
    || sample
      .total_observations
      .checked_mul(quadrature.low.len() + quadrature.high.len())
      .and_then(|v| v.checked_mul(dimension))
      .and_then(|v| v.checked_mul(dimension))
      .is_none_or(|v| v > 20000000)
  {
    return Err(invalid(&format!(
      "candidate rows<=40000 and{}*candidate rows*(slopes+2)^2<=20000000 required",
      quadrature.low.len() + quadrature.high.len(),
    )));
  }
  if problem
    .response
    .iter()
    .any(|v| *v < 0. || *v > 100000. || v.fract() != 0.)
  {
    return Err(invalid("selected counts must be exact integers0..100000"));
  }
  let mut mapping: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
  for (i, key) in keys.iter().enumerate() {
    mapping.entry(key).or_default().push(i);
  }
  let groups: Vec<_> = mapping.into_values().collect();
  let g = groups.len();
  if g <= dimension || g > 10000 {
    return Err(invalid(
      "selected groups must exceed joint parameter count and be<=10000",
    ));
  }
  let totals: Vec<_> = groups
    .iter()
    .map(|rows| sum(rows.iter().map(|&i| problem.response[i])))
    .collect();
  if totals.iter().any(|v| *v > 100000.) {
    return Err(invalid("selected group totals must be<=100000"));
  }
  let mut x: Vec<Vec<_>> = problem
    .design_matrix
    .iter()
    .map(|row| std::iter::once(1.).chain(row.iter().copied()).collect())
    .collect();
  let mut scales: Vec<_> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |a, row| a.hypot(row[j])) / (n as f64).sqrt())
    .collect();
  scales[0] = 1.;
  if scales.iter().any(|v| !v.is_finite() || *v <= 0.) {
    return Err(invalid("invalid predictor scale"));
  }
  for row in &mut x {
    for (v, scale) in row.iter_mut().zip(&scales) {
      *v /= scale;
    }
  }
  crate::matrix::qr_decompose_with_response(&x, &vec![0.; n])?;
  // Detach only owned initialization metadata; null/starter use the same selected rows.
  let mut pooled_problem = problem.clone();
  pooled_problem.sample.as_mut().unwrap().cluster_groups = None;
  let pooled = fit_poisson(&pooled_problem, false)?;
  let start: Vec<_> = if let Some(raw) = initial_raw_parameters {
    if raw.len() != dimension || raw.iter().any(|v| !v.is_finite()) {
      return Err(invalid(
        "copied initializer requires finite raw joint parameters",
      ));
    }
    raw[..k]
      .iter()
      .zip(&scales)
      .map(|(b, s)| b * s)
      .chain(std::iter::once(raw[k]))
      .collect()
  } else {
    pooled
      .coefficients
      .iter()
      .zip(&scales)
      .map(|(b, s)| b.value * s)
      .chain(std::iter::once(0.))
      .collect()
  };
  let core = optimize_with_rule(
    &x,
    &problem.response,
    &groups,
    start,
    options.max_iterations,
    quadrature,
  )?;
  scales.push(1.);
  let values: Vec<_> = core
    .parameters
    .iter()
    .zip(&scales)
    .map(|(v, s)| v / s)
    .collect();
  let covariance: Vec<Vec<_>> = core
    .inverse
    .iter()
    .enumerate()
    .map(|(i, row)| {
      row
        .iter()
        .enumerate()
        .map(|(j, v)| v / g as f64 / scales[i] / scales[j])
        .collect()
    })
    .collect();
  if covariance.iter().flatten().any(|v| !v.is_finite()) {
    return Err(invalid("covariance is not representable in original units"));
  }
  let mut coefficients = names
    .iter()
    .enumerate()
    .map(|(i, name)| estimate(name.clone(), values[i], covariance[i][i]))
    .collect::<Result<Vec<_>, _>>()?;
  let ln_variance = coefficients.pop().unwrap();
  let sigma = (ln_variance.value / 2.).exp();
  let sigma_standard_error = sigma * ln_variance.standard_error / 2.;
  let sigma_confidence_lower = (ln_variance.confidence_lower / 2.).exp();
  let sigma_confidence_upper = (ln_variance.confidence_upper / 2.).exp();
  if [
    sigma,
    sigma_standard_error,
    sigma_confidence_lower,
    sigma_confidence_upper,
  ]
  .iter()
  .any(|v| !v.is_finite() || *v <= 0.)
  {
    return Err(invalid("sigma uncertainty is not representable"));
  }
  let beta_covariance: Vec<_> = covariance[..k]
    .iter()
    .map(|row| row[..k].to_vec())
    .collect();
  let wald = crate::binary::omnibus_wald(&values[..k], &beta_covariance, true, None);
  let (Some(wald_statistic), Some(wald_p_value)) = (wald.statistic, wald.p_value) else {
    return Err(invalid("slope Wald covariance is not full rank"));
  };
  let lr = 2. * (core.evaluation.log_likelihood - pooled.log_likelihood);
  if !lr.is_finite() || lr <= 1e-8 {
    return Err(invalid(
      "positive variance is not distinguishable from pooled boundary",
    ));
  }
  let variance = ln_variance.value.exp();
  let fitted_means = x
    .iter()
    .map(|row| {
      let eta = sum(row.iter().zip(&core.parameters).map(|(a, b)| a * b)) + variance / 2.;
      if !eta.is_finite() || !(-700. ..=700.).contains(&eta) {
        return Err(invalid("population log mean exceeds [-700,700]"));
      }
      Ok(eta.exp())
    })
    .collect::<Result<Vec<_>, _>>()?;
  Ok(NormalPoissonResult {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    entity_variable: entity_variable.into(),
    observation_count: n,
    group_count: g,
    zero_total_group_count: totals.iter().filter(|&&v| v == 0.).count(),
    singleton_group_count: groups.iter().filter(|v| v.len() == 1).count(),
    minimum_group_size: groups.iter().map(Vec::len).min().unwrap(),
    maximum_group_size: groups.iter().map(Vec::len).max().unwrap(),
    maximum_group_count_sum: totals.iter().copied().fold(0., f64::max),
    zero_count: problem.response.iter().filter(|&&v| v == 0.).count(),
    count_sum: sum(problem.response.iter().copied()),
    coefficients,
    ln_variance,
    sigma,
    sigma_standard_error,
    sigma_confidence_lower,
    sigma_confidence_upper,
    covariance_parameter_names: names,
    covariance,
    log_likelihood: core.evaluation.log_likelihood,
    poisson_log_likelihood: pooled.log_likelihood,
    variance_likelihood_ratio: lr,
    variance_likelihood_ratio_p_value: 0.5 * crate::distributions::chi_square_tail(lr, 1),
    wald_statistic,
    wald_df: p,
    wald_p_value,
    iterations: core.iterations,
    max_abs_scaled_score: core.score_norm,
    max_abs_scaled_newton_step: core.relative_step,
    max_relative_quadrature_difference: core.evaluation.max_quadrature_difference,
    fitted_means,
    sample: sample.clone(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  fn control() -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<usize>>) {
    let mut x = vec![];
    let mut y = vec![];
    let mut groups = vec![];
    for line in include_str!("../../../tests/fixtures/mma23_gamma_re_bootstrap/source.asc")
      .lines()
      .filter(|line| !line.trim().is_empty())
    {
      let cells: Vec<_> = line.split_whitespace().collect();
      let mut rows = vec![];
      for j in 0..5 {
        rows.push(y.len());
        x.push(vec![1., cells[10 + j].parse().unwrap()]);
        y.push(cells[20 + j].parse().unwrap());
      }
      groups.push(rows);
    }
    assert_eq!((x.len(), groups.len()), (200, 40));
    (x, y, groups)
  }
  fn control_problem() -> EstimationProblem {
    let (x, y, groups) = control();
    let n = y.len();
    let mut keys = vec![String::new(); n];
    for (i, rows) in groups.iter().enumerate() {
      for &row in rows {
        keys[row] = (i + 1).to_string();
      }
    }
    EstimationProblem::new(
      "pat",
      vec!["logr".into()],
      y,
      x.iter().map(|row| row[1..].to_vec()).collect(),
    )
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap())
  }
  #[test]
  fn public_fit_matches_all42_independent_r_fields_full_covariance_and_sample() {
    use std::collections::BTreeMap;
    let p = control_problem();
    let before = p.clone();
    for (quadrature, r) in [
      (
        ORDINARY,
        fit_normal_poisson(&p, "id", &NormalPoissonOptions::default()).unwrap(),
      ),
      (
        REFINED,
        fit_with_rule(&p, "id", &NormalPoissonOptions::default(), REFINED, None).unwrap(),
      ),
    ] {
      assert_eq!(p, before);
      assert_eq!(r.sample, p.sample.clone().unwrap());
      assert_eq!(
        (r.observation_count, r.group_count, r.zero_total_group_count),
        (200, 40, 8)
      );
      assert_eq!(r.covariance_parameter_names, ["_cons", "logr", "lnsigma2"]);
      let reference: BTreeMap<_, _> =
        include_str!("../../../tests/fixtures/mma23_normal_re/reference.csv")
          .lines()
          .skip(1)
          .map(|line| {
            let (k, v) = line.split_once(',').unwrap();
            (k, v.parse::<f64>().unwrap())
          })
          .collect();
      assert_eq!(reference.len(), 42);
      let mut actual = BTreeMap::from([
        ("candidate_rows".into(), 200.),
        ("firms".into(), 40.),
        ("zero_total_firms".into(), 8.),
        ("sigma".into(), r.sigma),
        ("log_likelihood".into(), r.log_likelihood),
        ("poisson_log_likelihood".into(), r.poisson_log_likelihood),
        ("lr_variance_zero".into(), r.variance_likelihood_ratio),
        (
          "p_lr_variance_zero".into(),
          r.variance_likelihood_ratio_p_value,
        ),
        ("se_sigma".into(), r.sigma_standard_error),
        ("lower_sigma".into(), r.sigma_confidence_lower),
        ("upper_sigma".into(), r.sigma_confidence_upper),
      ]);
      let (x, y, groups) = control();
      let values = vec![
        r.coefficients[0].value,
        r.coefficients[1].value,
        r.ln_variance.value,
      ];
      let e = evaluate_with_rule(&x, &y, &groups, &values, quadrature).unwrap();
      for (i, (name, c)) in ["intercept", "slope", "lnvariance"]
        .iter()
        .zip(r.coefficients.iter().chain(std::iter::once(&r.ln_variance)))
        .enumerate()
      {
        for (prefix, value) in [
          ("estimate", c.value),
          ("se", c.standard_error),
          ("z", c.statistic),
          ("p", c.p_value),
          ("lower", c.confidence_lower),
          ("upper", c.confidence_upper),
          ("score", e.score[i] * 40.),
        ] {
          actual.insert(format!("{prefix}_{name}"), value);
        }
      }
      for (i, a) in ['a', 'b', 't'].iter().enumerate() {
        for (j, b) in ['a', 'b', 't'].iter().enumerate() {
          let (a, b) = if i <= j { (a, b) } else { (b, a) };
          let value = reference[format!("covariance_{a}{b}").as_str()];
          assert!((r.covariance[i][j] - value).abs() <= 1e-9 + 1e-10 * value.abs());
          if i <= j {
            actual.insert(format!("covariance_{a}{b}"), r.covariance[i][j]);
          }
        }
      }
      actual.insert("mean_mu".into(), sum(r.fitted_means.iter().copied()) / 200.);
      actual.insert(
        "min_mu".into(),
        r.fitted_means.iter().copied().fold(f64::INFINITY, f64::min),
      );
      actual.insert(
        "max_mu".into(),
        r.fitted_means
          .iter()
          .copied()
          .fold(f64::NEG_INFINITY, f64::max),
      );
      actual.insert(
        "mean_residual".into(),
        sum(y.iter().zip(&r.fitted_means).map(|(a, b)| a - b)) / 200.,
      );
      assert_eq!(actual.len(), 42);
      for (name, value) in reference {
        let a = actual[name];
        if name.starts_with("p_") && value > 0. {
          assert!((a / value - 1.).abs() <= 1e-9, "{name}: {a} vs {value}");
        } else {
          assert!(
            (a - value).abs() <= 1e-9 + 1e-10 * value.abs(),
            "{name}: {a} vs {value}"
          );
        }
      }
      assert_eq!(
        serde_json::from_str::<NormalPoissonResult>(&serde_json::to_string(&r).unwrap()).unwrap(),
        r
      );
      for (actual, line) in r.fitted_means.iter().zip(
        include_str!("../../../tests/fixtures/mma23_normal_re/predictions.csv")
          .lines()
          .skip(1),
      ) {
        let expected: f64 = line.split(',').nth(4).unwrap().parse().unwrap();
        assert!((actual - expected).abs() <= 1e-9 + 1e-10 * expected.abs());
      }
    }
  }
  #[test]
  fn multivariate_unbalanced_singletons_match_all57_direct_r_fields_and_predictions() {
    use std::collections::BTreeMap;
    let records: Vec<Vec<f64>> =
      include_str!("../tests/fixtures/normal_poisson/multivariate/input.csv")
        .lines()
        .skip(1)
        .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
        .collect();
    assert_eq!(records.len(), 120);
    assert!(records.iter().all(|row| row.len() == 6));
    let x: Vec<Vec<_>> = records.iter().map(|row| vec![row[4], row[5]]).collect();
    let y: Vec<_> = records.iter().map(|row| row[3]).collect();
    let keys: Vec<_> = records
      .iter()
      .map(|row| (row[1] as usize).to_string())
      .collect();
    let retained: Vec<_> = records.iter().map(|row| row[0] as usize).collect();
    for row in &records {
      assert_eq!(row[0], (row[1] - 1.) * 5. + row[2] - 1975.);
    }
    let sample = EstimationSample::new(retained, 200, 80, None, Some(keys.clone())).unwrap();
    let p = EstimationProblem::new("y", vec!["x".into(), "z".into()], y.clone(), x)
      .unwrap()
      .with_intercept_name("_cons")
      .with_sample(sample.clone());
    let before = p.clone();
    let r = fit_normal_poisson(&p, "id", &NormalPoissonOptions::default()).unwrap();
    assert_eq!(p, before);
    assert_eq!(r.sample, sample);
    assert_eq!(
      (
        r.observation_count,
        r.group_count,
        r.singleton_group_count,
        r.minimum_group_size,
        r.maximum_group_size
      ),
      (120, 40, 8, 1, 5)
    );
    assert_eq!(
      r.covariance_parameter_names,
      ["_cons", "x", "z", "lnsigma2"]
    );
    let reference: BTreeMap<_, _> =
      include_str!("../tests/fixtures/normal_poisson/multivariate/reference.csv")
        .lines()
        .skip(1)
        .map(|line| {
          let (k, v) = line.split_once(',').unwrap();
          (k, v.parse::<f64>().unwrap())
        })
        .collect();
    assert_eq!(reference.len(), 57);
    let mut actual = BTreeMap::from([
      ("observations".into(), 120.),
      ("groups".into(), 40.),
      ("zero_groups".into(), r.zero_total_group_count as f64),
      ("singletons".into(), 8.),
      ("log_likelihood".into(), r.log_likelihood),
      ("poisson_log_likelihood".into(), r.poisson_log_likelihood),
      ("lr_variance_zero".into(), r.variance_likelihood_ratio),
      (
        "p_lr_variance_zero".into(),
        r.variance_likelihood_ratio_p_value,
      ),
      ("sigma".into(), r.sigma),
      ("se_sigma".into(), r.sigma_standard_error),
      ("lower_sigma".into(), r.sigma_confidence_lower),
      ("upper_sigma".into(), r.sigma_confidence_upper),
      ("wald".into(), r.wald_statistic),
      ("wald_df".into(), r.wald_df as f64),
      ("wald_p".into(), r.wald_p_value),
    ]);
    let labels = ["intercept", "x", "z", "lnvariance"];
    let coefficients: Vec<_> = r
      .coefficients
      .iter()
      .chain(std::iter::once(&r.ln_variance))
      .collect();
    let parameters: Vec<_> = coefficients.iter().map(|c| c.value).collect();
    let design: Vec<_> = p
      .design_matrix
      .iter()
      .map(|row| std::iter::once(1.).chain(row.iter().copied()).collect())
      .collect();
    let mut grouped: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, key) in keys.iter().enumerate() {
      grouped.entry(key).or_default().push(i);
    }
    let e = evaluate(
      &design,
      &y,
      &grouped.into_values().collect::<Vec<_>>(),
      &parameters,
    )
    .unwrap();
    for (i, c) in coefficients.iter().enumerate() {
      for (prefix, value) in [
        ("estimate", c.value),
        ("se", c.standard_error),
        ("z", c.statistic),
        ("p", c.p_value),
        ("lower", c.confidence_lower),
        ("upper", c.confidence_upper),
        ("score", e.score[i] * 40.),
      ] {
        actual.insert(format!("{prefix}_{}", labels[i]), value);
      }
      for j in 0..4 {
        let (a, b) = if i <= j {
          (labels[i], labels[j])
        } else {
          (labels[j], labels[i])
        };
        let name = format!("covariance_{a}_{b}");
        let expected = reference[name.as_str()];
        assert!((r.covariance[i][j] - expected).abs() <= 1e-9 + 1e-10 * expected.abs());
        if i <= j {
          actual.insert(name, r.covariance[i][j]);
        }
      }
    }
    actual.insert("mean_mu".into(), sum(r.fitted_means.iter().copied()) / 120.);
    actual.insert(
      "min_mu".into(),
      r.fitted_means.iter().copied().fold(f64::INFINITY, f64::min),
    );
    actual.insert(
      "max_mu".into(),
      r.fitted_means
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max),
    );
    actual.insert(
      "mean_residual".into(),
      sum(y.iter().zip(&r.fitted_means).map(|(a, b)| a - b)) / 120.,
    );
    assert_eq!(actual.len(), 57);
    for (name, value) in reference {
      let a = actual[name];
      if (name.starts_with("p_") || name == "wald_p") && value > 0. {
        assert!((a / value - 1.).abs() <= 1e-9, "{name}: {a} vs {value}");
      } else {
        assert!(
          (a - value).abs() <= 1e-9 + 1e-10 * value.abs(),
          "{name}: {a} vs {value}"
        );
      }
    }
    let predicted: Vec<Vec<f64>> =
      include_str!("../tests/fixtures/normal_poisson/multivariate/predictions.csv")
        .lines()
        .skip(1)
        .map(|line| line.split(',').map(|s| s.parse().unwrap()).collect())
        .collect();
    assert_eq!(predicted.len(), 120);
    for (i, row) in predicted.iter().enumerate() {
      assert_eq!(row.len(), 8);
      assert_eq!(row[..6], records[i]);
      assert!((r.fitted_means[i] - row[6]).abs() <= 1e-9 + 1e-10 * row[6].abs());
      assert!((y[i] - r.fitted_means[i] - row[7]).abs() <= 1e-9 + 1e-10 * row[7].abs());
    }
    let mut changed = p.clone();
    for row in &mut changed.design_matrix {
      row[0] *= 1e3;
      row[1] *= 1e-3;
    }
    changed.design_matrix.reverse();
    changed.response.reverse();
    let changed_sample = changed.sample.as_mut().unwrap();
    changed_sample.retained_indices.reverse();
    changed_sample.cluster_groups.as_mut().unwrap().reverse();
    let scaled = fit_normal_poisson(&changed, "id", &NormalPoissonOptions::default()).unwrap();
    assert_eq!(scaled.sample, changed.sample.unwrap());
    let units = [1., 1e3, 1e-3, 1.];
    let scaled_values: Vec<_> = scaled
      .coefficients
      .iter()
      .chain(std::iter::once(&scaled.ln_variance))
      .map(|c| c.value)
      .collect();
    for i in 0..4 {
      assert!((scaled_values[i] * units[i] - parameters[i]).abs() < 1e-9);
      for j in 0..4 {
        assert!((scaled.covariance[i][j] * units[i] * units[j] - r.covariance[i][j]).abs() < 1e-9);
      }
    }
    assert!((scaled.wald_statistic - r.wald_statistic).abs() < 1e-9);
    for (a, b) in scaled.fitted_means.iter().zip(r.fitted_means.iter().rev()) {
      assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs());
    }
  }
  #[test]
  fn public_fit_preprocessing_rescaling_and_failure_contracts() {
    let p = control_problem();
    let base = fit_normal_poisson(&p, "id", &NormalPoissonOptions::default()).unwrap();
    for scale in [1e-3, 1e3] {
      let mut changed = p.clone();
      for row in &mut changed.design_matrix {
        row[0] *= scale;
      }
      let r = fit_normal_poisson(&changed, "id", &NormalPoissonOptions::default()).unwrap();
      assert!((r.coefficients[1].value * scale - base.coefficients[1].value).abs() < 1e-9);
      for (a, b) in r.fitted_means.iter().zip(&base.fitted_means) {
        assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs());
      }
      for i in 0..3 {
        for j in 0..3 {
          let a = if i == 1 { scale } else { 1. };
          let b = if j == 1 { scale } else { 1. };
          assert!((r.covariance[i][j] * a * b - base.covariance[i][j]).abs() < 1e-9);
        }
      }
    }
    for case in 0..10 {
      let mut bad = p.clone();
      match case {
        0 => bad.include_intercept = false,
        1 => bad.weights = Some(vec![1.; 200]),
        2 => bad.sample = None,
        3 => bad.sample.as_mut().unwrap().retained_indices[0] = 1,
        4 => {
          let sample = bad.sample.as_mut().unwrap();
          sample.total_observations = 40001;
          sample.dropped_observations = 39801;
        }
        5 => bad.response[0] = 0.5,
        6 => bad.predictor_names[0] = "lnsigma2".into(),
        7 => bad.sample.as_mut().unwrap().cluster_groups = Some(vec!["one".into(); 200]),
        8 => bad.design_matrix.iter_mut().for_each(|row| row[0] = 1.),
        9 => {
          bad.sample.as_mut().unwrap().total_observations = 13000;
          bad.sample.as_mut().unwrap().dropped_observations = 12800;
        }
        _ => unreachable!(),
      }
      assert!(
        fit_normal_poisson(&bad, "id", &NormalPoissonOptions::default()).is_err(),
        "case{case}"
      );
    }
    assert!(fit_normal_poisson(&p, "", &NormalPoissonOptions::default()).is_err());
  }
  #[test]
  fn copied_raw_initializer_preserves_rms_units_and_rejects_invalid_frames() {
    let original = control_problem();
    let base = fit_normal_poisson(&original, "id", &NormalPoissonOptions::default()).unwrap();
    let raw = [
      base.coefficients[0].value,
      base.coefficients[1].value,
      base.ln_variance.value,
    ];
    for unit in [1., 1e3, 1e-3] {
      let mut p = original.clone();
      for row in &mut p.design_matrix {
        row[0] *= unit;
      }
      let warm = [raw[0], raw[1] / unit, raw[2]];
      let fit = fit_with_rule(
        &p,
        "id",
        &NormalPoissonOptions::default(),
        REFINED,
        Some(&warm),
      )
      .unwrap();
      assert_eq!(p.sample, original.sample);
      assert!(fit.iterations <= 2);
      assert!((fit.coefficients[0].value - raw[0]).abs() < 1e-9);
      assert!((fit.coefficients[1].value * unit - raw[1]).abs() < 1e-9);
      assert!((fit.ln_variance.value - raw[2]).abs() < 1e-9);
      for i in 0..3 {
        for j in 0..3 {
          let a = if i == 1 { unit } else { 1. };
          let b = if j == 1 { unit } else { 1. };
          assert!((fit.covariance[i][j] * a * b - base.covariance[i][j]).abs() < 1e-9);
        }
      }
    }
    for invalid in [
      vec![],
      vec![0., 0.],
      vec![0., f64::NAN, 0.],
      vec![0., 0., 9.],
    ] {
      assert!(
        fit_with_rule(
          &original,
          "id",
          &NormalPoissonOptions::default(),
          REFINED,
          Some(&invalid)
        )
        .is_err()
      );
    }
  }

  #[test]
  fn stable_full_increment_matches_resolved_levels_and_rejects_rounded_noops() {
    let (x, y, groups) = control();
    let parameters = vec![1.2479608264839344, 0.5673452307402028, 0.];
    let current = evaluate(&x, &y, &groups, &parameters).unwrap();
    for multiplier in [-1., 1.] {
      let trial: Vec<_> = parameters
        .iter()
        .zip([1e-6, -2e-6, 1e-6])
        .map(|(b, d)| b + multiplier * d)
        .collect();
      let next = evaluate(&x, &y, &groups, &trial).unwrap();
      let delta = increment::stable_likelihood_increment(
        &x,
        &y,
        &groups,
        &parameters,
        &trial,
        &current.score,
        ORDINARY,
      )
      .unwrap()
      .unwrap();
      assert!((delta - (next.log_likelihood - current.log_likelihood)).abs() < 2e-10);
      assert_eq!(
        delta.is_sign_positive(),
        (next.log_likelihood - current.log_likelihood).is_sign_positive()
      );
    }
    assert!(
      increment::stable_likelihood_increment(
        &x,
        &y,
        &groups,
        &parameters,
        &parameters,
        &current.score,
        ORDINARY,
      )
      .unwrap_err()
      .to_string()
      .contains("no parameter progress")
    );
    for (dt, small) in [
      (1e-4, true),
      (f64::from_bits(1e-4_f64.to_bits() + 1), false),
    ] {
      let trial = vec![parameters[0], parameters[1], dt];
      assert_eq!(
        increment::stable_likelihood_increment(
          &x,
          &y,
          &groups,
          &parameters,
          &trial,
          &current.score,
          ORDINARY
        )
        .unwrap()
        .is_some(),
        small
      );
    }
    let large = vec![parameters[0] + 0.01, parameters[1], parameters[2]];
    assert!(
      increment::stable_likelihood_increment(
        &x,
        &y,
        &groups,
        &parameters,
        &large,
        &current.score,
        ORDINARY
      )
      .unwrap()
      .is_none()
    );
  }

  #[test]
  fn optimizer_matches_independent_control_point_covariance_and_predictions() {
    let (x, y, groups) = control();
    let fit = optimize(
      &x,
      &y,
      &groups,
      vec![1.2479608264839344, 0.5673452307402028, 0.],
      100,
    )
    .unwrap();
    assert!(fit.iterations <= 100 && fit.score_norm <= 1e-10 && fit.relative_step <= 1e-12);
    let reference: std::collections::BTreeMap<_, _> =
      include_str!("../../../tests/fixtures/mma23_normal_re/reference.csv")
        .lines()
        .skip(1)
        .map(|line| {
          let (k, v) = line.split_once(',').unwrap();
          (k, v.parse::<f64>().unwrap())
        })
        .collect();
    let close = |a: f64, b: f64| assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs(), "{a} vs {b}");
    for (i, name) in [
      "estimate_intercept",
      "estimate_slope",
      "estimate_lnvariance",
    ]
    .iter()
    .enumerate()
    {
      close(fit.parameters[i], reference[name]);
    }
    close(fit.evaluation.log_likelihood, reference["log_likelihood"]);
    for (i, a) in ['a', 'b', 't'].iter().enumerate() {
      for (j, b) in ['a', 'b', 't'].iter().enumerate().skip(i) {
        close(
          fit.inverse[i][j] / groups.len() as f64,
          reference[format!("covariance_{a}{b}").as_str()],
        );
      }
    }
    let predictions: Vec<_> =
      include_str!("../../../tests/fixtures/mma23_normal_re/predictions.csv")
        .lines()
        .skip(1)
        .collect();
    assert_eq!(predictions.len(), 200);
    for (i, line) in predictions.iter().enumerate() {
      let cells: Vec<_> = line.split(',').collect();
      assert_eq!(cells[0].parse::<usize>().unwrap(), i / 5 + 1);
      assert_eq!(cells[1].parse::<usize>().unwrap(), 1975 + i % 5);
      assert_eq!(cells[2].parse::<f64>().unwrap(), x[i][1]);
      assert_eq!(cells[3].parse::<f64>().unwrap(), y[i]);
      let mu =
        (fit.parameters[0] + fit.parameters[1] * x[i][1] + fit.parameters[2].exp() / 2.).exp();
      close(mu, cells[4].parse().unwrap());
      close(y[i] - mu, cells[5].parse().unwrap());
    }
  }
  #[test]
  fn optimizer_and_whitened_information_reject_unsupported_states() {
    let (x, y, groups) = control();
    let start = vec![1.2479608264839344, 0.5673452307402028, 0.];
    for limit in [0, 1, 101] {
      assert!(optimize(&x, &y, &groups, start.clone(), limit).is_err());
    }
    assert!(
      optimize(&x, &y, &groups, vec![start[0], start[1], 2.], 100)
        .unwrap_err()
        .to_string()
        .contains("quadrature disagreement")
    );
    for matrix in [
      vec![vec![1., 2.], vec![2., 1.]],
      vec![vec![1., 1. - 1e-14], vec![1. - 1e-14, 1.]],
      vec![vec![-1., 0.], vec![0., 1.]],
    ] {
      assert!(scaled_inverse(&matrix).is_none());
    }
    let inverse = scaled_inverse(&[vec![1e-12, 0.25], vec![0.25, 1e12]]).unwrap();
    assert!((inverse[0][0] / (1e12 / 0.9375) - 1.).abs() < 1e-12);
    assert!((inverse[1][1] / (1e-12 / 0.9375) - 1.).abs() < 1e-12);
  }
  #[test]
  fn fixed_rules_have_positive_tail_weights_and_independent_gaussian_moments() {
    for rule in [
      &GH80[..],
      &GH96[..],
      &refined_nodes::GH176[..],
      &refined_nodes::GH208[..],
    ] {
      assert!(
        rule
          .iter()
          .all(|(z, w)| z.is_finite() && w.is_finite() && *w > 0.)
      );
      for pair in rule.windows(2) {
        assert!(pair[0].0 < pair[1].0);
      }
      let mut exact = std::f64::consts::PI.sqrt();
      for power in 0..=12 {
        if power > 0 {
          exact *= (2 * power - 1) as f64 / 2.;
        }
        let actual = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power)));
        assert!(
          (actual / exact - 1.).abs() < 2e-13,
          "power{power}: {actual} vs {exact}"
        );
        let odd = sum(rule.iter().map(|(z, w)| w * z.powi(2 * power + 1)));
        assert!(odd.abs() < 2e-12 * (1. + exact), "odd power{power}: {odd}");
      }
    }
  }
  #[test]
  fn posterior_moments_match_direct_r_improper_integration() {
    let fixture = include_str!("../tests/fixtures/normal_poisson/posterior.csv");
    let cases: Vec<_> = fixture.lines().skip(1).collect();
    assert_eq!(cases.len(), 6);
    for line in cases {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      assert_eq!(v.len(), 11);
      for quadrature in [ORDINARY, REFINED] {
        let p = posterior_with_rule(v[0], v[1], v[2], quadrature).unwrap();
        let actual = [
          p.log_integral,
          p.mean_exp_u,
          p.mean_u2,
          p.var_exp_u,
          p.cov_exp_u_u2,
          p.var_u2,
          p.mean_u,
          p.cov_exp_u_u,
        ];
        for (&a, &b) in actual.iter().zip(&v[3..]) {
          assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs(), "{v:?}: {a} vs {b}");
        }
        assert!(p.max_relative_difference <= 1e-9);
        let variance = v[2].exp();
        let score = v[0] - v[1] * p.mode.exp() - p.mode / variance;
        assert!(
          score.abs() <= 1e-10 * (1. + v[0] + v[1] * p.mode.exp() + (p.mode / variance).abs())
        );
        assert_eq!(p.curvature, v[1] * p.mode.exp() + 1. / variance);
      }
    }
  }
  #[test]
  fn joint_multivariate_scores_and_information_match_richardson_differences() {
    let x = vec![
      vec![1., -1., 0.2],
      vec![1., 0.3, -0.8],
      vec![1., 1.2, 0.1],
      vec![1., -0.4, 0.7],
      vec![1., 0.2, -1.],
      vec![1., 1.1, 0.8],
      vec![1., -0.9, 0.5],
    ];
    let y = vec![0., 3., 5., 1., 4., 2., 8.];
    let groups = vec![vec![0, 1, 2], vec![3], vec![4, 5, 6]];
    let parameters = vec![0.2, 0.4, -0.3, 0.6];
    let e = evaluate(&x, &y, &groups, &parameters).unwrap();
    assert!(e.max_quadrature_difference <= 1e-9);
    for j in 0..4 {
      let difference = |h: f64| {
        let mut plus = parameters.clone();
        plus[j] += h;
        let a = evaluate(&x, &y, &groups, &plus).unwrap();
        let mut minus = parameters.clone();
        minus[j] -= h;
        let b = evaluate(&x, &y, &groups, &minus).unwrap();
        (
          (a.log_likelihood - b.log_likelihood) / (2. * h * groups.len() as f64),
          a.score
            .iter()
            .zip(b.score)
            .map(|(u, v)| -(u - v) / (2. * h))
            .collect::<Vec<_>>(),
        )
      };
      let (d1, i1) = difference(2e-4);
      let (d2, i2) = difference(1e-4);
      let derivative = (4. * d2 - d1) / 3.;
      assert!(
        (e.score[j] - derivative).abs() <= 1e-8 * (1. + e.score[j].abs()),
        "score{j}: {} vs {derivative}",
        e.score[j]
      );
      for i in 0..4 {
        let numeric = (4. * i2[i] - i1[i]) / 3.;
        assert!(
          (e.information[i][j] - numeric).abs() <= 1e-8 * (1. + e.information[i][j].abs()),
          "information{i},{j}: {} vs {numeric}",
          e.information[i][j]
        );
        assert_eq!(e.information[i][j], e.information[j][i]);
      }
    }
  }
  #[test]
  fn invalid_group_frames_and_fixed_predictors_reject_without_panics() {
    let x = vec![vec![1., 0.], vec![1., 1.]];
    let y = vec![0., 2.];
    let p = [0., 1., 0.];
    for groups in [
      vec![],
      vec![vec![]],
      vec![vec![0]],
      vec![vec![0, 1, 2]],
      vec![vec![0, 0, 1]],
    ] {
      assert!(evaluate(&x, &y, &groups, &p).is_err());
    }
    assert!(evaluate(&x, &y, &[vec![0, 1]], &[0., 41., 0.]).is_err());
    assert!(evaluate(&[vec![], vec![]], &y, &[vec![0, 1]], &p).is_err());
  }
  #[test]
  fn stein_score_and_information_avoid_high_count_subtraction() {
    for line in include_str!("../tests/fixtures/normal_poisson/posterior.csv")
      .lines()
      .skip(1)
    {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      let (s, m, t) = (v[0], v[1], v[2]);
      let variance = t.exp();
      let p = posterior(s, m, t).unwrap();
      // Integration by parts: S-E(lambda)=E(u)/v and
      // E(lambda)-Var(lambda)=Cov(lambda,u)/v. The latter avoids O(S) cancellation.
      let score = p.mean_u / variance;
      assert!(
        (score - (s - m * p.mean_exp_u)).abs() <= 1e-9 + 1e-10 * (s + (m * p.mean_exp_u).abs())
      );
      let information = m * p.cov_exp_u_u / variance;
      assert!(information > 0. && information.is_finite());
      let difference = |h: f64| {
        let plus = posterior(s, m * h.exp(), t).unwrap();
        let minus = posterior(s, m * (-h).exp(), t).unwrap();
        -(plus.mean_u - minus.mean_u) / (2. * h * variance)
      };
      let derivative = (4. * difference(1e-4) - difference(2e-4)) / 3.;
      assert!(
        (information - derivative).abs() <= 1e-8 * (1. + information.abs()),
        "{v:?}: {information} vs {derivative}"
      );
    }
  }
  #[test]
  fn high_variance_control_start_rejects_actual_intensity_moment_disagreement() {
    let source = include_str!("../../../tests/fixtures/mma23_gamma_re_bootstrap/source.asc");
    let mut rejected = 0;
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
      let tokens: Vec<_> = line.split_whitespace().collect();
      let total = tokens[20..25]
        .iter()
        .map(|v| v.parse::<f64>().unwrap())
        .sum();
      // Independent R Poisson GLM start on the existing 200-record control.
      let m = tokens[10..15]
        .iter()
        .map(|v| (1.2479608264839344 + 0.5673452307402028 * v.parse::<f64>().unwrap()).exp())
        .sum();
      if let Err(error) = posterior(total, m, 2.) {
        assert!(
          error.to_string().contains("quadrature disagreement"),
          "{error}"
        );
        rejected += 1;
      }
    }
    assert!(
      rejected > 0,
      "high-variance start must expose actual 80/96 disagreement"
    );
  }
  #[test]
  fn invalid_parameters_and_unbracketed_modes_are_errors() {
    for (s, m, t) in [
      (0., 0., 0.),
      (-1., 1., 0.),
      (1.5, 1., 0.),
      (100001., 1., 0.),
      (1., 1., 8.),
      (1., 1., -8.),
      (f64::NAN, 1., 0.),
      (1., f64::INFINITY, 0.),
      (100000., 1e-100, 0.),
    ] {
      assert!(posterior(s, m, t).is_err(), "{s},{m},{t}");
    }
  }
}
