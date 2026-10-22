#![forbid(unsafe_code)]

use crate::{
  CoefficientEstimate, CovarianceType, EstimationProblem, LeastSquaresOptions, LeastSquaresResult,
  StatsError, f_distribution_survival_probability, fit_least_squares_with_options,
  student_t_pvalue,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Retained estimation-sample unit center; predictions must not recompute it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedEffect {
  pub key: String,
  pub observation_count: usize,
  pub response_mean: f64,
  pub predictor_means: Vec<f64>,
  /// Total unit intercept in y = alpha_g + X beta.
  pub intercept: f64,
  /// Unit deviation from the observation-weighted normalized common intercept.
  pub effect: f64,
}

/// Classical nested-model test that all individual intercepts are equal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitEffectsFTest {
  pub f_statistic: f64,
  pub df_num: usize,
  pub df_denom: usize,
  pub p_value: f64,
}

/// One-way within fit. Only slope coefficients have reported covariance/inference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedEffectsResult {
  pub entity_variable: String,
  pub outcome: String,
  /// Within-projected slope fit: fitted_values are demeaned X beta, not levels.
  /// parameter_count counts slopes; residual DF additionally absorbs G intercepts.
  pub within: LeastSquaresResult,
  pub effects: Vec<FixedEffect>,
  pub fitted_values: Vec<f64>,
  /// Descriptive normalization, not an independently inferred coefficient.
  pub common_intercept: f64,
  /// R² of conditional level fits including unit intercepts, not Stata overall R².
  pub conditional_r_squared: Option<f64>,
  pub minimum_group_size: usize,
  pub maximum_group_size: usize,
  pub singleton_groups: usize,
  /// Homoskedastic pooled-versus-within F test; withheld for clustered fits.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub unit_effects_f_test: Option<UnitEffectsFTest>,
  /// Why the conventional pooled-versus-within F test was not reported.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub unit_effects_f_test_unavailable_reason: Option<String>,
}

fn average(values: impl Iterator<Item = f64> + Clone) -> Result<f64, StatsError> {
  let anchor = values.clone().next().ok_or(StatsError::EmptySample)?;
  let count = values.clone().count();
  let mean = anchor + values.map(|value| value - anchor).sum::<f64>() / count as f64;
  if !mean.is_finite() {
    return Err(StatsError::NonFiniteValue(
      "unit centering overflowed".into(),
    ));
  }
  Ok(mean)
}

fn unit_effects_f_test(
  problem: &EstimationProblem,
  within: &LeastSquaresResult,
  group_count: usize,
) -> (Option<UnitEffectsFTest>, Option<String>) {
  if matches!(
    &within.covariance.covariance_type,
    CovarianceType::Cluster(_)
  ) {
    return (
      None,
      Some(
        "the classical homoskedastic reference is withheld for unit-clustered covariance".into(),
      ),
    );
  }
  let unavailable = |reason: &str| (None, Some(reason.to_owned()));
  let Some(df_num) = group_count.checked_sub(1).filter(|&df| df > 0) else {
    return unavailable("the retained panel has no testable unit-effect restrictions");
  };
  let df_denom = within.fit_statistics.degrees_of_freedom;
  if df_denom == 0 {
    return unavailable("the within fit has no residual degrees of freedom");
  }
  let Ok(pooled) = fit_least_squares_with_options(
    problem,
    &LeastSquaresOptions {
      covariance_type: CovarianceType::NonRobust,
    },
  ) else {
    return unavailable("the same-sample pooled comparison could not be estimated");
  };
  let (Some(rss_pooled), Some(rss_within)) = (
    pooled.fit_statistics.residual_sum_of_squares,
    within.fit_statistics.residual_sum_of_squares,
  ) else {
    return unavailable("the pooled or within residual sum of squares is unavailable");
  };
  if !rss_pooled.is_finite() || !rss_within.is_finite() || rss_within <= 0.0 {
    return unavailable("the nested-model residual variance is degenerate or nonfinite");
  }
  let reduction = rss_pooled - rss_within;
  if !reduction.is_finite() || reduction < 0.0 {
    return unavailable("the pooled-versus-within residual comparison is numerically invalid");
  }
  let f_statistic = (reduction / df_num as f64) / (rss_within / df_denom as f64);
  let p_value = f_distribution_survival_probability(f_statistic, df_num as f64, df_denom as f64);
  if !f_statistic.is_finite() || !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
    return unavailable("the nested-model F reference is nonfinite");
  }
  (
    Some(UnitEffectsFTest {
      f_statistic,
      df_num,
      df_denom,
      p_value,
    }),
    None,
  )
}

