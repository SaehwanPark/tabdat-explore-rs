use crate::{
  EstimationProblem, InstrumentalVariablesProblem, StatsError, fit_least_squares, transpose,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn invalid(message: &str) -> StatsError {
  StatsError::IncompatibleHypothesis(format!("robust overidentification score: {message}"))
}

// Scaling before summing squares avoids overflow/underflow from variable units.
fn unit_column(values: &[f64]) -> Result<Vec<f64>, StatsError> {
  if values.iter().any(|v| !v.is_finite()) {
    return Err(invalid("nonfinite auxiliary column"));
  }
  let scale = values.iter().map(|v| v.abs()).fold(0.0, f64::max);
  if scale == 0.0 {
    return Err(invalid(
      "zero auxiliary column or structural residual variance",
    ));
  }
  let norm = values
    .iter()
    .map(|v| (v / scale).powi(2))
    .sum::<f64>()
    .sqrt();
  Ok(values.iter().map(|v| (v / scale) / norm).collect())
}

fn cluster_score_statistic(group_score_columns: &[Vec<f64>]) -> Result<f64, StatsError> {
  let q = group_score_columns.len();
  let group_count = group_score_columns.first().map_or(0, Vec::len);
  if q == 0 || group_count <= q || group_score_columns.iter().any(|c| c.len() != group_count) {
    return Err(invalid("clustered score covariance has insufficient rank"));
  }
  let normalized = group_score_columns
    .iter()
    .map(|column| unit_column(column).map_err(|_| invalid("singular clustered score covariance")))
    .collect::<Result<Vec<_>, _>>()?;
  let score_sum = normalized
    .iter()
    .map(|column| column.iter().sum::<f64>())
    .collect::<Vec<_>>();
  let mut correlation = vec![vec![0.0; q]; q];
  for (i, row) in correlation.iter_mut().enumerate() {
    for (j, value) in row.iter_mut().enumerate() {
      *value = normalized[i]
        .iter()
        .zip(&normalized[j])
        .map(|(a, b)| a * b)
        .sum();
    }
  }
  let inverse = crate::matrix::invert(&correlation)
    .map_err(|_| invalid("singular clustered score covariance"))?;
  let statistic = score_sum
    .iter()
    .enumerate()
    .map(|(i, a)| {
      score_sum
        .iter()
        .enumerate()
        .map(|(j, b)| a * inverse[i][j] * b)
        .sum::<f64>()
    })
    .sum::<f64>();
  if !statistic.is_finite() || statistic < 0.0 {
    return Err(invalid("clustered score statistic is unavailable"));
  }
  Ok(statistic)
}

fn auxiliary_fit(
  columns: &[Vec<f64>],
  response: Vec<f64>,
) -> Result<crate::LeastSquaresResult, StatsError> {
  let columns = columns
    .iter()
    .map(|c| unit_column(c))
    .collect::<Result<Vec<_>, _>>()?;
  let design = transpose(&columns)?;
  let names = (0..columns.len())
    .map(|j| format!("score_column_{j}"))
    .collect();
  fit_least_squares(
    &EstimationProblem::new("score_response", names, response, design)?.with_intercept(false),
  )
}

// The whole residualized instrument space has rank L-K, but its original columns
// are dependent. Pivoted, twice-reorthogonalized Gram-Schmidt selects exactly
// that many directions; it never silently lowers the test DF. Inputs came from
// unit-normalized instruments, so the 1e-12 rank bound is dimensionless.
fn complement_basis(
  mut columns: Vec<Vec<f64>>,
  dimension: usize,
) -> Result<Vec<Vec<f64>>, StatsError> {
  let mut basis = Vec::with_capacity(dimension);
  for _ in 0..dimension {
    let (index, norm_squared) = columns
      .iter()
      .enumerate()
      .map(|(i, c)| (i, c.iter().map(|v| v * v).sum::<f64>()))
      .max_by(|a, b| a.1.total_cmp(&b.1))
      .ok_or_else(|| invalid("instrument complement rank unavailable"))?;
    if !norm_squared.is_finite() || norm_squared <= 1e-24 {
      return Err(invalid(
        "instrument complement is numerically rank deficient",
      ));
    }
    let direction = columns
      .remove(index)
      .into_iter()
      .map(|v| v / norm_squared.sqrt())
      .collect::<Vec<_>>();
    for column in &mut columns {
      for _ in 0..2 {
        let projection = column
          .iter()
          .zip(&direction)
          .map(|(a, b)| a * b)
          .sum::<f64>();
        for (value, q) in column.iter_mut().zip(&direction) {
          *value -= projection * q;
        }
      }
    }
    basis.push(direction);
  }
  Ok(basis)
}

/// Result of the robust residualized-instrument overidentification score test.
/// Optional group fields are present only for the one-way clustered CR0 path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustOveridScoreResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub instrument_parameter_count: usize,
  pub structural_parameter_count: usize,
  pub degrees_of_freedom: usize,
  /// Score OPG multiplier; this score test uses 1.0 for HC0 and clustered CR0.
  pub covariance_correction: f64,
  /// Retained parent cluster variable; absent for the independent-row score.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster_variable: Option<String>,
  /// Number of distinct retained groups; absent for the independent-row score.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cluster_group_count: Option<usize>,
  pub statistic: f64,
  pub p_value: f64,
}

