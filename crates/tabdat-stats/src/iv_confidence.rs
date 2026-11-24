use serde::{Deserialize, Serialize};

use crate::{
  CovarianceType, EstimationProblem, InstrumentalVariablesProblem, IvOptions, StatsError,
  f_distribution_survival_probability, fit_least_squares, fit_two_stage_least_squares,
  normal_quantile,
};

/// Shape of the scalar AR nonrejection set; rays include their finite endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArConfidenceSetShape {
  Bounded,
  Disjoint,
  Multiple,
  AllReal,
  Empty,
  LowerRay,
  UpperRay,
  Singleton,
}

/// Closed finite endpoints; absent lower/upper bounds mean negative/positive infinity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArConfidenceInterval {
  pub lower: Option<f64>,
  pub upper: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AndersonRubinConfidenceSet {
  pub endogenous: String,
  pub excluded_instruments: Vec<String>,
  pub confidence_level: f64,
  pub observation_count: usize,
  pub numerator_df: usize,
  pub denominator_df: usize,
  pub critical_f: f64,
  pub shape: ArConfidenceSetShape,
  pub intervals: Vec<ArConfidenceInterval>,
}

/// Projection of a joint homoskedastic AR region onto one endogenous coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedArConfidenceSet {
  pub coefficient: String,
  pub nuisance_endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub confidence_level: f64,
  pub observation_count: usize,
  pub numerator_df: usize,
  pub denominator_df: usize,
  pub critical_f: f64,
  pub shape: ArConfidenceSetShape,
  pub intervals: Vec<ArConfidenceInterval>,
  /// True when the scaled nuisance curvature was not safely positive definite,
  /// forcing the conservative all-real set.
  pub all_real_conservative_fallback: bool,
}

/// HC1/one-way CR1 scalar AR confidence set for a single excluded instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustAndersonRubinConfidenceSet {
  pub endogenous: String,
  pub excluded_instrument: String,
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

mod robust_multi;
pub use robust_multi::{
  RobustMultipleInstrumentAndersonRubinConfidenceSet,
  robust_multiple_instrument_anderson_rubin_confidence_set,
};

fn invalid(message: &str) -> StatsError {
  StatsError::IncompatibleHypothesis(message.into())
}

fn critical_f(alpha: f64, numerator_df: usize, denominator_df: usize) -> Result<f64, StatsError> {
  if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
    return Err(invalid(
      "AR confidence level exceeds floating-point probability resolution",
    ));
  }
  let survival =
    |value| f_distribution_survival_probability(value, numerator_df as f64, denominator_df as f64);
  let mut high = 1.0;
  while survival(high) > alpha {
    high *= 2.0;
    if !high.is_finite() {
      return Err(invalid("AR critical F exceeds the finite numerical range"));
    }
  }
  let mut low = 0.0;
  for _ in 0..100 {
    let mid = low + (high - low) / 2.0;
    let tail = survival(mid);
    if !tail.is_finite() {
      return Err(invalid("AR critical F could not be computed"));
    }
    if tail > alpha {
      low = mid;
    } else {
      high = mid;
    }
  }
  Ok(low + (high - low) / 2.0)
}

fn residualize(
  values: &[f64],
  design: &[Vec<f64>],
  names: &[String],
  intercept: bool,
) -> Result<Vec<f64>, StatsError> {
  if names.is_empty() && !intercept {
    return Ok(values.to_vec());
  }
  Ok(
    fit_least_squares(
      &EstimationProblem::new(
        "ar_ci_auxiliary",
        names.to_vec(),
        values.to_vec(),
        design.to_vec(),
      )?
      .with_intercept(intercept),
    )?
    .residuals,
  )
}

fn interval(lower: Option<f64>, upper: Option<f64>) -> ArConfidenceInterval {
  ArConfidenceInterval { lower, upper }
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
  left.iter().zip(right).map(|(a, b)| a * b).sum()
}

const PROFILE_MIN_SCALED_PIVOT: f64 = 1e-12;