/// Unweighted one-way FE with unit groups in problem.sample.cluster_groups. The input
/// intercept flag must be true: the mean model absorbs one intercept per unit.
/// Classical DF is N-G-p; clustered covariance uses the plm `sss` correction
/// G/(G-1)*(N-1)/(N-p), not N-G-p, because the effects are nested in the clusters.
pub fn fit_fixed_effects(
  problem: &EstimationProblem,
  entity_variable: &str,
  clustered: bool,
) -> Result<FixedEffectsResult, StatsError> {
  problem.validate()?;
  if !problem.include_intercept || problem.weights.is_some() || problem.predictor_names.is_empty() {
    return Err(StatsError::InvalidEstimation(
      "within FE requires unit intercepts, at least one predictor and no weights".into(),
    ));
  }
  if entity_variable.is_empty() {
    return Err(StatsError::InvalidParameterName(
      "entity variable must be named".into(),
    ));
  }
  let n = problem.response.len();
  let p = problem.predictor_names.len();
  let keys = problem
    .sample
    .as_ref()
    .and_then(|sample| sample.cluster_groups.as_ref())
    .ok_or_else(|| {
      StatsError::InvalidEstimation("within FE requires sample-aligned unit identifiers".into())
    })?;
  if keys.len() != n {
    return Err(StatsError::DimensionMismatch(
      "unit identifiers must align with the estimation sample".into(),
    ));
  }
  let sample = problem.sample.as_ref().unwrap();
  if n.checked_add(sample.dropped_observations) != Some(sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .any(|&index| index >= sample.total_observations)
  {
    return Err(StatsError::DimensionMismatch(
      "invalid candidate-row sample mapping".into(),
    ));
  }
  let mut lookup = BTreeMap::new();
  let mut members: Vec<Vec<usize>> = Vec::new();
  let mut unit_keys = Vec::new();
  let mut group_index = Vec::with_capacity(n);
  for (row, key) in keys.iter().enumerate() {
    let index = *lookup.entry(key).or_insert_with(|| {
      let index = members.len();
      members.push(Vec::new());
      unit_keys.push(key.clone());
      index
    });
    members[index].push(row);
    group_index.push(index);
  }
  let g = members.len();
  if n <= g + p {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters: g + p,
    });
  }
  let df = n - g - p;
  let mut effects = Vec::with_capacity(g);
  for (key, indices) in unit_keys.into_iter().zip(&members) {
    let response_mean = average(indices.iter().map(|&i| problem.response[i]))?;
    let predictor_means = (0..p)
      .map(|j| average(indices.iter().map(|&i| problem.design_matrix[i][j])))
      .collect::<Result<Vec<_>, _>>()?;
    effects.push(FixedEffect {
      key,
      observation_count: indices.len(),
      response_mean,
      predictor_means,
      intercept: 0.0,
      effect: 0.0,
    });
  }
  let mut projected = problem.clone();
  projected.include_intercept = false;
  for (row, &index) in group_index.iter().enumerate() {
    projected.response[row] = problem.response[row] - effects[index].response_mean;
    for j in 0..p {
      projected.design_matrix[row][j] =
        problem.design_matrix[row][j] - effects[index].predictor_means[j];
    }
  }
  if problem
    .sample
    .as_ref()
    .is_some_and(|sample| sample.weights.is_some())
  {
    return Err(StatsError::InvalidEstimation(
      "within FE does not support sample weights".into(),
    ));
  }
  let covariance_type = if clustered {
    CovarianceType::Cluster(entity_variable.into())
  } else {
    CovarianceType::NonRobust
  };
  let mut within =
    fit_least_squares_with_options(&projected, &LeastSquaresOptions { covariance_type })?;
  if !clustered {
    let correction = (n - p) as f64 / df as f64;
    for row in &mut within.covariance.matrix {
      for entry in row {
        *entry *= correction;
      }
    }
    let se = within.covariance.standard_errors();
    within.coefficients = within
      .coefficients
      .iter()
      .zip(se)
      .map(|(coefficient, se)| {
        let p_value = (se > 0.0).then(|| student_t_pvalue(coefficient.value / se, df as f64));
        CoefficientEstimate::new(&coefficient.name, coefficient.value, Some(se), p_value)
      })
      .collect();
  }
  let rss = within.fit_statistics.residual_sum_of_squares.unwrap();
  let within_tss = within.fit_statistics.total_sum_of_squares.unwrap();
  if !rss.is_finite()
    || !within_tss.is_finite()
    || within
      .covariance
      .matrix
      .iter()
      .flatten()
      .any(|value| !value.is_finite())
  {
    return Err(StatsError::NonFiniteValue(
      "within covariance or sums of squares overflowed".into(),
    ));
  }
  within.fit_statistics.degrees_of_freedom = df;
  within.fit_statistics.root_mse = Some((rss / df as f64).sqrt());
  // Different packages adjust within R² against different TSS degrees of freedom.
  // Leave that optional measure unavailable rather than silently select one.
  within.fit_statistics.adjusted_r_squared = None;
  within.fit_statistics.f_statistic =
    (!clustered && rss > 0.0).then(|| ((within_tss - rss) / p as f64) / (rss / df as f64));
  within.fit_statistics.log_likelihood = None;
  within.diagnostics.method = "within_fixed_effects".into();
  let beta: Vec<_> = within
    .coefficients
    .iter()
    .map(|coefficient| coefficient.value)
    .collect();
  let grand_y = average(problem.response.iter().copied())?;
  let grand_x = (0..p)
    .map(|j| average(problem.design_matrix.iter().map(|row| row[j])))
    .collect::<Result<Vec<_>, _>>()?;
  let common_intercept = grand_y - grand_x.iter().zip(&beta).map(|(x, b)| x * b).sum::<f64>();
  for effect in &mut effects {
    effect.intercept = effect.response_mean
      - effect
        .predictor_means
        .iter()
        .zip(&beta)
        .map(|(x, b)| x * b)
        .sum::<f64>();
    effect.effect = effect.intercept - common_intercept;
    if !effect.intercept.is_finite() || !effect.effect.is_finite() || !common_intercept.is_finite()
    {
      return Err(StatsError::NonFiniteValue(
        "unit intercept recovery overflowed".into(),
      ));
    }
  }
  let fitted_values = within
    .fitted_values
    .iter()
    .zip(&group_index)
    .map(|(value, &index)| value + effects[index].response_mean)
    .collect::<Vec<_>>();
  if fitted_values.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "conditional fitted values overflowed".into(),
    ));
  }
  let tss = problem
    .response
    .iter()
    .map(|value| (value - grand_y).powi(2))
    .sum::<f64>();
  if !tss.is_finite() {
    return Err(StatsError::NonFiniteValue(
      "level total sum of squares overflowed".into(),
    ));
  }
  let conditional_r_squared = (tss > 0.0).then(|| (1.0 - rss / tss).clamp(0.0, 1.0));
  let minimum_group_size = members.iter().map(Vec::len).min().unwrap();
  let maximum_group_size = members.iter().map(Vec::len).max().unwrap();
  let singleton_groups = members.iter().filter(|rows| rows.len() == 1).count();
  let (unit_effects_f_test, unit_effects_f_test_unavailable_reason) =
    unit_effects_f_test(problem, &within, g);
  Ok(FixedEffectsResult {
    entity_variable: entity_variable.into(),
    outcome: problem.outcome.clone(),
    within,
    effects,
    fitted_values,
    common_intercept,
    conditional_r_squared,
    minimum_group_size,
    maximum_group_size,
    singleton_groups,
    unit_effects_f_test,
    unit_effects_f_test_unavailable_reason,
  })
}
