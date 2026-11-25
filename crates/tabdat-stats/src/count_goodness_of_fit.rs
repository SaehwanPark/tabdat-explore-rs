//! Score-adjusted grouped goodness of fit for retained NB2 ML fits.

use crate::count_probabilities::{
  CountProbabilityLaw, count_bin_probabilities, count_probability_table,
};
use crate::distributions::chi_square_tail;
use crate::matrix::{add_compensated, qr_decompose_with_response};
use crate::nb2::{
  CountCache, Nb2Result, log_ratio_remainder, positive_inverse, scores_and_information_for_fit,
  softplus,
};
use crate::{EstimationProblem, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The source workflow's default partition is 0..9 and >=10. Smaller tail
/// cutoffs are admitted so sparse upper cells can be combined deliberately.
pub const MAX_NB2_COUNT_GOF_TAIL: usize = 10;
/// Bounds the leading n*k^2 work of the auxiliary Householder QR.
pub const MAX_NB2_COUNT_GOF_QR_WORK: usize = 100_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountGoodnessOfFitCluster {
  /// One-way cluster identifier used in the retained NB2 fit.
  pub variable: String,
  /// Number of distinct retained groups.
  pub group_count: usize,
  /// Multiplier applied to the uncentered cluster moment covariance.
  pub covariance_correction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountGoodnessOfFitResult {
  pub observation_count: usize,
  pub tail_from: usize,
  pub table: crate::CountProbabilityTable,
  pub statistic: f64,
  pub degrees_of_freedom: usize,
  pub p_value: f64,
  pub score_parameter_names: Vec<String>,
  /// Cell-moment plus NB2 score dimension. The unclustered test fits these
  /// columns as an auxiliary regression; the clustered test uses them to form
  /// the score-adjusted moment covariance.
  pub auxiliary_regressor_count: usize,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster: Option<CountGoodnessOfFitCluster>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub moment_covariance: Option<Vec<Vec<f64>>>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("NB2 count goodness of fit: {message}"))
}

fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("NB2 count goodness of fit: {message}"))
}

fn scaled_column_norm(values: impl Iterator<Item = f64>) -> f64 {
  let values: Vec<f64> = values.collect();
  let scale = values
    .iter()
    .fold(0.0_f64, |current, value| current.max(value.abs()));
  if scale == 0.0 || !scale.is_finite() {
    return scale;
  }
  scale
    * values
      .iter()
      .map(|value| (value / scale).powi(2))
      .sum::<f64>()
      .sqrt()
}