// Cholesky is applied to a matrix standardized to unit diagonal. A small or
// nonpositive pivot declines exact profiling; callers return the conservative all-real set.
fn positive_definite_factor(matrix: &[Vec<f64>]) -> Option<(Vec<f64>, Vec<Vec<f64>>)> {
  let n = matrix.len();
  let scales = matrix
    .iter()
    .enumerate()
    .map(|(index, row)| row[index].sqrt())
    .collect::<Vec<_>>();
  if scales
    .iter()
    .any(|scale| !scale.is_finite() || *scale <= 0.0)
  {
    return None;
  }
  let standardized = matrix
    .iter()
    .enumerate()
    .map(|(i, row)| {
      row
        .iter()
        .enumerate()
        .map(|(j, value)| value / scales[i] / scales[j])
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  if standardized
    .iter()
    .flatten()
    .any(|value| !value.is_finite())
  {
    return None;
  }
  let mut lower = vec![vec![0.0; n]; n];
  for i in 0..n {
    for j in 0..=i {
      let prior = (0..j).map(|k| lower[i][k] * lower[j][k]).sum::<f64>();
      let value = standardized[i][j] - prior;
      if i == j {
        if !value.is_finite() || value <= PROFILE_MIN_SCALED_PIVOT {
          return None;
        }
        lower[i][j] = value.sqrt();
      } else {
        lower[i][j] = value / lower[j][j];
      }
    }
  }
  Some((scales, lower))
}

fn cholesky_solve(lower: &[Vec<f64>], rhs: &[f64]) -> Option<Vec<f64>> {
  let n = lower.len();
  let mut forward = vec![0.0; n];
  for i in 0..n {
    let prior = (0..i).map(|j| lower[i][j] * forward[j]).sum::<f64>();
    forward[i] = (rhs[i] - prior) / lower[i][i];
  }
  let mut solution = vec![0.0; n];
  for i in (0..n).rev() {
    let prior = (i + 1..n).map(|j| lower[j][i] * solution[j]).sum::<f64>();
    solution[i] = (forward[i] - prior) / lower[i][i];
  }
  solution
    .iter()
    .all(|value| value.is_finite())
    .then_some(solution)
}

fn all_real_projection(
  problem: &InstrumentalVariablesProblem,
  coefficient: &str,
  nuisance_endogenous: Vec<String>,
  level: f64,
  df: usize,
  critical: f64,
) -> ProjectedArConfidenceSet {
  ProjectedArConfidenceSet {
    coefficient: coefficient.to_owned(),
    nuisance_endogenous,
    excluded_instruments: problem.instrument_names.clone(),
    confidence_level: level,
    observation_count: problem.response.len(),
    numerator_df: problem.instrument_names.len(),
    denominator_df: df,
    critical_f: critical,
    shape: ArConfidenceSetShape::AllReal,
    intervals: vec![interval(None, None)],
    all_real_conservative_fallback: true,
  }
}

/// Project the joint homoskedastic AR confidence region onto one endogenous
/// coefficient, leaving every other endogenous coefficient unrestricted.
///
/// Projection has at-least-nominal coverage because it projects a joint region.
/// The nuisance quadratic is profiled analytically when its curvature is safely
/// positive definite after unit-diagonal scaling (Cholesky pivots above 1e-12).
/// Otherwise the all-real set is returned as a conservative fallback; no nuisance
/// coefficient is fixed at an estimate or searched on a grid.
pub fn projected_anderson_rubin_confidence_set(
  problem: &InstrumentalVariablesProblem,
  coefficient: &str,
  level: f64,
) -> Result<ProjectedArConfidenceSet, StatsError> {
  problem.validate()?;
  if problem.endogenous_names.len() < 2 {
    return Err(invalid(
      "projected AR confidence sets require at least two endogenous regressors",
    ));
  }
  if !level.is_finite() || level <= 0.0 || level >= 100.0 {
    return Err(invalid(
      "AR confidence level must be finite and strictly between 0 and 100",
    ));
  }
  let Some(target) = problem
    .endogenous_names
    .iter()
    .position(|name| name == coefficient)
  else {
    return Err(invalid(
      "projected AR coefficient must name an endogenous regressor in the retained fit",
    ));
  };
  let nuisance_indices = (0..problem.endogenous_names.len())
    .filter(|index| *index != target)
    .collect::<Vec<_>>();
  let nuisance_endogenous = nuisance_indices
    .iter()
    .map(|index| problem.endogenous_names[*index].clone())
    .collect::<Vec<_>>();
  let n = problem.response.len();
  let names = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect::<Vec<_>>();
  let parameters = names.len() + usize::from(problem.include_intercept);
  if n <= parameters {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters,
    });
  }
  let df = n - parameters;
  let critical = critical_f(1.0 - level / 100.0, problem.instrument_names.len(), df)?;
  let design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(w, z)| w.iter().chain(z).copied().collect())
    .collect::<Vec<Vec<f64>>>();

  // Center the quadratic at the existing 2SLS estimate to reduce cancellation;
  // the center affects only arithmetic, never the projected test or its coverage.
  let fitted = fit_two_stage_least_squares(problem, &IvOptions::default())?;
  let centers = fitted
    .regression
    .coefficients
    .iter()
    .filter(|estimate| problem.endogenous_names.contains(&estimate.name))
    .map(|estimate| estimate.value)
    .collect::<Vec<_>>();
  if centers.len() != problem.endogenous_names.len() {
    return Err(invalid("projected AR coefficient center is incomplete"));
  }
  let centered_response = problem
    .response
    .iter()
    .zip(&problem.endogenous)
    .map(|(y, row)| y - row.iter().zip(&centers).map(|(x, b)| x * b).sum::<f64>())
    .collect::<Vec<_>>();
  if centered_response.iter().any(|value| !value.is_finite()) {
    return Err(invalid("projected AR centered outcome is non-finite"));
  }
  let restricted_y = residualize(
    &centered_response,
    &problem.exogenous,
    &problem.exogenous_names,
    problem.include_intercept,
  )?;
  let unrestricted_y = residualize(
    &centered_response,
    &design,
    &names,
    problem.include_intercept,
  )?;
  let mut restricted_x = Vec::with_capacity(problem.endogenous_names.len());
  let mut unrestricted_x = Vec::with_capacity(problem.endogenous_names.len());
  for column in 0..problem.endogenous_names.len() {
    let values = problem
      .endogenous
      .iter()
      .map(|row| row[column])
      .collect::<Vec<_>>();
    restricted_x.push(residualize(
      &values,
      &problem.exogenous,
      &problem.exogenous_names,
      problem.include_intercept,
    )?);
    unrestricted_x.push(residualize(
      &values,
      &design,
      &names,
      problem.include_intercept,
    )?);
  }
  let unrestricted_y_ss = dot(&unrestricted_y, &unrestricted_y);
  if !unrestricted_y_ss.is_finite() || unrestricted_y_ss <= 0.0 {
    return Err(invalid(
      "AR confidence-set auxiliary residual variance is undefined or non-finite",
    ));
  }
  let innovation = fit_least_squares(
    &EstimationProblem::new(
      "projected_ar_auxiliary_variance",
      problem.endogenous_names.clone(),
      unrestricted_y.clone(),
      (0..n)
        .map(|row| {
          unrestricted_x
            .iter()
            .map(|column| column[row])
            .collect::<Vec<_>>()
        })
        .collect(),
    )?
    .with_intercept(false),
  )?
  .residuals
  .iter()
  .map(|value| value * value)
  .sum::<f64>();
  if !innovation.is_finite() || innovation <= 1e-24 * unrestricted_y_ss {
    return Err(invalid(
      "AR confidence-set auxiliary residual pair is numerically degenerate",
    ));
  }

  let k = 1.0 + problem.instrument_names.len() as f64 * critical / df as f64;
  if !k.is_finite() || k <= 1.0 {
    return Err(invalid(
      "AR critical increment exceeds floating-point resolution",
    ));
  }
  let dimension = problem.endogenous_names.len();
  let mut curvature = vec![vec![0.0; dimension]; dimension];
  let mut linear = vec![0.0; dimension];
  for i in 0..dimension {
    linear[i] = dot(&restricted_x[i], &restricted_y) - k * dot(&unrestricted_x[i], &unrestricted_y);
    for j in 0..dimension {
      curvature[i][j] =
        dot(&restricted_x[i], &restricted_x[j]) - k * dot(&unrestricted_x[i], &unrestricted_x[j]);
    }
  }
  let constant = dot(&restricted_y, &restricted_y) - k * unrestricted_y_ss;
  if !constant.is_finite()
    || linear.iter().any(|value| !value.is_finite())
    || curvature.iter().flatten().any(|value| !value.is_finite())
  {
    return Err(invalid("projected AR quadratic is non-finite"));
  }

  let nuisance_curvature = nuisance_indices
    .iter()
    .map(|i| nuisance_indices.iter().map(|j| curvature[*i][*j]).collect())
    .collect::<Vec<Vec<f64>>>();
  let Some((nuisance_scales, factor)) = positive_definite_factor(&nuisance_curvature) else {
    return Ok(all_real_projection(
      problem,
      coefficient,
      nuisance_endogenous,
      level,
      df,
      critical,
    ));
  };
  let nuisance_linear = nuisance_indices
    .iter()
    .map(|index| linear[*index])
    .collect::<Vec<_>>();
  let nuisance_cross = nuisance_indices
    .iter()
    .map(|index| curvature[*index][target])
    .collect::<Vec<_>>();
  let scaled_linear = nuisance_indices
    .iter()
    .enumerate()
    .map(|(position, index)| linear[*index] / nuisance_scales[position])
    .collect::<Vec<_>>();
  let scaled_cross = nuisance_cross
    .iter()
    .enumerate()
    .map(|(position, value)| value / nuisance_scales[position])
    .collect::<Vec<_>>();
  let Some(solution_linear) = cholesky_solve(&factor, &scaled_linear).map(|solution| {
    solution
      .iter()
      .zip(&nuisance_scales)
      .map(|(value, scale)| value / scale)
      .collect::<Vec<_>>()
  }) else {
    return Ok(all_real_projection(
      problem,
      coefficient,
      nuisance_endogenous,
      level,
      df,
      critical,
    ));
  };
  let Some(solution_cross) = cholesky_solve(&factor, &scaled_cross).map(|solution| {
    solution
      .iter()
      .zip(&nuisance_scales)
      .map(|(value, scale)| value / scale)
      .collect::<Vec<_>>()
  }) else {
    return Ok(all_real_projection(
      problem,
      coefficient,
      nuisance_endogenous,
      level,
      df,
      critical,
    ));
  };
  let profile_a = curvature[target][target] - dot(&nuisance_cross, &solution_cross);
  let profile_b = -2.0 * linear[target] + 2.0 * dot(&nuisance_cross, &solution_linear);
  let profile_c = constant - dot(&nuisance_linear, &solution_linear);
  let (shape, mut intervals) = quadratic_set(profile_a, profile_b, profile_c)?;
  let center = centers[target];
  for interval in &mut intervals {
    for bound in [&mut interval.lower, &mut interval.upper]
      .into_iter()
      .flatten()
    {
      *bound += center;
      if !bound.is_finite() {
        return Err(invalid(
          "AR confidence-set endpoint exceeds the finite numerical range",
        ));
      }
    }
  }
  let collapsed = match shape {
    ArConfidenceSetShape::Bounded => intervals[0].lower >= intervals[0].upper,
    ArConfidenceSetShape::Disjoint => intervals[0].upper >= intervals[1].lower,
    _ => false,
  };
  if collapsed {
    return Err(invalid(
      "AR confidence-set endpoints exceed floating-point resolution",
    ));
  }
  Ok(ProjectedArConfidenceSet {
    coefficient: coefficient.to_owned(),
    nuisance_endogenous,
    excluded_instruments: problem.instrument_names.clone(),
    confidence_level: level,
    observation_count: n,
    numerator_df: problem.instrument_names.len(),
    denominator_df: df,
    critical_f: critical,
    shape,
    intervals,
    all_real_conservative_fallback: false,
  })
}

