use crate::{Coefficient, GretlError, PanelProblem};
use gretl_sys::{GretlDataset, GretlModel};
#[derive(Debug, Clone)]
pub struct DynamicPanelResult {
  pub coefficients: Vec<Coefficient>,
  /// Entity-summed one-step sandwich, no finite-N correction.
  pub covariance: Vec<Vec<f64>>,
  pub sample_rows: Vec<usize>,
  pub keys: Vec<(usize, usize)>,
  pub groups: usize,
  pub instruments: usize,
}
/// One-step difference GMM: L1.y, collapsed y(t-2), differenced exogenous x.
pub fn run_dynamic_panel(problem: &PanelProblem) -> Result<DynamicPanelResult, GretlError> {
  let invalid = |m: &str| GretlError::InvalidProblem(m.into());
  let n = problem.data.len();
  let p = problem.predictors.len();
  let t = problem.periods;
  if !(4..=32).contains(&t)
    || !n.is_multiple_of(t)
    || !(8..=4096).contains(&(n / t))
    || !(1..=16).contains(&p)
    || problem.keys.len() != n
    || problem
      .keys
      .iter()
      .enumerate()
      .any(|(i, k)| *k != (i / t, i % t))
    || problem.data.iter().any(|r| r.len() != p + 1)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid(
      "dynamic panel requires bounded canonical complete balanced sample",
    ));
  }
  let lag_name = format!("L1.{}", problem.outcome);
  let mut names = std::collections::HashSet::new();
  for name in std::iter::once(&problem.outcome).chain(&problem.predictors) {
    if name.is_empty()
      || name.contains('\0')
      || name == "intercept"
      || name == &lag_name
      || !names.insert(name)
    {
      return Err(invalid("invalid/duplicate/reserved dynamic column name"));
    }
  }
  let mut dset = GretlDataset::new(p + 2, n);
  for i in 1..p + 2 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (row, values) in problem.data.iter().enumerate() {
    for (i, v) in values.iter().enumerate() {
      dset.set_value(i + 1, row, *v);
    }
  }
  let mut list = vec![(2 * p + 4) as i32, 1, -100, 1];
  list.extend((2..p + 2).map(|i| i as i32));
  // An explicit GMM spec disables native implicit x instruments.
  list.push(-100);
  list.extend((2..p + 2).map(|i| i as i32));
  let model = GretlModel::dynamic_panel(&dset, &list, t);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  let expected: Vec<_> = (0..n).filter(|i| i % t >= 2).collect();
  let actual: Vec<_> = model
    .sample_mask(n)
    .into_iter()
    .enumerate()
    .filter(|(_, keep)| *keep)
    .map(|(i, _)| i)
    .collect();
  let instruments = model.dynamic_panel_instruments();
  if model.ncoeff() != p + 1
    || model.nobs() != expected.len()
    || actual != expected
    || instruments != p + 1
  {
    return Err(invalid("dynamic panel changed design/instruments/sample"));
  }
  let labels: Vec<_> = std::iter::once(lag_name.as_str())
    .chain(problem.predictors.iter().map(String::as_str))
    .collect();
  let coefficients: Vec<_> = (0..p + 1)
    .map(|i| Coefficient {
      name: labels[i].into(),
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let covariance: Vec<Vec<_>> = (0..p + 1)
    .map(|i| (0..p + 1).map(|j| model.vcv(i, j)).collect())
    .collect();
  if coefficients
    .iter()
    .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error <= 0.)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite dynamic coefficient/covariance"));
  }
  Ok(DynamicPanelResult {
    coefficients,
    covariance,
    keys: actual.iter().map(|&i| problem.keys[i]).collect(),
    sample_rows: actual,
    groups: n / t,
    instruments,
  })
}
