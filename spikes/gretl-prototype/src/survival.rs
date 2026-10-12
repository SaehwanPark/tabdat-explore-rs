use crate::{Coefficient, GretlError};
use gretl_sys::{GretlModel, Stat};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurvivalDistribution {
  Exponential,
  Weibull,
}
#[derive(Debug, Clone)]
pub struct SurvivalProblem {
  pub time: String,
  pub failure: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  /// Positive time, binary failure (1 event, 0 right censored), predictors.
  pub data: Vec<Vec<f64>>,
}
#[derive(Debug, Clone)]
pub struct SurvivalResult {
  pub distribution: SurvivalDistribution,
  /// PH log-rate coefficients, followed by log(shape) for Weibull.
  pub coefficients: Vec<Coefficient>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub sample_rows: Vec<usize>,
  pub failure_rows: Vec<usize>,
  pub censored_rows: Vec<usize>,
}

pub fn run_survival(
  problem: &SurvivalProblem,
  distribution: SurvivalDistribution,
) -> Result<SurvivalResult, GretlError> {
  let invalid = |m: &str| GretlError::InvalidProblem(m.into());
  let n = problem.data.len();
  let p = problem.predictors.len();
  let k = p + 1;
  let weibull = distribution == SurvivalDistribution::Weibull;
  let dim = k + usize::from(weibull);
  crate::check_parameter_count(dim)?;
  if !problem.include_intercept
    || n <= dim
    || problem.data.iter().any(|r| r.len() != p + 2)
    || problem.data.iter().flatten().any(|v| !v.is_finite())
    || i32::try_from(n).is_err()
    || i32::try_from(p + 5).is_err()
  {
    return Err(invalid(
      "survival requires an intercept and finite rectangular complete sample",
    ));
  }
  if problem
    .data
    .iter()
    .any(|r| r[0] <= 0. || (r[1] != 0. && r[1] != 1.))
  {
    return Err(invalid("survival needs positive times and binary failures"));
  }
  let failure_rows: Vec<_> = problem
    .data
    .iter()
    .enumerate()
    .filter(|(_, r)| r[1] == 1.)
    .map(|(i, _)| i)
    .collect();
  let censored_rows: Vec<_> = problem
    .data
    .iter()
    .enumerate()
    .filter(|(_, r)| r[1] == 0.)
    .map(|(i, _)| i)
    .collect();
  if failure_rows.is_empty() {
    return Err(invalid("all-censored survival sample is unidentified"));
  }
  let mut seen = std::collections::HashSet::new();
  for name in [&problem.time, &problem.failure]
    .into_iter()
    .chain(&problem.predictors)
  {
    if name.is_empty()
      || name.contains('\0')
      || name == "intercept"
      || name == "log_shape"
      || !seen.insert(name)
    {
      return Err(invalid(
        "survival names must be distinct, nonempty, NUL-free and not reserved",
      ));
    }
  }
  let mut dset = crate::new_dataset(p + 3, n)?;
  for i in 1..p + 3 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    dset.set_value(1, t, row[0]);
    for (i, v) in row[2..].iter().enumerate() {
      dset.set_value(i + 2, t, *v);
    }
    // Native duration uses 1=censored, opposite the public 1=failure contract.
    dset.set_value(p + 2, t, 1. - row[1]);
  }
  let mut list = vec![0, 1, 0];
  list.extend((2..p + 2).map(|i| i as i32));
  // An all-zero censor dummy is collinear with the initial OLS design; omit it
  // entirely for complete events, avoiding native precheck's blind rank decrement.
  if !censored_rows.is_empty() {
    list.extend([-100, (p + 2) as i32]);
  }
  list[0] = (list.len() - 1) as i32;
  let model = GretlModel::survival(&dset, &list, !weibull);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != n || model.ncoeff() != dim {
    return Err(invalid("survival changed full-rank sample/design"));
  }
  let native: Vec<_> = (0..dim).map(|i| model.coeff(i)).collect();
  let vcv: Vec<Vec<_>> = (0..dim)
    .map(|i| (0..dim).map(|j| model.vcv(i, j)).collect())
    .collect();
  let sigma = if weibull { native[k] } else { 1. };
  if sigma <= 0.
    || !sigma.is_finite()
    || native.iter().any(|v| !v.is_finite())
    || vcv.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite native survival result"));
  }
  // Transform AFT beta/raw sigma to PH gamma/log(shape), retaining all cross
  // terms: gamma=-beta/sigma, log(shape)=-log(sigma), V_PH=J V_AFT J'.
  // At most two nonzero derivatives per parameter: keep the transform O(k²),
  // not a four-loop dense product for models with many predictors.
  let derivatives: Vec<Vec<(usize, f64)>> = (0..dim)
    .map(|i| {
      if i < k {
        let mut entries = vec![(i, -1. / sigma)];
        if weibull {
          entries.push((k, native[i] / sigma.powi(2)));
        }
        entries
      } else {
        vec![(k, -1. / sigma)]
      }
    })
    .collect();
  let covariance: Vec<Vec<f64>> = (0..dim)
    .map(|i| {
      (0..dim)
        .map(|j| {
          derivatives[i]
            .iter()
            .map(|&(a, ja)| {
              derivatives[j]
                .iter()
                .map(|&(b, jb)| ja * vcv[a][b] * jb)
                .sum::<f64>()
            })
            .sum()
        })
        .collect()
    })
    .collect();
  let labels: Vec<_> = std::iter::once("intercept")
    .chain(problem.predictors.iter().map(String::as_str))
    .chain(std::iter::once("log_shape").filter(|_| weibull))
    .collect();
  let coefficients: Vec<_> = (0..dim)
    .map(|i| Coefficient {
      name: labels[i].into(),
      value: if i < k {
        -native[i] / sigma
      } else {
        -sigma.ln()
      },
      standard_error: covariance[i][i].sqrt(),
    })
    .collect();
  // duration.c evaluates density on log(time). Restore the time-density
  // Jacobian only for observed events; censored survivor terms do not change.
  let log_likelihood = model.scalar(Stat::LogLikelihood)
    - problem
      .data
      .iter()
      .filter(|row| row[1] == 1.0)
      .map(|row| row[0].ln())
      .sum::<f64>();
  if !log_likelihood.is_finite()
    || coefficients
      .iter()
      .any(|c| !c.value.is_finite() || !c.standard_error.is_finite() || c.standard_error <= 0.)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite PH survival result"));
  }
  Ok(SurvivalResult {
    distribution,
    coefficients,
    covariance,
    log_likelihood,
    sample_rows: (0..n).collect(),
    failure_rows,
    censored_rows,
  })
}