// Solve a*b^2 + b_coefficient*b + c <= 0 without a finite search grid. Scaling
// avoids discriminant overflow; the stable root formula avoids cancellation.
fn quadratic_set(
  a: f64,
  b: f64,
  c: f64,
) -> Result<(ArConfidenceSetShape, Vec<ArConfidenceInterval>), StatsError> {
  use ArConfidenceSetShape::*;
  if [a, b, c].iter().any(|value| !value.is_finite()) {
    return Err(invalid("AR confidence-set quadratic is non-finite"));
  }
  let scale = a.abs().max(b.abs()).max(c.abs());
  if scale == 0.0 {
    return Ok((AllReal, vec![interval(None, None)]));
  }
  let scaled = [a / scale, b / scale, c / scale];
  if [a, b, c]
    .iter()
    .zip(scaled)
    .any(|(original, scaled)| *original != 0.0 && scaled == 0.0)
  {
    return Err(invalid("AR confidence-set coefficient scaling underflow"));
  }
  let [a, b, c] = scaled;
  let finite = |value: f64| {
    if value.is_finite() {
      Ok(value)
    } else {
      Err(invalid(
        "AR confidence-set endpoint exceeds the finite numerical range",
      ))
    }
  };
  if a == 0.0 {
    if b == 0.0 {
      return Ok(if c <= 0.0 {
        (AllReal, vec![interval(None, None)])
      } else {
        (Empty, vec![])
      });
    }
    let root = finite(-c / b)?;
    return Ok(if b > 0.0 {
      (LowerRay, vec![interval(None, Some(root))])
    } else {
      (UpperRay, vec![interval(Some(root), None)])
    });
  }
  // Compare |b| with 2*sqrt(|a|)*sqrt(|c|) before squaring; directly
  // forming b²-4ac can underflow and turn two real roots into a singleton.
  let t = 2.0 * a.abs().sqrt() * c.abs().sqrt();
  let (sign, sqrt_discriminant) = if c == 0.0 {
    (if b == 0.0 { 0 } else { 1 }, b.abs())
  } else if a.is_sign_positive() != c.is_sign_positive() {
    (1, b.hypot(t))
  } else if b.abs() > t {
    (1, (b.abs() - t).sqrt() * (b.abs() + t).sqrt())
  } else {
    (if b.abs() == t { 0 } else { -1 }, 0.0)
  };
  if sign <= 0 {
    if a < 0.0 {
      return Ok((AllReal, vec![interval(None, None)]));
    }
    if sign == 0 {
      let root = finite(-b / (2.0 * a))?;
      return Ok((Singleton, vec![interval(Some(root), Some(root))]));
    }
    return Ok((Empty, vec![]));
  }
  let q = -0.5
    * (b
      + if b >= 0.0 {
        sqrt_discriminant
      } else {
        -sqrt_discriminant
      });
  let (first, second) = (finite(q / a)?, finite(c / q)?);
  let (lower, upper) = (first.min(second), first.max(second));
  Ok(if a > 0.0 {
    (Bounded, vec![interval(Some(lower), Some(upper))])
  } else {
    (
      Disjoint,
      vec![interval(None, Some(lower)), interval(Some(upper), None)],
    )
  })
}

