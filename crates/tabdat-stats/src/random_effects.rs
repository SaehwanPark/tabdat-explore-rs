#![forbid(unsafe_code)]

use crate::{
  CoefficientEstimate, EstimationProblem, LeastSquaresResult, PostEstimationModel, StatsError,
  fit_fixed_effects, fit_least_squares, normal_cdf,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Balanced one-way Swamy–Arora feasible GLS. The stored regression is on the
/// quasi-demeaned response/design, including the transformed intercept column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomEffectsResult {
  pub entity_variable: String,
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub gls: LeastSquaresResult,
  pub group_count: usize,
  pub group_size: usize,
  pub sigma_e_squared: f64,
  pub sigma_u_squared: f64,
  pub raw_sigma_u_squared: f64,
  pub theta: f64,
  pub rho: f64,
  pub confidence_intervals: Vec<[f64; 2]>,
  pub wald_statistic: f64,
  pub wald_p_value: f64,
}

/// Joint finite cases must already be selected and sample-aligned unit keys
/// supplied. Equal retained group sizes T>=2 are required. The within and between
/// preliminary regressions must both identify all slopes. No weights or robust
/// covariance are installed. Coefficients use asymptotic normal inference.
pub fn fit_random_effects(
  problem: &EstimationProblem,
  entity_variable: &str,
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
  let mut gls = fit_least_squares(&transformed)?;
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
  let restrictions: Vec<_> = (1..=p)
    .map(|j| (0..=p).map(|i| f64::from(i == j)).collect())
    .collect();
  let wald = PostEstimationModel::from_least_squares(&gls)
    .test_linear_hypothesis(&restrictions, &vec![0.0; p])?;
  let wald_statistic = wald.chi2_statistic;
  let wald_p_value = crate::distributions::chi_square_tail(wald_statistic, p);
  if !wald_statistic.is_finite() || !wald_p_value.is_finite() {
    return Err(StatsError::InvalidEstimation(
      "RE slope Wald calculation is nonfinite".into(),
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
    wald_statistic,
    wald_p_value,
  })
}
