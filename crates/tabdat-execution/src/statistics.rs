use crate::*;

impl Session {
  /// Execute a linear regression command (OLS, WLS, GLS, robust, cluster).
  pub fn execute_regress(
    &mut self,
    command: &RegressCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "regress" })?
      .clone();

    let mut referenced_variables = Vec::new();
    referenced_variables.push(command.outcome.clone());
    referenced_variables.extend(command.predictors.iter().cloned());
    if let Some(w) = &command.weight_variable {
      referenced_variables.push(w.clone());
    }
    if let Some(c) = &command.cluster_variable {
      referenced_variables.push(c.clone());
    }

    let column_types: HashMap<&str, &str> = dataset
      .columns
      .iter()
      .map(|col| (col.name.as_str(), col.data_type.as_str()))
      .collect();

    let missing = referenced_variables
      .iter()
      .filter(|var| !column_types.contains_key(var.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::RegressUnknownVariable { variables: missing });
    }

    // Verify outcome, predictors, and weight (if present) are numeric
    let mut numeric_candidates = Vec::new();
    numeric_candidates.push(command.outcome.clone());
    numeric_candidates.extend(command.predictors.iter().cloned());
    if let Some(w) = &command.weight_variable {
      numeric_candidates.push(w.clone());
    }

    let non_numeric = numeric_candidates
      .iter()
      .filter(|var| {
        column_types
          .get(var.as_str())
          .is_none_or(|data_type| !is_numeric_data_type(data_type))
      })
      .cloned()
      .collect::<Vec<_>>();
    if !non_numeric.is_empty() {
      return Err(RuntimeError::RegressRequiresNumeric {
        variables: non_numeric,
      });
    }

    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "regress" })?;

    // Prepare SELECT query
    let mut select_columns = Vec::new();
    select_columns.push(quote_identifier(&command.outcome));
    for p in &command.predictors {
      select_columns.push(quote_identifier(p));
    }
    if let Some(c) = &command.cluster_variable {
      select_columns.push(quote_identifier(c));
    }
    if let Some(w) = &command.weight_variable {
      select_columns.push(quote_identifier(w));
    }

    let select_sql = select_columns.join(", ");
    let query = format!("SELECT {select_sql} FROM {ACTIVE_TABLE}");

    let p_count = command.predictors.len();
    let has_cluster = command.cluster_variable.is_some();
    let has_weight = command.weight_variable.is_some();
    let cluster_idx = 1 + p_count;
    let weight_idx = 1 + p_count + if has_cluster { 1 } else { 0 };

    let mut total_rows = 0usize;
    let mut retained_indices = Vec::new();
    let mut response = Vec::new();
    let mut design_matrix = Vec::new();
    let mut cluster_groups = Vec::new();
    let mut weights = Vec::new();

    backend.visit_rows(
      &query,
      |row| {
        let row_idx = total_rows;
        total_rows += 1;

        // Extract outcome
        let outcome_cell = read_owned_cell(row, 0, true, "could not read outcome cell")?;
        let outcome_val = match cell_to_f64(outcome_cell) {
          Some(v) if v.is_finite() => v,
          _ => return Ok(()),
        };

        // Extract predictors
        let mut pred_vals = Vec::with_capacity(p_count);
        let mut pred_valid = true;
        for j in 0..p_count {
          let cell = read_owned_cell(row, 1 + j, true, "could not read predictor cell")?;
          match cell_to_f64(cell) {
            Some(v) if v.is_finite() => pred_vals.push(v),
            _ => {
              pred_valid = false;
              break;
            }
          }
        }
        if !pred_valid {
          return Ok(());
        }

        // Extract cluster if present
        let cluster_val = if has_cluster {
          let cell = read_owned_cell(row, cluster_idx, true, "could not read cluster cell")?;
          match cell_to_string(cell) {
            Some(s) => s,
            None => return Ok(()),
          }
        } else {
          String::new()
        };

        // Extract weight if present
        let weight_val = if has_weight {
          let cell = read_owned_cell(row, weight_idx, true, "could not read weight cell")?;
          let raw_w = match cell_to_f64(cell) {
            Some(v) if v.is_finite() => v,
            _ => return Ok(()),
          };
          match command.estimator {
            RegressEstimator::Wls => {
              if raw_w <= 0.0 {
                return Err(RuntimeError::RegressRequiresPositiveWeights);
              }
              raw_w
            }
            RegressEstimator::Gls => {
              if raw_w <= 0.0 {
                return Err(RuntimeError::RegressRequiresPositiveSigma);
              }
              1.0 / raw_w
            }
            RegressEstimator::Ols => raw_w,
          }
        } else {
          1.0
        };

        retained_indices.push(row_idx);
        response.push(outcome_val);
        design_matrix.push(pred_vals);
        if has_cluster {
          cluster_groups.push(cluster_val);
        }
        if has_weight {
          weights.push(weight_val);
        }
        Ok(())
      },
      |message| RuntimeError::RegressFailed { message },
    )?;

    if retained_indices.is_empty() {
      return Err(RuntimeError::RegressNoObservations);
    }

    let dropped_observations = total_rows - retained_indices.len();
    let sample = tabdat_stats::EstimationSample::new(
      retained_indices,
      total_rows,
      dropped_observations,
      has_weight.then_some(weights.clone()),
      has_cluster.then_some(cluster_groups),
    )
    .map_err(|e| RuntimeError::RegressFailed {
      message: e.to_string(),
    })?;

    let mut problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design_matrix,
    )
    .map_err(|e| RuntimeError::RegressFailed {
      message: e.to_string(),
    })?
    .with_intercept(command.include_intercept)
    .with_sample(sample);

    if has_weight {
      problem = problem.with_weights(weights);
    }

    let covariance_type = if let Some(cluster_var) = &command.cluster_variable {
      tabdat_stats::CovarianceType::Cluster(cluster_var.clone())
    } else if command.robust {
      tabdat_stats::CovarianceType::RobustHc1
    } else {
      tabdat_stats::CovarianceType::NonRobust
    };

    let options = tabdat_stats::LeastSquaresOptions { covariance_type };
    let least_squares_result = tabdat_stats::fit_least_squares_with_options(&problem, &options)
      .map_err(|e| RuntimeError::RegressFailed {
        message: e.to_string(),
      })?;

    self.estimation = Some(EstimationState::Linear(Box::new(LinearRegressionState {
      outcome: command.outcome.clone(),
      predictors: command.predictors.clone(),
      predictor_design: problem.design_matrix,
      include_intercept: command.include_intercept,
      result: least_squares_result.clone(),
    })));

    let covariance_str = match &least_squares_result.covariance.covariance_type {
      tabdat_stats::CovarianceType::NonRobust => "nonrobust".to_string(),
      tabdat_stats::CovarianceType::RobustHc1 => "robust".to_string(),
      tabdat_stats::CovarianceType::Cluster(var) => format!("cluster({var})"),
    };
    let estimator_str = match command.estimator {
      RegressEstimator::Ols => "ols".to_string(),
      RegressEstimator::Wls => "wls".to_string(),
      RegressEstimator::Gls => "gls".to_string(),
    };

    let result = RegressionResult {
      outcome: command.outcome.clone(),
      predictors: command.predictors.clone(),
      estimator: estimator_str,
      covariance: covariance_str,
      observation_count: least_squares_result.fit_statistics.observation_count,
      include_intercept: command.include_intercept,
      r_squared: least_squares_result.fit_statistics.r_squared,
      adjusted_r_squared: least_squares_result.fit_statistics.adjusted_r_squared,
      root_mse: least_squares_result.fit_statistics.root_mse,
      coefficients: least_squares_result.coefficients.clone(),
      least_squares: least_squares_result,
    };

    Ok(ExecutionResult::Regression(Box::new(result)))
  }
  pub(crate) fn execute_lincom(
    &self,
    command: &LincomCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let model = self
      .linear_state()
      .ok_or(RuntimeError::LincomRequiresPriorEstimation)?;
    let parameter_indices = model
      .result
      .parameter_names
      .iter()
      .enumerate()
      .map(|(index, name)| (name.clone(), index))
      .collect::<HashMap<_, _>>();
    let combination = affine_lincom_expression(
      &command.expression,
      &parameter_indices,
      model.result.parameter_names.len(),
    )?;
    let post_estimation = tabdat_stats::PostEstimationModel::from_least_squares(&model.result);
    let inference = post_estimation
      .linear_combination_with_inference(&combination.weights, combination.offset, 95.0)
      .map_err(|error| RuntimeError::LincomFailed {
        message: error.to_string(),
      })?;

    Ok(ExecutionResult::Lincom(LincomResult {
      label: format_lincom_expression(&command.expression),
      estimate: inference.estimate,
      standard_error: inference.standard_error,
      statistic: inference.statistic.unwrap_or(f64::NAN),
      p_value: inference.p_value,
      ci_lower: inference.ci_lower,
      ci_upper: inference.ci_upper,
      ci_level: inference.confidence_level,
      df_residual: Some(inference.degrees_of_freedom),
    }))
  }
  pub(crate) fn execute_test(
    &self,
    command: &TestCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let model = self
      .linear_state()
      .ok_or(RuntimeError::TestRequiresPriorEstimation)?;
    if command.constraints.is_empty() {
      return Err(RuntimeError::TestFailed {
        message: "test command expects a list of variables or constraints".to_owned(),
      });
    }

    let parameter_indices = model
      .result
      .parameter_names
      .iter()
      .enumerate()
      .map(|(index, name)| (name.clone(), index))
      .collect::<HashMap<_, _>>();
    let mut r_matrix = Vec::with_capacity(command.constraints.len());
    let mut r_vector = Vec::with_capacity(command.constraints.len());
    let mut constraints = Vec::with_capacity(command.constraints.len());
    for constraint in &command.constraints {
      let affine = affine_test_expression(
        constraint,
        &parameter_indices,
        model.result.parameter_names.len(),
      )
      .map_err(|message| RuntimeError::TestFailed { message })?;
      r_matrix.push(affine.weights);
      r_vector.push(-affine.offset);
      constraints.push(format_test_constraint(constraint));
    }

    let post_estimation = tabdat_stats::PostEstimationModel::from_least_squares(&model.result);
    let wald = post_estimation
      .test_linear_hypothesis(&r_matrix, &r_vector)
      .map_err(|error| RuntimeError::TestFailed {
        message: match error {
          tabdat_stats::StatsError::SingularMatrix(_) => {
            "constraints are collinear or singular".to_owned()
          }
          other => other.to_string(),
        },
      })?;
    if wald.df_denom == 0 {
      return Err(RuntimeError::TestFailed {
        message: "test requires positive residual degrees of freedom".to_owned(),
      });
    }

    let p_value = tabdat_stats::f_distribution_survival_probability(
      wald.f_statistic,
      wald.df_num as f64,
      wald.df_denom as f64,
    );
    Ok(ExecutionResult::Test(TestResult {
      constraints,
      statistic: wald.f_statistic,
      p_value,
      df: wald.df_num,
      df_residual: Some(wald.df_denom),
      is_chi2: false,
    }))
  }
  pub(crate) fn execute_ttest(
    &self,
    command: &TtestCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "ttest" })?;

    if let Some(null_value) = command.value.as_deref() {
      validate_ttest_numeric(dataset, &[&command.varname1])?;
      let null_value = null_value
        .parse::<f64>()
        .map_err(|_| RuntimeError::TtestFailed {
          message: format!("ttest: invalid null value: {null_value}"),
        })?;
      let backend = self
        .backend
        .as_ref()
        .ok_or(RuntimeError::NoActiveDataset { command: "ttest" })?;
      let query = format!(
        "SELECT {} FROM {ACTIVE_TABLE}",
        quote_identifier(&command.varname1)
      );
      let mut observations = Vec::new();
      backend.visit_rows(
        &query,
        |row| {
          let value = read_owned_cell(row, 0, false, "ttest could not read a numeric observation")?;
          if let Some(value) = cell_to_f64(value).filter(|value| value.is_finite()) {
            observations.push(value);
          }
          Ok(())
        },
        |message| RuntimeError::TtestFailed { message },
      )?;
      if observations.is_empty() {
        return Err(RuntimeError::TtestNoValidNumericObservations);
      }
      let statistics =
        tabdat_stats::one_sample_ttest(&observations, null_value).map_err(|error| {
          RuntimeError::TtestFailed {
            message: error.to_string(),
          }
        })?;
      return Ok(ExecutionResult::Ttest(Box::new(TtestResult {
        varname1: command.varname1.clone(),
        varname2: None,
        by_variable: None,
        is_paired: false,
        is_welch: false,
        null_value,
        group1_stats: ttest_group_stats(&command.varname1, &statistics.group1),
        group2_stats: None,
        combined_stats: None,
        difference_stats: ttest_group_stats("mean", &statistics.difference),
        t_statistic: statistics.statistic,
        degrees_of_freedom: statistics.degrees_of_freedom,
        p_left: statistics.p_left,
        p_two: statistics.p_two,
        p_right: statistics.p_right,
      })));
    }

    if let Some(varname2) = command.varname2.as_deref() {
      validate_ttest_numeric(dataset, &[&command.varname1, varname2])?;
      let backend = self
        .backend
        .as_ref()
        .ok_or(RuntimeError::NoActiveDataset { command: "ttest" })?;
      let query = format!(
        "SELECT {}, {} FROM {ACTIVE_TABLE}",
        quote_identifier(&command.varname1),
        quote_identifier(varname2)
      );
      let mut first = Vec::new();
      let mut second = Vec::new();
      backend.visit_rows(
        &query,
        |row| {
          let first_value = read_owned_cell(
            row,
            0,
            false,
            "ttest could not read the first paired observation",
          )?;
          let second_value = read_owned_cell(
            row,
            1,
            false,
            "ttest could not read the second paired observation",
          )?;
          if let (Some(left), Some(right)) = (cell_to_f64(first_value), cell_to_f64(second_value))
            && left.is_finite()
            && right.is_finite()
          {
            first.push(left);
            second.push(right);
          }
          Ok(())
        },
        |message| RuntimeError::TtestFailed { message },
      )?;
      if first.is_empty() {
        return Err(RuntimeError::TtestNoValidPairedNumericObservations);
      }
      let statistics =
        tabdat_stats::paired_ttest(&first, &second).map_err(|error| RuntimeError::TtestFailed {
          message: error.to_string(),
        })?;
      let group2 = statistics
        .group2
        .as_ref()
        .expect("paired t-test has two groups");
      return Ok(ExecutionResult::Ttest(Box::new(TtestResult {
        varname1: command.varname1.clone(),
        varname2: Some(varname2.to_owned()),
        by_variable: None,
        is_paired: true,
        is_welch: false,
        null_value: 0.0,
        group1_stats: ttest_group_stats(&command.varname1, &statistics.group1),
        group2_stats: Some(ttest_group_stats(varname2, group2)),
        combined_stats: None,
        difference_stats: ttest_group_stats("diff", &statistics.difference),
        t_statistic: statistics.statistic,
        degrees_of_freedom: statistics.degrees_of_freedom,
        p_left: statistics.p_left,
        p_two: statistics.p_two,
        p_right: statistics.p_right,
      })));
    }

    if let Some(by_variable) = command.by_variable.as_deref() {
      if !dataset
        .columns
        .iter()
        .any(|column| column.name == by_variable)
      {
        return Err(RuntimeError::TtestUnknownVariable {
          variables: vec![by_variable.to_owned()],
        });
      }
      validate_ttest_numeric(dataset, &[&command.varname1])?;
      let group_type = dataset
        .columns
        .iter()
        .find(|column| column.name == by_variable)
        .map(|column| column.data_type.as_str())
        .expect("group variable was checked above");
      let (group_expression, group_kind) = ttest_group_expression(by_variable, group_type)?;
      let backend = self
        .backend
        .as_ref()
        .ok_or(RuntimeError::NoActiveDataset { command: "ttest" })?;
      let quoted_group = quote_identifier(by_variable);
      let query = format!(
        "SELECT {}, {group_expression} FROM {ACTIVE_TABLE} WHERE {quoted_group} IS NOT NULL",
        quote_identifier(&command.varname1)
      );
      let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
      backend.visit_rows(
        &query,
        |row| {
          let value = read_owned_cell(row, 0, false, "ttest could not read a numeric observation")?;
          let Some(value) = cell_to_f64(value).filter(|value| value.is_finite()) else {
            return Ok(());
          };
          let group_name = match group_kind {
            TtestGroupValueKind::Numeric => {
              let group =
                read_owned_cell(row, 1, false, "ttest could not read a numeric group value")?;
              python_ttest_group_label(group).ok_or_else(|| RuntimeError::TtestFailed {
                message: "ttest could not stringify a numeric group value".to_owned(),
              })?
            }
            TtestGroupValueKind::Boolean | TtestGroupValueKind::Text => {
              read_owned_text(row, 1, "ttest could not stringify a group value")?
            }
            TtestGroupValueKind::Temporal => {
              let value =
                read_owned_text(row, 1, "ttest could not stringify a temporal group value")?;
              python_temporal_label(&value)
            }
            TtestGroupValueKind::Blob => {
              let hex = read_owned_text(row, 1, "ttest could not read a blob group value")?;
              let bytes = decode_hex(&hex).ok_or_else(|| RuntimeError::TtestFailed {
                message: "ttest could not decode a blob group value".to_owned(),
              })?;
              python_bytes_repr(&bytes)
            }
          };
          groups.entry(group_name).or_default().push(value);
          Ok(())
        },
        |message| RuntimeError::TtestFailed { message },
      )?;
      if groups.len() != 2 {
        return Err(RuntimeError::TtestInvalidGroupCount {
          found: groups.len(),
        });
      }
      let mut grouped_samples = groups.into_iter();
      let (group1_name, group1_values) = grouped_samples.next().expect("two groups were checked");
      let (group2_name, group2_values) = grouped_samples.next().expect("two groups were checked");
      let statistics =
        tabdat_stats::independent_ttest(&group1_values, &group2_values, command.welch).map_err(
          |error| RuntimeError::TtestFailed {
            message: error.to_string(),
          },
        )?;
      let group2 = statistics
        .group2
        .as_ref()
        .expect("independent t-test has two groups");
      let combined = statistics
        .combined
        .as_ref()
        .expect("independent t-test has a combined sample");
      return Ok(ExecutionResult::Ttest(Box::new(TtestResult {
        varname1: command.varname1.clone(),
        varname2: None,
        by_variable: Some(by_variable.to_owned()),
        is_paired: false,
        is_welch: command.welch,
        null_value: 0.0,
        group1_stats: ttest_group_stats(&group1_name, &statistics.group1),
        group2_stats: Some(ttest_group_stats(&group2_name, group2)),
        combined_stats: Some(ttest_group_stats("combined", combined)),
        difference_stats: ttest_group_stats("diff", &statistics.difference),
        t_statistic: statistics.statistic,
        degrees_of_freedom: statistics.degrees_of_freedom,
        p_left: statistics.p_left,
        p_two: statistics.p_two,
        p_right: statistics.p_right,
      })));
    }

    Err(RuntimeError::TtestFailed {
      message: "ttest: invalid state".to_owned(),
    })
  }
  pub(crate) fn execute_estat(
    &self,
    command: &EstatCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.subcommand != EstatSubcommand::Vif {
      return Err(RuntimeError::UnsupportedCommand { name: "estat" });
    }
    if self.active_dataset.is_none() {
      return Err(RuntimeError::NoActiveDataset { command: "estat" });
    }
    let model = self
      .linear_state()
      .ok_or(RuntimeError::EstatRequiresPriorRegression)?;
    let vif_values = tabdat_stats::variance_inflation_factors(
      &model.predictors,
      &model.predictor_design,
      model.include_intercept,
    )
    .map_err(|_| RuntimeError::EstatVifFailed)?;

    let mut rows = Vec::with_capacity(vif_values.len() + 1);
    let mut available_values = Vec::with_capacity(vif_values.len());
    for (variable, vif) in model.predictors.iter().zip(vif_values) {
      let vif = vif.filter(|value| !value.is_nan());
      if let Some(value) = vif {
        available_values.push(value);
        rows.push(vec![
          CellValue::Text(variable.clone()),
          CellValue::Float(value),
        ]);
      } else {
        rows.push(vec![CellValue::Text(variable.clone()), CellValue::Null]);
      }
    }
    if !available_values.is_empty() {
      let mean_vif =
        tabdat_stats::mean(&available_values).map_err(|_| RuntimeError::EstatVifFailed)?;
      rows.push(vec![
        CellValue::Text("mean_vif".to_owned()),
        CellValue::Float(mean_vif),
      ]);
    }

    Ok(ExecutionResult::Table(TableResult {
      headers: vec!["Variable".to_owned(), "VIF".to_owned()],
      rows,
    }))
  }
  pub(crate) fn execute_predict(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "predict" })?
      .clone();

    if !matches!(command.kind, PredictKind::Xb | PredictKind::Residuals) {
      return Err(RuntimeError::UnsupportedCommand { name: "predict" });
    }

    let (outcome, predictors, intercept, coefficients) = {
      let model = self
        .linear_state()
        .ok_or(RuntimeError::PredictRequiresPriorRegression)?;
      if dataset
        .columns
        .iter()
        .any(|column| column.name == command.target_variable)
      {
        return Err(RuntimeError::PredictTargetExists {
          variable: command.target_variable.clone(),
        });
      }

      let mut required = model.predictors.clone();
      if command.kind == PredictKind::Residuals {
        required.push(model.outcome.clone());
      }
      let missing = required
        .iter()
        .filter(|variable| {
          !dataset
            .columns
            .iter()
            .any(|column| column.name == **variable)
        })
        .cloned()
        .collect::<Vec<_>>();
      if !missing.is_empty() {
        return Err(RuntimeError::PredictUnknownVariable { variables: missing });
      }

      let coefficient_offset = usize::from(model.include_intercept);
      if model.result.coefficients.len() != model.predictors.len() + coefficient_offset {
        return Err(RuntimeError::PredictFailed);
      }
      let intercept = model
        .include_intercept
        .then(|| model.result.coefficients[0].value);
      let coefficients = model.result.coefficients[coefficient_offset..]
        .iter()
        .map(|coefficient| coefficient.value)
        .collect::<Vec<_>>();
      (
        model.outcome.clone(),
        model.predictors.clone(),
        intercept,
        coefficients,
      )
    };

    let prediction = LinearPrediction {
      target_variable: &command.target_variable,
      predictor_names: &predictors,
      predictor_coefficients: &coefficients,
      intercept,
      outcome_variable: &outcome,
      residuals: command.kind == PredictKind::Residuals,
    };
    let next_dataset = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_linear_prediction_column(&dataset, &prediction)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next_dataset,
    }))
  }
}

