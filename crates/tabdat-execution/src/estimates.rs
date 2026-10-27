use crate::*;

fn failure(message: impl Into<String>) -> RuntimeError {
  RuntimeError::EstimatesFailed {
    message: message.into(),
  }
}

fn snapshot(name: &str, state: &EstimationState) -> Result<EstimateSnapshot, RuntimeError> {
  let mut report = match state {
    EstimationState::Linear(state) => {
      if state.estimator.as_deref() != Some("ols") {
        return Err(failure(
          "store supports unweighted OLS, logit, probit, Poisson and NLS only; refit if estimator provenance is unavailable",
        ));
      }
      let result = &state.result;
      let sample = result
        .sample
        .as_ref()
        .ok_or_else(|| failure("fit sample is unavailable; refit before storing"))?;
      if sample.weights.is_some() {
        return Err(failure("weighted linear reports are not installed"));
      }
      let (covariance, df) = match &result.covariance.covariance_type {
        tabdat_stats::CovarianceType::NonRobust => {
          ("nonrobust".into(), result.degrees_of_freedom())
        }
        tabdat_stats::CovarianceType::RobustHc1 => ("robust".into(), result.degrees_of_freedom()),
        tabdat_stats::CovarianceType::Cluster(variable) => {
          let groups = sample
            .cluster_groups
            .as_ref()
            .ok_or_else(|| failure("cluster provenance is unavailable"))?;
          let count = groups.iter().collect::<BTreeSet<_>>().len();
          let df = count
            .checked_sub(1)
            .filter(|df| *df > 0)
            .ok_or_else(|| failure("cluster inference requires at least two groups"))?;
          (format!("cluster({variable})"), df)
        }
      };
      let likelihood = result
        .fit_statistics
        .log_likelihood
        .filter(|value| value.is_finite());
      EstimateSnapshot {
        name: name.into(),
        estimator: "ols".into(),
        outcome: state.outcome.clone(),
        nls_model: None,
        nls_centered_r_squared: None,
        nls_covariance_correction: None,
        poisson_offset_variable: None,
        poisson_offset_coefficient: None,
        poisson_null_log_likelihood: None,
        poisson_null_model: None,
        poisson_covariance_correction: None,
        poisson_mcfadden_r_squared: None,
        poisson_zero_count: None,
        poisson_count_sum: None,
        predictors: state.predictors.clone(),
        include_intercept: state.include_intercept,
        cluster_variable: match &result.covariance.covariance_type {
          tabdat_stats::CovarianceType::Cluster(variable) => Some(variable.clone()),
          _ => None,
        },
        cluster_count: matches!(
          result.covariance.covariance_type,
          tabdat_stats::CovarianceType::Cluster(_)
        )
        .then_some(df + 1),
        covariance,
        covariance_parameter_names: result.covariance.parameter_names.clone(),
        covariance_matrix: result.covariance.matrix.clone(),
        coefficients: result.coefficients.clone(),
        observation_count: result.observation_count(),
        event_count: None,
        sample: sample.clone(),
        reference_distribution: "student_t".into(),
        inference_degrees_of_freedom: Some(df),
        log_likelihood: likelihood,
        log_likelihood_convention: likelihood
          .map(|_| "gaussian_working_model_ml_variance_rss_over_n".into()),
        r_squared: result.r_squared().filter(|v| v.is_finite()),
        mcfadden_r_squared: None,
        root_mse: result.root_mse().filter(|v| v.is_finite()),
        df_residual: Some(result.degrees_of_freedom()),
      }
    }
    EstimationState::Logit(state) | EstimationState::Probit(state) => {
      let result = &state.result;
      let estimator = if matches!(state.result.model, tabdat_stats::BinaryModel::Logit) {
        "logit"
      } else {
        "probit"
      };
      let covariance = if result.cluster.is_some() {
        "cluster_sandwich_g_over_g_minus_1"
      } else if result.robust {
        "robust_sandwich_n_over_n_minus_1"
      } else if estimator == "logit" {
        "model_based_fisher_information"
      } else {
        "model_based_observed_information"
      };
      EstimateSnapshot {
        name: name.into(),
        estimator: estimator.into(),
        outcome: result.outcome.clone(),
        nls_model: None,
        nls_centered_r_squared: None,
        nls_covariance_correction: None,
        poisson_offset_variable: None,
        poisson_offset_coefficient: None,
        poisson_null_log_likelihood: None,
        poisson_null_model: None,
        poisson_covariance_correction: None,
        poisson_mcfadden_r_squared: None,
        poisson_zero_count: None,
        poisson_count_sum: None,
        predictors: result.predictor_names.clone(),
        include_intercept: result.include_intercept,
        covariance: covariance.into(),
        cluster_variable: result.cluster.as_ref().map(|c| c.variable.clone()),
        cluster_count: result.cluster.as_ref().map(|c| c.group_count),
        covariance_parameter_names: result.coefficients.iter().map(|c| c.name.clone()).collect(),
        covariance_matrix: result.covariance.clone(),
        coefficients: result
          .coefficients
          .iter()
          .map(|c| tabdat_stats::CoefficientEstimate {
            name: c.name.clone(),
            value: c.value,
            standard_error: Some(c.standard_error),
            statistic: Some(c.statistic),
            p_value: Some(c.p_value),
          })
          .collect(),
        observation_count: result.observation_count,
        event_count: Some(result.event_count),
        sample: result
          .sample
          .clone()
          .ok_or_else(|| failure("fit sample is unavailable; refit before storing"))?,
        reference_distribution: "asymptotic_standard_normal".into(),
        inference_degrees_of_freedom: None,
        log_likelihood: Some(result.log_likelihood),
        log_likelihood_convention: Some("bernoulli".into()),
        r_squared: None,
        mcfadden_r_squared: Some(result.mcfadden_r_squared),
        root_mse: None,
        df_residual: None,
      }
    }
    EstimationState::PoissonFit(state) => {
      let result = &state.result;
      let sample = result
        .sample
        .as_ref()
        .ok_or_else(|| failure("fit sample is unavailable; refit before storing"))?;
      if sample.weights.is_some() {
        return Err(failure("weighted Poisson reports are not installed"));
      }
      let (covariance, correction) = if let Some(cluster) = &result.cluster {
        let groups = sample
          .cluster_groups
          .as_ref()
          .ok_or_else(|| failure("cluster provenance is unavailable"))?;
        let count = groups.iter().collect::<BTreeSet<_>>().len();
        if count != cluster.group_count || count < 2 {
          return Err(failure(
            "cluster provenance is inconsistent; refit before storing",
          ));
        }
        (
          "cluster_sandwich_g_over_g_minus_1",
          count as f64 / (count - 1) as f64,
        )
      } else if result.robust {
        let n = result.observation_count;
        if n < 2 {
          return Err(failure(
            "robust covariance requires at least two observations",
          ));
        }
        (
          "robust_sandwich_n_over_n_minus_1",
          n as f64 / (n - 1) as f64,
        )
      } else {
        ("model_based_poisson_information", 1.0)
      };
      EstimateSnapshot {
        name: name.into(),
        estimator: "poisson".into(),
        outcome: result.outcome.clone(),
        nls_model: None,
        nls_centered_r_squared: None,
        nls_covariance_correction: None,
        poisson_offset_variable: result.offset_variable.clone(),
        poisson_offset_coefficient: result.offset_variable.as_ref().map(|_| 1.0),
        poisson_null_log_likelihood: Some(result.null_log_likelihood),
        poisson_null_model: Some(
          match (result.offset_variable.is_some(), result.include_intercept) {
            (true, true) => "intercept_only_with_same_fixed_offset",
            (true, false) => "all_coefficients_zero_with_same_fixed_offset",
            (false, true) => "intercept_only_estimated_mean",
            (false, false) => "unit_mean_all_coefficients_zero",
          }
          .into(),
        ),
        poisson_covariance_correction: Some(correction),
        poisson_mcfadden_r_squared: result
          .mcfadden_r_squared
          .is_finite()
          .then_some(result.mcfadden_r_squared),
        poisson_zero_count: Some(result.zero_count),
        poisson_count_sum: Some(result.count_sum),
        predictors: result.predictor_names.clone(),
        include_intercept: result.include_intercept,
        covariance: covariance.into(),
        cluster_variable: result.cluster.as_ref().map(|c| c.variable.clone()),
        cluster_count: result.cluster.as_ref().map(|c| c.group_count),
        covariance_parameter_names: result.coefficients.iter().map(|c| c.name.clone()).collect(),
        covariance_matrix: result.covariance.clone(),
        coefficients: result
          .coefficients
          .iter()
          .map(|c| tabdat_stats::CoefficientEstimate {
            name: c.name.clone(),
            value: c.value,
            standard_error: Some(c.standard_error),
            statistic: c.statistic,
            p_value: c.p_value,
          })
          .collect(),
        observation_count: result.observation_count,
        event_count: None,
        sample: sample.clone(),
        reference_distribution: "asymptotic_standard_normal".into(),
        inference_degrees_of_freedom: None,
        log_likelihood: Some(result.log_likelihood),
        log_likelihood_convention: Some("poisson_count_pmf".into()),
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: None,
        df_residual: Some(result.df_residual),
      }
    }
    EstimationState::NonlinearFit(result) => {
      EstimateSnapshot {
        name: name.into(),
        estimator: "nls".into(),
        outcome: result.model.outcome.clone(),
        nls_model: Some(result.model.clone()),
        nls_centered_r_squared: result.centered_r_squared.filter(|v| v.is_finite()),
        nls_covariance_correction: Some(result.covariance_correction),
        poisson_offset_variable: None,
        poisson_offset_coefficient: None,
        poisson_null_log_likelihood: None,
        poisson_null_model: None,
        poisson_covariance_correction: None,
        poisson_mcfadden_r_squared: None,
        poisson_zero_count: None,
        poisson_count_sum: None,
        predictors: result.model.predictor_names.clone(),
        // NLS has no implicit intercept; the formula alone defines its parameters.
        include_intercept: false,
        covariance: if result.robust {
          "hc1_gauss_newton"
        } else {
          "working_homoskedastic_gauss_newton"
        }
        .into(),
        cluster_variable: None,
        cluster_count: None,
        covariance_parameter_names: result.model.parameter_names.clone(),
        covariance_matrix: result.covariance_matrix.clone(),
        coefficients: result
          .coefficients
          .iter()
          .map(|c| tabdat_stats::CoefficientEstimate {
            name: c.name.clone(),
            value: c.value,
            standard_error: Some(c.standard_error),
            statistic: c.statistic,
            p_value: c.p_value,
          })
          .collect(),
        observation_count: result.observation_count,
        event_count: None,
        sample: result.sample.clone(),
        reference_distribution: "approximate_student_t".into(),
        inference_degrees_of_freedom: Some(result.residual_degrees_of_freedom),
        log_likelihood: None,
        log_likelihood_convention: None,
        // The existing table labels this field as OLS R-squared.
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: Some(result.root_mean_squared_error),
        df_residual: Some(result.residual_degrees_of_freedom),
      }
    }
    _ => {
      return Err(failure(
        "store supports unweighted OLS, logit, probit, Poisson and NLS only",
      ));
    }
  };
  for coefficient in &mut report.coefficients {
    coefficient.standard_error = coefficient.standard_error.filter(|v| v.is_finite());
    coefficient.statistic = coefficient.statistic.filter(|v| v.is_finite());
    coefficient.p_value = coefficient.p_value.filter(|v| v.is_finite());
  }
  if report.coefficients.len() > MAX_COMPARISON_TERMS {
    return Err(failure("store supports at most 64 coefficients per report"));
  }
  if report
    .coefficients
    .iter()
    .map(|c| &c.name)
    .collect::<BTreeSet<_>>()
    .len()
    != report.coefficients.len()
  {
    return Err(failure(
      "coefficient names are ambiguous; rename the conflicting predictor and refit before storing",
    ));
  }
  Ok(report)
}

