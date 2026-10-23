use crate::{
  EstimationProblem, InstrumentalVariablesProblem, StatsError, fit_least_squares, transpose,
};
use serde::{Deserialize, Serialize};

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

/// Independent-row heteroskedasticity-robust Wooldridge overidentification score.
/// The uncentered score OPG has no N-K correction. This is not Hansen J, a
/// cluster/survey test, weak-IV-robust inference or proof of instrument validity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustOveridScoreResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub instrument_parameter_count: usize,
  pub structural_parameter_count: usize,
  pub degrees_of_freedom: usize,
  pub covariance_correction: f64,
  pub statistic: f64,
  pub p_value: f64,
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
  problem.validate()?;
  let df = problem
    .instrument_names
    .len()
    .checked_sub(problem.endogenous_names.len())
    .filter(|q| *q > 0)
    .ok_or_else(|| {
      invalid("requires overidentification: more excluded instruments than endogenous regressors")
    })?;
  let n = problem.response.len();
  if residuals.len() != n {
    return Err(StatsError::DimensionMismatch(
      "robust overidentification residuals must match the retained estimation sample".into(),
    ));
  }
  let u = unit_column(residuals)?;
  let mut z = Vec::new();
  if problem.include_intercept {
    z.push(vec![1.0; n]);
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
  let basis = complement_basis(complement, df)?;
  let scores = basis
    .iter()
    .map(|q| q.iter().zip(&u).map(|(a, b)| a * b).collect::<Vec<_>>())
    .collect::<Vec<_>>();
  let score_fit = auxiliary_fit(&scores, vec![1.0; n])?;
  // Projection norm is N-RSS without subtracting nearly equal numbers for a
  // small statistic. Unit rescaling of u or the basis leaves this space intact.
  let statistic = score_fit.fitted_values.iter().map(|v| v * v).sum::<f64>();
  if !statistic.is_finite() {
    return Err(invalid("nonfinite score projection norm"));
  }
  let statistic = statistic.min(n as f64);
  let p_value = crate::distributions::chi_square_tail(statistic, df);
  if !statistic.is_finite() || !p_value.is_finite() {
    return Err(invalid("statistic or chi-square tail unavailable"));
  }
  Ok(RobustOveridScoreResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: n,
    instrument_parameter_count: z.len(),
    structural_parameter_count: projected.len(),
    degrees_of_freedom: df,
    covariance_correction: 1.0,
    statistic,
    p_value,
  })
}
