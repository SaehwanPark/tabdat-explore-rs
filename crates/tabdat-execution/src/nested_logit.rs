use crate::*;

fn nlogit_error(message: String) -> RuntimeError {
  RuntimeError::NlogitFailed { message }
}
fn choice_error(error: RuntimeError) -> RuntimeError {
  match error {
    RuntimeError::ClogitFailed { message } => nlogit_error(message),
    RuntimeError::NoActiveDataset { .. } => RuntimeError::NoActiveDataset { command: "nlogit" },
    other => other,
  }
}

impl Session {
  pub(crate) fn execute_nlogit(
    &mut self,
    command: &NlogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let lower = command.predictors.len();
    let upper = command.upper_predictors.len();
    if lower == 0 || lower > 16 || upper > 16 || lower + upper + 2 > 32 {
      return Err(nlogit_error("requires1..16 lower and0..16 upper variables, with at most32 parameters including two nest scales".into()));
    }
    let predictors = command
      .predictors
      .iter()
      .chain(&command.upper_predictors)
      .cloned()
      .collect::<Vec<_>>();
    let frame = self
      .read_choice_frame(
        &command.group_variable,
        &command.alternative_variable,
        &predictors,
        Some(&command.outcome),
        Some(&command.nest_variable),
      )
      .map_err(choice_error)?;
    let choice = frame.retained_problem(
      &command.outcome,
      &command.group_variable,
      &command.alternative_variable,
      &predictors,
      nlogit_error,
    )?;
    // The shared sample builder preserves raw row order and selects whole sets.
    // Nest keys were audited before missingness, not converted through f64.
    let retained = &choice
      .design
      .sample
      .as_ref()
      .expect("choice provenance attached")
      .retained_indices;
    let nest_codes = retained
      .iter()
      .map(|&i| frame.rows[i].nest.expect("nest key audited"))
      .collect();
    let problem = tabdat_stats::NonNormalizedNestedLogitProblem {
      choice,
      lower_predictor_count: lower,
      nest_variable: command.nest_variable.clone(),
      nest_codes,
    };
    let result = tabdat_stats::fit_non_normalized_nested_logit(&problem, &Default::default())
      .map_err(|e| nlogit_error(e.to_string()))?;
    self.estimation = Some(EstimationState::NonNormalizedNestedLogit(Box::new(
      NonNormalizedNestedLogitState {
        problem,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::NonNormalizedNestedLogit(Box::new(result)))
  }

  pub(crate) fn execute_nlogit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.kind != PredictKind::Pr
      || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
    {
      return Err(nlogit_error("non-normalized nested choice supports only predict newvar, pr on complete current choice sets".into()));
    }
    let Some(EstimationState::NonNormalizedNestedLogit(state)) = &self.estimation else {
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
    let result = &state.result;
    let frame = self
      .read_choice_frame(
        &result.group_variable,
        &result.alternative_variable,
        &result.predictor_names,
        None,
        Some(&result.nest_variable),
      )
      .map_err(choice_error)?;
    if frame.levels != result.alternative_levels
      || frame.alternative_nests != result.alternative_nests
    {
      return Err(nlogit_error(
        "prediction requires the complete retained alternative/nest map".into(),
      ));
    }
    let mut values = vec![None; frame.rows.len()];
    for case in &frame.cases {
      if case.iter().any(|&i| frame.rows[i].predictors.is_none()) {
        continue;
      }
      let x = case
        .iter()
        .map(|&i| frame.rows[i].predictors.clone().unwrap())
        .collect::<Vec<_>>();
      let codes = case
        .iter()
        .map(|&i| frame.rows[i].alternative)
        .collect::<Vec<_>>();
      let nests = case
        .iter()
        .map(|&i| frame.rows[i].nest.unwrap())
        .collect::<Vec<_>>();
      let probabilities =
        tabdat_stats::non_normalized_nested_predict_set(result, &x, &codes, &nests)
          .map_err(|e| nlogit_error(e.to_string()))?;
      for (&i, probability) in case.iter().zip(probabilities) {
        values[i] = Some(probability);
      }
    }
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
