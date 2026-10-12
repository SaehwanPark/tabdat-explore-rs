use crate::{Coefficient, GretlError};
use gretl_sys::{GretlDataset, GretlModel, Stat};
#[derive(Debug, Clone)]
pub struct HeckmanProblem {
  pub outcome: String,
  pub outcome_predictors: Vec<String>,
  pub selection: String,
  pub selection_predictors: Vec<String>,
  pub include_intercept: bool,
  /// Outcome, outcome predictors, binary selection, selection predictors.
  pub data: Vec<Vec<f64>>,
}
#[derive(Debug, Clone)]
pub struct HeckmanResult {
  pub outcome_coefficients: Vec<Coefficient>,
  pub selection_coefficients: Vec<Coefficient>,
  /// Proper two-step correction including first-stage uncertainty.
  pub outcome_covariance: Vec<Vec<f64>>,
  /// Observed-information probit covariance; no cross-equation claim.
  pub selection_covariance: Vec<Vec<f64>>,
  pub sigma: f64,
  pub rho: f64,
  pub selection_rows: Vec<usize>,
  pub outcome_rows: Vec<usize>,
}

pub fn run_heckman(problem: &HeckmanProblem) -> Result<HeckmanResult, GretlError> {
  let invalid = |m: &str| GretlError::InvalidProblem(m.into());
  let p = problem.outcome_predictors.len();
  let q = problem.selection_predictors.len();
  let n = problem.data.len();
  let m = p + 2;
  let l = q + 1;
  let total = m + l;
  if !total
    .checked_mul(total + 1)
    .is_some_and(|cells| i32::try_from(cells).is_ok())
  {
    return Err(invalid(
      "two-step packed covariance exceeds native integer arithmetic",
    ));
  }
  if !problem.include_intercept
    || q == 0
    || n <= l
    || i32::try_from(n).is_err()
    || i32::try_from(p + q + 6).is_err()
    || problem.data.iter().any(|r| r.len() != p + q + 2)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid(
      "two-step selection needs intercepts and finite complete rectangular sample",
    ));
  }
  if problem
    .data
    .iter()
    .any(|r| r[p + 1] != 0. && r[p + 1] != 1.)
  {
    return Err(invalid("selection dependent must be binary"));
  }
  let outcome_rows: Vec<_> = problem
    .data
    .iter()
    .enumerate()
    .filter(|(_, r)| r[p + 1] == 1.)
    .map(|(i, _)| i)
    .collect();
  if outcome_rows.len() <= m || outcome_rows.len() == n {
    return Err(invalid(
      "selection needs both classes and sufficient selected residual degrees of freedom",
    ));
  }
  if problem.outcome == problem.selection {
    return Err(invalid("equation outcomes must have distinct names"));
  }
  for names in [
    std::iter::once(&problem.outcome)
      .chain(&problem.outcome_predictors)
      .collect::<Vec<_>>(),
    std::iter::once(&problem.selection)
      .chain(&problem.selection_predictors)
      .collect(),
  ] {
    let mut seen = std::collections::HashSet::new();
    if names.iter().any(|name| {
      name.is_empty()
        || name.contains('\0')
        || ["intercept", "mills_lambda"].contains(&name.as_str())
        || !seen.insert(name.as_str())
    }) {
      return Err(invalid(
        "equation names must be distinct, nonempty, NUL-free and not reserved",
      ));
    }
  }
  // Identifiers refer to one column even when used in different equation roles.
  for (i, outname) in std::iter::once(&problem.outcome)
    .chain(&problem.outcome_predictors)
    .enumerate()
  {
    for (j, selname) in std::iter::once(&problem.selection)
      .chain(&problem.selection_predictors)
      .enumerate()
    {
      if outname == selname && problem.data.iter().any(|r| r[i] != r[p + 1 + j]) {
        return Err(invalid(
          "shared names require identical columns across equation roles",
        ));
      }
    }
  }
  let mut dset = GretlDataset::new(p + q + 3, n);
  for i in 1..p + q + 3 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    for (i, v) in row.iter().enumerate() {
      dset.set_value(i + 1, t, *v);
    }
  }
  let mut list = vec![0, 1, 0];
  list.extend((2..p + 2).map(|i| i as i32));
  list.extend([-100, (p + 2) as i32, 0]);
  list.extend((p + 3..p + q + 3).map(|i| i as i32));
  list[0] = (list.len() - 1) as i32;
  let model = GretlModel::heckman(&dset, &list);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.ncoeff() != m + l || model.nobs() != outcome_rows.len() || model.selection_nobs() != n {
    return Err(invalid(
      "selection changed full-rank equation design/sample",
    ));
  }
  let outcome_names: Vec<_> = std::iter::once("intercept")
    .chain(problem.outcome_predictors.iter().map(String::as_str))
    .chain(std::iter::once("mills_lambda"))
    .collect();
  let selection_names: Vec<_> = std::iter::once("intercept")
    .chain(problem.selection_predictors.iter().map(String::as_str))
    .collect();
  let outcome_coefficients: Vec<_> = (0..m)
    .map(|i| Coefficient {
      name: outcome_names[i].into(),
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let selection_coefficients: Vec<_> = (0..l)
    .map(|i| Coefficient {
      name: selection_names[i].into(),
      value: model.coeff(m + i),
      standard_error: model.sderr(m + i),
    })
    .collect();
  // Native cross-equation zeros are placeholders, not estimated independence.
  // Expose equation blocks separately, never a falsely labeled full joint VCV.
  let outcome_covariance: Vec<Vec<_>> = (0..m)
    .map(|i| (0..m).map(|j| model.vcv(i, j)).collect())
    .collect();
  let selection_covariance: Vec<Vec<_>> = (0..l)
    .map(|i| (0..l).map(|j| model.vcv(m + i, m + j)).collect())
    .collect();
  let sigma = model.scalar(Stat::Sigma);
  let rho = model.scalar(Stat::Rho);
  if !sigma.is_finite()
    || sigma <= 0.
    || !rho.is_finite()
    || rho.abs() >= 1.
    || outcome_coefficients
      .iter()
      .chain(&selection_coefficients)
      .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error <= 0.)
    || outcome_covariance
      .iter()
      .chain(&selection_covariance)
      .flatten()
      .any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite/correlation-boundary two-step output"));
  }
  Ok(HeckmanResult {
    outcome_coefficients,
    selection_coefficients,
    outcome_covariance,
    selection_covariance,
    sigma,
    rho,
    selection_rows: (0..n).collect(),
    outcome_rows,
  })
}
