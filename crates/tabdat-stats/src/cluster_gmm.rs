#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::distributions::chi_square_tail;
use crate::matrix::{choice_information_inverse, multiply, multiply_vector, sandwich, transpose};
use crate::{InstrumentalVariablesProblem, IvOptions, StatsError, fit_two_stage_least_squares};

/// Two-step cluster-moment GMM, including the author's updated-moment diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterGmmResult {
  pub parameter_names: Vec<String>,
  pub coefficients: Vec<f64>,
  pub fitted_values: Vec<f64>,
  pub residuals: Vec<f64>,
  pub fixed_covariance: Vec<Vec<f64>>,
  pub updated_covariance: Vec<Vec<f64>>,
  pub fixed_j: Option<f64>,
  pub updated_j: Option<f64>,
  pub fixed_j_p_value: Option<f64>,
  pub updated_j_p_value: Option<f64>,
  pub cluster_count: usize,
}

// Correlation scaling makes the positive-definiteness test invariant to units.
fn spd_inverse(matrix: &[Vec<f64>], label: &str) -> Result<Vec<Vec<f64>>, StatsError> {
  let n = matrix.len();
  if n == 0 || matrix.iter().any(|row| row.len() != n) {
    return Err(StatsError::DimensionMismatch(format!(
      "{label} must be square"
    )));
  }
  let scales: Vec<_> = (0..n).map(|i| matrix[i][i].sqrt()).collect();
  if scales.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(StatsError::SingularMatrix(format!(
      "{label} is not positive definite"
    )));
  }
  let correlation = (0..n)
    .map(|i| {
      (0..n)
        .map(|j| matrix[i][j] / scales[i] / scales[j])
        .collect()
    })
    .collect::<Vec<Vec<f64>>>();
  let inverse = choice_information_inverse(&correlation)
    .map_err(|_| StatsError::SingularMatrix(format!("{label} is singular or ill-conditioned")))?;
  Ok(
    (0..n)
      .map(|i| {
        (0..n)
          .map(|j| inverse[i][j] / scales[i] / scales[j])
          .collect()
      })
      .collect(),
  )
}

fn moments(z: &[Vec<f64>], residuals: &[f64], groups: &[String]) -> Vec<Vec<f64>> {
  let l = z[0].len();
  let mut sums = BTreeMap::<&str, Vec<f64>>::new();
  for ((row, u), group) in z.iter().zip(residuals).zip(groups) {
    let score = sums.entry(group.as_str()).or_insert_with(|| vec![0.0; l]);
    for j in 0..l {
      score[j] += row[j] * u;
    }
  }
  let mut s = vec![vec![0.0; l]; l];
  for score in sums.values() {
    for i in 0..l {
      for j in 0..l {
        s[i][j] += score[i] * score[j];
      }
    }
  }
  s
}

fn require_residual_variation(
  response: &[f64],
  fitted: &[f64],
  label: &str,
) -> Result<(), StatsError> {
  let scale = response
    .iter()
    .chain(fitted)
    .fold(0.0_f64, |a, v| a.max(v.abs()));
  let max_residual = response
    .iter()
    .zip(fitted)
    .fold(0.0_f64, |a, (y, f)| a.max((y - f).abs()));
  if !scale.is_finite() || !max_residual.is_finite() || max_residual <= 128.0 * f64::EPSILON * scale
  {
    return Err(StatsError::EstimationFailed(format!(
      "{label} residual variation is numerically degenerate"
    )));
  }
  Ok(())
}

fn column_scales(rows: &[Vec<f64>]) -> Result<Vec<f64>, StatsError> {
  let n = rows.len();
  let mut sums = vec![0.0; rows[0].len()];
  for row in rows {
    for (sum, value) in sums.iter_mut().zip(row) {
      *sum += value * value / n as f64;
    }
  }
  let scales = sums.iter().map(|sum| sum.sqrt()).collect::<Vec<_>>();
  if scales
    .iter()
    .any(|value| !value.is_finite() || *value <= 0.0)
  {
    return Err(StatsError::SingularMatrix(
      "GMM design has zero or non-finite column scale".into(),
    ));
  }
  Ok(scales)
}

