use crate::*;

fn error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::XtGeeFailed {
    message: message.into(),
  }
}
impl Session {
  pub(crate) fn execute_xtgee_poisson(
    &mut self,
    command: &XtGeeCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let problem = self.prepare_gee_problem(command)?;
    let result =
      tabdat_stats::fit_poisson_gee(&problem, &command.entity_variable, &Default::default())
        .map_err(|e| error(e.to_string()))?;
    self.estimation = Some(EstimationState::PoissonGee(Box::new(result.clone())));
    Ok(ExecutionResult::PoissonGee(Box::new(result)))
  }

  pub(crate) fn execute_poisson_gee_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || !matches!(
        command.kind,
        PredictKind::MeanCount | PredictKind::Xb | PredictKind::Residuals
      )
    {
      return Err(error(
        "Poisson GEE supports marginal n, xb or residuals only; no intervals, std, count probabilities or saved draws",
      ));
    }
    let Some(EstimationState::PoissonGee(result)) = &self.estimation else {
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
    let mut required = result.predictor_names.clone();
    if residual {
      required.push(result.outcome.clone());
    }
    let types: HashMap<_, _> = dataset
      .columns
      .iter()
      .map(|c| (c.name.as_str(), c.data_type.as_str()))
      .collect();
    let missing = required
      .iter()
      .filter(|n| !types.contains_key(n.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    if required
      .iter()
      .any(|n| !is_numeric_data_type(types[n.as_str()]))
    {
      return Err(error("current outcome/predictors must be numeric"));
    }
    let coefficients = &result.working.coefficients;
    if coefficients.len() != result.predictor_names.len() + 1
      || coefficients.iter().any(|c| !c.value.is_finite())
      || coefficients
        .iter()
        .skip(1)
        .map(|c| &c.name)
        .ne(result.predictor_names.iter())
    {
      return Err(error(
        "invalid owned Poisson GEE coefficient dimensions/names",
      ));
    }
    let beta = coefficients.iter().map(|c| c.value).collect::<Vec<_>>();
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
          if values.len() >= 40000
            || (values.len() + 1).saturating_mul(beta.len() * beta.len()) > 20_000_000
          {
            return Err(error(
              "prediction rows<=40000 and rows*K^2<=20000000 required",
            ));
          }
          let mut xb = beta[0];
          for (j, b) in beta[1..].iter().enumerate() {
            let Some(x) = cell_to_f64(row.cell(j).map_err(|_| error("could not read predictor"))?)
              .filter(|v| v.is_finite())
            else {
              values.push(None);
              return Ok(());
            };
            xb += b * x;
          }
          if !xb.is_finite() {
            return Err(error("nonfinite predicted index"));
          }
          let value = if command.kind == PredictKind::Xb {
            Some(xb)
          } else {
            let mean = xb.exp();
            if !mean.is_finite() || mean <= 0.0 {
              return Err(error("unrepresentable predicted marginal mean"));
            }
            if residual {
              cell_to_f64(
                row
                  .cell(beta.len() - 1)
                  .map_err(|_| error("could not read outcome"))?,
              )
              .filter(|v| v.is_finite())
              .map(|y| y - mean)
            } else {
              Some(mean)
            }
          };
          if value.is_some_and(|v| !v.is_finite()) {
            return Err(error("nonfinite predicted residual"));
          }
          values.push(value);
          Ok(())
        },
        error,
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
