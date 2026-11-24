use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{CovarianceType, InstrumentalVariablesProblem, StatsError};

use super::{ArConfidenceInterval, ArConfidenceSetShape, interval, invalid, residualize};

/// HC1/one-way CR1 AR confidence set for one endogenous regressor and two,
/// three or four excluded instruments. The joint-null degrees of freedom equal the
/// number of excluded instruments.
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

type Polynomial = Vec<f64>;
type ScorePair = (Vec<f64>, Vec<f64>);

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
/// used. Repeated roots are retained for the caller to reject or classify.
fn roots_on_unit_interval(polynomial: &[f64]) -> Result<Vec<f64>, StatsError> {
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

  let critical_points = roots_on_unit_interval(&polynomial_derivative(&polynomial))?;
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
    if near_zero(&polynomial, *point, value) {
      roots.push(*point);
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
    // A monotone section with a zero at one endpoint cannot contain another
    // root in its interior.
    if near_zero(&polynomial, left, left_value) || near_zero(&polynomial, right, right_value) {
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
  let log_scale = log_bound + 2.0_f64.ln();
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
  let (scaled, scale) = scaled_polynomial(polynomial)?;
  let roots = roots_on_unit_interval(&scaled)?;
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
  if roots_in_scale.len() > 8 {
    return Err(invalid("robust AR boundary has more than eight real roots"));
  }
  let derivative = polynomial_derivative(&scaled);
  let mut roots = Vec::with_capacity(roots_in_scale.len());
  for root in &roots_in_scale {
    let value = polynomial_value(&scaled, *root);
    let slope = polynomial_value(&derivative, *root);
    if !value.is_finite() || !near_zero(&scaled, *root, value) {
      return Err(invalid(
        "robust AR boundary root residual exceeds numerical tolerance",
      ));
    }
    if !slope.is_finite() || near_zero(&derivative, *root, slope) {
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
  if !left_tail.is_finite() || near_zero(&scaled, -1.0, left_tail) {
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
    if !value.is_finite() || near_zero(&scaled, probe, value) {
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
  if intervals.len() > 5 {
    return Err(invalid(
      "robust AR confidence set has more than five components",
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
/// and two, three or four excluded instruments. The confidence boundary has
/// degree at most four, six or eight, respectively; every real root is isolated
/// by recursive derivative partitioning, without evaluating a finite beta grid.
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
  if problem.endogenous_names.len() != 1 || !(2..=4).contains(&instrument_count) {
    return Err(invalid(
      "robust multi-instrument AR confidence sets require one endogenous regressor and two, three or four excluded instruments",
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
  fn repeated_and_nearly_coincident_roots_fail_and_monomials_are_retained() {
    let repeated = polynomial_multiply(&[-1.0, 1.0], &[-1.0, 1.0]);
    assert!(real_polynomial_roots(&repeated, true).is_err());

    let close_roots = polynomial_multiply(&[-1.0, 1.0], &[-1.000_001, 1.0]);
    assert!(real_polynomial_roots(&close_roots, true).is_err());

    let monomial = [0.0, 0.0, 1.0];
    assert_eq!(real_polynomial_roots(&monomial, false).unwrap(), [0.0]);
    assert!(real_polynomial_roots(&monomial, true).is_err());
  }
}