struct OveridScoreWork {
  observation_count: usize,
  instrument_parameter_count: usize,
  structural_parameter_count: usize,
  degrees_of_freedom: usize,
  scores: Vec<Vec<f64>>,
}

fn overid_score_work(
  problem: &InstrumentalVariablesProblem,
  residuals: &[f64],
) -> Result<OveridScoreWork, StatsError> {
  problem.validate()?;
  let degrees_of_freedom = problem
    .instrument_names
    .len()
    .checked_sub(problem.endogenous_names.len())
    .filter(|q| *q > 0)
    .ok_or_else(|| {
      invalid("requires overidentification: more excluded instruments than endogenous regressors")
    })?;
  let observation_count = problem.response.len();
  if residuals.len() != observation_count {
    return Err(StatsError::DimensionMismatch(
      "robust overidentification residuals must match the retained estimation sample".into(),
    ));
  }
  let u = unit_column(residuals)?;
  let mut z = Vec::new();
  if problem.include_intercept {
    z.push(vec![1.0; observation_count]);
  }
  for j in 0..problem.exogenous_names.len() {
    z.push(problem.exogenous.iter().map(|row| row[j]).collect());
  }
  let mut projected = z.clone();
  for j in 0..problem.instrument_names.len() {
    z.push(problem.instruments.iter().map(|row| row[j]).collect());
  }
  let z = z
    .iter()
    .map(|c| unit_column(c))
    .collect::<Result<Vec<_>, _>>()?;
  for j in 0..problem.endogenous_names.len() {
    let x = unit_column(
      &problem
        .endogenous
        .iter()
        .map(|row| row[j])
        .collect::<Vec<_>>(),
    )?;
    projected.push(auxiliary_fit(&z, x)?.fitted_values);
  }
  let complement = z
    .iter()
    .map(|column| auxiliary_fit(&projected, column.clone()).map(|fit| fit.residuals))
    .collect::<Result<Vec<_>, _>>()?;
  let basis = complement_basis(complement, degrees_of_freedom)?;
  let scores = basis
    .iter()
    .map(|q| q.iter().zip(&u).map(|(a, b)| a * b).collect::<Vec<_>>())
    .collect::<Vec<_>>();
  Ok(OveridScoreWork {
    observation_count,
    instrument_parameter_count: z.len(),
    structural_parameter_count: projected.len(),
    degrees_of_freedom,
    scores,
  })
}