fn affine_lincom_expression(
  expression: &GenerateExpression,
  parameter_indices: &HashMap<String, usize>,
  parameter_count: usize,
) -> Result<AffineCombination, RuntimeError> {
  match expression {
    GenerateExpression::Identifier(name) => {
      let Some(&index) = parameter_indices.get(name) else {
        return Err(RuntimeError::LincomFailed {
          message: format!("variable '{name}' not found in active model coefficients"),
        });
      };
      let mut weights = vec![0.0; parameter_count];
      weights[index] = 1.0;
      Ok(AffineCombination {
        offset: 0.0,
        weights,
      })
    }
    GenerateExpression::Number(text) => {
      let value = text
        .parse::<f64>()
        .map_err(|_| RuntimeError::LincomFailed {
          message: "unsupported expression type in linear testing".to_owned(),
        })?;
      Ok(AffineCombination {
        offset: value,
        weights: vec![0.0; parameter_count],
      })
    }
    GenerateExpression::UnaryMinus(operand) => {
      let mut affine = affine_lincom_expression(operand, parameter_indices, parameter_count)?;
      affine.offset = -affine.offset;
      for weight in &mut affine.weights {
        *weight = -*weight;
      }
      Ok(affine)
    }
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let left_depends = lincom_expression_depends_on_coefficients(left);
      let right_depends = lincom_expression_depends_on_coefficients(right);
      if *operator == GenerateBinaryOperator::Multiply && left_depends && right_depends {
        return Err(RuntimeError::LincomFailed {
          message: "nonlinear coefficient multiplication is not supported in linear testing"
            .to_owned(),
        });
      }
      if *operator == GenerateBinaryOperator::Divide && right_depends {
        return Err(RuntimeError::LincomFailed {
          message: "nonlinear coefficient division is not supported in linear testing".to_owned(),
        });
      }

      let left = affine_lincom_expression(left, parameter_indices, parameter_count)?;
      let right = affine_lincom_expression(right, parameter_indices, parameter_count)?;
      match operator {
        GenerateBinaryOperator::Add => Ok(combine_affine(left, right, false)),
        GenerateBinaryOperator::Subtract => Ok(combine_affine(left, right, true)),
        GenerateBinaryOperator::Multiply => {
          let (mut affine, scale) = if left_depends {
            (left, right.offset)
          } else if right_depends {
            (right, left.offset)
          } else {
            (left, right.offset)
          };
          affine.offset *= scale;
          for weight in &mut affine.weights {
            *weight *= scale;
          }
          Ok(affine)
        }
        GenerateBinaryOperator::Divide => {
          if right.offset == 0.0 {
            return Err(RuntimeError::LincomFailed {
              message: "division by zero in expression".to_owned(),
            });
          }
          let scale = 1.0 / right.offset;
          let mut affine = left;
          affine.offset *= scale;
          for weight in &mut affine.weights {
            *weight *= scale;
          }
          Ok(affine)
        }
        unsupported => Err(RuntimeError::LincomFailed {
          message: format!(
            "unsupported binary operator: {}",
            lincom_operator_text(*unsupported)
          ),
        }),
      }
    }
    GenerateExpression::FunctionCall { .. } => Err(RuntimeError::LincomFailed {
      message: "functions are not supported in linear testing".to_owned(),
    }),
    GenerateExpression::String(_) | GenerateExpression::Null => Err(RuntimeError::LincomFailed {
      message: "unsupported expression type in linear testing".to_owned(),
    }),
  }
}

