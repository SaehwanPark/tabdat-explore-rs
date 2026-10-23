#![forbid(unsafe_code)]

use crate::bootstrap::{draw_index, summarize_pairs};
use crate::distributions::{normal_cdf, normal_quantile};
use crate::quantile::{quantile_design, quantile_point_coefficients};
use crate::{BOOTSTRAP_RNG, BootstrapCoefficient, EstimationProblem, EstimationSample, StatsError};
use rand_xoshiro::{Xoshiro256PlusPlus, rand_core::SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileBootstrapResult {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub quantiles: Vec<f64>,
  pub replications: usize,
  pub seed: u64,
  pub rng: String,
  pub confidence_level: f64,
  pub interval_method: String,
  pub coefficients: Vec<BootstrapCoefficient>,
  pub covariance_parameter_names: Vec<String>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub sample: Option<EstimationSample>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub contrast: Option<QuantileBootstrapContrast>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileBootstrapContrast {
  pub parameter: String,
  pub lower_quantile: f64,
  pub upper_quantile: f64,
  /// Higher-quantile coefficient minus lower-quantile coefficient; null is zero.
  pub estimate: f64,
  pub standard_error: Option<f64>,
  pub statistic: Option<f64>,
  pub p_value: Option<f64>,
  pub ci_lower: Option<f64>,
  pub ci_upper: Option<f64>,
  pub confidence_level: f64,
  pub reference_distribution: Option<String>,
  pub interval_method: Option<String>,
  pub inference_unavailable_reason: Option<String>,
}

fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!("quantile pairs bootstrap: {}", message.into()))
}

fn contrast_summary(
  parameter: &str,
  quantiles: &[f64],
  original: &[f64],
  estimates: &[Vec<f64>],
  column: usize,
  k: usize,
) -> Result<QuantileBootstrapContrast, StatsError> {
  let estimate = original[k + column] - original[column];
  if !estimate.is_finite() {
    return Err(invalid("original contrast difference is not representable"));
  }
  // Equivalent to c'Vc, but avoids catastrophic subtraction of nearly equal
  // covariance entries. Common replicate draws supply the cross-quantile term.
  let differences: Vec<_> = estimates[k + column]
    .iter()
    .zip(&estimates[column])
    .map(|(upper, lower)| upper - lower)
    .collect();
  let mut result = QuantileBootstrapContrast {
    parameter: parameter.into(),
    lower_quantile: quantiles[0],
    upper_quantile: quantiles[1],
    estimate,
    standard_error: None,
    statistic: None,
    p_value: None,
    ci_lower: None,
    ci_upper: None,
    confidence_level: 95.0,
    reference_distribution: None,
    interval_method: None,
    inference_unavailable_reason: Some("contrast_variance_not_representable".into()),
  };
  if differences.iter().any(|v| !v.is_finite()) {
    return Ok(result);
  }
  let Ok((summary, _)) = summarize_pairs(&[parameter.into()], &[estimate], vec![differences])
  else {
    return Ok(result);
  };
  let se = summary[0].standard_error;
  result.standard_error = Some(se);
  if se <= 0.0 {
    result.inference_unavailable_reason = Some("zero_contrast_variance".into());
    return Ok(result);
  }
  let statistic = estimate / se;
  let critical = normal_quantile(0.975);
  let lower = estimate - critical * se;
  let upper = estimate + critical * se;
  if [statistic, lower, upper].iter().any(|v| !v.is_finite()) {
    result.inference_unavailable_reason =
      Some("contrast_normal_inference_not_representable".into());
    return Ok(result);
  }
  result.statistic = Some(statistic);
  result.p_value = Some(2.0 * normal_cdf(-statistic.abs()));
  result.ci_lower = Some(lower);
  result.ci_upper = Some(upper);
  result.reference_distribution = Some("asymptotic_standard_normal".into());
  result.interval_method = Some("normal_wald".into());
  result.inference_unavailable_reason = None;
  Ok(result)
}

/// Joint IID pairs bootstrap: each replication shares its N row draws across
/// all quantiles. Marginal type7 percentile intervals do not provide simultaneous
/// coverage. Failed/ambiguous fits abort, with no discards or redraws.
pub fn bootstrap_quantiles(
  problem: &EstimationProblem,
  quantiles: &[f64],
  replications: usize,
  seed: u64,
) -> Result<QuantileBootstrapResult, StatsError> {
  bootstrap_quantiles_impl(problem, quantiles, replications, seed, None)
}

/// Request one higher-minus-lower coefficient contrast on exactly two quantiles.
/// Normal inference uses the shared-pairs difference variance, not marginal SEs.
pub fn bootstrap_quantiles_with_contrast(
  problem: &EstimationProblem,
  quantiles: &[f64],
  replications: usize,
  seed: u64,
  parameter: &str,
) -> Result<QuantileBootstrapResult, StatsError> {
  bootstrap_quantiles_impl(problem, quantiles, replications, seed, Some(parameter))
}

