use crate::*;

fn mlogit_error(message: String) -> RuntimeError {
  RuntimeError::MlogitFailed { message }
}

// Check integer/decimal identity before f64 conversion can round a noninteger or
// an out-of-domain key into a permitted category. Complete-case exclusion occurs
// before this validation is used to reject the row.
fn exact_category(cell: &CellValue) -> Option<i64> {
  let value = match cell {
    CellValue::Float(v) => return tabdat_stats::multinomial_outcome_code(*v).ok(),
    CellValue::SignedInteger(v) => i64::try_from(*v).ok()?,
    CellValue::UnsignedInteger(v) => i64::try_from(*v).ok()?,
    CellValue::Decimal { value, scale, .. } => {
      let divisor = 10_i128.checked_pow(u32::from(*scale))?;
      if value % divisor != 0 {
        return None;
      }
      i64::try_from(value / divisor).ok()?
    }
    _ => return None,
  };
  (value.unsigned_abs() <= 9_007_199_254_740_991).then_some(value)
}

impl Session {
  pub(crate) fn execute_mlogit(
    &mut self,
    command: &MlogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Like the executable linear/GLM families, a failed new fit clears stale inference.
    self.estimation = None;
    if command.predictors.len() + usize::from(command.include_intercept) > 32 {
      return Err(mlogit_error(
        "requires K<=32 including the intercept".into(),
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "mlogit" })?;
    let variables = std::iter::once(command.outcome.as_str())
      .chain(command.predictors.iter().map(String::as_str))
      .collect::<Vec<_>>();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| mlogit_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(mlogit_error(format!("numeric variable required: {name}")));
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
    self.backend.as_ref().ok_or(RuntimeError::NoActiveDataset { command:"mlogit" })?.visit_rows(&query,|row| {
      let index=total; total+=1; let mut code=None; let mut values=Vec::with_capacity(variables.len());
      for j in 0..variables.len() {
        let cell = row.cell(j).map_err(|_|mlogit_error("could not read numeric cell".into()))?;
        if j==0 { code=exact_category(cell); }
        let Some(value)=cell_to_f64(cell).filter(|v|v.is_finite()) else { return Ok(()); };
        values.push(value);
      }
      let code=code.ok_or_else(||mlogit_error("outcome must be an exact integer code within +/- (2^53-1) on the complete finite sample".into()))?;
      if response.len()>=10_000 { return Err(mlogit_error("requires N<=10000 complete finite observations".into())); }
      retained.push(index); response.push(code as f64); design.push(values[1..].to_vec()); Ok(())
    },mlogit_error)?;
    let sample = tabdat_stats::EstimationSample::new(
      retained.clone(),
      total,
      total - retained.len(),
      None,
      None,
    )
    .map_err(|e| mlogit_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design,
    )
    .map_err(|e| mlogit_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result = tabdat_stats::fit_multinomial_logit(
      &problem,
      &tabdat_stats::MultinomialOptions {
        base_outcome: command.base_outcome,
        ..Default::default()
      },
    )
    .map_err(|e| mlogit_error(e.to_string()))?;
    self.estimation = Some(EstimationState::MultinomialLogit(Box::new(
      MultinomialLogitState {
        problem,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::MultinomialLogit(Box::new(result)))
  }

  pub(crate) fn execute_mlogit_margins(
    &self,
    state: &MultinomialLogitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    let result = tabdat_stats::multinomial_marginal_effects(&state.problem, &state.result)
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })?;
    Ok(ExecutionResult::MultinomialMarginalEffects(Box::new(
      result,
    )))
  }

  pub(crate) fn execute_mlogit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let (category, probability) = match command.kind {
      PredictKind::MultinomialPr(c) => (c, true),
      PredictKind::MultinomialXb(c) => (c, false),
      _ => {
        return Err(mlogit_error(
          "predict requires explicit pr outcome(code) or xb outcome(code) after mlogit".into(),
        ));
      }
    };
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
    {
      return Err(mlogit_error("multinomial prediction supports only scalar pr/xb outcome(code), without uncertainty or saving()".into()));
    }
    let Some(EstimationState::MultinomialLogit(state)) = &self.estimation else {
      return Err(RuntimeError::PredictRequiresPriorRegression);
    };
    let selected = state
      .result
      .category_levels
      .binary_search(&category)
      .map_err(|_| {
        mlogit_error(format!(
          "outcome {category} is absent from the retained category levels"
        ))
      })?;
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
    let names = &state.problem.predictor_names;
    let missing = names
      .iter()
      .filter(|n| !dataset.columns.iter().any(|c| c.name == **n))
      .cloned()
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in names {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(mlogit_error(format!(
          "prediction requires numeric predictor: {name}"
        )));
      }
    }
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
          let mut predictors = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let cell = row
              .cell(j)
              .map_err(|_| mlogit_error("could not read prediction cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            predictors.push(value);
          }
          let (p, xb) = tabdat_stats::multinomial_predict_row(&state.result, &predictors)
            .map_err(|e| mlogit_error(e.to_string()))?;
          values.push(Some(if probability {
            p[selected]
          } else {
            xb[selected]
          }));
          Ok(())
        },
        mlogit_error,
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
