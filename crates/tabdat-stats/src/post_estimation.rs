#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::distributions::chi_square_tail;
use crate::error::StatsError;
use crate::estimates::{CovarianceMatrix, CovarianceType};
use crate::matrix::{
  invert, multiply, multiply_vector, student_t_pvalue, student_t_quantile, transpose,
};
use crate::result::LeastSquaresResult;
use std::collections::BTreeSet;

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

/// Result of a linear combination with Student-t inference and a confidence interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearCombinationInferenceResult {
  /// Estimated point combination value, including any constant offset.
  pub estimate: f64,
  /// Standard error sqrt(c' * V * c).
  pub standard_error: f64,
  /// Test statistic against the null hypothesis that the combination equals zero.
  pub statistic: Option<f64>,
  /// Two-sided Student-t p-value, or NaN when the standard error is zero.
  pub p_value: f64,
  /// Lower confidence interval endpoint.
  pub ci_lower: f64,
  /// Upper confidence interval endpoint.
  pub ci_upper: f64,
  /// Confidence level in percent.
  pub confidence_level: f64,
  /// Residual degrees of freedom used for inference.
  pub degrees_of_freedom: usize,
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

/// Asymptotic Wald test against a chi-square reference distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaldChiSquareTestResult {
  /// Chi-square Wald statistic.
  pub statistic: f64,
  /// Number of tested restrictions and chi-square degrees of freedom.
  pub degrees_of_freedom: usize,
  /// Upper-tail chi-square probability.
  pub p_value: f64,
}

pub(crate) fn validate_mean_coefficient_names(
  coefficient_names: &[String],
  predictor_names: &[String],
  include_intercept: bool,
) -> Result<(), StatsError> {
  let unique_names = coefficient_names
    .iter()
    .map(String::as_str)
    .collect::<BTreeSet<_>>();
  let predictor_order_matches = coefficient_names
    .iter()
    .skip(usize::from(include_intercept))
    .map(String::as_str)
    .eq(predictor_names.iter().map(String::as_str));
  if coefficient_names.len() != predictor_names.len() + usize::from(include_intercept)
    || unique_names.len() != coefficient_names.len()
    || coefficient_names.iter().any(String::is_empty)
    || !predictor_order_matches
  {
    return Err(StatsError::IncompatibleHypothesis(
      "Wald test requires the exact fitted mean coefficient names and order".into(),
    ));
  }
  Ok(())
}

pub(crate) fn wald_chi_square_test(
  parameters: &[f64],
  covariance: &[Vec<f64>],
  r_matrix: &[Vec<f64>],
  r_vector: &[f64],
) -> Result<WaldChiSquareTestResult, StatsError> {
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
  let p = parameters.len();
  if p == 0 || covariance.len() != p {
    return Err(StatsError::DimensionMismatch(format!(
      "covariance row count ({}) must match positive parameter count ({p})",
      covariance.len()
    )));
  }
  if parameters.iter().any(|value| !value.is_finite())
    || r_vector.iter().any(|value| !value.is_finite())
    || r_matrix.iter().flatten().any(|value| !value.is_finite())
    || covariance.iter().flatten().any(|value| !value.is_finite())
  {
    return Err(StatsError::IncompatibleHypothesis(
      "Wald parameters, covariance and restrictions must be finite".into(),
    ));
  }
  for (idx, row) in r_matrix.iter().enumerate() {
    if row.len() != p {
      return Err(StatsError::DimensionMismatch(format!(
        "R row {idx} length ({}) must match parameter count ({p})",
        row.len()
      )));
    }
  }
  for (idx, row) in covariance.iter().enumerate() {
    if row.len() != p {
      return Err(StatsError::DimensionMismatch(format!(
        "covariance row {idx} length ({}) must match parameter count ({p})",
        row.len()
      )));
    }
  }

  let r_beta = multiply_vector(r_matrix, parameters)?;
  let difference = r_beta
    .iter()
    .zip(r_vector)
    .map(|(value, null)| value - null)
    .collect::<Vec<_>>();
  let r_v = multiply(r_matrix, covariance)?;
  let r_v_rt = multiply(&r_v, &transpose(r_matrix)?)?;
  let covariance_scale = r_v_rt
    .iter()
    .flatten()
    .fold(0.0_f64, |scale, value| scale.max(value.abs()));
  if covariance_scale == 0.0 || !covariance_scale.is_finite() {
    return Err(StatsError::SingularMatrix(
      "restriction covariance matrix is zero or non-finite".into(),
    ));
  }
  let normalized_covariance = r_v_rt
    .iter()
    .map(|row| row.iter().map(|value| value / covariance_scale).collect())
    .collect::<Vec<Vec<_>>>();
  let inverse = invert(&normalized_covariance)?;
  let difference_scale = difference
    .iter()
    .fold(0.0_f64, |scale, value| scale.max(value.abs()));
  let statistic = if difference_scale == 0.0 {
    0.0
  } else {
    let normalized_difference = difference
      .iter()
      .map(|value| value / difference_scale)
      .collect::<Vec<_>>();
    let inverse_difference = multiply_vector(&inverse, &normalized_difference)?;
    let normalized_quadratic = normalized_difference
      .iter()
      .zip(&inverse_difference)
      .map(|(value, inverse_value)| value * inverse_value)
      .sum::<f64>();
    let scale = difference_scale / covariance_scale.sqrt();
    normalized_quadratic * scale * scale
  };
  if !statistic.is_finite() || statistic < 0.0 {
    return Err(StatsError::IncompatibleHypothesis(
      "Wald chi-square statistic is negative or non-finite".into(),
    ));
  }
  let p_value = chi_square_tail(statistic, q);
  if !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
    return Err(StatsError::IncompatibleHypothesis(
      "Wald chi-square probability is invalid".into(),
    ));
  }
  Ok(WaldChiSquareTestResult {
    statistic,
    degrees_of_freedom: q,
    p_value,
  })
}

