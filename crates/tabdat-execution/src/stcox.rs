use crate::*;

fn cox_error(message: String) -> RuntimeError {
  RuntimeError::StcoxFailed(message)
}

impl Session {
  pub(crate) fn execute_stcox(
    &mut self,
    command: &StcoxCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Executed fit attempts clear stale inference, not data or labels. Parse errors
    // never reach this boundary and therefore cannot invalidate an existing fit.
    self.estimation = None;
    if command.predictors.len() > tabdat_stats::MAX_COX_PARAMETERS {
      return Err(cox_error("at most64 Cox predictors are supported".into()));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "stcox" })?;
    let names: Vec<&str> = [
      command.time_variable.as_str(),
      command.failure_variable.as_str(),
    ]
    .into_iter()
    .chain(command.predictors.iter().map(String::as_str))
    .collect();
    for name in &names {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| cox_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(cox_error(format!("numeric variable required: {name}")));
      }
    }
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      names
        .iter()
        .map(|s| quote_identifier(s))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut total = 0;
    let mut retained = Vec::new();
    let mut time = Vec::new();
    let mut events = Vec::new();
    let mut design_matrix = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "stcox" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let mut values = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let cell = row
              .cell(j)
              .map_err(|_| cox_error("could not read estimation cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            values.push(value);
          }
          if retained.len() >= tabdat_stats::MAX_COX_OBSERVATIONS {
            return Err(cox_error(
              "at most10000 complete finite estimation rows are supported".into(),
            ));
          }
          // Validate original event identity only AFTER joint finite eligibility:
          // near-one DECIMAL codes can otherwise round to an accepted f64 one.
          let event_cell = row
            .cell(1)
            .map_err(|_| cox_error("could not read failure code".into()))?;
          let event = match exact_integer_code(event_cell) {
            Some(0) => 0.0,
            Some(1) => 1.0,
            _ => {
              return Err(cox_error(
                "retained failure codes must be exactly0 or1".into(),
              ));
            }
          };
          retained.push(index);
          time.push(values[0]);
          events.push(event);
          design_matrix.push(values[2..].to_vec());
          Ok(())
        },
        cox_error,
      )?;
    let sample =
      tabdat_stats::EstimationSample::new(retained, total, total - time.len(), None, None)
        .map_err(|e| cox_error(e.to_string()))?;
    let result = tabdat_stats::fit_cox(
      &tabdat_stats::CoxProblem {
        time_variable: command.time_variable.clone(),
        failure_variable: command.failure_variable.clone(),
        predictor_names: command.predictors.clone(),
        time,
        events,
        design_matrix,
        sample: Some(sample),
      },
      &tabdat_stats::CoxOptions {
        robust: command.robust,
        ..Default::default()
      },
    )
    .map_err(|e| cox_error(e.to_string()))?;
    self.estimation = Some(EstimationState::CoxFit(Box::new(result.clone())));
    Ok(ExecutionResult::Cox(Box::new(result)))
  }

  pub(crate) fn execute_cox_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let kind = match command.kind {
      PredictKind::Xb => tabdat_stats::CoxPredictionKind::Xb,
      PredictKind::CumulativeHazard => tabdat_stats::CoxPredictionKind::CumulativeHazard,
      PredictKind::Survival => tabdat_stats::CoxPredictionKind::Survival,
      _ => return Err(cox_error("Cox prediction supports only xb, cumhaz or survival; Breslow jumps do not identify an instantaneous hazard".into())),
    };
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(cox_error(
        "Cox point prediction does not support auxiliary options or uncertainty intervals".into(),
      ));
    }
    let Some(EstimationState::CoxFit(state)) = &self.estimation else {
      return Err(cox_error(
        "prediction requires a successful stcox fit".into(),
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
    if dataset.row_count > tabdat_stats::MAX_COX_OBSERVATIONS as u64 {
      return Err(cox_error(
        "prediction supports at most10000 current rows".into(),
      ));
    }
    let mut names = state.predictor_names.clone();
    if command.kind != PredictKind::Xb {
      names.push(state.time_variable.clone());
    }
    let missing: Vec<String> = names
      .iter()
      .filter(|n| !dataset.columns.iter().any(|c| c.name == **n))
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
        return Err(cox_error(format!(
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
              .map_err(|_| cox_error("could not read prediction cell".into()))?;
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
          output.push(Some(
            tabdat_stats::cox_predict_row(state, &values, time, kind)
              .map_err(|e| cox_error(e.to_string()))?,
          ));
          Ok(())
        },
        cox_error,
      )?;
    // Score every row before transactional typed-column publication. Any schema,
    // support or numerical error preserves the dataset, labels and owned baseline.
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
