use crate::*;

use tabdat_stats::BinaryModel;

/// Family-specific failed-fit error shape; logit messages stay byte-identical
/// to the pre-probit slice.
fn binary_error(model: BinaryModel) -> fn(String) -> RuntimeError {
  match model {
    BinaryModel::Logit => |message| RuntimeError::LogitFailed { message },
    BinaryModel::Probit => |message| RuntimeError::ProbitFailed { message },
  }
}

fn binary_family(model: BinaryModel) -> &'static str {
  match model {
    BinaryModel::Logit => "logit",
    BinaryModel::Probit => "probit",
  }
}

fn exact_binary_outcome(cell: &CellValue) -> bool {
  match cell {
    CellValue::Float(value) => *value == 0.0 || *value == 1.0,
    CellValue::SignedInteger(value) => *value == 0 || *value == 1,
    CellValue::UnsignedInteger(value) => *value == 0 || *value == 1,
    CellValue::Decimal { value, scale, .. } => {
      *value == 0 || 10_i128.checked_pow(u32::from(*scale)) == Some(*value)
    }
    _ => false,
  }
}

fn binary_cluster_key(cell: &CellValue) -> Result<Option<String>, &'static str> {
  Ok(match cell {
    CellValue::Null => None,
    CellValue::Float(value) if !value.is_finite() => None,
    CellValue::Float(value) => Some(if *value == 0.0 {
      "0".to_owned()
    } else {
      value.to_string()
    }),
    CellValue::Text(value) => Some(value.clone()),
    CellValue::SignedInteger(value) => Some(value.to_string()),
    CellValue::UnsignedInteger(value) => Some(value.to_string()),
    // Never round cluster identity through f64: adjacent wide DECIMAL IDs
    // can otherwise collapse into a single group. Scale is fixed per column.
    CellValue::Decimal { value, scale, .. } => Some(format!("{value}e-{scale}")),
    CellValue::Boolean(value) => Some(value.to_string()),
    CellValue::Bytes(_) => return Err("cluster variable must be numeric, text or boolean"),
  })
}

impl Session {
  pub(crate) fn execute_logit(
    &mut self,
    command: &LogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.execute_binary_fit(
      &command.outcome,
      &command.predictors,
      command.robust,
      command.cluster_variable.as_deref(),
      command.include_intercept,
      BinaryModel::Logit,
    )
  }

  pub(crate) fn execute_probit(
    &mut self,
    command: &ProbitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.execute_binary_fit(
      &command.outcome,
      &command.predictors,
      command.robust,
      command.cluster_variable.as_deref(),
      command.include_intercept,
      BinaryModel::Probit,
    )
  }

