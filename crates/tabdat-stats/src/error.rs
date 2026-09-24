#![forbid(unsafe_code)]

use std::fmt;

/// Normalized errors produced across statistical estimation and inference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatsError {
  /// The estimation sample has zero observations.
  EmptySample,
  /// A dimension mismatch occurred between variables, rows, or matrix operands.
  DimensionMismatch(String),
  /// Matrix inversion failed because the matrix is singular or near-singular.
  SingularMatrix(String),
  /// Saturated model: degrees of freedom <= 0.
  ZeroDegreesOfFreedom {
    observations: usize,
    parameters: usize,
  },
  /// Weights are non-positive or invalid.
  NonPositiveWeights,
  /// A parameter or variable name is invalid or empty.
  InvalidParameterName(String),
  /// A numerical input contains a non-finite value (NaN or infinity).
  NonFiniteValue(String),
  /// Predictors are perfectly collinear.
  CollinearPredictors(String),
  /// The requested parameter name was not found in the estimated model.
  UnknownParameter(String),
  /// The hypothesis restriction matrix or vector is incompatible with model parameters.
  IncompatibleHypothesis(String),
}

impl fmt::Display for StatsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::EmptySample => write!(f, "estimation requires at least one observation"),
      Self::DimensionMismatch(msg) => write!(f, "dimension mismatch: {msg}"),
      Self::SingularMatrix(msg) => write!(f, "matrix is singular and cannot be inverted: {msg}"),
      Self::ZeroDegreesOfFreedom {
        observations,
        parameters,
      } => write!(
        f,
        "least squares requires positive residual degrees of freedom (observations: {observations}, parameters: {parameters})"
      ),
      Self::NonPositiveWeights => write!(f, "weights must be positive"),
      Self::InvalidParameterName(msg) => write!(f, "invalid parameter name: {msg}"),
      Self::NonFiniteValue(msg) => write!(f, "non-finite value encountered: {msg}"),
      Self::CollinearPredictors(msg) => write!(f, "collinear predictors: {msg}"),
      Self::UnknownParameter(name) => write!(f, "unknown parameter: {name}"),
      Self::IncompatibleHypothesis(msg) => write!(f, "incompatible hypothesis test: {msg}"),
    }
  }
}

impl std::error::Error for StatsError {}