fn validate_fit(problem: &EstimationProblem, result: &Nb2Result) -> Result<(), StatsError> {
  problem.validate()?;
  if result.robust != result.cluster.is_some() {
    return Err(invalid(
      "requires a model-based unclustered NB2 fit or a one-way clustered NB2 fit",
    ));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|sample| sample.weights.is_some())
  {
    return Err(invalid("weighted fits are not supported"));
  }
  if result.offset_variable.is_some() {
    return Err(invalid(
      "fixed-offset fits are not supported in this workflow",
    ));
  }
  let sample_groups = problem
    .sample
    .as_ref()
    .and_then(|sample| sample.cluster_groups.as_deref());
  match (&result.cluster, sample_groups) {
    (None, Some(_)) => return Err(invalid("cluster identifiers require a clustered NB2 fit")),
    (Some(_), None) => {
      return Err(invalid(
        "clustered NB2 fit requires retained cluster identifiers",
      ));
    }
    (Some(cluster), Some(groups)) => {
      let group_count = groups
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
      if groups.len() != problem.observation_count() || group_count != cluster.group_count {
        return Err(invalid(
          "cluster identifiers do not match the retained clustered NB2 fit",
        ));
      }
    }
    (None, None) => {}
  }
  let expected_parameters = problem.predictor_names.len() + usize::from(problem.include_intercept);
  if result.outcome != problem.outcome
    || result.predictor_names != problem.predictor_names
    || result.include_intercept != problem.include_intercept
    || result.observation_count != problem.observation_count()
    || result.sample != problem.sample
    || result.fitted_means.len() != problem.observation_count()
    || result.coefficients.len() != expected_parameters
    || result.dispersion.alpha <= 0.0
    || !result.dispersion.alpha.is_finite()
    || !result.dispersion.log_alpha.is_finite()
    || result.dispersion.log_alpha.abs() >= 12.0
    || problem
      .response
      .iter()
      .any(|y| !y.is_finite() || *y < 0.0 || *y > crate::MAX_NB2_COUNT || y.fract() != 0.0)
    || result
      .fitted_means
      .iter()
      .any(|mean| !mean.is_finite() || *mean <= 0.0)
  {
    return Err(invalid(
      "retained NB2 fit, parameter order or estimation sample does not match the supplied problem",
    ));
  }
  let mut coefficient_names = Vec::with_capacity(expected_parameters);
  if problem.include_intercept {
    coefficient_names.push(problem.intercept_name.as_str());
  }
  coefficient_names.extend(problem.predictor_names.iter().map(String::as_str));
  if result
    .coefficients
    .iter()
    .zip(&coefficient_names)
    .any(|(coefficient, expected)| {
      coefficient.name.as_str() != *expected || !coefficient.value.is_finite()
    })
  {
    return Err(invalid(
      "NB2 coefficients are not finite or in retained design order",
    ));
  }
  let alpha_from_log = result.dispersion.log_alpha.exp();
  if (alpha_from_log.ln() - result.dispersion.alpha.ln()).abs() > 1e-12 {
    return Err(invalid("NB2 alpha and lnalpha do not agree"));
  }
  let (intercept, slope_coefficients) = if problem.include_intercept {
    (result.coefficients[0].value, &result.coefficients[1..])
  } else {
    (0.0, &result.coefficients[..])
  };
  for (row, mean) in problem.design_matrix.iter().zip(&result.fitted_means) {
    let eta = intercept
      + row
        .iter()
        .zip(slope_coefficients)
        .map(|(x, coefficient)| x * coefficient.value)
        .sum::<f64>();
    if !eta.is_finite() || (eta - mean.ln()).abs() > 1e-9 * (1.0 + eta.abs()) {
      return Err(invalid(
        "retained fitted means do not match the NB2 coefficients and design",
      ));
    }
  }
  Ok(())
}

