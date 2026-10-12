use crate::{Coefficient, GretlError};
use gretl_sys::GretlModel;

#[derive(Debug, Clone)]
pub struct IvProblem {
  pub outcome: String,
  pub exogenous: Vec<String>,
  pub endogenous: String,
  pub instruments: Vec<String>,
  pub include_intercept: bool,
  /// outcome, exogenous columns, endogenous column, excluded instruments.
  pub data: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct IvResult {
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub sample_rows: Vec<usize>,
  pub r_squared: f64,
}

pub fn run_iv(problem: &IvProblem) -> Result<IvResult, GretlError> {
  let invalid = |m: &str| GretlError::InvalidProblem(m.into());
  let nobs = problem.data.len();
  let k = problem.exogenous.len() + 1 + usize::from(problem.include_intercept);
  crate::check_parameter_count(k)?;
  crate::check_parameter_count(
    problem.instruments.len() + problem.exogenous.len() + usize::from(problem.include_intercept),
  )?;
  let ncol = 2 + problem.exogenous.len() + problem.instruments.len();
  if nobs <= k
    || problem.instruments.is_empty()
    || problem.data.iter().any(|r| r.len() != ncol)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid(
      "IV requires a rectangular finite complete identified sample",
    ));
  }
  let names: Vec<_> = std::iter::once(&problem.outcome)
    .chain(problem.exogenous.iter())
    .chain(std::iter::once(&problem.endogenous))
    .chain(problem.instruments.iter())
    .collect();
  let mut seen = std::collections::HashSet::new();
  if names
    .iter()
    .any(|name| name.contains('\0') || !seen.insert(name.as_str()))
  {
    return Err(invalid("IV variable names must be distinct and NUL-free"));
  }
  let encoded_len = problem
    .exogenous
    .len()
    .checked_mul(2)
    .and_then(|count| count.checked_add(problem.instruments.len()))
    .and_then(|count| count.checked_add(4 + 2 * usize::from(problem.include_intercept)))
    .ok_or_else(|| invalid("IV list length overflow"))?;
  if i32::try_from(encoded_len).is_err() {
    return Err(invalid("IV list exceeds native limits"));
  }
  let nvar = ncol
    .checked_add(1)
    .ok_or_else(|| invalid("too many variables"))?;
  if i32::try_from(nvar).is_err() || i32::try_from(nobs).is_err() {
    return Err(invalid("IV dimensions exceed native limits"));
  }
  let mut dset = crate::new_dataset(nvar, nobs)?;
  for (i, _) in names.iter().enumerate() {
    dset.set_varname(i + 1, &format!("td_v{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    for (i, v) in row.iter().enumerate() {
      dset.set_value(i + 1, t, *v);
    }
  }
  let endogenous = 2 + problem.exogenous.len();
  let mut list = vec![0, 1];
  if problem.include_intercept {
    list.push(0);
  }
  list.extend((2..=endogenous).map(|i| i as i32));
  list.push(-100);
  if problem.include_intercept {
    list.push(0);
  }
  list.extend((2..endogenous).map(|i| i as i32));
  list.extend((endogenous + 1..nvar).map(|i| i as i32));
  list[0] = (list.len() - 1) as i32;
  let model = GretlModel::iv(&dset, &list);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != nobs || model.ncoeff() != k {
    return Err(invalid("IV requires unchanged full-rank sample/design"));
  }
  let labels: Vec<_> = if problem.include_intercept {
    std::iter::once("intercept")
      .chain(problem.exogenous.iter().map(String::as_str))
      .chain(std::iter::once(problem.endogenous.as_str()))
      .collect()
  } else {
    problem
      .exogenous
      .iter()
      .map(String::as_str)
      .chain(std::iter::once(problem.endogenous.as_str()))
      .collect()
  };
  let coefficients: Vec<_> = (0..k)
    .map(|i| Coefficient {
      name: labels[i].into(),
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let covariance: Vec<Vec<f64>> = (0..k)
    .map(|i| (0..k).map(|j| model.vcv(i, j)).collect())
    .collect();
  // Gretl reports a correlation-based IV R². TabDat/linearmodels use original
  // structural residual SSE, not fitted-X or correlation-squared diagnostics.
  let offset = usize::from(problem.include_intercept);
  let mean = if problem.include_intercept {
    problem.data.iter().map(|row| row[0]).sum::<f64>() / nobs as f64
  } else {
    0.0
  };
  let mut ess = 0.0;
  let mut tss = 0.0;
  for row in &problem.data {
    let prediction = if problem.include_intercept {
      coefficients[0].value
    } else {
      0.0
    } + row[1..=problem.exogenous.len() + 1]
      .iter()
      .zip(&coefficients[offset..])
      .map(|(value, coefficient)| value * coefficient.value)
      .sum::<f64>();
    ess += (row[0] - prediction).powi(2);
    tss += (row[0] - mean).powi(2);
  }
  let r_squared = 1.0 - ess / tss;
  if coefficients
    .iter()
    .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error < 0.0)
    || covariance.iter().flatten().any(|v| !v.is_finite())
    || !r_squared.is_finite()
  {
    return Err(invalid("nonfinite IV output"));
  }
  Ok(IvResult {
    coefficients,
    covariance,
    sample_rows: (0..nobs).collect(),
    r_squared,
  })
}
