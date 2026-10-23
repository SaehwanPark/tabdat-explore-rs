//! Foundational statistical contracts, estimation samples, and pure baseline inference.
//!
//! This crate provides backend-independent statistical problem definitions, explicit
//! sample provenance tracking, parameter estimates, covariance matrices, post-estimation
//! state, and numerical least squares fitting without any runtime backend (DuckDB, C FFI)
//! dependencies.

#![forbid(unsafe_code)]

mod binary;
mod bootstrap;
mod count_probabilities;
pub mod diagnostics;
mod distributions;
pub mod error;
pub mod estimates;
mod fixed_effects;
pub mod iv;
mod iv_confidence;
mod iv_endogenous;
mod iv_overid;
pub mod least_squares;
pub mod matrix;
mod nb2;
mod poisson;
pub mod post_estimation;
pub mod problem;
mod quantile;
mod quantile_bootstrap;
mod random_effects;
mod random_effects_ml;
pub mod result;
pub mod sample;
pub mod traits;
pub mod ttest;

pub use binary::{
  BinaryCluster, BinaryLogitResult, BinaryMleResult, BinaryModel, BinaryOmnibusWald, LogitEstimate,
  LogitMarginalEffects, LogitOptions, ProbitOptions, binary_average_marginal_effects,
  fit_binary_clustered, fit_binary_logit, fit_binary_logit_with_options, fit_binary_probit,
  fit_binary_probit_with_options, fit_binary_robust, logit_average_marginal_effects,
};
pub use bootstrap::{
  BOOTSTRAP_RNG, BootstrapCoefficient, BootstrapOlsResult, ClusterBootstrapOlsResult,
  MAX_BOOTSTRAP_REPLICATIONS, MAX_BOOTSTRAP_WORK, bootstrap_ols, bootstrap_ols_clustered,
};
pub use count_probabilities::{
  CountProbabilityBin, CountProbabilityLaw, CountProbabilityTable, MAX_COUNT_POINT_CUTOFF,
  MAX_COUNT_PROBABILITY_TAIL, count_bin_probabilities, count_point_probabilities,
  count_probability_table,
};
pub use diagnostics::variance_inflation_factors;
pub use distributions::{normal_cdf, normal_pdf, normal_quantile};
pub use error::StatsError;
pub use estimates::{
  CoefficientEstimate, CovarianceMatrix, CovarianceType, EstimationDiagnostics, FitStatistics,
};
pub use fixed_effects::{FixedEffect, FixedEffectsResult, UnitEffectsFTest, fit_fixed_effects};
pub use iv::{
  AndersonRubinResult, FirstStageDiagnostic, InstrumentalVariablesProblem, IvOptions, IvResult,
  RobustAndersonRubinResult, anderson_rubin_test, fit_two_stage_least_squares,
  robust_anderson_rubin_test,
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
pub use matrix::{
  covariance_matrix, f_distribution_survival_probability, invert, mean, multiply, multiply_vector,
  sample_covariance, sample_variance, scale, student_t_pvalue, student_t_quantile, transpose,
};
pub use nb2::{
  MAX_NB2_COUNT, Nb2Cluster, Nb2Dispersion, Nb2DispersionTest, Nb2Estimate, Nb2MarginalEffects,
  Nb2Options, Nb2Result, fit_nb2, fit_nb2_clustered, fit_nb2_with_offset, fit_nb2_with_options,
  nb2_average_marginal_effects,
};
pub use poisson::{
  MAX_POISSON_COUNT, PoissonCluster, PoissonEstimate, PoissonMarginalEffects, PoissonOmnibusWald,
  PoissonOptions, PoissonResult, fit_poisson, fit_poisson_clustered, fit_poisson_with_offset,
  fit_poisson_with_options, poisson_average_marginal_effects,
};
pub use post_estimation::{
  LinearCombinationInferenceResult, LinearCombinationResult, PostEstimationModel, WaldTestResult,
};
pub use problem::EstimationProblem;
pub use random_effects::{
  RandomEffectsResult, fit_random_effects, fit_random_effects_clustered,
  fit_random_effects_unbalanced, fit_random_effects_unbalanced_clustered,
};
pub use random_effects_ml::{
  RandomEffectsMlClusterInference, RandomEffectsMlInference, RandomEffectsMlInterior,
  RandomEffectsMlUnitVarianceTest, RandomEffectsMlVarianceComponentEstimate,
  RandomEffectsMlVarianceComponentInference, fit_random_effects_ml,
  fit_random_effects_ml_clustered, fit_random_effects_ml_interior,
};
pub use result::LeastSquaresResult;
pub use sample::EstimationSample;
pub use traits::Estimator;
pub use ttest::{
  TtestSampleStatistics, TtestStatistics, independent_ttest, one_sample_ttest, paired_ttest,
};

pub use quantile::{QuantileCoefficient, QuantileResult, fit_quantile_iid};
pub use quantile_bootstrap::{QuantileBootstrapResult, bootstrap_quantiles};
