#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::estimates::{
  CoefficientEstimate, CovarianceMatrix, EstimationDiagnostics, FitStatistics,
};
use crate::sample::EstimationSample;

/// Complete result returned after fitting a linear least squares estimation problem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeastSquaresResult {
  /// Parameter names in order (intercept followed by predictors).
  pub parameter_names: Vec<String>,
  /// Point estimates and standard errors for each parameter.
  pub coefficients: Vec<CoefficientEstimate>,
  /// In-sample fitted values y_hat = X * beta.
  pub fitted_values: Vec<f64>,
  /// Residuals e = y - y_hat.
  pub residuals: Vec<f64>,
  /// Parameter variance-covariance matrix.
  pub covariance: CovarianceMatrix,
  /// Convergence and method diagnostics.
  pub diagnostics: EstimationDiagnostics,
  /// Goodness of fit and model statistics.
  pub fit_statistics: FitStatistics,
  /// Explicit sample tracking.
  pub sample: Option<EstimationSample>,
}

impl LeastSquaresResult {
  /// Look up a coefficient estimate by name.
  pub fn coefficient(&self, name: &str) -> Option<&CoefficientEstimate> {
    self.coefficients.iter().find(|c| c.name == name)
  }

  /// Look up estimated point value by name.
  pub fn parameter_value(&self, name: &str) -> Option<f64> {
    self.coefficient(name).map(|c| c.value)
  }

  /// Convenience accessor for R^2.
  pub fn r_squared(&self) -> Option<f64> {
    self.fit_statistics.r_squared
  }

  /// Convenience accessor for adjusted R^2.
  pub fn adjusted_r_squared(&self) -> Option<f64> {
    self.fit_statistics.adjusted_r_squared
  }

  /// Convenience accessor for Root MSE.
  pub fn root_mse(&self) -> Option<f64> {
    self.fit_statistics.root_mse
  }

  /// Observation count (N).
  pub fn observation_count(&self) -> usize {
    self.fit_statistics.observation_count
  }

  /// Residual degrees of freedom.
  pub fn degrees_of_freedom(&self) -> usize {
    self.fit_statistics.degrees_of_freedom
  }
}
