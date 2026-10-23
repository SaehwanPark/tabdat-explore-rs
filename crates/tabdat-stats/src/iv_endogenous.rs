use crate::{
  CovarianceType, EstimationProblem, InstrumentalVariablesProblem, LeastSquaresOptions,
  LeastSquaresResult, StatsError, f_distribution_survival_probability, fit_least_squares,
  fit_least_squares_with_options, mean,
};
use serde::{Deserialize, Serialize};

fn invalid(message: &str) -> StatsError {
  StatsError::IncompatibleHypothesis(message.into())
}
fn norm_squared(values: &[f64]) -> f64 {
  values.iter().map(|value| value * value).sum()
}

// Rescaling leaves the nested spaces and zero-innovation hypothesis unchanged,
// and makes QR rank checks insensitive to variable units.
fn auxiliary_problem(
  outcome: &str,
  response: Vec<f64>,
  names: Vec<String>,
  mut design: Vec<Vec<f64>>,
  intercept: bool,
) -> Result<EstimationProblem, StatsError> {
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
  Ok(EstimationProblem::new(outcome, names, response, design)?.with_intercept(intercept))
}

fn auxiliary_fit(
  outcome: &str,
  response: Vec<f64>,
  names: Vec<String>,
  design: Vec<Vec<f64>>,
  intercept: bool,
) -> Result<LeastSquaresResult, StatsError> {
  fit_least_squares(&auxiliary_problem(
    outcome, response, names, design, intercept,
  )?)
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

fn endogeneity_fits(
  problem: &InstrumentalVariablesProblem,
  covariance_type: &CovarianceType,
) -> Result<(LeastSquaresResult, LeastSquaresResult), StatsError> {
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
  let mut augmented_problem = auxiliary_problem(
    &problem.outcome,
    problem.response.clone(),
    augmented_names,
    augmented_design,
    problem.include_intercept,
  )?;
  augmented_problem.sample = problem.sample.clone();
  let augmented = fit_least_squares_with_options(
    &augmented_problem,
    &LeastSquaresOptions {
      covariance_type: covariance_type.clone(),
    },
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
  Ok((restricted, augmented))
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
  let (restricted, augmented) = endogeneity_fits(problem, &CovarianceType::NonRobust)?;
  let m = problem.endogenous_names.len();
  let n = problem.response.len();
  let df = augmented.fit_statistics.degrees_of_freedom;
  let rss_r = norm_squared(&restricted.residuals);
  let rss_u = norm_squared(&augmented.residuals);
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

/// Joint residual-inclusion Wald test using auxiliary HC1 or one-way CR1 scores.
/// The chi-square reference is asymptotic under valid instruments, the maintained
/// model and conventional identification. Clusters must be independent with a
/// growing group count; this is not a weak-IV or few-cluster procedure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobustEndogeneityResult {
  pub endogenous: Vec<String>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub auxiliary_parameter_count: usize,
  pub auxiliary_residual_df: usize,
  pub covariance_type: CovarianceType,
  pub covariance_correction: f64,
  pub cluster_group_count: Option<usize>,
  pub degrees_of_freedom: usize,
  pub statistic: f64,
  pub p_value: f64,
}

/// Test zero coefficients on all first-stage innovations in augmented OLS.
/// Under this joint null the first-stage nuisance estimation does not contribute
/// at first order. Covariance corrections use the augmented parameter count,
/// not the parent IV count. No subset, score or covariance-difference test is used.
pub fn robust_endogeneity_test(
  problem: &InstrumentalVariablesProblem,
  covariance_type: &CovarianceType,
) -> Result<RobustEndogeneityResult, StatsError> {
  if *covariance_type == CovarianceType::NonRobust {
    return Err(invalid(
      "robust endogeneity requires HC1 or one-way cluster covariance",
    ));
  }
  let (_, augmented) = endogeneity_fits(problem, covariance_type)?;
  let n = augmented.observation_count();
  let k = augmented.fit_statistics.parameter_count;
  let df = n - k;
  let groups = if matches!(covariance_type, CovarianceType::Cluster(_)) {
    Some(
      augmented
        .sample
        .as_ref()
        .unwrap()
        .cluster_groups
        .as_ref()
        .unwrap()
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len(),
    )
  } else {
    None
  };
  let correction = groups.map_or(n as f64 / df as f64, |g| {
    g as f64 / (g - 1) as f64 * (n - 1) as f64 / df as f64
  });
  let m = problem.endogenous_names.len();
  let start = k - m;
  let values = augmented.coefficients[start..]
    .iter()
    .map(|c| c.value)
    .collect::<Vec<_>>();
  let covariance = augmented.covariance.matrix[start..]
    .iter()
    .map(|row| row[start..].to_vec())
    .collect::<Vec<_>>();
  let wald = crate::binary::omnibus_wald(&values, &covariance, false, groups);
  if let Some(reason) = wald.unavailable_reason {
    return Err(invalid(&format!(
      "endogeneity innovation Wald test unavailable: {reason}"
    )));
  }
  Ok(RobustEndogeneityResult {
    endogenous: problem.endogenous_names.clone(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: n,
    auxiliary_parameter_count: k,
    auxiliary_residual_df: df,
    covariance_type: covariance_type.clone(),
    covariance_correction: correction,
    cluster_group_count: groups,
    degrees_of_freedom: m,
    statistic: wald.statistic.unwrap(),
    p_value: wald.p_value.unwrap(),
  })
}
