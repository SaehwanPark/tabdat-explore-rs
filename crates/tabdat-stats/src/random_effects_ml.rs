#![forbid(unsafe_code)]
// Indexed matrix expressions follow the observed-information formulas directly.
#![allow(clippy::needless_range_loop)]

use crate::{
  EstimationProblem, LeastSquaresResult, StatsError, fit_fixed_effects, fit_least_squares,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::f64::consts::PI;

/// Work-in-progress interior Gaussian ML kernel. No runtime command is installed.
/// Joint parameter order is intercept/slopes, log(sigma_e²), log(sigma_e²+T*sigma_u²).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomEffectsMlInterior {
  pub regression: LeastSquaresResult,
  pub joint_covariance: Vec<Vec<f64>>,
  pub sigma_e_squared: f64,
  pub sigma_u_squared: f64,
  pub eta: f64,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub null_sigma_e_squared: f64,
  pub null_sigma_u_squared: f64,
  pub likelihood_ratio: f64,
  /// Ordinary slope LR reference is withheld if the null unit variance is on its boundary.
  pub likelihood_ratio_p_value: Option<f64>,
  pub profile_score: f64,
  pub group_count: usize,
  pub group_size: usize,
}

struct Profile {
  fit: LeastSquaresResult,
  a: f64,
  v: f64,
  ll: f64,
  score: f64,
  within_residuals: Vec<f64>,
  between_residuals: Vec<f64>,
}

fn failed(message: &str) -> StatsError {
  StatsError::InvalidEstimation(message.into())
}

fn profile(
  problem: &EstimationProblem,
  means: &[Vec<f64>],
  response_means: &[f64],
  g: usize,
  eta: f64,
) -> Result<Profile, StatsError> {
  let q = (-eta).exp();
  let root = q.sqrt();
  let n = problem.response.len();
  let mut projected = problem.clone();
  projected.include_intercept = false;
  projected
    .predictor_names
    .insert(0, problem.intercept_name.clone());
  for i in 0..n {
    projected.response[i] = (problem.response[i] - response_means[i]) + root * response_means[i];
    for j in 0..problem.predictor_names.len() {
      projected.design_matrix[i][j] =
        (problem.design_matrix[i][j] - means[i][j]) + root * means[i][j];
    }
    projected.design_matrix[i].insert(0, root);
  }
  let fit = fit_least_squares(&projected)?;
  let b: Vec<_> = fit.coefficients.iter().map(|c| c.value).collect();
  let mut ew = Vec::with_capacity(n);
  let mut eb = Vec::with_capacity(n);
  for i in 0..n {
    ew.push(
      (problem.response[i] - response_means[i])
        - problem.design_matrix[i]
          .iter()
          .zip(&means[i])
          .zip(&b[1..])
          .map(|((x, m), b)| (x - m) * b)
          .sum::<f64>(),
    );
    eb.push(
      response_means[i]
        - b[0]
        - means[i]
          .iter()
          .zip(&b[1..])
          .map(|(x, b)| x * b)
          .sum::<f64>(),
    );
  }
  let sw = ew.iter().map(|e| e * e).sum::<f64>();
  let sb = eb.iter().map(|e| e * e).sum::<f64>();
  let s = sw + q * sb;
  let a = s / n as f64;
  let v = a / q;
  let ll = -0.5 * (n as f64 * ((2.0 * PI).ln() + 1.0 + a.ln()) + g as f64 * eta);
  let score = 0.5 * (n as f64 * q * sb / s - g as f64);
  if !a.is_finite() || a <= 0.0 || !v.is_finite() || !ll.is_finite() || !score.is_finite() {
    return Err(failed(
      "ML profile variance/likelihood is nonfinite or degenerate",
    ));
  }
  Ok(Profile {
    fit,
    a,
    v,
    ll,
    score,
    within_residuals: ew,
    between_residuals: eb,
  })
}

