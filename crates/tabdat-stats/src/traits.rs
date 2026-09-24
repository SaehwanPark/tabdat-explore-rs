#![forbid(unsafe_code)]

use crate::error::StatsError;

/// Capability trait for statistical estimators.
///
/// Decouples model specification problems from specific numerical routes or backends.
pub trait Estimator<Problem, Result> {
  /// Fit the given problem specification and produce an owned result or typed error.
  fn fit(&self, problem: &Problem) -> std::result::Result<Result, StatsError>;
}