/// Score-adjusted grouped NB2 test on the retained estimation sample.
///
/// Model-based unclustered fits use Andrews' no-intercept `1 ~ (d-p) + score`
/// regression. One-way clustered fits use the independent-cluster extension of
/// its joint moment/score influence function with model-based observed
/// information and a `G/(G-1)` covariance correction. The clustered reference
/// requires independent clusters, correctly specified conditional NB2
/// marginals, many clusters and a full-rank grouped-moment covariance. Both
/// modes evaluate scores and cell probabilities on the exact retained sample.
pub fn nb2_count_goodness_of_fit(
  problem: &EstimationProblem,
  result: &Nb2Result,
  tail_from: usize,
) -> Result<CountGoodnessOfFitResult, StatsError> {
  if !(1..=MAX_NB2_COUNT_GOF_TAIL).contains(&tail_from) {
    return Err(invalid("tail cutoff must be an integer from 1 to 10"));
  }
  validate_fit(problem, result)?;
  if result.cluster.is_some() {
    return clustered_count_goodness_of_fit(problem, result, tail_from);
  }

  let n = problem.observation_count();
  let beta_count = result.coefficients.len();
  let score_count = beta_count + 1;
  let auxiliary_column_count = tail_from + score_count;
  if n <= auxiliary_column_count {
    return Err(StatsError::InsufficientObservations(format!(
      "NB2 count goodness of fit requires more than {auxiliary_column_count} retained rows"
    )));
  }
  let qr_work = auxiliary_column_count
    .checked_mul(auxiliary_column_count)
    .and_then(|columns_squared| n.checked_mul(columns_squared))
    .ok_or_else(|| invalid("auxiliary-regression work estimate overflowed"))?;
  if qr_work > MAX_NB2_COUNT_GOF_QR_WORK {
    return Err(invalid(&format!(
      "auxiliary regression exceeds the {MAX_NB2_COUNT_GOF_QR_WORK} n*k^2 work bound"
    )));
  }

  let alpha = result.dispersion.alpha;
  let r = (-result.dispersion.log_alpha).exp();
  if !r.is_finite() || r <= 0.0 {
    return Err(invalid("NB2 size parameter is not finite and positive"));
  }
  let maximum_count = problem.response.iter().copied().fold(0.0, f64::max) as usize;
  let cache = CountCache::new(maximum_count, r);
  let mut design = Vec::with_capacity(n);
  let mut observed = Vec::with_capacity(n);
  let mut observed_bins = vec![0_usize; tail_from + 1];

  for ((predictors, &y), &mean) in problem
    .design_matrix
    .iter()
    .zip(&problem.response)
    .zip(&result.fitted_means)
  {
    let count = y as usize;
    let cell = count.min(tail_from);
    observed_bins[cell] += 1;
    let probabilities =
      count_bin_probabilities(CountProbabilityLaw::Nb2 { alpha }, mean, tail_from)?;
    let mut row = Vec::with_capacity(auxiliary_column_count);
    for (j, probability) in probabilities[..tail_from].iter().enumerate() {
      row.push((if cell == j { 1.0 } else { 0.0 }) - probability);
    }

    let mut score_design = Vec::with_capacity(beta_count);
    if problem.include_intercept {
      score_design.push(1.0);
    }
    score_design.extend_from_slice(predictors);
    let eta_score = (y - mean) * (r / (r + mean));
    row.extend(score_design.iter().map(|x| x * eta_score));

    let q = mean / (r + mean);
    let ratio = softplus(mean.ln() + result.dispersion.log_alpha);
    let remainder = log_ratio_remainder(q, ratio);
    let alpha_score = cache.score_sum[count] + r * remainder - y * q;
    row.push(alpha_score);
    if row.len() != auxiliary_column_count || row.iter().any(|value| !value.is_finite()) {
      return Err(failed(
        "auxiliary design contains a nonfinite score or cell moment",
      ));
    }
    design.push(row);
    observed.push(1.0);
  }

  // Scaling columns does not change the no-intercept regression's span and
  // prevents the score columns from numerically overwhelming cell residuals.
  let mut column_norms = Vec::with_capacity(auxiliary_column_count);
  for column in 0..auxiliary_column_count {
    let norm = scaled_column_norm(design.iter().map(|row| row[column]));
    if !norm.is_finite() || norm <= 1e-12 {
      return Err(failed("auxiliary cell/score design is rank deficient"));
    }
    column_norms.push(norm);
  }
  for row in &mut design {
    for (value, norm) in row.iter_mut().zip(&column_norms) {
      *value /= norm;
    }
  }
  let decomposition = qr_decompose_with_response(&design, &observed)
    .map_err(|error| failed(&format!("auxiliary regression failed: {error}")))?;
  let (mut statistic, mut correction) = (0.0, 0.0);
  for value in decomposition.qty.into_iter().take(auxiliary_column_count) {
    add_compensated(&mut statistic, &mut correction, value * value);
  }
  if !statistic.is_finite() || statistic < 0.0 || statistic > n as f64 * (1.0 + 1e-10) {
    return Err(failed(
      "auxiliary regression produced an invalid uncentered R-squared",
    ));
  }
  statistic = statistic.min(n as f64);
  let degrees_of_freedom = tail_from;
  let p_value = chi_square_tail(statistic, degrees_of_freedom);
  if !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
    return Err(failed("chi-square tail probability is not representable"));
  }

  let table = count_probability_table(
    CountProbabilityLaw::Nb2 { alpha },
    &problem.response,
    &result.fitted_means,
    tail_from,
  )?;
  if table.bins.len() != observed_bins.len()
    || table
      .bins
      .iter()
      .zip(&observed_bins)
      .any(|(bin, observed)| bin.observed_count != *observed)
  {
    return Err(failed(
      "observed cell frequencies do not match retained count moments",
    ));
  }

  let mut score_parameter_names: Vec<String> = result
    .coefficients
    .iter()
    .map(|coefficient| coefficient.name.clone())
    .collect();
  score_parameter_names.push("lnalpha".to_owned());
  Ok(CountGoodnessOfFitResult {
    observation_count: n,
    tail_from,
    table,
    statistic,
    degrees_of_freedom,
    p_value,
    score_parameter_names,
    auxiliary_regressor_count: auxiliary_column_count,
    cluster: None,
    moment_covariance: None,
  })
}

