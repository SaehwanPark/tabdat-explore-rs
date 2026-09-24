//! Foundational statistical contracts, estimation samples, and pure baseline inference.
//!
//! This crate provides backend-independent statistical problem definitions, explicit
//! sample provenance tracking, parameter estimates, covariance matrices, post-estimation
//! state, and numerical least squares fitting without any runtime backend (DuckDB, C FFI)
//! dependencies.

#![forbid(unsafe_code)]

pub mod error;
pub mod estimates;
pub mod least_squares;
pub mod matrix;
pub mod post_estimation;
pub mod problem;
pub mod result;
pub mod sample;
pub mod traits;

pub use error::StatsError;
pub use estimates::{
  CoefficientEstimate, CovarianceMatrix, CovarianceType, EstimationDiagnostics, FitStatistics,
};
pub use least_squares::{
  LeastSquaresOptions, PureLeastSquaresEstimator, fit_least_squares,
  fit_least_squares_with_options, predict_linear_response,
};
pub use matrix::{
  covariance_matrix, invert, mean, multiply, multiply_vector, sample_covariance, sample_variance,
  scale, transpose,
};
pub use post_estimation::{LinearCombinationResult, PostEstimationModel, WaldTestResult};
pub use problem::EstimationProblem;
pub use result::LeastSquaresResult;
pub use sample::EstimationSample;
pub use traits::Estimator;
