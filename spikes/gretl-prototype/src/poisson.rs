use crate::{Coefficient, EstimationProblem, GretlError};
use gretl_sys::{GretlModel, Stat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountFamily {
  Poisson,
  NegativeBinomial2,
}

#[derive(Debug, Clone)]
pub struct CountResult {
  pub family: CountFamily,
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub sample_rows: Vec<usize>,
  pub log_likelihood: f64,
}

pub type PoissonResult = CountResult;
pub type NegativeBinomialResult = CountResult;

/// Integer classical counts only; no offset, weights, robust covariance,
/// missing-row construction or product model state is implied.
pub fn run_poisson(problem: &EstimationProblem) -> Result<PoissonResult, GretlError> {
  run_count(problem, CountFamily::Poisson)
}

/// NB2: Var(Y)=mu+alpha*mu^2. The final parameter is raw alpha, not log(alpha).
pub fn run_negative_binomial(
  problem: &EstimationProblem,
) -> Result<NegativeBinomialResult, GretlError> {
  run_count(problem, CountFamily::NegativeBinomial2)
}

fn run_count(problem: &EstimationProblem, family: CountFamily) -> Result<CountResult, GretlError> {
  let invalid = |message: &str| GretlError::InvalidProblem(message.into());
  let nobs = problem.nobs();
  let ncol = problem.ncol();
  let nb2 = family == CountFamily::NegativeBinomial2;
  let k = problem.predictors.len() + usize::from(problem.include_intercept) + usize::from(nb2);
  crate::check_parameter_count(k)?;
  if nobs <= k || problem.predictors.is_empty() || problem.data.iter().any(|r| r.len() != ncol) {
    return Err(invalid("empty, ragged or undersized count problem"));
  }
  if problem.data.iter().flatten().any(|v| !v.is_finite())
    || problem
      .data
      .iter()
      .any(|r| r[0] < 0.0 || r[0].fract() != 0.0 || r[0] > (1_u64 << 53) as f64)
  {
    return Err(invalid(
      "count spike requires finite data and exact nonnegative integer counts",
    ));
  }
  if problem.outcome.contains('\0') || problem.predictors.iter().any(|name| name.contains('\0')) {
    return Err(invalid("variable name contains NUL"));
  }
  let nvar = ncol
    .checked_add(1)
    .ok_or_else(|| invalid("too many variables"))?;
  if i32::try_from(nvar).is_err() || i32::try_from(nobs).is_err() {
    return Err(invalid("dimensions exceed native limits"));
  }
  let mut dataset = crate::new_dataset(nvar, nobs)?;
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
  let model = if nb2 {
    GretlModel::negative_binomial(&dataset, &list)
  } else {
    GretlModel::poisson(&dataset, &list)
  };
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != nobs || model.ncoeff() != k {
    return Err(invalid("count spike requires a full-rank complete sample"));
  }
  let coefficients: Vec<_> = (0..k)
    .map(|i| Coefficient {
      name: if nb2 && i == k - 1 {
        "alpha".into()
      } else if problem.include_intercept && i == 0 {
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
    .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error < 0.0)
    || covariance.iter().flatten().any(|v| !v.is_finite())
    || !log_likelihood.is_finite()
  {
    return Err(invalid("count native output is invalid/nonfinite"));
  }
  if nb2 && coefficients[k - 1].value <= 0.0 {
    return Err(invalid("NB2 dispersion must be positive"));
  }
  Ok(CountResult {
    family,
    coefficients,
    covariance,
    sample_rows: (0..nobs).collect(),
    log_likelihood,
  })
}
