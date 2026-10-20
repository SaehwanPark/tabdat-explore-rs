use crate::*;

fn fit_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::FixedEffectsFailed {
    message: message.into(),
  }
}
fn prediction_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::FixedEffectsPredictionFailed {
    message: message.into(),
  }
}
fn integer_time(cell: &CellValue) -> bool {
  match cell {
    CellValue::SignedInteger(_) | CellValue::UnsignedInteger(_) => true,
    CellValue::Float(value) => {
      value.is_finite() && value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_991.0
    }
    CellValue::Decimal { value, scale, .. } => 10_i128
      .checked_pow(u32::from(*scale))
      .is_some_and(|divisor| value % divisor == 0),
    _ => false,
  }
}

fn panel_fit_error(command: &XtRegCommand, message: impl Into<String>) -> RuntimeError {
  if command.estimator == XtRegEstimator::FixedEffects {
    fit_error(message)
  } else {
    RuntimeError::RandomEffectsFailed {
      message: message.into(),
    }
  }
}

pub(crate) struct PanelFitInput {
  pub problem: tabdat_stats::EstimationProblem,
  pub entity: String,
  pub time: String,
  pub entity_data_type: String,
}

impl Session {
  pub(crate) fn prepare_panel_problem(
    &mut self,
    command: &XtRegCommand,
  ) -> Result<PanelFitInput, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "xtreg" })?
      .clone();
    let (Some(entity), Some(time)) = (&command.id_variable, &command.time_variable) else {
      return Err(panel_fit_error(
        command,
        "panel fit requires explicit id(variable) and time(variable)",
      ));
    };
    if entity == time {
      return Err(panel_fit_error(command, "id() and time() must be distinct"));
    }
    if command
      .cluster_variable
      .as_ref()
      .is_some_and(|name| name != entity)
    {
      return Err(panel_fit_error(
        command,
        format!("panel fit supports only cluster({entity}); robust also clusters on {entity}"),
      ));
    }
    let mut required = vec![entity.clone(), time.clone(), command.outcome.clone()];
    required.extend(command.predictors.iter().cloned());
    let types: HashMap<_, _> = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect();
    let missing: Vec<_> = required
      .iter()
      .filter(|name| !types.contains_key(name.as_str()))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(panel_fit_error(
        command,
        format!("unknown variables: {}", missing.join(", ")),
      ));
    }
    let nonnumeric: Vec<_> = required[1..]
      .iter()
      .filter(|name| !is_numeric_data_type(types[name.as_str()]))
      .cloned()
      .collect();
    if !nonnumeric.is_empty() {
      return Err(panel_fit_error(
        command,
        format!(
          "numeric time/outcome/predictors required: {}",
          nonnumeric.join(", ")
        ),
      ));
    }
    // Audit all raw keys, even rows subsequently omitted for missing model values.
    self.execute_isid(vec![entity.clone(), time.clone()], false)?;
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      required
        .iter()
        .map(|name| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| panel_fit_error(command, "data backend unavailable"))?;
    let mut total = 0;
    let mut retained = Vec::new();
    let mut groups = Vec::new();
    let mut y = Vec::new();
    let mut x = Vec::new();
    backend.visit_rows(
      &query,
      |row| {
        let source = total;
        total += 1;
        let entity_cell = row
          .cell(0)
          .map_err(|_| panel_fit_error(command, "could not read entity key"))?;
        let key = cluster_key(entity_cell)
          .map_err(|message| panel_fit_error(command, message))?
          .ok_or_else(|| panel_fit_error(command, "entity keys must be nonmissing and finite"))?;
        if !integer_time(
          row
            .cell(1)
            .map_err(|_| panel_fit_error(command, "could not read time key"))?,
        ) {
          return Err(panel_fit_error(
            command,
            "time keys must be exact finite integers (DOUBLE within ±(2^53-1))",
          ));
        }
        let value = cell_to_f64(
          row
            .cell(2)
            .map_err(|_| panel_fit_error(command, "could not read outcome"))?,
        )
        .filter(|v| v.is_finite());
        let mut predictors = Vec::with_capacity(command.predictors.len());
        for i in 0..command.predictors.len() {
          let Some(value) = cell_to_f64(
            row
              .cell(i + 3)
              .map_err(|_| panel_fit_error(command, "could not read predictor"))?,
          )
          .filter(|v| v.is_finite()) else {
            return Ok(());
          };
          predictors.push(value);
        }
        if let Some(value) = value {
          retained.push(source);
          groups.push(key);
          y.push(value);
          x.push(predictors);
        }
        Ok(())
      },
      |message| panel_fit_error(command, message),
    )?;
    let sample =
      tabdat_stats::EstimationSample::new(retained, total, total - y.len(), None, Some(groups))
        .map_err(|error| panel_fit_error(command, error.to_string()))?;
    let problem =
      tabdat_stats::EstimationProblem::new(&command.outcome, command.predictors.clone(), y, x)
        .map_err(|error| panel_fit_error(command, error.to_string()))?
        .with_sample(sample);
    Ok(PanelFitInput {
      problem,
      entity: entity.clone(),
      time: time.clone(),
      entity_data_type: types[entity.as_str()].into(),
    })
  }

  pub(crate) fn execute_fixed_effects(
    &mut self,
    command: &XtRegCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let input = self.prepare_panel_problem(command)?;
    let PanelFitInput {
      problem,
      entity,
      time,
      entity_data_type,
    } = input;
    let result = tabdat_stats::fit_fixed_effects(
      &problem,
      &entity,
      command.robust || command.cluster_variable.is_some(),
    )
    .map_err(|error| fit_error(error.to_string()))?;
    let state = FixedEffectsState {
      time_variable: time,
      entity_data_type,
      result,
    };
    self.estimation = Some(EstimationState::FixedEffects(Box::new(state.clone())));
    Ok(ExecutionResult::FixedEffects(Box::new(state)))
  }

  pub(crate) fn execute_fixed_effects_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.interval || command.std || command.saving.is_some() {
      return Err(prediction_error(
        "intervals, std and saved draws are not implemented",
      ));
    }
    if !matches!(
      command.kind,
      PredictKind::Xb
        | PredictKind::FixedEffectsResponse
        | PredictKind::FixedEffect
        | PredictKind::Residuals
    ) {
      return Err(prediction_error("use xb, xbu, u or residuals"));
    }
    let Some(EstimationState::FixedEffects(model)) = &self.estimation else {
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
      .any(|column| column.name == command.target_variable)
    {
      return Err(RuntimeError::PredictTargetExists {
        variable: command.target_variable.clone(),
      });
    }
    let conditional = command.kind != PredictKind::Xb;
    let uses_predictors = command.kind != PredictKind::FixedEffect;
    let residual = command.kind == PredictKind::Residuals;
    let mut required = Vec::new();
    let entity_index = conditional.then(|| {
      required.push(model.result.entity_variable.clone());
      0
    });
    let x_start = required.len();
    if uses_predictors {
      required.extend(model.result.within.parameter_names.iter().cloned());
    }
    let outcome_index = residual.then(|| {
      let index = required.len();
      required.push(model.result.outcome.clone());
      index
    });
    let types: HashMap<_, _> = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect();
    let missing: Vec<_> = required
      .iter()
      .filter(|name| !types.contains_key(name.as_str()))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    if conditional && types[model.result.entity_variable.as_str()] != model.entity_data_type {
      return Err(prediction_error(
        "current entity type must match the fitted entity type",
      ));
    }
    for name in &required[usize::from(conditional)..] {
      if !is_numeric_data_type(types[name.as_str()]) {
        return Err(prediction_error(format!(
          "current model variable {name} must be numeric"
        )));
      }
    }
    let effects: BTreeMap<_, _> = model
      .result
      .effects
      .iter()
      .map(|effect| (effect.key.as_str(), effect))
      .collect();
    let beta: Vec<_> = model
      .result
      .within
      .coefficients
      .iter()
      .map(|coefficient| coefficient.value)
      .collect();
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      required
        .iter()
        .map(|name| quote_identifier(name))
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
          let effect = if let Some(index) = entity_index {
            let key = cluster_key(
              row
                .cell(index)
                .map_err(|_| prediction_error("could not read entity key"))?,
            )
            .map_err(prediction_error)?;
            let Some(effect) = key
              .as_ref()
              .and_then(|key| effects.get(key.as_str()))
              .copied()
            else {
              values.push(None);
              return Ok(());
            };
            Some(effect)
          } else {
            None
          };
          if command.kind == PredictKind::FixedEffect {
            values.push(effect.map(|effect| effect.effect));
            return Ok(());
          }
          let mut index = 0.0;
          for (i, b) in beta.iter().enumerate() {
            let Some(x) = cell_to_f64(
              row
                .cell(x_start + i)
                .map_err(|_| prediction_error("could not read predictor"))?,
            )
            .filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            index += b * (x - effect.map_or(0.0, |effect| effect.predictor_means[i]));
          }
          let value = if let Some(outcome_index) = outcome_index {
            cell_to_f64(
              row
                .cell(outcome_index)
                .map_err(|_| prediction_error("could not read outcome"))?,
            )
            .map(|y| (y - effect.unwrap().response_mean) - index)
          } else {
            Some(
              index + effect.map_or(model.result.common_intercept, |effect| effect.response_mean),
            )
          };
          values.push(value.filter(|value| value.is_finite()));
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
