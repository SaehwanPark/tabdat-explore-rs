use crate::*;

fn lowess_error(message: String) -> RuntimeError {
  RuntimeError::LowessFailed(message)
}

impl Session {
  pub(crate) fn execute_lowess(
    &mut self,
    command: &LowessCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "lowess" })?
      .clone();
    if command.target_variable.is_empty()
      || dataset
        .columns
        .iter()
        .any(|c| c.name.eq_ignore_ascii_case(&command.target_variable))
    {
      return Err(lowess_error(
        "target must be a nonempty new column name (case-insensitive collisions are not replaced)"
          .into(),
      ));
    }
    for name in [&command.outcome, &command.predictor] {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| lowess_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(lowess_error(format!("numeric variable required: {name}")));
      }
    }
    let bandwidth = command
      .bandwidth
      .parse::<f64>()
      .map_err(|_| lowess_error("bandwidth must be a number".into()))?;
    let query = format!(
      "SELECT {}, {} FROM {ACTIVE_TABLE}",
      quote_identifier(&command.outcome),
      quote_identifier(&command.predictor)
    );
    let (mut total, mut retained, mut outcome, mut predictor) =
      (0, Vec::new(), Vec::new(), Vec::new());
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "lowess" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let y = row
            .cell(0)
            .map_err(|_| lowess_error("could not read numeric outcome cell".into()))?;
          let x = row
            .cell(1)
            .map_err(|_| lowess_error("could not read numeric predictor cell".into()))?;
          let (Some(y), Some(x)) = (
            cell_to_f64(y).filter(|v| v.is_finite()),
            cell_to_f64(x).filter(|v| v.is_finite()),
          ) else {
            return Ok(());
          };
          if retained.len() >= tabdat_stats::MAX_LOWESS_OBSERVATIONS {
            return Err(lowess_error(
              "at most10000 joint-finite observations are supported".into(),
            ));
          }
          retained.push(index);
          outcome.push(y);
          predictor.push(x);
          Ok(())
        },
        lowess_error,
      )?;
    let smoothing = tabdat_stats::smooth_lowess(&predictor, &outcome, bandwidth)
      .map_err(|e| lowess_error(e.to_string()))?;
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, dropped, None, None)
      .map_err(|e| lowess_error(e.to_string()))?;
    let mut values = vec![None; total];
    for (index, value) in sample.retained_indices.iter().zip(smoothing.values) {
      values[*index] = Some(value);
    }
    // No data/model mutation during sample construction or smoothing. Typed
    // staged publication keeps original storage/order and excluded-row NULLs;
    // unlike a new estimator attempt this operation never clears the old fit.
    let next = self
      .backend
      .as_mut()
      .ok_or_else(|| lowess_error("data backend unavailable".into()))?
      .add_numeric_prediction_column(&dataset, &command.target_variable, &values)
      .map_err(|_| lowess_error("could not publish new numeric column".into()))?;
    let dataset = self.sync_active_dataset(next)?;
    Ok(ExecutionResult::Lowess(Box::new(LowessResult {
      dataset,
      outcome: command.outcome.clone(),
      predictor: command.predictor.clone(),
      target_variable: command.target_variable.clone(),
      bandwidth,
      sample,
      half_window: smoothing.half_window,
      local_constant_windows: smoothing.local_constant_windows,
    })))
  }
}
