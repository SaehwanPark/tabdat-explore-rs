use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{CovarianceType, InstrumentalVariablesProblem, StatsError};

use super::{ArConfidenceInterval, ArConfidenceSetShape, interval, invalid, residualize};

/// HC1/one-way CR1 AR confidence set for one endogenous regressor and two through
/// eight excluded instruments. The joint-null degrees of freedom
/// equal the number of excluded instruments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustMultipleInstrumentAndersonRubinConfidenceSet {
  pub endogenous: String,
  pub excluded_instruments: Vec<String>,
  pub confidence_level: f64,
  pub observation_count: usize,
  pub auxiliary_parameter_count: usize,
  pub auxiliary_residual_df: usize,
  pub covariance_type: CovarianceType,
  pub covariance_correction: f64,
  pub cluster_group_count: Option<usize>,
  pub degrees_of_freedom: usize,
  pub critical_chi_square: f64,
  pub shape: ArConfidenceSetShape,
  pub intervals: Vec<ArConfidenceInterval>,
}

/// Projection of a robust joint AR region for exactly two endogenous
/// coefficients and two excluded instruments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustProjectedArConfidenceSet {
  pub coefficient: String,
  pub nuisance_endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub confidence_level: f64,
  pub observation_count: usize,
  pub auxiliary_parameter_count: usize,
  pub auxiliary_residual_df: usize,
  pub covariance_type: CovarianceType,
  pub covariance_correction: f64,
  pub cluster_group_count: Option<usize>,
  pub degrees_of_freedom: usize,
  pub critical_chi_square: f64,
  pub shape: ArConfidenceSetShape,
  pub intervals: Vec<ArConfidenceInterval>,
  /// True because projected output reports the closed conservative envelope.
  pub closure_conservative: bool,
  /// True when numerical ambiguity forced the conservative all-real set.
  pub all_real_conservative_fallback: bool,
}

type Polynomial = Vec<f64>;
type ScorePair = (Vec<f64>, Vec<f64>);
type BivariatePolynomial = Vec<Polynomial>;
type ProjectionIntervalSet = (
  ArConfidenceSetShape,
  Vec<ArConfidenceInterval>,
  Vec<f64>,
  Vec<f64>,
);

struct NuisanceMinimum {
  included: bool,
  minimizer: f64,
  stationary_points: Vec<f64>,
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
  left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn polynomial_add(left: &[f64], right: &[f64]) -> Polynomial {
  let mut result = vec![0.0; left.len().max(right.len())];
  for (index, value) in left.iter().enumerate() {
    result[index] += value;
  }
  for (index, value) in right.iter().enumerate() {
    result[index] += value;
  }
  result
}

fn polynomial_scale(polynomial: &[f64], scale: f64) -> Polynomial {
  polynomial
    .iter()
    .map(|coefficient| coefficient * scale)
    .collect()
}

fn polynomial_multiply(left: &[f64], right: &[f64]) -> Polynomial {
  let mut result = vec![0.0; left.len() + right.len() - 1];
  for (left_index, left_value) in left.iter().enumerate() {
    for (right_index, right_value) in right.iter().enumerate() {
      result[left_index + right_index] += left_value * right_value;
    }
  }
  result
}

fn bivariate_trim(polynomial: &mut BivariatePolynomial) {
  while polynomial.len() > 1
    && polynomial
      .last()
      .is_some_and(|coefficient| coefficient.iter().all(|value| *value == 0.0))
  {
    polynomial.pop();
  }
}

fn bivariate_add(left: &[Polynomial], right: &[Polynomial]) -> BivariatePolynomial {
  let mut result = (0..left.len().max(right.len()))
    .map(|nuisance_power| {
      polynomial_add(
        left.get(nuisance_power).map_or(&[0.0], Vec::as_slice),
        right.get(nuisance_power).map_or(&[0.0], Vec::as_slice),
      )
    })
    .collect::<Vec<_>>();
  bivariate_trim(&mut result);
  result
}

fn bivariate_scale(polynomial: &[Polynomial], scale: f64) -> BivariatePolynomial {
  let mut result = polynomial
    .iter()
    .map(|coefficient| polynomial_scale(coefficient, scale))
    .collect::<Vec<_>>();
  bivariate_trim(&mut result);
  result
}

fn bivariate_multiply(left: &[Polynomial], right: &[Polynomial]) -> BivariatePolynomial {
  let mut result = vec![vec![0.0]; left.len() + right.len() - 1];
  for (left_power, left_coefficient) in left.iter().enumerate() {
    for (right_power, right_coefficient) in right.iter().enumerate() {
      let product = polynomial_multiply(left_coefficient, right_coefficient);
      result[left_power + right_power] =
        polynomial_add(&result[left_power + right_power], &product);
    }
  }
  bivariate_trim(&mut result);
  result
}

fn bivariate_value_at_target(polynomial: &[Polynomial], target: f64) -> Polynomial {
  polynomial
    .iter()
    .map(|coefficient| polynomial_value(coefficient, target))
    .collect()
}

fn resultant_candidate_is_real_boundary(
  boundary: &[Polynomial],
  candidate: f64,
) -> Result<bool, StatsError> {
  let evaluated = bivariate_value_at_target(boundary, candidate);
  let derivative = bivariate_nuisance_derivative(boundary);
  let evaluated_derivative = bivariate_value_at_target(&derivative, candidate);
  let minimum = nuisance_polynomial_minimum(&evaluated, true)?;
  let mut closest_residual = f64::INFINITY;
  for nuisance in &minimum.stationary_points {
    let value = polynomial_value(&evaluated, *nuisance);
    let derivative_value = polynomial_value(&evaluated_derivative, *nuisance);
    let value_scale = polynomial_residual_scale(&evaluated, *nuisance);
    let derivative_scale = polynomial_residual_scale(&evaluated_derivative, *nuisance);
    if [value, derivative_value, value_scale, derivative_scale]
      .iter()
      .any(|number| !number.is_finite())
    {
      return Err(invalid(
        "robust AR resultant candidate residual is non-finite",
      ));
    }
    let value_residual = if value_scale == 0.0 {
      0.0
    } else {
      value.abs() / value_scale
    };
    let derivative_residual = if derivative_scale == 0.0 {
      0.0
    } else {
      derivative_value.abs() / derivative_scale
    };
    closest_residual = closest_residual.min(value_residual.max(derivative_residual));
    if value_residual <= 2e-8 && derivative_residual <= 2e-8 {
      return Ok(true);
    }
  }
  if closest_residual <= 1e-5 {
    return Err(invalid(
      "robust AR resultant candidate has an unresolved common-root residual",
    ));
  }
  Ok(false)
}

fn bivariate_nuisance_derivative(polynomial: &[Polynomial]) -> BivariatePolynomial {
  polynomial
    .iter()
    .enumerate()
    .skip(1)
    .map(|(power, coefficient)| polynomial_scale(coefficient, power as f64))
    .collect()
}

fn polynomial_resultant_with_nuisance_derivative(
  polynomial: &[Polynomial],
) -> Result<Polynomial, StatsError> {
  let mut polynomial = polynomial.to_vec();
  bivariate_trim(&mut polynomial);
  let degree = polynomial.len().saturating_sub(1);
  if degree == 0 {
    return Ok(vec![0.0]);
  }
  let derivative = bivariate_nuisance_derivative(&polynomial);
  let derivative_degree = derivative.len().saturating_sub(1);
  if derivative_degree + 1 != degree {
    return Err(invalid(
      "robust AR projection resultant degree is ambiguous",
    ));
  }
  let size = degree + derivative_degree;
  let mut matrix = vec![vec![vec![0.0]; size]; size];
  for row in 0..derivative_degree {
    for (offset, coefficient) in polynomial.iter().rev().enumerate() {
      matrix[row][row + offset] = coefficient.clone();
    }
  }
  for row in 0..degree {
    for (offset, coefficient) in derivative.iter().rev().enumerate() {
      matrix[derivative_degree + row][row + offset] = coefficient.clone();
    }
  }
  let resultant = polynomial_matrix_determinant(&matrix);
  if resultant.iter().any(|coefficient| !coefficient.is_finite()) {
    return Err(invalid("robust AR projection resultant is non-finite"));
  }
  Ok(resultant)
}

fn score_covariance_well_conditioned_at(
  score_covariance: &[[BivariatePolynomial; 2]; 2],
  target: f64,
  nuisance: f64,
) -> bool {
  let evaluate = |polynomial: &[Polynomial]| {
    polynomial_value(&bivariate_value_at_target(polynomial, target), nuisance)
  };
  let s11 = evaluate(&score_covariance[0][0]);
  let s12 = evaluate(&score_covariance[0][1]);
  let s22 = evaluate(&score_covariance[1][1]);
  let scale = s11.abs().max(s12.abs()).max(s22.abs());
  if !scale.is_finite() || scale <= 0.0 {
    return false;
  }
  let a = s11 / scale;
  let b = s12 / scale;
  let d = s22 / scale;
  let diagonal_product = a * d;
  let determinant = diagonal_product - b * b;
  diagonal_product.is_finite()
    && determinant.is_finite()
    && diagonal_product > 0.0
    && determinant > 1e-12 * diagonal_product
}

fn distinct_projection_candidates(mut roots: Vec<f64>) -> Result<Vec<f64>, StatsError> {
  if roots.iter().any(|root| !root.is_finite()) {
    return Err(invalid("robust AR projection boundary is non-finite"));
  }
  roots.sort_by(f64::total_cmp);
  let mut distinct = Vec::with_capacity(roots.len());
  for root in roots {
    let Some(previous) = distinct.last_mut() else {
      distinct.push(root);
      continue;
    };
    let scale = 1.0 + root.abs().max(previous.abs());
    let distance = (root - *previous).abs();
    if distance <= 32.0 * f64::EPSILON * scale {
      *previous += (root - *previous) / 2.0;
    } else if distance <= 1e-7 * scale {
      return Err(invalid(
        "robust AR projection boundaries are too close to distinguish reliably",
      ));
    } else {
      distinct.push(root);
    }
  }
  if distinct.len() > 32 {
    return Err(invalid(
      "robust AR projection has too many boundary candidates",
    ));
  }
  Ok(distinct)
}

/// Return whether a nuisance value exists with polynomial value <= 0, and a
/// finite witness. The polynomial is scaled before root isolation; numerical
/// degree loss at a boundary is treated as an error for the caller's fallback.
fn nuisance_polynomial_minimum(
  polynomial: &[f64],
  boundary_candidate: bool,
) -> Result<NuisanceMinimum, StatsError> {
  let mut coefficients = polynomial.to_vec();
  if coefficients.iter().any(|value| !value.is_finite()) {
    return Err(invalid("robust AR projection polynomial is non-finite"));
  }
  let scale = coefficients
    .iter()
    .map(|value| value.abs())
    .fold(0.0, f64::max);
  if scale == 0.0 {
    return Ok(NuisanceMinimum {
      included: true,
      minimizer: 0.0,
      stationary_points: Vec::new(),
    });
  }
  for coefficient in &mut coefficients {
    *coefficient /= scale;
  }
  let mut dropped_leading_coefficients = 0;
  while coefficients.len() > 1 {
    let leading = coefficients.last().copied().unwrap_or(0.0).abs();
    if leading == 0.0 {
      coefficients.pop();
    } else if boundary_candidate && leading <= 1e-10 {
      dropped_leading_coefficients += 1;
      if dropped_leading_coefficients > 1 {
        return Err(invalid(
          "robust AR projection boundary has multiple unresolved nuisance degree drops",
        ));
      }
      coefficients.pop();
    } else if leading <= 1e-13 {
      return Err(invalid(
        "robust AR projection nuisance degree is numerically ambiguous",
      ));
    } else {
      break;
    }
  }
  let degree = coefficients.len() - 1;
  if degree == 0 {
    return Ok(NuisanceMinimum {
      included: coefficients[0] <= 0.0,
      minimizer: 0.0,
      stationary_points: Vec::new(),
    });
  }
  let derivative = polynomial_derivative(&coefficients);
  let critical_points = projected_real_polynomial_roots(&derivative, false)?;
  validate_real_polynomial_roots(&derivative, &critical_points)?;
  for point in &critical_points {
    let value = polynomial_value(&derivative, *point);
    if !value.is_finite() || !near_zero(&derivative, *point, value) {
      return Err(invalid(
        "robust AR projection nuisance stationary-point residual is too large",
      ));
    }
  }
  if degree % 2 == 1 || coefficients[degree] < 0.0 {
    let positive_tail = coefficients[degree] < 0.0 || degree.is_multiple_of(2);
    let mut bound = 1.0_f64.max(
      critical_points
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max),
    );
    for _ in 0..100 {
      let candidate = if positive_tail { bound } else { -bound };
      let value = polynomial_value(&coefficients, candidate);
      if value.is_finite() && value <= 0.0 {
        return Ok(NuisanceMinimum {
          included: true,
          minimizer: candidate,
          stationary_points: critical_points,
        });
      }
      bound *= 2.0;
      if !bound.is_finite() {
        break;
      }
    }
    return Err(invalid(
      "robust AR projection nuisance tail could not be evaluated reliably",
    ));
  }
  if critical_points.is_empty() {
    return Err(invalid(
      "robust AR projection nuisance minimum could not be isolated",
    ));
  }
  let mut minimum = f64::INFINITY;
  let mut minimizer = 0.0;
  for point in critical_points.iter().copied() {
    let value = polynomial_value(&coefficients, point);
    if !value.is_finite() {
      return Err(invalid("robust AR projection minimum is non-finite"));
    }
    if value < minimum {
      minimum = value;
      minimizer = point;
    }
  }
  if boundary_candidate && minimum.abs() <= 2e-10 {
    return Ok(NuisanceMinimum {
      included: true,
      minimizer,
      stationary_points: critical_points,
    });
  }
  if !boundary_candidate && minimum.abs() <= 2e-12 {
    return Err(invalid(
      "robust AR projection cell membership is numerically ambiguous",
    ));
  }
  Ok(NuisanceMinimum {
    included: minimum <= 0.0,
    minimizer,
    stationary_points: critical_points,
  })
}