fn affine_test_expression(
  expression: &GenerateExpression,
  parameter_indices: &HashMap<String, usize>,
  parameter_count: usize,
) -> Result<AffineCombination, String> {
  match affine_lincom_expression(expression, parameter_indices, parameter_count) {
    Ok(affine) => Ok(affine),
    Err(RuntimeError::LincomFailed { message }) => Err(message),
    Err(error) => Err(error.to_string()),
  }
}

fn combine_affine(
  left: AffineCombination,
  right: AffineCombination,
  subtract: bool,
) -> AffineCombination {
  let sign = if subtract { -1.0 } else { 1.0 };
  AffineCombination {
    offset: left.offset + sign * right.offset,
    weights: left
      .weights
      .iter()
      .zip(right.weights.iter())
      .map(|(&left, &right)| left + sign * right)
      .collect(),
  }
}

fn lincom_expression_depends_on_coefficients(expression: &GenerateExpression) -> bool {
  match expression {
    GenerateExpression::Identifier(_) => true,
    GenerateExpression::UnaryMinus(operand) => lincom_expression_depends_on_coefficients(operand),
    GenerateExpression::Binary { left, right, .. } => {
      lincom_expression_depends_on_coefficients(left)
        || lincom_expression_depends_on_coefficients(right)
    }
    GenerateExpression::FunctionCall { arguments, .. } => arguments
      .iter()
      .any(lincom_expression_depends_on_coefficients),
    GenerateExpression::Number(_) | GenerateExpression::String(_) | GenerateExpression::Null => {
      false
    }
  }
}

