#![forbid(unsafe_code)]

use crate::distributions::{normal_cdf, normal_pdf, normal_quantile};
use crate::matrix::{
  add_compensated, qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r,
};
use crate::{EstimationProblem, EstimationSample, StatsError};
use clarabel::{algebra::CscMatrix, solver::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileCoefficient {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: f64,
  pub p_value: f64,
  pub ci_lower: f64,
  pub ci_upper: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub quantile: f64,
  pub observation_count: usize,
  pub coefficients: Vec<QuantileCoefficient>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub covariance_parameter_names: Vec<String>,
  pub check_loss: f64,
  pub null_check_loss: Option<f64>,
  pub pseudo_r_squared: Option<f64>,
  pub sparsity: f64,
  pub sparsity_observation_count: usize,
  pub bandwidth_probability: f64,
  pub solver_iterations: u32,
  pub relative_duality_gap: f64,
  pub max_score_violation: f64,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!("quantile regression: {}", message.into()))
}

fn sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut result, mut correction) = (0.0, 0.0);
  for v in values {
    add_compensated(&mut result, &mut correction, v);
  }
  result
}
fn check_loss(residuals: impl Iterator<Item = f64>, tau: f64) -> f64 {
  sum(residuals.map(|r| r * if r < 0.0 { tau - 1.0 } else { tau }))
}

struct PointFit {
  coefficients: Vec<f64>,
  iterations: u32,
  gap: f64,
  score: f64,
  design_inverse: Vec<Vec<f64>>,
}

// Dual LP: max y'u, X'u=0, tau-1 <= u <= tau. Only K equality rows couple
// observations. Pure-Rust sparse QDLDL avoids native LP/BLAS/Python dependencies.
fn point_fit_attempt(
  x: &[Vec<f64>],
  y: &[f64],
  tau: f64,
  intercept: bool,
  tolerance: f64,
) -> Result<PointFit, StatsError> {
  let n = y.len();
  let k = x[0].len();
  let offset = usize::from(intercept);
  let mut centers = vec![0.0; k];
  let mut scales = vec![1.0; k];
  for j in offset..k {
    if intercept {
      centers[j] = sum(x.iter().map(|r| r[j] / n as f64));
    }
    scales[j] = x
      .iter()
      .map(|r| (r[j] - centers[j]).abs())
      .fold(0.0, f64::max);
    if !scales[j].is_finite() || scales[j] <= 0.0 {
      return Err(invalid("rank-deficient or unrepresentable design"));
    }
  }
  let center_y = if intercept {
    sum(y.iter().map(|v| v / n as f64))
  } else {
    0.0
  };
  let scale_y = y.iter().map(|v| (v - center_y).abs()).fold(0.0, f64::max);
  if !center_y.is_finite() || !scale_y.is_finite() || scale_y <= 0.0 {
    return Err(invalid(
      "positive representable response variation is required",
    ));
  }
  let scaled: Vec<Vec<f64>> = x
    .iter()
    .map(|r| (0..k).map(|j| (r[j] - centers[j]) / scales[j]).collect())
    .collect();
  let response: Vec<_> = y.iter().map(|v| (v - center_y) / scale_y).collect();
  let qr = qr_decompose_with_response(&scaled, &response)?;
  let mut pointers = Vec::with_capacity(n + 1);
  let mut rows = Vec::with_capacity(n * (k + 2));
  let mut values = Vec::with_capacity(n * (k + 2));
  for (i, row) in scaled.iter().enumerate() {
    pointers.push(rows.len());
    for (j, &value) in row.iter().enumerate() {
      rows.push(j);
      values.push(value);
    }
    rows.push(k + i);
    values.push(1.0);
    rows.push(k + n + i);
    values.push(-1.0);
  }
  pointers.push(rows.len());
  let a = CscMatrix::new(k + 2 * n, n, pointers, rows, values);
  let p = CscMatrix::zeros((n, n));
  let q: Vec<_> = response.iter().map(|v| -v).collect();
  let mut b = vec![0.0; k];
  b.extend(vec![tau; n]);
  b.extend(vec![1.0 - tau; n]);
  let settings = DefaultSettingsBuilder::default()
    .verbose(false)
    .max_iter(200)
    .time_limit(10.0)
    .tol_gap_abs(tolerance)
    .tol_gap_rel(tolerance)
    .tol_feas(tolerance)
    .build()
    .map_err(|e| invalid(e.to_string()))?;
  let mut solver = DefaultSolver::new(
    &p,
    &q,
    &a,
    &b,
    &[ZeroConeT(k), NonnegativeConeT(2 * n)],
    settings,
  )
  .map_err(|e| invalid(e.to_string()))?;
  solver.solve();
  if solver.solution.status != SolverStatus::Solved {
    return Err(invalid(format!(
      "LP did not solve: {:?}",
      solver.solution.status
    )));
  }
  let u = &solver.solution.x;
  if u
    .iter()
    .any(|v| !v.is_finite() || *v < tau - 1.0 - 1e-8 || *v > tau + 1e-8)
  {
    return Err(invalid("LP subgradient bounds failed"));
  }
  // Strictly interior subgradients force zero residuals at any optimum. A full
  // rank subset certifies an isolated coefficient vector and permits QR polish.
  let active: Vec<_> = u
    .iter()
    .enumerate()
    .filter(|(_, v)| **v > tau - 1.0 + 1e-5 && **v < tau - 1e-5)
    .map(|(i, _)| i)
    .collect();
  let ax: Vec<_> = active.iter().map(|&i| scaled[i].clone()).collect();
  let ay: Vec<_> = active.iter().map(|&i| response[i]).collect();
  let beta = qr_decompose_with_response(&ax, &ay)
    .and_then(|qr| solve_upper_triangular(&qr.r, &qr.qty))
    .map_err(|_| invalid("optimal coefficients are not isolated at solver precision"))?;
  if ax
    .iter()
    .zip(&ay)
    .any(|(row, y)| (y - sum(row.iter().zip(&beta).map(|(x, b)| x * b))).abs() > 1e-9)
  {
    return Err(invalid(
      "active LP constraints are unresolved at solver precision",
    ));
  }
  let loss = check_loss(
    scaled
      .iter()
      .zip(&response)
      .map(|(row, y)| y - sum(row.iter().zip(&beta).map(|(x, b)| x * b))),
    tau,
  );
  let dual = sum(response.iter().zip(u).map(|(y, u)| y * u));
  let gap = (loss - dual).abs() / loss.abs().max(1.0);
  let score = (0..k)
    .map(|j| sum(scaled.iter().zip(u).map(|(r, u)| r[j] * u)).abs())
    .fold(0.0, f64::max);
  if !gap.is_finite() || gap > 1e-8 || !score.is_finite() || score > 1e-7 {
    return Err(invalid("LP optimality certificate failed after QR polish"));
  }
  let mut coefficients: Vec<_> = beta
    .iter()
    .enumerate()
    .map(|(j, b)| b * scale_y / scales[j])
    .collect();
  if intercept {
    coefficients[0] += center_y - sum((1..k).map(|j| coefficients[j] * centers[j]));
  }
  if coefficients.iter().any(|v| !v.is_finite()) {
    return Err(invalid("coefficient rescaling overflowed"));
  }
  // If X_scaled = X_original * T, then inv(X'X) = T inv(Xs'Xs) T'.
  // Keep this calculation in scaled coordinates until the final unit transform.
  let inverse = xtx_inverse_from_r(&qr.r)?;
  let mut transform = vec![vec![0.0; k]; k];
  for j in 0..k {
    transform[j][j] = 1.0 / scales[j];
    if intercept && j > 0 {
      transform[0][j] = -centers[j] / scales[j];
    }
  }
  let mut design_inverse = vec![vec![0.0; k]; k];
  for i in 0..k {
    for j in 0..=i {
      let value = sum(
        (0..k)
          .flat_map(|a| (0..k).map(move |b| (a, b)))
          .map(|(a, b)| transform[i][a] * inverse[a][b] * transform[j][b]),
      );
      design_inverse[i][j] = value;
      design_inverse[j][i] = value;
    }
  }
  Ok(PointFit {
    coefficients,
    iterations: solver.solution.iterations,
    gap,
    score,
    design_inverse,
  })
}

