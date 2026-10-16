use crate::*;

fn logit_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::LogitFailed {
    message: message.into(),
  }
}

impl Session {
  pub(crate) fn execute_logit(
    &mut self,
    command: &LogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Match the supported linear/IV policy: a failed new fit clears stale inference.
    self.estimation = None;
    if command.robust || command.cluster_variable.is_some() {
      return Err(logit_error(
        "robust/cluster logit covariance is not implemented; use an unweighted model-based fit",
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "logit" })?;
    let variables: Vec<_> = std::iter::once(&command.outcome)
      .chain(&command.predictors)
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| logit_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(logit_error(format!("numeric variable required: {name}")));
      }
    }
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      variables
        .iter()
        .map(|name| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut total_rows = 0;
    let mut retained = Vec::new();
    let mut response = Vec::new();
    let mut design = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "logit" })?
      .visit_rows(
        &query,
        |row| {
          let index = total_rows;
          total_rows += 1;
          let mut values = Vec::with_capacity(variables.len());
          for j in 0..variables.len() {
            let cell = row.cell(j).map_err(|error| {
              logit_error(match error {
                CellReadError::Backend(message) => message,
                _ => "could not read logit numeric cell".into(),
              })
            })?;
            match cell_to_f64(cell) {
              Some(v) if v.is_finite() => values.push(v),
              _ => return Ok(()),
            }
          }
          retained.push(index);
          response.push(values[0]);
          design.push(values[1..].to_vec());
          Ok(())
        },
        logit_error,
      )?;
    if retained.is_empty() {
      return Err(logit_error("no complete finite observations"));
    }
    let dropped = total_rows - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total_rows, dropped, None, None)
      .map_err(|e| logit_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design,
    )
    .map_err(|e| logit_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result =
      tabdat_stats::fit_binary_logit(&problem).map_err(|e| logit_error(e.to_string()))?;
    self.estimation = Some(EstimationState::Logit(Box::new(BinaryLogitState {
      problem,
      result: result.clone(),
    })));
    Ok(ExecutionResult::Logit(Box::new(result)))
  }

  pub(crate) fn execute_logit_margins(&self) -> Result<ExecutionResult, RuntimeError> {
    let Some(EstimationState::Logit(state)) = &self.estimation else {
      return Err(RuntimeError::EstatMarginsRequiresPriorLogit);
    };
    let result = tabdat_stats::logit_average_marginal_effects(&state.problem, &state.result)
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })?;
    Ok(ExecutionResult::LogitMarginalEffects(result))
  }

  pub(crate) fn execute_logit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(command.kind, PredictKind::Pr | PredictKind::Xb)
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(RuntimeError::PredictLogitUnsupported);
    }
    let Some(EstimationState::Logit(state)) = &self.estimation else {
      return Err(RuntimeError::PredictRequiresPriorRegression);
    };
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "predict" })?
      .clone();
    if dataset
      .columns
      .iter()
      .any(|c| c.name == command.target_variable)
    {
      return Err(RuntimeError::PredictTargetExists {
        variable: command.target_variable.clone(),
      });
    }
    let predictors = &state.problem.predictor_names;
    let missing: Vec<_> = predictors
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in predictors {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(logit_error(format!(
          "prediction requires numeric predictor: {name}"
        )));
      }
    }
    let offset = usize::from(state.problem.include_intercept);
    let coefficients: Vec<_> = state.result.coefficients[offset..]
      .iter()
      .map(|c| c.value)
      .collect();
    let prediction = LinearPrediction {
      target_variable: &command.target_variable,
      predictor_names: predictors,
      predictor_coefficients: &coefficients,
      intercept: state
        .problem
        .include_intercept
        .then(|| state.result.coefficients[0].value),
      outcome_variable: &state.problem.outcome,
      residuals: false,
    };
    let next = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_logit_prediction_column(&dataset, &prediction, command.kind == PredictKind::Pr)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next = self.sync_active_dataset(next)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }
}
