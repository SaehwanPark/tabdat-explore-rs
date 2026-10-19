//! Bounded unweighted NB2 ML with jointly estimated log dispersion.

use crate::binary::{BinaryCluster, BinaryOmnibusWald, omnibus_wald};
use crate::distributions::chi_square_tail;
use crate::matrix::{add_compensated, invert, qr_decompose_with_response};
use crate::poisson::deviance_part;
use crate::{EstimationProblem, EstimationSample, PoissonEstimate, StatsError, fit_poisson};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Bounds the per-iteration integer gamma-ratio recurrence, not data truncation.
pub const MAX_NB2_COUNT: f64 = 100_000.0;
const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;
const LOG_ALPHA_BOUND: f64 = 12.0;

/// Fixed normal coefficient/effect fields shared with the installed count fit.
pub type Nb2Estimate = PoissonEstimate;
/// Shared one-way group metadata; asymptotic normal, not G-1 t inference.
pub type Nb2Cluster = BinaryCluster;

#[derive(Clone, Copy)]
enum Nb2Covariance<'a> {
  ModelBased,
  Robust,
  Cluster(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nb2Options {
  /// Outer Newton iterations per full/restricted fit; each can evaluate
  /// multiple damped/backtracked candidates.
  pub max_iterations: usize,
}
impl Default for Nb2Options {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Alpha intervals transform the log-alpha interval, not alpha +/- z*SE.
/// No normal alpha=0 test is implied: that null lies on the model boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nb2Dispersion {
  pub alpha: f64,
  pub log_alpha: f64,
  pub alpha_standard_error: f64,
  pub log_alpha_standard_error: f64,
  pub alpha_confidence_lower: Option<f64>,
  pub alpha_confidence_upper: Option<f64>,
  pub log_alpha_confidence_lower: Option<f64>,
  pub log_alpha_confidence_upper: Option<f64>,
}

/// Covariance order is mean-model parameters followed by lnalpha. Model-based
/// bread is full inverse observed information, not conditional GLM/Fisher V.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nb2Result {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub robust: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster: Option<Nb2Cluster>,
  pub observation_count: usize,
  pub zero_count: usize,
  pub count_sum: f64,
  pub coefficients: Vec<Nb2Estimate>,
  pub covariance: Vec<Vec<f64>>,
  pub dispersion: Nb2Dispersion,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub null_alpha: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  pub likelihood_ratio_p_value: Option<f64>,
  pub omnibus_wald: Option<BinaryOmnibusWald>,
  pub mcfadden_r_squared: f64,
  pub residual_deviance: f64,
  pub pearson_chi_square: f64,
  pub df_residual: usize,
  pub pearson_dispersion: f64,
  /// Full-model outer iterations, excluding candidate and restricted-fit work.
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_means: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nb2MarginalEffects {
  pub robust: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster: Option<Nb2Cluster>,
  pub observation_count: usize,
  pub effects: Vec<Nb2Estimate>,
}

fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("nbreg NB2: {message}"))
}
fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("nbreg NB2: {message}"))
}
fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|x| x.abs()).fold(0.0, f64::max)
}
fn normal_inference(name: String, value: f64, variance: f64) -> Result<Nb2Estimate, StatsError> {
  if !value.is_finite() || !variance.is_finite() || variance < 0.0 {
    return Err(failed("nonfinite estimate/variance or negative variance"));
  }
  let se = variance.sqrt();
  let z = value / se;
  let p = chi_square_tail(z * z, 1);
  let lower = value - NORMAL_95 * se;
  let upper = value + NORMAL_95 * se;
  let available = se > 0.0 && [z, p, lower, upper].iter().all(|v| v.is_finite());
  Ok(Nb2Estimate {
    name,
    value,
    standard_error: se,
    statistic: available.then_some(z),
    p_value: available.then_some(p),
    confidence_lower: available.then_some(lower),
    confidence_upper: available.then_some(upper),
  })
}

