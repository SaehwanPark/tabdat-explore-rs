//! Nonrobust centred-rank tricube row smoothing, not nearest-neighbour LOWESS.
use crate::StatsError;
use crate::matrix::add_compensated;

pub const MAX_LOWESS_OBSERVATIONS: usize = 10_000;
pub const MAX_LOWESS_WORK: usize = 20_000_000;

/// Fits are restored to input order. No estimator state or out-of-sample model.
#[derive(Debug, Clone, PartialEq)]
pub struct LowessSmoothing {
  pub values: Vec<f64>,
  pub half_window: usize,
  pub local_constant_windows: usize,
}

fn compensated_sum(values: impl Iterator<Item = f64>) -> f64 {
  let (mut sum, mut correction) = (0.0, 0.0);
  for value in values {
    add_compensated(&mut sum, &mut correction, value);
  }
  sum
}

/// Already joint-finite pairs only; missing-row selection belongs to the caller.
/// Numeric-X sorting is stable, including signed-zero ties. Rank windows shrink
/// at endpoints and can give different fits to tied X: their input order matters.
/// Coincident-X windows use local means; k=0 is the exact identity smoother.
pub fn smooth_lowess(
  predictor: &[f64],
  outcome: &[f64],
  bandwidth: f64,
) -> Result<LowessSmoothing, StatsError> {
  let n = predictor.len();
  if n != outcome.len() {
    return Err(StatsError::DimensionMismatch(
      "LOWESS requires equal X/Y lengths".into(),
    ));
  }
  if !(2..=MAX_LOWESS_OBSERVATIONS).contains(&n) {
    return Err(StatsError::InvalidEstimation(format!(
      "LOWESS requires 2..={MAX_LOWESS_OBSERVATIONS} joint-finite observations"
    )));
  }
  if !bandwidth.is_finite() || bandwidth <= 0.0 || bandwidth >= 1.0 {
    return Err(StatsError::InvalidEstimation(
      "LOWESS bandwidth must be finite and strictly between 0 and 1".into(),
    ));
  }
  if predictor.iter().chain(outcome).any(|v| !v.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "LOWESS requires already joint-finite X/Y pairs".into(),
    ));
  }
  if n as f64 * bandwidth < 0.5 {
    return Err(StatsError::InvalidEstimation(
      "LOWESS bandwidth is too small: N * bandwidth must be at least 0.5".into(),
    ));
  }
  let k = ((n as f64 * bandwidth - 0.5) / 2.0).floor() as usize;
  if n * n.min(2 * k + 1) > MAX_LOWESS_WORK {
    return Err(StatsError::InvalidEstimation(format!(
      "LOWESS N * min(N, 2*k+1) exceeds {MAX_LOWESS_WORK} window-work limit"
    )));
  }
  if k == 0 {
    return Ok(LowessSmoothing {
      values: outcome.to_vec(),
      half_window: k,
      local_constant_windows: n,
    });
  }
  let mut order: Vec<usize> = (0..n).collect();
  // Stable numeric comparison, not total_cmp: -0 and +0 denote the same X.
  order.sort_by(|a, b| {
    predictor[*a]
      .partial_cmp(&predictor[*b])
      .expect("finite X validated")
  });
  let mut values = vec![0.0; n];
  let mut local_constant_windows = 0;
  for i in 0..n {
    let left = i.saturating_sub(k);
    let right = (i + k).min(n - 1);
    let x = predictor[order[i]];
    let radius = (x - predictor[order[left]])
      .abs()
      .max((predictor[order[right]] - x).abs());
    if !radius.is_finite() {
      return Err(StatsError::NonFiniteValue(
        "LOWESS local predictor difference is unrepresentable".into(),
      ));
    }
    let indices = &order[left..=right];
    // Scaling is local: a huge response OUTSIDE this window must not erase
    // representable small local values by underflow before their fit is solved.
    let y_scale = indices
      .iter()
      .map(|j| outcome[*j].abs())
      .fold(0.0_f64, f64::max);
    let y_scale = if y_scale == 0.0 { 1.0 } else { y_scale };
    let value = if radius == 0.0 {
      local_constant_windows += 1;
      compensated_sum(indices.iter().map(|j| outcome[*j] / y_scale)) / indices.len() as f64
    } else {
      // Use z=(X-X_i)/radius and scaled Y. Avoid raw-X normal equations and
      // overflowing radius*1.0001; these transformations do not change WLS.
      let points: Vec<(f64, f64, f64)> = indices
        .iter()
        .map(|j| {
          let z = (predictor[*j] - x) / radius;
          let w = (1.0 - (z.abs() / 1.0001).powi(3)).powi(3);
          (w, z, outcome[*j] / y_scale)
        })
        .collect();
      let weight = compensated_sum(points.iter().map(|(w, _, _)| *w));
      let mean_x = compensated_sum(points.iter().map(|(w, z, _)| w * z)) / weight;
      let mean_y = compensated_sum(points.iter().map(|(w, _, v)| w * v)) / weight;
      let variance = compensated_sum(points.iter().map(|(w, z, _)| w * (z - mean_x).powi(2)));
      let covariance = compensated_sum(
        points
          .iter()
          .map(|(w, z, v)| w * (z - mean_x) * (v - mean_y)),
      );
      // Positive radius includes two distinct endpoints with positive weights.
      // A failed numerical variance is NOT permission for another fallback fit.
      if !variance.is_finite() || variance <= 0.0 {
        return Err(StatsError::EstimationFailed(
          "LOWESS positive-radius window has invalid weighted variance".into(),
        ));
      }
      mean_y - mean_x * (covariance / variance)
    } * y_scale;
    if !value.is_finite() {
      return Err(StatsError::EstimationFailed(
        "LOWESS fitted value is unrepresentable".into(),
      ));
    }
    values[order[i]] = value;
  }
  Ok(LowessSmoothing {
    values,
    half_window: k,
    local_constant_windows,
  })
}

#[cfg(test)]
mod tests;
