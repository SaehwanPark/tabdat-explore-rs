#![forbid(unsafe_code)]

use std::f64::consts::PI;

use crate::error::StatsError;
use crate::estimates::{
  CoefficientEstimate, CovarianceMatrix, CovarianceType, EstimationDiagnostics, FitStatistics,
};
use crate::matrix::{
  multiply_vector, qr_decompose_with_response, scale, solve_upper_triangular, xtx_inverse_from_r,
};
use crate::problem::EstimationProblem;
use crate::result::LeastSquaresResult;
use crate::traits::Estimator;

/// Pure Rust baseline linear least squares estimator (OLS and WLS).
#[derive(Debug, Default, Clone, Copy)]
pub struct PureLeastSquaresEstimator;

impl Estimator<EstimationProblem, LeastSquaresResult> for PureLeastSquaresEstimator {
  fn fit(&self, problem: &EstimationProblem) -> Result<LeastSquaresResult, StatsError> {
    fit_least_squares(problem)
  }
}

/// Out-of-sample or in-sample linear prediction: y_hat = X * beta.
pub fn predict_linear_response(
  design_matrix: &[Vec<f64>],
  parameters: &[f64],
) -> Result<Vec<f64>, StatsError> {
  if design_matrix.is_empty() {
    return Err(StatsError::EmptySample);
  }
  let cols = design_matrix[0].len();
  if cols != parameters.len() {
    return Err(StatsError::DimensionMismatch(format!(
      "design_matrix column count ({cols}) must match parameter count ({})",
      parameters.len()
    )));
  }
  for (r, row) in design_matrix.iter().enumerate() {
    if row.len() != cols {
      return Err(StatsError::DimensionMismatch(format!(
        "row {r} length ({}) does not match expected columns ({cols})",
        row.len()
      )));
    }
  }
  multiply_vector(design_matrix, parameters)
}

/// Fit an estimation problem using ordinary or weighted least squares.
pub fn fit_least_squares(problem: &EstimationProblem) -> Result<LeastSquaresResult, StatsError> {
  problem.validate()?;

  let nobs = problem.observation_count();
  let n_params = problem.parameter_count();

  if nobs <= n_params {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: nobs,
      parameters: n_params,
    });
  }

  let df = nobs - n_params;

  // Build design matrix with optional intercept column at index 0
  let mut design_matrix = Vec::with_capacity(nobs);
  for row in &problem.design_matrix {
    let mut augmented_row = Vec::with_capacity(n_params);
    if problem.include_intercept {
      augmented_row.push(1.0);
    }
    augmented_row.extend(row.iter().copied());
    design_matrix.push(augmented_row);
  }

  let parameter_names = problem.parameter_names();
  let response = &problem.response;

  // Apply weights if present
  let weights: Vec<f64> = match &problem.weights {
    Some(w) => w.clone(),
    None => vec![1.0; nobs],
  };

  let mut weighted_design = Vec::with_capacity(nobs);
  let mut weighted_response = Vec::with_capacity(nobs);
  for (i, row) in design_matrix.iter().enumerate() {
    let sqrt_w = weights[i].sqrt();
    let w_row: Vec<f64> = row.iter().map(|&v| v * sqrt_w).collect();
    weighted_design.push(w_row);
    weighted_response.push(response[i] * sqrt_w);
  }

  // Householder QR decomposition: avoids condition-number squaring
  let qr = qr_decompose_with_response(&weighted_design, &weighted_response)?;
  let beta = solve_upper_triangular(&qr.r, &qr.qty)?;
  let xtx_inv = xtx_inverse_from_r(&qr.r)?;

  // Fitted values y_hat = X * beta (using original unweighted X)
  let fitted_values = multiply_vector(&design_matrix, &beta)?;

  // Residuals e = y - y_hat
  let mut residuals = Vec::with_capacity(nobs);
  let mut rss = 0.0;
  for i in 0..nobs {
    let res = response[i] - fitted_values[i];
    residuals.push(res);
    rss += weights[i] * res * res;
  }

  // Total sum of squares and R^2
  let (tss, ess) = if problem.include_intercept {
    let sum_w: f64 = weights.iter().sum();
    let y_bar: f64 = response
      .iter()
      .zip(weights.iter())
      .map(|(&y, &w)| w * y)
      .sum::<f64>()
      / sum_w;
    let tss_val: f64 = response
      .iter()
      .zip(weights.iter())
      .map(|(&y, &w)| w * (y - y_bar).powi(2))
      .sum();
    (tss_val, tss_val - rss)
  } else {
    let tss_val: f64 = response
      .iter()
      .zip(weights.iter())
      .map(|(&y, &w)| w * y * y)
      .sum();
    (tss_val, tss_val - rss)
  };

  let r_squared = if tss > 0.0 {
    Some((1.0 - rss / tss).clamp(0.0, 1.0))
  } else {
    None
  };

  let adjusted_r_squared = if tss > 0.0 && nobs > 1 {
    let n_f = nobs as f64;
    let p_f = n_params as f64;
    let adj = 1.0 - (rss / (n_f - p_f)) / (tss / (n_f - 1.0));
    Some(adj)
  } else {
    None
  };

  let sigma_sq = rss / (df as f64);
  let root_mse = sigma_sq.max(0.0).sqrt();

  // F-statistic for joint hypothesis of non-intercept predictors
  let f_stat = if problem.include_intercept && !problem.predictor_names.is_empty() {
    let k = problem.predictor_names.len() as f64;
    let denom = rss / (df as f64);
    if denom > 0.0 && ess >= 0.0 {
      Some((ess / k) / denom)
    } else {
      None
    }
  } else {
    None
  };

  // Gaussian log-likelihood
  let log_likelihood = if rss > 0.0 {
    let n_f = nobs as f64;
    let s2 = rss / n_f;
    Some(-0.5 * n_f * (1.0 + (2.0 * PI * s2).ln()))
  } else {
    None
  };

  // Parameter covariance matrix: V = sigma^2 * (X'X)^(-1)
  let cov_matrix = scale(&xtx_inv, sigma_sq);
  let covariance = CovarianceMatrix::new(
    parameter_names.clone(),
    CovarianceType::NonRobust,
    cov_matrix,
  )?;

  let standard_errors = covariance.standard_errors();
  let mut coefficients = Vec::with_capacity(n_params);
  for (idx, name) in parameter_names.iter().enumerate() {
    let b = beta[idx];
    let se = standard_errors[idx];
    coefficients.push(CoefficientEstimate::new(name, b, Some(se), None));
  }

  let diagnostics = EstimationDiagnostics {
    method: "least_squares".into(),
    converged: true,
    iterations: 1,
    objective_value: Some(rss),
    residual_sum_of_squares: Some(rss),
  };

  let fit_statistics = FitStatistics {
    observation_count: nobs,
    parameter_count: n_params,
    degrees_of_freedom: df,
    r_squared,
    adjusted_r_squared,
    root_mse: Some(root_mse),
    residual_sum_of_squares: Some(rss),
    total_sum_of_squares: Some(tss),
    f_statistic: f_stat,
    log_likelihood,
  };

  Ok(LeastSquaresResult {
    parameter_names,
    coefficients,
    fitted_values,
    residuals,
    covariance,
    diagnostics,
    fit_statistics,
    sample: problem.sample.clone(),
  })
}
