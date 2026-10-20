//! Unweighted log-link Poisson ML / independent-row or one-way clustered QMLE.

use crate::binary::{BinaryCluster, BinaryOmnibusWald, omnibus_wald};
use crate::distributions::chi_square_tail;
use crate::matrix::{
  add_compensated, qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r,
};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_POISSON_COUNT: f64 = 9_007_199_254_740_991.0;
const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Shared coefficient-only chi-square report contract; no family-specific refit.
pub type PoissonOmnibusWald = BinaryOmnibusWald;
/// Shared one-way score-sandwich metadata; normal, not G-1 Student-t inference.
pub type PoissonCluster = BinaryCluster;

#[derive(Clone, Copy)]
enum PoissonCovariance<'a> {
  ModelBased,
  Robust,
  Cluster(&'a str),
}

/// Fixed 95% normal inference. Zero-variance or unrepresentable inference is
/// unavailable, not a zero p-value or a spurious exact confidence interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoissonEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub confidence_lower: Option<f64>,
  pub confidence_upper: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoissonOptions {
  pub max_iterations: usize,
}
impl Default for PoissonOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Model-based inverse information, row-score sandwich with N/(N-1), or
/// group-score sandwich with G/(G-1), never OLS HC1. LR inference assumes the
/// independent Poisson distribution; robust/clustered fits retain descriptive
/// likelihoods but suppress the LR reference/tail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoissonResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub offset_variable: Option<String>,
  pub robust: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster: Option<PoissonCluster>,
  pub observation_count: usize,
  pub zero_count: usize,
  pub count_sum: f64,
  pub coefficients: Vec<PoissonEstimate>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub omnibus_wald: Option<PoissonOmnibusWald>,
  pub mcfadden_r_squared: f64,
  pub residual_deviance: f64,
  pub pearson_chi_square: f64,
  pub df_residual: usize,
  pub pearson_dispersion: f64,
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_means: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

/// Continuous predictor-column derivatives, conditional on retained X.
/// Not discrete changes, original-variable chain rules or random-X uncertainty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoissonMarginalEffects {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub offset_variable: Option<String>,
  pub robust: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster: Option<PoissonCluster>,
  pub observation_count: usize,
  pub effects: Vec<PoissonEstimate>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("poisson: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("poisson: {message}"))
}

