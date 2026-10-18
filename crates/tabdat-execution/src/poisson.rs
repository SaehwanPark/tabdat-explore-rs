use crate::*;

fn poisson_error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::PoissonFailed {
    message: message.into(),
  }
}

// Validate integer identity BEFORE f64 conversion: a nearby fractional DECIMAL
// or an out-of-range wide integer must not round onto a supported count.
fn exact_poisson_count(cell: &CellValue) -> Option<f64> {
  const MAX: i128 = 9_007_199_254_740_991;
  match cell {
    CellValue::Float(y)
      if y.is_finite()
        && *y >= 0.0
        && *y <= tabdat_stats::MAX_POISSON_COUNT
        && y.fract() == 0.0 =>
    {
      Some(if *y == 0.0 { 0.0 } else { *y })
    }
    CellValue::SignedInteger(y) if (0..=MAX).contains(y) => Some(*y as f64),
    CellValue::UnsignedInteger(y) if *y <= MAX as u128 => Some(*y as f64),
    CellValue::Decimal { value, scale, .. } if *value >= 0 => {
      let divisor = 10_i128.checked_pow(u32::from(*scale))?;
      (*value % divisor == 0 && *value / divisor <= MAX).then(|| (*value / divisor) as f64)
    }
    _ => None,
  }
}

impl Session {
  pub(crate) fn execute_poisson(
    &mut self,
    command: &PoissonCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    if command.cluster_variable.is_some() {
      return Err(poisson_error(
        "clustered Poisson is not implemented; robust assumes independent rows",
      ));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "poisson" })?;
    let variables: Vec<&str> = std::iter::once(command.outcome.as_str())
      .chain(command.predictors.iter().map(String::as_str))
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| poisson_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(poisson_error(format!("numeric variable required: {name}")));
      }
    }
    let columns: Vec<String> = variables
      .iter()
      .map(|name| quote_identifier(name))
      .collect();
    let query = format!("SELECT {} FROM {ACTIVE_TABLE}", columns.join(", "));
    let mut total = 0;
    let mut retained = Vec::new();
    let mut response = Vec::new();
    let mut design = Vec::new();
    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "poisson" })?;
    backend.visit_rows(
      &query,
      |row| {
        let index = total;
        total += 1;
        let mut values = Vec::with_capacity(variables.len());
        let mut count = None;
        for j in 0..variables.len() {
          let cell = row.cell(j).map_err(|error| match error {
            CellReadError::Backend(message) => poisson_error(message),
            _ => poisson_error("could not read Poisson numeric cell"),
          })?;
          if j == 0 { count = exact_poisson_count(cell); }
          match cell_to_f64(cell) {
            Some(value) if value.is_finite() => values.push(value),
            _ => return Ok(()),
          }
        }
        // Domain rejection follows joint finite-case selection: an invalid
        // finite outcome beside a missing predictor is not an estimation row.
        let count = count.ok_or_else(|| {
          poisson_error(
            "outcome must be an exact nonnegative integer at most 2^53-1 on the complete finite estimation sample",
          )
        })?;
        retained.push(index);
        response.push(count);
        design.push(values[1..].to_vec());
        Ok(())
      },
      poisson_error,
    )?;
    if retained.is_empty() {
      return Err(poisson_error("no complete finite observations"));
    }
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, dropped, None, None)
      .map_err(|e| poisson_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design,
    )
    .map_err(|e| poisson_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result = tabdat_stats::fit_poisson(&problem, command.robust)
      .map_err(|e| poisson_error(e.to_string()))?;
    self.estimation = Some(EstimationState::PoissonFit(Box::new(PoissonFitState {
      problem,
      result: result.clone(),
    })));
    Ok(ExecutionResult::Poisson(Box::new(result)))
  }

  pub(crate) fn execute_poisson_margins(
    &self,
    state: &PoissonFitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    tabdat_stats::poisson_average_marginal_effects(&state.problem, &state.result)
      .map(|result| ExecutionResult::PoissonMarginalEffects(Box::new(result)))
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })
  }

  pub(crate) fn execute_poisson_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(command.kind, PredictKind::Xb | PredictKind::MeanCount)
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(RuntimeError::PredictPoissonUnsupported);
    }
    let Some(EstimationState::PoissonFit(state)) = &self.estimation else {
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
    let predictors = &state.problem.predictor_names;
    let missing: Vec<_> = predictors
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in predictors {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(poisson_error(format!(
          "prediction requires numeric predictor: {name}"
        )));
      }
    }
    let offset = usize::from(state.problem.include_intercept);
    let coefficients: Vec<_> = state.result.coefficients[offset..]
      .iter()
      .map(|c| c.value)
      .collect();
    let prediction = LinearPrediction {
      target_variable: &command.target_variable,
      predictor_names: predictors,
      predictor_coefficients: &coefficients,
      intercept: state
        .problem
        .include_intercept
        .then(|| state.result.coefficients[0].value),
      outcome_variable: &state.problem.outcome,
      residuals: false,
    };
    let added = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_poisson_prediction_column(
        &dataset,
        &prediction,
        command.kind == PredictKind::MeanCount,
      )
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next = self.sync_active_dataset(added)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }
}

#[test]
fn exact_counts_reject_rounding_fractional_decimal_and_wide_integer_identities() {
  assert_eq!(exact_poisson_count(&CellValue::Float(-0.0)), Some(0.0));
  assert_eq!(exact_poisson_count(&CellValue::Float(0.5)), None);
  assert_eq!(
    exact_poisson_count(&CellValue::SignedInteger(9_007_199_254_740_992)),
    None
  );
  let near_one = CellValue::Decimal {
    width: 38,
    scale: 20,
    value: 100_000_000_000_000_000_001,
  };
  assert_eq!(exact_poisson_count(&near_one), None);
  let integer = CellValue::Decimal {
    width: 38,
    scale: 20,
    value: 300_000_000_000_000_000_000,
  };
  assert_eq!(exact_poisson_count(&integer), Some(3.0));
}
