use crate::*;

fn fit_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::RandomEffectsFailed {
    message: message.into(),
  }
}
fn prediction_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::RandomEffectsPredictionFailed {
    message: message.into(),
  }
}

impl Session {
  pub(crate) fn execute_random_effects(
    &mut self,
    command: &XtRegCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let input = self.prepare_panel_problem(command)?;
    let fit = if command.robust || command.cluster_variable.is_some() {
      tabdat_stats::fit_random_effects_clustered
    } else {
      tabdat_stats::fit_random_effects
    };
    let result =
      fit(&input.problem, &input.entity).map_err(|error| fit_error(error.to_string()))?;
    let state = RandomEffectsState {
      time_variable: input.time,
      entity_data_type: input.entity_data_type,
      result,
    };
    self.estimation = Some(EstimationState::RandomEffects(Box::new(state.clone())));
    Ok(ExecutionResult::RandomEffects(Box::new(state)))
  }

  pub(crate) fn execute_random_effects_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.interval
      || command.std
      || command.saving.is_some()
      || !matches!(command.kind, PredictKind::Xb | PredictKind::Residuals)
    {
      return Err(prediction_error(
        "RE-GLS supports population xb or residuals only; no conditional unit effects, intervals, std or saved draws",
      ));
    }
    let Some(EstimationState::RandomEffects(model)) = &self.estimation else {
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
    let residual = command.kind == PredictKind::Residuals;
    let mut required = model.result.predictor_names.clone();
    if residual {
      required.push(model.result.outcome.clone());
    }
    let types: HashMap<_, _> = dataset
      .columns
      .iter()
      .map(|c| (c.name.as_str(), c.data_type.as_str()))
      .collect();
    let missing: Vec<_> = required
      .iter()
      .filter(|n| !types.contains_key(n.as_str()))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    if required
      .iter()
      .any(|n| !is_numeric_data_type(types[n.as_str()]))
    {
      return Err(prediction_error(
        "current outcome/predictors must be numeric",
      ));
    }
    let beta: Vec<_> = model
      .result
      .gls
      .coefficients
      .iter()
      .map(|c| c.value)
      .collect();
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      required
        .iter()
        .map(|n| quote_identifier(n))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut values = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let mut xb = beta[0];
          for (j, b) in beta[1..].iter().enumerate() {
            let Some(x) = cell_to_f64(
              row
                .cell(j)
                .map_err(|_| prediction_error("could not read predictor"))?,
            )
            .filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            xb += b * x;
          }
          let value = if residual {
            cell_to_f64(
              row
                .cell(beta.len() - 1)
                .map_err(|_| prediction_error("could not read outcome"))?,
            )
            .map(|y| y - xb)
          } else {
            Some(xb)
          };
          values.push(value.filter(|v| v.is_finite()));
          Ok(())
        },
        prediction_error,
      )?;
    let next = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_numeric_prediction_column(&dataset, &command.target_variable, &values)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let dataset = self.sync_active_dataset(next)?;
    Ok(ExecutionResult::Prediction(PredictionResult { dataset }))
  }
}
