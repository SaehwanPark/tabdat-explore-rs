#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::error::StatsError;
use crate::estimates::CovarianceMatrix;
use crate::matrix::{invert, multiply, multiply_vector, transpose};
use crate::result::LeastSquaresResult;

/// Owned post-estimation model state for hypothesis testing, predictions, and combinations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostEstimationModel {
  /// Ordered parameter names.
  pub parameter_names: Vec<String>,
  /// Point coefficient vector beta.
  pub parameters: Vec<f64>,
  /// Covariance matrix V = Var(beta).
  pub covariance: CovarianceMatrix,
  /// Residual degrees of freedom.
  pub degrees_of_freedom: usize,
}

/// Result of evaluating a linear combination of coefficients (c' * beta).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearCombinationResult {
  /// Estimated point combination value c' * beta.
  pub estimate: f64,
  /// Standard error sqrt(c' * V * c).
  pub standard_error: f64,
  /// Test statistic against null hypothesis c' * beta = 0.
  pub statistic: Option<f64>,
}

/// Result of testing a joint linear hypothesis R * beta = r.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaldTestResult {
  /// Wald F-statistic (W / q).
  pub f_statistic: f64,
  /// Chi-squared statistic W.
  pub chi2_statistic: f64,
  /// Numerator degrees of freedom (number of restrictions q).
  pub df_num: usize,
  /// Denominator degrees of freedom (residual df).
  pub df_denom: usize,
}

impl PostEstimationModel {
  /// Create post-estimation model from an estimated least squares result.
  pub fn from_least_squares(result: &LeastSquaresResult) -> Self {
    let parameters: Vec<f64> = result.coefficients.iter().map(|c| c.value).collect();
    Self {
      parameter_names: result.parameter_names.clone(),
      parameters,
      covariance: result.covariance.clone(),
      degrees_of_freedom: result.fit_statistics.degrees_of_freedom,
    }
  }

  /// Look up estimated coefficient point value by parameter name.
  pub fn parameter(&self, name: &str) -> Result<f64, StatsError> {
    match self.covariance.index_of(name) {
      Some(idx) => Ok(self.parameters[idx]),
      None => Err(StatsError::UnknownParameter(name.to_string())),
    }
  }

  /// Evaluate a linear combination of parameters: c' * beta.
  ///
  /// `weights` must match parameter count and ordering.
  pub fn linear_combination(&self, weights: &[f64]) -> Result<LinearCombinationResult, StatsError> {
    let p = self.parameters.len();
    if weights.len() != p {
      return Err(StatsError::DimensionMismatch(format!(
        "combination weights length ({}) must match parameter count ({p})",
        weights.len()
      )));
    }

    let estimate: f64 = weights
      .iter()
      .zip(self.parameters.iter())
      .map(|(&c, &b)| c * b)
      .sum();

    // Variance = c' * V * c
    let v_c = multiply_vector(&self.covariance.matrix, weights)?;
    let variance: f64 = weights.iter().zip(v_c.iter()).map(|(&c, &vc)| c * vc).sum();
    let se = variance.max(0.0).sqrt();

    let statistic = if se > 0.0 { Some(estimate / se) } else { None };

    Ok(LinearCombinationResult {
      estimate,
      standard_error: se,
      statistic,
    })
  }

  /// Test general linear hypothesis: R * beta = r.
  ///
  /// `r_matrix` is q x P, `r_vector` is q x 1.
  pub fn test_linear_hypothesis(
    &self,
    r_matrix: &[Vec<f64>],
    r_vector: &[f64],
  ) -> Result<WaldTestResult, StatsError> {
    let q = r_matrix.len();
    if q == 0 {
      return Err(StatsError::IncompatibleHypothesis(
        "restriction matrix R must have at least one row".into(),
      ));
    }
    if r_vector.len() != q {
      return Err(StatsError::DimensionMismatch(format!(
        "restriction vector r length ({}) must match R rows ({q})",
        r_vector.len()
      )));
    }
    let p = self.parameters.len();
    for (idx, row) in r_matrix.iter().enumerate() {
      if row.len() != p {
        return Err(StatsError::DimensionMismatch(format!(
          "R row {idx} length ({}) must match parameter count ({p})",
          row.len()
        )));
      }
    }

    // diff = R * beta - r
    let r_beta = multiply_vector(r_matrix, &self.parameters)?;
    let mut diff = vec![0.0; q];
    for i in 0..q {
      diff[i] = r_beta[i] - r_vector[i];
    }

    // middle = (R * V * R')^(-1)
    let r_v = multiply(r_matrix, &self.covariance.matrix)?;
    let r_t = transpose(r_matrix)?;
    let r_v_rt = multiply(&r_v, &r_t)?;
    let r_v_rt_inv = invert(&r_v_rt)?;

    // W = diff' * middle * diff
    let inv_diff = multiply_vector(&r_v_rt_inv, &diff)?;
    let chi2: f64 = diff
      .iter()
      .zip(inv_diff.iter())
      .map(|(&d, &id)| d * id)
      .sum();
    let f_stat = chi2 / (q as f64);

    Ok(WaldTestResult {
      f_statistic: f_stat,
      chi2_statistic: chi2,
      df_num: q,
      df_denom: self.degrees_of_freedom,
    })
  }
}