/// Invert homoskedastic AR tests for exactly one endogenous coefficient.
///
/// Included-exogenous coefficients are unrestricted nuisance parameters. No
/// endogenous nuisance coefficients are plugged in or projected out. Confidence
/// levels are percentages strictly between 0 and 100. The retained complete
/// sample is fixed. Numerically degenerate auxiliary variance is an error.
pub fn anderson_rubin_confidence_set(
  problem: &InstrumentalVariablesProblem,
  level: f64,
) -> Result<AndersonRubinConfidenceSet, StatsError> {
  problem.validate()?;
  if problem.endogenous_names.len() != 1 {
    return Err(invalid(
      "AR confidence sets require exactly one endogenous regressor; endogenous nuisance/subvector inference is not implemented",
    ));
  }
  if !level.is_finite() || level <= 0.0 || level >= 100.0 {
    return Err(invalid(
      "AR confidence level must be finite and strictly between 0 and 100",
    ));
  }
  let n = problem.response.len();
  let names = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect::<Vec<_>>();
  let parameters = names.len() + usize::from(problem.include_intercept);
  if n <= parameters {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters,
    });
  }
  let df = n - parameters;
  let critical = critical_f(1.0 - level / 100.0, problem.instrument_names.len(), df)?;
  let design = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(w, z)| w.iter().chain(z).copied().collect())
    .collect::<Vec<Vec<f64>>>();
  let x = problem
    .endogenous
    .iter()
    .map(|row| row[0])
    .collect::<Vec<_>>();
  let ry = residualize(
    &problem.response,
    &problem.exogenous,
    &problem.exogenous_names,
    problem.include_intercept,
  )?;
  let rx = residualize(
    &x,
    &problem.exogenous,
    &problem.exogenous_names,
    problem.include_intercept,
  )?;
  let uy = residualize(
    &problem.response,
    &design,
    &names,
    problem.include_intercept,
  )?;
  let ux = residualize(&x, &design, &names, problem.include_intercept)?;
  let dot = |left: &[f64], right: &[f64]| left.iter().zip(right).map(|(a, b)| a * b).sum::<f64>();
  let (uyy, uxx, uxy) = (dot(&uy, &uy), dot(&ux, &ux), dot(&ux, &uy));
  if !uyy.is_finite() || !uxx.is_finite() || !uxy.is_finite() || uyy <= 0.0 {
    return Err(invalid(
      "AR confidence-set auxiliary residual variance is undefined or non-finite",
    ));
  }
  if uxx > 0.0 {
    let projection = uxy / uxx;
    let innovation = uy
      .iter()
      .zip(&ux)
      .map(|(y, x)| (y - projection * x).powi(2))
      .sum::<f64>();
    // A residual pair with relative innovation norm <= 1e-12 is numerically
    // rank deficient, matching the existing QR pivot scale. Do not invert a
    // quadratic across a null where auxiliary variance cannot be established.
    if !innovation.is_finite() || innovation <= 1e-24 * uyy {
      return Err(invalid(
        "AR confidence-set auxiliary residual pair is numerically degenerate",
      ));
    }
  }
  let rxx = dot(&rx, &rx);
  let center = if rxx > 0.0 { dot(&rx, &ry) / rxx } else { 0.0 };
  if !center.is_finite() {
    return Err(invalid("AR confidence-set centering is non-finite"));
  }
  // Solve in beta-center coordinates: forming a discriminant around a large
  // coefficient can otherwise erase a narrow, genuinely nonzero interval.
  let centered_r = ry
    .iter()
    .zip(&rx)
    .map(|(y, x)| y - center * x)
    .collect::<Vec<_>>();
  let centered_u = uy
    .iter()
    .zip(&ux)
    .map(|(y, x)| y - center * x)
    .collect::<Vec<_>>();
  let k = 1.0 + problem.instrument_names.len() as f64 * critical / df as f64;
  if !k.is_finite() || k <= 1.0 {
    return Err(invalid(
      "AR critical increment exceeds floating-point resolution",
    ));
  }
  let a = rxx - k * uxx;
  let b = -2.0 * (dot(&rx, &centered_r) - k * dot(&ux, &centered_u));
  let c = dot(&centered_r, &centered_r) - k * dot(&centered_u, &centered_u);
  let (shape, mut intervals) = quadratic_set(a, b, c)?;
  for interval in &mut intervals {
    for bound in [&mut interval.lower, &mut interval.upper]
      .into_iter()
      .flatten()
    {
      *bound += center;
      if !bound.is_finite() {
        return Err(invalid(
          "AR confidence-set endpoint exceeds the finite numerical range",
        ));
      }
    }
  }
  let collapsed = match shape {
    ArConfidenceSetShape::Bounded => intervals[0].lower >= intervals[0].upper,
    ArConfidenceSetShape::Disjoint => intervals[0].upper >= intervals[1].lower,
    _ => false,
  };
  if collapsed {
    return Err(invalid(
      "AR confidence-set endpoints exceed floating-point resolution",
    ));
  }
  Ok(AndersonRubinConfidenceSet {
    endogenous: problem.endogenous_names[0].clone(),
    excluded_instruments: problem.instrument_names.clone(),
    confidence_level: level,
    observation_count: n,
    numerator_df: problem.instrument_names.len(),
    denominator_df: df,
    critical_f: critical,
    shape,
    intervals,
  })
}