  fn execute_binary_fit(
    &mut self,
    outcome: &str,
    predictors: &[String],
    robust: bool,
    cluster_variable: Option<&str>,
    include_intercept: bool,
    model: BinaryModel,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Match the supported linear/IV policy: a failed new fit clears stale inference.
    self.estimation = None;
    let error = binary_error(model);
    if robust && cluster_variable.is_some() {
      return Err(error("cannot combine robust and cluster".to_owned()));
    }
    let command_name = binary_family(model);
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: command_name,
      })?;
    let variables: Vec<&str> = std::iter::once(outcome)
      .chain(predictors.iter().map(String::as_str))
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(error(format!("numeric variable required: {name}")));
      }
    }
    let mut select_columns: Vec<String> = variables
      .iter()
      .map(|name| quote_identifier(name))
      .collect();
    if let Some(name) = cluster_variable {
      let column = dataset
        .columns
        .iter()
        .find(|column| column.name == name)
        .ok_or_else(|| error(format!("unknown cluster variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type)
        && !matches!(column.data_type.as_str(), "VARCHAR" | "BOOLEAN")
      {
        return Err(error(
          "cluster variable must be numeric, text or boolean".to_owned(),
        ));
      }
      select_columns.push(quote_identifier(name));
    }
    let query = format!("SELECT {} FROM {ACTIVE_TABLE}", select_columns.join(", "));
    let mut total_rows = 0;
    let mut retained = Vec::new();
    let mut response = Vec::new();
    let mut design = Vec::new();
    let mut cluster_groups = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: command_name,
      })?
      .visit_rows(
        &query,
        |row| {
          let index = total_rows;
          total_rows += 1;
          let mut values = Vec::with_capacity(variables.len());
          let mut outcome_is_binary = false;
          for j in 0..variables.len() {
            let cell = row.cell(j).map_err(|read_error| {
              error(match read_error {
                CellReadError::Backend(message) => message,
                _ => format!("could not read {command_name} numeric cell"),
              })
            })?;
            if j == 0 {
              // Validate the owned numeric value before f64 rounding can map a
              // nearby nonbinary DECIMAL onto one. Exclusions still come first.
              outcome_is_binary = exact_binary_outcome(cell);
            }
            match cell_to_f64(cell) {
              Some(v) if v.is_finite() => values.push(v),
              _ => return Ok(()),
            }
          }
          let cluster_group = if cluster_variable.is_some() {
            let cell = row
              .cell(variables.len())
              .map_err(|_| error("could not read cluster cell".to_owned()))?;
            let Some(group) =
              binary_cluster_key(cell).map_err(|message| error(message.to_owned()))?
            else {
              return Ok(());
            };
            Some(group)
          } else {
            None
          };
          if !outcome_is_binary {
            return Err(error(
              "outcome must be exactly 0 or 1 on the complete finite estimation sample".to_owned(),
            ));
          }
          retained.push(index);
          // Canonicalize validated exact DECIMAL one: its f64 division need
          // not itself round to bit-exact one at every scale.
          response.push(if values[0] == 0.0 { 0.0 } else { 1.0 });
          design.push(values[1..].to_vec());
          if let Some(group) = cluster_group {
            cluster_groups.push(group);
          }
          Ok(())
        },
        error,
      )?;
    if retained.is_empty() {
      return Err(error("no complete finite observations".to_owned()));
    }
    let dropped = total_rows - retained.len();
    let sample = tabdat_stats::EstimationSample::new(
      retained,
      total_rows,
      dropped,
      None,
      cluster_variable.is_some().then_some(cluster_groups),
    )
    .map_err(|e| error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      outcome.to_owned(),
      predictors.to_vec(),
      response,
      design,
    )
    .map_err(|e| error(e.to_string()))?
    .with_intercept(include_intercept)
    .with_sample(sample);
    let result = if let Some(variable) = cluster_variable {
      tabdat_stats::fit_binary_clustered(&problem, model, variable)
    } else if robust {
      tabdat_stats::fit_binary_robust(&problem, model)
    } else {
      match model {
        BinaryModel::Logit => tabdat_stats::fit_binary_logit(&problem),
        BinaryModel::Probit => tabdat_stats::fit_binary_probit(&problem),
      }
    }
    .map_err(|e| error(e.to_string()))?;
    self.estimation = Some(match model {
      BinaryModel::Logit => EstimationState::Logit(Box::new(BinaryLogitState {
        problem,
        result: result.clone(),
      })),
      BinaryModel::Probit => EstimationState::Probit(Box::new(BinaryProbitState {
        problem,
        result: result.clone(),
      })),
    });
    Ok(match model {
      BinaryModel::Logit => ExecutionResult::Logit(Box::new(result)),
      BinaryModel::Probit => ExecutionResult::Probit(Box::new(result)),
    })
  }

  pub(crate) fn execute_logit_margins(&self) -> Result<ExecutionResult, RuntimeError> {
    let Some(EstimationState::Logit(state)) = &self.estimation else {
      return Err(RuntimeError::EstatMarginsRequiresPriorLogit);
    };
    let result = tabdat_stats::logit_average_marginal_effects(&state.problem, &state.result)
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })?;
    Ok(ExecutionResult::LogitMarginalEffects(result))
  }

  pub(crate) fn execute_probit_margins(
    &self,
    state: &BinaryProbitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    let result = tabdat_stats::binary_average_marginal_effects(&state.problem, &state.result)
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })?;
    Ok(ExecutionResult::ProbitMarginalEffects(Box::new(result)))
  }

  pub(crate) fn execute_logit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.execute_binary_prediction(command, BinaryModel::Logit)
  }

  pub(crate) fn execute_probit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.execute_binary_prediction(command, BinaryModel::Probit)
  }

  fn execute_binary_prediction(
    &mut self,
    command: &PredictCommand,
    model: BinaryModel,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(command.kind, PredictKind::Pr | PredictKind::Xb)
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(match model {
        BinaryModel::Logit => RuntimeError::PredictLogitUnsupported,
        BinaryModel::Probit => RuntimeError::PredictBinaryUnsupported,
      });
    }
    let error = binary_error(model);
    let Some(state) = (match (&self.estimation, model) {
      (Some(EstimationState::Logit(state)), BinaryModel::Logit) => Some(state),
      (Some(EstimationState::Probit(state)), BinaryModel::Probit) => Some(state),
      _ => None,
    }) else {
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
        return Err(error(format!(
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
    let probability = command.kind == PredictKind::Pr;
    let added = {
      let backend = self.backend.as_mut().ok_or(RuntimeError::PredictFailed)?;
      match model {
        BinaryModel::Logit => {
          backend.add_logit_prediction_column(&dataset, &prediction, probability)
        }
        BinaryModel::Probit => {
          backend.add_probit_prediction_column(&dataset, &prediction, probability)
        }
      }
      .map_err(|_| RuntimeError::PredictFailed)?
    };
    let next = self.sync_active_dataset(added)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }
}
