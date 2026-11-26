use crate::*;

fn maximum_score_error(message: String) -> RuntimeError {
  RuntimeError::MaximumScoreFailed { message }
}

impl Session {
  pub(crate) fn execute_maximum_score(
    &mut self,
    command: &MaximumScoreCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // A failed new fit must not leave an older estimator available for prediction.
    self.estimation = None;
    let regression = RegressCommand {
      outcome: command.outcome.clone(),
      predictors: vec![command.predictor.clone()],
      include_intercept: true,
      estimator: RegressEstimator::Ols,
      weight_variable: None,
      robust: false,
      cluster_variable: None,
    };
    let problem = self
      .prepare_regression_problem(&regression, None, false)
      .map_err(|error| match error {
        RuntimeError::NoActiveDataset { .. } => RuntimeError::NoActiveDataset {
          command: "maxscore",
        },
        other => maximum_score_error(other.to_string()),
      })?;
    let predictor = problem
      .design_matrix
      .iter()
      .map(|row| {
        row
          .first()
          .copied()
          .filter(|value| value.is_finite())
          .ok_or_else(|| maximum_score_error("invalid one-predictor design".into()))
      })
      .collect::<Result<Vec<_>, _>>()?;
    let sample = problem
      .sample
      .clone()
      .ok_or_else(|| maximum_score_error("estimation sample provenance is unavailable".into()))?;
    let result = tabdat_stats::fit_binary_maximum_score(&problem.response, &predictor)
      .map_err(|error| maximum_score_error(error.to_string()))?;
    let fit = MaximumScoreFit {
      outcome: command.outcome.clone(),
      predictor: command.predictor.clone(),
      sample,
      result,
    };
    self.estimation = Some(EstimationState::MaximumScoreFit(Box::new(fit.clone())));
    Ok(ExecutionResult::MaximumScore(Box::new(fit)))
  }

  pub(crate) fn execute_maximum_score_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.kind != PredictKind::MaximumScoreClass
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(maximum_score_error(
        "predict supports only class, without interval/std/saving".into(),
      ));
    }
    let Some(EstimationState::MaximumScoreFit(fit)) = &self.estimation else {
      return Err(maximum_score_error(
        "predict class requires a prior successful maxscore fit".into(),
      ));
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
    let predictor_column = dataset
      .columns
      .iter()
      .find(|column| column.name == fit.predictor)
      .ok_or_else(|| RuntimeError::PredictUnknownVariable {
        variables: vec![fit.predictor.clone()],
      })?;
    if !is_numeric_data_type(&predictor_column.data_type) {
      return Err(maximum_score_error(format!(
        "numeric predictor required: {}",
        fit.predictor
      )));
    }

    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      quote_identifier(&fit.predictor)
    );
    let mut values = Vec::with_capacity(usize::try_from(dataset.row_count).unwrap_or(0));
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let cell = row
            .cell(0)
            .map_err(|_| maximum_score_error("could not read prediction cell".into()))?;
          let value = cell_to_f64(cell).filter(|value| value.is_finite());
          values.push(
            value
              .map(|value| {
                tabdat_stats::maximum_score_predict(&fit.result.selected_rule, value)
                  .map(|class| if class { 1.0 } else { 0.0 })
                  .map_err(|error| maximum_score_error(error.to_string()))
              })
              .transpose()?,
          );
          Ok(())
        },
        maximum_score_error,
      )?;
    if u64::try_from(values.len()).ok() != Some(dataset.row_count) {
      return Err(RuntimeError::PredictFailed);
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