fn lincom_operator_text(operator: GenerateBinaryOperator) -> &'static str {
  match operator {
    GenerateBinaryOperator::Add => "+",
    GenerateBinaryOperator::Subtract => "-",
    GenerateBinaryOperator::Multiply => "*",
    GenerateBinaryOperator::Divide => "/",
    GenerateBinaryOperator::Equal => "==",
    GenerateBinaryOperator::NotEqual => "!=",
    GenerateBinaryOperator::Less => "<",
    GenerateBinaryOperator::LessOrEqual => "<=",
    GenerateBinaryOperator::Greater => ">",
    GenerateBinaryOperator::GreaterOrEqual => ">=",
  }
}

fn format_lincom_expression(expression: &GenerateExpression) -> String {
  match expression {
    GenerateExpression::Identifier(name) => name.clone(),
    GenerateExpression::Number(text) => format_lincom_number(text),
    GenerateExpression::UnaryMinus(operand) => {
      format!("-({})", format_lincom_expression(operand))
    }
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => format!(
      "({} {} {})",
      format_lincom_expression(left),
      lincom_operator_text(*operator),
      format_lincom_expression(right)
    ),
    GenerateExpression::String(_)
    | GenerateExpression::Null
    | GenerateExpression::FunctionCall { .. } => "<expr>".to_owned(),
  }
}

