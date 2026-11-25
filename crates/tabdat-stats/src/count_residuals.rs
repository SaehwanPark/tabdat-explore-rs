//! Current-row Pearson and deviance residuals for Poisson and NB2 laws.

use crate::{CountProbabilityLaw, MAX_NB2_COUNT, MAX_POISSON_COUNT, StatsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountResidualKind {
  Pearson,
  Deviance,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("count residuals: {message}"))
}

fn log1pmx(value: f64) -> f64 {
  if value.abs() >= 0.01 {
    return value.ln_1p() - value;
  }
  let (mut sum, mut correction) = (0.0, 0.0);
  let mut power = value * value;
  for degree in 2..=100 {
    let term = if degree % 2 == 0 {
      -power / degree as f64
    } else {
      power / degree as f64
    };
    let adjusted = term - correction;
    let next = sum + adjusted;
    correction = (next - sum) - adjusted;
    sum = next;
    if term.abs() <= sum.abs() * f64::EPSILON {
      break;
    }
    power *= value;
  }
  sum
}

fn log1p_product(left: f64, right: f64) -> f64 {
  let product = left * right;
  if product.is_finite() {
    product.ln_1p()
  } else {
    left.ln() + right.ln()
  }
}

fn signed_root(half_deviance: f64, observed: f64, mean: f64) -> Option<f64> {
  if !half_deviance.is_finite() || half_deviance < 0.0 {
    return None;
  }
  let magnitude = 2.0_f64.sqrt() * half_deviance.sqrt();
  let residual = if observed < mean {
    -magnitude
  } else {
    magnitude
  };
  residual.is_finite().then_some(residual)
}

fn poisson_deviance(observed: f64, mean: f64) -> Option<f64> {
  if observed == 0.0 {
    return Some(mean);
  }
  if mean == 0.0 {
    return None;
  }
  let relative_difference = (observed - mean) / mean;
  let half_deviance = if relative_difference.is_finite() && relative_difference.abs() < 0.01 {
    mean
      * (relative_difference * relative_difference
        + (1.0 + relative_difference) * log1pmx(relative_difference))
  } else {
    observed * (observed.ln() - mean.ln()) - (observed - mean)
  };
  (half_deviance.is_finite() && half_deviance >= 0.0).then_some(half_deviance)
}

fn nb2_deviance(observed: f64, mean: f64, alpha: f64) -> Option<f64> {
  let inverse_alpha = 1.0 / alpha;
  if observed == 0.0 {
    let half_deviance = inverse_alpha * log1p_product(alpha, mean);
    return (half_deviance.is_finite() && half_deviance >= 0.0).then_some(half_deviance);
  }
  if mean == 0.0 {
    return None;
  }
  let relative_difference = (observed - mean) / mean;
  let half_deviance = if relative_difference.is_finite() && relative_difference.abs() < 0.01 {
    let alpha_mean = alpha * mean;
    let fraction = if alpha_mean.is_finite() {
      alpha_mean / (1.0 + alpha_mean)
    } else {
      1.0
    };
    let quadratic_scale = if alpha_mean.is_finite() {
      mean / (1.0 + alpha_mean)
    } else {
      inverse_alpha
    };
    let log_ratio = fraction * relative_difference;
    quadratic_scale * relative_difference * relative_difference
      + observed * log1pmx(relative_difference)
      - (observed + inverse_alpha) * log1pmx(log_ratio)
  } else {
    let first = observed * (observed.ln() - mean.ln());
    let log_ratio = log1p_product(alpha, observed) - log1p_product(alpha, mean);
    first - (observed + inverse_alpha) * log_ratio
  };
  (half_deviance.is_finite() && half_deviance >= 0.0).then_some(half_deviance)
}

/// Compute an unscaled current-row Pearson or deviance residual for a count law.
///
/// `Ok(None)` means the residual is undefined or not representable, such as a
/// Pearson residual when both the observation and fitted mean are zero. Invalid
/// count/model inputs return an error so callers can fail their mutation.
pub fn count_diagnostic_residual(
  law: CountProbabilityLaw,
  observed: i128,
  mean: f64,
  kind: CountResidualKind,
) -> Result<Option<f64>, StatsError> {
  let maximum = match law {
    CountProbabilityLaw::Poisson => MAX_POISSON_COUNT,
    CountProbabilityLaw::Nb2 { .. } => MAX_NB2_COUNT,
  };
  let observed_value = observed as f64;
  if observed < 0 || !observed_value.is_finite() || observed_value > maximum {
    return Err(invalid(
      "observed value must be an exact supported nonnegative count",
    ));
  }
  if !mean.is_finite() || mean < 0.0 {
    return Err(invalid("fitted mean must be finite and nonnegative"));
  }
  if let CountProbabilityLaw::Nb2 { alpha } = law
    && (!alpha.is_finite() || alpha <= 0.0 || alpha.ln().abs() >= 12.0)
  {
    return Err(invalid(
      "NB2 alpha must have log(alpha) strictly in (-12,12)",
    ));
  }

  let observed = observed_value;
  if kind == CountResidualKind::Pearson {
    if mean == 0.0 {
      return Ok(None);
    }
    let residual = match law {
      CountProbabilityLaw::Poisson => (observed - mean) / mean.sqrt(),
      CountProbabilityLaw::Nb2 { alpha } => {
        let alpha_mean = alpha * mean;
        if alpha_mean.is_finite() {
          ((observed - mean) / mean.sqrt()) / (1.0 + alpha_mean).sqrt()
        } else {
          (observed / mean - 1.0) / (alpha + 1.0 / mean).sqrt()
        }
      }
    };
    return Ok(residual.is_finite().then_some(residual));
  }

  if mean == 0.0 {
    return Ok((observed == 0.0).then_some(0.0));
  }
  let deviance = match law {
    CountProbabilityLaw::Poisson => poisson_deviance(observed, mean),
    CountProbabilityLaw::Nb2 { alpha } => nb2_deviance(observed, mean, alpha),
  };
  Ok(deviance.and_then(|value| signed_root(value, observed, mean)))
}