fn projection_interval_set(
  boundary: &[Polynomial],
  target_center: f64,
) -> Result<ProjectionIntervalSet, StatsError> {
  let mut candidates = Vec::new();
  for coefficient in boundary {
    let roots = projected_real_polynomial_roots(coefficient, false)?;
    validate_real_polynomial_roots(coefficient, &roots)?;
    candidates.extend(roots);
  }
  let resultant = polynomial_resultant_with_nuisance_derivative(boundary)?;
  if boundary.len() > 1 && resultant.iter().all(|value| *value == 0.0) {
    return Err(invalid(
      "robust AR projection resultant is identically zero",
    ));
  }
  let resultant_candidates = projected_real_polynomial_roots(&resultant, false)?;
  validate_real_polynomial_roots(&resultant, &resultant_candidates)?;
  let mut verified_resultant_candidates = Vec::new();
  for candidate in resultant_candidates {
    if resultant_candidate_is_real_boundary(boundary, candidate)? {
      verified_resultant_candidates.push(candidate);
      candidates.push(candidate);
    }
  }
  let candidates = distinct_projection_candidates(candidates)?;
  let sample_for_interval = |lower: Option<f64>, upper: Option<f64>| -> Result<f64, StatsError> {
    match (lower, upper) {
      (None, None) => Ok(0.0),
      (None, Some(upper)) => {
        let step = 1.0_f64.max(upper.abs());
        let sample = upper - step;
        sample.is_finite().then_some(sample).ok_or_else(|| {
          invalid("robust AR projection interval sample exceeds the numerical range")
        })
      }
      (Some(lower), None) => {
        let step = 1.0_f64.max(lower.abs());
        let sample = lower + step;
        sample.is_finite().then_some(sample).ok_or_else(|| {
          invalid("robust AR projection interval sample exceeds the numerical range")
        })
      }
      (Some(lower), Some(upper)) => Ok(lower + (upper - lower) / 2.0),
    }
  };
  let mut intervals = Vec::new();
  let mut lower: Option<f64> = None;
  for upper in candidates
    .iter()
    .copied()
    .map(Some)
    .chain(std::iter::once(None))
  {
    let sample = sample_for_interval(lower, upper)?;
    let evaluated = bivariate_value_at_target(boundary, sample);
    let minimum = nuisance_polynomial_minimum(&evaluated, false)?;
    if minimum.included {
      intervals.push(interval(
        lower.map(|value| value + target_center),
        upper.map(|value| value + target_center),
      ));
    }
    lower = upper;
  }
  for candidate in candidates.iter().copied() {
    let evaluated = bivariate_value_at_target(boundary, candidate);
    let minimum = nuisance_polynomial_minimum(&evaluated, true)?;
    if minimum.included {
      let value = candidate + target_center;
      if !value.is_finite() {
        return Err(invalid("robust AR projection endpoint is non-finite"));
      }
      intervals.push(interval(Some(value), Some(value)));
    }
  }
  intervals.sort_by(|left, right| match (left.lower, right.lower) {
    (None, None) => std::cmp::Ordering::Equal,
    (None, Some(_)) => std::cmp::Ordering::Less,
    (Some(_), None) => std::cmp::Ordering::Greater,
    (Some(left), Some(right)) => left.total_cmp(&right),
  });
  let mut merged: Vec<ArConfidenceInterval> = Vec::with_capacity(intervals.len());
  for current in intervals {
    let Some(previous) = merged.last_mut() else {
      merged.push(current);
      continue;
    };
    let touches = match (previous.upper, current.lower) {
      (None, _) | (_, None) => true,
      (Some(previous), Some(current)) => current <= previous + 1e-9 * (1.0 + previous.abs()),
    };
    if touches {
      previous.upper = match (previous.upper, current.upper) {
        (None, _) | (_, None) => None,
        (Some(previous), Some(current)) => Some(previous.max(current)),
      };
    } else {
      merged.push(current);
    }
  }
  if merged.len() > 7 {
    return Err(invalid("robust AR projection has too many components"));
  }
  for current in &mut merged {
    for bound in [&mut current.lower, &mut current.upper]
      .into_iter()
      .flatten()
    {
      if !bound.is_finite() {
        return Err(invalid("robust AR projection endpoint is non-finite"));
      }
    }
  }
  let shape = match merged.as_slice() {
    [] => ArConfidenceSetShape::Empty,
    [only] if only.lower.is_none() && only.upper.is_none() => ArConfidenceSetShape::AllReal,
    [only] if only.lower.is_none() => ArConfidenceSetShape::LowerRay,
    [only] if only.upper.is_none() => ArConfidenceSetShape::UpperRay,
    [only] if only.lower == only.upper => ArConfidenceSetShape::Singleton,
    [_] => ArConfidenceSetShape::Bounded,
    [_, _] => ArConfidenceSetShape::Disjoint,
    _ => ArConfidenceSetShape::Multiple,
  };
  Ok((shape, merged, candidates, verified_resultant_candidates))
}

fn polynomial_matrix_determinant(matrix: &[Vec<Polynomial>]) -> Polynomial {
  fn expand(
    matrix: &[Vec<Polynomial>],
    row: usize,
    columns: &mut Vec<usize>,
    product: &[f64],
    permutation_sign: f64,
    result: &mut Polynomial,
  ) {
    if row == matrix.len() {
      *result = polynomial_add(result, &polynomial_scale(product, permutation_sign));
      return;
    }
    for column in 0..matrix.len() {
      if columns.contains(&column) {
        continue;
      }
      let inversions = columns
        .iter()
        .filter(|previous| **previous > column)
        .count();
      let next_sign = if inversions % 2 == 0 {
        permutation_sign
      } else {
        -permutation_sign
      };
      columns.push(column);
      let next_product = polynomial_multiply(product, &matrix[row][column]);
      expand(matrix, row + 1, columns, &next_product, next_sign, result);
      columns.pop();
    }
  }

  if matrix.is_empty() {
    return vec![1.0];
  }
  let mut result = vec![0.0];
  expand(
    matrix,
    0,
    &mut Vec::with_capacity(matrix.len()),
    &[1.0],
    1.0,
    &mut result,
  );
  result
}

fn polynomial_derivative(polynomial: &[f64]) -> Polynomial {
  polynomial
    .iter()
    .enumerate()
    .skip(1)
    .map(|(power, coefficient)| *coefficient * power as f64)
    .collect()
}

fn polynomial_value(polynomial: &[f64], point: f64) -> f64 {
  polynomial
    .iter()
    .rev()
    .fold(0.0, |value, coefficient| value * point + coefficient)
}

fn polynomial_value_scale(polynomial: &[f64], point: f64) -> f64 {
  polynomial.iter().rev().fold(0.0, |value, coefficient| {
    value * point.abs() + coefficient.abs()
  })
}

fn polynomial_residual_scale(polynomial: &[f64], point: f64) -> f64 {
  polynomial_value_scale(polynomial, point)
    .max(polynomial_value_scale(polynomial, point.abs().max(1.0)))
}

fn sign(value: f64) -> i8 {
  if value < 0.0 {
    -1
  } else if value > 0.0 {
    1
  } else {
    0
  }
}

fn near_zero(polynomial: &[f64], point: f64, value: f64) -> bool {
  let scale = polynomial_value_scale(polynomial, point);
  let evaluation_floor =
    64.0 * f64::EPSILON * polynomial_value_scale(polynomial, point.abs().max(1.0));
  scale == 0.0 || value.abs() <= 2e-12 * scale || value.abs() <= evaluation_floor
}

fn scaled_near_zero(polynomial: &[f64], point: f64, value: f64) -> bool {
  let scale = polynomial_value_scale(polynomial, point);
  // Compare against terms at this point. A unit-interval-wide floor can dwarf
  // low-order values near zero after high-degree variable scaling and invent roots.
  scale == 0.0 || value.abs() <= 2e-12 * scale
}