fn format_test_constraint(expression: &GenerateExpression) -> String {
  match expression {
    GenerateExpression::Binary {
      left,
      operator: GenerateBinaryOperator::Subtract,
      right,
    } => format!(
      "{} = {}",
      format_lincom_expression(left),
      format_lincom_expression(right)
    ),
    _ => format!("{} = 0", format_lincom_expression(expression)),
  }
}

fn format_lincom_number(text: &str) -> String {
  if !text.contains('.') {
    let integer = text.trim_start_matches('0');
    return if integer.is_empty() {
      "0".to_owned()
    } else {
      integer.to_owned()
    };
  }
  let Ok(value) = text.parse::<f64>() else {
    return text.to_owned();
  };
  python_float_literal(value)
}

fn python_float_literal(value: f64) -> String {
  if !value.is_finite() {
    return value.to_string();
  }
  if value == 0.0 {
    return "0.0".to_owned();
  }

  let rendered = value.to_string();
  let (mantissa, exponent) = rendered
    .split_once('e')
    .or_else(|| rendered.split_once('E'))
    .map(|(mantissa, exponent)| (mantissa, exponent.parse::<i32>().unwrap_or(0)))
    .unwrap_or((&rendered, 0));
  let (sign, unsigned_mantissa) = mantissa
    .strip_prefix('-')
    .map(|unsigned| ("-", unsigned))
    .unwrap_or(("", mantissa));
  let decimal_position = unsigned_mantissa
    .find('.')
    .unwrap_or(unsigned_mantissa.len()) as i32;
  let digits = unsigned_mantissa
    .chars()
    .filter(char::is_ascii_digit)
    .collect::<String>();
  let Some(first_nonzero) = digits.bytes().position(|digit| digit != b'0') else {
    return "0.0".to_owned();
  };
  let order = exponent + decimal_position - first_nonzero as i32 - 1;
  let significant = digits[first_nonzero..].trim_end_matches('0');

  // Python keeps this exponent range in fixed notation when formatting float literals.
  if !(-4..16).contains(&order) {
    let mantissa = if significant.len() == 1 {
      significant.to_owned()
    } else {
      format!("{}.{}", &significant[..1], &significant[1..])
    };
    return format!("{sign}{mantissa}e{order:+03}");
  }

  let decimal_position = order + 1;
  let fixed = if decimal_position <= 0 {
    format!(
      "0.{}{significant}",
      "0".repeat((-decimal_position) as usize)
    )
  } else if decimal_position as usize >= significant.len() {
    format!(
      "{}{zeros}.0",
      significant,
      zeros = "0".repeat(decimal_position as usize - significant.len())
    )
  } else {
    let split = decimal_position as usize;
    format!("{}.{}", &significant[..split], &significant[split..])
  };
  format!("{sign}{fixed}")
}

