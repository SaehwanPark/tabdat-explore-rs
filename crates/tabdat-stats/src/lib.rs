//! Foundational statistical contracts, estimation samples, and pure baseline inference.
//!
//! This crate provides backend-independent statistical problem definitions, explicit
//! sample provenance tracking, parameter estimates, covariance matrices, post-estimation
//! state, and numerical least squares fitting without any runtime backend (DuckDB, C FFI)
//! dependencies.

#![forbid(unsafe_code)]

pub mod diagnostics;
mod distributions;
pub mod error;
pub mod estimates;
pub mod iv;
mod iv_confidence;
mod iv_endogenous;
mod iv_overid;
pub mod least_squares;
mod logit;
pub mod matrix;
pub mod post_estimation;
pub mod problem;
pub mod result;
pub mod sample;
pub mod traits;
pub mod ttest;

pub use diagnostics::variance_inflation_factors;
pub use error::StatsError;
pub use estimates::{
  CoefficientEstimate, CovarianceMatrix, CovarianceType, EstimationDiagnostics, FitStatistics,
};
pub use iv::{
  AndersonRubinResult, FirstStageDiagnostic, InstrumentalVariablesProblem, IvOptions, IvResult,
  anderson_rubin_test, fit_two_stage_least_squares,
};
pub use iv_confidence::{
  AndersonRubinConfidenceSet, ArConfidenceInterval, ArConfidenceSetShape,
  anderson_rubin_confidence_set,
};
pub use iv_endogenous::{WuHausmanResult, wu_hausman_test};
pub use iv_overid::{SarganResult, sargan_test};
pub use least_squares::{
  LeastSquaresOptions, PureLeastSquaresEstimator, fit_least_squares,
  fit_least_squares_with_options, predict_linear_response,
};
pub use logit::{
  BinaryLogitResult, LogitEstimate, LogitMarginalEffects, LogitOptions, fit_binary_logit,
  fit_binary_logit_with_options, logit_average_marginal_effects,
};
pub use matrix::{
  covariance_matrix, f_distribution_survival_probability, invert, mean, multiply, multiply_vector,
  sample_covariance, sample_variance, scale, student_t_pvalue, student_t_quantile, transpose,
};
pub use post_estimation::{
  LinearCombinationInferenceResult, LinearCombinationResult, PostEstimationModel, WaldTestResult,
};
pub use problem::EstimationProblem;
pub use result::LeastSquaresResult;
pub use sample::EstimationSample;
pub use traits::Estimator;
pub use ttest::{
  TtestSampleStatistics, TtestStatistics, independent_ttest, one_sample_ttest, paired_ttest,
};
