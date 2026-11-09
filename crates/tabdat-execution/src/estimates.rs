use crate::*;

fn failure(message: impl Into<String>) -> RuntimeError {
  RuntimeError::EstimatesFailed {
    message: message.into(),
  }
}

fn validate_panel_report(report: &EstimateSnapshot) -> Result<(), RuntimeError> {
  let Some(panel) = &report.panel else {
    return Ok(());
  };
  let sample = &report.sample;
  let groups = sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| failure("panel group provenance is unavailable; refit before storing"))?;
  let k = report.coefficients.len();
  if sample.weights.is_some()
    || sample.retained_indices.len() != report.observation_count
    || sample
      .retained_indices
      .len()
      .checked_add(sample.dropped_observations)
      != Some(sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .any(|i| *i >= sample.total_observations)
    || sample
      .retained_indices
      .iter()
      .collect::<BTreeSet<_>>()
      .len()
      != report.observation_count
    || groups.len() != report.observation_count
    || groups.iter().collect::<BTreeSet<_>>().len() != panel.group_count
    || panel.group_count < 2
    || panel.entity_variable.is_empty()
    || panel.time_variable.as_ref().is_some_and(String::is_empty)
    || report.coefficients.iter().any(|c| !c.value.is_finite())
    || report.covariance_matrix.len() != k
    || report
      .covariance_matrix
      .iter()
      .enumerate()
      .any(|(i, row)| row.len() != k || row.iter().any(|v| !v.is_finite()) || row[i] < 0.0)
    || report.covariance_parameter_names
      != report
        .coefficients
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
    || report.covariance_parameter_names
      != std::iter::once("intercept".to_owned())
        .chain(report.predictors.iter().cloned())
        .collect::<Vec<_>>()
  {
    return Err(failure(
      "panel report provenance/dimensions are inconsistent or weighted; refit an unweighted supported model before storing",
    ));
  }
  Ok(())
}