fn bisect_root(polynomial: &[f64], mut left: f64, mut right: f64) -> Result<f64, StatsError> {
  let mut left_value = polynomial_value(polynomial, left);
  let right_value = polynomial_value(polynomial, right);
  if sign(left_value) == 0 {
    return Ok(left);
  }
  if sign(right_value) == 0 {
    return Ok(right);
  }
  if sign(left_value) == sign(right_value) {
    return Err(invalid("robust AR polynomial root could not be bracketed"));
  }
  for _ in 0..128 {
    let midpoint = left + (right - left) / 2.0;
    if midpoint == left || midpoint == right {
      break;
    }
    let midpoint_value = polynomial_value(polynomial, midpoint);
    if !midpoint_value.is_finite() {
      return Err(invalid("robust AR polynomial evaluation is non-finite"));
    }
    if sign(midpoint_value) == 0 {
      return Ok(midpoint);
    }
    if sign(midpoint_value) == sign(left_value) {
      left = midpoint;
      left_value = midpoint_value;
    } else {
      right = midpoint;
    }
    if right - left <= 8.0 * f64::EPSILON * (1.0 + left.abs().max(right.abs())) {
      break;
    }
  }
  Ok(left + (right - left) / 2.0)
}

/// Isolate every distinct real root of a polynomial on [-1, 1]. Recursive
/// derivative roots partition it into monotone intervals, so no finite grid is
/// used. Repeated roots are retained for the caller to reject or classify;
/// near-tangent critical values that cannot be classified reliably are errors.
fn roots_on_unit_interval(polynomial: &[f64]) -> Result<Vec<f64>, StatsError> {
  roots_on_unit_interval_with_policy(polynomial, false)
}

fn roots_on_unit_interval_with_policy(
  polynomial: &[f64],
  strict_tangencies: bool,
) -> Result<Vec<f64>, StatsError> {
  let mut polynomial = polynomial.to_vec();
  while polynomial.len() > 1 && polynomial.last() == Some(&0.0) {
    polynomial.pop();
  }
  let maximum = polynomial
    .iter()
    .map(|coefficient| coefficient.abs())
    .fold(0.0, f64::max);
  if !maximum.is_finite() {
    return Err(invalid("robust AR polynomial coefficients are non-finite"));
  }
  if maximum == 0.0 || polynomial.len() == 1 {
    return Ok(Vec::new());
  }
  for coefficient in &mut polynomial {
    *coefficient /= maximum;
  }
  if polynomial.len() == 2 {
    let root = -polynomial[0] / polynomial[1];
    return Ok(
      ((-1.0..=1.0).contains(&root) && root.is_finite())
        .then_some(root)
        .into_iter()
        .collect(),
    );
  }

  let critical_points =
    roots_on_unit_interval_with_policy(&polynomial_derivative(&polynomial), strict_tangencies)?;
  let mut partitions = Vec::with_capacity(critical_points.len() + 2);
  partitions.push(-1.0);
  partitions.extend(critical_points.iter().copied());
  partitions.push(1.0);

  let mut roots = Vec::new();
  for point in &critical_points {
    let value = polynomial_value(&polynomial, *point);
    if !value.is_finite() {
      return Err(invalid("robust AR polynomial evaluation is non-finite"));
    }
    if value == 0.0 || (!strict_tangencies && scaled_near_zero(&polynomial, *point, value)) {
      roots.push(*point);
    } else if strict_tangencies
      && value.abs() <= 64.0 * f64::EPSILON * polynomial_residual_scale(&polynomial, *point)
    {
      return Err(invalid(
        "robust AR root count is ambiguous near a stationary point",
      ));
    }
  }
  for pair in partitions.windows(2) {
    let left = pair[0];
    let right = pair[1];
    let left_value = polynomial_value(&polynomial, left);
    let right_value = polynomial_value(&polynomial, right);
    if !left_value.is_finite() || !right_value.is_finite() {
      return Err(invalid("robust AR polynomial evaluation is non-finite"));
    }
    let left_is_root = if strict_tangencies {
      left_value == 0.0
    } else {
      scaled_near_zero(&polynomial, left, left_value)
    };
    let right_is_root = if strict_tangencies {
      right_value == 0.0
    } else {
      scaled_near_zero(&polynomial, right, right_value)
    };
    if (left == -1.0 && left_is_root) || (right == 1.0 && right_is_root) {
      return Err(invalid(
        "robust AR root lies on an unresolved isolation bound",
      ));
    }
    // A monotone section with a zero at one endpoint cannot contain another
    // root in its interior.
    if left_is_root || right_is_root {
      continue;
    }
    if sign(left_value) != sign(right_value) {
      roots.push(bisect_root(&polynomial, left, right)?);
    }
  }
  roots.sort_by(f64::total_cmp);
  let mut distinct = Vec::with_capacity(roots.len());
  for root in roots {
    if distinct.last().is_some_and(|previous: &f64| {
      (root - previous).abs() <= 1e-10 * (1.0 + root.abs().max(previous.abs()))
    }) {
      return Err(invalid(
        "robust AR polynomial roots are too close to distinguish reliably",
      ));
    }
    distinct.push(root);
  }
  Ok(distinct)
}

/// Scale a real polynomial to [-1, 1] with a Fujiwara root bound. Logarithmic
/// coefficient scaling avoids overflow while preserving all finite roots.
fn scaled_polynomial(polynomial: &[f64]) -> Result<(Polynomial, f64), StatsError> {
  let mut polynomial = polynomial.to_vec();
  while polynomial.len() > 1 && polynomial.last() == Some(&0.0) {
    polynomial.pop();
  }
  if polynomial
    .iter()
    .any(|coefficient| !coefficient.is_finite())
  {
    return Err(invalid("robust AR polynomial coefficients are non-finite"));
  }
  let maximum = polynomial
    .iter()
    .map(|coefficient| coefficient.abs())
    .fold(0.0, f64::max);
  if maximum == 0.0 {
    return Ok((vec![0.0], 1.0));
  }
  for coefficient in &mut polynomial {
    *coefficient /= maximum;
  }
  let degree = polynomial.len() - 1;
  if degree == 0 {
    return Ok((polynomial, 1.0));
  }
  let leading = polynomial[degree].abs();
  if leading <= 1e-13 {
    return Err(invalid("robust AR root degree is numerically ambiguous"));
  }
  let leading_log = leading.ln();
  let mut log_bound = f64::NEG_INFINITY;
  for (power, coefficient) in polynomial.iter().enumerate().take(degree) {
    if *coefficient != 0.0 {
      let candidate =
        2.0_f64.ln() + (coefficient.abs().ln() - leading_log) / (degree - power) as f64;
      log_bound = log_bound.max(candidate);
    }
  }
  if log_bound == f64::NEG_INFINITY {
    // A monomial has the repeated real root zero (unless it is linear).
    // Preserve it so callers can report singularity/repeated-root explicitly.
    return Ok((polynomial, 1.0));
  }
  // Degree-fourteen and degree-sixteen boundaries need less expansion to keep
  // low-order terms above the absolute evaluation floor; lower degrees retain
  // the prior scale because projected polynomials share this solver.
  let margin = if matches!(degree, 14 | 16) {
    1.25_f64
  } else {
    2.0_f64
  };
  let log_scale = log_bound + margin.ln();
  if !log_scale.is_finite() || log_scale >= 700.0 {
    return Err(invalid(
      "robust AR confidence-set roots exceed the finite numerical range",
    ));
  }
  let scale = log_scale.exp();
  let term_logs = polynomial
    .iter()
    .enumerate()
    .filter(|(_, coefficient)| **coefficient != 0.0)
    .map(|(power, coefficient)| coefficient.abs().ln() + power as f64 * log_scale)
    .collect::<Vec<_>>();
  let maximum_log = term_logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
  if term_logs
    .iter()
    .any(|term_log| maximum_log - term_log > 650.0)
  {
    return Err(invalid(
      "robust AR confidence-set roots exceed polynomial scaling resolution",
    ));
  }
  let scaled = polynomial
    .iter()
    .enumerate()
    .map(|(power, coefficient)| {
      if *coefficient == 0.0 {
        0.0
      } else {
        coefficient.signum()
          * (coefficient.abs().ln() + power as f64 * log_scale - maximum_log).exp()
      }
    })
    .collect::<Vec<_>>();
  Ok((scaled, scale))
}

fn real_polynomial_roots(
  polynomial: &[f64],
  require_simple_roots: bool,
) -> Result<Vec<f64>, StatsError> {
  real_polynomial_roots_with_policy(polynomial, require_simple_roots, false)
}

fn projected_real_polynomial_roots(
  polynomial: &[f64],
  require_simple_roots: bool,
) -> Result<Vec<f64>, StatsError> {
  real_polynomial_roots_with_policy(polynomial, require_simple_roots, true)
}

fn real_polynomial_roots_with_policy(
  polynomial: &[f64],
  require_simple_roots: bool,
  strict_tangencies: bool,
) -> Result<Vec<f64>, StatsError> {
  let (scaled, scale) = scaled_polynomial(polynomial)?;
  let roots = roots_on_unit_interval_with_policy(&scaled, strict_tangencies)?;
  if require_simple_roots {
    let derivative = polynomial_derivative(&scaled);
    for root in &roots {
      let slope = polynomial_value(&derivative, *root);
      if !slope.is_finite() || near_zero(&derivative, *root, slope) {
        return Err(invalid(
          "robust AR boundary has a repeated or numerically ambiguous real root",
        ));
      }
    }
  }
  roots
    .into_iter()
    .map(|root| {
      let root = root * scale;
      root
        .is_finite()
        .then_some(root)
        .ok_or_else(|| invalid("robust AR confidence-set endpoint is non-finite"))
    })
    .collect()
}

