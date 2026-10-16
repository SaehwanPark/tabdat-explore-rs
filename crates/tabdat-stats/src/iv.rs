#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::StatsError;
use crate::estimates::{
  CoefficientEstimate, CovarianceMatrix, CovarianceType, EstimationDiagnostics, FitStatistics,
};
use crate::least_squares::fit_least_squares;
use crate::matrix::{
  f_distribution_survival_probability, multiply_vector, qr_decompose_with_response, sandwich,
  scale, solve_upper_triangular, student_t_pvalue, xtx_inverse_from_r,
};
use crate::problem::EstimationProblem;
use crate::result::LeastSquaresResult;
use crate::sample::EstimationSample;

/// Unweighted linear IV design: included exogenous variables instrument themselves,
/// while `instruments` contains only excluded instruments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentalVariablesProblem {
  pub outcome: String,
  pub exogenous_names: Vec<String>,
  pub endogenous_names: Vec<String>,
  pub instrument_names: Vec<String>,
  pub response: Vec<f64>,
  /// Row-major included-exogenous design (one row per candidate observation).
  pub exogenous: Vec<Vec<f64>>,
  /// Row-major endogenous-regressor design, in `endogenous_names` order.
  pub endogenous: Vec<Vec<f64>>,
  /// Row-major excluded-instrument design, in `instrument_names` order.
  pub instruments: Vec<Vec<f64>>,
  pub include_intercept: bool,
  pub sample: Option<EstimationSample>,
}

/// Covariance choice for a two-stage least-squares fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IvOptions {
  pub covariance_type: CovarianceType,
}

impl Default for IvOptions {
  fn default() -> Self {
    Self {
      covariance_type: CovarianceType::NonRobust,
    }
  }
}

/// First-stage relevance diagnostics for one endogenous regressor (not weak-IV-robust inference).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirstStageDiagnostic {
  pub endogenous: String,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub numerator_df: usize,
  pub denominator_df: usize,
  pub partial_r_squared: Option<f64>,
  /// Shea's design-only partial R-squared, accounting for other endogenous regressors.
  #[serde(default)]
  pub shea_partial_r_squared: Option<f64>,
  pub f_statistic: Option<f64>,
  pub p_value: Option<f64>,
}

/// 2SLS coefficient result plus conventional first-stage relevance summaries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IvResult {
  pub regression: LeastSquaresResult,
  pub first_stage: Vec<FirstStageDiagnostic>,
}

impl InstrumentalVariablesProblem {
  pub fn validate(&self) -> Result<(), StatsError> {
    if self.outcome.trim().is_empty() {
      return Err(StatsError::InvalidParameterName(
        "outcome variable name must not be empty".into(),
      ));
    }
    if self.endogenous_names.is_empty() {
      return Err(StatsError::InvalidParameterName(
        "2SLS requires at least one endogenous regressor".into(),
      ));
    }
    if self.instrument_names.is_empty() {
      return Err(StatsError::Underidentified {
        endogenous_regressors: self.endogenous_names.len(),
        excluded_instruments: 0,
      });
    }

    let mut names = BTreeMap::<&str, &'static str>::new();
    for (group_name, variables) in [
      ("exogenous", &self.exogenous_names),
      ("endogenous", &self.endogenous_names),
      ("excluded instrument", &self.instrument_names),
    ] {
      for name in variables {
        if name.trim().is_empty() {
          return Err(StatsError::InvalidParameterName(format!(
            "{group_name} variable name must not be empty"
          )));
        }
        if let Some(previous_group) = names.insert(name.as_str(), group_name) {
          return Err(StatsError::InvalidParameterName(format!(
            "variable '{name}' is repeated as {previous_group} and {group_name}"
          )));
        }
      }
    }
    if names.contains_key(self.outcome.as_str()) {
      return Err(StatsError::InvalidParameterName(
        "outcome must not also be a regressor or instrument".into(),
      ));
    }

    if self.response.is_empty() {
      return Err(StatsError::EmptySample);
    }
    let nobs = self.response.len();
    validate_matrix(
      &self.exogenous,
      nobs,
      self.exogenous_names.len(),
      "exogenous",
    )?;
    validate_matrix(
      &self.endogenous,
      nobs,
      self.endogenous_names.len(),
      "endogenous",
    )?;
    validate_matrix(
      &self.instruments,
      nobs,
      self.instrument_names.len(),
      "instrument",
    )?;
    for (index, value) in self.response.iter().enumerate() {
      if !value.is_finite() {
        return Err(StatsError::NonFiniteValue(format!(
          "non-finite outcome at row {index}"
        )));
      }
    }
    if self
      .exogenous
      .iter()
      .chain(self.endogenous.iter())
      .chain(self.instruments.iter())
      .flatten()
      .any(|value| !value.is_finite())
    {
      return Err(StatsError::NonFiniteValue(
        "non-finite value in IV design".into(),
      ));
    }
    if let Some(sample) = &self.sample {
      if sample.retained_count() != nobs {
        return Err(StatsError::DimensionMismatch(format!(
          "sample retained count ({}) must match response length ({nobs})",
          sample.retained_count()
        )));
      }
      if sample.weights.is_some() {
        return Err(StatsError::IncompatibleHypothesis(
          "weights are not supported by this 2SLS slice".into(),
        ));
      }
    }

    let parameter_count = self.exogenous_names.len()
      + self.endogenous_names.len()
      + usize::from(self.include_intercept);
    let instrument_count = self.exogenous_names.len()
      + self.instrument_names.len()
      + usize::from(self.include_intercept);
    if instrument_count < parameter_count {
      return Err(StatsError::Underidentified {
        endogenous_regressors: self.endogenous_names.len(),
        excluded_instruments: instrument_count
          - self.exogenous_names.len()
          - usize::from(self.include_intercept),
      });
    }
    if nobs <= parameter_count {
      return Err(StatsError::ZeroDegreesOfFreedom {
        observations: nobs,
        parameters: parameter_count,
      });
    }
    Ok(())
  }
}