fn cell_to_f64(cell: &CellValue) -> Option<f64> {
  match cell {
    CellValue::Float(f) => Some(*f),
    CellValue::SignedInteger(i) => Some(*i as f64),
    CellValue::UnsignedInteger(u) => Some(*u as f64),
    CellValue::Decimal { value, scale, .. } => {
      let divisor = 10f64.powi(*scale as i32);
      Some(*value as f64 / divisor)
    }
    _ => None,
  }
}

fn cell_to_string(cell: &CellValue) -> Option<String> {
  match cell {
    CellValue::Null => None,
    CellValue::Text(s) => Some(s.clone()),
    CellValue::SignedInteger(i) => Some(i.to_string()),
    CellValue::UnsignedInteger(u) => Some(u.to_string()),
    CellValue::Float(f) => Some(f.to_string()),
    CellValue::Boolean(b) => Some(b.to_string()),
    CellValue::Decimal { value, scale, .. } => {
      let divisor = 10f64.powi(*scale as i32);
      Some(((*value as f64) / divisor).to_string())
    }
    _ => None,
  }
}

fn validate_ttest_numeric(dataset: &DatasetInfo, variables: &[&str]) -> Result<(), RuntimeError> {
  let unknown = variables
    .iter()
    .filter(|variable| {
      !dataset
        .columns
        .iter()
        .any(|column| column.name == **variable)
    })
    .map(|variable| (*variable).to_owned())
    .collect::<Vec<_>>();
  if !unknown.is_empty() {
    return Err(RuntimeError::TtestUnknownVariable { variables: unknown });
  }

  let non_numeric = variables
    .iter()
    .filter(|variable| {
      dataset
        .columns
        .iter()
        .find(|column| column.name == **variable)
        .is_none_or(|column| !is_numeric_data_type(&column.data_type))
    })
    .map(|variable| (*variable).to_owned())
    .collect::<Vec<_>>();
  if !non_numeric.is_empty() {
    return Err(RuntimeError::TtestRequiresNumeric {
      variables: non_numeric,
    });
  }
  Ok(())
}