// Gamma(y+r)/(Gamma(r)*y!) and its dispersion derivatives by integer recurrence.
// No cancellation of large log-gamma/digamma terms or unbounded per-row loops.
struct CountCache {
  log_coefficient: Vec<f64>,
  score_sum: Vec<f64>,
  curvature_sum: Vec<f64>,
}
impl CountCache {
  fn new(maximum: usize, r: f64) -> Self {
    let mut result = Self {
      log_coefficient: vec![0.0; maximum + 1],
      score_sum: vec![0.0; maximum + 1],
      curvature_sum: vec![0.0; maximum + 1],
    };
    let (mut c, mut a, mut b) = (0.0, 0.0, 0.0);
    let (mut ec, mut ea, mut eb) = (0.0, 0.0, 0.0);
    for j in 0..maximum {
      let jf = j as f64;
      let term = if j == 0 {
        r.ln()
      } else {
        ((r - 1.0) / (jf + 1.0)).ln_1p()
      };
      add_compensated(&mut c, &mut ec, term);
      add_compensated(&mut a, &mut ea, jf / (r + jf));
      add_compensated(&mut b, &mut eb, (jf / (r + jf)) * (r / (r + jf)));
      result.log_coefficient[j + 1] = c;
      result.score_sum[j + 1] = a;
      result.curvature_sum[j + 1] = b;
    }
    result
  }
}
fn softplus(value: f64) -> f64 {
  if value > 0.0 {
    value + (-value).exp().ln_1p()
  } else {
    value.exp().ln_1p()
  }
}
// -log(1-q)-q, using its positive series when subtraction would lose precision.
fn log_ratio_remainder(q: f64, log_ratio: f64) -> f64 {
  if q >= 0.01 {
    return log_ratio - q;
  }
  let mut term = q * q;
  let mut sum = 0.0;
  for j in 2..=100 {
    let next = sum + term / j as f64;
    if next == sum {
      return sum;
    }
    sum = next;
    term *= q;
  }
  f64::NAN
}
fn nb_deviance_part(y: f64, mu: f64, r: f64, eta: f64, log_ratio: f64) -> f64 {
  if y == 0.0 {
    return r * log_ratio;
  }
  let delta = y - mu;
  if delta.abs() < 0.1 * mu {
    // Difference of two Poisson deviances can erase the NB deviance when
    // r << mu. Expand their difference, preserving 1-(mu/(mu+r))^(j-1).
    let log_inverse_ratio = (r / mu).ln_1p();
    let mut power = delta * (delta / mu);
    let mut sum = 0.0;
    for j in 2..=100 {
      let factor = (-((j - 1) as f64) * log_inverse_ratio).exp_m1();
      let next = sum - power * factor / (j * (j - 1)) as f64;
      if next == sum {
        return sum;
      }
      sum = next;
      power *= -delta / mu;
    }
    return f64::NAN;
  }
  deviance_part(y, mu, eta) - deviance_part(y + r, mu + r, (mu + r).ln())
}
struct Evaluation {
  means: Vec<f64>,
  scores: Vec<Vec<f64>>,
  score: Vec<f64>,
  information: Vec<Vec<f64>>,
  log_likelihood: f64,
  deviance: f64,
  pearson: f64,
}
fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  gamma: &[f64],
  dispersion_scale: f64,
) -> Result<Evaluation, StatsError> {
  let k = gamma.len() - 1;
  let a = gamma[k] / dispersion_scale;
  if !a.is_finite() || a.abs() >= LOG_ALPHA_BOUND {
    return Err(failed(
      "log dispersion is at/outside the supported (-12,12) interior",
    ));
  }
  let r = (-a).exp();
  let maximum = y.iter().copied().fold(0.0, f64::max) as usize;
  let cache = CountCache::new(maximum, r);
  let mut result = Evaluation {
    means: Vec::with_capacity(y.len()),
    scores: Vec::with_capacity(y.len()),
    score: vec![0.0; k + 1],
    information: vec![vec![0.0; k + 1]; k + 1],
    log_likelihood: 0.0,
    deviance: 0.0,
    pearson: 0.0,
  };
  // Compensate the long row sums: small covariance errors can be amplified
  // by the multi-slope Wald quadratic even when individual SEs look unchanged.
  let mut score_correction = vec![0.0; k + 1];
  let mut information_correction = vec![vec![0.0; k + 1]; k + 1];
  let (mut ll_correction, mut deviance_correction, mut pearson_correction) = (0.0, 0.0, 0.0);
  for (row, &y) in x.iter().zip(y) {
    let eta: f64 = row.iter().zip(gamma).map(|(x, b)| x * b).sum();
    let mu = eta.exp();
    if !eta.is_finite() || !mu.is_finite() || mu <= 0.0 {
      return Err(failed("positive fitted mean is not representable"));
    }
    let total = r + mu;
    let q = mu / total;
    let ratio = softplus(eta + a);
    let remainder = log_ratio_remainder(q, ratio);
    let eta_score = (y - mu) * (r / total);
    let cross = q * eta_score;
    let index = y as usize;
    let a_score = cache.score_sum[index] + r * remainder - y * q;
    let a_information = r * remainder + cross - cache.curvature_sum[index];
    let weight = (mu * (r / total)) * ((r + y) / total);
    let mut score: Vec<f64> = row.iter().map(|x| x * eta_score).collect();
    score.push(a_score / dispersion_scale);
    for (i, s) in score.iter().enumerate() {
      add_compensated(&mut result.score[i], &mut score_correction[i], *s);
      if i < k {
        for (j, t) in row.iter().enumerate() {
          add_compensated(
            &mut result.information[i][j],
            &mut information_correction[i][j],
            row[i] * t * weight,
          );
        }
        add_compensated(
          &mut result.information[i][k],
          &mut information_correction[i][k],
          row[i] * cross / dispersion_scale,
        );
        result.information[k][i] = result.information[i][k];
      }
    }
    add_compensated(
      &mut result.information[k][k],
      &mut information_correction[k][k],
      a_information / (dispersion_scale * dispersion_scale),
    );
    add_compensated(
      &mut result.log_likelihood,
      &mut ll_correction,
      cache.log_coefficient[index] - y * softplus(-a - eta) - r * ratio,
    );
    let deviance = nb_deviance_part(y, mu, r, eta, ratio);
    if !deviance.is_finite() || deviance < 0.0 {
      return Err(failed("nonfinite or negative NB deviance"));
    }
    add_compensated(
      &mut result.deviance,
      &mut deviance_correction,
      2.0 * deviance,
    );
    // mu*(1+alpha*mu) is the NB2 variance, not Poisson variance=mu.
    let variance = mu * (1.0 + mu / r);
    if !variance.is_finite() || variance <= 0.0 {
      return Err(failed("positive NB2 variance is not representable"));
    }
    let standardized = (y - mu) / variance.sqrt();
    add_compensated(
      &mut result.pearson,
      &mut pearson_correction,
      standardized * standardized,
    );
    result.means.push(mu);
    result.scores.push(score);
  }
  if !result.log_likelihood.is_finite()
    || !result.deviance.is_finite()
    || !result.pearson.is_finite()
    || result
      .score
      .iter()
      .chain(result.information.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed(
      "nonfinite likelihood, score, curvature or diagnostics",
    ));
  }
  Ok(result)
}