fn validate_matrix(
  matrix: &[Vec<f64>],
  rows: usize,
  columns: usize,
  name: &str,
) -> Result<(), StatsError> {
  if matrix.len() != rows {
    return Err(StatsError::DimensionMismatch(format!(
      "{name} matrix rows ({}) must match observation count ({rows})",
      matrix.len()
    )));
  }
  for (index, row) in matrix.iter().enumerate() {
    if row.len() != columns {
      return Err(StatsError::DimensionMismatch(format!(
        "{name} matrix row {index} has {} columns; expected {columns}",
        row.len()
      )));
    }
  }
  Ok(())
}

fn concatenate_columns(left: &[Vec<f64>], right: &[Vec<f64>]) -> Vec<Vec<f64>> {
  left
    .iter()
    .zip(right)
    .map(|(left_row, right_row)| {
      left_row
        .iter()
        .chain(right_row)
        .copied()
        .collect::<Vec<_>>()
    })
    .collect()
}

fn add_intercept(matrix: &[Vec<f64>], include_intercept: bool) -> Vec<Vec<f64>> {
  matrix
    .iter()
    .map(|row| {
      if include_intercept {
        std::iter::once(1.0).chain(row.iter().copied()).collect()
      } else {
        row.clone()
      }
    })
    .collect()
}

fn fit_first_stage(
  problem: &InstrumentalVariablesProblem,
  endogenous_index: usize,
  instrument_names: &[String],
  restricted_names: &[String],
) -> Result<(Vec<f64>, FirstStageDiagnostic), StatsError> {
  let response = problem
    .endogenous
    .iter()
    .map(|row| row[endogenous_index])
    .collect::<Vec<_>>();
  let full_problem = EstimationProblem::new(
    problem.endogenous_names[endogenous_index].clone(),
    instrument_names.to_vec(),
    response.clone(),
    concatenate_columns(&problem.exogenous, &problem.instruments),
  )?
  .with_intercept(problem.include_intercept);
  let full = fit_least_squares(&full_problem)?;

  let restricted_rss = if restricted_names.is_empty() && !problem.include_intercept {
    response.iter().map(|value| value * value).sum()
  } else {
    let restricted_problem = EstimationProblem::new(
      problem.endogenous_names[endogenous_index].clone(),
      restricted_names.to_vec(),
      response.clone(),
      problem.exogenous.clone(),
    )?
    .with_intercept(problem.include_intercept);
    fit_least_squares(&restricted_problem)?
      .fit_statistics
      .residual_sum_of_squares
      .ok_or_else(|| StatsError::DimensionMismatch("first-stage RSS is unavailable".into()))?
  };
  let unrestricted_rss = full
    .fit_statistics
    .residual_sum_of_squares
    .ok_or_else(|| StatsError::DimensionMismatch("first-stage RSS is unavailable".into()))?;
  let numerator_df = problem.instrument_names.len();
  let denominator_df = full.fit_statistics.degrees_of_freedom;
  let (partial_r_squared, f_statistic, p_value) = if restricted_rss > 0.0 && unrestricted_rss > 0.0
  {
    // Nested-model RSS is monotone in exact arithmetic; max(0) protects the
    // reported test against a negative value from floating-point roundoff.
    let reduction = (restricted_rss - unrestricted_rss).max(0.0);
    let partial_r_squared = Some((reduction / restricted_rss).clamp(0.0, 1.0));
    let f_statistic = (denominator_df > 0)
      .then(|| (reduction / numerator_df as f64) / (unrestricted_rss / denominator_df as f64));
    let p_value = f_statistic
      .map(|f| f_distribution_survival_probability(f, numerator_df as f64, denominator_df as f64));
    (partial_r_squared, f_statistic, p_value)
  } else {
    (None, None, None)
  };

  Ok((
    full.fitted_values,
    FirstStageDiagnostic {
      endogenous: problem.endogenous_names[endogenous_index].clone(),
      excluded_instruments: problem.instrument_names.clone(),
      observation_count: problem.response.len(),
      numerator_df,
      denominator_df,
      partial_r_squared,
      shea_partial_r_squared: None,
      f_statistic,
      p_value,
    },
  ))
}

