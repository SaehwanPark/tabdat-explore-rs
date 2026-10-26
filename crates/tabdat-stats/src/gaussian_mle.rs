//! Shared guarded Newton solve for concave parameterizations.
//! Model-specific likelihoods enforce their own support (positive final scale in
//! Gaussian/Weibull families, unrestricted Cox coefficients/Gompertz gamma). Neither damping
//! nor a pseudoinverse enters the returned information.
use crate::StatsError;
use crate::matrix::choice_information_inverse;

pub(crate) struct Evaluation {
  pub log_likelihood: f64,
  pub score: Vec<f64>,
  pub information: Vec<Vec<f64>>,
}
pub(crate) struct Solution {
  pub theta: Vec<f64>,
  pub evaluation: Evaluation,
  pub inverse: Vec<Vec<f64>>,
  pub iterations: usize,
  pub score_bound: f64,
  pub step_bound: f64,
}
pub(crate) fn maximize(
  mut theta: Vec<f64>,
  n: usize,
  max_iterations: usize,
  family: &str,
  evaluate: impl Fn(&[f64]) -> Result<Evaluation, StatsError>,
) -> Result<Solution, StatsError> {
  let failed = |message: &str| StatsError::EstimationFailed(format!("{family}: {message}"));
  let n = n as f64;
  for iteration in 1..=max_iterations {
    let e = evaluate(&theta)?;
    let average: Vec<Vec<f64>> = e
      .information
      .iter()
      .map(|row| row.iter().map(|v| v / n).collect())
      .collect();
    let inverse = choice_information_inverse(&average)
      .map_err(|_| failed("singular or numerically degenerate joint information"))?;
    let score: Vec<f64> = e.score.iter().map(|v| v / n).collect();
    let step: Vec<f64> = inverse
      .iter()
      .map(|row| row.iter().zip(&score).map(|(a, b)| a * b).sum())
      .collect();
    let score_bound = score.iter().map(|v| v.abs()).fold(0.0, f64::max);
    let step_bound = step
      .iter()
      .zip(&theta)
      .map(|(d, t)| d.abs() / (1.0 + t.abs()))
      .fold(0.0, f64::max);
    if score_bound <= 1e-10 && step_bound <= 1e-10 {
      return Ok(Solution {
        theta,
        evaluation: e,
        inverse: inverse
          .iter()
          .map(|row| row.iter().map(|v| v / n).collect())
          .collect(),
        iterations: iteration,
        score_bound,
        step_bound,
      });
    }
    let mut alpha = 1.0;
    let mut accepted = None;
    for _ in 0..60 {
      let candidate: Vec<f64> = theta
        .iter()
        .zip(&step)
        .map(|(t, d)| t + alpha * d)
        .collect();
      if let Ok(next) = evaluate(&candidate)
        && next.log_likelihood
          >= e.log_likelihood - 32.0 * f64::EPSILON * (1.0 + e.log_likelihood.abs())
      {
        accepted = Some(candidate);
        break;
      }
      alpha *= 0.5;
    }
    theta = accepted.ok_or_else(|| failed("Newton line search failed"))?;
  }
  Err(failed(
    "iteration limit reached without score/step convergence; no finite regular fit established",
  ))
}
