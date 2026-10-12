use crate::{Coefficient, EstimationProblem, GretlError};
use gretl_sys::{GretlDataset, GretlModel, Stat};

#[derive(Debug, Clone)]
pub struct TobitResult {
  pub coefficients: Vec<Coefficient>,
  /// Joint beta/log(sigma) observed-information covariance, scale last.
  pub covariance: Vec<Vec<f64>>,
  pub log_sigma: f64,
  pub log_likelihood: f64,
  pub sample_rows: Vec<usize>,
}

pub fn run_tobit(
  problem: &EstimationProblem,
  lower: f64,
  upper: Option<f64>,
) -> Result<TobitResult, GretlError> {
  let invalid = |m: &str| GretlError::InvalidProblem(m.into());
  let p = problem.predictors.len();
  let n = problem.data.len();
  let k = p + usize::from(problem.include_intercept);
  if !lower.is_finite()
    || upper.is_some_and(|u| !u.is_finite() || u <= lower)
    || k == 0
    || n <= k + 1
    || problem.data.iter().any(|r| r.len() != p + 1)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
    || i32::try_from(n).is_err()
    || i32::try_from(p + 3).is_err()
  {
    return Err(invalid(
      "Tobit requires finite rectangular complete data and ordered limits",
    ));
  }
  let mut seen = std::collections::HashSet::new();
  for name in std::iter::once(&problem.outcome).chain(&problem.predictors) {
    if name.is_empty() || name.contains('\0') || name == "intercept" || !seen.insert(name) {
      return Err(invalid(
        "Tobit names must be distinct, nonempty, NUL-free and not intercept",
      ));
    }
  }
  if problem
    .data
    .iter()
    .all(|r| r[0] <= lower || upper.is_some_and(|u| r[0] >= u))
  {
    return Err(invalid("Tobit needs at least one uncensored observation"));
  }
  let mut dset = GretlDataset::new(p + 2, n);
  for i in 0..p + 1 {
    dset.set_varname(i + 1, &format!("td_v{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    for (i, v) in row.iter().enumerate() {
      dset.set_value(i + 1, t, *v);
    }
  }
  let mut list = vec![0, 1];
  if problem.include_intercept {
    list.push(0);
  }
  list.extend((2..p + 2).map(|i| i as i32));
  list[0] = (list.len() - 1) as i32;
  let model = GretlModel::tobit(&dset, &list, lower, upper);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != n || model.ncoeff() != k {
    return Err(invalid("Tobit changed full-rank sample/design"));
  }
  let labels: Vec<_> = std::iter::once("intercept")
    .filter(|_| problem.include_intercept)
    .chain(problem.predictors.iter().map(String::as_str))
    .collect();
  let coefficients: Vec<_> = (0..k)
    .map(|i| Coefficient {
      name: labels[i].into(),
      value: model.coeff(i),
      standard_error: model.sderr(i),
    })
    .collect();
  let covariance: Vec<Vec<_>> = (0..k + 1)
    .map(|i| (0..k + 1).map(|j| model.tobit_vcv(i, j)).collect())
    .collect();
  let log_sigma = model.scalar(Stat::Sigma).ln();
  let log_likelihood = model.scalar(Stat::LogLikelihood);
  if !log_sigma.is_finite()
    || !log_likelihood.is_finite()
    || coefficients
      .iter()
      .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error <= 0.)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite Tobit result"));
  }
  Ok(TobitResult {
    coefficients,
    covariance,
    log_sigma,
    log_likelihood,
    sample_rows: (0..n).collect(),
  })
}