/// Fit linear two-stage least squares with classical, HC1, or one-way clustered covariance.
///
/// The estimation sample is already selected by the caller and must be complete for all
/// included variables, excluded instruments, and (when clustered) cluster identifiers.
/// Inference uses residual degrees of freedom for classical/HC1 covariance and G-1 for
/// one-way cluster covariance. First-stage tests are conventional homoskedastic partial F
/// tests of the excluded instruments conditional on the included exogenous regressors.
pub fn fit_two_stage_least_squares(
  problem: &InstrumentalVariablesProblem,
  options: &IvOptions,
) -> Result<IvResult, StatsError> {
  problem.validate()?;

  let nobs = problem.response.len();
  let endogenous_count = problem.endogenous_names.len();
  let instrument_names = problem
    .exogenous_names
    .iter()
    .chain(problem.instrument_names.iter())
    .cloned()
    .collect::<Vec<_>>();
  let restricted_names = problem.exogenous_names.clone();
  let mut fitted_endogenous = Vec::with_capacity(endogenous_count);
  let mut first_stage = Vec::with_capacity(endogenous_count);
  for index in 0..endogenous_count {
    let (fitted, diagnostic) =
      fit_first_stage(problem, index, &instrument_names, &restricted_names)?;
    fitted_endogenous.push(fitted);
    first_stage.push(diagnostic);
  }

  let mut original_design = add_intercept(&problem.exogenous, problem.include_intercept);
  for (row_index, row) in original_design.iter_mut().enumerate() {
    row.extend(
      problem
        .endogenous_names
        .iter()
        .enumerate()
        .map(|(column, _)| problem.endogenous[row_index][column]),
    );
  }
  let parameter_names = problem
    .include_intercept
    .then(|| "intercept".to_owned())
    .into_iter()
    .chain(problem.exogenous_names.iter().cloned())
    .chain(problem.endogenous_names.iter().cloned())
    .collect::<Vec<_>>();

  let mut projected_design = add_intercept(&problem.exogenous, problem.include_intercept);
  for (row_index, row) in projected_design.iter_mut().enumerate() {
    row.extend(fitted_endogenous.iter().map(|fitted| fitted[row_index]));
  }
  let second_stage_qr = qr_decompose_with_response(&projected_design, &problem.response)?;
  let beta = solve_upper_triangular(&second_stage_qr.r, &second_stage_qr.qty)?;
  let bread = xtx_inverse_from_r(&second_stage_qr.r)?;
  // Godfrey's classical OLS/IV variance-and-R² identity cancels the outcome
  // variance and equal residual-DF corrections, leaving this design-only ratio.
  // Do not use the requested HC1/cluster covariance: Shea measures relevance,
  // and remains defined even when the outcome has zero residual variance.
  let original_qr = qr_decompose_with_response(&original_design, &problem.response)?;
  let original_bread = xtx_inverse_from_r(&original_qr.r)?;
  let endogenous_offset = usize::from(problem.include_intercept) + problem.exogenous_names.len();
  for (index, diagnostic) in first_stage.iter_mut().enumerate() {
    let parameter = endogenous_offset + index;
    let ratio = original_bread[parameter][parameter] / bread[parameter][parameter];
    diagnostic.shea_partial_r_squared = ratio.is_finite().then(|| ratio.clamp(0.0, 1.0));
  }
  let fitted_values = multiply_vector(&original_design, &beta)?;
  let residuals = problem
    .response
    .iter()
    .zip(&fitted_values)
    .map(|(actual, fitted)| actual - fitted)
    .collect::<Vec<_>>();
  let rss = residuals
    .iter()
    .map(|residual| residual * residual)
    .sum::<f64>();
  let parameter_count = parameter_names.len();
  let residual_df = nobs - parameter_count;
  let tss = if problem.include_intercept {
    let average = problem.response.iter().sum::<f64>() / nobs as f64;
    problem
      .response
      .iter()
      .map(|value| (value - average).powi(2))
      .sum::<f64>()
  } else {
    problem.response.iter().map(|value| value * value).sum()
  };
  let r_squared = (tss > 0.0).then(|| 1.0 - rss / tss);
  let adjusted_r_squared = (tss > 0.0 && residual_df > 0).then(|| {
    let tss_df = if problem.include_intercept {
      (nobs - 1) as f64
    } else {
      nobs as f64
    };
    1.0 - (rss / residual_df as f64) / (tss / tss_df)
  });
  let sigma_squared = rss / residual_df as f64;

  let (covariance_matrix, inference_df) = match &options.covariance_type {
    CovarianceType::NonRobust => (scale(&bread, sigma_squared), residual_df as f64),
    CovarianceType::RobustHc1 => {
      let correction = nobs as f64 / residual_df as f64;
      let meat = score_meat(&projected_design, &residuals, correction);
      (sandwich(&bread, &meat)?, residual_df as f64)
    }
    CovarianceType::Cluster(cluster_variable) => {
      let sample = problem.sample.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(
          "cluster covariance requires an estimation sample with cluster identifiers".into(),
        )
      })?;
      let groups = sample.cluster_groups.as_ref().ok_or_else(|| {
        StatsError::DimensionMismatch(format!(
          "cluster covariance on variable '{cluster_variable}' requires cluster group identifiers"
        ))
      })?;
      if groups.len() != nobs {
        return Err(StatsError::DimensionMismatch(format!(
          "cluster group identifier count ({}) does not match retained observations ({nobs})",
          groups.len()
        )));
      }
      let mut clusters = BTreeMap::<&str, Vec<usize>>::new();
      for (index, group) in groups.iter().enumerate() {
        clusters.entry(group).or_default().push(index);
      }
      let group_count = clusters.len();
      if group_count < 2 {
        return Err(StatsError::InsufficientObservations(
          "cluster covariance requires at least 2 distinct clusters".into(),
        ));
      }
      let group_count_f = group_count as f64;
      let cluster_correction =
        (group_count_f / (group_count_f - 1.0)) * ((nobs as f64 - 1.0) / residual_df as f64);
      let meat = cluster_score_meat(&projected_design, &residuals, &clusters, cluster_correction);
      (sandwich(&bread, &meat)?, (group_count - 1) as f64)
    }
  };

  let covariance = CovarianceMatrix::new(
    parameter_names.clone(),
    options.covariance_type.clone(),
    covariance_matrix,
  )?;
  let standard_errors = covariance.standard_errors();
  let coefficients = parameter_names
    .iter()
    .enumerate()
    .map(|(index, name)| {
      let standard_error = standard_errors[index];
      let p_value = (standard_error > 0.0)
        .then(|| student_t_pvalue(beta[index] / standard_error, inference_df))
        .filter(|value| !value.is_nan());
      CoefficientEstimate::new(name, beta[index], Some(standard_error), p_value)
    })
    .collect();
  let regression = LeastSquaresResult {
    parameter_names,
    coefficients,
    fitted_values,
    residuals,
    covariance,
    diagnostics: EstimationDiagnostics {
      method: "two_stage_least_squares".into(),
      converged: true,
      iterations: 1,
      objective_value: Some(rss),
      residual_sum_of_squares: Some(rss),
    },
    fit_statistics: FitStatistics {
      observation_count: nobs,
      parameter_count,
      degrees_of_freedom: residual_df,
      r_squared,
      adjusted_r_squared,
      root_mse: Some(sigma_squared.sqrt()),
      residual_sum_of_squares: Some(rss),
      total_sum_of_squares: Some(tss),
      // The ordinary least-squares overall F test is not an IV model test.
      f_statistic: None,
      log_likelihood: None,
    },
    sample: problem.sample.clone(),
  };

  Ok(IvResult {
    regression,
    first_stage,
  })
}