fn clustered_count_goodness_of_fit(
  problem: &EstimationProblem,
  result: &Nb2Result,
  tail_from: usize,
) -> Result<CountGoodnessOfFitResult, StatsError> {
  let n = problem.observation_count();
  let groups = problem
    .sample
    .as_ref()
    .and_then(|sample| sample.cluster_groups.as_deref())
    .ok_or_else(|| invalid("cluster identifiers are unavailable"))?;
  let cluster = result
    .cluster
    .as_ref()
    .ok_or_else(|| invalid("clustered NB2 fit metadata is unavailable"))?;
  let group_count = cluster.group_count;
  if group_count < 2 {
    return Err(invalid("cluster covariance requires at least two groups"));
  }

  let beta_count = result.coefficients.len();
  let parameter_count = beta_count + 1;
  let moment_count = tail_from;
  let auxiliary_regressor_count = moment_count + parameter_count;
  let mut moments = Vec::with_capacity(n);
  let mut observed_bins = vec![0_usize; moment_count + 1];
  let alpha = result.dispersion.alpha;
  let r = (-result.dispersion.log_alpha).exp();
  if !r.is_finite() || r <= 0.0 {
    return Err(invalid("NB2 size parameter is not finite and positive"));
  }
  let cache = CountCache::new(tail_from.saturating_sub(1), r);
  let mut c_sum = vec![vec![0.0; parameter_count]; moment_count];
  let mut c_correction = vec![vec![0.0; parameter_count]; moment_count];

  for ((predictors, &y), &mean) in problem
    .design_matrix
    .iter()
    .zip(&problem.response)
    .zip(&result.fitted_means)
  {
    let count = y as usize;
    observed_bins[count.min(tail_from)] += 1;
    let probabilities =
      count_bin_probabilities(CountProbabilityLaw::Nb2 { alpha }, mean, tail_from)?;
    let mut row_design = Vec::with_capacity(beta_count);
    if problem.include_intercept {
      row_design.push(1.0);
    }
    row_design.extend_from_slice(predictors);

    let mut row_moments = Vec::with_capacity(moment_count);
    let eta = mean.ln();
    let log_alpha = result.dispersion.log_alpha;
    let q = mean / (r + mean);
    let ratio = softplus(eta + log_alpha);
    let remainder = log_ratio_remainder(q, ratio);
    for cell in 0..moment_count {
      let probability = probabilities[cell];
      row_moments.push((if count == cell { 1.0 } else { 0.0 }) - probability);
      let cell_count = cell as f64;
      let eta_score = (cell_count - mean) * (r / (r + mean));
      let log_alpha_score = cache.score_sum[cell] + r * remainder - cell_count * q;
      let dp_deta = probability * eta_score;
      let dp_dlog_alpha = probability * log_alpha_score;
      for parameter in 0..beta_count {
        add_compensated(
          &mut c_sum[cell][parameter],
          &mut c_correction[cell][parameter],
          -dp_deta * row_design[parameter],
        );
      }
      add_compensated(
        &mut c_sum[cell][beta_count],
        &mut c_correction[cell][beta_count],
        -dp_dlog_alpha,
      );
    }
    moments.push(row_moments);
  }

  let score_information = scores_and_information_for_fit(problem, result)?;
  let scores = score_information.scores;
  let information = score_information.information;
  if scores.len() != n
    || scores.iter().any(|score| score.len() != parameter_count)
    || c_sum
      .iter()
      .flatten()
      .chain(information.iter().flatten())
      .any(|value| !value.is_finite())
  {
    return Err(failed("cluster score or derivative dimensions are invalid"));
  }
  let bread = positive_inverse(&information)
    .ok_or_else(|| failed("full NB2 observed information is not positive definite"))?;
  let mut projection = vec![vec![0.0; parameter_count]; moment_count];
  for (cell, row) in projection.iter_mut().enumerate() {
    for (parameter, value) in row.iter_mut().enumerate() {
      let mut correction = 0.0;
      for other in 0..parameter_count {
        add_compensated(
          value,
          &mut correction,
          c_sum[cell][other] * bread[other][parameter],
        );
      }
    }
  }

  let mut cluster_sums = BTreeMap::<&str, (Vec<f64>, Vec<f64>)>::new();
  for ((group, row_moments), score) in groups.iter().zip(&moments).zip(&scores) {
    let entry = cluster_sums
      .entry(group.as_str())
      .or_insert_with(|| (vec![0.0; moment_count], vec![0.0; moment_count]));
    for cell in 0..moment_count {
      let mut adjustment = 0.0;
      let mut adjustment_correction = 0.0;
      for (coefficient, score) in projection[cell].iter().zip(score) {
        add_compensated(
          &mut adjustment,
          &mut adjustment_correction,
          coefficient * score,
        );
      }
      add_compensated(
        &mut entry.0[cell],
        &mut entry.1[cell],
        row_moments[cell] + adjustment,
      );
    }
  }
  if cluster_sums.len() != group_count {
    return Err(invalid("cluster group count changed after NB2 estimation"));
  }

  let correction = group_count as f64 / (group_count - 1) as f64;
  let mut meat = vec![vec![0.0; moment_count]; moment_count];
  let mut meat_correction = vec![vec![0.0; moment_count]; moment_count];
  for (sum, _) in cluster_sums.into_values() {
    for row in 0..moment_count {
      for column in 0..moment_count {
        add_compensated(
          &mut meat[row][column],
          &mut meat_correction[row][column],
          sum[row] * sum[column],
        );
      }
    }
  }
  let moment_covariance: Vec<Vec<f64>> = meat
    .iter()
    .map(|row| {
      row
        .iter()
        .map(|value| correction * value / n as f64)
        .collect()
    })
    .collect();
  if moment_covariance
    .iter()
    .flatten()
    .any(|value| !value.is_finite())
  {
    return Err(failed("cluster-adjusted moment covariance is non-finite"));
  }

  // Normalize before the Cholesky rank check so rare cells are not rejected
  // merely because their variances are small in the original probability scale.
  let scales: Vec<f64> = (0..moment_count)
    .map(|cell| moment_covariance[cell][cell].sqrt())
    .collect();
  if scales
    .iter()
    .any(|scale| !scale.is_finite() || *scale <= 0.0)
  {
    return Err(failed(
      "cluster-adjusted moment covariance has zero variance",
    ));
  }
  let correlation: Vec<Vec<f64>> = moment_covariance
    .iter()
    .enumerate()
    .map(|(row, values)| {
      values
        .iter()
        .enumerate()
        .map(|(column, value)| value / scales[row] / scales[column])
        .collect()
    })
    .collect();
  let correlation_inverse = positive_inverse(&correlation)
    .ok_or_else(|| failed("cluster-adjusted grouped moments have a rank-deficient covariance"))?;
  let mut moment_sum = vec![0.0; moment_count];
  let mut moment_sum_correction = vec![0.0; moment_count];
  for row in &moments {
    for cell in 0..moment_count {
      add_compensated(
        &mut moment_sum[cell],
        &mut moment_sum_correction[cell],
        row[cell],
      );
    }
  }
  let mean_moments: Vec<f64> = (0..moment_count)
    .map(|cell| moment_sum[cell] / n as f64 / scales[cell])
    .collect();
  let mut quadratic = 0.0;
  let mut quadratic_correction = 0.0;
  for row in 0..moment_count {
    for column in 0..moment_count {
      add_compensated(
        &mut quadratic,
        &mut quadratic_correction,
        mean_moments[row] * correlation_inverse[row][column] * mean_moments[column],
      );
    }
  }
  let statistic = n as f64 * quadratic;
  if !statistic.is_finite() || statistic < 0.0 {
    return Err(failed(
      "cluster-adjusted quadratic form is non-finite or negative",
    ));
  }
  let degrees_of_freedom = moment_count;
  let p_value = chi_square_tail(statistic, degrees_of_freedom);
  if !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
    return Err(failed(
      "cluster-adjusted chi-square tail is not representable",
    ));
  }

  let table = count_probability_table(
    CountProbabilityLaw::Nb2 { alpha },
    &problem.response,
    &result.fitted_means,
    tail_from,
  )?;
  if table.bins.len() != observed_bins.len()
    || table
      .bins
      .iter()
      .zip(&observed_bins)
      .any(|(bin, observed)| bin.observed_count != *observed)
  {
    return Err(failed(
      "observed cell frequencies do not match retained count moments",
    ));
  }
  let mut score_parameter_names: Vec<String> = result
    .coefficients
    .iter()
    .map(|coefficient| coefficient.name.clone())
    .collect();
  score_parameter_names.push("lnalpha".to_owned());
  Ok(CountGoodnessOfFitResult {
    observation_count: n,
    tail_from,
    table,
    statistic,
    degrees_of_freedom,
    p_value,
    score_parameter_names,
    auxiliary_regressor_count,
    cluster: Some(CountGoodnessOfFitCluster {
      variable: cluster.variable.clone(),
      group_count,
      covariance_correction: correction,
    }),
    moment_covariance: Some(moment_covariance),
  })
}