fn point_fit(x: &[Vec<f64>], y: &[f64], tau: f64, intercept: bool) -> Result<PointFit, StatsError> {
  point_fit_attempt(x, y, tau, intercept, 1e-10).or_else(|initial| {
    // Near-zero residuals can look active when their dual is almost at a bound.
    // Refine the same LP rather than weakening the active/gap/score certificates.
    point_fit_attempt(x, y, tau, intercept, 1e-12).map_err(|_| initial)
  })
}

// Bootstrap point fits deliberately omit residual-spacing inference: duplicate
// pairs can have zero residual spacings without invalidating their coefficients.
pub(crate) fn quantile_design(
  problem: &EstimationProblem,
  tau: f64,
) -> Result<Vec<Vec<f64>>, StatsError> {
  problem.validate()?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if !tau.is_finite() || !(0.05..=0.95).contains(&tau) {
    return Err(invalid(
      "supported quantiles are finite values from 0.05 to 0.95",
    ));
  }
  if problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|s| s.weights.is_some() || s.cluster_groups.is_some())
  {
    return Err(invalid(
      "only unweighted independent observations are supported",
    ));
  }
  if n <= k || n > 10000 || k > 32 || n.checked_mul(k * k).is_none_or(|v| v > 10_000_000) {
    return Err(invalid("requires N>K, N<=10000, K<=32 and N*K²<=10000000"));
  }
  let design: Vec<Vec<f64>> = problem
    .design_matrix
    .iter()
    .map(|r| {
      let mut row = Vec::with_capacity(k);
      if problem.include_intercept {
        row.push(1.0);
      }
      row.extend_from_slice(r);
      row
    })
    .collect();
  Ok(design)
}