/// Homoskedastic Anderson–Rubin test of a joint null for every endogenous coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AndersonRubinResult {
  pub endogenous: Vec<String>,
  pub null_values: Vec<f64>,
  pub excluded_instruments: Vec<String>,
  pub observation_count: usize,
  pub numerator_df: usize,
  pub denominator_df: usize,
  pub statistic: f64,
  pub p_value: f64,
}

/// Test all endogenous coefficients jointly, leaving included-exogenous coefficients
/// unrestricted. The auxiliary regression tests excluded instruments in y - X*b0.
/// This is weak-identification robust, not heteroskedasticity/cluster robust or a
/// test of instrument validity. Exact F inference requires classical normal errors;
/// homoskedastic large-sample validity still requires valid instruments.
///
/// No endogenous nuisance coefficients may be left unspecified. The caller supplies
/// the fixed complete estimation sample. Undefined auxiliary variance is an error.
pub fn anderson_rubin_test(
  problem: &InstrumentalVariablesProblem,
  null_values: &[f64],
) -> Result<AndersonRubinResult, StatsError> {
  problem.validate()?;
  if null_values.len() != problem.endogenous_names.len() {
    return Err(StatsError::DimensionMismatch(
      "Anderson-Rubin null must specify every endogenous coefficient in fit order".into(),
    ));
  }
  if null_values.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "Anderson-Rubin null must be finite".into(),
    ));
  }
  let adjusted = problem
    .response
    .iter()
    .zip(&problem.endogenous)
    .map(|(y, row)| {
      y - row
        .iter()
        .zip(null_values)
        .map(|(x, beta)| x * beta)
        .sum::<f64>()
    })
    .collect::<Vec<_>>();
  if adjusted.iter().any(|value| !value.is_finite()) {
    return Err(StatsError::NonFiniteValue(
      "Anderson-Rubin adjusted outcome is non-finite".into(),
    ));
  }
  let names = problem
    .exogenous_names
    .iter()
    .chain(&problem.instrument_names)
    .cloned()
    .collect::<Vec<_>>();
  let full = fit_least_squares(
    &EstimationProblem::new(
      "ar_adjusted_outcome",
      names,
      adjusted.clone(),
      concatenate_columns(&problem.exogenous, &problem.instruments),
    )?
    .with_intercept(problem.include_intercept),
  )?;
  let restricted_rss = if problem.exogenous_names.is_empty() && !problem.include_intercept {
    adjusted.iter().map(|value| value * value).sum::<f64>()
  } else {
    fit_least_squares(
      &EstimationProblem::new(
        "ar_adjusted_outcome",
        problem.exogenous_names.clone(),
        adjusted,
        problem.exogenous.clone(),
      )?
      .with_intercept(problem.include_intercept),
    )?
    .residuals
    .iter()
    .map(|e| e * e)
    .sum()
  };
  let unrestricted_rss = full.residuals.iter().map(|e| e * e).sum::<f64>();
  let numerator_df = problem.instrument_names.len();
  let denominator_df = full.fit_statistics.degrees_of_freedom;
  // Nested residual sums of squares are monotone in exact arithmetic.
  let statistic = ((restricted_rss - unrestricted_rss).max(0.0) / numerator_df as f64)
    / (unrestricted_rss / denominator_df as f64);
  if unrestricted_rss <= 0.0 || !statistic.is_finite() {
    return Err(StatsError::IncompatibleHypothesis(
      "Anderson-Rubin auxiliary residual variance is undefined or non-finite".into(),
    ));
  }
  let p_value =
    f_distribution_survival_probability(statistic, numerator_df as f64, denominator_df as f64);
  Ok(AndersonRubinResult {
    endogenous: problem.endogenous_names.clone(),
    null_values: null_values.to_vec(),
    excluded_instruments: problem.instrument_names.clone(),
    observation_count: problem.response.len(),
    numerator_df,
    denominator_df,
    statistic,
    p_value,
  })
}

fn score_meat(design: &[Vec<f64>], residuals: &[f64], correction: f64) -> Vec<Vec<f64>> {
  let parameters = design[0].len();
  let mut meat = vec![vec![0.0; parameters]; parameters];
  for (row, residual) in design.iter().zip(residuals) {
    let scale = correction * residual.powi(2);
    for j in 0..parameters {
      for k in 0..parameters {
        meat[j][k] += scale * row[j] * row[k];
      }
    }
  }
  meat
}

fn cluster_score_meat(
  design: &[Vec<f64>],
  residuals: &[f64],
  clusters: &BTreeMap<&str, Vec<usize>>,
  correction: f64,
) -> Vec<Vec<f64>> {
  let parameters = design[0].len();
  let mut meat = vec![vec![0.0; parameters]; parameters];
  for indices in clusters.values() {
    let mut score = vec![0.0; parameters];
    for &index in indices {
      for parameter in 0..parameters {
        score[parameter] += design[index][parameter] * residuals[index];
      }
    }
    for row in 0..parameters {
      for column in 0..parameters {
        meat[row][column] += correction * score[row] * score[column];
      }
    }
  }
  meat
}
