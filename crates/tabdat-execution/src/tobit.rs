use crate::*;

fn tobit_error(message: String) -> RuntimeError {
  RuntimeError::TobitFailed(message)
}

impl Session {
  pub(crate) fn execute_tobit(
    &mut self,
    command: &TobitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Existing fit-family convention: a failed new fit clears stale inference,
    // while fitting itself never changes data, column metadata or labels.
    self.estimation = None;
    if command.upper_limit.is_some() || command.robust || command.cluster_variable.is_some() {
      return Err(tobit_error("only lower-censored IID model-based Gaussian inference is implemented; upper censoring, robust and cluster options are unavailable".into()));
    }
    let lower = command
      .lower_limit
      .parse::<f64>()
      .ok()
      .filter(|v| v.is_finite())
      .ok_or_else(|| tobit_error("ll() must be a finite numeric lower limit".into()))?;
    let parameter_count = command.predictors.len() + usize::from(command.include_intercept);
    if !(1..=32).contains(&parameter_count) {
      return Err(tobit_error(
        "requires1..32 location parameters including any intercept".into(),
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "tobit" })?;
    let variables: Vec<&str> = std::iter::once(command.outcome.as_str())
      .chain(command.predictors.iter().map(String::as_str))
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| tobit_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(tobit_error(format!("numeric variable required: {name}")));
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
    let mut total = 0;
    let mut retained = Vec::new();
    let mut response = Vec::new();
    let mut design = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "tobit" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let mut values = Vec::with_capacity(variables.len());
          for j in 0..variables.len() {
            let cell = row
              .cell(j)
              .map_err(|_| tobit_error("could not read numeric estimation cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            values.push(value);
          }
          if retained.len() >= 10_000 {
            return Err(tobit_error(
              "at most10000 complete finite estimation rows are supported".into(),
            ));
          }
          retained.push(index);
          response.push(values[0]);
          design.push(values[1..].to_vec());
          Ok(())
        },
        tobit_error,
      )?;
    if retained.is_empty() {
      return Err(tobit_error("no complete finite observations".into()));
    }
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, dropped, None, None)
      .map_err(|e| tobit_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design,
    )
    .map_err(|e| tobit_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result = tabdat_stats::fit_tobit(&problem, lower, &tabdat_stats::TobitOptions::default())
      .map_err(|e| tobit_error(e.to_string()))?;
    self.estimation = Some(EstimationState::TobitFit(Box::new(result.clone())));
    Ok(ExecutionResult::Tobit(Box::new(result)))
  }

  pub(crate) fn execute_tobit_margins(
    &self,
    state: &tabdat_stats::TobitResult,
  ) -> Result<ExecutionResult, RuntimeError> {
    tabdat_stats::tobit_average_marginal_effects(state)
      .map(|r| ExecutionResult::TobitMarginalEffects(Box::new(r)))
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })
  }

  pub(crate) fn execute_tobit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(
      command.kind,
      PredictKind::Xb | PredictKind::Pr | PredictKind::TobitMean
    ) || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(tobit_error("prediction supports only xb, pr (uncensoring probability) or mean (observed censored mean), without auxiliary options".into()));
    }
    let Some(EstimationState::TobitFit(state)) = &self.estimation else {
      return Err(tobit_error(
        "predict mean requires a prior successful lower-censored Gaussian Tobit fit".into(),
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
    if dataset.row_count > 10_000 {
      return Err(tobit_error(
        "prediction supports at most10000 current rows".into(),
      ));
    }
    let names = &state.predictor_names;
    let missing: Vec<_> = names
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in names {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(tobit_error(format!(
          "prediction requires numeric predictor: {name}"
        )));
      }
    }
    let select = if names.is_empty() {
      "1".into()
    } else {
      names
        .iter()
        .map(|name| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ")
    };
    let query = format!("SELECT {select} FROM {ACTIVE_TABLE}");
    let mut values = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let mut predictors = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let cell = row
              .cell(j)
              .map_err(|_| tobit_error("could not read prediction cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            predictors.push(value);
          }
          let (latent, probability, mean) = tabdat_stats::tobit_predict_row(state, &predictors)
            .map_err(|e| tobit_error(e.to_string()))?;
          values.push(Some(match command.kind {
            PredictKind::Pr => probability,
            PredictKind::TobitMean => mean,
            _ => latent,
          }));
          Ok(())
        },
        tobit_error,
      )?;
    // Scoring finishes before the existing typed staging adapter publishes any
    // column. Numerical/schema/target failures preserve data and the owned fit.
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
