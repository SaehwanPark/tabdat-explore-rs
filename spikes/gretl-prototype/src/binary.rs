use crate::{Coefficient, EstimationProblem, GretlError};
use gretl_sys::{GretlDataset, GretlModel, Stat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryLink {
  Logit,
  Probit,
}

#[derive(Debug, Clone)]
pub struct BinaryResult {
  pub link: BinaryLink,
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub sample_rows: Vec<usize>,
  pub log_likelihood: f64,
}

/// Classical MLE only. This finite/complete-data feasibility path does not
/// construct a product model state or promise missing-row/separation parity.
pub fn run_binary(
  problem: &EstimationProblem,
  link: BinaryLink,
) -> Result<BinaryResult, GretlError> {
  let nobs = problem.nobs();
  let ncol = problem.ncol();
  if nobs == 0 || problem.predictors.is_empty() || problem.data.iter().any(|r| r.len() != ncol) {
    return Err(GretlError::InvalidProblem(
      "empty or ragged binary problem".into(),
    ));
  }
  if problem.data.iter().flatten().any(|v| !v.is_finite())
    || problem.data.iter().any(|r| r[0] != 0.0 && r[0] != 1.0)
  {
    return Err(GretlError::InvalidProblem(
      "binary spike requires finite data and 0/1 outcomes".into(),
    ));
  }
  if problem.outcome.contains('\0') || problem.predictors.iter().any(|n| n.contains('\0')) {
    return Err(GretlError::InvalidProblem(
      "variable name contains NUL".into(),
    ));
  }
  crate::check_parameter_count(problem.predictors.len() + usize::from(problem.include_intercept))?;
  let nvar = ncol
    .checked_add(1)
    .ok_or_else(|| GretlError::InvalidProblem("too many variables".into()))?;
  if i32::try_from(nvar).is_err() || i32::try_from(nobs).is_err() {
    return Err(GretlError::InvalidProblem(
      "dimensions exceed native limits".into(),
    ));
  }
  let mut dataset = GretlDataset::new(nvar, nobs);
  dataset.set_varname(1, "td_y");
  for i in 0..problem.predictors.len() {
    dataset.set_varname(i + 2, &format!("td_x{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    for (i, value) in row.iter().enumerate() {
      dataset.set_value(i + 1, t, *value);
    }
  }
  let mut list = vec![0, 1];
  if problem.include_intercept {
    list.push(0);
  }
  list.extend((2..nvar).map(|i| i as i32));
  list[0] = (list.len() - 1) as i32;
  let native_link = match link {
    BinaryLink::Logit => gretl_sys::BinaryLink::Logit,
    BinaryLink::Probit => gretl_sys::BinaryLink::Probit,
  };
  let model = GretlModel::binary(&dataset, &list, native_link);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  let k = problem.predictors.len() + usize::from(problem.include_intercept);
  if model.nobs() != nobs || model.ncoeff() != k || nobs <= k {
    return Err(GretlError::InvalidProblem(
      "binary spike requires a full-rank complete sample".into(),
    ));
  }
  let coefficients: Vec<_> = (0..k)
    .map(|i| Coefficient {
      name: if problem.include_intercept && i == 0 {
        "intercept".into()
      } else {
        problem.predictors[i - usize::from(problem.include_intercept)].clone()
      },
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let covariance: Vec<Vec<f64>> = (0..k)
    .map(|i| (0..k).map(|j| model.vcv(i, j)).collect())
    .collect();
  let log_likelihood = model.scalar(Stat::LogLikelihood);
  if coefficients
    .iter()
    .any(|c| !c.value.is_finite() || !c.standard_error.is_finite())
    || covariance.iter().flatten().any(|v| !v.is_finite())
    || !log_likelihood.is_finite()
  {
    return Err(GretlError::InvalidProblem(
      "binary native output is non-finite".into(),
    ));
  }
  Ok(BinaryResult {
    link,
    coefficients,
    covariance,
    sample_rows: (0..nobs).collect(),
    log_likelihood,
  })
}