/// Invert the HC1/CR1 AR Wald test for one endogenous regressor and one excluded
/// instrument. Included exogenous coefficients remain unrestricted.
///
/// For each null, the auxiliary regression is `y - beta*x` on the included
/// exogenous regressors and the excluded instrument. With one tested auxiliary
/// coefficient, the chi-square(1) Wald non-rejection condition reduces to a
/// quadratic inequality. This function solves that inequality directly; it does
/// not evaluate a finite grid. HC1 assumes independent rows. Cluster inference
/// assumes independent groups with many groups and has no few-cluster guarantee.
pub fn robust_anderson_rubin_confidence_set(
  problem: &InstrumentalVariablesProblem,
  level: f64,
  covariance_type: &CovarianceType,
) -> Result<RobustAndersonRubinConfidenceSet, StatsError> {
  if *covariance_type == CovarianceType::NonRobust {
    return Err(invalid(
      "robust AR confidence sets require HC1 or one-way cluster covariance",
    ));
  }
  if problem.endogenous_names.len() != 1 || problem.instrument_names.len() != 1 {
    return Err(invalid(
      "robust AR confidence sets require exactly one endogenous regressor and one excluded instrument",
    ));
  }
  if !level.is_finite() || level <= 0.0 || level >= 100.0 {
    return Err(invalid(
      "AR confidence level must be finite and strictly between 0 and 100",
    ));
  }
  problem.validate()?;
  let alpha = 1.0 - level / 100.0;
  let upper_normal_probability = 1.0 - alpha / 2.0;
  let z = normal_quantile(upper_normal_probability);
  let critical_chi_square = z * z;
  if !alpha.is_finite()
    || alpha <= 0.0
    || upper_normal_probability >= 1.0
    || !critical_chi_square.is_finite()
  {
    return Err(invalid(
      "AR confidence level exceeds floating-point probability resolution",
    ));
  }

  let n = problem.response.len();
  let parameter_count = problem.exogenous_names.len() + usize::from(problem.include_intercept) + 1;
  if n <= parameter_count {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters: parameter_count,
    });
  }
  let residual_df = n - parameter_count;
  let (cluster_group_count, covariance_correction, cluster_groups) = match covariance_type {
    CovarianceType::NonRobust => unreachable!("nonrobust covariance rejected above"),
    CovarianceType::RobustHc1 => (None, n as f64 / residual_df as f64, None),
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
      let count = groups
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
      if count < 2 {
        return Err(StatsError::InsufficientObservations(
          "cluster covariance requires at least 2 distinct clusters".into(),
        ));
      }
      let correction = count as f64 / (count - 1) as f64 * (n - 1) as f64 / residual_df as f64;
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
  let z = problem
    .instruments
    .iter()
    .map(|row| row[0])
    .collect::<Vec<_>>();
  let control_names = problem.exogenous_names.clone();
  let residual_z = residualize(
    &z,
    &problem.exogenous,
    &control_names,
    problem.include_intercept,
  )?;
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
  let instrument_norm = dot(&residual_z, &residual_z);
  let reduced_form_y = dot(&residual_z, &residual_y);
  let reduced_form_x = dot(&residual_z, &residual_x);
  if !instrument_norm.is_finite()
    || instrument_norm <= 0.0
    || !reduced_form_y.is_finite()
    || !reduced_form_x.is_finite()
  {
    return Err(invalid(
      "robust AR confidence-set auxiliary design is rank deficient or non-finite",
    ));
  }
  let center = if reduced_form_x != 0.0 {
    reduced_form_y / reduced_form_x
  } else {
    0.0
  };
  if !center.is_finite() {
    return Err(invalid("robust AR confidence-set centering is non-finite"));
  }

  let mut auxiliary_names = control_names;
  auxiliary_names.push(problem.instrument_names[0].clone());
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

  let (score_y, score_x) = if let Some(groups) = cluster_groups {
    let mut totals = std::collections::BTreeMap::<&str, (f64, f64)>::new();
    for ((group, z), (residual_y, residual_x)) in groups
      .iter()
      .zip(&residual_z)
      .zip(residual_y_full.iter().zip(&residual_x_full))
    {
      let entry = totals.entry(group.as_str()).or_default();
      entry.0 += z * residual_y;
      entry.1 += z * residual_x;
    }
    let pairs = totals.into_values().collect::<Vec<_>>();
    (
      pairs.iter().map(|pair| pair.0).collect::<Vec<_>>(),
      pairs.iter().map(|pair| pair.1).collect::<Vec<_>>(),
    )
  } else {
    (
      residual_z
        .iter()
        .zip(&residual_y_full)
        .map(|(z, residual)| z * residual)
        .collect(),
      residual_z
        .iter()
        .zip(&residual_x_full)
        .map(|(z, residual)| z * residual)
        .collect(),
    )
  };
  if score_y
    .iter()
    .chain(&score_x)
    .any(|score| !score.is_finite())
  {
    return Err(invalid("robust AR confidence-set scores are non-finite"));
  }
  let score_y_norm = dot(&score_y, &score_y);
  let score_x_norm = dot(&score_x, &score_x);
  if !score_y_norm.is_finite() || !score_x_norm.is_finite() {
    return Err(invalid("robust AR confidence-set covariance is non-finite"));
  }
  if score_y_norm == 0.0 && score_x_norm == 0.0 {
    return Err(invalid(
      "robust AR confidence-set auxiliary covariance is zero for every null",
    ));
  }
  if score_x_norm > 0.0 {
    let projection = dot(&score_y, &score_x) / score_x_norm;
    let innovation = score_y
      .iter()
      .zip(&score_x)
      .map(|(y, x)| (y - projection * x).powi(2))
      .sum::<f64>();
    if !innovation.is_finite() || innovation <= 1e-24 * score_y_norm {
      return Err(invalid(
        "robust AR confidence-set auxiliary covariance is degenerate at a candidate null",
      ));
    }
  }

  let centered_response = residual_y
    .iter()
    .zip(&residual_x)
    .map(|(y, x)| y - center * x)
    .collect::<Vec<_>>();
  let centered_residual = residual_y_full
    .iter()
    .zip(&residual_x_full)
    .map(|(y, x)| y - center * x)
    .collect::<Vec<_>>();
  if centered_response
    .iter()
    .chain(&centered_residual)
    .any(|value| !value.is_finite())
  {
    return Err(invalid(
      "robust AR confidence-set centered outcome is non-finite",
    ));
  }
  let centered_score_y = if let Some(groups) = cluster_groups {
    let mut totals = std::collections::BTreeMap::<&str, f64>::new();
    for ((group, z), residual) in groups.iter().zip(&residual_z).zip(&centered_residual) {
      *totals.entry(group.as_str()).or_default() += z * residual;
    }
    totals.into_values().collect::<Vec<_>>()
  } else {
    residual_z
      .iter()
      .zip(&centered_residual)
      .map(|(z, residual)| z * residual)
      .collect::<Vec<_>>()
  };
  let centered_moment = dot(&residual_z, &centered_response);
  let reduced_form_slope = reduced_form_x;
  let scale = critical_chi_square * covariance_correction;
  let quadratic_a = reduced_form_slope.powi(2) - scale * score_x_norm;
  let quadratic_b =
    -2.0 * centered_moment * reduced_form_slope + 2.0 * scale * dot(&centered_score_y, &score_x);
  let quadratic_c = centered_moment.powi(2) - scale * dot(&centered_score_y, &centered_score_y);
  let (shape, mut intervals) = quadratic_set(quadratic_a, quadratic_b, quadratic_c)?;
  for interval in &mut intervals {
    for bound in [&mut interval.lower, &mut interval.upper]
      .into_iter()
      .flatten()
    {
      *bound += center;
      if !bound.is_finite() {
        return Err(invalid(
          "robust AR confidence-set endpoint exceeds the finite numerical range",
        ));
      }
    }
  }
  let collapsed = match shape {
    ArConfidenceSetShape::Bounded => intervals[0].lower >= intervals[0].upper,
    ArConfidenceSetShape::Disjoint => intervals[0].upper >= intervals[1].lower,
    _ => false,
  };
  if collapsed {
    return Err(invalid(
      "robust AR confidence-set endpoints exceed floating-point resolution",
    ));
  }
  Ok(RobustAndersonRubinConfidenceSet {
    endogenous: problem.endogenous_names[0].clone(),
    excluded_instrument: problem.instrument_names[0].clone(),
    confidence_level: level,
    observation_count: n,
    auxiliary_parameter_count: parameter_count,
    auxiliary_residual_df: residual_df,
    covariance_type: covariance_type.clone(),
    covariance_correction,
    cluster_group_count,
    degrees_of_freedom: 1,
    critical_chi_square,
    shape,
    intervals,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn quadratic_roots_avoid_cancellation_and_do_not_hide_range_loss() {
    let (shape, bounds) = quadratic_set(1.0, 1e16, 1.0).unwrap();
    assert_eq!(shape, ArConfidenceSetShape::Bounded);
    assert!((bounds[0].lower.unwrap() / -1e16 - 1.0).abs() < 1e-14);
    assert!((bounds[0].upper.unwrap() / -1e-16 - 1.0).abs() < 1e-14);
    assert!(quadratic_set(f64::MIN_POSITIVE, f64::MAX, 1.0).is_err());
    assert!(quadratic_set(f64::NAN, 0.0, 1.0).is_err());
  }

  #[test]
  fn quadratic_boundaries_follow_the_closed_nonrejection_inequality() {
    use ArConfidenceSetShape::*;
    for (coefficients, shape, bounds) in [
      (
        [1.0, 0.0, -1.0],
        Bounded,
        vec![interval(Some(-1.0), Some(1.0))],
      ),
      (
        [-1.0, 0.0, 1.0],
        Disjoint,
        vec![interval(None, Some(-1.0)), interval(Some(1.0), None)],
      ),
      (
        [1.0, 0.0, 0.0],
        Singleton,
        vec![interval(Some(0.0), Some(0.0))],
      ),
      ([-1.0, 0.0, 0.0], AllReal, vec![interval(None, None)]),
      ([0.0, 1.0, -2.0], LowerRay, vec![interval(None, Some(2.0))]),
      ([0.0, -1.0, 2.0], UpperRay, vec![interval(Some(2.0), None)]),
      ([0.0, 0.0, 1.0], Empty, vec![]),
      ([0.0, 0.0, 0.0], AllReal, vec![interval(None, None)]),
    ] {
      assert_eq!(
        quadratic_set(coefficients[0], coefficients[1], coefficients[2]).unwrap(),
        (shape, bounds)
      );
    }
  }
}