pub(crate) fn quantile_point_coefficients(
  x: &[Vec<f64>],
  y: &[f64],
  tau: f64,
  intercept: bool,
) -> Result<Vec<f64>, StatsError> {
  point_fit(x, y, tau, intercept).map(|p| p.coefficients)
}

/// Unweighted conditional quantiles with IID-location-error sparsity covariance.
/// Hall–Sheather bandwidth and local residual-spacing median regression follow
/// quantreg::summary.rq(se="iid", hs=TRUE). Tails here are asymptotic normal.
pub fn fit_quantile_iid(
  problem: &EstimationProblem,
  tau: f64,
) -> Result<QuantileResult, StatsError> {
  let design = quantile_design(problem, tau)?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  let point = point_fit(&design, &problem.response, tau, problem.include_intercept)?;
  let residuals: Vec<_> = design
    .iter()
    .zip(&problem.response)
    .map(|(r, y)| y - sum(r.iter().zip(&point.coefficients).map(|(x, b)| x * b)))
    .collect();
  let z = normal_quantile(tau);
  let z95 = normal_quantile(0.975);
  let bandwidth = (n as f64).powf(-1.0 / 3.0)
    * z95.powf(2.0 / 3.0)
    * (1.5 * normal_pdf(z).powi(2) / (2.0 * z * z + 1.0)).cbrt();
  let h = (k + 1).max((n as f64 * bandwidth).ceil() as usize);
  let pz = residuals
    .iter()
    .filter(|v| v.abs() < f64::EPSILON.sqrt())
    .count();
  if h.checked_add(pz + 1).is_none_or(|end| end > n) {
    return Err(invalid(
      "insufficient nonzero residuals for IID sparsity estimation",
    ));
  }
  let mut nearest = residuals.clone();
  nearest.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
  let mut local = nearest[pz..pz + h + 1].to_vec();
  local.sort_by(f64::total_cmp);
  let spacing: Vec<_> = (pz + 1..=pz + h + 1)
    .map(|i| vec![1.0, i as f64 / (n - k) as f64])
    .collect();
  let sparsity = point_fit(&spacing, &local, 0.5, true)?.coefficients[1];
  if !sparsity.is_finite() || sparsity <= 0.0 {
    return Err(invalid("IID error sparsity must be positive and finite"));
  }
  let scale = tau * (1.0 - tau) * sparsity * sparsity;
  let covariance: Vec<Vec<f64>> = point
    .design_inverse
    .iter()
    .map(|r| r.iter().map(|v| v * scale).collect())
    .collect();
  let names = problem.parameter_names();
  let coefficients: Vec<_> = point
    .coefficients
    .iter()
    .enumerate()
    .map(|(j, &value)| {
      let se = covariance[j][j].sqrt();
      let statistic = value / se;
      QuantileCoefficient {
        name: names[j].clone(),
        value,
        standard_error: se,
        statistic,
        p_value: 2.0 * normal_cdf(-statistic.abs()),
        ci_lower: value - z95 * se,
        ci_upper: value + z95 * se,
      }
    })
    .collect();
  if covariance.iter().flatten().any(|v| !v.is_finite())
    || coefficients.iter().any(|b| {
      [
        b.value,
        b.standard_error,
        b.statistic,
        b.p_value,
        b.ci_lower,
        b.ci_upper,
      ]
      .iter()
      .any(|v| !v.is_finite())
        || b.standard_error <= 0.0
    })
  {
    return Err(invalid("IID inference overflowed or is degenerate"));
  }
  let loss = check_loss(residuals.into_iter(), tau);
  let null = if problem.include_intercept {
    let mut y = problem.response.clone();
    y.sort_by(f64::total_cmp);
    let index = ((n as f64 * tau).ceil() as usize)
      .saturating_sub(1)
      .min(n - 1);
    Some(check_loss(y.iter().map(|v| v - y[index]), tau))
  } else {
    None
  };
  let pseudo = null.filter(|v| *v > 0.0).map(|v| 1.0 - loss / v);
  if !loss.is_finite()
    || null.is_some_and(|v| !v.is_finite())
    || pseudo.is_some_and(|v| !v.is_finite())
  {
    return Err(invalid("check-loss summaries overflowed"));
  }
  Ok(QuantileResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    quantile: tau,
    observation_count: n,
    coefficients,
    covariance_matrix: covariance,
    covariance_parameter_names: names,
    check_loss: loss,
    null_check_loss: null,
    pseudo_r_squared: pseudo,
    sparsity,
    sparsity_observation_count: h + 1,
    bandwidth_probability: bandwidth,
    solver_iterations: point.iterations,
    relative_duality_gap: point.gap,
    max_score_violation: point.score,
    sample: problem.sample.clone(),
  })
}
