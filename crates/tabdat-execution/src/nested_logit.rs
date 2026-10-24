use crate::*;

fn nlogit_error(message: String) -> RuntimeError {
  RuntimeError::NlogitFailed { message }
}
fn normalized_common_error(message: String) -> RuntimeError {
  RuntimeError::NlogitCommonFailed(message)
}
fn choice_error(error: RuntimeError, make_error: fn(String) -> RuntimeError) -> RuntimeError {
  match error {
    RuntimeError::ClogitFailed { message } => make_error(message),
    RuntimeError::NoActiveDataset { .. } => RuntimeError::NoActiveDataset { command: "nlogit" },
    other => other,
  }
}

impl Session {
  fn nlogit_problem(
    &self,
    command: &NlogitCommand,
    common: bool,
  ) -> Result<tabdat_stats::NestedChoiceProblem, RuntimeError> {
    let make_error = if common {
      normalized_common_error
    } else {
      nlogit_error
    };
    let lower = command.predictors.len();
    let upper = command.upper_predictors.len();
    let scales = if common { 1 } else { 2 };
    if lower == 0 || lower > 16 || upper > 16 || lower + upper + scales > 32 {
      return Err(make_error(format!(
        "requires1..16 lower and0..16 upper variables, with at most32 parameters including {scales} nest scale(s)"
      )));
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
      .map_err(|e| choice_error(e, make_error))?;
    let choice = frame.retained_problem(
      &command.outcome,
      &command.group_variable,
      &command.alternative_variable,
      &predictors,
      make_error,
    )?;
    // Structural nest keys are audited before missingness; retain original raw
    // rows for both likelihoods and let the statistical core choose its scaling.
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
    Ok(tabdat_stats::NestedChoiceProblem {
      choice,
      lower_predictor_count: lower,
      nest_variable: command.nest_variable.clone(),
      nest_codes,
    })
  }

  pub(crate) fn execute_nlogit(
    &mut self,
    command: &NlogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let problem = self.nlogit_problem(command, false)?;
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

  pub(crate) fn execute_nlogit_normalized_common(
    &mut self,
    command: &NlogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let problem = self.nlogit_problem(command, true)?;
    let result = tabdat_stats::fit_normalized_common_nested_logit(&problem, &Default::default())
      .map_err(|e| normalized_common_error(e.to_string()))?;
    self.estimation = Some(EstimationState::NormalizedCommonNestedLogit(Box::new(
      NormalizedCommonNestedLogitState {
        problem,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::NormalizedCommonNestedLogit(Box::new(
      result,
    )))
  }

  pub(crate) fn execute_nlogit_prediction(
    &mut self,
    command: &PredictCommand,
    common: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let make_error = if common {
      normalized_common_error
    } else {
      nlogit_error
    };
    if command.kind != PredictKind::Pr
      || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
    {
      return Err(make_error(if common {
        "normalized common-scale nested choice supports only predict newvar, pr on complete current choice sets".into()
      } else {
        "non-normalized nested choice supports only predict newvar, pr on complete current choice sets".into()
      }));
    }
    let state = self.estimation.as_ref();
    let (group, alternative, nest, predictors, levels, alternative_nests) = match state {
      Some(EstimationState::NonNormalizedNestedLogit(s)) if !common => (
        &s.result.group_variable,
        &s.result.alternative_variable,
        &s.result.nest_variable,
        &s.result.predictor_names,
        &s.result.alternative_levels,
        &s.result.alternative_nests,
      ),
      Some(EstimationState::NormalizedCommonNestedLogit(s)) if common => (
        &s.result.group_variable,
        &s.result.alternative_variable,
        &s.result.nest_variable,
        &s.result.predictor_names,
        &s.result.alternative_levels,
        &s.result.alternative_nests,
      ),
      _ => return Err(RuntimeError::PredictRequiresPriorRegression),
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
    let frame = self
      .read_choice_frame(group, alternative, predictors, None, Some(nest))
      .map_err(|e| choice_error(e, make_error))?;
    if frame.levels != *levels || frame.alternative_nests != *alternative_nests {
      return Err(make_error(
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
      let probabilities = match state {
        Some(EstimationState::NonNormalizedNestedLogit(s)) => {
          tabdat_stats::non_normalized_nested_predict_set(&s.result, &x, &codes, &nests)
        }
        Some(EstimationState::NormalizedCommonNestedLogit(s)) => {
          tabdat_stats::normalized_common_nested_predict_set(&s.result, &x, &codes, &nests)
        }
        _ => unreachable!("nested prediction state validated"),
      }
      .map_err(|e| make_error(e.to_string()))?;
      for (&i, probability) in case.iter().zip(probabilities) {
        values[i] = Some(probability);
      }
    }
    // Publish only after every complete set's prediction has succeeded.
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