fn bootstrap_quantiles_impl(
  problem: &EstimationProblem,
  quantiles: &[f64],
  replications: usize,
  seed: u64,
  contrast: Option<&str>,
) -> Result<QuantileBootstrapResult, StatsError> {
  if quantiles.is_empty()
    || quantiles.len() > 19
    || quantiles
      .iter()
      .any(|q| !q.is_finite() || !(0.05..=0.95).contains(q))
    || quantiles.windows(2).any(|w| w[0] >= w[1])
  {
    return Err(invalid(
      "supply 1..19 strictly increasing finite quantiles from 0.05 to 0.95",
    ));
  }
  if !(2..=crate::MAX_BOOTSTRAP_REPLICATIONS).contains(&replications) {
    return Err(invalid("replications must be from 2 to 5000"));
  }
  let design = quantile_design(problem, quantiles[0])?;
  let contrast_column = contrast
    .map(|parameter| {
      if quantiles.len() != 2 {
        return Err(invalid("contrast requires exactly two quantiles"));
      }
      let names = problem.parameter_names();
      let mut columns = names
        .iter()
        .enumerate()
        .filter(|(_, name)| *name == parameter);
      let column = columns
        .next()
        .map(|(i, _)| i)
        .ok_or_else(|| invalid(format!("unknown contrast coefficient: {parameter}")))?;
      if columns.next().is_some() {
        return Err(invalid(format!(
          "ambiguous contrast coefficient: {parameter}"
        )));
      }
      Ok(column)
    })
    .transpose()?;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  // Count all coefficient fits, including originals; solver iteration/refinement
  // costs remain separately bounded by the quantile solver.
  let work = n
    .checked_mul(k * k)
    .and_then(|v| v.checked_mul(quantiles.len()))
    .and_then(|v| v.checked_mul(replications + 1));
  if work.is_none_or(|v| v > crate::MAX_BOOTSTRAP_WORK) {
    return Err(invalid(
      "N*K²*quantiles*(replications+1) must be at most 100000000",
    ));
  }
  let mut names = Vec::with_capacity(k * quantiles.len());
  let mut original = Vec::with_capacity(k * quantiles.len());
  for &q in quantiles {
    let beta =
      quantile_point_coefficients(&design, &problem.response, q, problem.include_intercept)
        .map_err(|e| invalid(format!("original quantile {q} failed: {e}")))?;
    original.extend(beta);
    names.extend(
      problem
        .parameter_names()
        .iter()
        .map(|name| format!("q{q}:{name}")),
    );
  }
  let mut estimates = vec![Vec::with_capacity(replications); original.len()];
  let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
  for replicate in 0..replications {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for _ in 0..n {
      let index = draw_index(&mut rng, n);
      x.push(design[index].clone());
      y.push(problem.response[index]);
    }
    for (i, &q) in quantiles.iter().enumerate() {
      let beta =
        quantile_point_coefficients(&x, &y, q, problem.include_intercept).map_err(|e| {
          invalid(format!(
            "replicate {} quantile {q} failed: {e}; no draws were discarded or retried",
            replicate + 1
          ))
        })?;
      for (column, value) in estimates[i * k..(i + 1) * k].iter_mut().zip(beta) {
        column.push(value);
      }
    }
  }
  let contrast = contrast
    .zip(contrast_column)
    .map(|(parameter, column)| {
      contrast_summary(parameter, quantiles, &original, &estimates, column, k)
    })
    .transpose()?;
  let (coefficients, covariance_matrix) = summarize_pairs(&names, &original, estimates)?;
  Ok(QuantileBootstrapResult {
    outcome: problem.outcome.clone(),
    predictors: problem.predictor_names.clone(),
    include_intercept: problem.include_intercept,
    observation_count: n,
    quantiles: quantiles.to_vec(),
    replications,
    seed,
    rng: BOOTSTRAP_RNG.into(),
    confidence_level: 95.0,
    interval_method: "percentile_r_type7".into(),
    coefficients,
    covariance_parameter_names: names,
    covariance_matrix,
    sample: problem.sample.clone(),
    contrast,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn contrast_overflow_withholds_inference_without_nonfinite_serialization() {
    let c = contrast_summary(
      "x",
      &[0.25, 0.75],
      &[1., 2.],
      &[vec![-5e153, 5e153], vec![5e153, -5e153]],
      0,
      1,
    )
    .unwrap();
    assert!(c.standard_error.is_none() && c.reference_distribution.is_none());
    assert_eq!(
      c.inference_unavailable_reason.as_deref(),
      Some("contrast_variance_not_representable")
    );
    assert!(serde_json::to_value(&c).unwrap()["standard_error"].is_null());
    assert!(
      contrast_summary(
        "x",
        &[0.25, 0.75],
        &[-f64::MAX, f64::MAX],
        &[vec![0., 1.], vec![1., 2.]],
        0,
        1
      )
      .is_err()
    );
  }
  #[test]
  fn shared_differences_resolve_variance_lost_by_covariance_subtraction() {
    let lower = vec![-2e8, -1e8, 1e8, 2e8];
    let upper: Vec<_> = lower.iter().map(|v| v + 1. + v * 1e-10).collect();
    let contrast = contrast_summary("x", &[0.25, 0.75], &[0., 1.], &[lower, upper], 0, 1).unwrap();
    let value = serde_json::json!({"contrast":contrast});
    let mut compared = 0;
    for line in include_str!("../tests/fixtures/quantile-bootstrap/contrast_reference.csv")
      .lines()
      .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != "cancellation" {
        continue;
      }
      let actual = value.pointer(row[1]).unwrap().as_f64().unwrap();
      let expected: f64 = row[2].parse().unwrap();
      let tolerance = if row[1].ends_with("p_value") {
        expected.abs() * 1e-7 + 1e-300
      } else {
        1e-8
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{row:?}: {actual} != {expected}"
      );
      compared += 1;
    }
    assert_eq!(compared, 6);
  }
}
