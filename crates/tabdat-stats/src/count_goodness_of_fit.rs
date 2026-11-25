//! Score-adjusted grouped goodness of fit for an unclustered NB2 ML model.

use crate::count_probabilities::{
  CountProbabilityLaw, count_bin_probabilities, count_probability_table,
};
use crate::distributions::chi_square_tail;
use crate::matrix::{add_compensated, qr_decompose_with_response};
use crate::nb2::{CountCache, Nb2Result, log_ratio_remainder, softplus};
use crate::{EstimationProblem, StatsError};
use serde::{Deserialize, Serialize};

/// The source workflow's default partition is 0..9 and >=10. Smaller tail
/// cutoffs are admitted so sparse upper cells can be combined deliberately.
pub const MAX_NB2_COUNT_GOF_TAIL: usize = 10;
/// Bounds the leading n*k^2 work of the auxiliary Householder QR.
pub const MAX_NB2_COUNT_GOF_QR_WORK: usize = 100_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountGoodnessOfFitResult {
  pub observation_count: usize,
  pub tail_from: usize,
  pub table: crate::CountProbabilityTable,
  pub statistic: f64,
  pub degrees_of_freedom: usize,
  pub p_value: f64,
  pub score_parameter_names: Vec<String>,
  pub auxiliary_regressor_count: usize,
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
  if result.robust || result.cluster.is_some() {
    return Err(invalid(
      "requires an unclustered model-based NB2 fit; no robust or cluster reference is installed",
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
  if problem
    .sample
    .as_ref()
    .is_some_and(|sample| sample.cluster_groups.is_some())
  {
    return Err(invalid("clustered estimation samples are not supported"));
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

/// Andrews' grouped NB2 test, using the no-intercept `1 ~ (d-p) + score`
/// auxiliary regression and its asymptotic chi-square(J-1) reference.
///
/// The likelihood scores and cell probabilities are evaluated on exactly the
/// retained estimation sample. The implemented reference assumes independent
/// rows and a correctly specified conditional NB2 distribution.
pub fn nb2_count_goodness_of_fit(
  problem: &EstimationProblem,
  result: &Nb2Result,
  tail_from: usize,
) -> Result<CountGoodnessOfFitResult, StatsError> {
  if !(1..=MAX_NB2_COUNT_GOF_TAIL).contains(&tail_from) {
    return Err(invalid("tail cutoff must be an integer from 1 to 10"));
  }
  validate_fit(problem, result)?;

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
  })
}