// Positive curvature is required for the FINAL undamped information, not just
// positive covariance diagonal entries. Intermediate damping never changes V.
fn positive_inverse(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
  let n = matrix.len();
  let mut l = vec![vec![0.0; n]; n];
  for i in 0..n {
    for j in 0..=i {
      let prior: f64 = (0..j).map(|s| l[i][s] * l[j][s]).sum();
      let value = matrix[i][j] - prior;
      if i == j {
        if !value.is_finite() || value <= 1e-12 {
          return None;
        }
        l[i][j] = value.sqrt();
      } else {
        l[i][j] = value / l[j][j];
      }
    }
  }
  invert(matrix)
    .ok()
    .filter(|v| v.iter().flatten().all(|v| v.is_finite()))
}
fn times(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
  matrix
    .iter()
    .map(|row| row.iter().zip(vector).map(|(a, b)| a * b).sum())
    .collect()
}
#[derive(Clone)]
struct CoreFit {
  values: Vec<f64>,
  covariance: Vec<Vec<f64>>,
  means: Vec<f64>,
  log_likelihood: f64,
  deviance: f64,
  pearson: f64,
  iterations: usize,
  score_norm: f64,
  step_norm: f64,
}
fn joint_score_sandwich(
  bread: &[Vec<f64>],
  scores: &[Vec<f64>],
  groups: Option<&[String]>,
) -> Vec<Vec<f64>> {
  let p = bread.len();
  let grouped;
  let (scores, multiplier) = if let Some(groups) = groups {
    let mut sums: BTreeMap<&str, Vec<(f64, f64)>> = BTreeMap::new();
    for (score, group) in scores.iter().zip(groups) {
      let sum = sums
        .entry(group.as_str())
        .or_insert_with(|| vec![(0.0, 0.0); p]);
      for ((sum, correction), value) in sum.iter_mut().zip(score) {
        add_compensated(sum, correction, *value);
      }
    }
    let g = sums.len();
    // Aggregate the FULL beta/lnalpha score before outer products. Dropping
    // cross parameters or within-person cross terms changes the estimator's V.
    grouped = sums
      .into_values()
      .map(|row| row.into_iter().map(|(s, _)| s).collect())
      .collect::<Vec<Vec<f64>>>();
    (grouped.as_slice(), g as f64 / (g - 1) as f64)
  } else {
    (scores, scores.len() as f64 / (scores.len() - 1) as f64)
  };
  let mut v = vec![vec![0.0; p]; p];
  let mut corrections = vec![vec![0.0; p]; p];
  for score in scores {
    let influence = times(bread, score);
    for (i, row) in v.iter_mut().enumerate() {
      for (j, cell) in row.iter_mut().enumerate() {
        add_compensated(cell, &mut corrections[i][j], influence[i] * influence[j]);
      }
    }
  }
  for row in &mut v {
    for value in row {
      *value *= multiplier;
    }
  }
  v
}
fn optimize(
  x: &[Vec<f64>],
  y: &[f64],
  beta: &[f64],
  initial_means: &[f64],
  robust: bool,
  groups: Option<&[String]>,
  options: &Nb2Options,
) -> Result<CoreFit, StatsError> {
  let k = beta.len();
  let n = y.len();
  let mut scales: Vec<f64> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])))
    .collect();
  scales.push((n as f64).sqrt());
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid("invalid design norms"));
  }
  let x: Vec<Vec<f64>> = x
    .iter()
    .map(|row| row.iter().zip(&scales).map(|(x, s)| x / s).collect())
    .collect();
  if k > 0 {
    qr_decompose_with_response(&x, &vec![0.0; n])?;
  }
  let numerator: f64 = y
    .iter()
    .zip(initial_means)
    .map(|(y, mu)| (y - mu).powi(2) - mu)
    .sum();
  let denominator: f64 = initial_means.iter().map(|mu| mu * mu).sum();
  let start = (numerator / denominator).clamp((-8.0_f64).exp(), 8.0_f64.exp());
  if !start.is_finite() {
    return Err(failed("initial dispersion is not representable"));
  }
  let mut gamma: Vec<f64> = beta.iter().zip(&scales).map(|(b, s)| b * s).collect();
  gamma.push(start.ln() * scales[k]);
  for iteration in 1..=options.max_iterations {
    let current = evaluate(&x, y, &gamma, scales[k])?;
    let inverse = positive_inverse(&current.information);
    let full_step = inverse.as_ref().map(|bread| times(bread, &current.score));
    let score_norm = maximum_absolute(&current.score);
    if let (Some(bread), Some(step)) = (&inverse, &full_step)
      && score_norm <= TOLERANCE
      && maximum_absolute(step) <= TOLERANCE
    {
      let covariance = if robust {
        joint_score_sandwich(bread, &current.scores, groups)
      } else {
        bread.clone()
      };
      let covariance: Vec<Vec<f64>> = covariance
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
      let values: Vec<f64> = gamma.iter().zip(&scales).map(|(g, s)| g / s).collect();
      if values
        .iter()
        .chain(covariance.iter().flatten())
        .any(|v| !v.is_finite())
      {
        return Err(failed(
          "fit/covariance is not representable in original units",
        ));
      }
      return Ok(CoreFit {
        values,
        covariance,
        means: current.means,
        log_likelihood: current.log_likelihood,
        deviance: current.deviance,
        pearson: current.pearson,
        iterations: iteration,
        score_norm,
        step_norm: maximum_absolute(step),
      });
    }
    let mut accepted = None;
    // Try undamped Newton first, then bounded diagonal damping when curvature
    // is indefinite or no improving finite Newton step is found.
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
      let directional: f64 = step.iter().zip(&current.score).map(|(a, b)| a * b).sum();
      if !directional.is_finite() || directional <= 0.0 {
        continue;
      }
      let mut fraction = 1.0;
      for _ in 0..50 {
        let candidate: Vec<f64> = gamma
          .iter()
          .zip(&step)
          .map(|(b, d)| b + fraction * d)
          .collect();
        if let Ok(next) = evaluate(&x, y, &candidate, scales[k]) {
          let slack = 1e-12 * (1.0 + current.log_likelihood.abs());
          if next.log_likelihood >= current.log_likelihood + 1e-4 * fraction * directional - slack {
            accepted = Some(candidate);
            break;
          }
        }
        fraction *= 0.5;
      }
      if accepted.is_some() {
        break;
      }
    }
    gamma = accepted.ok_or_else(|| {
      failed("no improving finite positive-curvature step; possible dispersion boundary")
    })?;
  }
  Err(failed(
    "iteration limit without absolute score and full undamped-step convergence; possible dispersion boundary",
  ))
}