fn inference(name: String, value: f64, variance: f64) -> Result<PoissonEstimate, StatsError> {
  if !value.is_finite() || !variance.is_finite() || variance < 0.0 {
    return Err(failed(
      "coefficient/effect or variance is non-finite or negative",
    ));
  }
  let standard_error = variance.sqrt();
  let z = value / standard_error;
  let p = chi_square_tail(z * z, 1);
  let lower = value - NORMAL_95 * standard_error;
  let upper = value + NORMAL_95 * standard_error;
  let available = standard_error > 0.0 && [z, p, lower, upper].iter().all(|v| v.is_finite());
  Ok(PoissonEstimate {
    name,
    value,
    standard_error,
    statistic: available.then_some(z),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lower),
    confidence_upper: available.then_some(upper),
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

// log P(Y=y | mu=y). The Stirling correction avoids cancellation of terms
// of size y*log(y) when the log probability itself is only O(log(y)).
fn saturated_log_probability(y: f64) -> f64 {
  if y == 0.0 {
    return 0.0;
  }
  if y < 16.0 {
    // Integer log-factorials preserve small-count PMF accuracy independently
    // of the more approximate general log-gamma helper.
    let log_factorial: f64 = (2..=y as u32).map(|i| f64::from(i).ln()).sum();
    return y * y.ln() - y - log_factorial;
  }
  let z = 1.0 / (y * y);
  let correction =
    (1.0 / 12.0 - z * (1.0 / 360.0 - z * (1.0 / 1260.0 - z * (1.0 / 1680.0 - z / 1188.0)))) / y;
  -0.5 * ((2.0 * std::f64::consts::PI).ln() + y.ln()) - correction
}

// y*log(y/mu)+mu-y, with a convergent atanh series near y=mu. Neither a
// rounded ratio of one nor subtraction of O(y) terms may erase deviance.
pub(crate) fn deviance_part(y: f64, mu: f64, eta: f64) -> f64 {
  if y == 0.0 {
    return mu;
  }
  if (y - mu).abs() < 0.1 * (y + mu) {
    let v = (y - mu) / (y + mu);
    let mut sum = (y - mu) * v;
    let mut term = 2.0 * y * v;
    for j in 1..=100 {
      term *= v * v;
      let next = sum + term / (2 * j + 1) as f64;
      if next == sum {
        return sum;
      }
      sum = next;
    }
    return f64::NAN;
  }
  y * (y.ln() - eta) + mu - y
}

struct Evaluation {
  means: Vec<f64>,
  residuals: Vec<f64>,
  log_likelihood: f64,
  deviance: f64,
}
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  beta: &[f64],
  saturated: &[f64],
  offsets: &[f64],
) -> Result<Evaluation, StatsError> {
  let mut result = Evaluation {
    means: Vec::with_capacity(y.len()),
    residuals: Vec::with_capacity(y.len()),
    log_likelihood: 0.0,
    deviance: 0.0,
  };
  for (((row, &y), &sat), &offset) in x.iter().zip(y).zip(saturated).zip(offsets) {
    let eta: f64 = row.iter().zip(beta).map(|(a, b)| a * b).sum::<f64>() + offset;
    let mu = eta.exp();
    if !eta.is_finite() || !mu.is_finite() || mu <= 0.0 {
      return Err(failed(
        "linear index/positive fitted mean is not representable",
      ));
    }
    let d = deviance_part(y, mu, eta);
    if !d.is_finite() || d < 0.0 {
      return Err(failed("deviance is non-finite or negative"));
    }
    result.log_likelihood += sat - d;
    result.deviance += 2.0 * d;
    result.means.push(mu);
    result.residuals.push(y - mu);
  }
  if !result.log_likelihood.is_finite() || !result.deviance.is_finite() {
    return Err(failed("likelihood/deviance accumulation is non-finite"));
  }
  Ok(result)
}
fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

fn clustered_covariance(
  bread: &[Vec<f64>],
  x: &[Vec<f64>],
  residuals: &[f64],
  groups: &[String],
) -> Vec<Vec<f64>> {
  let k = bread.len();
  let mut grouped: BTreeMap<&str, Vec<(f64, f64)>> = BTreeMap::new();
  for ((row, residual), group) in x.iter().zip(residuals).zip(groups) {
    let score = grouped
      .entry(group.as_str())
      .or_insert_with(|| vec![(0.0, 0.0); k]);
    for ((sum, correction), value) in score.iter_mut().zip(row) {
      add_compensated(sum, correction, value * residual);
    }
  }
  let correction = grouped.len() as f64 / (grouped.len() - 1) as f64;
  let mut result = vec![vec![0.0; k]; k];
  let mut errors = vec![vec![0.0; k]; k];
  // Group the scores BEFORE the outer product, retaining within-group cross
  // terms. Compensate both long group sums and the covariance accumulation.
  for score in grouped.values() {
    let influence: Vec<f64> = bread
      .iter()
      .map(|row| row.iter().zip(score).map(|(b, (s, _))| b * s).sum())
      .collect();
    for i in 0..k {
      for j in 0..k {
        add_compensated(
          &mut result[i][j],
          &mut errors[i][j],
          influence[i] * influence[j],
        );
      }
    }
  }
  for row in &mut result {
    for value in row {
      *value *= correction;
    }
  }
  result
}

pub fn fit_poisson(problem: &EstimationProblem, robust: bool) -> Result<PoissonResult, StatsError> {
  fit_poisson_with_options(problem, robust, &PoissonOptions::default())
}

/// Scaled Newton/Fisher QR and deterministic line search. A vanishing score
/// alone is not convergence at a Poisson boundary: the FULL step must be small.
/// This is a bounded numerical fit, not an analytic existence/separation test.
pub fn fit_poisson_with_options(
  problem: &EstimationProblem,
  robust: bool,
  options: &PoissonOptions,
) -> Result<PoissonResult, StatsError> {
  fit_poisson_covariance(
    problem,
    if robust {
      PoissonCovariance::Robust
    } else {
      PoissonCovariance::ModelBased
    },
    options,
    &vec![0.0; problem.observation_count()],
  )
}

/// Fit with a fixed, finite additive log-mean offset aligned to retained rows.
/// The offset is not a parameter; the nested null keeps it fixed. Clustered
/// fits require attached groups; `robust` and `cluster_variable` are exclusive.
/// Applications may attach the source name to `result.offset_variable` for
/// current-data prediction; this numerical entry point accepts only values.
pub fn fit_poisson_with_offset(
  problem: &EstimationProblem,
  offsets: &[f64],
  robust: bool,
  cluster_variable: Option<&str>,
) -> Result<PoissonResult, StatsError> {
  if robust && cluster_variable.is_some() {
    return Err(invalid("cannot combine robust and cluster"));
  }
  let covariance_type = match cluster_variable {
    Some(variable) => PoissonCovariance::Cluster(variable),
    None if robust => PoissonCovariance::Robust,
    None => PoissonCovariance::ModelBased,
  };
  fit_poisson_covariance(
    problem,
    covariance_type,
    &PoissonOptions::default(),
    offsets,
  )
}

/// One-way group-score sandwich, G/(G-1), with normal inference and no LR tail.
/// The explicitly attached groups must align with the retained estimation rows.
pub fn fit_poisson_clustered(
  problem: &EstimationProblem,
  variable: &str,
) -> Result<PoissonResult, StatsError> {
  fit_poisson_covariance(
    problem,
    PoissonCovariance::Cluster(variable),
    &PoissonOptions::default(),
    &vec![0.0; problem.observation_count()],
  )
}

fn fit_poisson_covariance(
  problem: &EstimationProblem,
  covariance_type: PoissonCovariance<'_>,
  options: &PoissonOptions,
  offsets: &[f64],
) -> Result<PoissonResult, StatsError> {
  problem.validate()?;
  if offsets.len() != problem.observation_count() || offsets.iter().any(|v| !v.is_finite()) {
    return Err(invalid(
      "offsets must be finite and align with the estimation sample",
    ));
  }
  let robust = !matches!(covariance_type, PoissonCovariance::ModelBased);
  if options.max_iterations == 0 {
    return Err(invalid("max_iterations must be positive"));
  }
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("weights are not implemented"));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|s| s.cluster_groups.as_ref());
  let cluster = match covariance_type {
    PoissonCovariance::Cluster(variable) => {
      if variable.trim().is_empty() {
        return Err(invalid("cluster variable must be named"));
      }
      let groups = groups.ok_or_else(|| invalid("cluster groups are required"))?;
      if groups.len() != problem.observation_count() {
        return Err(invalid(
          "cluster groups must align with the estimation sample",
        ));
      }
      let group_count = groups.iter().collect::<BTreeSet<_>>().len();
      if group_count < 2 {
        return Err(invalid(
          "cluster covariance requires at least two retained groups",
        ));
      }
      Some(PoissonCluster {
        variable: variable.to_owned(),
        group_count,
      })
    }
    _ => {
      if groups.is_some() {
        return Err(invalid("cluster groups require a clustered fit"));
      }
      None
    }
  };
  if problem
    .response
    .iter()
    .any(|y| *y < 0.0 || y.fract() != 0.0 || *y > MAX_POISSON_COUNT)
  {
    return Err(invalid(
      "outcome must be an exact nonnegative integer at most 2^53-1",
    ));
  }
  let (n, k) = (problem.observation_count(), problem.parameter_count());
  if n <= k {
    return Err(StatsError::InsufficientObservations(
      "poisson requires more observations than parameters".into(),
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid(
      "parameter names must be distinct, including the intercept",
    ));
  }
  let count_sum: f64 = problem.response.iter().sum();
  if !count_sum.is_finite() {
    return Err(invalid("count sum is non-finite"));
  }
  if problem.include_intercept && count_sum == 0.0 {
    return Err(invalid("all-zero outcome has no finite intercept fit"));
  }
  let mut x = full_design(problem);
  let scales: Vec<f64> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])))
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid("predictor norms must be finite and positive"));
  }
  for row in &mut x {
    for (value, scale) in row.iter_mut().zip(&scales) {
      *value /= scale;
    }
  }
  let zero = vec![0.0; n];
  qr_decompose_with_response(&x, &zero)?;
  let saturated: Vec<f64> = problem
    .response
    .iter()
    .map(|&y| saturated_log_probability(y))
    .collect();
  let mut beta = vec![0.0; k];
  if problem.include_intercept {
    let null_intercept = if offsets.iter().all(|v| *v == 0.0) {
      // Preserve the installed no-offset starting point exactly.
      (count_sum / n as f64).ln()
    } else {
      // log(sum(exp(offset))) avoids overflow in the intercept-only null MLE.
      let maximum = offsets.iter().copied().fold(f64::NEG_INFINITY, f64::max);
      let log_exposure_sum = maximum
        + offsets
          .iter()
          .map(|v| (v - maximum).exp())
          .sum::<f64>()
          .ln();
      count_sum.ln() - log_exposure_sum
    };
    beta[0] = null_intercept * scales[0];
  }
  let null_log_likelihood =
    evaluate(&x, &problem.response, &beta, &saturated, offsets)?.log_likelihood;
  for iteration in 1..=options.max_iterations {
    let current = evaluate(&x, &problem.response, &beta, &saturated, offsets)?;
    let weighted: Vec<Vec<f64>> = x
      .iter()
      .zip(&current.means)
      .map(|(row, mu)| row.iter().map(|v| v * mu.sqrt()).collect())
      .collect();
    let qr = qr_decompose_with_response(&weighted, &zero).map_err(|_| {
      failed("information is singular or numerically unidentified (possible boundary fit)")
    })?;
    let score: Vec<f64> = (0..k)
      .map(|j| {
        x.iter()
          .zip(&current.residuals)
          .map(|(row, r)| row[j] * r)
          .sum()
      })
      .collect();
    let mut intermediate = vec![0.0; k];
    for i in 0..k {
      let preceding: f64 = (0..i).map(|j| qr.r[j][i] * intermediate[j]).sum();
      intermediate[i] = (score[i] - preceding) / qr.r[i][i];
    }
    let step = solve_upper_triangular(&qr.r, &intermediate)?;
    if score.iter().chain(&step).any(|v| !v.is_finite()) {
      return Err(failed("Newton step or score is non-finite"));
    }
    let (score_norm, step_norm) = (maximum_absolute(&score), maximum_absolute(&step));
    if score_norm <= TOLERANCE && step_norm <= TOLERANCE {
      let bread = xtx_inverse_from_r(&qr.r)?;
      let scaled_covariance = if cluster.is_some() {
        clustered_covariance(&bread, &x, &current.residuals, groups.unwrap())
      } else if robust {
        let mut sandwich = vec![vec![0.0; k]; k];
        for (row, r) in x.iter().zip(&current.residuals) {
          let influence: Vec<f64> = bread
            .iter()
            .map(|b| b.iter().zip(row).map(|(a, x)| a * x).sum::<f64>() * r)
            .collect();
          for i in 0..k {
            for j in 0..k {
              sandwich[i][j] += influence[i] * influence[j];
            }
          }
        }
        for row in &mut sandwich {
          for v in row {
            *v *= n as f64 / (n - 1) as f64;
          }
        }
        sandwich
      } else {
        bread
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
          "covariance cannot be represented in original predictor units",
        ));
      }
      let coefficients: Vec<PoissonEstimate> = (0..k)
        .map(|j| inference(names[j].clone(), beta[j] / scales[j], covariance[j][j]))
        .collect::<Result<_, _>>()?;
      let values: Vec<f64> = coefficients.iter().map(|c| c.value).collect();
      let wald = robust.then(|| {
        omnibus_wald(
          &values,
          &covariance,
          problem.include_intercept,
          cluster.as_ref().map(|c| c.group_count),
        )
      });
      let lr = 2.0 * (current.log_likelihood - null_log_likelihood);
      if !lr.is_finite() || lr < -1e-10 * (1.0 + null_log_likelihood.abs()) {
        return Err(failed("likelihood is below the nested null model"));
      }
      let lr = lr.max(0.0);
      let df = problem.predictor_names.len();
      let p = (!robust && df > 0).then(|| chi_square_tail(lr, df));
      if p.is_some_and(|p| !p.is_finite()) {
        return Err(failed("likelihood-ratio inference is unavailable"));
      }
      let pearson: f64 = current
        .residuals
        .iter()
        .zip(&current.means)
        .map(|(r, mu)| (r / mu.sqrt()).powi(2))
        .sum();
      if !pearson.is_finite() {
        return Err(failed("Pearson diagnostic is non-finite"));
      }
      return Ok(PoissonResult {
        outcome: problem.outcome.clone(),
        predictor_names: problem.predictor_names.clone(),
        include_intercept: problem.include_intercept,
        offset_variable: None,
        robust,
        cluster,
        observation_count: n,
        zero_count: problem.response.iter().filter(|&&y| y == 0.0).count(),
        count_sum,
        coefficients,
        covariance,
        log_likelihood: current.log_likelihood,
        null_log_likelihood,
        likelihood_ratio: lr,
        likelihood_ratio_df: df,
        likelihood_ratio_p_value: p,
        omnibus_wald: wald,
        mcfadden_r_squared: -lr / (2.0 * null_log_likelihood),
        residual_deviance: current.deviance,
        pearson_chi_square: pearson,
        df_residual: n - k,
        pearson_dispersion: pearson / (n - k) as f64,
        iterations: iteration,
        max_abs_scaled_score: score_norm,
        max_abs_scaled_newton_step: step_norm,
        fitted_means: current.means,
        sample: problem.sample.clone(),
      });
    }
    let directional: f64 = score.iter().zip(&step).map(|(a, b)| a * b).sum();
    let mut fraction = 1.0;
    let mut accepted = None;
    for _ in 0..50 {
      let candidate: Vec<f64> = beta
        .iter()
        .zip(&step)
        .map(|(b, d)| b + fraction * d)
        .collect();
      if let Ok(next) = evaluate(&x, &problem.response, &candidate, &saturated, offsets) {
        let slack = 1e-12 * (1.0 + current.log_likelihood.abs());
        if next.log_likelihood >= current.log_likelihood + 1e-4 * fraction * directional - slack {
          accepted = Some(candidate);
          break;
        }
      }
      fraction *= 0.5;
    }
    beta = accepted.ok_or_else(|| failed("backtracking could not find a finite improving step"))?;
  }
  Err(failed(
    "iteration limit reached without absolute full-step and score convergence",
  ))
}