/// Interior-only development kernel: equal retained counts T>=2 and full within
/// rank, with G>K. Search eta=log(1+T*sigma_u²/sigma_e²) on [0,24], bracket
/// positive-to-negative profile-score crossings on a 0.5 grid and polish by
/// bisection. Compare every detected local maximum with the zero-component
/// boundary. A boundary optimum or unresolved upper tail is an explicit error.
/// This bounded search is not a proof of global optimality for arbitrary designs.
pub fn fit_random_effects_ml_interior(
  problem: &EstimationProblem,
  entity: &str,
) -> Result<RandomEffectsMlInterior, StatsError> {
  let within = fit_fixed_effects(problem, entity, false)?;
  let g = within.effects.len();
  let t = within.minimum_group_size;
  let p = problem.predictor_names.len();
  let k = p + 1;
  if t < 2 || t != within.maximum_group_size || g <= k {
    return Err(failed(
      "ML kernel requires balanced T>=2, G>K and full within rank",
    ));
  }
  let centers: BTreeMap<_, _> = within.effects.iter().map(|e| (e.key.as_str(), e)).collect();
  let groups = problem
    .sample
    .as_ref()
    .unwrap()
    .cluster_groups
    .as_ref()
    .unwrap();
  let means: Vec<_> = groups
    .iter()
    .map(|key| centers[key.as_str()].predictor_means.clone())
    .collect();
  let ym: Vec<_> = groups
    .iter()
    .map(|key| centers[key.as_str()].response_mean)
    .collect();
  let mut previous = profile(problem, &means, &ym, g, 0.0)?;
  let mut best_eta = 0.0;
  let mut best_ll = previous.ll;
  for step in 1..=48 {
    let right = 0.5 * f64::from(step);
    let current = profile(problem, &means, &ym, g, right)?;
    if previous.score > 0.0 && current.score <= 0.0 {
      let mut lo = right - 0.5;
      let mut hi = right;
      for _ in 0..48 {
        let mid = 0.5 * (lo + hi);
        let candidate = profile(problem, &means, &ym, g, mid)?;
        if candidate.score > 0.0 {
          lo = mid;
        } else {
          hi = mid;
        }
        if hi - lo <= 1e-12 {
          break;
        }
      }
      let eta = 0.5 * (lo + hi);
      let candidate = profile(problem, &means, &ym, g, eta)?;
      if candidate.ll > best_ll {
        best_ll = candidate.ll;
        best_eta = eta;
      }
    }
    if step == 48 && (current.score >= 0.0 || current.ll >= best_ll) {
      return Err(failed("ML upper search boundary is unresolved"));
    }
    previous = current;
  }
  if best_eta == 0.0 {
    return Err(failed(
      "zero-unit-variance ML boundary is not yet installed",
    ));
  }
  let mut r = profile(problem, &means, &ym, g, best_eta)?;
  if r.score.abs() > 1e-9 * problem.response.len() as f64 {
    return Err(failed("ML profile did not reach a stationary point"));
  }
  let n = problem.response.len();
  let anchor = problem.response[0];
  let grand_y = anchor + problem.response.iter().map(|y| y - anchor).sum::<f64>() / n as f64;
  let null_sw = problem
    .response
    .iter()
    .zip(&ym)
    .map(|(y, m)| (y - m).powi(2))
    .sum::<f64>();
  let null_sb = within
    .effects
    .iter()
    .map(|e| t as f64 * (e.response_mean - grand_y).powi(2))
    .sum::<f64>();
  let null_within_variance = null_sw / (n - g) as f64;
  let null_mean_variance = null_sb / g as f64;
  let null_boundary = null_mean_variance <= null_within_variance;
  let (null_a, null_u, null_ll) = if null_boundary {
    let a = (null_sw + null_sb) / n as f64;
    (a, 0.0, -0.5 * n as f64 * ((2.0 * PI).ln() + 1.0 + a.ln()))
  } else {
    let a = null_within_variance;
    let v = null_mean_variance;
    (
      a,
      (v - a) / t as f64,
      -0.5
        * ((n - g) as f64 * ((2.0 * PI).ln() + 1.0 + a.ln())
          + g as f64 * ((2.0 * PI).ln() + 1.0 + v.ln())),
    )
  };
  let lr = 2.0 * (r.ll - null_ll);
  if !null_ll.is_finite()
    || !null_a.is_finite()
    || null_a <= 0.0
    || !null_u.is_finite()
    || !lr.is_finite()
    || lr < -1e-8
  {
    return Err(failed(
      "ML restricted null is degenerate or exceeds the full likelihood",
    ));
  }
  let likelihood_ratio = lr.max(0.0);
  let likelihood_ratio_p_value =
    (!null_boundary).then(|| crate::distributions::chi_square_tail(likelihood_ratio, p));
  let mut information = vec![vec![0.0; k + 2]; k + 2];
  for i in 0..n {
    let mut xw = vec![0.0];
    xw.extend(
      problem.design_matrix[i]
        .iter()
        .zip(&means[i])
        .map(|(x, m)| x - m),
    );
    let mut xb = vec![1.0];
    xb.extend_from_slice(&means[i]);
    for j in 0..k {
      for l in 0..k {
        information[j][l] += xw[j] * xw[l] / r.a + xb[j] * xb[l] / r.v;
      }
      information[j][k] += xw[j] * r.within_residuals[i] / r.a;
      information[j][k + 1] += xb[j] * r.between_residuals[i] / r.v;
    }
    information[k][k] += r.within_residuals[i].powi(2) / (2.0 * r.a);
    information[k + 1][k + 1] += r.between_residuals[i].powi(2) / (2.0 * r.v);
  }
  for j in 0..k {
    information[k][j] = information[j][k];
    information[k + 1][j] = information[j][k + 1];
  }
  let scales: Vec<_> = (0..k + 2).map(|j| information[j][j].sqrt()).collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(failed("ML observed information has invalid diagonal"));
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
  let inverse = crate::nb2::positive_inverse(&normalized)
    .ok_or_else(|| failed("ML joint observed information is not positive definite"))?;
  let covariance: Vec<Vec<_>> = inverse
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
    return Err(failed("ML joint covariance overflowed"));
  }
  r.fit.covariance.matrix = covariance[..k]
    .iter()
    .map(|row| row[..k].to_vec())
    .collect();
  let se = r.fit.covariance.standard_errors();
  r.fit.coefficients = r
    .fit
    .coefficients
    .iter()
    .zip(se)
    .map(|(b, se)| {
      crate::CoefficientEstimate::new(
        &b.name,
        b.value,
        Some(se),
        Some(2.0 * crate::normal_cdf(-(b.value / se).abs())),
      )
    })
    .collect();
  r.fit.fit_statistics.log_likelihood = Some(r.ll);
  r.fit.fit_statistics.r_squared = None;
  r.fit.fit_statistics.adjusted_r_squared = None;
  r.fit.fit_statistics.f_statistic = None;
  r.fit.diagnostics.method = "balanced_gaussian_random_effects_ml_interior_wip".into();
  Ok(RandomEffectsMlInterior {
    regression: r.fit,
    joint_covariance: covariance,
    sigma_e_squared: r.a,
    sigma_u_squared: (r.v - r.a) / t as f64,
    eta: best_eta,
    log_likelihood: r.ll,
    null_log_likelihood: null_ll,
    null_sigma_e_squared: null_a,
    null_sigma_u_squared: null_u,
    likelihood_ratio,
    likelihood_ratio_p_value,
    profile_score: r.score,
    group_count: g,
    group_size: t,
  })
}
