#![forbid(unsafe_code)]

use crate::{
  CoefficientEstimate, CovarianceType, EstimationProblem, LeastSquaresOptions, LeastSquaresResult,
  StatsError, fit_fixed_effects, fit_least_squares, fit_least_squares_with_options, normal_cdf,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One-way RE fit (balanced Swamy–Arora GLS or Gaussian marginal ML).
/// The stored regression is on the
/// quasi-demeaned response/design, including the transformed intercept column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomEffectsResult {
  pub entity_variable: String,
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub gls: LeastSquaresResult,
  pub group_count: usize,
  /// Common retained count, or zero for unbalanced ML; use group_size_range().
  pub group_size: usize,
  pub sigma_e_squared: f64,
  pub sigma_u_squared: f64,
  pub raw_sigma_u_squared: f64,
  /// Common theta, or theta at the minimum retained group size for unbalanced ML.
  pub theta: f64,
  pub rho: f64,
  pub confidence_intervals: Vec<[f64; 2]>,
  pub wald_statistic: Option<f64>,
  pub wald_p_value: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub wald_unavailable_reason: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ml: Option<crate::RandomEffectsMlInference>,
}

impl RandomEffectsResult {
  /// Retained-model counts, independent of any subsequent active-data changes.
  pub fn group_size_range(&self) -> Option<(usize, usize)> {
    if self.group_size > 0 {
      return Some((self.group_size, self.group_size));
    }
    let mut counts = BTreeMap::new();
    for key in self.gls.sample.as_ref()?.cluster_groups.as_ref()? {
      *counts.entry(key.as_str()).or_insert(0_usize) += 1;
    }
    Some((*counts.values().min()?, *counts.values().max()?))
  }

  pub fn theta_range(&self) -> Option<(f64, f64)> {
    if self.group_size > 0 {
      return Some((self.theta, self.theta));
    }
    let (lo, hi) = self.group_size_range()?;
    let theta = |size: usize| {
      1.0
        - (self.sigma_e_squared / (self.sigma_e_squared + size as f64 * self.sigma_u_squared))
          .sqrt()
    };
    Some((theta(lo), theta(hi)))
  }
}

/// Joint finite cases must already be selected and sample-aligned unit keys
/// supplied. Equal retained group sizes T>=2 are required. The within and between
/// preliminary regressions must both identify all slopes. No weights are installed.
/// Coefficients use asymptotic normal inference.
pub fn fit_random_effects(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(problem, entity_variable, false)
}

/// Unit-score sandwich of the quasi-demeaned design/residuals, with correction
/// G/(G-1)*(N-1)/(N-K). Components and coefficients are unchanged. Inference is
/// asymptotic normal/chi-square, not G-1 t/F. Singular slope Wald tests are
/// explicitly unavailable while positive marginal coefficient SEs are retained.
pub fn fit_random_effects_clustered(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(problem, entity_variable, true)
}

