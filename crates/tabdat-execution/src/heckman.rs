use crate::row_conversion::exact_integer_code;
use crate::*;

fn heckman_error(message: String) -> RuntimeError {
  RuntimeError::HeckmanFailed(message)
}

impl Session {
  pub(crate) fn execute_heckman(
    &mut self,
    command: &HeckmanCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    if !command.twostep || command.robust || command.cluster_variable.is_some() {
      return Err(heckman_error("only explicit twostep with IID model-based corrected covariance is implemented; default ML, robust and cluster inference are unavailable".into()));
    }
    let p = command.predictors.len() + usize::from(command.include_intercept) + 1;
    let q = command.selection_predictors.len() + usize::from(command.include_intercept);
    if p > tabdat_stats::MAX_HECKMAN_PARAMETERS
      || !(1..=tabdat_stats::MAX_HECKMAN_PARAMETERS).contains(&q)
    {
      return Err(heckman_error(
        "requires1..32 parameters per stage, including any intercept and outcome Mills coefficient"
          .into(),
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "heckman" })?;
    let mut names = Vec::new();
    for name in std::iter::once(&command.selection_dependent)
      .chain(&command.selection_predictors)
      .chain(std::iter::once(&command.outcome))
      .chain(&command.predictors)
    {
      if !names.contains(name) {
        names.push(name.clone());
      }
    }
    for name in &names {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| heckman_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(heckman_error(format!("numeric variable required: {name}")));
      }
    }
    let position = |name: &String| {
      names
        .iter()
        .position(|n| n == name)
        .expect("union contains variable")
    };
    let d_position = position(&command.selection_dependent);
    let y_position = position(&command.outcome);
    let z_positions: Vec<_> = command.selection_predictors.iter().map(position).collect();
    let x_positions: Vec<_> = command.predictors.iter().map(position).collect();
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      names
        .iter()
        .map(|n| quote_identifier(n))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut total = 0;
    let mut eligible = Vec::new();
    let mut selected = Vec::new();
    let mut d = Vec::new();
    let mut z = Vec::new();
    let mut y = Vec::new();
    let mut x = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "heckman" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let cell = row
            .cell(d_position)
            .map_err(|_| heckman_error("could not read selection cell".into()))?;
          if cell_to_f64(cell).filter(|v| v.is_finite()).is_none() {
            return Ok(());
          }
          let mut zi = Vec::with_capacity(z_positions.len());
          for &j in &z_positions {
            let c = row
              .cell(j)
              .map_err(|_| heckman_error("could not read selection regressor".into()))?;
            let Some(v) = cell_to_f64(c).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            zi.push(v);
          }
          // Exact DECIMAL/integer identity is checked before any f64 rounding.
          let flag = match exact_integer_code(cell) {
            Some(0) => 0.0,
            Some(1) => 1.0,
            _ => {
              return Err(heckman_error(
                "selection outcome must be exactly0 or1 on finite selection-design rows".into(),
              ));
            }
          };
          let mut xi = Vec::new();
          let mut yi = 0.0;
          if flag == 1.0 {
            let c = row
              .cell(y_position)
              .map_err(|_| heckman_error("could not read outcome cell".into()))?;
            let Some(value) = cell_to_f64(c).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            yi = value;
            for &j in &x_positions {
              let c = row
                .cell(j)
                .map_err(|_| heckman_error("could not read outcome regressor".into()))?;
              let Some(value) = cell_to_f64(c).filter(|v| v.is_finite()) else {
                return Ok(());
              };
              xi.push(value);
            }
          }
          let next_n = eligible.len() + 1;
          let next_selected = selected.len() + usize::from(flag == 1.0);
          if next_n > tabdat_stats::MAX_HECKMAN_OBSERVATIONS
            || next_n * q * q + next_selected * p * p > tabdat_stats::MAX_HECKMAN_WORK
          {
            return Err(heckman_error(
              "eligible two-stage sample exceeds10000 rows or expanded-work bound".into(),
            ));
          }
          eligible.push(index);
          d.push(flag);
          z.push(zi);
          if flag == 1.0 {
            selected.push(index);
            y.push(yi);
            x.push(xi);
          }
          Ok(())
        },
        heckman_error,
      )?;
    let selection_sample = tabdat_stats::EstimationSample::new(
      eligible.clone(),
      total,
      total - eligible.len(),
      None,
      None,
    )
    .map_err(|e| heckman_error(e.to_string()))?;
    let outcome_sample = tabdat_stats::EstimationSample::new(
      selected.clone(),
      total,
      total - selected.len(),
      None,
      None,
    )
    .map_err(|e| heckman_error(e.to_string()))?;
    let selection = tabdat_stats::EstimationProblem::new(
      command.selection_dependent.clone(),
      command.selection_predictors.clone(),
      d,
      z,
    )
    .map_err(|e| heckman_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(selection_sample);
    let outcome = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      y,
      x,
    )
    .map_err(|e| heckman_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(outcome_sample);
    let result = tabdat_stats::fit_heckman_two_step(
      &selection,
      &outcome,
      &tabdat_stats::ProbitOptions::default(),
    )
    .map_err(|e| heckman_error(e.to_string()))?;
    self.estimation = Some(EstimationState::HeckmanTwoStepFit(Box::new(result.clone())));
    Ok(ExecutionResult::HeckmanTwoStep(Box::new(result)))
  }

  pub(crate) fn execute_heckman_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    use tabdat_stats::HeckmanPredictionKind as Kind;
    let kind = match command.kind {
      PredictKind::Xb => Kind::Xb,
      PredictKind::Pr => Kind::Pr,
      PredictKind::SelectionMills => Kind::Mills,
      PredictKind::SelectionConditionalMean => Kind::ConditionalSelectedMean,
      PredictKind::SelectionObservedContribution => Kind::ObservedContributionMean,
      _ => {
        return Err(heckman_error(
          "prediction supports only xb, pr, mills, ycond and yexpected".into(),
        ));
      }
    };
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(heckman_error(
        "two-step predictions are point-only, without auxiliary options".into(),
      ));
    }
    let Some(EstimationState::HeckmanTwoStepFit(state)) = &self.estimation else {
      return Err(heckman_error(
        "a successful Heckman twostep fit is required".into(),
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
    if dataset.row_count > tabdat_stats::MAX_HECKMAN_OBSERVATIONS as u64 {
      return Err(heckman_error(
        "prediction supports at most10000 current rows".into(),
      ));
    }
    let needs_x = matches!(
      kind,
      Kind::Xb | Kind::ConditionalSelectedMean | Kind::ObservedContributionMean
    );
    let needs_z = kind != Kind::Xb;
    let mut names = Vec::new();
    for name in state
      .predictor_names
      .iter()
      .filter(|_| needs_x)
      .chain(state.selection.predictor_names.iter().filter(|_| needs_z))
    {
      if !names.contains(name) {
        names.push(name.clone());
      }
    }
    let missing: Vec<_> = names
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
        return Err(heckman_error(format!(
          "prediction requires numeric predictor: {name}"
        )));
      }
    }
    let positions = |source: &Vec<String>, needed: bool| {
      if needed {
        source
          .iter()
          .map(|name| {
            names
              .iter()
              .position(|n| n == name)
              .expect("required variable")
          })
          .collect::<Vec<_>>()
      } else {
        Vec::new()
      }
    };
    let x_positions = positions(&state.predictor_names, needs_x);
    let z_positions = positions(&state.selection.predictor_names, needs_z);
    let select = if names.is_empty() {
      "1".into()
    } else {
      names
        .iter()
        .map(|n| quote_identifier(n))
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
          if values.len() >= tabdat_stats::MAX_HECKMAN_OBSERVATIONS {
            return Err(heckman_error(
              "prediction supports at most10000 current rows".into(),
            ));
          }
          let mut cells = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let c = row
              .cell(j)
              .map_err(|_| heckman_error("could not read prediction cell".into()))?;
            let Some(value) = cell_to_f64(c).filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            cells.push(value);
          }
          let x: Vec<_> = x_positions.iter().map(|&i| cells[i]).collect();
          let z: Vec<_> = z_positions.iter().map(|&i| cells[i]).collect();
          values.push(Some(
            tabdat_stats::heckman_predict_row(state, &x, &z, kind)
              .map_err(|e| heckman_error(e.to_string()))?,
          ));
          Ok(())
        },
        heckman_error,
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
