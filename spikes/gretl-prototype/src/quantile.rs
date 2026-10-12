use crate::{Coefficient, EstimationProblem, GretlError, run_ols};
use gretl_sys::{GretlModel, normal_quantile};

#[derive(Debug, Clone)]
pub struct QuantileResult {
  pub quantile: f64,
  pub coefficients: Vec<Coefficient>,
  /// Canonical iid Epanechnikov/Hall-Sheather covariance, not native sparsity.
  pub covariance: Vec<Vec<f64>>,
  /// Explicit backend diagnostic: gretl's different iid sparsity estimator.
  pub native_iid_covariance: Vec<Vec<f64>>,
  pub bandwidth: f64,
  pub residual_density: f64,
  pub pseudo_r_squared: f64,
  pub sample_rows: Vec<usize>,
}

fn percentile(mut values: Vec<f64>, q: f64) -> f64 {
  values.sort_by(f64::total_cmp);
  let position = (values.len() - 1) as f64 * q;
  let lo = position.floor() as usize;
  let hi = position.ceil() as usize;
  values[lo] + (values[hi] - values[lo]) * (position - lo as f64)
}
fn check_loss(residual: f64, q: f64) -> f64 {
  if residual >= 0. {
    q * residual
  } else {
    (q - 1.) * residual
  }
}

pub fn run_quantile(problem: &EstimationProblem, q: f64) -> Result<QuantileResult, GretlError> {
  let invalid = |message: &str| GretlError::InvalidProblem(message.into());
  if !q.is_finite() || !(0.01..=0.99).contains(&q) {
    return Err(invalid("quantile must be .01 through .99"));
  }
  crate::check_parameter_count(problem.predictors.len() + usize::from(problem.include_intercept))?;
  if i32::try_from(problem.predictors.len() + 3).is_err() {
    return Err(invalid("quantile list dimension exceeds native limits"));
  }
  let mut seen = std::collections::HashSet::new();
  for name in std::iter::once(&problem.outcome).chain(&problem.predictors) {
    if name.is_empty() || name.contains('\0') || name == "intercept" || !seen.insert(name) {
      return Err(invalid(
        "quantile names must be distinct, nonempty, NUL-free and not intercept",
      ));
    }
  }
  // Reuse the checked owned QR/OLS path for (X'X)^-1. Classical OLS covariance
  // is sigma² (X'X)^-1; this avoids introducing a second inversion backend.
  let ols = run_ols(problem)?;
  let sigma_squared = ols.sigma.powi(2);
  if !sigma_squared.is_finite() || sigma_squared <= 0. {
    return Err(invalid("quantile density requires nondegenerate outcome"));
  }
  let n = problem.data.len();
  let p = problem.predictors.len();
  let k = ols.ncoeff;
  let mut dset = crate::new_dataset(p + 2, n)?;
  for i in 0..p + 1 {
    dset.set_varname(i + 1, &format!("td_v{i}"));
  }
  for (t, row) in problem.data.iter().enumerate() {
    for (i, value) in row.iter().enumerate() {
      dset.set_value(i + 1, t, *value);
    }
  }
  let mut list = vec![0, 1];
  if problem.include_intercept {
    list.push(0);
  }
  list.extend((2..p + 2).map(|i| i as i32));
  list[0] = (list.len() - 1) as i32;
  let model = GretlModel::quantile(&dset, &list, q);
  if model.errcode() != 0 {
    return Err(GretlError::EstimationFailed {
      code: model.errcode(),
    });
  }
  if model.nobs() != n || model.ncoeff() != k {
    return Err(invalid("quantile changed full-rank sample/design"));
  }
  let beta: Vec<_> = (0..k).map(|i| model.coeff(i)).collect();
  let native_iid_covariance: Vec<Vec<_>> = (0..k)
    .map(|i| (0..k).map(|j| model.vcv(i, j)).collect())
    .collect();
  if beta.iter().any(|v| !v.is_finite())
    || native_iid_covariance
      .iter()
      .flatten()
      .any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite native quantile output"));
  }
  let offset = usize::from(problem.include_intercept);
  let residuals: Vec<_> = problem
    .data
    .iter()
    .map(|row| {
      row[0]
        - (if problem.include_intercept {
          beta[0]
        } else {
          0.
        } + row[1..]
          .iter()
          .zip(&beta[offset..])
          .map(|(x, b)| x * b)
          .sum::<f64>())
    })
    .collect();
  if residuals.iter().any(|v| !v.is_finite()) {
    return Err(invalid("nonfinite quantile residuals"));
  }
  let outcomes: Vec<_> = problem.data.iter().map(|row| row[0]).collect();
  let mean = outcomes.iter().sum::<f64>() / n as f64;
  let std = (outcomes.iter().map(|y| (y - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
  let iqr = percentile(residuals.clone(), 0.75) - percentile(residuals.clone(), 0.25);
  let z = normal_quantile(q);
  let phi = (-z * z / 2.).exp() / (2. * std::f64::consts::PI).sqrt();
  let h = (n as f64).powf(-1. / 3.)
    * normal_quantile(0.975).powf(2. / 3.)
    * (1.5 * phi * phi / (2. * z * z + 1.)).powf(1. / 3.);
  if q - h <= 0. || q + h >= 1. || !std.is_finite() {
    return Err(invalid(
      "Hall-Sheather bandwidth outside probability domain",
    ));
  }
  let bandwidth = std.min(iqr / 1.34) * (normal_quantile(q + h) - normal_quantile(q - h));
  if !bandwidth.is_finite() || bandwidth <= 0. {
    return Err(invalid("degenerate quantile bandwidth"));
  }
  let residual_density = residuals
    .iter()
    .map(|e| {
      let u = e / bandwidth;
      if u.abs() <= 1. {
        0.75 * (1. - u * u)
      } else {
        0.
      }
    })
    .sum::<f64>()
    / (n as f64 * bandwidth);
  if !residual_density.is_finite() || residual_density <= 0. {
    return Err(invalid("degenerate quantile residual density"));
  }
  let scale = q * (1. - q) / (residual_density.powi(2) * sigma_squared);
  let covariance: Vec<Vec<_>> = ols
    .covariance
    .iter()
    .map(|row| row.iter().map(|v| v * scale).collect())
    .collect();
  let coefficients: Vec<_> = (0..k)
    .map(|i| Coefficient {
      name: ols.coefficients[i].name.clone(),
      value: beta[i],
      standard_error: covariance[i][i].sqrt(),
    })
    .collect();
  let null = percentile(outcomes.clone(), q);
  let pseudo_r_squared = 1.
    - residuals.iter().map(|e| check_loss(*e, q)).sum::<f64>()
      / outcomes
        .iter()
        .map(|y| check_loss(y - null, q))
        .sum::<f64>();
  if !pseudo_r_squared.is_finite()
    || coefficients
      .iter()
      .any(|c| !c.standard_error.is_finite() || c.standard_error <= 0.)
    || covariance.iter().flatten().any(|v| !v.is_finite())
  {
    return Err(invalid("nonfinite canonical quantile result"));
  }
  Ok(QuantileResult {
    quantile: q,
    coefficients,
    covariance,
    native_iid_covariance,
    bandwidth,
    residual_density,
    pseudo_r_squared,
    sample_rows: (0..n).collect(),
  })
}
