use crate::{
  EstimationProblem, InstrumentalVariablesProblem, LeastSquaresResult, StatsError,
  f_distribution_survival_probability, fit_least_squares, mean,
};
use serde::{Deserialize, Serialize};

fn invalid(message: &str) -> StatsError {
  StatsError::IncompatibleHypothesis(message.into())
}
fn norm_squared(values: &[f64]) -> f64 {
  values.iter().map(|value| value * value).sum()
}

// Only residuals are used. Rescaling auxiliary predictors leaves the nested
// models unchanged and makes QR rank checks insensitive to variable units.
fn auxiliary_fit(
  outcome: &str,
  response: Vec<f64>,
  names: Vec<String>,
  mut design: Vec<Vec<f64>>,
  intercept: bool,
) -> Result<LeastSquaresResult, StatsError> {
  for column in 0..names.len() {
    let norm = design
      .iter()
      .map(|row| row[column] * row[column])
      .sum::<f64>()
      .sqrt();
    if !norm.is_finite() || norm <= 0.0 {
      return Err(invalid(
        "Wu-Hausman auxiliary predictor has zero or nonfinite norm",
      ));
    }
    for row in &mut design {
      row[column] /= norm;
    }
  }
  fit_least_squares(
    &EstimationProblem::new(outcome, names, response, design)?.with_intercept(intercept),
  )
}

/// Joint, homoskedastic regression-based Wu-Hausman F test of exogeneity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WuHausmanResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub numerator_df: usize,
  pub denominator_df: usize,
  pub statistic: f64,
  pub p_value: f64,
}

/// Test all declared endogenous regressors jointly, on this retained IV sample.
/// Uses auxiliary first stages and restricted/innovation-augmented OLS, not a
/// 2SLS refit, Durbin LM or covariance-difference Hausman calculation. Inference
/// presumes valid instruments, homoskedastic errors and the maintained model;
/// conventional identification conditions apply. An exact F additionally needs
/// conditional normality. This does not deliver weak-IV-robust coefficient inference.
pub fn wu_hausman_test(
  problem: &InstrumentalVariablesProblem,
) -> Result<WuHausmanResult, StatsError> {
  problem.validate()?;
  let m = problem.endogenous_names.len();
  let n = problem.response.len();
  let k = problem.exogenous_names.len() + m + usize::from(problem.include_intercept);
  if n <= k + m {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: n,
      parameters: k + m,
    });
  }
  let instrument_names: Vec<String> = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect();
  let instrument_design: Vec<Vec<f64>> = problem
    .exogenous
    .iter()
    .zip(&problem.instruments)
    .map(|(w, z)| w.iter().chain(z).copied().collect())
    .collect();
  let original_names: Vec<String> = problem
    .exogenous_names
    .iter()
    .chain(&problem.endogenous_names)
    .cloned()
    .collect();
  let original_design: Vec<Vec<f64>> = problem
    .exogenous
    .iter()
    .zip(&problem.endogenous)
    .map(|(w, x)| w.iter().chain(x).copied().collect())
    .collect();
  let mut augmented_names = original_names.clone();
  let mut augmented_design = original_design.clone();
  for index in 0..m {
    let x: Vec<f64> = problem.endogenous.iter().map(|row| row[index]).collect();
    let partial_x = if problem.exogenous_names.is_empty() && !problem.include_intercept {
      x.clone()
    } else {
      auxiliary_fit(
        &problem.endogenous_names[index],
        x.clone(),
        problem.exogenous_names.clone(),
        problem.exogenous.clone(),
        problem.include_intercept,
      )?
      .residuals
    };
    let v = auxiliary_fit(
      &problem.endogenous_names[index],
      x,
      instrument_names.clone(),
      instrument_design.clone(),
      problem.include_intercept,
    )?
    .residuals;
    let partial_norm = norm_squared(&partial_x);
    let innovation_norm = norm_squared(&v);
    if !partial_norm.is_finite()
      || partial_norm <= 0.0
      || !innovation_norm.is_finite()
      || innovation_norm <= 1e-24 * partial_norm
    {
      return Err(invalid(
        "Wu-Hausman first-stage innovation is undefined or numerically degenerate",
      ));
    }
    // Internal labels must not collide with any supported user variable name.
    let mut name = format!("__wu_innovation_{index}");
    while name == problem.outcome || augmented_names.contains(&name) {
      name.push('_');
    }
    augmented_names.push(name);
    for (row, value) in augmented_design.iter_mut().zip(v) {
      row.push(value);
    }
  }
  let restricted = auxiliary_fit(
    &problem.outcome,
    problem.response.clone(),
    original_names,
    original_design,
    problem.include_intercept,
  )?;
  let augmented = auxiliary_fit(
    &problem.outcome,
    problem.response.clone(),
    augmented_names,
    augmented_design,
    problem.include_intercept,
  )?;
  let rss_r = norm_squared(&restricted.residuals);
  let rss_u = norm_squared(&augmented.residuals);
  let center = if problem.include_intercept {
    mean(&problem.response)?
  } else {
    0.0
  };
  let outcome_scale = problem
    .response
    .iter()
    .map(|value| (value - center).powi(2))
    .sum::<f64>();
  let constant_outcome = problem.include_intercept
    && problem
      .response
      .iter()
      .all(|value| *value == problem.response[0]);
  if constant_outcome
    || !outcome_scale.is_finite()
    || outcome_scale <= 0.0
    || !rss_r.is_finite()
    || rss_r <= 0.0
    || !rss_u.is_finite()
    || rss_u <= 1e-24 * outcome_scale
  {
    return Err(invalid(
      "Wu-Hausman auxiliary residual variance is undefined or numerically degenerate",
    ));
  }
  let df = n - k - m;
  // The augmented space contains the restricted space; roundoff can make a
  // theoretically zero RSS reduction slightly negative. Divide by RSS before
  // applying DF, avoiding underflow from RSS/DF for tiny outcome units.
  let statistic = (rss_r - rss_u).max(0.0) / rss_u * (df as f64 / m as f64);
  let p_value = f_distribution_survival_probability(statistic, m as f64, df as f64);
  if !statistic.is_finite() || !p_value.is_finite() {
    return Err(invalid("Wu-Hausman statistic or F tail is unavailable"));
  }
  Ok(WuHausmanResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: n,
    numerator_df: m,
    denominator_df: df,
    statistic,
    p_value,
  })
}