fn fit_random_effects_impl(
  problem: &EstimationProblem,
  entity_variable: &str,
  clustered: bool,
) -> Result<RandomEffectsResult, StatsError> {
  let within = fit_fixed_effects(problem, entity_variable, false)?;
  let g = within.effects.len();
  let p = problem.predictor_names.len();
  let t = within.minimum_group_size;
  if t < 2 || within.maximum_group_size != t {
    return Err(StatsError::InvalidEstimation(
      "RE-GLS requires equal retained unit counts T >= 2; unbalanced variance estimation is not implemented".into(),
    ));
  }
  if g <= p + 1 {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: g,
      parameters: p + 1,
    });
  }
  if problem
    .predictor_names
    .iter()
    .any(|name| name == &problem.intercept_name)
  {
    return Err(StatsError::InvalidParameterName(
      "RE predictor conflicts with intercept name".into(),
    ));
  }
  let between_problem = EstimationProblem::new(
    &problem.outcome,
    problem.predictor_names.clone(),
    within.effects.iter().map(|e| e.response_mean).collect(),
    within
      .effects
      .iter()
      .map(|e| e.predictor_means.clone())
      .collect(),
  )?
  .with_intercept_name(&problem.intercept_name);
  let between = fit_least_squares(&between_problem)?;
  let sigma_e_squared = within
    .within
    .fit_statistics
    .residual_sum_of_squares
    .unwrap()
    / within.within.fit_statistics.degrees_of_freedom as f64;
  let raw_sigma_u_squared = between.fit_statistics.residual_sum_of_squares.unwrap()
    / (g - p - 1) as f64
    - sigma_e_squared / t as f64;
  let sigma_u_squared = raw_sigma_u_squared.max(0.0);
  let variance = sigma_e_squared + t as f64 * sigma_u_squared;
  if !sigma_e_squared.is_finite()
    || sigma_e_squared <= 0.0
    || !raw_sigma_u_squared.is_finite()
    || !variance.is_finite()
  {
    return Err(StatsError::InvalidEstimation(
      "RE variance components require finite positive idiosyncratic variance".into(),
    ));
  }
  let theta = 1.0 - (sigma_e_squared / variance).sqrt();
  if theta >= 1.0 {
    return Err(StatsError::InvalidEstimation(
      "RE transformation loses the intercept at machine precision".into(),
    ));
  }
  let centers: BTreeMap<_, _> = within.effects.iter().map(|e| (e.key.as_str(), e)).collect();
  let keys = problem
    .sample
    .as_ref()
    .unwrap()
    .cluster_groups
    .as_ref()
    .unwrap();
  let mut transformed = problem.clone();
  transformed.include_intercept = false;
  transformed
    .predictor_names
    .insert(0, problem.intercept_name.clone());
  for (i, key) in keys.iter().enumerate() {
    let center = centers[key.as_str()];
    transformed.response[i] -= theta * center.response_mean;
    for j in 0..p {
      transformed.design_matrix[i][j] -= theta * center.predictor_means[j];
    }
    transformed.design_matrix[i].insert(0, 1.0 - theta);
  }
  let covariance_type = if clustered {
    CovarianceType::Cluster(entity_variable.into())
  } else {
    CovarianceType::NonRobust
  };
  let mut gls =
    fit_least_squares_with_options(&transformed, &LeastSquaresOptions { covariance_type })?;
  if !gls
    .fit_statistics
    .residual_sum_of_squares
    .is_some_and(|v| v.is_finite() && v > 0.0)
    || gls
      .covariance
      .matrix
      .iter()
      .flatten()
      .any(|v| !v.is_finite())
  {
    return Err(StatsError::InvalidEstimation(
      "RE transformed covariance or residual scale is degenerate".into(),
    ));
  }
  let se = gls.covariance.standard_errors();
  if se.iter().any(|se| !se.is_finite() || *se <= 0.0) {
    return Err(StatsError::InvalidEstimation(
      "RE covariance requires finite positive coefficient variances".into(),
    ));
  }
  gls.coefficients = gls
    .coefficients
    .iter()
    .zip(se)
    .map(|(b, se)| {
      let p_value = (se > 0.0).then(|| 2.0 * normal_cdf(-(b.value / se).abs()));
      CoefficientEstimate::new(&b.name, b.value, Some(se), p_value)
    })
    .collect();
  let critical = crate::normal_quantile(0.975);
  let confidence_intervals: Vec<[f64; 2]> = gls
    .coefficients
    .iter()
    .map(|b| {
      let se = b.standard_error.unwrap();
      [b.value - critical * se, b.value + critical * se]
    })
    .collect();
  if confidence_intervals
    .iter()
    .flatten()
    .any(|v| !v.is_finite())
  {
    return Err(StatsError::NonFiniteValue(
      "RE coefficient intervals overflowed".into(),
    ));
  }
  let values: Vec<_> = gls.coefficients.iter().map(|c| c.value).collect();
  let wald = crate::binary::omnibus_wald(
    &values,
    &gls.covariance.matrix,
    true,
    clustered.then_some(g),
  );
  if !clustered && wald.unavailable_reason.is_some() {
    return Err(StatsError::InvalidEstimation(
      "RE slope Wald calculation is singular or nonfinite".into(),
    ));
  }
  // Uncentered no-intercept OLS summaries would describe the transformed
  // implementation, not standard panel within/between/overall R² or likelihood.
  gls.fit_statistics.r_squared = None;
  gls.fit_statistics.adjusted_r_squared = None;
  gls.fit_statistics.log_likelihood = None;
  gls.fit_statistics.f_statistic = None;
  gls.diagnostics.method = "balanced_swamy_arora_random_effects_gls".into();
  Ok(RandomEffectsResult {
    ml: None,
    entity_variable: entity_variable.into(),
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    gls,
    group_count: g,
    group_size: t,
    sigma_e_squared,
    sigma_u_squared,
    raw_sigma_u_squared,
    theta,
    rho: sigma_u_squared / (sigma_e_squared + sigma_u_squared),
    confidence_intervals,
    wald_statistic: wald.statistic,
    wald_p_value: wald.p_value,
    wald_unavailable_reason: wald.unavailable_reason,
  })
}
