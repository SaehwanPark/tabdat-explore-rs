use crate::distributions::chi_square_tail;
use crate::{EstimationProblem, InstrumentalVariablesProblem, StatsError, fit_least_squares};
use serde::{Deserialize, Serialize};

/// Homoskedastic, asymptotic Sargan N*R² test, not Basmann or robust Hansen J.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SarganResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub degrees_of_freedom: usize,
  pub statistic: f64,
  pub p_value: f64,
}

/// Test overidentifying restrictions using retained 2SLS structural residuals.
/// The caller must supply residuals from this exact unweighted design/sample,
/// not auxiliary second-stage residuals. No 2SLS refit or sample selection occurs.
/// Inference assumes homoskedasticity, a correctly specified maintained model,
/// and standard strong-identification asymptotics. Failure to reject does not
/// prove instrument validity or resolve weak identification.
pub fn sargan_test(
  problem: &InstrumentalVariablesProblem,
  residuals: &[f64],
) -> Result<SarganResult, StatsError> {
  problem.validate()?;
  if problem.instrument_names.len() <= problem.endogenous_names.len() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan requires overidentification: more excluded instruments than endogenous regressors"
        .into(),
    ));
  }
  if residuals.len() != problem.response.len() {
    return Err(StatsError::DimensionMismatch(
      "Sargan residuals must match the retained estimation sample".into(),
    ));
  }
  if residuals.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "Sargan structural residuals must be finite".into(),
    ));
  }
  let rss = residuals.iter().map(|value| value * value).sum::<f64>();
  if !rss.is_finite() || rss <= 0.0 {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan structural residual variance is undefined or non-finite".into(),
    ));
  }
  let names = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect();
  let design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(w, z)| w.iter().chain(z).copied().collect())
    .collect();
  let auxiliary = fit_least_squares(
    &EstimationProblem::new(
      "sargan_structural_residual",
      names,
      residuals.to_vec(),
      design,
    )?
    .with_intercept(problem.include_intercept),
  )?;
  let projected_rss = auxiliary
    .fitted_values
    .iter()
    .map(|value| value * value)
    .sum::<f64>();
  if !projected_rss.is_finite() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan projected residual norm is non-finite".into(),
    ));
  }
  // Direct projection norm avoids cancellation when the statistic is tiny.
  // Uncentered u'u also defines noconstant N*R²; no N-K correction. Projection
  // cannot increase the norm in exact arithmetic; cap roundoff at N.
  let statistic = problem.response.len() as f64 * (projected_rss / rss).min(1.0);
  let df = problem.instrument_names.len() - problem.endogenous_names.len();
  let p_value = chi_square_tail(statistic, df);
  if !statistic.is_finite() || !p_value.is_finite() {
    return Err(StatsError::IncompatibleHypothesis(
      "Sargan statistic or chi-square tail is unavailable".into(),
    ));
  }
  Ok(SarganResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: residuals.len(),
    degrees_of_freedom: df,
    statistic,
    p_value,
  })
}
