//! Safe facade over libgretl OLS for the TabDat estimator-backend spike.
//!
//! This crate is `forbid(unsafe_code)`: it defines Rust-owned
//! [`EstimationProblem`] / [`EstimationResult`] types and orchestrates an OLS
//! fit by driving the safe RAII wrappers in [`gretl_sys`]. All `unsafe` and
//! all knowledge of libgretl's C layout stays inside the `gretl-sys` crate and
//! its C shim; no raw pointer or foreign struct can appear here — the compiler
//! enforces that.
//!
//! This is a feasibility spike, not product support. It proves that a
//! Rust-owned estimator boundary can call libgretl OLS and produce
//! coefficients, standard errors, and fit statistics that match both the
//! NIST certified reference and the Python TabDat oracle.

#![forbid(unsafe_code)]

use gretl_sys::{GretlDataset, GretlModel, OlsCovariance, Stat};
use std::os::raw::c_int;

#[cfg(feature = "tobit-plugin")]
mod tobit;
#[cfg(feature = "tobit-plugin")]
pub use tobit::{TobitResult, run_tobit};
#[cfg(feature = "quantile-plugin")]
mod quantile;
#[cfg(feature = "quantile-plugin")]
pub use quantile::{QuantileResult, run_quantile};
#[cfg(feature = "survival-plugin")]
mod survival;
#[cfg(feature = "survival-plugin")]
pub use survival::{SurvivalDistribution, SurvivalProblem, SurvivalResult, run_survival};
#[cfg(feature = "heckman-plugin")]
mod heckman;
#[cfg(feature = "heckman-plugin")]
pub use heckman::{HeckmanProblem, HeckmanResult, run_heckman};
#[cfg(feature = "dpanel-plugin")]
mod dynamic_panel;
#[cfg(feature = "dpanel-plugin")]
pub use dynamic_panel::{DynamicPanelResult, run_dynamic_panel};
mod binary;
pub use binary::{BinaryLink, BinaryResult, run_binary};
mod panel;
pub use panel::{PanelEstimator, PanelProblem, PanelResult, run_panel};
mod iv;
pub use iv::{IvProblem, IvResult, run_iv};
#[cfg(feature = "count-plugin")]
mod poisson;
#[cfg(feature = "count-plugin")]
pub use poisson::{
  CountFamily, CountResult, NegativeBinomialResult, PoissonResult, run_negative_binomial,
  run_poisson,
};

/// A Rust-owned estimation problem: an outcome, predictors, and the data.
///
/// `data` is a row-major matrix where column 0 is the outcome and columns
/// 1..=k are the predictors, in the same order as [`Self::predictors`].
#[derive(Debug, Clone)]
pub struct EstimationProblem {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub data: Vec<Vec<f64>>,
}

impl EstimationProblem {
  /// Number of observations (rows).
  pub fn nobs(&self) -> usize {
    self.data.len()
  }

  /// Number of columns (1 outcome + predictors).
  pub fn ncol(&self) -> usize {
    self.predictors.len() + 1
  }
}

/// One coefficient estimate with its standard error.
#[derive(Debug, Clone, PartialEq)]
pub struct Coefficient {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CovarianceMode {
  Classical,
  Hc1,
  Clustered,
}

/// A Rust-owned OLS result. All values are copied out of the native model.
#[derive(Debug, Clone)]
pub struct EstimationResult {
  pub nobs: usize,
  pub ncoeff: usize,
  pub dfn: usize,
  pub dfd: usize,
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub covariance_mode: CovarianceMode,
  /// Zero-based input rows retained by this finite-data-only spike.
  pub sample_rows: Vec<usize>,
  /// TabDat inference convention: G-1 for clustered covariance, n-k otherwise.
  pub inference_df: usize,
  pub r_squared: f64,
  pub adj_r_squared: f64,
  pub sigma: f64,
  pub ess: f64,
  pub tss: f64,
  pub f_statistic: f64,
  pub log_likelihood: f64,
}

/// An error from the libgretl OLS boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GretlError {
  /// The problem is malformed (e.g. ragged data, empty predictors).
  InvalidProblem(String),
  /// libgretl reported an estimation failure with the given error code.
  EstimationFailed { code: i32 },
}

