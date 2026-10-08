#![forbid(unsafe_code)]

use crate::{EstimationProblem, StatsError, fit_least_squares};

/// Compute variance-inflation factors for an ordered predictor design.
///
/// Each auxiliary model regresses one predictor on the remaining predictors, using the
/// supplied intercept convention. A missing R-squared is returned as `None`; infinite VIF
/// values are preserved when an auxiliary fit explains a predictor exactly.
///
/// # Errors
///
/// Returns `StatsError` when the design is empty or malformed, or when any auxiliary model
/// cannot be estimated.
pub fn variance_inflation_factors(
  predictor_names: &[String],
  design_matrix: &[Vec<f64>],
  include_intercept: bool,
) -> Result<Vec<Option<f64>>, StatsError> {
  if predictor_names.is_empty() {
    return Err(StatsError::InvalidParameterName(
      "VIF requires at least one predictor".to_owned(),
    ));
  }
  if design_matrix.is_empty() {
    return Err(StatsError::EmptySample);
  }
  for (row_index, row) in design_matrix.iter().enumerate() {
    if row.len() != predictor_names.len() {
      return Err(StatsError::DimensionMismatch(format!(
        "VIF design row {row_index} has {} columns, expected {}",
        row.len(),
        predictor_names.len()
      )));
    }
  }

  predictor_names
    .iter()
    .enumerate()
    .map(|(target_index, target_name)| {
      let response = design_matrix
        .iter()
        .map(|row| row[target_index])
        .collect::<Vec<_>>();
      let auxiliary_predictor_names = predictor_names
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != target_index)
        .map(|(_, name)| name.clone())
        .collect::<Vec<_>>();
      let auxiliary_design = design_matrix
        .iter()
        .map(|row| {
          row
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != target_index)
            .map(|(_, value)| *value)
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
      let problem = EstimationProblem::new(
        target_name.clone(),
        auxiliary_predictor_names,
        response,
        auxiliary_design,
      )?
      .with_intercept(include_intercept);
      let auxiliary_fit = fit_least_squares(&problem)?;
      Ok(
        auxiliary_fit
          .r_squared()
          .map(|r_squared| 1.0 / (1.0 - r_squared)),
      )
    })
    .collect()
}
