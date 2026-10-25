//! JSON result envelopes for the bounded CLI execution surface.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Number, Value, json};
use tabdat_domain::{
  CellValue, CodebookResult, CollapseStatistic, DatasetInfo, ExecutionResult, LoadResult,
  MissingResult, PreviewResult, RegressionResult, SummarizeResult, SummaryRow, TtestGroupStats,
  TtestResult, summarize_detail_requested,
};
use tabdat_language::{Command, EstatCommand, EstatSubcommand, ExecutionMode, LazyEngine};

use crate::catalog::ResultEnvelope;

pub const BATCH_UNSUPPORTED_MESSAGE: &str = "--json successful batch results are currently supported only for use, describe, status, count, collapse count/mean, duplicates, isid, assert, datasignature, codebook, summarize, missing, tabulate, by ...: count/summarize (native means), generate, replace, rename, recode, label, encode, decode, sql, join, append, reshape, keep/drop/select variable lists, bounded keep/drop if, sort, gsort, save, export, head, tail, ivregress 2sls, regress, logit, probit, poisson, nbreg, qreg, bootstrap, predict, lincom, test, ttest, estat vif/ar/overid/endogenous/margins/countprob, scatter (SVG saving/noopen), estimates store/table/drop, and run (supported script leaves only), and xtreg fe/re/mle with explicit id()/time()";
pub const SCRIPT_UNSUPPORTED_MESSAGE: &str = "--json successful script results are currently supported only for use, describe, status, count, collapse count/mean, duplicates, isid, assert, datasignature, codebook, summarize, missing, tabulate, by ...: count/summarize (native means), generate, replace, rename, recode, label, encode, decode, sql, join, append, reshape, keep/drop/select variable lists, bounded keep/drop if, sort, gsort, save, export, head, tail, ivregress 2sls, regress, logit, probit, poisson, nbreg, qreg, bootstrap, predict, lincom, test, ttest, scatter (SVG saving/noopen), estimates store/table/drop, and estat vif/ar/overid/endogenous/margins/countprob, and xtreg fe/re/mle with explicit id()/time()";

fn binary_wald_json(test: Option<&tabdat_domain::BinaryOmnibusWald>) -> Value {
  test.map_or(Value::Null, |test| {
    json!({
      "null_hypothesis": "all_non_intercept_coefficients_zero",
      "statistic": test.statistic,
      "degrees_of_freedom": test.degrees_of_freedom,
      "p_value": test.p_value,
      "reference_distribution": test.statistic.map(|_| "asymptotic_chi_square"),
      "unavailable_reason": test.unavailable_reason,
    })
  })
}

fn add_count_cluster_metadata(
  data: &mut Value,
  cluster: Option<&tabdat_domain::BinaryCluster>,
  covariance: &str,
) {
  if let Some(c) = cluster {
    data["covariance"] = json!(covariance);
    data["cluster"] = json!({"variable": c.variable, "group_count": c.group_count,
      "finite_sample_multiplier": c.group_count as f64/(c.group_count-1) as f64,
      "independence_assumption": "across_clusters"});
  }
}

/// Whether this command can execute in the JSON batch CLI. `run` streams
/// supported script leaves without a container envelope; `exit` has no result.
/// ByResult retains native grouped AVG semantics; it is not a finite-sample
/// SummarizeResult. Conditional/option-bearing grouped summaries stay unsupported.
pub fn supports_batch_command(command: &Command) -> bool {
  match command {
    Command::Heckman { command } => command.twostep,
    Command::TabulateQuery {
      command,
      aggregation: None,
      ..
    } => command.row_variables.len() == 1 && command.column_variables.len() <= 1,
    Command::Collapse { command } => matches!(
      command.statistic,
      CollapseStatistic::Count | CollapseStatistic::Mean
    ),
    Command::SummarizeQuery { options, .. } => {
      options.is_empty() || summarize_detail_requested(options)
    }
    Command::By { command } => matches!(
      command.command.as_ref(),
      Command::Count | Command::Summarize { .. }
    ),
    Command::IvRegress { command } => {
      command.estimator == tabdat_domain::IvEstimator::TwoStageLeastSquares
    }
    Command::XtReg { .. } => true,
    Command::Status
    | Command::RegressQuery { .. }
    | Command::CountQuery { .. }
    | Command::Assert { .. }
    | Command::Datasignature
    | Command::Duplicates { .. }
    | Command::Isid { .. }
    | Command::RowFilter { .. }
    | Command::Reshape { .. }
    | Command::Append { .. }
    | Command::Join { .. }
    | Command::Sql { .. }
    | Command::Encode { .. }
    | Command::Decode { .. }
    | Command::Label { .. }
    | Command::Recode { .. }
    | Command::Rename { .. }
    | Command::Sort { .. }
    | Command::Gsort { .. }
    | Command::Use { .. }
    | Command::Generate { .. }
    | Command::Replace { .. }
    | Command::Keep { .. }
    | Command::Drop { .. }
    | Command::Select { .. }
    | Command::Save { .. }
    | Command::Export { .. }
    | Command::Scatter { .. }
    | Command::Estimates { .. }
    | Command::Describe
    | Command::Count
    | Command::Codebook { .. }
    | Command::Summarize { .. }
    | Command::Missing { .. }
    | Command::Tabulate { .. }
    | Command::Head { .. }
    | Command::Tail { .. }
    | Command::Regress { .. }
    | Command::Bootstrap { .. }
    | Command::QuantileBootstrap { .. }
    | Command::Logit { .. }
    | Command::Probit { .. }
    | Command::Mlogit { .. }
    | Command::Clogit { .. }
    | Command::Nlogit { .. }
    | Command::NlogitNormalizedCommon(..)
    | Command::Tobit { .. }
    | Command::Streg { .. }
    | Command::Stcox { .. }
    | Command::Truncreg { .. }
    | Command::Poisson { .. }
    | Command::Nbreg { .. }
    | Command::Qreg { .. }
    | Command::Predict { .. }
    | Command::Lincom { .. }
    | Command::Test { .. }
    | Command::Ttest { .. }
    | Command::Estat {
      command:
        EstatCommand {
          subcommand:
            EstatSubcommand::Vif
            | EstatSubcommand::AndersonRubin { .. }
            | EstatSubcommand::AndersonRubinConfidenceSet { .. }
            | EstatSubcommand::AndersonRubinProjectedConfidenceSet { .. }
            | EstatSubcommand::Overid
            | EstatSubcommand::Endogenous
            | EstatSubcommand::Margins
            | EstatSubcommand::CountProbabilities { .. },
          ..
        },
    }
    | Command::Exit
    | Command::Run { .. } => true,
    _ => false,
  }
}

/// Whether a script command can execute without producing an unsupported JSON
/// success result. Scripts share the batch admission policy, including `run`
/// containers (whose leaves are checked individually) and result-free `exit`.
pub fn supports_script_command(command: &Command) -> bool {
  supports_batch_command(command)
}

fn bootstrap_json(result: &tabdat_domain::BootstrapOlsResult) -> Value {
  let mut data = serde_json::to_value(result).expect("finite bootstrap result");
  data["estimator"] = json!("iid_pairs_bootstrap_ols");
  // Decimal text preserves every u64 seed in JavaScript/IEEE754 consumers.
  data["seed"] = json!(result.seed.to_string());
  data["sample"] = match &result.sample {
    Some(s) => {
      json!({"total_observations":s.total_observations,"dropped_observations":s.dropped_observations})
    }
    None => Value::Null,
  };
  data
}