fn snapshot(
  name: &str,
  state: &EstimationState,
  selection: Option<GaussianGeeReportCovariance>,
) -> Result<EstimateSnapshot, RuntimeError> {
  if selection.is_some() && !matches!(state, EstimationState::GaussianGee(_)) {
    return Err(failure(
      "covariance(working|sandwich) is only available for Gaussian GEE reports",
    ));
  }
  let mut report = match state {
    EstimationState::Linear(state) => {
      if state.estimator.as_deref() != Some("ols") {
        return Err(failure(
          "store supports unweighted OLS for linear reports; refit if estimator provenance is unavailable",
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
        panel: None,
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
        nb2: None,
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
        panel: None,
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
        nb2: None,
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
        panel: None,
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
        nb2: None,
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
    EstimationState::NegativeBinomialFit(state) => {
      let result = &state.result;
      let sample = result
        .sample
        .as_ref()
        .ok_or_else(|| failure("fit sample is unavailable; refit before storing"))?;
      if sample.weights.is_some() {
        return Err(failure("weighted NB2 reports are not installed"));
      }
      let mean_parameters = result.coefficients.len();
      let joint_parameters = mean_parameters + 1;
      if result.covariance.len() != joint_parameters
        || result.covariance.iter().enumerate().any(|(i, row)| {
          row.len() != joint_parameters
            || row.iter().any(|v| !v.is_finite())
            || row[i] < 0.0
        })
        || result.observation_count != sample.retained_indices.len()
        || result
          .coefficients
          .iter()
          .any(|c| !c.value.is_finite() || !c.standard_error.is_finite())
        || !result.log_likelihood.is_finite()
        || !result.null_log_likelihood.is_finite()
        || !result.dispersion.alpha.is_finite()
        || result.dispersion.alpha <= 0.0
        || !result.dispersion.log_alpha.is_finite()
        || !result.dispersion.log_alpha_standard_error.is_finite()
        || result.dispersion.log_alpha_standard_error < 0.0
        || !result.dispersion.alpha_standard_error.is_finite()
        || !result.mcfadden_r_squared.is_finite()
      {
        return Err(failure(
          "NB2 joint fit provenance/dimensions are inconsistent; refit before storing",
        ));
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
          "cluster_joint_score_sandwich_g_over_g_minus_1",
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
          "robust_joint_score_sandwich_n_over_n_minus_1",
          n as f64 / (n - 1) as f64,
        )
      } else {
        ("model_based_joint_observed_information", 1.0)
      };
      let mut coefficients: Vec<_> = result
        .coefficients
        .iter()
        .map(|c| tabdat_stats::CoefficientEstimate {
          name: c.name.clone(),
          value: c.value,
          standard_error: Some(c.standard_error),
          statistic: c.statistic,
          p_value: c.p_value,
        })
        .collect();
      coefficients.push(tabdat_stats::CoefficientEstimate {
        name: "lnalpha".into(),
        value: result.dispersion.log_alpha,
        standard_error: Some(result.dispersion.log_alpha_standard_error),
        // The retained dispersion test is a one-sided boundary LR, not a
        // two-sided normal-Wald statistic, so no z/p is invented here.
        statistic: None,
        p_value: None,
      });
      EstimateSnapshot {
        panel: None,
        name: name.into(),
        estimator: "nb2".into(),
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
        nb2: Some(Nb2EstimateDetails {
          offset_variable: result.offset_variable.clone(),
          covariance_correction: correction,
          zero_count: result.zero_count,
          count_sum: result.count_sum,
          null_model: match (result.offset_variable.is_some(), result.include_intercept) {
            (true, true) => "intercept_only_with_same_fixed_offset_refit_alpha",
            (true, false) => "unit_mean_with_same_fixed_offset_refit_alpha",
            (false, true) => "intercept_only_estimated_mean_refit_alpha",
            (false, false) => "unit_mean_all_coefficients_zero_refit_alpha",
          }
          .into(),
          null_log_likelihood: result.null_log_likelihood,
          null_alpha: result.null_alpha,
          mcfadden_r_squared: result.mcfadden_r_squared,
          alpha: result.dispersion.alpha,
          alpha_standard_error: result.dispersion.alpha_standard_error,
          alpha_confidence_lower: result.dispersion.alpha_confidence_lower,
          alpha_confidence_upper: result.dispersion.alpha_confidence_upper,
          dispersion_test: result.dispersion_test.clone(),
        }),
        predictors: result.predictor_names.clone(),
        include_intercept: result.include_intercept,
        covariance: covariance.into(),
        cluster_variable: result.cluster.as_ref().map(|c| c.variable.clone()),
        cluster_count: result.cluster.as_ref().map(|c| c.group_count),
        covariance_parameter_names: coefficients.iter().map(|c| c.name.clone()).collect(),
        covariance_matrix: result.covariance.clone(),
        coefficients,
        observation_count: result.observation_count,
        event_count: None,
        sample: sample.clone(),
        reference_distribution: "asymptotic_standard_normal".into(),
        inference_degrees_of_freedom: None,
        log_likelihood: Some(result.log_likelihood),
        log_likelihood_convention: Some("nb2_count_pmf".into()),
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: None,
        df_residual: Some(result.df_residual),
      }
    }
    EstimationState::FixedEffects(state) => {
      let result = &state.result;
      let fit = &result.within;
      if fit.covariance.covariance_type != tabdat_stats::CovarianceType::NonRobust {
        return Err(failure(
          "panel FE reports currently require model-based covariance and normalized common-intercept inference",
        ));
      }
      let model = result.normalized_level.as_ref().ok_or_else(|| failure("FE normalized common-intercept inference is unavailable; refit a supported model before storing"))?;
      if model.parameters.len() != fit.coefficients.len() + 1
        || model.parameter_names.first().map(String::as_str) != Some("_cons")
        || model.parameter_names[1..] != fit.parameter_names
        || model.parameters[0] != result.common_intercept
        || model.parameters[1..] != fit.coefficients.iter().map(|c| c.value).collect::<Vec<_>>()
        || model.degrees_of_freedom != fit.degrees_of_freedom()
        || model.covariance.covariance_type != tabdat_stats::CovarianceType::NonRobust
        || model.covariance.parameter_names != model.parameter_names
        || model.parameters.iter().any(|v| !v.is_finite())
        || model.covariance.matrix.len() != model.parameters.len()
        || model.covariance.matrix.iter().enumerate().any(|(i, row)| {
          row.len() != model.parameters.len() || row.iter().any(|v| !v.is_finite()) || row[i] < 0.0
        })
      {
        return Err(failure(
          "FE normalized coefficient provenance is inconsistent; refit before storing",
        ));
      }
      // This is the same retained normalization used by native FE reporting,
      // not a pooled refit or a replacement inference for the within slopes.
      let mut weights = vec![0.0; model.parameters.len()];
      weights[0] = 1.0;
      let common = model
        .linear_combination_with_inference(&weights, 0.0, 95.0)
        .map_err(|e| failure(format!("FE common-intercept inference unavailable: {e}")))?;
      let mut coefficients = vec![tabdat_stats::CoefficientEstimate {
        name: "intercept".into(),
        value: common.estimate,
        standard_error: Some(common.standard_error),
        statistic: common.statistic,
        p_value: Some(common.p_value),
      }];
      coefficients.extend(fit.coefficients.clone());
      EstimateSnapshot {
        panel: Some(PanelEstimateProvenance {
          entity_variable: result.entity_variable.clone(),
          time_variable: Some(state.time_variable.clone()),
          group_count: result.effects.len(),
          intercept_convention: "observation_weighted_zero_effects".into(),
        }),
        name: name.into(),
        estimator: "within_fixed_effects".into(),
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
        nb2: None,
        predictors: fit.parameter_names.clone(),
        include_intercept: true,
        covariance: "model_based_within_fe_normalized_levels".into(),
        cluster_variable: None,
        cluster_count: None,
        covariance_parameter_names: coefficients.iter().map(|c| c.name.clone()).collect(),
        covariance_matrix: model.covariance.matrix.clone(),
        coefficients,
        observation_count: fit.observation_count(),
        event_count: None,
        sample: fit
          .sample
          .clone()
          .ok_or_else(|| failure("FE fit sample is unavailable; refit before storing"))?,
        reference_distribution: "student_t".into(),
        inference_degrees_of_freedom: Some(fit.degrees_of_freedom()),
        log_likelihood: None,
        log_likelihood_convention: None,
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: None,
        df_residual: Some(fit.degrees_of_freedom()),
      }
    }
    EstimationState::RandomEffects(state) => {
      use tabdat_stats::RandomEffectsComponentMethod as Method;
      let result = &state.result;
      let fit = &result.gls;
      if fit.covariance.covariance_type != tabdat_stats::CovarianceType::NonRobust
        || result.ml.as_ref().is_some_and(|ml| ml.cluster.is_some())
      {
        return Err(failure(
          "panel RE reports currently require model-based covariance",
        ));
      }
      if fit.parameter_names.first().map(String::as_str) != Some("intercept")
        || (result.ml.is_some() != (result.component_method == Method::GaussianMarginalMl))
        || result
          .ml
          .as_ref()
          .is_some_and(|ml| !ml.log_likelihood.is_finite())
      {
        return Err(failure(
          "RE method/intercept provenance is unavailable or inconsistent; refit before storing",
        ));
      }
      let estimator = match result.component_method {
        Method::SwamyArora => "swamy_arora_random_effects_gls",
        Method::EqualGroupBetween => "equal_group_between_random_effects_gls",
        Method::GaussianMarginalMl => "gaussian_random_intercept_marginal_ml",
      };
      EstimateSnapshot {
        panel: Some(PanelEstimateProvenance {
          entity_variable: result.entity_variable.clone(),
          time_variable: Some(state.time_variable.clone()),
          group_count: result.group_count,
          intercept_convention: "population_mean_intercept".into(),
        }),
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
        nb2: None,
        predictors: result.predictor_names.clone(),
        include_intercept: true,
        covariance: if result.ml.is_some() {
          "model_based_observed_marginal_information"
        } else {
          "model_based_quasi_gls_rss_over_n_minus_k"
        }
        .into(),
        cluster_variable: None,
        cluster_count: None,
        covariance_parameter_names: fit.covariance.parameter_names.clone(),
        covariance_matrix: fit.covariance.matrix.clone(),
        coefficients: fit.coefficients.clone(),
        observation_count: fit.observation_count(),
        event_count: None,
        sample: fit
          .sample
          .clone()
          .ok_or_else(|| failure("RE fit sample is unavailable; refit before storing"))?,
        reference_distribution: "asymptotic_standard_normal".into(),
        inference_degrees_of_freedom: None,
        log_likelihood: result.ml.as_ref().map(|ml| ml.log_likelihood),
        log_likelihood_convention: result
          .ml
          .as_ref()
          .map(|_| "gaussian_random_intercept_marginal_ml_full_constants".into()),
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: None,
        df_residual: Some(fit.degrees_of_freedom()),
      }
    }
    EstimationState::GaussianGee(result) => {
      let selected = selection.ok_or_else(|| failure("Gaussian GEE has two covariance inferences; use estimates store NAME, covariance(working) or covariance(sandwich)"))?;
      let (inference, covariance, clustered) = match selected {
        GaussianGeeReportCovariance::Working => {
          (&result.working, "working_gaussian_exchangeable", false)
        }
        GaussianGeeReportCovariance::Sandwich => {
          (&result.sandwich, "unit_sandwich_uncorrected", true)
        }
      };
      if inference.coefficients.first().map(|c| c.name.as_str()) != Some("_cons") {
        return Err(failure(
          "Gaussian GEE intercept provenance is unavailable; refit before storing",
        ));
      }
      let coefficients: Vec<_> = inference
        .coefficients
        .iter()
        .enumerate()
        .map(|(i, c)| tabdat_stats::CoefficientEstimate {
          name: if i == 0 {
            "intercept".into()
          } else {
            c.name.clone()
          },
          value: c.value,
          standard_error: Some(c.standard_error),
          statistic: Some(c.statistic),
          p_value: Some(c.p_value),
        })
        .collect();
      EstimateSnapshot {
        panel: Some(PanelEstimateProvenance {
          entity_variable: result.entity_variable.clone(),
          time_variable: None,
          group_count: result.group_count,
          intercept_convention: "population_mean_intercept".into(),
        }),
        name: name.into(),
        estimator: "gaussian_exchangeable_gee".into(),
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
        nb2: None,
        predictors: result.predictor_names.clone(),
        include_intercept: true,
        covariance: covariance.into(),
        cluster_variable: clustered.then(|| result.entity_variable.clone()),
        cluster_count: clustered.then_some(result.group_count),
        covariance_parameter_names: coefficients.iter().map(|c| c.name.clone()).collect(),
        covariance_matrix: inference.covariance.clone(),
        coefficients,
        observation_count: result.observation_count,
        event_count: None,
        sample: result.sample.clone(),
        reference_distribution: "asymptotic_standard_normal".into(),
        inference_degrees_of_freedom: None,
        log_likelihood: None,
        log_likelihood_convention: None,
        r_squared: None,
        mcfadden_r_squared: None,
        root_mse: None,
        df_residual: None,
      }
    }
    EstimationState::NonlinearFit(result) => {
      EstimateSnapshot {
        panel: None,
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
        nb2: None,
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
        "store supports unweighted OLS, logit, probit, Poisson, NB2, NLS, model-based FE/RE, and explicitly selected Gaussian GEE reports",
      ));
    }
  };
  validate_panel_report(&report)?;
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
      EstimatesCommand::Store(name) | EstimatesCommand::StoreGaussianGee { name, .. } => {
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
          failure("store requires a successful current supported fit; Gaussian GEE additionally requires covariance(working|sandwich)")
        })?;
        let selection = match command {
          EstimatesCommand::StoreGaussianGee { covariance, .. } => Some(*covariance),
          _ => None,
        };
        let report = snapshot(name, state, selection)?;
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

#[cfg(test)]
mod tests {
  use super::*;

  fn problem() -> tabdat_stats::EstimationProblem {
    let rows: Vec<_> =
      include_str!("../../../tests/fixtures/mma24_household_mixed_report/balanced/input.csv")
        .lines()
        .skip(1)
        .map(|l| l.split(',').collect::<Vec<_>>())
        .collect();
    let sample = tabdat_stats::EstimationSample::new(
      (0..rows.len()).collect(),
      rows.len(),
      0,
      None,
      Some(rows.iter().map(|r| r[8].to_owned()).collect()),
    )
    .unwrap();
    tabdat_stats::EstimationProblem::new(
      "lnexp12m",
      ["lnhhexp", "age", "sex", "hhsize", "farm", "educ"]
        .map(str::to_owned)
        .to_vec(),
      rows.iter().map(|r| r[1].parse().unwrap()).collect(),
      rows
        .iter()
        .map(|r| [7, 2, 3, 4, 5, 6].map(|j| r[j].parse().unwrap()).to_vec())
        .collect(),
    )
    .unwrap()
    .with_sample(sample)
  }
  fn states() -> Vec<EstimationState> {
    let p = problem();
    vec![
      EstimationState::FixedEffects(Box::new(FixedEffectsState {
        time_variable: "source_row".into(),
        entity_data_type: "BIGINT".into(),
        result: tabdat_stats::fit_fixed_effects(&p, "commune", false).unwrap(),
      })),
      EstimationState::RandomEffects(Box::new(RandomEffectsState {
        time_variable: "source_row".into(),
        entity_data_type: "BIGINT".into(),
        result: tabdat_stats::fit_random_effects_equal_group_between(&p, "commune").unwrap(),
        hausman_problem: None,
      })),
      // Match the installed GEE frontend's explicit intercept label.
      EstimationState::GaussianGee(Box::new(
        tabdat_stats::fit_gaussian_gee(
          &p.clone().with_intercept_name("_cons"),
          "commune",
          &Default::default(),
        )
        .unwrap(),
      )),
    ]
  }
  fn request(state: &EstimationState) -> EstimatesCommand {
    if matches!(state, EstimationState::GaussianGee(_)) {
      EstimatesCommand::StoreGaussianGee {
        name: "owned".into(),
        covariance: GaussianGeeReportCovariance::Working,
      }
    } else {
      EstimatesCommand::Store("owned".into())
    }
  }
  #[test]
  fn owned_panel_store_and_table_need_no_active_data_or_backend() {
    for state in states() {
      let command = request(&state);
      let mut s = Session::new();
      s.estimation = Some(state);
      let before = s.state();
      s.execute(Command::Estimates { command }).unwrap();
      assert_eq!(s.state().estimation, before.estimation);
      assert_eq!(s.state().active_dataset, before.active_dataset);
      s.estimation = None;
      let before = s.state();
      let ExecutionResult::EstimateTable(table) = s
        .execute(Command::Estimates {
          command: EstimatesCommand::Table(vec!["owned".into()]),
        })
        .unwrap()
      else {
        panic!()
      };
      assert_eq!(table.models[0].observation_count, 192);
      assert_eq!(s.state(), before);
      assert!(s.backend.is_none());
    }
  }
  #[test]
  fn missing_or_inconsistent_panel_provenance_fails_before_registry_effects() {
    for original in states() {
      for mode in 0..8 {
        let mut state = original.clone();
        match &mut state {
          EstimationState::FixedEffects(s) => match mode {
            0 => s.result.normalized_level = None,
            1 => {
              s.result
                .normalized_level
                .as_mut()
                .unwrap()
                .covariance
                .matrix
                .pop();
            }
            2 => {
              s.result
                .normalized_level
                .as_mut()
                .unwrap()
                .degrees_of_freedom += 1
            }
            3 => s.result.within.sample.as_mut().unwrap().cluster_groups = None,
            4 => s.result.within.sample.as_mut().unwrap().retained_indices[0] = usize::MAX,
            5 => s.result.within.sample.as_mut().unwrap().weights = Some(vec![1.0; 192]),
            6 => {
              s.result
                .normalized_level
                .as_mut()
                .unwrap()
                .covariance
                .parameter_names[0] = "wrong".into()
            }
            _ => {
              s.result
                .normalized_level
                .as_mut()
                .unwrap()
                .covariance
                .matrix[0][0] = -1.0
            }
          },
          EstimationState::RandomEffects(s) => match mode {
            0 => {
              s.result.component_method =
                tabdat_stats::RandomEffectsComponentMethod::GaussianMarginalMl
            }
            1 => s.result.gls.sample = None,
            2 => {
              s.result.gls.covariance.matrix.pop();
            }
            3 => s.result.gls.sample.as_mut().unwrap().cluster_groups = None,
            4 => s.result.gls.sample.as_mut().unwrap().retained_indices[0] = usize::MAX,
            5 => s.result.gls.sample.as_mut().unwrap().weights = Some(vec![1.0; 192]),
            6 => s.result.predictor_names.reverse(),
            _ => s.result.gls.covariance.matrix[0][0] = -1.0,
          },
          EstimationState::GaussianGee(s) => match mode {
            0 => s.working.coefficients[0].name = "wrong".into(),
            1 => {
              s.working.covariance.pop();
            }
            2 => s.sample.total_observations += 1,
            3 => s.sample.cluster_groups = None,
            4 => s.sample.retained_indices[0] = usize::MAX,
            5 => s.sample.weights = Some(vec![1.0; 192]),
            6 => s.group_count += 1,
            _ => s.working.covariance[0][0] = -1.0,
          },
          _ => unreachable!(),
        }
        let command = request(&state);
        let mut s = Session::new();
        s.estimation = Some(state);
        let before = s.state();
        assert!(
          s.execute(Command::Estimates { command }).is_err(),
          "mode {mode}"
        );
        assert_eq!(s.state(), before);
        assert!(s.backend.is_none());
      }
    }
  }
}