impl PostEstimationModel {
  /// Create post-estimation model from an estimated least squares result.
  pub fn from_least_squares(result: &LeastSquaresResult) -> Self {
    let parameters: Vec<f64> = result.coefficients.iter().map(|c| c.value).collect();
    // Cluster post-estimation must use the same G-1 inference df as coefficients,
    // while fit statistics continue to report residual N-K degrees of freedom.
    let degrees_of_freedom = match (
      &result.covariance.covariance_type,
      result
        .sample
        .as_ref()
        .and_then(|sample| sample.cluster_groups.as_ref()),
    ) {
      (CovarianceType::Cluster(_), Some(groups)) => groups
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
        .saturating_sub(1),
      _ => result.fit_statistics.degrees_of_freedom,
    };
    Self {
      parameter_names: result.parameter_names.clone(),
      parameters,
      covariance: result.covariance.clone(),
      degrees_of_freedom,
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
    self.linear_combination_with_offset(weights, 0.0)
  }

  /// Evaluate a linear combination with a constant offset: a + c' * beta.
  ///
  /// The offset shifts the estimate and null statistic, but does not change variance.
  pub fn linear_combination_with_offset(
    &self,
    weights: &[f64],
    offset: f64,
  ) -> Result<LinearCombinationResult, StatsError> {
    let p = self.parameters.len();
    if weights.len() != p {
      return Err(StatsError::DimensionMismatch(format!(
        "combination weights length ({}) must match parameter count ({p})",
        weights.len()
      )));
    }

    let estimate = offset
      + weights
        .iter()
        .zip(self.parameters.iter())
        .map(|(&c, &b)| c * b)
        .sum::<f64>();

    // Variance = c' * V * c; the constant offset has no variance.
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

  /// Evaluate a linear combination and compute its two-sided Student-t inference.
  ///
  /// `weights` follow parameter order; `offset` is a coefficient-independent constant. The
  /// confidence level is expressed as a percentage in the open interval `(0, 100)`. Returns
  /// `IncompatibleHypothesis` for an invalid level and `InsufficientObservations` when the model
  /// has no positive residual degrees of freedom; callers needing a normal-reference fallback
  /// must select that policy explicitly.
  pub fn linear_combination_with_inference(
    &self,
    weights: &[f64],
    offset: f64,
    confidence_level: f64,
  ) -> Result<LinearCombinationInferenceResult, StatsError> {
    if !confidence_level.is_finite() || confidence_level <= 0.0 || confidence_level >= 100.0 {
      return Err(StatsError::IncompatibleHypothesis(
        "confidence level must be between 0 and 100".into(),
      ));
    }
    if self.degrees_of_freedom == 0 {
      return Err(StatsError::InsufficientObservations(
        "linear combination inference requires positive residual degrees of freedom".into(),
      ));
    }

    let combination = self.linear_combination_with_offset(weights, offset)?;
    let (p_value, ci_lower, ci_upper) = if combination.standard_error == 0.0 {
      (f64::NAN, combination.estimate, combination.estimate)
    } else {
      let statistic = combination.estimate / combination.standard_error;
      let df = self.degrees_of_freedom as f64;
      let p_value = student_t_pvalue(statistic, df);
      let critical_value = student_t_quantile(0.5 + confidence_level / 200.0, df);
      (
        p_value,
        combination.estimate - critical_value * combination.standard_error,
        combination.estimate + critical_value * combination.standard_error,
      )
    };

    Ok(LinearCombinationInferenceResult {
      estimate: combination.estimate,
      standard_error: combination.standard_error,
      statistic: combination.statistic,
      p_value,
      ci_lower,
      ci_upper,
      confidence_level,
      degrees_of_freedom: self.degrees_of_freedom,
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
    let wald = self.test_linear_hypothesis_chi_square(r_matrix, r_vector)?;
    Ok(WaldTestResult {
      f_statistic: wald.statistic / wald.degrees_of_freedom as f64,
      chi2_statistic: wald.statistic,
      df_num: wald.degrees_of_freedom,
      df_denom: self.degrees_of_freedom,
    })
  }

  /// Test linear restrictions against their asymptotic chi-square reference.
  /// This form does not require or invent finite-sample residual degrees of freedom.
  pub fn test_linear_hypothesis_chi_square(
    &self,
    r_matrix: &[Vec<f64>],
    r_vector: &[f64],
  ) -> Result<WaldChiSquareTestResult, StatsError> {
    wald_chi_square_test(
      &self.parameters,
      &self.covariance.matrix,
      r_matrix,
      r_vector,
    )
  }
}