/// Test overidentification using structural residuals from this exact retained
/// unweighted 2SLS design/sample. No structural refit or row selection occurs.
///
/// With Z the full instruments and Xhat=P_Z X, form a rank-(L-K) basis Q for
/// (I-P_Xhat)Z, then regress ones on diag(u)Q without an intercept. N-RSS has an
/// asymptotic chi-square(L-K) reference under valid instruments, a correctly
/// specified maintained model and conventional strong-identification regularity.
/// Under the null the projected-out space removes the first-order effect of
/// 2SLS nuisance estimation; using all raw moments with an arbitrary weight
/// would not in general produce a chi-square(L-K) statistic.
pub fn robust_overid_score_test(
  problem: &InstrumentalVariablesProblem,
  residuals: &[f64],
) -> Result<RobustOveridScoreResult, StatsError> {
  if problem
    .sample
    .as_ref()
    .is_some_and(|sample| sample.cluster_groups.is_some())
  {
    return Err(invalid(
      "retained cluster identifiers require the clustered score test",
    ));
  }
  let work = overid_score_work(problem, residuals)?;
  let score_fit = auxiliary_fit(&work.scores, vec![1.0; work.observation_count])?;
  // Projection norm is N-RSS without subtracting nearly equal numbers for a
  // small statistic. Unit rescaling of u or the basis leaves this space intact.
  let statistic = score_fit.fitted_values.iter().map(|v| v * v).sum::<f64>();
  if !statistic.is_finite() {
    return Err(invalid("nonfinite score projection norm"));
  }
  let statistic = statistic.min(work.observation_count as f64);
  let p_value = crate::distributions::chi_square_tail(statistic, work.degrees_of_freedom);
  if !statistic.is_finite() || !p_value.is_finite() {
    return Err(invalid("statistic or chi-square tail unavailable"));
  }
  Ok(RobustOveridScoreResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: work.observation_count,
    instrument_parameter_count: work.instrument_parameter_count,
    structural_parameter_count: work.structural_parameter_count,
    degrees_of_freedom: work.degrees_of_freedom,
    covariance_correction: 1.0,
    cluster_variable: None,
    cluster_group_count: None,
    statistic,
    p_value,
  })
}

/// One-way cluster-robust Wooldridge overidentification score using the retained
/// IV sample's group IDs. Cluster score sums use an uncentered CR0 covariance:
/// the score test has no regression-residual degrees-of-freedom correction.
/// Its chi-square(q) reference requires independent groups, many groups and
/// conventional strong-identification regularity; it is not a few-cluster test.
pub fn cluster_robust_overid_score_test(
  problem: &InstrumentalVariablesProblem,
  residuals: &[f64],
  cluster_variable: &str,
) -> Result<RobustOveridScoreResult, StatsError> {
  if cluster_variable.trim().is_empty() {
    return Err(invalid("cluster variable name must not be empty"));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|sample| sample.cluster_groups.as_deref())
    .ok_or_else(|| invalid("clustered score requires retained cluster identifiers"))?;
  let work = overid_score_work(problem, residuals)?;
  if groups.len() != work.observation_count {
    return Err(StatsError::DimensionMismatch(format!(
      "cluster identifier count ({}) must match the retained estimation sample ({})",
      groups.len(),
      work.observation_count
    )));
  }

  let mut cluster_scores = BTreeMap::<&str, Vec<f64>>::new();
  for (index, group) in groups.iter().enumerate() {
    let sum = cluster_scores
      .entry(group.as_str())
      .or_insert_with(|| vec![0.0; work.degrees_of_freedom]);
    for (j, column) in work.scores.iter().enumerate() {
      sum[j] += column[index];
    }
  }
  let cluster_group_count = cluster_scores.len();
  if cluster_group_count <= work.degrees_of_freedom {
    return Err(StatsError::InsufficientObservations(format!(
      "clustered overidentification score requires more clusters than test degrees of freedom ({})",
      work.degrees_of_freedom
    )));
  }

  let mut group_score_columns = (0..work.degrees_of_freedom)
    .map(|_| Vec::with_capacity(cluster_group_count))
    .collect::<Vec<_>>();
  for score in cluster_scores.values() {
    for (j, value) in score.iter().enumerate() {
      if !value.is_finite() {
        return Err(invalid("nonfinite cluster score sum"));
      }
      group_score_columns[j].push(*value);
    }
  }
  // Rescaling each score direction leaves the Wald quadratic unchanged and
  // keeps the covariance inversion independent of instrument/residual units.
  let statistic = cluster_score_statistic(&group_score_columns)?;
  let p_value = crate::distributions::chi_square_tail(statistic, work.degrees_of_freedom);
  if !p_value.is_finite() {
    return Err(invalid("clustered chi-square tail unavailable"));
  }
  Ok(RobustOveridScoreResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: work.observation_count,
    instrument_parameter_count: work.instrument_parameter_count,
    structural_parameter_count: work.structural_parameter_count,
    degrees_of_freedom: work.degrees_of_freedom,
    covariance_correction: 1.0,
    cluster_variable: Some(cluster_variable.to_owned()),
    cluster_group_count: Some(cluster_group_count),
    statistic,
    p_value,
  })
}

#[cfg(test)]
mod tests {
  use super::cluster_score_statistic;

  #[test]
  fn clustered_score_covariance_rejects_dependent_group_scores() {
    let grouped_scores = [vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0]];
    assert!(cluster_score_statistic(&grouped_scores).is_err());
  }
}