impl Session {
  pub(crate) fn execute_estimates(
    &mut self,
    command: &EstimatesCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    command.validate().map_err(failure)?;
    match command {
      EstimatesCommand::Store(name) => {
        if self.stored_estimates.contains_key(name) {
          return Err(failure(
            "name already stored; use estimates drop NAME before deliberately reusing it",
          ));
        }
        if self.stored_estimates.len() == MAX_STORED_ESTIMATES {
          return Err(failure(
            "at most 32 reports can be stored; drop an unneeded name first",
          ));
        }
        let state = self.estimation.as_ref().ok_or_else(|| {
          failure("store requires a successful current OLS, logit, probit, Poisson or NLS fit")
        })?;
        let report = snapshot(name, state)?;
        let estimator = report.estimator.clone();
        self.stored_estimates.insert(name.clone(), report);
        Ok(ExecutionResult::EstimateStore(EstimateStoreResult {
          name: name.clone(),
          estimator,
          stored_count: self.stored_estimates.len(),
        }))
      }
      EstimatesCommand::Table(names) | EstimatesCommand::Drop(names) => {
        // Validate the whole request before even a multi-name drop can mutate.
        for name in names {
          if !self.stored_estimates.contains_key(name) {
            return Err(failure(format!(
              "unknown stored estimate: {name}; store it first"
            )));
          }
        }
        if matches!(command, EstimatesCommand::Drop(_)) {
          for name in names {
            self.stored_estimates.remove(name);
          }
          return Ok(ExecutionResult::EstimateDrop(EstimateDropResult {
            names: names.clone(),
            stored_count: self.stored_estimates.len(),
          }));
        }
        let models: Vec<_> = names
          .iter()
          .map(|name| self.stored_estimates[name].clone())
          .collect();
        let mut parameter_names = Vec::new();
        for model in &models {
          for coefficient in &model.coefficients {
            if !parameter_names.contains(&coefficient.name) {
              parameter_names.push(coefficient.name.clone());
            }
          }
        }
        if parameter_names.len() > MAX_COMPARISON_TERMS {
          return Err(failure(
            "comparison union exceeds 64 coefficients; request fewer models",
          ));
        }
        Ok(ExecutionResult::EstimateTable(EstimateTableResult {
          models,
          parameter_names,
        }))
      }
    }
  }
}