impl std::fmt::Display for GretlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GretlError::InvalidProblem(msg) => write!(f, "invalid problem: {msg}"),
      GretlError::EstimationFailed { code } => {
        write!(f, "libgretl estimation failed with error code {code}")
      }
    }
  }
}

impl std::error::Error for GretlError {}

/// Run ordinary least squares on `problem` via libgretl.
///
/// The outcome is `problem.data[*][0]`; predictors are columns 1..=k. The
/// intercept is included when `problem.include_intercept` is true.
pub fn run_ols(problem: &EstimationProblem) -> Result<EstimationResult, GretlError> {
  fit_ols(problem, CovarianceMode::Classical, None)
}

pub fn run_ols_hc1(problem: &EstimationProblem) -> Result<EstimationResult, GretlError> {
  fit_ols(problem, CovarianceMode::Hc1, None)
}

pub fn run_ols_clustered(
  problem: &EstimationProblem,
  groups: &[i32],
) -> Result<EstimationResult, GretlError> {
  fit_ols(problem, CovarianceMode::Clustered, Some(groups))
}

fn fit_ols(
  problem: &EstimationProblem,
  mode: CovarianceMode,
  groups: Option<&[i32]>,
) -> Result<EstimationResult, GretlError> {
  let nobs = problem.nobs();
  let ncol = problem.ncol();
  if nobs == 0 {
    return Err(GretlError::InvalidProblem("no observations".into()));
  }
  if problem.predictors.is_empty() {
    return Err(GretlError::InvalidProblem("no predictors".into()));
  }
  for row in &problem.data {
    if row.len() != ncol {
      return Err(GretlError::InvalidProblem(format!(
        "ragged data: expected {ncol} columns, found {}",
        row.len()
      )));
    }
  }

  if problem.data.iter().flatten().any(|v| !v.is_finite()) {
    return Err(GretlError::InvalidProblem(
      "spike requires finite complete data".into(),
    ));
  }
  let clusters = if let Some(groups) = groups {
    if groups.len() != nobs {
      return Err(GretlError::InvalidProblem(
        "cluster count must match observations".into(),
      ));
    }
    let count = groups
      .iter()
      .collect::<std::collections::BTreeSet<_>>()
      .len();
    if count < 2 {
      return Err(GretlError::InvalidProblem(
        "need at least two clusters".into(),
      ));
    }
    count
  } else {
    0
  };
  if problem.outcome.contains('\0') || problem.predictors.iter().any(|name| name.contains('\0')) {
    return Err(GretlError::InvalidProblem(
      "variable name contains NUL".into(),
    ));
  }
  // Gretl always reserves variable zero, even when omitted from the model list.
  let has_const = problem.include_intercept;
  let n_predictors = problem.predictors.len();
  let nvar = ncol
    .checked_add(1 + usize::from(groups.is_some()))
    .ok_or_else(|| GretlError::InvalidProblem("too many variables".into()))?;
  if c_int::try_from(nvar).is_err() || c_int::try_from(nobs).is_err() {
    return Err(GretlError::InvalidProblem(
      "dimensions exceed native limits".into(),
    ));
  }

  let mut dset = GretlDataset::new(nvar, nobs);

  // Assign variable indices.
  let const_idx = 0usize;
  let outcome_idx = 1;
  let pred_base = outcome_idx + 1;

  if has_const {
    dset.set_varname(const_idx, "const");
  }
  // Internal native names avoid truncation/collision affecting cluster lookup.
  // The owned results keep the original Rust coefficient names.
  dset.set_varname(outcome_idx, "td_y");
  for i in 0..n_predictors {
    dset.set_varname(pred_base + i, &format!("td_x{i}"));
  }
  let cluster_idx = nvar - 1;
  if let Some(groups) = groups {
    dset.set_varname(cluster_idx, "td_cluster");
    for (t, group) in groups.iter().enumerate() {
      dset.set_value(cluster_idx, t, f64::from(*group));
    }
  }

  // Fill the data. Column 0 of the problem is the outcome; columns 1..=k are
  // predictors.
  for (t, row) in problem.data.iter().enumerate() {
    dset.set_value(outcome_idx, t, row[0]);
    for i in 0..n_predictors {
      dset.set_value(pred_base + i, t, row[1 + i]);
    }
  }

  // Build the gretl variable list: [count, outcome, 0(const), preds...].
  let mut list: Vec<c_int> = Vec::with_capacity(1 + 1 + usize::from(has_const) + n_predictors);
  let mut entries: Vec<c_int> = Vec::new();
  entries.push(outcome_idx as c_int);
  if has_const {
    entries.push(const_idx as c_int);
  }
  for i in 0..n_predictors {
    entries.push((pred_base + i) as c_int);
  }
  list.push(entries.len() as c_int);
  list.extend_from_slice(&entries);

  let native_covariance = match mode {
    CovarianceMode::Classical => OlsCovariance::Classical,
    CovarianceMode::Hc1 => OlsCovariance::Hc1,
    CovarianceMode::Clustered => OlsCovariance::Clustered {
      variable: cluster_idx,
    },
  };
  let model = GretlModel::ols_with_covariance(&dset, &list, native_covariance);

  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }

  let ncoeff = model.ncoeff();
  if ncoeff != n_predictors + usize::from(has_const) || model.nobs() != nobs || nobs <= ncoeff {
    return Err(GretlError::InvalidProblem(
      "spike requires full-rank complete sample with positive residual degrees of freedom".into(),
    ));
  }
  let covariance: Vec<Vec<f64>> = (0..ncoeff)
    .map(|i| (0..ncoeff).map(|j| model.vcv(i, j)).collect())
    .collect();
  let mut coefficients = Vec::with_capacity(ncoeff);
  for i in 0..ncoeff {
    let name = coeff_name(i, has_const, &problem.outcome, &problem.predictors);
    coefficients.push(Coefficient {
      name,
      value: model.coeff(i),
      standard_error: model.sderr(i),
    });
  }

  if covariance.iter().flatten().any(|v| !v.is_finite())
    || coefficients
      .iter()
      .any(|c| !c.value.is_finite() || !c.standard_error.is_finite())
  {
    return Err(GretlError::InvalidProblem(
      "native coefficient/covariance output is non-finite".into(),
    ));
  }

  Ok(EstimationResult {
    nobs: model.nobs(),
    ncoeff,
    dfn: model.dfn(),
    dfd: model.dfd(),
    coefficients,
    covariance,
    covariance_mode: mode,
    sample_rows: (0..nobs).collect(),
    inference_df: if mode == CovarianceMode::Clustered {
      clusters - 1
    } else {
      nobs - ncoeff
    },
    r_squared: model.scalar(Stat::RSquared),
    adj_r_squared: model.scalar(Stat::AdjRSquared),
    sigma: model.scalar(Stat::Sigma),
    ess: model.scalar(Stat::Ess),
    tss: model.scalar(Stat::Tss),
    f_statistic: model.scalar(Stat::FStatistic),
    log_likelihood: model.scalar(Stat::LogLikelihood),
  })
}

/// Map a gretl coefficient index to a human-readable name. gretl orders
/// coefficients as [outcome? no] — actually as [const?, then predictors in
/// list order]. The list order is [outcome, const?, preds...], but gretl's
/// coefficient vector excludes the outcome and is ordered [const?, preds...].
fn coeff_name(i: usize, has_const: bool, _outcome: &str, predictors: &[String]) -> String {
  if has_const {
    if i == 0 {
      "intercept".to_string()
    } else {
      predictors
        .get(i - 1)
        .cloned()
        .unwrap_or_else(|| format!("x{i}"))
    }
  } else if let Some(p) = predictors.get(i) {
    p.clone()
  } else {
    format!("x{i}")
  }
}