fn confidence_set_from_boundary(
  polynomial: &[f64],
  center: f64,
) -> Result<(ArConfidenceSetShape, Vec<ArConfidenceInterval>), StatsError> {
  let maximum = polynomial
    .iter()
    .map(|coefficient| coefficient.abs())
    .fold(0.0, f64::max);
  if !maximum.is_finite() {
    return Err(invalid("robust AR boundary coefficients are non-finite"));
  }
  if maximum == 0.0 {
    return Ok((ArConfidenceSetShape::AllReal, vec![interval(None, None)]));
  }
  let normalized = polynomial_scale(polynomial, 1.0 / maximum);
  let (scaled, variable_scale) = scaled_polynomial(&normalized)?;
  let roots_in_scale = roots_on_unit_interval(&scaled)?;
  if roots_in_scale.len() > 16 {
    return Err(invalid(
      "robust AR boundary has more than sixteen real roots",
    ));
  }
  let derivative = polynomial_derivative(&scaled);
  let mut roots = Vec::with_capacity(roots_in_scale.len());
  for root in &roots_in_scale {
    let value = polynomial_value(&scaled, *root);
    let slope = polynomial_value(&derivative, *root);
    if !value.is_finite() || !scaled_near_zero(&scaled, *root, value) {
      return Err(invalid(
        "robust AR boundary root residual exceeds numerical tolerance",
      ));
    }
    if !slope.is_finite() || scaled_near_zero(&derivative, *root, slope) {
      return Err(invalid(
        "robust AR boundary has a repeated or numerically ambiguous real root",
      ));
    }
    let root = root * variable_scale;
    if !root.is_finite() {
      return Err(invalid("robust AR confidence-set endpoint is non-finite"));
    }
    roots.push(root);
  }
  let left_tail = polynomial_value(&scaled, -1.0);
  if !left_tail.is_finite() || scaled_near_zero(&scaled, -1.0, left_tail) {
    return Err(invalid("robust AR root bound is numerically ambiguous"));
  }
  let mut sign_to_left = sign(left_tail);
  let mut sign_at_probe = sign_to_left;
  let mut partitions = Vec::with_capacity(roots_in_scale.len() + 2);
  partitions.push(-1.0);
  partitions.extend(roots_in_scale.iter().copied());
  partitions.push(1.0);
  for pair in partitions.windows(2) {
    let probe = pair[0] + (pair[1] - pair[0]) / 2.0;
    let value = polynomial_value(&scaled, probe);
    if !value.is_finite() || scaled_near_zero(&scaled, probe, value) {
      return Err(invalid(
        "robust AR boundary sign is numerically ambiguous between roots",
      ));
    }
    if sign(value) != sign_at_probe {
      return Err(invalid(
        "robust AR root isolation did not preserve boundary signs",
      ));
    }
    sign_at_probe = -sign_at_probe;
  }

  let mut intervals = Vec::new();
  let mut lower = None;
  for root in roots {
    if sign_to_left < 0 {
      intervals.push(interval(lower, Some(root + center)));
    }
    lower = Some(root + center);
    sign_to_left = -sign_to_left;
  }
  if sign_to_left < 0 {
    intervals.push(interval(lower, None));
  }
  if intervals.len() > 9 {
    return Err(invalid(
      "robust AR confidence set has more than nine components",
    ));
  }
  for pair in intervals.windows(2) {
    if pair[0]
      .upper
      .is_none_or(|upper| pair[1].lower.is_none_or(|lower| upper >= lower))
    {
      return Err(invalid(
        "robust AR confidence-set endpoints exceed floating-point resolution",
      ));
    }
  }
  if intervals.iter().any(|ci| {
    ci.lower
      .zip(ci.upper)
      .is_some_and(|(lower, upper)| lower >= upper)
  }) {
    return Err(invalid(
      "robust AR confidence-set endpoints exceed floating-point resolution",
    ));
  }
  let shape = match intervals.as_slice() {
    [] => ArConfidenceSetShape::Empty,
    [only] if only.lower.is_none() && only.upper.is_none() => ArConfidenceSetShape::AllReal,
    [only] if only.lower.is_none() => ArConfidenceSetShape::LowerRay,
    [only] if only.upper.is_none() => ArConfidenceSetShape::UpperRay,
    [_] => ArConfidenceSetShape::Bounded,
    [_, _] => ArConfidenceSetShape::Disjoint,
    _ => ArConfidenceSetShape::Multiple,
  };
  Ok((shape, intervals))
}

fn score_meat_is_well_conditioned(scores: &[ScorePair], delta: f64) -> Result<(), StatsError> {
  let dimension = scores.first().map_or(0, |(score_y, _)| score_y.len());
  let mut meat = vec![vec![0.0; dimension]; dimension];
  for (score_y, score_x) in scores {
    let score = score_y
      .iter()
      .zip(score_x)
      .map(|(value_y, value_x)| value_y - delta * value_x)
      .collect::<Vec<_>>();
    for row in 0..dimension {
      for column in 0..dimension {
        meat[row][column] += score[row] * score[column];
      }
    }
  }
  let diagonal_product = (0..dimension).fold(1.0, |product, index| product * meat[index][index]);
  let determinant = polynomial_matrix_determinant(
    &meat
      .iter()
      .map(|row| row.iter().map(|value| vec![*value]).collect())
      .collect::<Vec<Vec<Polynomial>>>(),
  )[0];
  if dimension == 0
    || meat.iter().flatten().any(|value| !value.is_finite())
    || !determinant.is_finite()
    || !diagonal_product.is_finite()
    || diagonal_product <= 0.0
    || determinant <= 1e-12 * diagonal_product
  {
    return Err(invalid(
      "robust AR score covariance is singular or numerically ill-conditioned",
    ));
  }
  Ok(())
}

fn polynomial_degree(polynomial: &[f64]) -> usize {
  polynomial
    .iter()
    .rposition(|coefficient| *coefficient != 0.0)
    .unwrap_or(0)
}

fn chi_square_critical_value(alpha: f64, degrees_of_freedom: usize) -> Result<f64, StatsError> {
  if degrees_of_freedom == 2 {
    return Ok(-2.0 * alpha.ln());
  }
  let mut low = 0.0;
  let mut high = degrees_of_freedom as f64;
  while crate::distributions::chi_square_tail(high, degrees_of_freedom) > alpha {
    high *= 2.0;
    if !high.is_finite() {
      return Err(invalid(
        "AR critical chi-square exceeds the finite numerical range",
      ));
    }
  }
  for _ in 0..100 {
    let midpoint = low + (high - low) / 2.0;
    if crate::distributions::chi_square_tail(midpoint, degrees_of_freedom) > alpha {
      low = midpoint;
    } else {
      high = midpoint;
    }
  }
  Ok(low + (high - low) / 2.0)
}

fn validate_score_condition_over_real_line(
  scores: &[ScorePair],
  score_covariance: &[Vec<Polynomial>],
  determinant: &[f64],
) -> Result<(), StatsError> {
  let diagonal_product = score_covariance
    .iter()
    .enumerate()
    .fold(vec![1.0], |product, (index, row)| {
      polynomial_multiply(&product, &row[index])
    });
  let determinant_slope = polynomial_derivative(determinant);
  let diagonal_slope = polynomial_derivative(&diagonal_product);
  let mut condition_slope = polynomial_add(
    &polynomial_multiply(&determinant_slope, &diagonal_product),
    &polynomial_scale(&polynomial_multiply(determinant, &diagonal_slope), -1.0),
  );
  let determinant_degree = polynomial_degree(determinant);
  let diagonal_degree = polynomial_degree(&diagonal_product);
  if determinant_degree == diagonal_degree && determinant_degree > 0 {
    // The leading derivative-ratio term cancels algebraically. Remove its
    // floating-point residue so it cannot be mistaken for a huge finite root.
    let canceled_degree = 2 * determinant_degree - 1;
    if let Some(coefficient) = condition_slope.get_mut(canceled_degree) {
      *coefficient = 0.0;
    }
  }
  if diagonal_product
    .iter()
    .chain(&condition_slope)
    .any(|coefficient| !coefficient.is_finite())
  {
    return Err(invalid("robust AR score covariance is non-finite"));
  }
  let critical_points = real_polynomial_roots(&condition_slope, false)?;
  for point in critical_points {
    score_meat_is_well_conditioned(scores, point)?;
  }

  let limiting_ratio = match determinant_degree.cmp(&diagonal_degree) {
    std::cmp::Ordering::Less => 0.0,
    std::cmp::Ordering::Equal => {
      determinant[determinant_degree] / diagonal_product[diagonal_degree]
    }
    std::cmp::Ordering::Greater => {
      return Err(invalid(
        "robust AR score covariance conditioning is numerically ambiguous at infinity",
      ));
    }
  };
  if !limiting_ratio.is_finite() || limiting_ratio <= 1e-12 {
    return Err(invalid(
      "robust AR score covariance becomes ill-conditioned for large null values",
    ));
  }
  if condition_slope
    .iter()
    .all(|coefficient| *coefficient == 0.0)
  {
    score_meat_is_well_conditioned(scores, 0.0)?;
  }
  Ok(())
}

fn confidence_set_probes(intervals: &[ArConfidenceInterval], center: f64) -> (Vec<f64>, Vec<f64>) {
  if intervals.is_empty() {
    let magnitude = center.abs().clamp(1.0, f64::MAX / 20.0) * 10.0;
    return (Vec::new(), vec![-magnitude, 0.0, magnitude]);
  }
  let mut inside = Vec::new();
  for ci in intervals {
    match (ci.lower, ci.upper) {
      (None, None) => {
        let magnitude = center.abs().clamp(1.0, f64::MAX / 20.0) * 10.0;
        inside.extend([-magnitude, center, magnitude]);
      }
      (None, Some(upper)) => {
        let step = 1.0_f64.max(upper.abs() * 0.01);
        inside.push(upper - step);
        let far_step = 10.0_f64.max(upper.abs() * 0.1);
        let far = upper - far_step;
        if far.is_finite() && far < upper {
          inside.push(far);
        }
      }
      (Some(lower), None) => {
        let step = 1.0_f64.max(lower.abs() * 0.01);
        inside.push(lower + step);
        let far_step = 10.0_f64.max(lower.abs() * 0.1);
        let far = lower + far_step;
        if far.is_finite() && far > lower {
          inside.push(far);
        }
      }
      (Some(lower), Some(upper)) => inside.push(lower / 2.0 + upper / 2.0),
    }
  }
  let mut outside = Vec::new();
  if let Some(first) = intervals.first()
    && let Some(lower) = first.lower
  {
    outside.push(lower - 1.0_f64.max(lower.abs() * 0.01));
    let far_step = 10.0_f64.max(lower.abs() * 0.1);
    let far = lower - far_step;
    if far.is_finite() && far < lower {
      outside.push(far);
    }
  }
  for pair in intervals.windows(2) {
    if let (Some(left), Some(right)) = (pair[0].upper, pair[1].lower) {
      outside.push(left / 2.0 + right / 2.0);
    }
  }
  if let Some(last) = intervals.last()
    && let Some(upper) = last.upper
  {
    outside.push(upper + 1.0_f64.max(upper.abs() * 0.01));
    let far_step = 10.0_f64.max(upper.abs() * 0.1);
    let far = upper + far_step;
    if far.is_finite() && far > upper {
      outside.push(far);
    }
  }
  (inside, outside)
}

