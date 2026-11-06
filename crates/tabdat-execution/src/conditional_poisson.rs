use crate::count::CountFamily;
use crate::*;

fn error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::XtPoissonFailed {
    message: message.into(),
  }
}
impl Session {
  pub(crate) fn execute_conditional_poisson(
    &mut self,
    command: &XtPoissonCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let k = command.predictors.len();
    if !(1..=32).contains(&k) {
      return Err(error(
        "requires1..32 numeric predictors, without a common intercept",
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "xtpoisson",
      })?;
    let column = dataset
      .columns
      .iter()
      .find(|c| c.name == command.entity_variable)
      .ok_or_else(|| error("unknown group variable"))?;
    let entity_data_type = column.data_type.clone();
    if !is_numeric_data_type(&entity_data_type)
      && !matches!(entity_data_type.as_str(), "VARCHAR" | "BOOLEAN")
    {
      return Err(error("group variable must be numeric, text or boolean"));
    }
    // A missing key cannot be assigned to a group even when y/X are missing.
    let mut rows = 0usize;
    let mut groups = BTreeSet::new();
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      quote_identifier(&command.entity_variable)
    );
    self
      .backend
      .as_ref()
      .ok_or_else(|| error("data backend unavailable"))?
      .visit_rows(
        &query,
        |row| {
          rows += 1;
          if rows > 40000 || rows.saturating_mul(k * k) > 20_000_000 {
            return Err(error(
              "candidate rows<=40000 and rows*K^2<=20000000 required",
            ));
          }
          let cell = row.cell(0).map_err(|_| error("could not read group key"))?;
          let key = cluster_key(cell)
            .map_err(error)?
            .ok_or_else(|| error("all raw group keys must be nonmissing and finite"))?;
          groups.insert(key);
          if groups.len() > 10000 {
            return Err(error("requires at most10000 candidate groups"));
          }
          Ok(())
        },
        error,
      )?;
    let (problem, _) = self.count_problem(
      CountFamily::ConditionalPoisson,
      &command.outcome,
      &command.predictors,
      false,
      Some(&command.entity_variable),
      None,
    )?;
    let result = tabdat_stats::fit_conditional_poisson(
      &problem,
      &command.entity_variable,
      &Default::default(),
    )
    .map_err(|e| error(e.to_string()))?;
    self.estimation = Some(EstimationState::ConditionalPoisson(Box::new(
      ConditionalPoissonState {
        entity_data_type,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::ConditionalPoisson(Box::new(result)))
  }

  pub(crate) fn execute_conditional_poisson_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(
      command.kind,
      PredictKind::Xb | PredictKind::MeanCount | PredictKind::Residuals
    ) || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
    {
      return Err(error(
        "supports slope-only xb, retained-group n or residuals only; no population/new-group forecast",
      ));
    }
    let Some(EstimationState::ConditionalPoisson(state)) = &self.estimation else {
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
    let group_required = command.kind != PredictKind::Xb;
    let residual = command.kind == PredictKind::Residuals;
    let mut variables = result.predictor_names.clone();
    if group_required {
      variables.push(result.entity_variable.clone());
    }
    if residual {
      variables.push(result.outcome.clone());
    }
    let k = result.predictor_names.len();
    for (i, name) in variables.iter().enumerate() {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| error(format!("required prediction variable missing: {name}")))?;
      if group_required && i == k {
        if column.data_type != state.entity_data_type {
          return Err(error("group logical type must match the fitted model"));
        }
      } else if !is_numeric_data_type(&column.data_type) {
        return Err(error(format!(
          "numeric prediction variable required: {name}"
        )));
      }
    }
    let effects = result
      .effects
      .iter()
      .map(|e| (e.key.as_str(), e))
      .collect::<BTreeMap<_, _>>();
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      variables
        .iter()
        .map(|v| quote_identifier(v))
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
          if values.len() >= 40000 || (values.len() + 1).saturating_mul(k * k) > 20_000_000 {
            return Err(error(
              "prediction rows<=40000 and rows*K^2<=20000000 required",
            ));
          }
          let mut x = Vec::with_capacity(k);
          for i in 0..k {
            let cell = row
              .cell(i)
              .map_err(|_| error("could not read prediction value"))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            x.push(value);
          }
          let value = if group_required {
            let cell = row
              .cell(k)
              .map_err(|_| error("could not read prediction group"))?;
            let key = cluster_key(cell).map_err(error)?;
            let Some(effect) = key.as_deref().and_then(|key| effects.get(key)) else {
              values.push(None);
              return Ok(());
            };
            let log_mean = effect.log_normalizer
              + x
                .iter()
                .zip(&effect.anchor)
                .zip(&result.coefficients)
                .map(|((v, a), b)| (v - a) * b.value)
                .sum::<f64>();
            if !log_mean.is_finite() {
              values.push(None);
              return Ok(());
            }
            let mean = log_mean.exp();
            if residual {
              let outcome = row
                .cell(k + 1)
                .map_err(|_| error("could not read current outcome"))?;
              cell_to_f64(outcome)
                .filter(|v| v.is_finite())
                .map(|v| v - mean)
            } else {
              Some(mean)
            }
          } else {
            Some(
              x.iter()
                .zip(&result.coefficients)
                .map(|(v, b)| v * b.value)
                .sum::<f64>(),
            )
          };
          values.push(value.filter(|v| v.is_finite()));
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
