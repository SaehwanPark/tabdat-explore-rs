use crate::{Coefficient, GretlError};
use gretl_sys::{GretlDataset, GretlModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelEstimator {
  FixedEffects,
  RandomEffects,
}

#[derive(Debug, Clone)]
pub struct PanelProblem {
  pub outcome: String,
  pub predictors: Vec<String>,
  /// Canonical zero-based entity/time keys, entity-major and complete balanced.
  pub keys: Vec<(usize, usize)>,
  pub periods: usize,
  /// Outcome followed by predictors; no constant column (RE adds an intercept).
  pub data: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct PanelResult {
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub sample_rows: Vec<usize>,
  pub keys: Vec<(usize, usize)>,
}

pub fn run_panel(
  problem: &PanelProblem,
  estimator: PanelEstimator,
) -> Result<PanelResult, GretlError> {
  let invalid = |message: &str| GretlError::InvalidProblem(message.into());
  let n = problem.data.len();
  let p = problem.predictors.len();
  let t = problem.periods;
  if t < 2
    || !n.is_multiple_of(t)
    || n / t <= p + 1
    || p == 0
    || problem.keys.len() != n
    || problem
      .keys
      .iter()
      .enumerate()
      .any(|(i, key)| *key != (i / t, i % t))
    || n <= n / t + p
    || problem.data.iter().any(|r| r.len() != p + 1)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
    || i32::try_from(n).is_err()
    || i32::try_from(p + 3).is_err()
  {
    return Err(invalid(
      "panel requires canonical balanced finite full-rank sample",
    ));
  }
  let mut seen = std::collections::HashSet::new();
  for name in std::iter::once(&problem.outcome).chain(&problem.predictors) {
    if name.is_empty() || name.contains('\0') || name == "intercept" || !seen.insert(name) {
      return Err(invalid(
        "panel names must be distinct, nonempty, NUL-free and not intercept",
      ));
    }
  }
  let mut dset = GretlDataset::new(p + 2, n);
  for i in 0..p + 1 {
    dset.set_varname(i + 1, &format!("td_v{i}"));
  }
  for (row, values) in problem.data.iter().enumerate() {
    for (col, value) in values.iter().enumerate() {
      dset.set_value(col + 1, row, *value);
    }
  }
  let mut list = vec![(p + 2) as i32, 1, 0];
  list.extend((2..p + 2).map(|i| i as i32));
  let model = GretlModel::panel(&dset, &list, t, estimator == PanelEstimator::RandomEffects);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != n || model.ncoeff() != p + 1 {
    return Err(invalid("changed panel sample/design"));
  }
  // FE's average intercept depends on normalization and is not exposed by the
  // Python within-slope contract. RE instead has an identified explicit intercept.
  let start = usize::from(estimator == PanelEstimator::FixedEffects);
  let labels: Vec<_> = std::iter::once("intercept")
    .chain(problem.predictors.iter().map(String::as_str))
    .collect();
  let coefficients: Vec<_> = (start..p + 1)
    .map(|i| Coefficient {
      name: labels[i].into(),
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let covariance: Vec<Vec<_>> = (start..p + 1)
    .map(|i| (start..p + 1).map(|j| model.vcv(i, j)).collect())
    .collect();
  if coefficients
    .iter()
    .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error < 0.)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite panel output"));
  }
  Ok(PanelResult {
    coefficients,
    covariance,
    sample_rows: (0..n).collect(),
    keys: problem.keys.clone(),
  })
}