/// Invert the HC1/one-way CR1 joint AR Wald test for one endogenous regressor
/// and two through eight excluded instruments. The confidence boundary has degree
/// at most twice the instrument count; every real root is
/// isolated by recursive derivative partitioning, without evaluating a finite
/// beta grid.
/// HC1 assumes independent rows. Cluster inference assumes independent groups
/// with many groups and has no few-cluster guarantee.
pub fn robust_multiple_instrument_anderson_rubin_confidence_set(
  problem: &InstrumentalVariablesProblem,
  level: f64,
  covariance_type: &CovarianceType,
) -> Result<RobustMultipleInstrumentAndersonRubinConfidenceSet, StatsError> {
  if matches!(covariance_type, CovarianceType::NonRobust) {
    return Err(invalid(
      "robust AR confidence sets require HC1 or one-way cluster covariance",
    ));
  }
  let instrument_count = problem.instrument_names.len();
  if problem.endogenous_names.len() != 1 || !(2..=8).contains(&instrument_count) {
    return Err(invalid(
      "robust multi-instrument AR confidence sets require one endogenous regressor and two to eight excluded instruments",
    ));
  }
  if !level.is_finite() || level <= 0.0 || level >= 100.0 {
    return Err(invalid(
      "AR confidence level must be finite and strictly between 0 and 100",
    ));
  }
  problem.validate()?;

  let alpha = 1.0 - level / 100.0;
  if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
    return Err(invalid(
      "AR confidence level exceeds floating-point probability resolution",
    ));
  }
  let critical_chi_square = chi_square_critical_value(alpha, instrument_count)?;
  if !critical_chi_square.is_finite() || critical_chi_square <= 0.0 {
    return Err(invalid("AR critical chi-square could not be computed"));
  }

  let n = problem.response.len();
  let auxiliary_parameter_count =
    problem.exogenous_names.len() + usize::from(problem.include_intercept) + instrument_count;
  if n <= auxiliary_parameter_count {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters: auxiliary_parameter_count,
    });
  }
  let auxiliary_residual_df = n - auxiliary_parameter_count;
  let (cluster_group_count, covariance_correction, cluster_groups) = match covariance_type {
    CovarianceType::NonRobust => unreachable!("nonrobust covariance rejected above"),
    CovarianceType::RobustHc1 => (None, n as f64 / auxiliary_residual_df as f64, None),
    CovarianceType::Cluster(variable) => {
      let sample = problem.sample.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(
          "cluster covariance requires an estimation sample with cluster identifiers".into(),
        )
      })?;
      let groups = sample.cluster_groups.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(format!(
          "cluster covariance on variable '{variable}' requires cluster group identifiers"
        ))
      })?;
      if groups.len() != n {
        return Err(StatsError::DimensionMismatch(format!(
          "cluster group identifier count ({}) does not match retained observations ({n})",
          groups.len()
        )));
      }
      let count = groups.iter().collect::<BTreeSet<_>>().len();
      if count < 2 {
        return Err(StatsError::InsufficientObservations(
          "cluster covariance requires at least 2 distinct clusters".into(),
        ));
      }
      let group_count_f = count as f64;
      let correction =
        (group_count_f / (group_count_f - 1.0)) * ((n as f64 - 1.0) / auxiliary_residual_df as f64);
      (Some(count), correction, Some(groups))
    }
  };
  if !covariance_correction.is_finite() || covariance_correction <= 0.0 {
    return Err(invalid("AR covariance correction is non-finite"));
  }

  let x = problem
    .endogenous
    .iter()
    .map(|row| row[0])
    .collect::<Vec<_>>();
  let control_names = problem.exogenous_names.clone();
  let mut orthonormal_instruments = Vec::<Vec<f64>>::with_capacity(instrument_count);
  for column in 0..instrument_count {
    let instrument = problem
      .instruments
      .iter()
      .map(|row| row[column])
      .collect::<Vec<_>>();
    let residualized = residualize(
      &instrument,
      &problem.exogenous,
      &control_names,
      problem.include_intercept,
    )?;
    let norm = dot(&residualized, &residualized);
    if !norm.is_finite() || norm <= 0.0 {
      return Err(invalid(
        "robust AR confidence-set auxiliary design is rank deficient or non-finite",
      ));
    }
    let mut basis = residualized
      .iter()
      .map(|value| value / norm.sqrt())
      .collect::<Vec<_>>();
    for _ in 0..2 {
      for previous in &orthonormal_instruments {
        let projection = dot(&basis, previous);
        for (value, previous_value) in basis.iter_mut().zip(previous) {
          *value -= projection * previous_value;
        }
      }
    }
    let orthogonal_norm = dot(&basis, &basis);
    if !orthogonal_norm.is_finite() || orthogonal_norm <= 1e-12 {
      return Err(invalid(
        "robust AR confidence-set excluded instruments are rank deficient after controls",
      ));
    }
    let scale = orthogonal_norm.sqrt();
    orthonormal_instruments.push(basis.into_iter().map(|value| value / scale).collect());
  }

  let residual_y = residualize(
    &problem.response,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
  let residual_x = residualize(
    &x,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
  let g_y = orthonormal_instruments
    .iter()
    .map(|basis| dot(basis, &residual_y))
    .collect::<Vec<_>>();
  let g_x = orthonormal_instruments
    .iter()
    .map(|basis| dot(basis, &residual_x))
    .collect::<Vec<_>>();
  if g_y.iter().chain(&g_x).any(|value| !value.is_finite()) {
    return Err(invalid("robust AR confidence-set moments are non-finite"));
  }
  let norm_gx = dot(&g_x, &g_x);
  let center = if norm_gx > 0.0 {
    dot(&g_y, &g_x) / norm_gx
  } else {
    0.0
  };
  if !center.is_finite() {
    return Err(invalid("robust AR confidence-set centering is non-finite"));
  }

  let mut auxiliary_names = control_names.clone();
  auxiliary_names.extend(problem.instrument_names.iter().cloned());
  let auxiliary_design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(controls, instruments)| {
      controls
        .iter()
        .chain(instruments)
        .copied()
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let residual_y_full = residualize(
    &problem.response,
    &auxiliary_design,
    &auxiliary_names,
    problem.include_intercept,
  )?;
  let residual_x_full = residualize(
    &x,
    &auxiliary_design,
    &auxiliary_names,
    problem.include_intercept,
  )?;
  let response_scale = dot(&problem.response, &problem.response);
  let endogenous_scale = dot(&x, &x);
  let response_innovation = dot(&residual_y_full, &residual_y_full);
  let endogenous_innovation = dot(&residual_x_full, &residual_x_full);
  if [
    response_scale,
    endogenous_scale,
    response_innovation,
    endogenous_innovation,
  ]
  .iter()
  .any(|value| !value.is_finite())
  {
    return Err(invalid(
      "robust AR auxiliary residual variance is non-finite",
    ));
  }
  let response_is_exact = response_scale == 0.0 || response_innovation <= 1e-24 * response_scale;
  let endogenous_is_exact =
    endogenous_scale == 0.0 || endogenous_innovation <= 1e-24 * endogenous_scale;
  if response_is_exact && endogenous_is_exact {
    return Err(invalid(
      "robust AR auxiliary score covariance is numerically zero for every null",
    ));
  }

  let score_pairs = if let Some(groups) = cluster_groups {
    let mut totals = BTreeMap::<&str, ScorePair>::new();
    for (index, group) in groups.iter().enumerate() {
      let entry = totals
        .entry(group.as_str())
        .or_insert_with(|| (vec![0.0; instrument_count], vec![0.0; instrument_count]));
      for (column, basis) in orthonormal_instruments.iter().enumerate() {
        entry.0[column] += basis[index] * residual_y_full[index];
        entry.1[column] += basis[index] * residual_x_full[index];
      }
    }
    totals.into_values().collect::<Vec<_>>()
  } else {
    (0..n)
      .map(|index| {
        (
          orthonormal_instruments
            .iter()
            .map(|basis| basis[index] * residual_y_full[index])
            .collect(),
          orthonormal_instruments
            .iter()
            .map(|basis| basis[index] * residual_x_full[index])
            .collect(),
        )
      })
      .collect::<Vec<_>>()
  };
  if score_pairs
    .iter()
    .flat_map(|(score_y, score_x)| score_y.iter().chain(score_x))
    .any(|value| !value.is_finite())
  {
    return Err(invalid("robust AR confidence-set scores are non-finite"));
  }

  let center_g = g_y
    .iter()
    .zip(&g_x)
    .map(|(value_y, value_x)| value_y - center * value_x)
    .collect::<Vec<_>>();
  let mut s0 = vec![vec![0.0; instrument_count]; instrument_count];
  let mut s1 = vec![vec![0.0; instrument_count]; instrument_count];
  let mut s2 = vec![vec![0.0; instrument_count]; instrument_count];
  let mut centered_scores = Vec::with_capacity(score_pairs.len());
  for (score_y, score_x) in &score_pairs {
    let score_center = score_y
      .iter()
      .zip(score_x)
      .map(|(value_y, value_x)| value_y - center * value_x)
      .collect::<Vec<_>>();
    centered_scores.push((score_center.clone(), score_x.clone()));
    for row in 0..instrument_count {
      for column in 0..instrument_count {
        s0[row][column] += score_center[row] * score_center[column];
        s1[row][column] -=
          score_center[row] * score_x[column] + score_x[row] * score_center[column];
        s2[row][column] += score_x[row] * score_x[column];
      }
    }
  }
  if s0
    .iter()
    .chain(&s1)
    .chain(&s2)
    .flatten()
    .any(|value| !value.is_finite())
  {
    return Err(invalid("robust AR score covariance is non-finite"));
  }
  let score_covariance = (0..instrument_count)
    .map(|row| {
      (0..instrument_count)
        .map(|column| vec![s0[row][column], s1[row][column], s2[row][column]])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let determinant = polynomial_matrix_determinant(&score_covariance);
  if determinant
    .iter()
    .any(|coefficient| !coefficient.is_finite())
  {
    return Err(invalid("robust AR score covariance is non-finite"));
  }
  let determinant_roots = real_polynomial_roots(&determinant, false)?;
  if !determinant_roots.is_empty() {
    return Err(invalid(
      "robust AR score covariance becomes singular for a real null value",
    ));
  }
  validate_score_condition_over_real_line(&centered_scores, &score_covariance, &determinant)?;

  let g_polynomials = center_g
    .iter()
    .zip(&g_x)
    .map(|(value, slope)| vec![*value, -*slope])
    .collect::<Vec<_>>();
  let numerator = if instrument_count == 2 {
    let g1 = &g_polynomials[0];
    let g2 = &g_polynomials[1];
    let s11 = &score_covariance[0][0];
    let s12 = &score_covariance[0][1];
    let s22 = &score_covariance[1][1];
    polynomial_add(
      &polynomial_add(
        &polynomial_multiply(&polynomial_multiply(g1, g1), s22),
        &polynomial_scale(
          &polynomial_multiply(&polynomial_multiply(g1, g2), s12),
          -2.0,
        ),
      ),
      &polynomial_multiply(&polynomial_multiply(g2, g2), s11),
    )
  } else {
    let mut block = vec![vec![vec![0.0]; instrument_count + 1]; instrument_count + 1];
    for row in 0..instrument_count {
      for column in 0..instrument_count {
        block[row][column] = score_covariance[row][column].clone();
      }
      block[row][instrument_count] = g_polynomials[row].clone();
      block[instrument_count][row] = g_polynomials[row].clone();
    }
    polynomial_scale(&polynomial_matrix_determinant(&block), -1.0)
  };
  let boundary = polynomial_add(
    &numerator,
    &polynomial_scale(&determinant, -critical_chi_square * covariance_correction),
  );
  if boundary.iter().any(|coefficient| !coefficient.is_finite()) {
    return Err(invalid("robust AR boundary coefficients are non-finite"));
  }
  let (shape, mut intervals) = confidence_set_from_boundary(&boundary, center)?;
  let (inside_probes, outside_probes) = confidence_set_probes(&intervals, center);
  for (beta, expected_included) in inside_probes
    .into_iter()
    .map(|beta| (beta, true))
    .chain(outside_probes.into_iter().map(|beta| (beta, false)))
  {
    if !beta.is_finite() {
      return Err(invalid("robust AR region probe is non-finite"));
    }
    let statistic = crate::robust_anderson_rubin_test(problem, &[beta], covariance_type)?;
    let included = statistic.statistic <= critical_chi_square;
    if included != expected_included {
      return Err(invalid(
        "robust AR confidence-set region failed direct Wald validation",
      ));
    }
  }
  for endpoint in intervals
    .iter()
    .flat_map(|ci| [ci.lower, ci.upper])
    .flatten()
  {
    let statistic = crate::robust_anderson_rubin_test(problem, &[endpoint], covariance_type)?;
    if !statistic.statistic.is_finite()
      || (statistic.statistic - critical_chi_square).abs() > 2e-7 * critical_chi_square.max(1.0)
    {
      return Err(invalid(
        "robust AR confidence-set endpoint failed direct Wald validation",
      ));
    }
  }
  for ci in &mut intervals {
    for bound in [&mut ci.lower, &mut ci.upper].into_iter().flatten() {
      if !bound.is_finite() {
        return Err(invalid(
          "robust AR confidence-set endpoint exceeds the finite numerical range",
        ));
      }
    }
  }

  Ok(RobustMultipleInstrumentAndersonRubinConfidenceSet {
    endogenous: problem.endogenous_names[0].clone(),
    excluded_instruments: problem.instrument_names.clone(),
    confidence_level: level,
    observation_count: n,
    auxiliary_parameter_count,
    auxiliary_residual_df,
    covariance_type: covariance_type.clone(),
    covariance_correction,
    cluster_group_count,
    degrees_of_freedom: instrument_count,
    critical_chi_square,
    shape,
    intervals,
  })
}

fn validate_real_polynomial_roots(polynomial: &[f64], roots: &[f64]) -> Result<(), StatsError> {
  let degree = polynomial_degree(polynomial);
  if roots.len() > degree || roots.iter().any(|root| !root.is_finite()) {
    return Err(invalid("robust AR polynomial root count is ambiguous"));
  }
  for root in roots {
    let value = polynomial_value(polynomial, *root);
    let scale = polynomial_residual_scale(polynomial, *root);
    if !value.is_finite() || !scale.is_finite() || (scale > 0.0 && value.abs() > 2e-8 * scale) {
      return Err(invalid("robust AR polynomial root residual is too large"));
    }
  }
  Ok(())
}

fn projection_open_cell_probes(
  lower: Option<f64>,
  upper: Option<f64>,
) -> Result<Vec<f64>, StatsError> {
  let probes = match (lower, upper) {
    (None, None) => vec![-100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0],
    (None, Some(upper)) => {
      let step = 1.0_f64.max(upper.abs());
      vec![
        upper - 0.25 * step,
        upper - step,
        upper - 10.0 * step,
        upper - 100.0 * step,
      ]
    }
    (Some(lower), None) => {
      let step = 1.0_f64.max(lower.abs());
      vec![
        lower + 0.25 * step,
        lower + step,
        lower + 10.0 * step,
        lower + 100.0 * step,
      ]
    }
    (Some(lower), Some(upper)) => {
      let width = upper - lower;
      vec![
        lower + 0.25 * width,
        lower + 0.5 * width,
        lower + 0.75 * width,
      ]
    }
  };
  if probes.iter().any(|probe| !probe.is_finite()) {
    return Err(invalid("robust AR projection tail probe is non-finite"));
  }
  Ok(probes)
}

fn projected_intervals_contain(intervals: &[ArConfidenceInterval], value: f64) -> bool {
  intervals.iter().any(|set| {
    set.lower.is_none_or(|lower| value >= lower) && set.upper.is_none_or(|upper| value <= upper)
  })
}

/// Project the HC1/one-way CR1 joint AR Wald region onto one coefficient for a
/// two-endogenous, two-instrument fit. The projection uses the complete joint
/// chi-square(2) region and does not plug in a nuisance coefficient estimate.
pub fn robust_projected_anderson_rubin_confidence_set(
  problem: &InstrumentalVariablesProblem,
  coefficient: &str,
  level: f64,
  covariance_type: &CovarianceType,
) -> Result<RobustProjectedArConfidenceSet, StatsError> {
  if matches!(covariance_type, CovarianceType::NonRobust) {
    return Err(invalid(
      "robust projected AR confidence sets require HC1 or one-way cluster covariance",
    ));
  }
  if problem.endogenous_names.len() != 2 || problem.instrument_names.len() != 2 {
    return Err(invalid(
      "robust projected AR confidence sets require exactly two endogenous regressors and two excluded instruments",
    ));
  }
  if !level.is_finite() || level <= 0.0 || level >= 100.0 {
    return Err(invalid(
      "AR confidence level must be finite and strictly between 0 and 100",
    ));
  }
  problem.validate()?;
  let Some(target_index) = problem
    .endogenous_names
    .iter()
    .position(|name| name == coefficient)
  else {
    return Err(invalid(
      "projected AR coefficient must name an endogenous regressor in the retained fit",
    ));
  };
  let nuisance_index = 1 - target_index;
  let coefficient_name = problem.endogenous_names[target_index].clone();
  let nuisance_name = problem.endogenous_names[nuisance_index].clone();
  let n = problem.response.len();
  let auxiliary_parameter_count = problem.exogenous_names.len()
    + usize::from(problem.include_intercept)
    + problem.instrument_names.len();
  if n <= auxiliary_parameter_count {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters: auxiliary_parameter_count,
    });
  }
  let auxiliary_residual_df = n - auxiliary_parameter_count;
  let (cluster_group_count, covariance_correction, cluster_groups) = match covariance_type {
    CovarianceType::NonRobust => unreachable!("nonrobust covariance rejected above"),
    CovarianceType::RobustHc1 => (None, n as f64 / auxiliary_residual_df as f64, None),
    CovarianceType::Cluster(variable) => {
      let sample = problem.sample.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(
          "cluster covariance requires an estimation sample with cluster identifiers".into(),
        )
      })?;
      let groups = sample.cluster_groups.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(format!(
          "cluster covariance on variable '{variable}' requires cluster group identifiers"
        ))
      })?;
      if groups.len() != n {
        return Err(StatsError::DimensionMismatch(format!(
          "cluster group identifier count ({}) does not match retained observations ({n})",
          groups.len()
        )));
      }
      let count = groups.iter().collect::<BTreeSet<_>>().len();
      if count < 3 {
        return Err(StatsError::InsufficientObservations(
          "cluster robust AR projection requires at least three groups for two restrictions".into(),
        ));
      }
      let group_count = count as f64;
      let correction =
        (group_count / (group_count - 1.0)) * ((n as f64 - 1.0) / auxiliary_residual_df as f64);
      (Some(count), correction, Some(groups))
    }
  };
  if !covariance_correction.is_finite() || covariance_correction <= 0.0 {
    return Err(invalid("AR covariance correction is non-finite"));
  }
  let alpha = 1.0 - level / 100.0;
  if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
    return Err(invalid(
      "AR confidence level exceeds floating-point probability resolution",
    ));
  }
  let critical_chi_square = chi_square_critical_value(alpha, 2)?;
  if !critical_chi_square.is_finite() || critical_chi_square <= 0.0 {
    return Err(invalid("AR critical chi-square could not be computed"));
  }
  let make_result = |shape: ArConfidenceSetShape,
                     intervals: Vec<ArConfidenceInterval>,
                     fallback: bool|
   -> RobustProjectedArConfidenceSet {
    RobustProjectedArConfidenceSet {
      coefficient: coefficient_name.clone(),
      nuisance_endogenous: vec![nuisance_name.clone()],
      excluded_instruments: problem.instrument_names.clone(),
      confidence_level: level,
      observation_count: n,
      auxiliary_parameter_count,
      auxiliary_residual_df,
      covariance_type: covariance_type.clone(),
      covariance_correction,
      cluster_group_count,
      degrees_of_freedom: 2,
      critical_chi_square,
      shape,
      intervals,
      closure_conservative: true,
      all_real_conservative_fallback: fallback,
    }
  };
  let fallback = || {
    make_result(
      ArConfidenceSetShape::AllReal,
      vec![interval(None, None)],
      true,
    )
  };

  // Orthogonalizing the residualized instruments is an invertible change of
  // basis for the two excluded-instrument restrictions and improves scaling.
  let control_names = problem.exogenous_names.clone();
  let mut orthonormal_instruments = Vec::<Vec<f64>>::with_capacity(2);
  for column in 0..2 {
    let instrument = problem
      .instruments
      .iter()
      .map(|row| row[column])
      .collect::<Vec<_>>();
    let residualized = residualize(
      &instrument,
      &problem.exogenous,
      &control_names,
      problem.include_intercept,
    )?;
    let norm = dot(&residualized, &residualized);
    if !norm.is_finite() || norm <= 0.0 {
      return Err(invalid(
        "robust AR projection auxiliary design is rank deficient or non-finite",
      ));
    }
    let mut basis = residualized
      .iter()
      .map(|value| value / norm.sqrt())
      .collect::<Vec<_>>();
    for _ in 0..2 {
      for previous in &orthonormal_instruments {
        let projection = dot(&basis, previous);
        for (value, previous_value) in basis.iter_mut().zip(previous) {
          *value -= projection * previous_value;
        }
      }
    }
    let orthogonal_norm = dot(&basis, &basis);
    if !orthogonal_norm.is_finite() || orthogonal_norm <= 1e-12 {
      return Err(invalid(
        "robust AR projection excluded instruments are rank deficient after controls",
      ));
    }
    let scale = orthogonal_norm.sqrt();
    orthonormal_instruments.push(basis.into_iter().map(|value| value / scale).collect());
  }

  let residual_y = residualize(
    &problem.response,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
  let target_x = problem
    .endogenous
    .iter()
    .map(|row| row[target_index])
    .collect::<Vec<_>>();
  let nuisance_x = problem
    .endogenous
    .iter()
    .map(|row| row[nuisance_index])
    .collect::<Vec<_>>();
  let residual_target_x = residualize(
    &target_x,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
  let residual_nuisance_x = residualize(
    &nuisance_x,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
  let g_y = orthonormal_instruments
    .iter()
    .map(|basis| dot(basis, &residual_y))
    .collect::<Vec<_>>();
  let g_target = orthonormal_instruments
    .iter()
    .map(|basis| dot(basis, &residual_target_x))
    .collect::<Vec<_>>();
  let g_nuisance = orthonormal_instruments
    .iter()
    .map(|basis| dot(basis, &residual_nuisance_x))
    .collect::<Vec<_>>();
  if g_y
    .iter()
    .chain(&g_target)
    .chain(&g_nuisance)
    .any(|value| !value.is_finite())
  {
    return Err(invalid("robust AR projection moments are non-finite"));
  }
  // A 2SLS-centered coordinate system reduces polynomial cancellation. If the
  // first-stage moment matrix is singular, zero remains a valid arithmetic
  // center and the confidence-set calculation continues without a plug-in.
  let gram_11 = dot(&g_target, &g_target);
  let gram_12 = dot(&g_target, &g_nuisance);
  let gram_22 = dot(&g_nuisance, &g_nuisance);
  let rhs_1 = dot(&g_target, &g_y);
  let rhs_2 = dot(&g_nuisance, &g_y);
  let gram_scale = gram_11.abs().max(gram_12.abs()).max(gram_22.abs());
  let (center_target, center_nuisance) = if gram_scale > 0.0 && gram_scale.is_finite() {
    let a = gram_11 / gram_scale;
    let b = gram_12 / gram_scale;
    let d = gram_22 / gram_scale;
    let r1 = rhs_1 / gram_scale;
    let r2 = rhs_2 / gram_scale;
    let determinant = a * d - b * b;
    if determinant.is_finite() && determinant > 1e-12 * (a * d).abs().max(b * b) {
      (
        (r1 * d - b * r2) / determinant,
        (a * r2 - b * r1) / determinant,
      )
    } else {
      (0.0, 0.0)
    }
  } else {
    (0.0, 0.0)
  };
  if !center_target.is_finite() || !center_nuisance.is_finite() {
    return Err(invalid("robust AR projection center is non-finite"));
  }

  let mut auxiliary_names = control_names.clone();
  auxiliary_names.extend(problem.instrument_names.iter().cloned());
  let auxiliary_design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(controls, instruments)| {
      controls
        .iter()
        .chain(instruments)
        .copied()
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let residual_y_full = residualize(
    &problem.response,
    &auxiliary_design,
    &auxiliary_names,
    problem.include_intercept,
  )?;
  let residual_target_full = residualize(
    &target_x,
    &auxiliary_design,
    &auxiliary_names,
    problem.include_intercept,
  )?;
  let residual_nuisance_full = residualize(
    &nuisance_x,
    &auxiliary_design,
    &auxiliary_names,
    problem.include_intercept,
  )?;
  let response_scale = dot(&problem.response, &problem.response);
  let target_scale = dot(&target_x, &target_x);
  let nuisance_scale = dot(&nuisance_x, &nuisance_x);
  let response_innovation = dot(&residual_y_full, &residual_y_full);
  let target_innovation = dot(&residual_target_full, &residual_target_full);
  let nuisance_innovation = dot(&residual_nuisance_full, &residual_nuisance_full);
  if [
    response_scale,
    target_scale,
    nuisance_scale,
    response_innovation,
    target_innovation,
    nuisance_innovation,
  ]
  .iter()
  .any(|value| !value.is_finite())
  {
    return Ok(fallback());
  }
  let response_is_exact = response_scale == 0.0 || response_innovation <= 1e-24 * response_scale;
  let target_is_exact = target_scale == 0.0 || target_innovation <= 1e-24 * target_scale;
  let nuisance_is_exact = nuisance_scale == 0.0 || nuisance_innovation <= 1e-24 * nuisance_scale;
  if response_is_exact && target_is_exact && nuisance_is_exact {
    return Err(invalid(
      "robust AR projection score covariance is numerically zero throughout the null plane",
    ));
  }
  let mut row_scores = Vec::with_capacity(n);
  for row in 0..n {
    let mut scores = [[0.0; 3]; 2];
    for instrument in 0..2 {
      let basis_value = orthonormal_instruments[instrument][row];
      let score_y = basis_value * residual_y_full[row];
      let score_target = basis_value * residual_target_full[row];
      let score_nuisance = basis_value * residual_nuisance_full[row];
      scores[instrument] = [
        score_y - center_target * score_target - center_nuisance * score_nuisance,
        -score_target,
        -score_nuisance,
      ];
    }
    row_scores.push(scores);
  }
  let score_groups = if let Some(groups) = cluster_groups {
    let mut totals = BTreeMap::<&str, [[f64; 3]; 2]>::new();
    for (index, group) in groups.iter().enumerate() {
      let score = totals.entry(group.as_str()).or_insert([[0.0; 3]; 2]);
      for instrument in 0..2 {
        for coefficient in 0..3 {
          score[instrument][coefficient] += row_scores[index][instrument][coefficient];
        }
      }
    }
    totals.into_values().collect::<Vec<_>>()
  } else {
    row_scores
  };
  if score_groups
    .iter()
    .flatten()
    .flatten()
    .any(|value| !value.is_finite())
  {
    return Ok(fallback());
  }
  let score_polynomial =
    |score: [f64; 3]| -> BivariatePolynomial { vec![vec![score[0], score[1]], vec![score[2]]] };
  let mut score_covariance = [
    [vec![vec![0.0]], vec![vec![0.0]]],
    [vec![vec![0.0]], vec![vec![0.0]]],
  ];
  for group in &score_groups {
    let first = score_polynomial(group[0]);
    let second = score_polynomial(group[1]);
    score_covariance[0][0] =
      bivariate_add(&score_covariance[0][0], &bivariate_multiply(&first, &first));
    score_covariance[0][1] = bivariate_add(
      &score_covariance[0][1],
      &bivariate_multiply(&first, &second),
    );
    score_covariance[1][1] = bivariate_add(
      &score_covariance[1][1],
      &bivariate_multiply(&second, &second),
    );
  }
  score_covariance[1][0] = score_covariance[0][1].clone();
  let determinant = bivariate_add(
    &bivariate_multiply(&score_covariance[0][0], &score_covariance[1][1]),
    &bivariate_scale(
      &bivariate_multiply(&score_covariance[0][1], &score_covariance[0][1]),
      -1.0,
    ),
  );
  let bivariate_maximum = |polynomial: &[Polynomial]| {
    polynomial
      .iter()
      .flatten()
      .map(|value| value.abs())
      .fold(0.0, f64::max)
  };
  let determinant_scale = (bivariate_maximum(&score_covariance[0][0])
    * bivariate_maximum(&score_covariance[1][1]))
  .max(bivariate_maximum(&score_covariance[0][1]).powi(2));
  let determinant_maximum = bivariate_maximum(&determinant);
  if !determinant_scale.is_finite() || !determinant_maximum.is_finite() {
    return Ok(fallback());
  }
  if determinant_scale == 0.0 || determinant_maximum <= 1e-12 * determinant_scale {
    return Err(invalid(
      "robust AR projection score covariance is singular throughout the null plane",
    ));
  }
  let moment_polynomial = |moment_y: f64, moment_target: f64, moment_nuisance: f64| {
    vec![
      vec![
        moment_y - center_target * moment_target - center_nuisance * moment_nuisance,
        -moment_target,
      ],
      vec![-moment_nuisance],
    ]
  };
  let moment_1 = moment_polynomial(g_y[0], g_target[0], g_nuisance[0]);
  let moment_2 = moment_polynomial(g_y[1], g_target[1], g_nuisance[1]);
  let numerator = bivariate_add(
    &bivariate_add(
      &bivariate_multiply(
        &score_covariance[1][1],
        &bivariate_multiply(&moment_1, &moment_1),
      ),
      &bivariate_scale(
        &bivariate_multiply(
          &score_covariance[0][1],
          &bivariate_multiply(&moment_1, &moment_2),
        ),
        -2.0,
      ),
    ),
    &bivariate_multiply(
      &score_covariance[0][0],
      &bivariate_multiply(&moment_2, &moment_2),
    ),
  );
  let boundary = bivariate_add(
    &numerator,
    &bivariate_scale(&determinant, -critical_chi_square * covariance_correction),
  );

  if boundary.iter().flatten().any(|value| !value.is_finite()) {
    return Ok(fallback());
  }
  let (shape, intervals, candidates, resultant_candidates) =
    match projection_interval_set(&boundary, center_target) {
      Ok(result) => result,
      Err(_) => return Ok(fallback()),
    };

  // Check every stationary nuisance value and a finite witness in the open
  // target cells against the direct auxiliary-OLS Wald test. A singular
  // covariance or polynomial disagreement falls back to all real.
  let validate_minimum = |target_offset: f64, minimum: &NuisanceMinimum| -> bool {
    let target = target_offset + center_target;
    if !target.is_finite() {
      return false;
    }
    let evaluated = bivariate_value_at_target(&boundary, target_offset);
    let mut nuisance_points = minimum.stationary_points.clone();
    if !nuisance_points.contains(&minimum.minimizer) {
      nuisance_points.push(minimum.minimizer);
    }
    nuisance_points.into_iter().all(|nuisance_offset| {
      let value = polynomial_value(&evaluated, nuisance_offset);
      let scale = polynomial_value_scale(&evaluated, nuisance_offset);
      let nuisance = nuisance_offset + center_nuisance;
      if !score_covariance_well_conditioned_at(&score_covariance, target_offset, nuisance_offset) {
        return false;
      }
      if !value.is_finite() || !scale.is_finite() || !nuisance.is_finite() {
        return false;
      }
      let null_values = if target_index == 0 {
        [target, nuisance]
      } else {
        [nuisance, target]
      };
      let direct = match crate::robust_anderson_rubin_test(problem, &null_values, covariance_type) {
        Ok(result) if result.statistic.is_finite() => result.statistic,
        _ => return false,
      };
      let expected_included = value <= 0.0;
      let direct_included = direct <= critical_chi_square;
      let polynomial_is_boundary = scale == 0.0 || value.abs() <= 2e-8 * scale;
      let direct_is_boundary =
        (direct - critical_chi_square).abs() <= 2e-7 * critical_chi_square.max(1.0);
      if polynomial_is_boundary {
        direct_is_boundary
      } else {
        direct_included == expected_included
      }
    })
  };

  let mut lower: Option<f64> = None;
  for upper in candidates
    .iter()
    .copied()
    .map(Some)
    .chain(std::iter::once(None))
  {
    let probes = match projection_open_cell_probes(lower, upper) {
      Ok(probes) => probes,
      Err(_) => return Ok(fallback()),
    };
    for target_offset in probes {
      let polynomial = bivariate_value_at_target(&boundary, target_offset);
      let minimum = match nuisance_polynomial_minimum(&polynomial, false) {
        Ok(value) => value,
        Err(_) => return Ok(fallback()),
      };
      if minimum.included != projected_intervals_contain(&intervals, target_offset + center_target)
      {
        return Ok(fallback());
      }
      if !validate_minimum(target_offset, &minimum) {
        return Ok(fallback());
      }
    }
    lower = upper;
  }
  for candidate in candidates.iter().copied() {
    let polynomial = bivariate_value_at_target(&boundary, candidate);
    let minimum = match nuisance_polynomial_minimum(&polynomial, true) {
      Ok(value) => value,
      Err(_) => return Ok(fallback()),
    };
    if minimum.included != projected_intervals_contain(&intervals, candidate + center_target) {
      return Ok(fallback());
    }
    if !validate_minimum(candidate, &minimum) {
      return Ok(fallback());
    }
    if resultant_candidates
      .iter()
      .any(|root| (root - candidate).abs() <= 1e-7 * (1.0 + root.abs().max(candidate.abs())))
      && !resultant_candidate_is_real_boundary(&boundary, candidate).unwrap_or(false)
    {
      return Ok(fallback());
    }
  }

  // Confirm finite reported endpoints against the full robust AR test. Closed
  // endpoints that only arise as limits at infinity are conservatively widened
  // to all real when they cannot be represented by a finite nuisance witness.
  for endpoint in intervals
    .iter()
    .flat_map(|set| [set.lower, set.upper])
    .flatten()
  {
    let target_offset = endpoint - center_target;
    let polynomial = bivariate_value_at_target(&boundary, target_offset);
    let minimum = match nuisance_polynomial_minimum(&polynomial, true) {
      Ok(value) => value,
      Err(_) => return Ok(fallback()),
    };
    let mut boundary_witnesses = minimum.stationary_points.clone();
    boundary_witnesses.push(minimum.minimizer);
    let has_boundary_witness = boundary_witnesses.iter().any(|nuisance| {
      let value = polynomial_value(&polynomial, *nuisance);
      let scale = polynomial_value_scale(&polynomial, *nuisance);
      scale == 0.0 || (scale.is_finite() && value.abs() <= 2e-8 * scale)
    });
    if !validate_minimum(target_offset, &minimum) || !has_boundary_witness {
      return Ok(fallback());
    }
  }
  Ok(make_result(shape, intervals, false))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn polynomial_with_roots(roots: &[f64], leading: f64) -> Polynomial {
    roots.iter().fold(vec![leading], |polynomial, root| {
      polynomial_multiply(&polynomial, &[-root, 1.0])
    })
  }

  fn assert_intervals_close(actual: &[ArConfidenceInterval], expected: &[ArConfidenceInterval]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
      for (actual, expected) in [
        (actual.lower, expected.lower),
        (actual.upper, expected.upper),
      ] {
        match (actual, expected) {
          (None, None) => {}
          (Some(actual), Some(expected)) => {
            assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
          }
          mismatch => panic!("interval endpoint mismatch: {mismatch:?}"),
        }
      }
    }
  }

  #[test]
  fn boundary_signs_cover_empty_bounded_disjoint_and_three_component_sets() {
    let (shape, intervals) = confidence_set_from_boundary(&[1.0], 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Empty);
    assert!(intervals.is_empty());

    let (shape, intervals) = confidence_set_from_boundary(&[-1.0], 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::AllReal);
    assert_eq!(intervals, [interval(None, None)]);

    let (shape, intervals) =
      confidence_set_from_boundary(&[-1.0, 0.0, 1.0, 0.0, 0.0], 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Bounded);
    assert_intervals_close(&intervals, &[interval(Some(-1.0), Some(1.0))]);

    let roots = [-2.0, -1.0, 1.0, 2.0];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, 1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Disjoint);
    assert_intervals_close(
      &intervals,
      &[
        interval(Some(-2.0), Some(-1.0)),
        interval(Some(1.0), Some(2.0)),
      ],
    );

    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-2.0)),
        interval(Some(-1.0), Some(1.0)),
        interval(Some(2.0), None),
      ],
    );
  }

  #[test]
  fn degree_six_boundary_supports_six_roots_and_four_components() {
    let roots = [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-3.0)),
        interval(Some(-2.0), Some(-1.0)),
        interval(Some(1.0), Some(2.0)),
        interval(Some(3.0), None),
      ],
    );
  }

  #[test]
  fn degree_eight_boundary_supports_eight_roots_and_five_components() {
    let roots = [-4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-4.0)),
        interval(Some(-3.0), Some(-2.0)),
        interval(Some(-1.0), Some(1.0)),
        interval(Some(2.0), Some(3.0)),
        interval(Some(4.0), None),
      ],
    );
  }

  #[test]
  fn degree_ten_boundary_supports_ten_roots_and_six_components() {
    let roots = [-5.0, -4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-5.0)),
        interval(Some(-4.0), Some(-3.0)),
        interval(Some(-2.0), Some(-1.0)),
        interval(Some(1.0), Some(2.0)),
        interval(Some(3.0), Some(4.0)),
        interval(Some(5.0), None),
      ],
    );
  }

  #[test]
  fn degree_twelve_boundary_supports_twelve_roots_and_seven_components() {
    let roots = [
      -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0,
    ];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-6.0)),
        interval(Some(-5.0), Some(-4.0)),
        interval(Some(-3.0), Some(-2.0)),
        interval(Some(-1.0), Some(1.0)),
        interval(Some(2.0), Some(3.0)),
        interval(Some(4.0), Some(5.0)),
        interval(Some(6.0), None),
      ],
    );
  }

  #[test]
  fn degree_fourteen_boundary_supports_fourteen_roots_and_eight_components() {
    let roots = [
      -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0,
    ];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-7.0)),
        interval(Some(-6.0), Some(-5.0)),
        interval(Some(-4.0), Some(-3.0)),
        interval(Some(-2.0), Some(-1.0)),
        interval(Some(1.0), Some(2.0)),
        interval(Some(3.0), Some(4.0)),
        interval(Some(5.0), Some(6.0)),
        interval(Some(7.0), None),
      ],
    );
  }

  #[test]
  fn degree_sixteen_boundary_supports_sixteen_roots_and_nine_components() {
    let roots = [
      -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
    ];
    let (shape, intervals) =
      confidence_set_from_boundary(&polynomial_with_roots(&roots, -1.0), 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Multiple);
    assert_intervals_close(
      &intervals,
      &[
        interval(None, Some(-8.0)),
        interval(Some(-7.0), Some(-6.0)),
        interval(Some(-5.0), Some(-4.0)),
        interval(Some(-3.0), Some(-2.0)),
        interval(Some(-1.0), Some(1.0)),
        interval(Some(2.0), Some(3.0)),
        interval(Some(4.0), Some(5.0)),
        interval(Some(6.0), Some(7.0)),
        interval(Some(8.0), None),
      ],
    );
  }

  #[test]
  fn robust_projection_partitions_bounded_disjoint_and_singleton_sets() {
    // (t^2 - 1)^2 + u^2 - .1 <= 0 has two bounded target components.
    let (shape, intervals, _, _) =
      projection_interval_set(&[vec![0.9, 0.0, -2.0, 0.0, 1.0], vec![0.0], vec![1.0]], 0.0)
        .unwrap();
    let inner = (1.0 - 0.1_f64.sqrt()).sqrt();
    let outer = (1.0 + 0.1_f64.sqrt()).sqrt();
    assert_eq!(shape, ArConfidenceSetShape::Disjoint);
    assert_intervals_close(
      &intervals,
      &[
        interval(Some(-outer), Some(-inner)),
        interval(Some(inner), Some(outer)),
      ],
    );

    // t^2 + u^2 <= 0 retains the isolated target value t=0.
    let (shape, intervals, _, _) =
      projection_interval_set(&[vec![0.0, 0.0, 1.0], vec![0.0], vec![1.0]], 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Singleton);
    assert_intervals_close(&intervals, &[interval(Some(0.0), Some(0.0))]);
  }

  #[test]
  fn robust_projection_partitions_nuisance_degree_drop_and_unbounded_tail() {
    // t*u^2 + u + 1 <= 0 is feasible for every t <= 1/4. At t=0,
    // the nuisance degree drops and the set remains feasible along a tail.
    let (shape, intervals, _, _) =
      projection_interval_set(&[vec![1.0], vec![1.0], vec![0.0, 1.0]], 0.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::LowerRay);
    assert_intervals_close(&intervals, &[interval(None, Some(0.25))]);
  }

  #[test]
  fn robust_projection_rejects_spurious_and_ambiguous_resultant_roots() {
    // (u^2 + 1)^2 + t has a real resultant root at t=0 from the
    // complex stationary points u=+/-i, but its real projection boundary is
    // t=-1 at u=0.
    let boundary = [vec![1.0, 1.0], vec![0.0], vec![2.0], vec![0.0], vec![1.0]];
    assert!(!resultant_candidate_is_real_boundary(&boundary, 0.0).unwrap());
    assert!(resultant_candidate_is_real_boundary(&boundary, -1.0).unwrap());

    // The full floating-point resultant also has a repeated real root here;
    // the projection solver detects that root-count ambiguity and fails closed.
    assert!(projection_interval_set(&boundary, 0.0).is_err());
  }

  #[test]
  fn robust_projection_tail_probes_cover_both_directions() {
    let probes = projection_open_cell_probes(None, None).unwrap();
    assert!(probes.iter().any(|probe| *probe <= -100.0));
    assert!(probes.iter().any(|probe| *probe >= 100.0));
    assert!(
      projection_open_cell_probes(Some(2.0), None)
        .unwrap()
        .iter()
        .any(|probe| *probe >= 200.0)
    );
    assert!(
      projection_open_cell_probes(None, Some(-2.0))
        .unwrap()
        .iter()
        .any(|probe| *probe <= -200.0)
    );
  }

  #[test]
  fn narrow_projection_is_resolved_after_scaling_and_raw_tangencies_fail_closed() {
    assert!(roots_on_unit_interval_with_policy(&[-1e-14, 0.0, 1.0], true).is_err());
    assert!(roots_on_unit_interval_with_policy(&[1.0 + 1e-14, -2.0, 1.0], true).is_err());

    // The corresponding narrow projection cannot be classified robustly and
    // therefore fails closed for the public all-real fallback path.
    assert!(projection_interval_set(&[vec![-1e-14, 0.0, 1.0], vec![0.0], vec![1.0]], 0.0).is_err());
  }

  #[test]
  fn repeated_and_nearly_coincident_roots_fail_and_monomials_are_retained() {
    let repeated = polynomial_multiply(&[-1.0, 1.0], &[-1.0, 1.0]);
    assert!(real_polynomial_roots(&repeated, true).is_err());

    let close_roots = polynomial_multiply(&[-1.0, 1.0], &[-1.000_001, 1.0]);
    assert!(real_polynomial_roots(&close_roots, true).is_err());

    let monomial = [0.0, 0.0, 1.0];
    assert_eq!(real_polynomial_roots(&monomial, false).unwrap(), [0.0]);
    assert!(real_polynomial_roots(&monomial, true).is_err());

    let near_tangent = [1.0 + 1e-14, -2.0, 1.0];
    assert!(projected_real_polynomial_roots(&near_tangent, false).is_err());
  }
}