/// Serialize owned execution results using the stable versioned CLI envelope.
/// CLI callers must apply command admission first: shared DTOs (such as ByResult)
/// do not imply that every command producing them is supported in JSON execution.
///
/// `None` means there is no leaf envelope: control containers such as `run`
/// are handled by the caller; other unsupported commands must be rejected before
/// execution by [`supports_batch_command`].
pub fn serialize_execution_result(result: &ExecutionResult) -> Option<String> {
  let (data, result_type) = match result {
    ExecutionResult::Quantile(result) => {
      let mut data = serde_json::to_value(result).expect("finite quantile result");
      data["estimator"] = json!("linear_conditional_quantile");
      data["covariance"] = json!("iid_location_hall_sheather_residual_spacing");
      data["reference_distribution"] = json!("asymptotic_standard_normal");
      data["confidence_level"] = json!(95);
      data["solver"] = json!("clarabel_0.11.1_rust_qdldl_dual_lp_qr_polish");
      data["sample"] = result.sample.as_ref().map_or(Value::Null,|s|json!({"total_observations":s.total_observations,"dropped_observations":s.dropped_observations}));
      (data, "QuantileRegressionResult")
    }
    ExecutionResult::QuantileBootstrap(result) => {
      let mut data = serde_json::to_value(result).expect("finite quantile bootstrap");
      data["estimator"] = json!("iid_pairs_bootstrap_joint_quantiles");
      data["seed"] = json!(result.seed.to_string());
      data["sample"] = match &result.sample {
        Some(s) => {
          json!({"total_observations":s.total_observations,"dropped_observations":s.dropped_observations})
        }
        None => Value::Null,
      };
      data["covariance_convention"] = json!("common_iid_pairs_b_minus_1");
      data["simultaneous_coverage"] = json!(false);
      (data, "QuantileBootstrapResult")
    }
    ExecutionResult::ClusterBootstrap(result) => {
      let mut data = bootstrap_json(&result.bootstrap);
      data["estimator"] = json!("cluster_pairs_bootstrap_ols");
      data["cluster_count"] = json!(result.cluster_count);
      data["cluster_sizes"] = json!(result.cluster_sizes);
      data["cluster_order"] = json!("first_seen_complete_case");
      data["replicate_observation_counts"] = json!(result.replicate_observation_counts);
      (data, "ClusterBootstrapOlsResult")
    }
    ExecutionResult::Bootstrap(result) => {
      let data = bootstrap_json(result);
      (data, "BootstrapOlsResult")
    }
    ExecutionResult::RobustEndogeneity(result) => {
      let mut data = serde_json::to_value(result).expect("finite robust endogeneity result");
      data["test"] = json!("endogeneity_augmented_regression_robust_wald");
      data.as_object_mut().unwrap().remove("covariance_type");
      data["covariance"] = json!(match &result.covariance_type {
        tabdat_domain::CovarianceType::RobustHc1 => "robust_hc1".to_owned(),
        tabdat_domain::CovarianceType::Cluster(variable) => format!("cluster({variable})"),
        tabdat_domain::CovarianceType::NonRobust => "homoskedastic".to_owned(),
      });
      data["reference_distribution"] = json!("asymptotic_chi_square");
      data["null_scope"] = json!("all_endogenous_joint");
      data["null_hypothesis"] = json!("all_declared_endogenous_regressors_exogenous");
      data["weak_iv_robust"] = json!(false);
      data["assumptions"] = json!(
        "valid_instruments_maintained_linear_model_strong_identification_independent_rows_or_independent_clusters_many_clusters_required_no_few_cluster_guarantee"
      );
      (data, "RobustEndogeneityResult")
    }
    ExecutionResult::WuHausman(result) => {
      let mut data = serde_json::to_value(result).expect("finite Wu-Hausman result");
      data["test"] = json!("wu_hausman_augmented_regression_f");
      data["covariance"] = json!("homoskedastic");
      data["null_scope"] = json!("all_endogenous_joint");
      data["null_hypothesis"] = json!("all_declared_endogenous_regressors_exogenous");
      data["weak_iv_robust"] = json!(false);
      data["assumptions"] = json!(
        "valid_instruments_homoskedastic_errors_maintained_model_strong_identification_exact_F_requires_conditional_normality"
      );
      (data, "WuHausmanResult")
    }
    ExecutionResult::RobustOveridScore(result) => {
      let mut data = serde_json::to_value(result).expect("finite robust overidentification score");
      data["test"] = json!("wooldridge_residualized_instrument_overid_score");
      data["covariance"] = json!("independent_row_uncentered_score_opg");
      data["reference_distribution"] = json!("asymptotic_chi_square");
      data["weak_iv_robust"] = json!(false);
      data["null_hypothesis"] = json!("overidentifying_restrictions_hold_given_maintained_model");
      data["assumptions"] = json!(
        "independent_rows_correctly_specified_maintained_model_valid_instruments_under_null_strong_identification"
      );
      (data, "RobustOveridScoreResult")
    }
    ExecutionResult::Sargan(result) => {
      let mut data = serde_json::to_value(result).expect("finite Sargan result");
      data["test"] = json!("sargan_n_r_squared");
      data["covariance"] = json!("homoskedastic");
      data["reference_distribution"] = json!("asymptotic_chi_square");
      data["weak_iv_robust"] = json!(false);
      data["null_hypothesis"] = json!("overidentifying_restrictions_hold_given_maintained_model");
      data["assumptions"] =
        json!("homoskedastic_errors_correctly_specified_maintained_model_strong_identification");
      (data, "SarganResult")
    }
    ExecutionResult::AndersonRubinConfidenceSet(result) => {
      let mut data = serde_json::to_value(result).expect("serializable AR confidence set");
      data["test"] = json!("anderson_rubin_scalar_confidence_set");
      data["covariance"] = json!("homoskedastic");
      data["weak_iv_robust"] = json!(true);
      data["endogenous_nuisance_policy"] = json!("none_single_endogenous_only");
      data["assumptions"] =
        json!("valid_instruments_homoskedastic_errors_exact_F_requires_conditional_normality");
      (data, "AndersonRubinConfidenceSet")
    }
    ExecutionResult::ProjectedAndersonRubinConfidenceSet(result) => {
      let mut data = serde_json::to_value(result).expect("serializable projected AR set");
      data["test"] = json!("anderson_rubin_projected_scalar_confidence_set");
      data["covariance"] = json!("homoskedastic");
      data["weak_iv_robust"] = json!(true);
      data["coverage"] = json!("at_least_confidence_level_by_projection");
      data["endogenous_nuisance_policy"] = json!("project_over_all_other_endogenous_coefficients");
      data["assumptions"] =
        json!("valid_instruments_homoskedastic_errors_exact_F_requires_conditional_normality");
      (data, "ProjectedAndersonRubinConfidenceSet")
    }
    ExecutionResult::RobustAndersonRubin(result) => {
      let mut data = serde_json::to_value(result).expect("finite robust AR result");
      data["test"] = json!("anderson_rubin_joint_robust_wald");
      data.as_object_mut().unwrap().remove("covariance_type");
      data["covariance"] = json!(match &result.covariance_type {
        tabdat_domain::CovarianceType::RobustHc1 => "robust_hc1".to_owned(),
        tabdat_domain::CovarianceType::Cluster(variable) => format!("cluster({variable})"),
        tabdat_domain::CovarianceType::NonRobust => "homoskedastic".to_owned(),
      });
      data["reference_distribution"] = json!("asymptotic_chi_square");
      data["null_scope"] = json!("all_endogenous_joint");
      data["weak_iv_robust"] = json!(true);
      data["assumptions"] = json!(
        "valid_instruments_correct_linear_model_independent_rows_or_independent_clusters_many_clusters_required_no_few_cluster_guarantee"
      );
      (data, "RobustAndersonRubinResult")
    }
    ExecutionResult::AndersonRubin(result) => {
      let mut data = serde_json::to_value(result).expect("finite AR result");
      data["test"] = json!("anderson_rubin_joint");
      data["covariance"] = json!("homoskedastic");
      data["null_scope"] = json!("all_endogenous_joint");
      data["weak_iv_robust"] = json!(true);
      data["assumptions"] =
        json!("valid_instruments_homoskedastic_errors_exact_F_requires_conditional_normality");
      (data, "AndersonRubinResult")
    }
    ExecutionResult::Status(result) => (
      json!({
        "data_initialized": result.data_initialized,
        "dataset": result.dataset.as_ref().map(dataset_info_data),
        "active_table_name": result.active_table_name,
        "named_tables": result.named_tables,
        "estimation_family": result.estimation_family,
        "linear_model": result.linear_model,
      }),
      "StatusResult",
    ),
    ExecutionResult::Load(load) => (load_result_data(load), "LoadResult"),
    ExecutionResult::SqlCreate(result) => (
      json!({"table_name": result.table_name, "dataset": dataset_info_data(&result.dataset)}),
      "SqlCreateResult",
    ),
    ExecutionResult::Activate(result) => (
      json!({"table_name": result.table_name, "dataset": dataset_info_data(&result.dataset)}),
      "ActivateResult",
    ),
    ExecutionResult::Save(result) => (
      persistence_result_data(&result.path, &result.dataset),
      "SaveResult",
    ),
    ExecutionResult::Export(result) => (
      persistence_result_data(&result.path, &result.dataset),
      "ExportResult",
    ),
    ExecutionResult::Plot(result) => (json!(result), "PlotArtifactResult"),
    ExecutionResult::EstimateStore(result) => (json!(result), "EstimateStoreResult"),
    ExecutionResult::EstimateTable(result) => (json!(result), "EstimateTableResult"),
    ExecutionResult::EstimateDrop(result) => (json!(result), "EstimateDropResult"),
    ExecutionResult::Generate(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "GenerateResult",
    ),
    ExecutionResult::Replace(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "ReplaceResult",
    ),
    ExecutionResult::Keep(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "KeepResult",
    ),
    ExecutionResult::Drop(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DropResult",
    ),
    ExecutionResult::Select(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "SelectResult",
    ),
    ExecutionResult::Reshape(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "ReshapeResult",
    ),
    ExecutionResult::By(result) => (table_result_data(&result.headers, &result.rows), "ByResult"),
    ExecutionResult::Append(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "AppendResult",
    ),
    ExecutionResult::Join(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "JoinResult",
    ),
    ExecutionResult::Encode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "EncodeResult",
    ),
    ExecutionResult::Decode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DecodeResult",
    ),
    ExecutionResult::Label(result) => (
      json!({"action": result.action, "message": result.message, "metadata": result.metadata}),
      "LabelResult",
    ),
    ExecutionResult::Recode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "RecodeResult",
    ),
    ExecutionResult::Rename(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "RenameResult",
    ),
    ExecutionResult::Sort(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "SortResult",
    ),
    ExecutionResult::Gsort(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "GsortResult",
    ),
    ExecutionResult::Describe(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DescribeResult",
    ),
    ExecutionResult::Count(count) => (json!({ "row_count": count.row_count }), "CountResult"),
    ExecutionResult::Duplicates(result) => (
      json!({"variables":result.variables,"total_rows":result.total_rows,"unique_groups":result.unique_groups,"duplicate_groups":result.duplicate_groups,"duplicate_rows":result.duplicate_rows,"extra_rows":result.extra_rows,"max_copies":result.max_copies}),
      "DuplicatesResult",
    ),
    ExecutionResult::Isid(result) => (
      json!({"variables":result.variables,"total_rows":result.total_rows,"unique_groups":result.unique_groups,"missing_key_rows":result.missing_key_rows,"missok":result.missok}),
      "IsidResult",
    ),
    ExecutionResult::Assert(result) => (
      json!({"checked":result.checked,"failed":result.failed}),
      "AssertResult",
    ),
    ExecutionResult::Datasignature(result) => (
      json!({"algorithm":result.algorithm,"signature":result.signature,"row_count":result.row_count,"column_count":result.column_count}),
      "DatasignatureResult",
    ),
    ExecutionResult::Collapse(result) => (
      json!({"dataset":dataset_info_data(&result.dataset)}),
      "CollapseResult",
    ),
    ExecutionResult::Codebook(result) => (codebook_result_data(result), "CodebookResult"),
    ExecutionResult::Summarize(summary) => (summarize_result_data(summary), "SummarizeResult"),
    ExecutionResult::SummarizeDetail(result) => (
      json!({
        "quantile_method": "linear_type_7",
        "rows": result.rows.iter().map(|row| json!({
          "summary": summary_row_data(&row.summary),
          "percentiles": row.percentiles.iter().map(|percentile| json!({
            "percentile": percentile.percentile,
            "value": percentile.value.and_then(Number::from_f64),
          })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
      }),
      "SummarizeDetailResult",
    ),
    ExecutionResult::Missing(result) => (missing_result_data(result), "MissingResult"),
    ExecutionResult::Tabulate(table) => (
      table_result_data(&table.headers, &table.rows),
      "TabulateResult",
    ),
    ExecutionResult::Table(table) => (
      table_result_data(&table.headers, &table.rows),
      "TableResult",
    ),
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      (preview_result_data(preview), "PreviewResult")
    }
    ExecutionResult::Logit(result) => (
      json!({
        "outcome": result.outcome, "predictors": result.predictor_names,
        "estimator": "logit", "cluster": result.cluster,
        "omnibus_wald": binary_wald_json(result.omnibus_wald.as_ref()),
        "covariance": if result.cluster.is_some() { "cluster_sandwich_g_over_g_minus_1" } else if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_fisher_information" },
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "include_intercept": result.include_intercept, "observation_count": result.observation_count,
        "event_count": result.event_count, "coefficients": result.coefficients,
        "covariance_matrix": result.covariance, "log_likelihood": result.log_likelihood,
        "null_log_likelihood": result.null_log_likelihood, "likelihood_ratio": result.likelihood_ratio,
        "likelihood_ratio_df": result.likelihood_ratio_df,
        "likelihood_ratio_p_value": result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution": if result.robust { None } else { Some("asymptotic_chi_square") },
        "mcfadden_r_squared": result.mcfadden_r_squared, "sample": result.sample,
        "converged": true, "iterations": result.iterations,
        "convergence_coordinate_system": "unit_column_norm",
        "convergence_absolute_tolerance": 1e-10,
        "max_abs_scaled_score": result.max_abs_scaled_score,
        "max_abs_scaled_newton_step": result.max_abs_scaled_newton_step,
      }),
      "BinaryLogitResult",
    ),
    ExecutionResult::LogitMarginalEffects(result) => (
      json!({
        "estimator": "logit", "effect": "average_continuous_predictor_column_derivative",
        "sample": "retained_estimation_design", "uncertainty": "conditional_on_observed_design_delta_method",
        "cluster": result.cluster,
        "covariance": if result.cluster.is_some() { "cluster_sandwich_g_over_g_minus_1" } else if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_fisher_information" }, "confidence_level": 95.0,
        "reference_distribution": "asymptotic_standard_normal",
        "observation_count": result.observation_count, "effects": result.effects,
      }),
      "LogitMarginalEffectsResult",
    ),
    ExecutionResult::NormalizedCommonNestedLogit(result) => {
      let mut data = serde_json::to_value(result).expect("finite normalized nested fit");
      let object = data.as_object_mut().expect("fit object");
      object.remove("fitted_probabilities");
      let predictors = object.remove("predictor_names").expect("predictors");
      object.insert("predictors".into(), predictors);
      let covariance = object.remove("covariance").expect("covariance");
      object.insert("covariance_matrix".into(), covariance);
      object.extend(json!({
        "estimator":"nlogit", "parameterization":"normalized_common_utility",
        "preferred_rum_normalization":true, "shared_nest_scale":true,
        "include_intercept":false, "parameter_count":result.predictor_names.len()+1,
        "missingness":"whole_choice_set_complete_finite_exclusion_after_structural_validation",
        "coefficient_test_null":0.0, "covariance":"model_based_joint_observed_normalized_common_nested_choice_hessian",
        "covariance_correction":1.0, "independent_unit":"choice_set",
        "reference_distribution":"asymptotic_standard_normal", "confidence_level":95.0,
        "confidence_interval_method":"normal_wald_natural_scale_may_leave_parameter_domain",
        "independence_null":"shared_positive_nest_scale_equal_one_slopes_reestimated",
        "independence_reference_distribution":"asymptotic_chi_square",
        "converged":true,"optimization_starts_attempted":2,
        "convergence_coordinate_system":"within_case_baseline_rms_beta_over_common_scale_and_log_scale_per_choice_set_score",
        "convergence_absolute_tolerance":1e-10
      }).as_object().expect("literal object").clone());
      (data, "NormalizedCommonNestedLogitResult")
    }
    ExecutionResult::NonNormalizedNestedLogit(result) => {
      // Split the literal objects rather than raising the crate's macro recursion limit.
      let mut data = json!({
        "estimator":"nlogit","parameterization":"non_normalized_conditional_utilities",
        "preferred_rum_normalization":false,"outcome":result.outcome,"predictors":result.predictor_names,
        "lower_predictor_count":result.lower_predictor_count,"include_intercept":false,
        "group_variable":result.group_variable,"alternative_variable":result.alternative_variable,"nest_variable":result.nest_variable,
        "alternative_levels":result.alternative_levels,"nest_levels":result.nest_levels,"alternative_nests":result.alternative_nests,"chosen_counts":result.chosen_counts,
        "observation_count":result.observation_count,"choice_set_count":result.choice_set_count,"choice_set_size":result.choice_set_size,
        "candidate_choice_set_count":result.candidate_choice_set_count,"excluded_choice_set_count":result.excluded_choice_set_count,"sample":result.sample,
        "parameter_count":result.predictor_names.len()+result.dissimilarities.len(),
        "missingness":"whole_choice_set_complete_finite_exclusion_after_structural_validation",
        "coefficients":result.coefficients,"coefficient_test_null":0.0,"dissimilarities":result.dissimilarities,
        "covariance_matrix":result.covariance,"covariance_parameter_names":result.covariance_parameter_names,
        "covariance":"model_based_joint_observed_nested_choice_hessian","covariance_correction":1.0,"independent_unit":"choice_set",
      });
      let inference = json!({
        "reference_distribution":"asymptotic_standard_normal","confidence_level":95.0,
        "confidence_interval_method":"normal_wald_natural_scale_may_leave_parameter_domain",
        "log_likelihood":result.log_likelihood,"restricted_log_likelihood":result.restricted_log_likelihood,
        "independence_null":"both_positive_nest_scales_equal_one_slopes_reestimated",
        "independence_likelihood_ratio":result.independence_likelihood_ratio,"independence_df":result.independence_df,
        "independence_p_value":result.independence_p_value,"independence_reference_distribution":"asymptotic_chi_square",
        "random_utility_iv_bounds_satisfied":result.random_utility_iv_bounds_satisfied,
        "converged":true,"iterations":result.iterations,"optimization_starts_attempted":2,"converged_starts":result.converged_starts,
        "convergence_coordinate_system":"raw_predictor_rms_and_log_scale_per_choice_set_score",
        "convergence_absolute_tolerance":1e-10,"max_abs_scaled_score":result.max_abs_scaled_score,"max_abs_scaled_newton_step":result.max_abs_scaled_newton_step,
      });
      data
        .as_object_mut()
        .expect("literal object")
        .extend(inference.as_object().expect("literal object").clone());
      (data, "NonNormalizedNestedLogitResult")
    }
    ExecutionResult::ConditionalLogit(result) => (
      json!({
        "estimator":"clogit","likelihood":"single_choice_conditional_multinomial",
        "outcome":result.outcome,"predictors":result.predictor_names,"include_intercept":false,
        "group_variable":result.group_variable,"alternative_variable":result.alternative_variable,
        "alternative_levels":result.alternative_levels,"chosen_counts":result.chosen_counts,
        "observation_count":result.observation_count,"choice_set_count":result.choice_set_count,
        "choice_set_size":result.choice_set_size,"candidate_choice_set_count":result.candidate_choice_set_count,
        "excluded_choice_set_count":result.excluded_choice_set_count,"sample":result.sample,
        "missingness":"whole_choice_set_complete_finite_exclusion_after_structural_validation",
        "coefficients":result.coefficients,"covariance_matrix":result.covariance,
        "covariance_parameter_names":result.predictor_names,"covariance":"model_based_joint_conditional_fisher_information",
        "independent_unit":"choice_set","covariance_correction":1.0,
        "reference_distribution":"asymptotic_standard_normal","confidence_level":95.0,
        "log_likelihood":result.log_likelihood,"null_log_likelihood":result.null_log_likelihood,
        "null_model":"equal_probabilities_within_complete_choice_sets_all_coefficients_zero",
        "likelihood_ratio":result.likelihood_ratio,"likelihood_ratio_df":result.likelihood_ratio_df,
        "likelihood_ratio_p_value":result.likelihood_ratio_p_value,"likelihood_ratio_reference_distribution":"asymptotic_chi_square",
        "mcfadden_r_squared":result.mcfadden_r_squared,"converged":true,"iterations":result.iterations,
        "convergence_coordinate_system":"within_first_alternative_rms_scaled_per_choice_set_score",
        "convergence_absolute_tolerance":1e-10,"max_abs_scaled_score":result.max_abs_scaled_score,
        "max_abs_scaled_newton_step":result.max_abs_scaled_newton_step,
      }),
      "ConditionalLogitResult",
    ),
    ExecutionResult::MultinomialLogit(result) => (
      json!({
        "estimator":"mlogit", "outcome":result.outcome, "predictors":result.predictor_names,
        "include_intercept":result.include_intercept, "category_levels":result.category_levels,
        "base_outcome":result.base_outcome, "category_counts":result.category_counts,
        "observation_count":result.observation_count, "coefficients":result.coefficients,
        "coefficient_order":"category_major_nonbase_intercept_then_predictors",
        "covariance_matrix":result.covariance, "covariance":"model_based_joint_fisher_information",
        "covariance_correction":1.0, "reference_distribution":"asymptotic_standard_normal", "confidence_level":95.0,
        "log_likelihood":result.log_likelihood,"null_log_likelihood":result.null_log_likelihood,
        "null_model":if result.include_intercept { "intercept_only_estimated_category_shares" } else { "equal_probabilities_all_coefficients_zero" },
        "likelihood_ratio":result.likelihood_ratio,"likelihood_ratio_df":result.likelihood_ratio_df,
        "likelihood_ratio_p_value":result.likelihood_ratio_p_value,"likelihood_ratio_reference_distribution":"asymptotic_chi_square",
        "mcfadden_r_squared":result.mcfadden_r_squared,"sample":result.sample,
        "converged":true,"iterations":result.iterations,"convergence_coordinate_system":"rms_column_scaled_per_observation_score",
        "convergence_absolute_tolerance":1e-10,"max_abs_scaled_score":result.max_abs_scaled_score,
        "max_abs_scaled_newton_step":result.max_abs_scaled_newton_step,
      }),
      "MultinomialLogitResult",
    ),
    ExecutionResult::MultinomialMarginalEffects(result) => (
      json!({
        "estimator":"mlogit","effect":"average_continuous_predictor_column_derivative",
        "sample":"retained_estimation_design","uncertainty":"conditional_on_observed_design_delta_method",
        "covariance":"model_based_joint_fisher_information","covariance_correction":1.0,
        "confidence_level":95.0,"reference_distribution":"asymptotic_standard_normal",
        "observation_count":result.observation_count,"category_levels":result.category_levels,
        "base_outcome":result.base_outcome,"effects":result.effects,
      }),
      "MultinomialMarginalEffectsResult",
    ),
    ExecutionResult::Probit(result) => (
      json!({
        "outcome": result.outcome, "predictors": result.predictor_names,
        "estimator": "probit", "cluster": result.cluster,
        "omnibus_wald": binary_wald_json(result.omnibus_wald.as_ref()),
        "covariance": if result.cluster.is_some() { "cluster_sandwich_g_over_g_minus_1" } else if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_observed_information" },
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "include_intercept": result.include_intercept, "observation_count": result.observation_count,
        "event_count": result.event_count, "coefficients": result.coefficients,
        "covariance_matrix": result.covariance, "log_likelihood": result.log_likelihood,
        "null_log_likelihood": result.null_log_likelihood, "likelihood_ratio": result.likelihood_ratio,
        "likelihood_ratio_df": result.likelihood_ratio_df,
        "likelihood_ratio_p_value": result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution": if result.robust { None } else { Some("asymptotic_chi_square") },
        "mcfadden_r_squared": result.mcfadden_r_squared, "sample": result.sample,
        "converged": true, "iterations": result.iterations,
        "convergence_coordinate_system": "unit_column_norm",
        "convergence_absolute_tolerance": 1e-10,
        "max_abs_scaled_score": result.max_abs_scaled_score,
        "max_abs_scaled_newton_step": result.max_abs_scaled_newton_step,
      }),
      "BinaryProbitResult",
    ),
    ExecutionResult::ProbitMarginalEffects(result) => (
      json!({
        "estimator": "probit", "effect": "average_continuous_predictor_column_derivative",
        "sample": "retained_estimation_design", "uncertainty": "conditional_on_observed_design_delta_method",
        "cluster": result.cluster,
        "covariance": if result.cluster.is_some() { "cluster_sandwich_g_over_g_minus_1" } else if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_observed_information" }, "confidence_level": 95.0,
        "reference_distribution": "asymptotic_standard_normal",
        "observation_count": result.observation_count, "effects": result.effects,
      }),
      "ProbitMarginalEffectsResult",
    ),
    ExecutionResult::HeckmanTwoStep(result) => (
      json!({
        "estimator":"heckman", "method":"twostep", "distribution":"iid_gaussian_sample_selection",
        "outcome":result.outcome, "predictors":result.predictor_names,
        "include_intercept":result.include_intercept,
        "observation_count":result.selection.observation_count,
        "selected_count":result.selected_count, "unselected_count":result.unselected_count,
        "selection_outcome":result.selection.outcome, "selection_predictors":result.selection.predictor_names,
        "selection_coefficients":result.selection.coefficients,
        "selection_covariance_matrix":result.selection.covariance,
        "selection_covariance_parameter_names":result.selection.coefficients.iter().map(|c|c.name.as_str()).collect::<Vec<_>>(),
        "selection_covariance":"model_based_observed_probit_information",
        "selection_sample":result.selection.sample, "selected_sample":result.sample,
        "selection_log_likelihood":result.selection.log_likelihood,
        "selection_iterations":result.selection.iterations,
        "selection_max_abs_scaled_score":result.selection.max_abs_scaled_score,
        "selection_max_abs_scaled_newton_step":result.selection.max_abs_scaled_newton_step,
        "max_abs_scaled_outcome_normal_equation":result.max_abs_scaled_normal_equation,
        "coefficients":result.coefficients, "covariance_matrix":result.covariance,
        "covariance_parameter_names":result.coefficients.iter().map(|c|c.name.as_str()).collect::<Vec<_>>(),
        "covariance":"model_based_heckman_two_step_generated_regressor_correction",
        "covariance_blocks":"separate_selection_and_augmented_outcome_no_cross_equation_inference",
        "covariance_correction":1.0,"reference_distribution":"asymptotic_standard_normal","confidence_level":95.0,
        "sigma":result.sigma,"rho":result.rho,"scale_correlation_inference":"points_only",
        "scale_moment":"selected_rss_over_selected_n_plus_alpha_squared_mean_delta",
        "outcome_sse":result.outcome_sse,
        "excluded_selection_predictors":result.excluded_selection_predictors,
        "identification_warning":if result.excluded_selection_predictors.is_empty() {Some("No excluded selection predictor; identification relies on the Gaussian functional form and may be weak.")} else {None},
        "exclusion_validity":"user_assumption_not_tested", "joint_log_likelihood":null,
        "converged":true,
      }),
      "HeckmanTwoStepResult",
    ),
    ExecutionResult::TruncatedGaussian(result) => (
      json!({
        "estimator":"truncreg", "distribution":"iid_homoskedastic_gaussian_conditioned_above_known_lower_limit",
        "outcome":result.outcome, "predictors":result.predictor_names,
        "include_intercept":result.include_intercept,"lower_limit":result.lower_limit,
        "support":"joint_finite_sample_then_strictly_above_limit_or_error_no_censoring",
        "observation_count":result.observation_count,"sample":result.sample,
        "coefficients":result.coefficients,"scale":result.scale,
        "scale_interval":"exponentiated_log_scale_normal_wald_95_not_sigma_zero_test",
        "covariance_matrix":result.covariance,
        "covariance_parameter_names":result.coefficients.iter().map(|c|c.name.as_str()).chain(std::iter::once("lnsigma")).collect::<Vec<_>>(),
        "covariance_parameter_order":"location_coefficients_then_lnsigma",
        "covariance":"model_based_joint_observed_beta_lnsigma_information","covariance_correction":1.0,
        "reference_distribution":"asymptotic_standard_normal","confidence_level":95.0,
        "log_likelihood":result.log_likelihood,"log_likelihood_convention":"gaussian_density_divided_by_population_retention_probability",
        "null_log_likelihood":result.null_log_likelihood,
        "null_model":if result.include_intercept {"intercept_and_scale_refitted_same_sample_bound"}else{"zero_latent_location_scale_refitted_same_sample_bound"},
        "likelihood_ratio":result.likelihood_ratio,"likelihood_ratio_df":result.likelihood_ratio_df,
        "likelihood_ratio_p_value":result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution":result.likelihood_ratio_p_value.map(|_|"asymptotic_chi_square"),
        "mean_conditional_derivative":result.mean_conditional_derivative,
        "mean_conditional_derivative_gradient":result.mean_conditional_derivative_gradient,
        "converged":true,"iterations":result.iterations,
        "convergence_coordinate_system":"rms_predictors_effective_response_canonical_eta_and_precision",
        "convergence_score_normalization":"per_observation_mean","convergence_step_normalization":"1_plus_abs_parameter","convergence_tolerance":1e-10,
        "max_abs_scaled_score":result.max_abs_scaled_score,"max_abs_scaled_newton_step":result.max_abs_scaled_newton_step,
      }),
      "TruncatedGaussianResult",
    ),
    ExecutionResult::TruncatedGaussianMarginalEffects(result) => (
      json!({"estimator":"truncreg","effect":"average_continuous_conditional_truncated_mean_derivative",
        "sample":"retained_estimation_design","lower_limit":result.lower_limit,
        "uncertainty":"conditional_on_observed_design_full_joint_beta_lnsigma_delta_method",
        "covariance":"model_based_joint_observed_beta_lnsigma_information","covariance_correction":1.0,
        "reference_distribution":"asymptotic_standard_normal","confidence_level":95.0,
        "observation_count":result.observation_count,"effects":result.effects}),
      "TruncatedGaussianMarginalEffectsResult",
    ),
    ExecutionResult::Cox(result) => {
      let mut data = serde_json::to_value(result).expect("finite Cox result");
      let object = data.as_object_mut().expect("Cox result object");
      for key in [
        "fitted_xb",
        "fitted_cumulative_hazard",
        "fitted_survival",
        "fitted_martingale",
        "predictor_names",
      ] {
        object.remove(key);
      }
      data["estimator"] = json!("stcox_breslow");
      data["predictors"] = json!(result.predictor_names);
      data["ties"] = json!("breslow");
      data["include_intercept"] = json!(false);
      data["support"] = json!(
        "entry_zero_single_spell_joint_finite_then_positive_time_exact_binary_event_or_error"
      );
      data["covariance"] = json!(if result.robust {
        "full_efficient_martingale_score_sandwich"
      } else {
        "inverse_partial_likelihood_observed_information"
      });
      data["covariance_parameter_names"] = json!(result.predictor_names);
      data["reference_distribution"] = json!("asymptotic_standard_normal");
      data["confidence_level"] = json!(95.0);
      data["log_likelihood_convention"] =
        json!("breslow_tied_partial_likelihood_not_duration_density");
      data["null_model"] = json!("all_beta_zero_same_sample");
      data["likelihood_ratio_reference_distribution"] = json!(
        result
          .likelihood_ratio_p_value
          .map(|_| "asymptotic_chi_square")
      );
      data["omnibus_wald_reference_distribution"] =
        json!(result.omnibus_wald.p_value.map(|_| "asymptotic_chi_square"));
      data["baseline_reference"] =
        json!("all_retained_predictor_means_in_predictor_order_including_dummy_means");
      data["baseline_uncertainty"] = json!("unavailable");
      data["prediction_time_convention"] =
        json!("right_continuous_step_including_ties_zero_before_first_hold_after_last");
      data["survival_interpretation"] =
        json!("net_cause_specific_not_all_cause_survival_or_competing_risk_cumulative_incidence");
      data["converged"] = json!(true);
      data["convergence_coordinate_system"] = json!("centered_rms_predictors");
      data["convergence_score_normalization"] = json!("per_observation_mean");
      data["convergence_step_normalization"] = json!("1_plus_abs_parameter");
      data["convergence_tolerance"] = json!(1e-10);
      (data, "CoxResult")
    }
    ExecutionResult::Weibull(result) => {
      let mut data = serde_json::to_value(result).expect("finite Weibull result");
      let object = data.as_object_mut().expect("Weibull result object");
      for key in [
        "fitted_xb",
        "fitted_hazard",
        "fitted_cumulative_hazard",
        "fitted_survival",
        "predictor_names",
      ] {
        object.remove(key);
      }
      data["estimator"] = json!("streg_weibull");
      data["distribution"] = json!("weibull_proportional_hazards_entry_zero_single_spell");
      data["predictors"] = json!(result.predictor_names);
      data["support"] = json!("joint_finite_sample_then_positive_time_exact_binary_event_or_error");
      data["covariance"] = json!(if result.robust {
        "robust_full_joint_observed_information_score_sandwich"
      } else {
        "model_based_full_joint_observed_information"
      });
      data["covariance_parameter_names"] = json!(
        result
          .coefficients
          .iter()
          .map(|c| c.name.as_str())
          .chain(std::iter::once("lnshape"))
          .collect::<Vec<_>>()
      );
      data["covariance_parameter_order"] = json!("ph_coefficients_then_lnshape");
      data["reference_distribution"] = json!("asymptotic_standard_normal");
      data["confidence_level"] = json!(95.0);
      data["shape_interval"] =
        json!("exponentiated_logshape_normal_wald_95_shape_one_test_not_shape_zero");
      data["log_likelihood_convention"] =
        json!("event_log_weibull_density_and_nonevent_log_survival");
      data["normalized_log_likelihood_convention"] =
        json!("duration_density_plus_fixed_sum_event_log_time");
      data["null_model"] = json!(if result.include_intercept {
        "intercept_and_shape_refitted_same_sample"
      } else {
        "zero_ph_index_shape_refitted_same_sample"
      });
      data["likelihood_ratio_reference_distribution"] = json!(
        result
          .likelihood_ratio_p_value
          .map(|_| "asymptotic_chi_square")
      );
      data["omnibus_wald_reference_distribution"] =
        json!(result.omnibus_wald.p_value.map(|_| "asymptotic_chi_square"));
      data["survival_interpretation"] =
        json!("net_cause_specific_not_all_cause_survival_or_competing_risk_cumulative_incidence");
      data["converged"] = json!(true);
      data["convergence_coordinate_system"] =
        json!("rms_predictors_positive_rescaled_natural_shape");
      data["convergence_score_normalization"] = json!("per_observation_mean");
      data["convergence_step_normalization"] = json!("1_plus_abs_parameter");
      data["convergence_tolerance"] = json!(1e-10);
      (data, "WeibullResult")
    }
    ExecutionResult::Tobit(result) => (
      json!({
        "estimator":"tobit", "distribution":"iid_homoskedastic_gaussian",
        "outcome":result.outcome, "predictors":result.predictor_names,
        "include_intercept":result.include_intercept, "lower_limit":result.lower_limit,
        "censoring":"recorded_response_at_or_below_lower_limit",
        "observation_count":result.observation_count, "left_censored_count":result.left_censored_count,
        "uncensored_count":result.uncensored_count, "sample":result.sample,
        "mean_uncensored_probability":result.mean_uncensored_probability,
        "mean_probability_gradient":result.mean_probability_gradient,
        "coefficients":result.coefficients, "scale":result.scale,
        "scale_interval":"exponentiated_log_scale_normal_wald_95_not_sigma_zero_test",
        "covariance_matrix":result.covariance,
        "covariance_parameter_names":result.coefficients.iter().map(|c|c.name.as_str()).chain(std::iter::once("lnsigma")).collect::<Vec<_>>(),
        "covariance_parameter_order":"location_coefficients_then_lnsigma",
        "covariance":"model_based_joint_observed_beta_lnsigma_information", "covariance_correction":1.0,
        "reference_distribution":"asymptotic_standard_normal", "confidence_level":95.0,
        "log_likelihood":result.log_likelihood,
        "log_likelihood_convention":"gaussian_uncensored_density_and_left_censor_probability",
        "null_log_likelihood":result.null_log_likelihood,
        "null_model":if result.include_intercept {"intercept_and_scale_refitted_same_sample_bound"}else{"zero_latent_location_scale_refitted_same_sample_bound"},
        "likelihood_ratio":result.likelihood_ratio, "likelihood_ratio_df":result.likelihood_ratio_df,
        "likelihood_ratio_p_value":result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution":result.likelihood_ratio_p_value.map(|_|"asymptotic_chi_square"),
        "converged":true, "iterations":result.iterations,
        "convergence_coordinate_system":"rms_predictors_delta_and_scaled_inverse_sigma",
        "convergence_score_normalization":"per_observation_mean",
        "convergence_step_normalization":"1_plus_abs_parameter", "convergence_tolerance":1e-10,
        "max_abs_scaled_score":result.max_abs_scaled_score,
        "max_abs_scaled_newton_step":result.max_abs_scaled_newton_step,
      }),
      "TobitResult",
    ),
    ExecutionResult::TobitMarginalEffects(result) => (
      json!({
        "estimator":"tobit", "effect":"average_continuous_observed_censored_mean_derivative",
        "sample":"retained_estimation_design", "lower_limit":result.lower_limit,
        "uncertainty":"conditional_on_observed_design_full_joint_beta_lnsigma_delta_method",
        "covariance":"model_based_joint_observed_beta_lnsigma_information", "covariance_correction":1.0,
        "reference_distribution":"asymptotic_standard_normal", "confidence_level":95.0,
        "observation_count":result.observation_count, "effects":result.effects,
      }),
      "TobitMarginalEffectsResult",
    ),
    ExecutionResult::Poisson(result) => {
      let mut data = json!({
        "estimator": "poisson", "outcome": result.outcome, "predictors": result.predictor_names,
        "include_intercept": result.include_intercept, "observation_count": result.observation_count,
        "zero_count": result.zero_count, "count_sum": result.count_sum,
        "coefficients": result.coefficients, "covariance_matrix": result.covariance,
        "covariance_parameter_names": result.coefficients.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        "covariance": if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_poisson_information" },
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "log_likelihood": result.log_likelihood, "log_likelihood_convention": "poisson_count_pmf",
        "null_log_likelihood": result.null_log_likelihood,
        "null_model": if result.include_intercept { "intercept_only_estimated_mean" } else { "unit_mean_all_coefficients_zero" },
        "likelihood_ratio": result.likelihood_ratio, "likelihood_ratio_df": result.likelihood_ratio_df,
        "likelihood_ratio_p_value": result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution": result.likelihood_ratio_p_value.map(|_| "asymptotic_chi_square"),
        "omnibus_wald": binary_wald_json(result.omnibus_wald.as_ref()),
        "mcfadden_r_squared": result.mcfadden_r_squared, "residual_deviance": result.residual_deviance,
        "pearson_chi_square": result.pearson_chi_square, "df_residual": result.df_residual,
        "pearson_dispersion": result.pearson_dispersion,
        "dispersion_inference": "descriptive_not_a_panel_or_specification_test",
        "sample": result.sample, "converged": true, "iterations": result.iterations,
        "convergence_coordinate_system": "unit_column_norm", "convergence_absolute_tolerance": 1e-10,
        "max_abs_scaled_score": result.max_abs_scaled_score,
        "max_abs_scaled_newton_step": result.max_abs_scaled_newton_step,
      });
      add_count_cluster_metadata(
        &mut data,
        result.cluster.as_ref(),
        "cluster_sandwich_g_over_g_minus_1",
      );
      if let Some(variable) = &result.offset_variable {
        data["offset_variable"] = json!(variable);
        data["offset_coefficient"] = json!(1.0);
        data["linear_index"] = json!("x_beta_plus_offset");
        data["null_model"] = json!(if result.include_intercept {
          "intercept_only_with_same_fixed_offset"
        } else {
          "all_coefficients_zero_with_same_fixed_offset"
        });
      }
      (data, "PoissonResult")
    }
    ExecutionResult::PoissonMarginalEffects(result) => {
      let mut data = json!({
        "estimator": "poisson", "effect": "average_continuous_predictor_column_derivative",
        "sample": "retained_estimation_design", "uncertainty": "conditional_on_observed_design_delta_method",
        "covariance": if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_poisson_information" },
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "observation_count": result.observation_count, "effects": result.effects,
      });
      add_count_cluster_metadata(
        &mut data,
        result.cluster.as_ref(),
        "cluster_sandwich_g_over_g_minus_1",
      );
      if let Some(variable) = &result.offset_variable {
        data["offset_variable"] = json!(variable);
        data["offset_treatment"] =
          json!("fixed_at_retained_values_in_derivatives_and_delta_uncertainty");
      }
      (data, "PoissonMarginalEffectsResult")
    }
    ExecutionResult::CountProbabilities(result) => {
      let mut data = serde_json::to_value(&result.table).expect("finite count probability table");
      data["estimator"] = json!(match result.distribution {
        tabdat_domain::CountProbabilityLaw::Poisson => "poisson",
        tabdat_domain::CountProbabilityLaw::Nb2 { .. } => "nb2",
      });
      data["outcome"] = json!(result.outcome);
      data["distribution"] = json!(result.distribution);
      data["sample"] = json!(result.sample);
      data["sample_role"] = json!("retained_estimation_sample");
      data["probability_role"] = json!("mean_of_individual_plugin_conditional_probabilities");
      data["covariance_role"] = json!("does_not_establish_distribution");
      data["uncertainty"] = json!("not_computed");
      data["goodness_of_fit_test"] = json!("not_performed");
      (data, "CountProbabilityResult")
    }
    ExecutionResult::Nbreg(result) => {
      let mut data = json!({
        "estimator": "nb2", "outcome": result.outcome, "predictors": result.predictor_names,
        "include_intercept": result.include_intercept, "observation_count": result.observation_count,
        "zero_count": result.zero_count, "count_sum": result.count_sum,
        "mean_parameter_count": result.coefficients.len(), "parameter_count": result.coefficients.len()+1,
        "coefficients": result.coefficients, "covariance_matrix": result.covariance,
        "covariance_parameter_names": result.coefficients.iter().map(|c| c.name.as_str()).chain(std::iter::once("lnalpha")).collect::<Vec<_>>(),
        "covariance": if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_joint_observed_information" },
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "variance_function": "mu_plus_alpha_mu_squared", "dispersion_parameterization": "alpha_exp_lnalpha_size_inverse_alpha",
        "dispersion": result.dispersion,
        "dispersion_boundary_test": "not_implemented_no_normal_alpha_zero_test",
        "dispersion_test": result.dispersion_test,
        "sample": result.sample,
      });
      let diagnostics = json!({
        "log_likelihood": result.log_likelihood, "log_likelihood_convention": "nb2_count_pmf",
        "null_log_likelihood": result.null_log_likelihood, "null_alpha": result.null_alpha,
        "null_model": if result.include_intercept { "intercept_only_mean_and_dispersion_reestimated" } else { "unit_mean_dispersion_reestimated" },
        "likelihood_ratio": result.likelihood_ratio, "likelihood_ratio_df": result.likelihood_ratio_df,
        "likelihood_ratio_p_value": result.likelihood_ratio_p_value,
        "likelihood_ratio_reference_distribution": result.likelihood_ratio_p_value.map(|_| "asymptotic_chi_square"),
        "omnibus_wald": binary_wald_json(result.omnibus_wald.as_ref()),
        "mcfadden_r_squared": result.mcfadden_r_squared, "residual_deviance": result.residual_deviance,
        "pearson_chi_square": result.pearson_chi_square, "df_residual": result.df_residual,
        "pearson_dispersion": result.pearson_dispersion,
        "dispersion_inference": "descriptive_not_a_panel_or_specification_test",
        "converged": true, "iterations": result.iterations,
        "iterations_role": "full_model_outer_iterations",
        "convergence_coordinate_system": "unit_mean_design_column_norm_and_log_alpha_sqrt_n",
        "convergence_absolute_tolerance": 1e-10,
        "max_abs_scaled_score": result.max_abs_scaled_score,
        "max_abs_scaled_newton_step": result.max_abs_scaled_newton_step,
        "supported_maximum_count": 100000, "supported_log_alpha_open_bounds": [-12.0,12.0],
      });
      data
        .as_object_mut()
        .unwrap()
        .extend(diagnostics.as_object().unwrap().clone());
      add_count_cluster_metadata(
        &mut data,
        result.cluster.as_ref(),
        "cluster_joint_score_sandwich_g_over_g_minus_1",
      );
      if let Some(variable) = &result.offset_variable {
        data["offset_variable"] = json!(variable);
        data["offset_coefficient"] = json!(1.0);
        data["linear_index"] = json!("x_beta_plus_offset");
        data["null_model"] = json!(if result.include_intercept {
          "intercept_and_dispersion_reestimated_with_same_fixed_offset"
        } else {
          "all_beta_zero_dispersion_reestimated_with_same_fixed_offset"
        });
      }
      (data, "NbregResult")
    }
    ExecutionResult::NbregMarginalEffects(result) => {
      let mut data = json!({
        "estimator": "nb2", "effect": "average_continuous_predictor_column_derivative",
        "sample": "retained_estimation_design", "uncertainty": "conditional_on_observed_design_delta_method",
        "covariance": if result.robust { "robust_sandwich_n_over_n_minus_1" } else { "model_based_joint_observed_information" },
        "covariance_role": "beta_block_of_full_joint_beta_lnalpha_covariance",
        "reference_distribution": "asymptotic_standard_normal", "confidence_level": 95.0,
        "observation_count": result.observation_count, "effects": result.effects,
      });
      add_count_cluster_metadata(
        &mut data,
        result.cluster.as_ref(),
        "cluster_joint_score_sandwich_g_over_g_minus_1",
      );
      if let Some(variable) = &result.offset_variable {
        data["offset_variable"] = json!(variable);
        data["offset_treatment"] =
          json!("fixed_at_retained_values_in_derivatives_and_delta_uncertainty");
      }
      (data, "NbregMarginalEffectsResult")
    }
    ExecutionResult::Regression(result) => (regression_result_data(result), "RegressionResult"),
    ExecutionResult::RandomEffects(result) => (
      random_effects_result_data(result),
      if result.result.group_size == 0 && result.result.ml.is_some() {
        "UnbalancedRandomEffectsMlResult"
      } else if result.result.group_size == 0 {
        "UnbalancedRandomEffectsGlsResult"
      } else {
        "RandomEffectsResult"
      },
    ),
    ExecutionResult::FixedEffects(result) => {
      (fixed_effects_result_data(result), "FixedEffectsResult")
    }
    ExecutionResult::Ttest(result) => (ttest_result_data(result), "TtestResult"),
    ExecutionResult::Prediction(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "PredictionResult",
    ),
    ExecutionResult::Lincom(result) => (
      json!({
        "label": result.label, "estimate": Number::from_f64(result.estimate),
        "standard_error": Number::from_f64(result.standard_error), "statistic": Number::from_f64(result.statistic),
        "p_value": Number::from_f64(result.p_value), "ci_lower": Number::from_f64(result.ci_lower),
        "ci_upper": Number::from_f64(result.ci_upper), "ci_level": Number::from_f64(result.ci_level), "df_residual": result.df_residual,
      }),
      "LincomResult",
    ),
    ExecutionResult::Test(result) => (
      json!({
        "constraints": result.constraints, "statistic": Number::from_f64(result.statistic),
        "p_value": Number::from_f64(result.p_value), "df": result.df,
        "df_residual": result.df_residual, "is_chi2": result.is_chi2,
      }),
      "TestResult",
    ),
    _ => return None,
  };

  let envelope = ResultEnvelope::new(data, result_type);
  Some(serde_json::to_string(&envelope).expect("JSON value envelopes serialize infallibly"))
}

fn persistence_result_data(path: &std::path::Path, dataset: &DatasetInfo) -> Value {
  json!({"path": path.to_string_lossy(), "dataset": dataset_info_data(dataset)})
}

fn table_result_data(headers: &[String], rows: &[Vec<CellValue>]) -> Value {
  json!({
    "headers": headers,
    "rows": rows.iter().map(|row| row.iter().map(cell_value_data).collect::<Vec<_>>()).collect::<Vec<_>>(),
  })
}

fn codebook_result_data(result: &CodebookResult) -> Value {
  json!({
    "missing_definition": "sql_null", "example_limit": 3, "examples_nonfinite_encoding": "null",
    "rows": result.rows.iter().map(|row| json!({
      "variable": row.variable, "data_type": row.data_type,
      "nonmissing": row.nonmissing, "missing": row.missing, "distinct": row.distinct,
      "examples": row.examples.iter().map(cell_value_data).collect::<Vec<_>>(),
    })).collect::<Vec<_>>()
  })
}

fn missing_result_data(result: &MissingResult) -> Value {
  json!({
    "missing_definition": "sql_null",
    "rows": result.rows.iter().map(|row| json!({
      "variable": row.variable, "data_type": row.data_type, "total": row.total,
      "missing": row.missing, "nonmissing": row.nonmissing,
      "missing_percent": Number::from_f64(row.missing_percent),
    })).collect::<Vec<_>>()
  })
}

fn ttest_group_data(group: &TtestGroupStats, count: Option<usize>) -> Value {
  json!({
    "name": group.name, "count": count,
    "mean": Number::from_f64(group.mean), "standard_error": Number::from_f64(group.standard_error),
    "standard_deviation": Number::from_f64(group.standard_deviation),
    "ci_lower": Number::from_f64(group.ci_lower), "ci_upper": Number::from_f64(group.ci_upper),
  })
}

fn ttest_result_data(result: &TtestResult) -> Value {
  // Independent differences store legacy residual-df/zero, not an observation
  // count. Never expose that field as a sample size in the CLI contract.
  let difference_count = result
    .by_variable
    .is_none()
    .then_some(result.difference_stats.observations);
  json!({
    "varname1": result.varname1, "varname2": result.varname2, "by_variable": result.by_variable,
    "is_paired": result.is_paired, "is_welch": result.is_welch,
    "null_value": Number::from_f64(result.null_value), "reference_distribution": "student_t", "ci_level": 95.0,
    "group1_stats": ttest_group_data(&result.group1_stats, Some(result.group1_stats.observations)),
    "group2_stats": result.group2_stats.as_ref().map(|group| ttest_group_data(group, Some(group.observations))),
    "combined_stats": result.combined_stats.as_ref().map(|group| ttest_group_data(group, Some(group.observations))),
    "difference_stats": ttest_group_data(&result.difference_stats, difference_count),
    "t_statistic": Number::from_f64(result.t_statistic), "degrees_of_freedom": Number::from_f64(result.degrees_of_freedom),
    "p_left": Number::from_f64(result.p_left), "p_two": Number::from_f64(result.p_two), "p_right": Number::from_f64(result.p_right),
  })
}

fn random_effects_result_data(state: &tabdat_domain::RandomEffectsState) -> Value {
  let r = &state.result;
  let fit = &r.gls;
  let clustered = matches!(
    fit.covariance.covariance_type,
    tabdat_domain::CovarianceType::Cluster(_)
  );
  let covariance = if r.ml.is_some() && !clustered {
    "observed_information".into()
  } else if clustered {
    format!("cluster({})", r.entity_variable)
  } else {
    "nonrobust".into()
  };
  let mut data = json!({
    "estimator":fit.diagnostics.method, "outcome":r.outcome,"predictors":r.predictor_names,
    "entity_variable":r.entity_variable,"time_variable":state.time_variable,"entity_data_type":state.entity_data_type,
    "observation_count":fit.fit_statistics.observation_count,"group_count":r.group_count,"group_size":r.group_size,
    "df_residual":fit.fit_statistics.degrees_of_freedom,"reference_distribution":"normal","covariance":covariance,
    "covariance_convention": if clustered { "unit_grouped_quasi_demeaned_scores" } else { "quasi_demeaned_rss_over_n_minus_k" },
    "covariance_matrix":fit.covariance.matrix,"covariance_parameter_names":fit.parameter_names,
    "sigma_e_squared":r.sigma_e_squared,"sigma_u_squared":r.sigma_u_squared,"raw_sigma_u_squared":r.raw_sigma_u_squared,
    "variance_component_method":if r.ml.is_some() { "balanced_gaussian_marginal_ml" } else if r.group_size == 0 { "unbalanced_swamy_arora" } else { "balanced_swamy_arora" },
    "theta":r.theta,"rho":r.rho,
    "transformed_root_mse":fit.fit_statistics.root_mse,"transformed_rss":fit.fit_statistics.residual_sum_of_squares,
    "wald_statistic":r.wald_statistic,"wald_p_value":r.wald_p_value,"wald_df":r.predictor_names.len(),"wald_reference":"chi_square",
    "sample":fit.sample.as_ref().map(|s|json!({"total_observations":s.total_observations,"dropped_observations":s.dropped_observations})),
    "coefficients":fit.coefficients.iter().zip(&r.confidence_intervals).map(|(c,ci)|json!({"name":c.name,"value":c.value,"standard_error":c.standard_error,
      "statistic":c.statistic,"p_value":c.p_value,"ci_lower":ci[0],
      "ci_upper":ci[1],"ci_level":95})).collect::<Vec<_>>()
  });
  if let Some(ml) = &r.ml {
    data["covariance_convention"] = json!(if ml.cluster.is_some() {
      "unit_grouped_joint_ml_scores_observed_bread"
    } else if ml.unit_variance_boundary() {
      "constrained_observed_information"
    } else {
      "joint_observed_information"
    });
    data["ml"] = json!(ml);
    data["ml"]["unit_variance_boundary"] = json!(ml.unit_variance_boundary());
    data["log_likelihood"] = json!(ml.log_likelihood);
    match r.variance_component_inference() {
      Ok(Some(inference)) => {
        data["ml"]["variance_component_inference"] = json!(inference);
      }
      Ok(None) => {
        data["ml"]["variance_component_inference_unavailable_reason"] =
          json!("variance-component inference requires an ML result");
      }
      Err(error) => {
        data["ml"]["variance_component_inference_unavailable_reason"] = json!(error.to_string());
      }
    }
    if r.group_size == 0 {
      let (minimum, maximum) = r.group_size_range().unwrap_or((0, 0));
      let (theta_min, theta_max) = r.theta_range().unwrap_or((f64::NAN, f64::NAN));
      data["balanced"] = json!(false);
      data["group_size"] = Value::Null;
      data["minimum_group_size"] = json!(minimum);
      data["maximum_group_size"] = json!(maximum);
      data["theta"] = Value::Null;
      data["theta_at_minimum_group_size"] = json!(r.theta);
      data["theta_min"] = json!(theta_min);
      data["theta_max"] = json!(theta_max);
      data["eta_reference_group_size"] = json!(minimum);
      data["variance_component_method"] = json!("unbalanced_gaussian_marginal_ml");
    }
  } else if r.group_size == 0 {
    let (minimum, maximum) = r.group_size_range().unwrap_or((0, 0));
    let (theta_min, theta_max) = r.theta_range().unwrap_or((f64::NAN, f64::NAN));
    data["balanced"] = json!(false);
    data["group_size"] = Value::Null;
    data["minimum_group_size"] = json!(minimum);
    data["maximum_group_size"] = json!(maximum);
    data["theta"] = Value::Null;
    data["theta_at_minimum_group_size"] = json!(r.theta);
    data["theta_min"] = json!(theta_min);
    data["theta_max"] = json!(theta_max);
  }
  if clustered {
    let n = fit.fit_statistics.observation_count;
    let g = r.group_count;
    let k = fit.parameter_names.len();
    data["cluster"] = if let Some(cluster) = r.ml.as_ref().and_then(|ml| ml.cluster.as_ref()) {
      json!({"variable":r.entity_variable,"group_count":g,
        "finite_sample_multiplier":cluster.finite_sample_multiplier,
        "correction_convention":"gaussian_marginal_ml_full_joint_scores_g_over_g_minus_1",
        "independence_assumption":"across_units"})
    } else {
      json!({"variable":r.entity_variable,"group_count":g,
        "finite_sample_multiplier":g as f64/(g-1) as f64*(n-1) as f64/(n-k) as f64,
        "correction_convention":"plm_arellano_sss_transformed_gls","independence_assumption":"across_units"})
    };
  }
  if let Some(reason) = &r.wald_unavailable_reason {
    data["wald_unavailable_reason"] = json!(reason);
  }
  data
}

fn fixed_effects_result_data(state: &tabdat_domain::FixedEffectsState) -> Value {
  let result = &state.result;
  let fit = &result.within;
  let stats = &fit.fit_statistics;
  let n = stats.observation_count;
  let g = result.effects.len();
  let p = fit.parameter_names.len();
  let clustered = matches!(
    fit.covariance.covariance_type,
    tabdat_domain::CovarianceType::Cluster(_)
  );
  let inference_df = if clustered {
    g.saturating_sub(1)
  } else {
    stats.degrees_of_freedom
  };
  let covariance = if clustered {
    format!("cluster({})", result.entity_variable)
  } else {
    "nonrobust".into()
  };
  let mut data = json!({
    "estimator":"within_fixed_effects","outcome":result.outcome,"predictors":fit.parameter_names,
    "entity_variable":result.entity_variable,"time_variable":state.time_variable,"entity_data_type":state.entity_data_type,
    "observation_count":n,"group_count":g,"slope_count":p,"absorbed_intercepts":g,
    "df_residual":stats.degrees_of_freedom,"inference_df":inference_df,"reference_distribution":"student_t",
    "covariance":covariance,"covariance_matrix":fit.covariance.matrix,"covariance_parameter_names":fit.parameter_names,
    "within_r_squared":stats.r_squared.and_then(Number::from_f64),"conditional_r_squared":result.conditional_r_squared.and_then(Number::from_f64),
    "root_mse":stats.root_mse.and_then(Number::from_f64),"residual_sum_of_squares":stats.residual_sum_of_squares.and_then(Number::from_f64),
    "within_total_sum_of_squares":stats.total_sum_of_squares.and_then(Number::from_f64),
    "minimum_group_size":result.minimum_group_size,"maximum_group_size":result.maximum_group_size,"singleton_groups":result.singleton_groups,
    "common_intercept":Number::from_f64(result.common_intercept),"common_intercept_inference":"normalization_point_only",
    "unit_effect_normalization":"estimation_observation_weighted_zero_mean",
    "sample":fit.sample.as_ref().map(|sample|json!({"total_observations":sample.total_observations,"dropped_observations":sample.dropped_observations})),
    "coefficients":fit.coefficients.iter().map(|c|json!({"name":c.name,"value":Number::from_f64(c.value),
      "standard_error":c.standard_error.and_then(Number::from_f64),"statistic":c.statistic.and_then(Number::from_f64),"p_value":c.p_value.and_then(Number::from_f64)})).collect::<Vec<_>>()
  });
  if clustered {
    data["cluster"] = json!({"variable":result.entity_variable,"group_count":g,
      "finite_sample_multiplier":Number::from_f64(g as f64/g.saturating_sub(1) as f64*(n-1) as f64/(n-p) as f64),
      "correction_convention":"plm_arellano_sss_nested_unit_effects","independence_assumption":"across_units"});
  }
  if let Some(test) = &result.unit_effects_f_test {
    data["unit_effects_f_test"] = json!({
      "null":"all unit intercepts are equal",
      "f_statistic":Number::from_f64(test.f_statistic),
      "df_num":test.df_num,
      "df_denom":test.df_denom,
      "p_value":Number::from_f64(test.p_value),
      "reference":"classical_homoskedastic_pooled_vs_within",
      "exact_finite_sample_requires_normal_errors":true
    });
  }
  if let Some(reason) = &result.unit_effects_f_test_unavailable_reason {
    data["unit_effects_f_test_unavailable_reason"] = json!(reason);
  }
  data
}

fn regression_result_data(result: &RegressionResult) -> Value {
  let mut data = json!({
    "outcome": result.outcome, "predictors": result.predictors,
    "estimator": result.estimator, "covariance": result.covariance,
    "observation_count": result.observation_count, "include_intercept": result.include_intercept,
    "df_residual": result.least_squares.fit_statistics.degrees_of_freedom,
    "r_squared": result.r_squared.and_then(Number::from_f64),
    "adjusted_r_squared": result.adjusted_r_squared.and_then(Number::from_f64),
    "root_mse": result.root_mse.and_then(Number::from_f64),
    "log_likelihood": result.gaussian_working_log_likelihood().and_then(Number::from_f64),
    "log_likelihood_convention": result.gaussian_working_log_likelihood().map(|_| "gaussian_working_model_ml_variance_rss_over_n"),
    "covariance_matrix": result.least_squares.covariance.matrix,
    "covariance_parameter_names": result.least_squares.covariance.parameter_names,
    "sample": result.least_squares.sample.as_ref().map(|sample| json!({
      "total_observations": sample.total_observations, "dropped_observations": sample.dropped_observations,
    })),
    "coefficients": result.coefficients.iter().map(|coefficient| json!({
      "name": coefficient.name, "value": Number::from_f64(coefficient.value),
      "standard_error": coefficient.standard_error.and_then(Number::from_f64),
      "statistic": coefficient.statistic.and_then(Number::from_f64),
      "p_value": coefficient.p_value.and_then(Number::from_f64),
    })).collect::<Vec<_>>()
  });
  if let Some(first_stage) = &result.first_stage {
    data["first_stage"] = json!({
      "test": "classical_excluded_instrument_partial_f",
      "weak_iv_robust_inference_available": false,
      "joint_ar_postestimation_available": true,
      "scalar_ar_confidence_set_available": result.covariance == "nonrobust" && first_stage.len() == 1,
      "coefficient_inference": "conventional_t_not_weak_iv_robust",
      "rows": first_stage.iter().map(|diagnostic| json!({
        "endogenous": diagnostic.endogenous,
        "excluded_instruments": diagnostic.excluded_instruments,
        "observation_count": diagnostic.observation_count,
        "numerator_df": diagnostic.numerator_df,
        "denominator_df": diagnostic.denominator_df,
        "partial_r_squared": diagnostic.partial_r_squared.and_then(Number::from_f64),
        "shea_partial_r_squared": diagnostic.shea_partial_r_squared.and_then(Number::from_f64),
        "f_statistic": diagnostic.f_statistic.and_then(Number::from_f64),
        "p_value": diagnostic.p_value.and_then(Number::from_f64),
      })).collect::<Vec<_>>()
    });
  }
  data
}

fn summary_row_data(row: &SummaryRow) -> Value {
  json!({
    "variable": row.variable,
    "count": row.count,
    "mean": row.mean.and_then(Number::from_f64),
    "std_dev": row.std_dev.and_then(Number::from_f64),
    "minimum": row.minimum.as_ref().map(cell_value_data),
    "maximum": row.maximum.as_ref().map(cell_value_data),
  })
}

fn summarize_result_data(result: &SummarizeResult) -> Value {
  json!({"rows": result.rows.iter().map(summary_row_data).collect::<Vec<_>>()})
}

fn preview_result_data(result: &PreviewResult) -> Value {
  json!({
    "columns": result.columns,
    "rows": result.rows.iter().map(|row| row.iter().map(cell_value_data).collect::<Vec<_>>()).collect::<Vec<_>>(),
  })
}

fn cell_value_data(value: &CellValue) -> Value {
  match value {
    CellValue::Null => Value::Null,
    CellValue::Boolean(value) => json!(value),
    CellValue::SignedInteger(value) => Value::Number(
      Number::from_i128(*value).expect("arbitrary precision is enabled for JSON results"),
    ),
    CellValue::UnsignedInteger(value) => Value::Number(
      Number::from_u128(*value).expect("arbitrary precision is enabled for JSON results"),
    ),
    CellValue::Float(value) => Number::from_f64(*value).map_or(Value::Null, Value::Number),
    CellValue::Decimal { scale, value, .. } => Value::String(decimal_string(*value, *scale)),
    CellValue::Text(value) => Value::String(value.clone()),
    CellValue::Bytes(value) => Value::String(format!("base64:{}", BASE64.encode(value))),
  }
}

fn decimal_string(value: i128, scale: u8) -> String {
  let scale = usize::from(scale);
  let digits = value.unsigned_abs().to_string();
  let formatted = if scale == 0 {
    digits
  } else if digits.len() <= scale {
    format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
  } else {
    let split = digits.len() - scale;
    format!("{}.{}", &digits[..split], &digits[split..])
  };

  if value.is_negative() {
    format!("-{formatted}")
  } else {
    formatted
  }
}

fn load_result_data(result: &LoadResult) -> Value {
  json!({ "dataset": dataset_info_data(&result.dataset) })
}

fn dataset_info_data(dataset: &DatasetInfo) -> Value {
  // Keep the Python DatasetInfo field set; panel and label metadata are not
  // represented by this bounded Rust result type.
  json!({
    "columns": dataset.columns.iter().map(|column| json!({
      "data_type": column.data_type,
      "name": column.name,
    })).collect::<Vec<_>>(),
    "execution_mode": match dataset.execution_mode {
      ExecutionMode::Eager => "eager",
      ExecutionMode::Lazy => "lazy",
    },
    "label_metadata": null,
    "lazy_engine": match dataset.lazy_engine {
      Some(LazyEngine::DuckDb) => Some("duckdb"),
      Some(LazyEngine::Polars) => Some("polars"),
      None => None,
    },
    "panel_metadata": null,
    "path": dataset.source.to_string_lossy(),
    "row_count": dataset.row_count,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unbalanced_random_effects_gls_uses_its_own_type_and_retained_count_metadata() {
    let sizes = [1, 2, 3, 4, 5, 3, 2, 4];
    let mut rows = Vec::new();
    let mut groups = Vec::new();
    for (unit, &size) in sizes.iter().enumerate() {
      let id = unit + 1;
      for time in 0..size {
        let x = 0.3 * id as f64 + 0.2 * time as f64 + 0.07 * ((id * time + 2) % 5) as f64;
        let z = 0.13 * (time * time) as f64 + 0.03 * ((id + 2 * time) % 4) as f64;
        let y = 1.0 + 0.7 * x - 0.3 * z
          + 0.3 * ((id * id + 1) % 5) as f64
          + 0.11 * ((id * time + 3) % 7) as f64;
        rows.push((y, x, z));
        groups.push(id.to_string());
      }
    }
    let n = rows.len();
    let problem = tabdat_stats::EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      rows.iter().map(|row| row.0).collect(),
      rows.iter().map(|row| vec![row.1, row.2]).collect(),
    )
    .unwrap()
    .with_sample(
      tabdat_stats::EstimationSample::new((0..n).collect(), n, 0, None, Some(groups)).unwrap(),
    );
    let result = tabdat_stats::fit_random_effects_unbalanced(&problem, "id").unwrap();
    let state = tabdat_domain::RandomEffectsState {
      time_variable: "time".into(),
      entity_data_type: "BIGINT".into(),
      result,
    };
    let result = ExecutionResult::RandomEffects(Box::new(state));
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(envelope["result_type"], "UnbalancedRandomEffectsGlsResult");
    let data = &envelope["data"];
    assert_eq!(data["variance_component_method"], "unbalanced_swamy_arora");
    assert_eq!(data["balanced"], false);
    assert_eq!(data["group_size"], Value::Null);
    assert_eq!(data["minimum_group_size"], 1);
    assert_eq!(data["maximum_group_size"], 5);
    assert_eq!(data["theta"], Value::Null);
    assert_eq!(data["theta_at_minimum_group_size"], data["theta_min"]);
    assert!(data["theta_min"].as_f64().unwrap() <= data["theta_max"].as_f64().unwrap());
  }

  #[test]
  fn expression_results_encode_owned_metadata_without_affected_or_overflow_counts() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "new\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Generate(tabdat_domain::GenerateResult {
          dataset: dataset.clone(),
        }),
        "GenerateResult",
      ),
      (
        ExecutionResult::Replace(tabdat_domain::ReplaceResult {
          dataset: dataset.clone(),
        }),
        "ReplaceResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
  }

  #[test]
  fn key_integrity_envelopes_preserve_owned_counts_names_and_do_not_admit_grouped_guards() {
    let variables = vec!["key\t列".into(), "key\t列".into()];
    let duplicates = ExecutionResult::Duplicates(tabdat_domain::DuplicatesResult {
      variables: variables.clone(),
      total_rows: u64::MAX,
      unique_groups: 1,
      duplicate_groups: 1,
      duplicate_rows: u64::MAX,
      extra_rows: u64::MAX - 1,
      max_copies: u64::MAX,
    });
    let isid = ExecutionResult::Isid(tabdat_domain::IsidResult {
      variables: variables.clone(),
      total_rows: u64::MAX,
      unique_groups: u64::MAX,
      missing_key_rows: 1,
      missok: true,
    });
    for (result, expected) in [
      (
        duplicates,
        json!({"schema_version":1,"result_type":"DuplicatesResult","data":{"variables":variables,"total_rows":u64::MAX,"unique_groups":1,"duplicate_groups":1,"duplicate_rows":u64::MAX,"extra_rows":u64::MAX-1,"max_copies":u64::MAX}}),
      ),
      (
        isid,
        json!({"schema_version":1,"result_type":"IsidResult","data":{"variables":variables,"total_rows":u64::MAX,"unique_groups":u64::MAX,"missing_key_rows":1,"missok":true}}),
      ),
    ] {
      let text = serialize_execution_result(&result).unwrap();
      assert_eq!(text.lines().count(), 1);
      assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), expected);
    }
    for source in [
      "duplicates",
      "duplicates report x x",
      "isid x",
      "isid x y, missok",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    for source in ["by g: duplicates x", "by g: isid x"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
  }

  #[test]
  fn conditional_summary_admission_allows_only_bare_detail_and_not_grouped_queries() {
    assert!(tabdat_language::parse_command("summarize x if x > 0,").is_err());
    for source in [
      "summarize x if g == 'A'",
      "summarize if g == null",
      "summarize x, detail",
      "summarize x if g == 'A', detail",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    for source in [
      "summarize x, detail detail",
      "summarize x if g == 'A', detail(x)",
      "summarize x if x > 0, limit=2",
      "by g: summarize x, detail",
      "by g: summarize x if x > 0",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
  }

  #[test]
  fn projection_and_row_filter_commands_share_owned_dataset_envelopes() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.parquet"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "wide\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Keep(tabdat_domain::KeepResult {
          dataset: dataset.clone(),
        }),
        "KeepResult",
      ),
      (
        ExecutionResult::Drop(tabdat_domain::DropResult {
          dataset: dataset.clone(),
        }),
        "DropResult",
      ),
      (
        ExecutionResult::Select(tabdat_domain::SelectResult {
          dataset: dataset.clone(),
        }),
        "SelectResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
    for source in [
      "keep x",
      "drop x",
      "select x x",
      "keep if x > 0",
      "drop if x == null",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    for source in ["by g: keep if x > 0", "by g: drop if x == null"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
  }

  #[test]
  fn ordering_results_encode_owned_metadata_without_inventing_keys_or_counts() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "sort\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, tag) in [
      (
        ExecutionResult::Sort(tabdat_domain::SortResult {
          dataset: dataset.clone(),
        }),
        "SortResult",
      ),
      (
        ExecutionResult::Gsort(tabdat_domain::GsortResult {
          dataset: dataset.clone(),
        }),
        "GsortResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], tag);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
    for source in ["sort x x", "gsort -x +y", "gsort `-x`"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    let command = tabdat_language::parse_command("by x: sort y").unwrap();
    assert!(!supports_batch_command(&command));
    assert!(!supports_script_command(&command));
  }

  #[test]
  fn rename_result_encodes_only_owned_post_operation_metadata() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "renamed\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Rename(tabdat_domain::RenameResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "RenameResult");
    assert_eq!(
      value["data"],
      json!({"dataset": dataset_info_data(&dataset)})
    );
    let command = tabdat_language::parse_command("rename old new").unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }

  #[test]
  fn grouped_tables_encode_owned_scalars_with_bounded_count_and_mean_admission() {
    let result = ExecutionResult::By(tabdat_domain::ByResult {
      headers: vec!["group\t列".into(), "Count".into()],
      rows: vec![
        vec![
          CellValue::Text("".into()),
          CellValue::UnsignedInteger(u128::from(u64::MAX)),
        ],
        vec![CellValue::Null, CellValue::SignedInteger(1)],
      ],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
      value,
      json!({"schema_version":1,"result_type":"ByResult","data":{"headers":["group\t列","Count"],"rows":[["",u64::MAX],[null,1]]}})
    );
    for source in [
      "by group: count",
      "by group other: count",
      "by group: summarize x",
      "by group other: summarize",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    for source in [
      "by group: summarize x, detail",
      "by group: summarize x if x > 0",
      "by group: tabulate x",
      "by group: sort x",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
  }

  #[test]
  fn nested_grouped_count_is_rejected_by_parser_and_raw_command_admission() {
    assert!(tabdat_language::parse_command("by group: by other: count").is_err());
    let command = Command::By {
      command: tabdat_domain::ByCommand {
        groups: vec!["group".into()],
        command: Box::new(Command::By {
          command: tabdat_domain::ByCommand {
            groups: vec!["other".into()],
            command: Box::new(Command::Count),
          },
        }),
      },
    };
    assert!(!supports_batch_command(&command));
    assert!(!supports_script_command(&command));
  }

  #[test]
  fn reshape_reports_owned_metadata_without_guessing_direction_or_changed_rows() {
    let dataset = DatasetInfo {
      source: "wide\t目录".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "x\n".into(),
        data_type: "DECIMAL(38,4)".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Reshape(tabdat_domain::ReshapeResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    assert_eq!(
      serde_json::from_str::<Value>(&serialized).unwrap(),
      json!({"schema_version":1,"result_type":"ReshapeResult","data":{"dataset":dataset_info_data(&dataset)}})
    );
    for source in [
      "reshape long x y, i(id) j(t)",
      "reshape wide x, i(id `group name`) j(t)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    let command = tabdat_language::parse_command("by id: reshape long x, i(id) j(t)").unwrap();
    assert!(!supports_batch_command(&command));
    assert!(!supports_script_command(&command));
  }

  #[test]
  fn append_result_preserves_owned_metadata_without_guessing_added_rows_or_sources() {
    let dataset = DatasetInfo {
      source: "primary\t目录".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "x\n".into(),
        data_type: "BIGINT".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Append(tabdat_domain::AppendResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
      value,
      json!({"schema_version":1,"result_type":"AppendResult","data":{"dataset":dataset_info_data(&dataset)}})
    );
    let command = tabdat_language::parse_command("append followup").unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }

  #[test]
  fn join_result_preserves_owned_metadata_without_guessed_matching_diagnostics() {
    let dataset = DatasetInfo {
      source: "primary\t目录".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "right\n".into(),
        data_type: "BIGINT".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Join(tabdat_domain::JoinResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
      value,
      json!({"schema_version":1,"result_type":"JoinResult","data":{"dataset":dataset_info_data(&dataset)}})
    );
    for source in [
      "join lookup on key",
      "join lookup on key year, how=left suffix(_lookup)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn sql_creation_and_activation_report_owned_names_and_metadata() {
    let dataset = DatasetInfo {
      source: "cohort\t目录".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "x\n".into(),
        data_type: "DECIMAL(38,4)".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, name) in [
      (
        ExecutionResult::SqlCreate(tabdat_domain::SqlCreateResult {
          table_name: "cohort\t目录".into(),
          dataset: dataset.clone(),
        }),
        "SqlCreateResult",
      ),
      (
        ExecutionResult::Activate(tabdat_domain::ActivateResult {
          table_name: "cohort\t目录".into(),
          dataset: dataset.clone(),
        }),
        "ActivateResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":name,"data":{"table_name":"cohort\t目录","dataset":dataset_info_data(&dataset)}})
      );
    }
    for source in [
      "sql select 1",
      "sql with q as (select 1 as x) select x from q into cohort",
      "use cohort",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn category_coding_reports_owned_schema_without_dictionary_or_guessed_counts() {
    let dataset = DatasetInfo {
      source: "source\t目录.csv".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "code\n".into(),
        data_type: "BIGINT".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, name) in [
      (
        ExecutionResult::Encode(tabdat_domain::EncodeResult {
          dataset: dataset.clone(),
        }),
        "EncodeResult",
      ),
      (
        ExecutionResult::Decode(tabdat_domain::DecodeResult {
          dataset: dataset.clone(),
        }),
        "DecodeResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":name,"data":{"dataset":dataset_info_data(&dataset)}})
      );
    }
    for source in [
      "encode x, generate(code)",
      "encode x, generate(code) label(codes)",
      "decode code, generate(text)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn label_results_preserve_owned_tagged_keys_order_and_absent_metadata() {
    use tabdat_domain::{LabelMetadata, LabelResult, LabelResultAction, LabelValue, ValueLabelSet};
    let metadata = LabelMetadata {
      variable_labels: vec![
        ("z\t列".into(), "Title\n\u{1b}".into()),
        ("a".into(), "A".into()),
      ],
      value_sets: vec![ValueLabelSet {
        name: "codes".into(),
        mappings: vec![
          (LabelValue::Integer(i64::MAX), "wide".into()),
          (
            LabelValue::Number("9007199254740993.001".into()),
            "exact".into(),
          ),
          (LabelValue::Text("1".into()), "text".into()),
          (LabelValue::Integer(i64::MIN), "negative".into()),
        ],
      }],
      attachments: vec![("z\t列".into(), "codes".into())],
    };
    for (action, spelling) in [
      (LabelResultAction::Variable, "variable"),
      (LabelResultAction::Define, "define"),
      (LabelResultAction::Values, "values"),
      (LabelResultAction::List, "list"),
      (LabelResultAction::Drop, "drop"),
    ] {
      let result = ExecutionResult::Label(LabelResult {
        action,
        message: "Done\n".into(),
        metadata: Some(metadata.clone()),
      });
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":"LabelResult","data":{
          "action":spelling,"message":"Done\n","metadata":{
            "variable_labels":[["z\t列","Title\n\u{1b}"],["a","A"]],
            "value_sets":[{"name":"codes","mappings":[
              [{"kind":"integer","payload":i64::MAX},"wide"],
              [{"kind":"number","payload":"9007199254740993.001"},"exact"],
              [{"kind":"text","payload":"1"},"text"],
              [{"kind":"integer","payload":i64::MIN},"negative"]
            ]}],"attachments":[["z\t列","codes"]]
          }
        }})
      );
    }
    let result = ExecutionResult::Label(LabelResult {
      action: LabelResultAction::List,
      message: "Label dictionary".into(),
      metadata: None,
    });
    let value: Value = serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(
      value["data"],
      json!({"action":"list","message":"Label dictionary","metadata":null})
    );
    for source in [
      "label variable x 'X'",
      "label define codes 1 'one'",
      "label values x codes",
      "label list codes",
      "label drop codes",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn recode_result_encodes_owned_metadata_without_rule_or_changed_cell_fields() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "recoded\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Recode(tabdat_domain::RecodeResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "RecodeResult");
    assert_eq!(
      value["data"],
      json!({"dataset": dataset_info_data(&dataset)})
    );
    for source in [
      "recode x (1=2), replace",
      "recode x (missing=0), generate(flag)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn persistence_json_encodes_requested_paths_and_owned_output_metadata() {
    let path = std::path::PathBuf::from("目录/out\t\".parquet");
    let dataset = DatasetInfo {
      source: path.clone(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "wide\n".to_owned(),
        data_type: "DECIMAL(38,2)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Save(tabdat_domain::SaveResult {
          path: path.clone(),
          dataset: dataset.clone(),
        }),
        "SaveResult",
      ),
      (
        ExecutionResult::Export(tabdat_domain::ExportResult {
          path: path.clone(),
          dataset: dataset.clone(),
        }),
        "ExportResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(value["data"]["path"], "目录/out\t\".parquet");
      assert_eq!(value["data"]["dataset"], dataset_info_data(&dataset));
      assert_eq!(value["data"]["dataset"]["row_count"], u64::MAX);
    }
    for input in [
      "save out.parquet",
      "save out.parquet, replace",
      "export out.csv",
      "export out.csv, replace",
    ] {
      let command = tabdat_language::parse_command(input).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn describe_json_preserves_owned_metadata_with_existing_placeholder_policy() {
    let result = ExecutionResult::Describe(tabdat_domain::DescribeResult {
      dataset: DatasetInfo {
        source: std::path::PathBuf::from("目录/source\t\".parquet"),
        row_count: u64::MAX,
        columns: vec![
          tabdat_domain::ColumnInfo {
            name: "é\t\"\n".to_owned(),
            data_type: "DECIMAL(38,2)".to_owned(),
          },
          tabdat_domain::ColumnInfo {
            name: "wide".to_owned(),
            data_type: "UHUGEINT".to_owned(),
          },
        ],
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
      },
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "DescribeResult");
    assert_eq!(
      value["data"]["dataset"],
      json!({
        "path": "目录/source\t\".parquet", "row_count": u64::MAX,
        "columns": [{"name": "é\t\"\n", "data_type": "DECIMAL(38,2)"}, {"name": "wide", "data_type": "UHUGEINT"}],
        "execution_mode": "eager", "lazy_engine": null,
        "label_metadata": null, "panel_metadata": null,
      })
    );
    assert!(supports_batch_command(&Command::Describe));
    assert!(supports_script_command(&Command::Describe));
  }

  #[test]
  fn owned_tables_encode_cells_without_reinterpreting_vif_labels() {
    let result = ExecutionResult::Table(tabdat_domain::TableResult {
      headers: vec!["Variable\n".to_owned(), "VIF".to_owned()],
      rows: vec![
        vec![
          CellValue::Text("mean_vif".to_owned()),
          CellValue::Float(0.0),
        ],
        vec![
          CellValue::Text("mean_vif".to_owned()),
          CellValue::Float(f64::INFINITY),
        ],
        vec![CellValue::Text("label\t\"\n".to_owned()), CellValue::Null],
        vec![
          CellValue::Float(f64::NAN),
          CellValue::Float(f64::NEG_INFINITY),
        ],
        vec![
          CellValue::SignedInteger(i128::MAX),
          CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          },
        ],
      ],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "TableResult");
    assert_eq!(value["data"]["headers"], json!(["Variable\n", "VIF"]));
    assert_eq!(
      value["data"]["rows"],
      json!([
        ["mean_vif", 0.0],
        ["mean_vif", null],
        ["label\t\"\n", null],
        [null, null],
        [i128::MAX, "-0.01"],
      ])
    );
    for input in ["estat residuals", "by x: summarize x, detail"] {
      let command = tabdat_language::parse_command(input).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
    let command = tabdat_language::parse_command("estat vif").unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }

  #[test]
  fn codebook_json_uses_existing_scalar_policy_without_loss_of_owned_labels_or_counts() {
    let row = |variable: &str, data_type: &str, examples| tabdat_domain::CodebookRow {
      variable: variable.to_owned(),
      data_type: data_type.to_owned(),
      nonmissing: u64::MAX,
      missing: 0,
      distinct: u64::MAX,
      examples,
    };
    let result = ExecutionResult::Codebook(CodebookResult {
      rows: vec![
        row(
          "signed",
          "HUGEINT",
          vec![
            CellValue::SignedInteger(i128::MIN),
            CellValue::SignedInteger(i128::MAX),
          ],
        ),
        row(
          "unsigned",
          "UHUGEINT",
          vec![CellValue::UnsignedInteger(u128::MAX)],
        ),
        row(
          "decimal",
          "DECIMAL(38,2)",
          vec![CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          }],
        ),
        row("blob", "BLOB", vec![CellValue::Bytes(vec![0, 255])]),
        row(
          "label\t\"\n",
          "VARCHAR",
          vec![CellValue::Text("text\t\"\n".to_owned())],
        ),
        row(
          "bool",
          "BOOLEAN",
          vec![CellValue::Boolean(true), CellValue::Boolean(false)],
        ),
        row(
          "nonfinite",
          "DOUBLE",
          vec![
            CellValue::Float(f64::NAN),
            CellValue::Float(f64::INFINITY),
            CellValue::Float(f64::NEG_INFINITY),
          ],
        ),
      ],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["result_type"], "CodebookResult");
    let rows = value["data"]["rows"].as_array().unwrap();
    assert!(
      rows
        .iter()
        .all(|row| row["nonmissing"] == u64::MAX && row["distinct"] == u64::MAX)
    );
    for (index, expected) in [
      json!([i128::MIN, i128::MAX]),
      json!([u128::MAX]),
      json!(["-0.01"]),
      json!(["base64:AP8="]),
      json!(["text\t\"\n"]),
      json!([true, false]),
      json!([null, null, null]),
    ]
    .iter()
    .enumerate()
    {
      assert_eq!(rows[index]["examples"], *expected);
    }
    assert_eq!(rows[4]["variable"], "label\t\"\n");
    let empty = ExecutionResult::Codebook(CodebookResult { rows: vec![] });
    let value: Value = serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(value["data"]["rows"], json!([]));
  }

  #[test]
  fn missing_json_preserves_large_counts_labels_null_percent_and_empty_rows() {
    let result = ExecutionResult::Missing(MissingResult {
      rows: vec![tabdat_domain::MissingRow {
        variable: "label\t\"\n".to_owned(),
        data_type: "DECIMAL(38,2)".to_owned(),
        total: u64::MAX,
        missing: 1,
        nonmissing: u64::MAX - 1,
        missing_percent: f64::NAN,
      }],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let envelope: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
      envelope,
      json!({"schema_version": 1, "result_type": "MissingResult", "data": {
        "missing_definition": "sql_null", "rows": [{"variable": "label\t\"\n", "data_type": "DECIMAL(38,2)",
        "total": u64::MAX, "missing": 1, "nonmissing": u64::MAX - 1, "missing_percent": null}]
      }})
    );
    let empty = ExecutionResult::Missing(MissingResult { rows: vec![] });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(
      envelope["data"],
      json!({"missing_definition": "sql_null", "rows": []})
    );
  }

  #[test]
  fn ttest_json_boundary_preserves_labels_zero_and_null_policies() {
    let group = TtestGroupStats {
      name: "label\tline\n\"quoted\"".to_owned(),
      observations: 3,
      mean: f64::NAN,
      standard_error: 0.0,
      standard_deviation: f64::INFINITY,
      ci_lower: f64::NEG_INFINITY,
      ci_upper: 2.0,
    };
    // Synthetic owned values exercise the encoder, not a fitted inference case.
    let result = ExecutionResult::Ttest(Box::new(TtestResult {
      varname1: "x".to_owned(),
      varname2: None,
      by_variable: Some("grp".to_owned()),
      is_paired: false,
      is_welch: false,
      null_value: 0.0,
      group1_stats: group.clone(),
      group2_stats: Some(TtestGroupStats {
        name: "B".to_owned(),
        ..group.clone()
      }),
      combined_stats: Some(TtestGroupStats {
        name: "combined".to_owned(),
        observations: 6,
        ..group.clone()
      }),
      difference_stats: TtestGroupStats {
        name: "diff".to_owned(),
        observations: 4,
        ..group
      },
      t_statistic: f64::INFINITY,
      degrees_of_freedom: 4.0,
      p_left: 0.0,
      p_two: f64::INFINITY,
      p_right: f64::NAN,
    }));
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let envelope: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(envelope["result_type"], "TtestResult");
    let data = &envelope["data"];
    assert_eq!(
      data["group1_stats"],
      json!({
        "name": "label\tline\n\"quoted\"", "count": 3, "mean": null,
        "standard_error": 0.0, "standard_deviation": null, "ci_lower": null, "ci_upper": 2.0,
      })
    );
    assert_eq!(data["combined_stats"]["count"], 6);
    assert!(data["difference_stats"]["count"].is_null());
    assert_eq!(data["null_value"], 0.0);
    assert_eq!(data["p_left"], 0.0);
    assert!(data["p_two"].is_null());
    assert!(data["p_right"].is_null());
    assert!(data["t_statistic"].is_null());
    assert_eq!(data["degrees_of_freedom"], 4.0);
  }

  #[test]
  fn tabulate_serialization_preserves_owned_labels_scalar_policies_and_empty_rows() {
    let result = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "label\tcategory".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![
        vec![
          CellValue::Text("label\n\"quoted\"".to_owned()),
          CellValue::SignedInteger(i128::MAX),
          CellValue::Float(50.0),
        ],
        vec![
          CellValue::Null,
          CellValue::SignedInteger(1),
          CellValue::Float(f64::NAN),
        ],
      ],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(
      envelope,
      json!({"schema_version": 1, "result_type": "TabulateResult", "data": {
        "headers": ["label\tcategory", "Count", "Percent"], "rows": [["label\n\"quoted\"", i128::MAX, 50.0], [null, 1, null]]
      }})
    );
    let empty = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec!["cat".to_owned(), "Count".to_owned(), "Percent".to_owned()],
      rows: vec![],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(
      envelope["data"],
      json!({"headers": ["cat", "Count", "Percent"], "rows": []})
    );
  }

  #[test]
  fn summary_serialization_preserves_order_precision_and_null_policies() {
    use tabdat_domain::SummaryRow;
    let large = SummaryRow {
      variable: "large".to_owned(),
      count: u64::MAX,
      mean: None,
      std_dev: None,
      minimum: Some(CellValue::SignedInteger(i128::MIN)),
      maximum: Some(CellValue::UnsignedInteger(u128::MAX)),
    };
    let result = ExecutionResult::Summarize(SummarizeResult {
      rows: vec![
        large.clone(),
        SummaryRow {
          variable: "empty".to_owned(),
          count: 0,
          mean: None,
          std_dev: None,
          minimum: None,
          maximum: None,
        },
        SummaryRow {
          variable: "decimal".to_owned(),
          count: 1,
          mean: Some(0.01),
          std_dev: None,
          minimum: Some(CellValue::Decimal {
            width: 38,
            scale: 2,
            value: 1,
          }),
          maximum: Some(CellValue::Decimal {
            width: 38,
            scale: 2,
            value: 1,
          }),
        },
        SummaryRow {
          variable: "nonfinite".to_owned(),
          count: 2,
          mean: Some(f64::INFINITY),
          std_dev: Some(f64::NAN),
          minimum: Some(CellValue::Float(f64::NEG_INFINITY)),
          maximum: Some(CellValue::Float(f64::INFINITY)),
        },
        large,
      ],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["result_type"], "SummarizeResult");
    let rows = envelope["data"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], rows[4]);
    assert_eq!(rows[0]["count"], json!(u64::MAX));
    assert_eq!(rows[0]["minimum"], json!(i128::MIN));
    assert_eq!(rows[0]["maximum"], json!(u128::MAX));
    assert_eq!(
      rows[1],
      json!({"variable": "empty", "count": 0, "mean": null, "std_dev": null, "minimum": null, "maximum": null})
    );
    assert_eq!(
      rows[2],
      json!({"variable": "decimal", "count": 1, "mean": 0.01, "std_dev": null, "minimum": "0.01", "maximum": "0.01"})
    );
    assert_eq!(
      rows[3],
      json!({"variable": "nonfinite", "count": 2, "mean": null, "std_dev": null, "minimum": null, "maximum": null})
    );
  }

  #[test]
  fn preview_serialization_preserves_cell_conversion_policies() {
    let result = ExecutionResult::Head(PreviewResult {
      columns: vec![
        "null".to_owned(),
        "boolean".to_owned(),
        "signed".to_owned(),
        "unsigned".to_owned(),
        "finite".to_owned(),
        "nan".to_owned(),
        "positive_infinity".to_owned(),
        "negative_infinity".to_owned(),
        "decimal".to_owned(),
        "small_decimal".to_owned(),
        "zero_decimal".to_owned(),
        "text".to_owned(),
        "bytes".to_owned(),
        "empty_bytes".to_owned(),
      ],
      rows: vec![vec![
        CellValue::Null,
        CellValue::Boolean(true),
        CellValue::SignedInteger(i128::MIN),
        CellValue::UnsignedInteger(u128::MAX),
        CellValue::Float(1.25),
        CellValue::Float(f64::NAN),
        CellValue::Float(f64::INFINITY),
        CellValue::Float(f64::NEG_INFINITY),
        CellValue::Decimal {
          width: 8,
          scale: 2,
          value: -120,
        },
        CellValue::Decimal {
          width: 5,
          scale: 3,
          value: 7,
        },
        CellValue::Decimal {
          width: 3,
          scale: 0,
          value: 0,
        },
        CellValue::Text("λ".to_owned()),
        CellValue::Bytes(vec![0xff, 0x00]),
        CellValue::Bytes(Vec::new()),
      ]],
    });

    let serialized = serialize_execution_result(&result).expect("preview is supported");
    let envelope: Value = serde_json::from_str(&serialized).expect("valid JSON");

    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["result_type"], "PreviewResult");
    assert_eq!(envelope["data"]["columns"][0], "null");
    let row = &envelope["data"]["rows"][0];
    assert_eq!(row[0], Value::Null);
    assert_eq!(row[1], true);
    assert_eq!(
      row[2].as_number().unwrap().to_string(),
      i128::MIN.to_string()
    );
    assert_eq!(
      row[3].as_number().unwrap().to_string(),
      u128::MAX.to_string()
    );
    assert_eq!(row[4], 1.25);
    assert_eq!(row[5], Value::Null);
    assert_eq!(row[6], Value::Null);
    assert_eq!(row[7], Value::Null);
    assert_eq!(row[8], "-1.20");
    assert_eq!(row[9], "0.007");
    assert_eq!(row[10], "0");
    assert_eq!(row[11], "λ");
    assert_eq!(row[12], "base64:/wA=");
    assert_eq!(row[13], "base64:");
    assert!(!serialized.contains("NaN"));
    assert!(!serialized.contains("Infinity"));
  }

  #[test]
  fn tail_and_empty_previews_use_the_same_result_envelope() {
    let result = ExecutionResult::Tail(PreviewResult {
      columns: vec!["x".to_owned()],
      rows: Vec::new(),
    });

    let serialized = serialize_execution_result(&result).expect("tail preview is supported");
    let envelope: Value = serde_json::from_str(&serialized).expect("valid JSON");

    assert_eq!(envelope["result_type"], "PreviewResult");
    assert_eq!(envelope["data"], json!({"columns": ["x"], "rows": []}));
  }
}
