#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::error::StatsError;

/// Individual parameter estimate and inferential statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoefficientEstimate {
  /// Parameter or variable name.
  pub name: String,
  /// Estimated point coefficient value beta.
  pub value: f64,
  /// Estimated standard error SE(beta).
  pub standard_error: Option<f64>,
  /// Test statistic (t or z ratio).
  pub statistic: Option<f64>,
  /// Two-sided p-value against the null hypothesis beta = 0.
  pub p_value: Option<f64>,
}

impl CoefficientEstimate {
  /// Construct a coefficient estimate with computed test statistic.
  pub fn new(
    name: impl Into<String>,
    value: f64,
    standard_error: Option<f64>,
    p_value: Option<f64>,
  ) -> Self {
    let statistic = match standard_error {
      Some(se) if se > 0.0 => Some(value / se),
      _ => None,
    };
    Self {
      name: name.into(),
      value,
      standard_error,
      statistic,
      p_value,
    }
  }
}

/// Categorical mode for variance-covariance estimation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CovarianceType {
  /// Classical non-robust OLS covariance: s^2 * (X'X)^(-1).
  NonRobust,
  /// Heteroskedasticity-robust covariance (Stata HC1: n / (n - k) correction).
  RobustHc1,
  /// Cluster-robust covariance clustered on a named entity variable.
  Cluster(String),
}

/// Symmetric variance-covariance matrix of estimated parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CovarianceMatrix {
  /// Ordered parameter names corresponding to rows and columns.
  pub parameter_names: Vec<String>,
  /// Covariance estimator mode.
  pub covariance_type: CovarianceType,
  /// Square symmetric matrix (P x P).
  pub matrix: Vec<Vec<f64>>,
}

impl CovarianceMatrix {
  /// Construct and validate a covariance matrix.
  pub fn new(
    parameter_names: Vec<String>,
    covariance_type: CovarianceType,
    matrix: Vec<Vec<f64>>,
  ) -> Result<Self, StatsError> {
    let dim = parameter_names.len();
    if matrix.len() != dim {
      return Err(StatsError::DimensionMismatch(format!(
        "covariance matrix row count ({}) must match parameter names count ({})",
        matrix.len(),
        dim
      )));
    }
    for (i, row) in matrix.iter().enumerate() {
      if row.len() != dim {
        return Err(StatsError::DimensionMismatch(format!(
          "covariance matrix row {i} length ({}) must match parameter count ({})",
          row.len(),
          dim
        )));
      }
    }
    Ok(Self {
      parameter_names,
      covariance_type,
      matrix,
    })
  }

  /// Dimension (parameter count) of the covariance matrix.
  #[inline]
  pub fn dimension(&self) -> usize {
    self.parameter_names.len()
  }

  /// Standard errors computed from the diagonal elements: sqrt(max(V_ii, 0.0)).
  pub fn standard_errors(&self) -> Vec<f64> {
    (0..self.dimension())
      .map(|i| self.matrix[i][i].max(0.0).sqrt())
      .collect()
  }

  /// Variance of the parameter at index `idx`.
  pub fn variance_at(&self, idx: usize) -> Result<f64, StatsError> {
    if idx >= self.dimension() {
      return Err(StatsError::DimensionMismatch(format!(
        "index {idx} out of bounds for dimension {}",
        self.dimension()
      )));
    }
    Ok(self.matrix[idx][idx])
  }

  /// Covariance between parameter at index `i` and parameter at index `j`.
  pub fn covariance_at(&self, i: usize, j: usize) -> Result<f64, StatsError> {
    let dim = self.dimension();
    if i >= dim || j >= dim {
      return Err(StatsError::DimensionMismatch(format!(
        "indices ({i}, {j}) out of bounds for dimension {dim}"
      )));
    }
    Ok(self.matrix[i][j])
  }

  /// Look up parameter index by name.
  pub fn index_of(&self, name: &str) -> Option<usize> {
    self.parameter_names.iter().position(|p| p == name)
  }
}

/// Convergence and diagnostic metadata for an estimation fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimationDiagnostics {
  /// Optimization or estimation algorithm (e.g. "least_squares", "mcmc").
  pub method: String,
  /// Whether the estimation converged successfully.
  pub converged: bool,
  /// Number of iterations executed (1 for closed-form OLS).
  pub iterations: usize,
  /// Final objective value (e.g. residual sum of squares or negative log-likelihood).
  pub objective_value: Option<f64>,
  /// Weighted or unweighted residual sum of squares (RSS).
  pub residual_sum_of_squares: Option<f64>,
}

/// Overall model goodness-of-fit statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitStatistics {
  /// Number of observations used in estimation (N).
  pub observation_count: usize,
  /// Number of estimated parameters (P).
  pub parameter_count: usize,
  /// Residual degrees of freedom (N - P).
  pub degrees_of_freedom: usize,
  /// Coefficient of determination R^2.
  pub r_squared: Option<f64>,
  /// Adjusted R^2 penalizing additional predictors.
  pub adjusted_r_squared: Option<f64>,
  /// Root mean squared error (residual standard error sigma).
  pub root_mse: Option<f64>,
  /// Residual sum of squares sum(e_i^2).
  pub residual_sum_of_squares: Option<f64>,
  /// Total sum of squares sum((y_i - y_bar)^2).
  pub total_sum_of_squares: Option<f64>,
  /// Overall model F-statistic testing joint significance of non-intercept predictors.
  pub f_statistic: Option<f64>,
  /// Model log-likelihood.
  pub log_likelihood: Option<f64>,
}