#[derive(Debug, Clone, Copy)]
enum TtestGroupValueKind {
  Numeric,
  Boolean,
  Blob,
  Temporal,
  Text,
}

fn ttest_group_expression(
  variable: &str,
  data_type: &str,
) -> Result<(String, TtestGroupValueKind), RuntimeError> {
  let quoted = quote_identifier(variable);
  let base_type = data_type
    .trim()
    .to_ascii_uppercase()
    .split('(')
    .next()
    .unwrap_or_default()
    .trim()
    .to_owned();
  if is_numeric_data_type(data_type) {
    return Ok((quoted, TtestGroupValueKind::Numeric));
  }
  if matches!(base_type.as_str(), "BOOLEAN" | "BOOL") {
    return Ok((
      format!("CASE WHEN {quoted} THEN 'True' ELSE 'False' END"),
      TtestGroupValueKind::Boolean,
    ));
  }
  if base_type == "BLOB" {
    return Ok((format!("hex({quoted})"), TtestGroupValueKind::Blob));
  }
  if matches!(
    base_type.as_str(),
    "DATE" | "TIME" | "TIMESTAMP" | "TIMESTAMP_S" | "TIMESTAMP_MS" | "TIMESTAMP_NS"
  ) {
    return Ok((
      format!("CAST({quoted} AS VARCHAR)"),
      TtestGroupValueKind::Temporal,
    ));
  }
  if matches!(
    base_type.as_str(),
    "VARCHAR" | "CHAR" | "BPCHAR" | "STRING" | "TEXT" | "UUID" | "JSON" | "ENUM" | "BIT"
  ) {
    return Ok((
      format!("CAST({quoted} AS VARCHAR)"),
      TtestGroupValueKind::Text,
    ));
  }
  Err(RuntimeError::TtestUnsupportedGroupType {
    variable: variable.to_owned(),
    data_type: data_type.to_owned(),
  })
}