/// Pass the unmodified fitting problem alongside its result. Metadata and
/// dimensions are checked, not a data fingerprint; the application retains this
/// owned pair rather than reconstructing a design from current mutable data.
pub fn poisson_average_marginal_effects(
  problem: &EstimationProblem,
  result: &PoissonResult,
) -> Result<PoissonMarginalEffects, StatsError> {
  problem.validate()?;
  let (n, k) = (problem.observation_count(), problem.parameter_count());
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
    || result.fitted_means.len() != n
    || result.covariance.len() != k
    || result.covariance.iter().any(|r| r.len() != k)
  {
    return Err(invalid(
      "marginal effects require the exact retained fit/design/sample",
    ));
  }
  if result
    .fitted_means
    .iter()
    .any(|mu| !mu.is_finite() || *mu <= 0.0)
    || result.coefficients.iter().any(|c| !c.value.is_finite())
    || result.covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(failed("marginal effects require finite fit quantities"));
  }
  let x = full_design(problem);
  let mean_mu = result.fitted_means.iter().sum::<f64>() / n as f64;
  let mean_mu_x: Vec<f64> = (0..k)
    .map(|j| {
      x.iter()
        .zip(&result.fitted_means)
        .map(|(row, mu)| row[j] * mu)
        .sum::<f64>()
        / n as f64
    })
    .collect();
  let mut effects = Vec::new();
  for (column, name) in problem.predictor_names.iter().enumerate() {
    let index = column + usize::from(problem.include_intercept);
    let b = result.coefficients[index].value;
    let mut gradient: Vec<f64> = mean_mu_x.iter().map(|m| b * m).collect();
    gradient[index] += mean_mu;
    let variance: f64 = (0..k)
      .map(|i| {
        (0..k)
          .map(|j| gradient[i] * result.covariance[i][j] * gradient[j])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(name.clone(), b * mean_mu, variance)?);
  }
  Ok(PoissonMarginalEffects {
    offset_variable: result.offset_variable.clone(),
    robust: result.robust,
    cluster: result.cluster.clone(),
    observation_count: n,
    effects,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn log_probabilities_near_large_count_means_match_r_dpois_without_cancellation() {
    // R4.6.1 dpois(y, mu=y, log=TRUE), including the exact count bound.
    for (y, expected) in [
      (1.0, -0.9999999999999999),
      (15.0, -2.2785183673077407),
      (16.0, -2.310_440_550_244_173),
      (1e5, -6.6754020990231195),
      (1e12, -14.734449091169031),
      (MAX_POISSON_COUNT, -19.287338818043224),
    ] {
      let actual = saturated_log_probability(y);
      assert!(
        (actual - expected).abs() < 1e-13,
        "y={y}: {actual} != {expected}"
      );
    }
    let actual =
      saturated_log_probability(1e8) - deviance_part(1e8, 1e8 + 1.0, (1e8 + 1.0_f64).ln());
    assert!((actual - (-10.12927891101419)).abs() < 1e-13);
    assert_eq!(
      saturated_log_probability(0.0) - deviance_part(0.0, 1.0, 0.0),
      -1.0
    );
  }
}
