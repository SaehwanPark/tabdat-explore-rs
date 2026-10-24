//! IID Gaussian sample selection by the conventional Heckman two-step method.
//! This is not joint ML. Only the selection and augmented-outcome covariance
//! blocks are provided; sigma/rho are moment points without invented SEs.

#![allow(clippy::needless_range_loop)]

use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf};
use crate::matrix::{
  choice_information_inverse, multiply, qr_decompose_with_response, sandwich,
  solve_upper_triangular, transpose, xtx_inverse_from_r,
};
use crate::{
  BinaryMleResult, EstimationProblem, EstimationSample, LogitEstimate, ProbitOptions, StatsError,
  fit_binary_probit_with_options,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_HECKMAN_OBSERVATIONS: usize = 10_000;
/// Selection parameters, or augmented outcome parameters INCLUDING Mills.
pub const MAX_HECKMAN_PARAMETERS: usize = 32;
pub const MAX_HECKMAN_WORK: usize = 20_000_000;
const MILLS_NAME: &str = "mills_coefficient";
const NORMAL_95: f64 = 1.959963984540054;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeckmanTwoStepResult {
  /// Model-based probit observed ML inference, not a joint selection likelihood.
  pub selection: BinaryMleResult,
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  /// Outcome parameter order (intercept first), then alpha=rho*sigma.
  pub coefficients: Vec<LogitEstimate>,
  /// Conventional generated-regressor correction; no residual-DF multiplier.
  pub covariance: Vec<Vec<f64>>,
  pub selected_count: usize,
  pub unselected_count: usize,
  pub sigma: f64,
  pub rho: f64,
  pub outcome_sse: f64,
  pub max_abs_scaled_normal_equation: f64,
  /// Positions IN the retained selection stage, in outcome-stage order.
  pub selected_selection_indices: Vec<usize>,
  /// Original candidate positions of selected, eligible units, when supplied.
  pub sample: Option<EstimationSample>,
  /// Nominal exclusions by column name, not a validity/identification test.
  pub excluded_selection_predictors: Vec<String>,
  /// Retained selected rows only; unused unselected X/y are not required.
  pub fitted_selected_means: Vec<f64>,
  pub fitted_selected_mills: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeckmanPredictionKind {
  Xb,
  Pr,
  Mills,
  ConditionalSelectedMean,
  ObservedContributionMean,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("heckman twostep: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("heckman twostep: {message}"))
}

fn validate_sample(problem: &EstimationProblem) -> Result<(), StatsError> {
  problem.validate()?;
  if problem.weights.is_some() {
    return Err(invalid("weights are not implemented"));
  }
  if let Some(s) = &problem.sample {
    if s.weights.is_some() || s.cluster_groups.is_some() {
      return Err(invalid("weights and cluster groups are not implemented"));
    }
    if s.retained_indices.len().checked_add(s.dropped_observations) != Some(s.total_observations)
      || s
        .retained_indices
        .iter()
        .any(|&i| i >= s.total_observations)
      || s.retained_indices.windows(2).any(|p| p[0] >= p[1])
    {
      return Err(invalid(
        "sample indices/counts must be ordered, unique and in range",
      ));
    }
  }
  Ok(())
}

fn full_design(problem: &EstimationProblem) -> Vec<Vec<f64>> {
  problem
    .design_matrix
    .iter()
    .map(|row| {
      let mut values = Vec::with_capacity(problem.parameter_count());
      if problem.include_intercept {
        values.push(1.0);
      }
      values.extend_from_slice(row);
      values
    })
    .collect()
}

fn column_scales(matrix: &[Vec<f64>]) -> Result<Vec<f64>, StatsError> {
  let scales: Vec<f64> = (0..matrix[0].len())
    .map(|j| {
      matrix.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])) / (matrix.len() as f64).sqrt()
    })
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(failed("predictor norms must be finite and positive"));
  }
  Ok(scales)
}

fn mills_terms(index: f64) -> Result<(f64, f64), StatsError> {
  let probability = normal_cdf(index);
  let mills = normal_pdf(index) / probability;
  let delta = mills * (mills + index);
  if !probability.is_finite()
    || probability <= 0.0
    || !mills.is_finite()
    || !delta.is_finite()
    || !(0.0..=1.0).contains(&delta)
  {
    return Err(failed(
      "Mills ratio/curvature is outside the represented numerical range",
    ));
  }
  Ok((mills, delta))
}

