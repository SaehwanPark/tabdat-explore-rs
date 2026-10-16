use serde::{Deserialize, Serialize};

use crate::{
  EstimationProblem, InstrumentalVariablesProblem, StatsError, f_distribution_survival_probability,
  fit_least_squares,
};

/// Shape of the scalar AR nonrejection set; rays include their finite endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArConfidenceSetShape {
  Bounded,
  Disjoint,
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

fn invalid(message: &str) -> StatsError {
  StatsError::IncompatibleHypothesis(message.into())
}

fn critical_f(alpha: f64, numerator_df: usize, denominator_df: usize) -> Result<f64, StatsError> {
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
  let k = 1.0 + problem.instrument_names.len() as f64 * critical / df as f64;
  let a = dot(&rx, &rx) - k * uxx;
  let b = -2.0 * (dot(&rx, &ry) - k * uxy);
  let c = dot(&ry, &ry) - k * uyy;
  let (shape, intervals) = quadratic_set(a, b, c)?;
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
