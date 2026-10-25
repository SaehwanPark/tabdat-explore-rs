use crate::*;

fn streg_error(message: String) -> RuntimeError {
  RuntimeError::StregFailed(message)
}

impl Session {
  pub(crate) fn execute_streg(
    &mut self,
    command: &StregCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // A new fit attempt invalidates stale inference, never its source data/labels.
    self.estimation = None;
    if command.distribution != StregDistribution::Weibull || command.cluster_variable.is_some() {
      return Err(streg_error("only entry-zero single-spell Weibull PH with model-based or independent-spell robust inference is implemented; native exponential and cluster are unavailable".into()));
    }
    let p = command.predictors.len() + usize::from(command.include_intercept) + 1;
    if p > tabdat_stats::MAX_WEIBULL_PARAMETERS {
      return Err(streg_error(
        "at most64 joint parameters including shape and any intercept are supported".into(),
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "streg" })?;
    let variables: Vec<&str> = [
      command.time_variable.as_str(),
      command.failure_variable.as_str(),
    ]
    .into_iter()
    .chain(command.predictors.iter().map(String::as_str))
    .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| streg_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(streg_error(format!("numeric variable required: {name}")));
      }
    }
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      variables
        .iter()
        .map(|s| quote_identifier(s))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut total = 0;
    let mut retained = Vec::new();
    let mut time = Vec::new();
    let mut events = Vec::new();
    let mut design = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "streg" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let mut values = Vec::with_capacity(variables.len());
          for j in 0..variables.len() {
            let cell = row
              .cell(j)
              .map_err(|_| streg_error("could not read numeric estimation cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            values.push(value);
          }
          if retained.len() >= tabdat_stats::MAX_WEIBULL_OBSERVATIONS {
            return Err(streg_error(
              "at most10000 complete finite estimation rows are supported".into(),
            ));
          }
          // Support validation belongs to the JOINT finite sample, not a separate
          // univariate scan that changes eligibility depending on column order.
          retained.push(index);
          time.push(values[0]);
          events.push(values[1]);
          design.push(values[2..].to_vec());
          Ok(())
        },
        streg_error,
      )?;
    if retained.is_empty() {
      return Err(streg_error("no complete finite observations".into()));
    }
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, dropped, None, None)
      .map_err(|e| streg_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.time_variable.clone(),
      command.predictors.clone(),
      time,
      design,
    )
    .map_err(|e| streg_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result = tabdat_stats::fit_weibull(
      &problem,
      &command.failure_variable,
      &events,
      &tabdat_stats::WeibullOptions {
        robust: command.robust,
        ..Default::default()
      },
    )
    .map_err(|e| streg_error(e.to_string()))?;
    self.estimation = Some(EstimationState::WeibullFit(Box::new(result.clone())));
    Ok(ExecutionResult::Weibull(Box::new(result)))
  }

  pub(crate) fn execute_weibull_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let kind = match command.kind {
      PredictKind::Xb => tabdat_stats::WeibullPredictionKind::Xb,
      PredictKind::Hazard => tabdat_stats::WeibullPredictionKind::Hazard,
      PredictKind::CumulativeHazard => tabdat_stats::WeibullPredictionKind::CumulativeHazard,
      PredictKind::Survival => tabdat_stats::WeibullPredictionKind::Survival,
      _ => return Err(streg_error("prediction supports only xb (PH index), hazard, cumhaz or survival; no count/mean/residual/effect interpretation".into())),
    };
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(streg_error(
        "Weibull point prediction does not support auxiliary options or uncertainty intervals"
          .into(),
      ));
    }
    let Some(EstimationState::WeibullFit(state)) = &self.estimation else {
      return Err(streg_error(
        "prediction requires a successful Weibull streg fit".into(),
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
      .any(|c| c.name == command.target_variable)
    {
      return Err(RuntimeError::PredictTargetExists {
        variable: command.target_variable.clone(),
      });
    }
    if dataset.row_count > tabdat_stats::MAX_WEIBULL_OBSERVATIONS as u64 {
      return Err(streg_error(
        "prediction supports at most10000 current rows".into(),
      ));
    }
    let mut names = state.predictor_names.clone();
    if command.kind != PredictKind::Xb {
      names.push(state.time_variable.clone());
    }
    let missing: Vec<String> = names
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in &names {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(streg_error(format!(
          "prediction requires numeric variable: {name}"
        )));
      }
    }
    let select = if names.is_empty() {
      "1".into()
    } else {
      names
        .iter()
        .map(|s| quote_identifier(s))
        .collect::<Vec<_>>()
        .join(", ")
    };
    let query = format!("SELECT {select} FROM {ACTIVE_TABLE}");
    let mut output = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let mut values = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let cell = row
              .cell(j)
              .map_err(|_| streg_error("could not read prediction cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              output.push(None);
              return Ok(());
            };
            values.push(value);
          }
          let time = if command.kind == PredictKind::Xb {
            None
          } else {
            values.pop()
          };
          let value = tabdat_stats::weibull_predict_row(state, &values, time, kind)
            .map_err(|e| streg_error(e.to_string()))?;
          output.push(Some(value));
          Ok(())
        },
        streg_error,
      )?;
    // Entire current-data scoring succeeds before typed transactional publication.
    // Invalid schema/support/overflow preserves data, labels AND the owned fit.
    let next = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_numeric_prediction_column(&dataset, &command.target_variable, &output)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let dataset = self.sync_active_dataset(next)?;
    Ok(ExecutionResult::Prediction(PredictionResult { dataset }))
  }
}