fn coefficient(name: String, value: f64, variance: f64) -> Result<LogitEstimate, StatsError> {
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
    return Err(failed(
      "normal coefficient inference is non-finite or has nonpositive variance",
    ));
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

/// Both inputs are already finite/eligible: selection contains ALL eligible
/// units; outcome contains only their D=1 subset in the SAME order. If original
/// samples are supplied, BOTH must be supplied and their identities must match.
/// Outcome columns may differ from selection columns; missing unselected X/y
/// never enter this boundary. CLI missingness policy is applied before calling.
/// At most10000 eligible rows,32 parameters per stage (including Mills), and
/// N*q²+N_selected*p²<=20million. Numerically regular scale requires sigma
/// relative to the centered outcome range (uncentered without intercept)>1e-12;
/// exact/nearly noiseless designs and non-interior correlations fail explicitly.
pub fn fit_heckman_two_step(
  selection: &EstimationProblem,
  outcome: &EstimationProblem,
  options: &ProbitOptions,
) -> Result<HeckmanTwoStepResult, StatsError> {
  validate_sample(selection)?;
  validate_sample(outcome)?;
  if selection.response.iter().any(|&d| d != 0.0 && d != 1.0) {
    return Err(invalid("selection outcome must be exactly 0 or 1"));
  }
  let selected: Vec<usize> = selection
    .response
    .iter()
    .enumerate()
    .filter_map(|(i, &d)| (d == 1.0).then_some(i))
    .collect();
  let n = selection.observation_count();
  let n1 = selected.len();
  let p = outcome.parameter_count() + 1;
  let q = selection.parameter_count();
  if n > MAX_HECKMAN_OBSERVATIONS || p > MAX_HECKMAN_PARAMETERS || q > MAX_HECKMAN_PARAMETERS {
    return Err(invalid(
      "requires at most10000 eligible rows and1..32 parameters per stage (outcome includes Mills)",
    ));
  }
  if n1 == 0 || n1 == n {
    return Err(invalid("requires both selected and unselected units"));
  }
  if n1 != outcome.observation_count() {
    return Err(invalid(
      "outcome rows must match the ordered selected subset",
    ));
  }
  if n <= q || n1 <= p {
    return Err(invalid(
      "requires more eligible/selected rows than their stage parameters",
    ));
  }
  if n * q * q + n1 * p * p > MAX_HECKMAN_WORK {
    return Err(invalid("two-stage design exceeds the expanded-work bound"));
  }
  match (&selection.sample, &outcome.sample) {
    (Some(a), Some(b)) => {
      let expected: Vec<usize> = selected.iter().map(|&i| a.retained_indices[i]).collect();
      if a.total_observations != b.total_observations || expected != b.retained_indices {
        return Err(invalid(
          "outcome sample must be the exact original-row selected subset",
        ));
      }
    }
    (None, None) => {}
    _ => return Err(invalid("both stage samples must be supplied together")),
  }
  let mut names = outcome.parameter_names();
  names.push(MILLS_NAME.to_owned());
  if names.iter().collect::<BTreeSet<_>>().len() != p {
    return Err(invalid(
      "outcome parameter names must be distinct, including intercept and mills_coefficient",
    ));
  }
  let first_stage = fit_binary_probit_with_options(selection, options)?;
  let z = full_design(selection);
  let mut w = full_design(outcome);
  let mut mills = Vec::with_capacity(n1);
  let mut delta = Vec::with_capacity(n1);
  for (row, &index) in w.iter_mut().zip(&selected) {
    let t: f64 = z[index]
      .iter()
      .zip(&first_stage.coefficients)
      .map(|(v, b)| v * b.value)
      .sum();
    let (lambda, curvature) = mills_terms(t)?;
    row.push(lambda);
    mills.push(lambda);
    delta.push(curvature);
  }
  let scales = column_scales(&w)?;
  let z_scales = column_scales(&z)?;
  // Center only when an intercept is fitted. An irrelevant outcome origin
  // must not set the noise scale or destroy translation equivariance.
  let origin = if outcome.include_intercept {
    outcome
      .response
      .iter()
      .copied()
      .fold(f64::INFINITY, f64::min)
  } else {
    0.0
  };
  let centered: Vec<f64> = outcome.response.iter().map(|v| v - origin).collect();
  let response_scale = centered.iter().fold(0.0_f64, |m, &y| m.max(y.abs()));
  if !response_scale.is_finite() || response_scale <= 0.0 || centered.iter().any(|v| !v.is_finite())
  {
    return Err(failed(
      "outcome scale is degenerate or outside the represented numerical range",
    ));
  }
  let y: Vec<f64> = centered.iter().map(|v| v / response_scale).collect();
  for row in &mut w {
    for (v, scale) in row.iter_mut().zip(&scales) {
      *v /= scale;
    }
  }
  let qr = qr_decompose_with_response(&w, &y)?;
  let b = solve_upper_triangular(&qr.r, &qr.qty)?;
  let a_inv = xtx_inverse_from_r(&qr.r)?;
  let fitted: Vec<f64> = w
    .iter()
    .map(|row| row.iter().zip(&b).map(|(v, c)| v * c).sum())
    .collect();
  let residuals: Vec<f64> = y.iter().zip(&fitted).map(|(v, f)| v - f).collect();
  let sse: f64 = residuals.iter().map(|v| v * v).sum();
  let alpha = b[p - 1] / scales[p - 1];
  let sigma_squared = sse / n1 as f64 + alpha * alpha * delta.iter().sum::<f64>() / n1 as f64;
  let sigma_scaled = sigma_squared.sqrt();
  let rho = alpha / sigma_scaled;
  if !sigma_scaled.is_finite() || sigma_scaled <= 1e-12 || !rho.is_finite() || rho.abs() >= 1.0 {
    return Err(failed(
      "two-step scale/correlation is not regular interior (relative sigma>1e-12, abs(rho)<1); clipping/fallback is not implemented",
    ));
  }
  let normal_equation = (0..p)
    .map(|j| {
      w.iter()
        .zip(&residuals)
        .map(|(row, r)| row[j] * r)
        .sum::<f64>()
        .abs()
        / n1 as f64
    })
    .fold(0.0_f64, f64::max);
  if !normal_equation.is_finite() || normal_equation > 1e-10 {
    return Err(failed(
      "selected outcome normal-equation certificate failed",
    ));
  }
  // Expected conditional outcome moments give model-based heteroskedastic
  // variance. The second term accounts for the generated first-stage Mills
  // regressor; ordinary or HC second-stage OLS omits it.
  let mut meat = vec![vec![0.0; p]; p];
  let mut f = vec![vec![0.0; q]; p];
  for (r, (&curvature, &original)) in delta.iter().zip(&selected).enumerate() {
    let conditional_variance = sigma_squared - alpha * alpha * curvature;
    if !conditional_variance.is_finite() || conditional_variance <= 0.0 {
      return Err(failed(
        "conditional outcome variance must be finite and positive",
      ));
    }
    for i in 0..p {
      for j in 0..p {
        meat[i][j] += w[r][i] * w[r][j] * conditional_variance;
      }
      for j in 0..q {
        f[i][j] += w[r][i] * curvature * (z[original][j] / z_scales[j]);
      }
    }
  }
  let selection_covariance: Vec<Vec<f64>> = (0..q)
    .map(|i| {
      (0..q)
        .map(|j| (first_stage.covariance[i][j] * z_scales[i]) * z_scales[j])
        .collect()
    })
    .collect();
  let generated = multiply(&multiply(&f, &selection_covariance)?, &transpose(&f)?)?;
  for i in 0..p {
    for j in 0..p {
      meat[i][j] += alpha * alpha * generated[i][j];
    }
  }
  let normalized_covariance = sandwich(&a_inv, &meat)?;
  let covariance: Vec<Vec<f64>> = (0..p)
    .map(|i| {
      (0..p)
        .map(|j| {
          let symmetric = 0.5 * (normalized_covariance[i][j] + normalized_covariance[j][i]);
          (symmetric * (response_scale / scales[i])) * (response_scale / scales[j])
        })
        .collect()
    })
    .collect();
  let mut coefficients = Vec::with_capacity(p);
  for i in 0..p {
    coefficients.push(coefficient(
      names[i].clone(),
      b[i] * (response_scale / scales[i])
        + if i == 0 && outcome.include_intercept {
          origin
        } else {
          0.0
        },
      covariance[i][i],
    )?);
  }
  let correlation: Vec<Vec<f64>> = (0..p)
    .map(|i| {
      (0..p)
        .map(|j| covariance[i][j] / coefficients[i].standard_error / coefficients[j].standard_error)
        .collect()
    })
    .collect();
  choice_information_inverse(&correlation)
    .map_err(|_| failed("corrected outcome covariance is not positive definite"))?;
  let sigma = sigma_scaled * response_scale;
  let outcome_sse = (sse * response_scale) * response_scale;
  let fitted_selected_means: Vec<f64> =
    fitted.iter().map(|v| v * response_scale + origin).collect();
  if !sigma.is_finite()
    || !outcome_sse.is_finite()
    || covariance.iter().flatten().any(|v| !v.is_finite())
    || fitted_selected_means.iter().any(|v| !v.is_finite())
  {
    return Err(failed(
      "two-step result is outside the represented numerical range",
    ));
  }
  let excluded_selection_predictors = selection
    .predictor_names
    .iter()
    .filter(|name| !outcome.predictor_names.contains(name))
    .cloned()
    .collect();
  Ok(HeckmanTwoStepResult {
    selection: first_stage,
    outcome: outcome.outcome.clone(),
    predictor_names: outcome.predictor_names.clone(),
    include_intercept: outcome.include_intercept,
    coefficients,
    covariance,
    selected_count: n1,
    unselected_count: n - n1,
    sigma,
    rho,
    outcome_sse,
    max_abs_scaled_normal_equation: normal_equation,
    selected_selection_indices: selected,
    sample: outcome.sample.clone(),
    excluded_selection_predictors,
    fitted_selected_means,
    fitted_selected_mills: mills,
  })
}

fn linear_index(
  values: &[f64],
  names: &[String],
  intercept: bool,
  coefficients: &[LogitEstimate],
) -> Result<f64, StatsError> {
  if values.len() != names.len()
    || values.iter().any(|v| !v.is_finite())
    || coefficients.len() != values.len() + usize::from(intercept)
  {
    return Err(invalid(
      "prediction design must match the fitted finite numeric columns",
    ));
  }
  let mut index = if intercept {
    coefficients[0].value
  } else {
    0.0
  };
  for (v, b) in values.iter().zip(&coefficients[usize::from(intercept)..]) {
    index += v * b.value;
  }
  if !index.is_finite() {
    return Err(failed(
      "prediction index is outside the represented numerical range",
    ));
  }
  Ok(index)
}

/// Point projections require ONLY the relevant X/Z. Unused slices are ignored;
/// current outcome and selection flags never enter prediction. `Pr` and the
/// zero-filled contribution can remain representable when Mills is not.
pub fn heckman_predict_row(
  result: &HeckmanTwoStepResult,
  outcome_values: &[f64],
  selection_values: &[f64],
  kind: HeckmanPredictionKind,
) -> Result<f64, StatsError> {
  let k = result
    .coefficients
    .len()
    .checked_sub(1)
    .ok_or_else(|| invalid("missing Mills coefficient"))?;
  let mean = if matches!(
    kind,
    HeckmanPredictionKind::Xb
      | HeckmanPredictionKind::ConditionalSelectedMean
      | HeckmanPredictionKind::ObservedContributionMean
  ) {
    linear_index(
      outcome_values,
      &result.predictor_names,
      result.include_intercept,
      &result.coefficients[..k],
    )?
  } else {
    0.0
  };
  let index = if kind != HeckmanPredictionKind::Xb {
    linear_index(
      selection_values,
      &result.selection.predictor_names,
      result.selection.include_intercept,
      &result.selection.coefficients,
    )?
  } else {
    0.0
  };
  let alpha = result.coefficients[k].value;
  let value = match kind {
    HeckmanPredictionKind::Xb => mean,
    HeckmanPredictionKind::Pr => normal_cdf(index),
    HeckmanPredictionKind::Mills => mills_terms(index)?.0,
    HeckmanPredictionKind::ConditionalSelectedMean => mean + alpha * mills_terms(index)?.0,
    HeckmanPredictionKind::ObservedContributionMean => {
      normal_cdf(index) * mean + alpha * normal_pdf(index)
    }
  };
  if !value.is_finite() {
    return Err(failed(
      "prediction is outside the represented numerical range",
    ));
  }
  Ok(value)
}