/// Fit no-intercept, all-endogenous linear IV using one-way cluster moments.
/// Both covariance matrices are uncorrected; J is unavailable when exactly identified.
pub fn fit_cluster_two_step_gmm(
  problem: &InstrumentalVariablesProblem,
) -> Result<ClusterGmmResult, StatsError> {
  problem.validate()?;
  let n = problem.response.len();
  let k = problem.endogenous_names.len();
  let l = problem.instrument_names.len();
  if problem.include_intercept || !problem.exogenous_names.is_empty() {
    return Err(StatsError::IncompatibleHypothesis(
      "cluster GMM requires noconstant and no included exogenous variables".into(),
    ));
  }
  if n > 10_000 || k > 8 || l > 16 {
    return Err(StatsError::IncompatibleHypothesis(
      "cluster GMM supports at most 10000 rows, 8 regressors and 16 instruments".into(),
    ));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|s| s.cluster_groups.as_ref())
    .ok_or_else(|| {
      StatsError::IncompatibleHypothesis("cluster GMM requires cluster identifiers".into())
    })?;
  if groups.len() != n {
    return Err(StatsError::DimensionMismatch(
      "cluster identifier count must equal sample size".into(),
    ));
  }
  let cluster_count = groups
    .iter()
    .collect::<std::collections::BTreeSet<_>>()
    .len();
  if cluster_count <= l {
    return Err(StatsError::InsufficientObservations(
      "cluster GMM requires more clusters than instruments".into(),
    ));
  }
  let pilot = fit_two_stage_least_squares(problem, &IvOptions::default())?;
  require_residual_variation(&problem.response, &pilot.regression.fitted_values, "pilot")?;
  let sx = column_scales(&problem.endogenous)?;
  let sz = column_scales(&problem.instruments)?;
  let x: Vec<Vec<f64>> = problem
    .endogenous
    .iter()
    .map(|row| row.iter().zip(&sx).map(|(v, s)| v / s).collect())
    .collect();
  let z: Vec<Vec<f64>> = problem
    .instruments
    .iter()
    .map(|row| row.iter().zip(&sz).map(|(v, s)| v / s).collect())
    .collect();
  let zt = transpose(&z)?;
  let a = multiply(&zt, &x)?;
  let b = multiply_vector(&zt, &problem.response)?;
  let s1 = moments(&z, &pilot.regression.residuals, groups);
  let w1 = spd_inverse(&s1, "pilot cluster moment covariance")?;
  let at = transpose(&a)?;
  let atw = multiply(&at, &w1)?;
  let bread = spd_inverse(&multiply(&atw, &a)?, "GMM information")?;
  let scaled_beta = multiply_vector(&bread, &multiply_vector(&atw, &b)?)?;
  let coefficients: Vec<_> = scaled_beta.iter().zip(&sx).map(|(b, s)| b / s).collect();
  let fitted_values = multiply_vector(&problem.endogenous, &coefficients)?;
  require_residual_variation(&problem.response, &fitted_values, "updated")?;
  let residuals: Vec<_> = problem
    .response
    .iter()
    .zip(&fitted_values)
    .map(|(y, f)| y - f)
    .collect();
  let s2 = moments(&z, &residuals, groups);
  let w2 = spd_inverse(&s2, "updated cluster moment covariance")?;
  let fixed_meat = sandwich(&atw, &s2)?;
  let fixed_scaled = sandwich(&bread, &fixed_meat)?;
  let updated_scaled = spd_inverse(
    &multiply(&multiply(&at, &w2)?, &a)?,
    "updated GMM information",
  )?;
  let rescale = |v: &[Vec<f64>]| -> Vec<Vec<f64>> {
    (0..k)
      .map(|i| (0..k).map(|j| v[i][j] / sx[i] / sx[j]).collect())
      .collect()
  };
  let fixed_covariance = rescale(&fixed_scaled);
  let updated_covariance = rescale(&updated_scaled);
  if (0..k).any(|i| {
    !fixed_covariance[i][i].is_finite()
      || fixed_covariance[i][i] <= 0.0
      || !updated_covariance[i][i].is_finite()
      || updated_covariance[i][i] <= 0.0
  }) {
    return Err(StatsError::EstimationFailed(
      "GMM coefficient covariance has nonpositive or non-finite variance".into(),
    ));
  }
  let m = multiply_vector(&zt, &residuals)?;
  let objective = |w: &[Vec<f64>]| -> Result<f64, StatsError> {
    let wm = multiply_vector(w, &m)?;
    let value = m.iter().zip(wm).map(|(a, b)| a * b).sum::<f64>();
    if !value.is_finite() || value < 0.0 {
      return Err(StatsError::EstimationFailed(
        "GMM objective is non-finite or negative".into(),
      ));
    }
    Ok(value)
  };
  let (fixed_j, updated_j, fixed_j_p_value, updated_j_p_value) = if l > k {
    let j1 = objective(&w1)?;
    let j2 = objective(&w2)?;
    (
      Some(j1),
      Some(j2),
      Some(chi_square_tail(j1, l - k)),
      Some(chi_square_tail(j2, l - k)),
    )
  } else {
    (None, None, None, None)
  };
  if fixed_j_p_value
    .iter()
    .chain(updated_j_p_value.iter())
    .any(|p| !p.is_finite())
  {
    return Err(StatsError::EstimationFailed(
      "GMM overidentification probability is non-finite".into(),
    ));
  }
  if coefficients
    .iter()
    .chain(&fitted_values)
    .chain(&residuals)
    .chain(fixed_covariance.iter().flatten())
    .chain(updated_covariance.iter().flatten())
    .any(|v| !v.is_finite())
  {
    return Err(StatsError::EstimationFailed(
      "GMM result contains non-finite values".into(),
    ));
  }
  Ok(ClusterGmmResult {
    parameter_names: problem.endogenous_names.clone(),
    coefficients,
    fitted_values,
    residuals,
    fixed_covariance,
    updated_covariance,
    fixed_j,
    updated_j,
    fixed_j_p_value,
    updated_j_p_value,
    cluster_count,
  })
}