fn ttest_group_stats(
  name: &str,
  statistics: &tabdat_stats::TtestSampleStatistics,
) -> TtestGroupStats {
  TtestGroupStats {
    name: name.to_owned(),
    observations: statistics.observations,
    mean: statistics.mean,
    standard_error: statistics.standard_error,
    standard_deviation: statistics.standard_deviation,
    ci_lower: statistics.ci_lower,
    ci_upper: statistics.ci_upper,
  }
}

fn python_ttest_group_label(value: &CellValue) -> Option<String> {
  match value {
    CellValue::Null => None,
    CellValue::Boolean(value) => Some(if *value { "True" } else { "False" }.to_owned()),
    CellValue::SignedInteger(value) => Some(value.to_string()),
    CellValue::UnsignedInteger(value) => Some(value.to_string()),
    CellValue::Float(value) => Some(python_ttest_float_label(*value)),
    CellValue::Decimal { scale, value, .. } => Some(python_decimal_label(*value, *scale)),
    CellValue::Text(value) => Some(value.clone()),
    CellValue::Bytes(value) => Some(python_bytes_repr(value)),
  }
}

fn python_ttest_float_label(value: f64) -> String {
  if value.is_nan() {
    "nan".to_owned()
  } else if value == f64::INFINITY {
    "inf".to_owned()
  } else if value == f64::NEG_INFINITY {
    "-inf".to_owned()
  } else if value == 0.0 && value.is_sign_negative() {
    "-0.0".to_owned()
  } else {
    python_float_literal(value)
  }
}

fn python_decimal_label(value: i128, scale: u8) -> String {
  let negative = value < 0;
  let digits = value.unsigned_abs().to_string();
  let scale = usize::from(scale);
  let rendered = if scale == 0 {
    digits
  } else if digits.len() <= scale {
    format!("0.{}{digits}", "0".repeat(scale - digits.len()))
  } else {
    let split = digits.len() - scale;
    format!("{}.{}", &digits[..split], &digits[split..])
  };
  if negative {
    format!("-{rendered}")
  } else {
    rendered
  }
}

fn python_temporal_label(value: &str) -> String {
  let Some((prefix, fractional)) = value.rsplit_once('.') else {
    return value.to_owned();
  };
  let precision = fractional.bytes().take_while(u8::is_ascii_digit).count();
  if precision == 0 {
    return value.to_owned();
  }
  let mut micros = fractional[..precision.min(6)].to_owned();
  micros.push_str(&"0".repeat(6 - micros.len()));
  let micros_value = micros.parse::<u32>().unwrap_or(0);
  let suffix = &fractional[precision..];
  if micros_value == 0 {
    format!("{prefix}{suffix}")
  } else {
    format!("{prefix}.{micros}{suffix}")
  }
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
  if !value.len().is_multiple_of(2) {
    return None;
  }
  value
    .as_bytes()
    .chunks_exact(2)
    .map(|pair| {
      let high = (pair[0] as char).to_digit(16)?;
      let low = (pair[1] as char).to_digit(16)?;
      Some(((high << 4) | low) as u8)
    })
    .collect()
}

fn python_bytes_repr(value: &[u8]) -> String {
  let quote = if value.contains(&b'\'') && !value.contains(&b'"') {
    b'"'
  } else {
    b'\''
  };
  let mut rendered = String::from("b");
  rendered.push(char::from(quote));
  for &byte in value {
    match byte {
      b'\\' => rendered.push_str("\\\\"),
      b'\t' => rendered.push_str("\\t"),
      b'\n' => rendered.push_str("\\n"),
      b'\r' => rendered.push_str("\\r"),
      byte if byte == quote => {
        rendered.push('\\');
        rendered.push(char::from(byte));
      }
      0x20..=0x7e => rendered.push(char::from(byte)),
      byte => rendered.push_str(&format!("\\x{byte:02x}")),
    }
  }
  rendered.push(char::from(quote));
  rendered
}