pub fn fit_nb2(problem: &EstimationProblem, robust: bool) -> Result<Nb2Result, StatsError> {
  fit_nb2_with_options(problem, robust, &Nb2Options::default())
}
pub fn fit_nb2_with_options(
  problem: &EstimationProblem,
  robust: bool,
  options: &Nb2Options,
) -> Result<Nb2Result, StatsError> {
  fit_nb2_covariance(
    problem,
    if robust {
      Nb2Covariance::Robust
    } else {
      Nb2Covariance::ModelBased
    },
    options,
  )
}

/// Full joint beta/lnalpha group-score sandwich with G/(G-1), normal inference
/// and no LR reference. Explicit groups must align with retained sample rows.
pub fn fit_nb2_clustered(
  problem: &EstimationProblem,
  variable: &str,
) -> Result<Nb2Result, StatsError> {
  fit_nb2_covariance(
    problem,
    Nb2Covariance::Cluster(variable),
    &Nb2Options::default(),
  )
}

fn fit_nb2_covariance(
  problem: &EstimationProblem,
  covariance_type: Nb2Covariance<'_>,
  options: &Nb2Options,
) -> Result<Nb2Result, StatsError> {
  problem.validate()?;
  let robust = !matches!(covariance_type, Nb2Covariance::ModelBased);
  if options.max_iterations == 0 {
    return Err(invalid("max_iterations must be positive"));
  }
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid("weights are not implemented"));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|s| s.cluster_groups.as_deref());
  let cluster = match covariance_type {
    Nb2Covariance::Cluster(variable) => {
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
      Some(Nb2Cluster {
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
    .any(|y| *y < 0.0 || y.fract() != 0.0 || *y > MAX_NB2_COUNT)
  {
    return Err(invalid("outcome must be an exact integer in 0..100000"));
  }
  let (n, k) = (problem.observation_count(), problem.parameter_count());
  if n <= k + 1 {
    return Err(invalid(
      "more observations than mean parameters plus dispersion are required",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k || names.iter().any(|n| n == "lnalpha") {
    return Err(invalid(
      "parameter names must be distinct; lnalpha is reserved",
    ));
  }
  let count_sum: f64 = problem.response.iter().sum();
  if count_sum == 0.0 {
    return Err(invalid("all-zero outcomes have no finite interior NB2 fit"));
  }
  let x: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::repeat_n(1.0, usize::from(problem.include_intercept))
        .chain(row.iter().copied())
        .collect()
    })
    .collect();
  // Starting coefficients use the SAME sample/design, without asking the
  // noncluster Poisson API to interpret NB2's attached group metadata.
  let poisson = if groups.is_some() {
    let mut starting = problem.clone();
    starting.sample.as_mut().unwrap().cluster_groups = None;
    fit_poisson(&starting, false)
  } else {
    fit_poisson(problem, false)
  }
  .map_err(|_| failed("Poisson starting fit unavailable"))?;
  let beta: Vec<f64> = poisson.coefficients.iter().map(|c| c.value).collect();
  let fit = optimize(
    &x,
    &problem.response,
    &beta,
    &poisson.fitted_means,
    robust,
    groups,
    options,
  )?;
  let null = if k == 1 && problem.include_intercept {
    fit.clone()
  } else {
    let mean = count_sum / n as f64;
    let (null_x, null_beta, null_means) = if problem.include_intercept {
      (vec![vec![1.0]; n], vec![mean.ln()], vec![mean; n])
    } else {
      (vec![vec![]; n], vec![], vec![1.0; n])
    };
    optimize(
      &null_x,
      &problem.response,
      &null_beta,
      &null_means,
      false,
      None,
      options,
    )?
  };
  let coefficients: Vec<_> = names
    .iter()
    .enumerate()
    .map(|(j, name)| normal_inference(name.clone(), fit.values[j], fit.covariance[j][j]))
    .collect::<Result<_, _>>()?;
  let a = fit.values[k];
  let alpha = a.exp();
  let variance = fit.covariance[k][k];
  if !variance.is_finite() || variance < 0.0 {
    return Err(failed("invalid dispersion variance"));
  }
  let se = variance.sqrt();
  let (lower, upper) = (a - NORMAL_95 * se, a + NORMAL_95 * se);
  let available = se > 0.0
    && [lower, upper, lower.exp(), upper.exp(), alpha * se]
      .iter()
      .all(|v| v.is_finite());
  let dispersion = Nb2Dispersion {
    alpha,
    log_alpha: a,
    alpha_standard_error: alpha * se,
    log_alpha_standard_error: se,
    alpha_confidence_lower: available.then(|| lower.exp()),
    alpha_confidence_upper: available.then(|| upper.exp()),
    log_alpha_confidence_lower: available.then_some(lower),
    log_alpha_confidence_upper: available.then_some(upper),
  };
  let lr = 2.0 * (fit.log_likelihood - null.log_likelihood);
  if !lr.is_finite() || lr < -1e-10 * (1.0 + null.log_likelihood.abs()) {
    return Err(failed("full likelihood is below the nested NB null"));
  }
  let lr = lr.max(0.0);
  let df = problem.predictor_names.len();
  let p = (!robust && df > 0).then(|| chi_square_tail(lr, df));
  if p.is_some_and(|p| !p.is_finite()) {
    return Err(failed("nonfinite LR inference"));
  }
  let beta_covariance: Vec<Vec<f64>> = fit.covariance[..k]
    .iter()
    .map(|row| row[..k].to_vec())
    .collect();
  let wald = robust.then(|| {
    omnibus_wald(
      &fit.values[..k],
      &beta_covariance,
      problem.include_intercept,
      cluster.as_ref().map(|c| c.group_count),
    )
  });
  Ok(Nb2Result {
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    robust,
    cluster,
    observation_count: n,
    zero_count: problem.response.iter().filter(|&&y| y == 0.0).count(),
    count_sum,
    coefficients,
    covariance: fit.covariance,
    dispersion,
    log_likelihood: fit.log_likelihood,
    null_log_likelihood: null.log_likelihood,
    null_alpha: null.values.last().unwrap().exp(),
    likelihood_ratio: lr,
    likelihood_ratio_df: df,
    likelihood_ratio_p_value: p,
    omnibus_wald: wald,
    mcfadden_r_squared: -lr / (2.0 * null.log_likelihood),
    residual_deviance: fit.deviance,
    pearson_chi_square: fit.pearson,
    df_residual: n - k - 1,
    pearson_dispersion: fit.pearson / (n - k - 1) as f64,
    iterations: fit.iterations,
    max_abs_scaled_score: fit.score_norm,
    max_abs_scaled_newton_step: fit.step_norm,
    fitted_means: fit.means,
    sample: problem.sample.clone(),
  })
}

/// Full joint V is used; the direct lnalpha gradient is zero, not a reason to
/// replace the beta block with conditional/Fisher covariance.
pub fn nb2_average_marginal_effects(
  problem: &EstimationProblem,
  result: &Nb2Result,
) -> Result<Nb2MarginalEffects, StatsError> {
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
    || result.covariance.len() != k + 1
    || result.covariance.iter().any(|r| r.len() != k + 1)
  {
    return Err(invalid(
      "margins require the exact retained fit/design/sample",
    ));
  }
  if result
    .fitted_means
    .iter()
    .any(|mu| !mu.is_finite() || *mu <= 0.0)
    || result.coefficients.iter().any(|c| !c.value.is_finite())
    || result.covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(failed("margins require finite fitted quantities"));
  }
  let x: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::repeat_n(1.0, usize::from(problem.include_intercept))
        .chain(row.iter().copied())
        .collect()
    })
    .collect();
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
    gradient.push(0.0);
    let variance: f64 = gradient
      .iter()
      .enumerate()
      .map(|(i, a)| {
        gradient
          .iter()
          .enumerate()
          .map(|(j, b)| a * result.covariance[i][j] * b)
          .sum::<f64>()
      })
      .sum();
    effects.push(normal_inference(name.clone(), b * mean_mu, variance)?);
  }
  Ok(Nb2MarginalEffects {
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
  fn grouped_full_joint_scores_keep_within_group_and_cross_parameter_terms() {
    let bread = vec![vec![2., 0.5], vec![0.5, 1.]];
    let scores = vec![vec![1., 2.], vec![3., -1.], vec![-2., 4.], vec![0., -2.]];
    let groups = vec!["a".into(), "a".into(), "b".into(), "b".into()];
    let v = joint_score_sandwich(&bread, &scores, Some(&groups));
    assert_eq!(v, vec![vec![162.5, 45.], vec![45., 20.]]);
    assert_ne!(v, joint_score_sandwich(&bread, &scores, None));
    let scores = vec![vec![1., 2.], vec![-1., -2.], vec![3., -4.], vec![-3., 4.]];
    let zero = joint_score_sandwich(&bread, &scores, Some(&groups));
    assert_eq!(zero, vec![vec![0.; 2]; 2]);
    let b = normal_inference("beta".into(), 1., zero[0][0]).unwrap();
    assert_eq!(b.standard_error, 0.);
    assert!(
      b.statistic.is_none()
        && b.p_value.is_none()
        && b.confidence_lower.is_none()
        && b.confidence_upper.is_none()
    );
  }

  #[test]
  fn bounded_count_pmf_scores_and_observed_curvature_match_independent_r() {
    for line in include_str!("../tests/fixtures/nb2_scalar_reference.csv")
      .lines()
      .skip(1)
    {
      let v: Vec<f64> = line.split(',').map(|s| s.parse().unwrap()).collect();
      let (y, mu, a) = (v[0], v[1], v[2]);
      let e = evaluate(&[vec![1.0]], &[y], &[mu.ln(), a], 1.0).unwrap();
      let actual = [
        e.log_likelihood,
        e.score[0],
        e.score[1],
        e.information[0][0],
        e.information[0][1],
        e.information[1][1],
      ];
      for (i, (&actual, &expected)) in actual.iter().zip(&v[3..]).enumerate() {
        // This scalar stress grid includes y=100000 and r=exp(11). R's
        // digamma/trigamma DIFFERENCES amplify rounding; their derivative
        // comparison has a separate absolute 1e-8 plus 1e-12 relative bound.
        // Full fitted inference retains the stricter 1e-10 reference contract.
        let tolerance = if i == 0 {
          1e-10
        } else {
          1e-8 + 1e-12 * expected.abs()
        };
        assert!(
          (actual - expected).abs() <= tolerance,
          "y={y}, a={a}, quantity={i}: {actual} != {expected}, tol={tolerance}"
        );
      }
      assert!(e.deviance >= 0.0);
      let r = (-a).exp();
      let approximation = (y - mu).powi(2) / (mu * (1.0 + mu / r));
      if y >= 1000.0 {
        assert!((e.deviance / approximation - 1.0).abs() < 0.001);
      }
    }
  }

  #[test]
  fn nonrepresentable_means_variances_and_dispersion_bounds_fail() {
    for parameters in [
      [710.0, 0.0],
      [709.0, 0.0],
      [-1000.0, 0.0],
      [0.0, -12.0],
      [0.0, 12.0],
    ] {
      assert!(evaluate(&[vec![1.0]], &[1.0], &parameters, 1.0).is_err());
    }
  }

  #[test]
  fn observed_information_requires_positive_definite_curvature() {
    assert!(positive_inverse(&[vec![1.0, 2.0], vec![2.0, 1.0]]).is_none());
    assert!(positive_inverse(&[vec![1.0, 1.0], vec![1.0, 1.0]]).is_none());
    assert!(positive_inverse(&[vec![1.0, 0.0], vec![0.0, 1.0]]).is_some());
  }
}
