#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::error::StatsError;
use crate::sample::EstimationSample;

/// Specification of an estimation problem for linear and generalized estimators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimationProblem {
  /// Name of the outcome (dependent) variable.
  pub outcome: String,
  /// Ordered names of the predictor (independent) variables corresponding to design columns.
  pub predictor_names: Vec<String>,
  /// Observed numeric response vector y.
  pub response: Vec<f64>,
  /// Design matrix X in row-major order (rows = observations, columns = predictors).
  pub design_matrix: Vec<Vec<f64>>,
  /// Whether to automatically include an intercept column. Defaults to true.
  pub include_intercept: bool,
  /// Parameter label for the intercept. Defaults to "intercept".
  pub intercept_name: String,
  /// Explicit estimation sample tracking.
  pub sample: Option<EstimationSample>,
  /// Optional observation weights (all values must be strictly positive).
  pub weights: Option<Vec<f64>>,
}

impl EstimationProblem {
  /// Create a new EstimationProblem, validating dimensions and data integrity.
  pub fn new(
    outcome: impl Into<String>,
    predictor_names: Vec<String>,
    response: Vec<f64>,
    design_matrix: Vec<Vec<f64>>,
  ) -> Result<Self, StatsError> {
    let problem = Self {
      outcome: outcome.into(),
      predictor_names,
      response,
      design_matrix,
      include_intercept: true,
      intercept_name: "intercept".to_string(),
      sample: None,
      weights: None,
    };
    problem.validate()?;
    Ok(problem)
  }

  /// Builder method: set whether to include intercept.
  pub fn with_intercept(mut self, include_intercept: bool) -> Self {
    self.include_intercept = include_intercept;
    self
  }

  /// Builder method: set custom intercept parameter name.
  pub fn with_intercept_name(mut self, name: impl Into<String>) -> Self {
    self.intercept_name = name.into();
    self
  }

  /// Builder method: attach explicit sample tracking.
  pub fn with_sample(mut self, sample: EstimationSample) -> Self {
    self.sample = Some(sample);
    self
  }

  /// Builder method: attach observation weights.
  pub fn with_weights(mut self, weights: Vec<f64>) -> Self {
    self.weights = Some(weights);
    self
  }

  /// Validate problem invariants.
  pub fn validate(&self) -> Result<(), StatsError> {
    if self.outcome.trim().is_empty() {
      return Err(StatsError::InvalidParameterName(
        "outcome variable name must not be empty".into(),
      ));
    }

    if self.predictor_names.is_empty() && !self.include_intercept {
      return Err(StatsError::InvalidParameterName(
        "model requires at least one predictor or an intercept".into(),
      ));
    }

    for name in &self.predictor_names {
      if name.trim().is_empty() {
        return Err(StatsError::InvalidParameterName(
          "predictor names must not be empty".into(),
        ));
      }
    }

    if self.include_intercept && self.intercept_name.trim().is_empty() {
      return Err(StatsError::InvalidParameterName(
        "intercept_name must not be empty when intercept is included".into(),
      ));
    }

    if self.response.is_empty() {
      return Err(StatsError::EmptySample);
    }

    let nobs = self.response.len();
    if self.design_matrix.len() != nobs {
      return Err(StatsError::DimensionMismatch(format!(
        "design_matrix rows ({}) must match response length ({})",
        self.design_matrix.len(),
        nobs
      )));
    }

    let n_pred = self.predictor_names.len();
    for (row_idx, row) in self.design_matrix.iter().enumerate() {
      if row.len() != n_pred {
        return Err(StatsError::DimensionMismatch(format!(
          "row {row_idx} column count ({}) must match predictor_names length ({})",
          row.len(),
          n_pred
        )));
      }
      for &val in row {
        if !val.is_finite() {
          return Err(StatsError::NonFiniteValue(format!(
            "non-finite value in design matrix at row {row_idx}"
          )));
        }
      }
    }

    for (idx, &val) in self.response.iter().enumerate() {
      if !val.is_finite() {
        return Err(StatsError::NonFiniteValue(format!(
          "non-finite value in response at index {idx}"
        )));
      }
    }

    if let Some(weights) = &self.weights {
      if weights.len() != nobs {
        return Err(StatsError::DimensionMismatch(format!(
          "weights length ({}) must match response length ({})",
          weights.len(),
          nobs
        )));
      }
      for (idx, &w) in weights.iter().enumerate() {
        if !w.is_finite() {
          return Err(StatsError::NonFiniteValue(format!(
            "non-finite weight at index {idx}"
          )));
        }
        if w <= 0.0 {
          return Err(StatsError::NonPositiveWeights);
        }
      }
    }

    if let Some(sample) = &self.sample
      && sample.retained_count() != nobs
    {
      return Err(StatsError::DimensionMismatch(format!(
        "sample retained count ({}) must match response length ({})",
        sample.retained_count(),
        nobs
      )));
    }

    Ok(())
  }

  /// Number of observations in the problem.
  #[inline]
  pub fn observation_count(&self) -> usize {
    self.response.len()
  }

  /// Total number of parameters (predictors + 1 if intercept included).
  #[inline]
  pub fn parameter_count(&self) -> usize {
    self.predictor_names.len() + usize::from(self.include_intercept)
  }

  /// Ordered parameter names (intercept followed by predictors).
  pub fn parameter_names(&self) -> Vec<String> {
    let mut names = Vec::with_capacity(self.parameter_count());
    if self.include_intercept {
      names.push(self.intercept_name.clone());
    }
    names.extend(self.predictor_names.iter().cloned());
    names
  }
}
