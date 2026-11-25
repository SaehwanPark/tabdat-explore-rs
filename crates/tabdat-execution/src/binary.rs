use crate::row_conversion::cluster_key as binary_cluster_key;
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

#[test]
fn cluster_identity_is_exact_and_only_finite_values_are_retained() {
  assert_eq!(
    binary_cluster_key(&CellValue::Float(-0.0)),
    binary_cluster_key(&CellValue::Float(0.0))
  );
  assert_eq!(binary_cluster_key(&CellValue::Float(f64::NAN)), Ok(None));
  assert_eq!(
    binary_cluster_key(&CellValue::Float(f64::INFINITY)),
    Ok(None)
  );
  assert_eq!(binary_cluster_key(&CellValue::Null), Ok(None));
  assert_eq!(
    binary_cluster_key(&CellValue::Text(String::new())),
    Ok(Some(String::new()))
  );
  assert!(binary_cluster_key(&CellValue::Bytes(vec![1])).is_err());
  let a = CellValue::Decimal {
    width: 38,
    scale: 0,
    value: 9_007_199_254_740_992,
  };
  let b = CellValue::Decimal {
    width: 38,
    scale: 0,
    value: 9_007_199_254_740_993,
  };
  assert_ne!(binary_cluster_key(&a), binary_cluster_key(&b));
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
    let terms = crate::regress_design::parse_regress_terms(predictors)
      .map_err(|failure| error(failure.to_string()))?;
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: command_name,
      })?;
    let outcome_column = dataset
      .columns
      .iter()
      .find(|column| column.name == outcome)
      .ok_or_else(|| error(format!("unknown variable: {outcome}")))?;
    if !is_numeric_data_type(&outcome_column.data_type) {
      return Err(error(format!("numeric variable required: {outcome}")));
    }
    for term in &terms {
      let name = term.variable();
      let column = dataset
        .columns
        .iter()
        .find(|column| column.name == name)
        .ok_or_else(|| error(format!("unknown variable: {name}")))?;
      let supported = match term {
        crate::regress_design::RegressTerm::Continuous { .. } => {
          is_numeric_data_type(&column.data_type)
        }
        crate::regress_design::RegressTerm::Factor { .. } => {
          is_numeric_data_type(&column.data_type)
            || matches!(column.data_type.as_str(), "VARCHAR" | "BOOLEAN")
        }
      };
      if !supported {
        return Err(error(match term {
          crate::regress_design::RegressTerm::Continuous { .. } => {
            format!("numeric variable required: {name}")
          }
          crate::regress_design::RegressTerm::Factor { .. } => {
            format!("factor variable must be text, boolean or numeric codes: {name}")
          }
        }));
      }
    }
    let mut select_columns =
      Vec::with_capacity(1 + terms.len() + usize::from(cluster_variable.is_some()));
    select_columns.push(quote_identifier(outcome));
    select_columns.extend(terms.iter().map(|term| quote_identifier(term.variable())));
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
          let outcome_cell = row.cell(0).map_err(|read_error| {
            error(match read_error {
              CellReadError::Backend(message) => message,
              _ => format!("could not read {command_name} outcome cell"),
            })
          })?;
          let outcome_is_binary = exact_binary_outcome(outcome_cell);
          // Exclude missing/nonfinite outcomes before inspecting the design.
          // A finite nonbinary outcome is rejected only when its whole row is
          // otherwise in the estimation sample, matching the existing policy.
          let Some(outcome_value) = cell_to_f64(outcome_cell).filter(|value| value.is_finite())
          else {
            return Ok(());
          };
          let mut raw_terms = Vec::with_capacity(terms.len());
          for (term_index, term) in terms.iter().enumerate() {
            let cell = row.cell(term_index + 1).map_err(|read_error| {
              error(match read_error {
                CellReadError::Backend(message) => message,
                _ => format!("could not read {command_name} predictor cell"),
              })
            })?;
            match term {
              crate::regress_design::RegressTerm::Continuous { .. } => {
                let Some(value) = cell_to_f64(cell).filter(|value| value.is_finite()) else {
                  return Ok(());
                };
                raw_terms.push(crate::regress_design::ObservedTermValue::Continuous(value));
              }
              crate::regress_design::RegressTerm::Factor { .. } => {
                let Some(value) = crate::regress_design::ObservedFactor::from_cell(cell)
                  .map_err(|failure| error(failure.to_string()))?
                else {
                  return Ok(());
                };
                raw_terms.push(crate::regress_design::ObservedTermValue::Factor(value));
              }
            }
          }
          let cluster_group = if cluster_variable.is_some() {
            let cell = row
              .cell(terms.len() + 1)
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
          response.push(if outcome_value == 0.0 { 0.0 } else { 1.0 });
          design.push(raw_terms);
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
    let expanded = crate::regress_design::expand_design(&terms, &design, include_intercept)
      .map_err(|failure| error(failure.to_string()))?;
    let design_terms = if crate::regress_design::has_factor_terms(&expanded.design_terms) {
      expanded.design_terms
    } else {
      Vec::new()
    };
    let problem = tabdat_stats::EstimationProblem::new(
      outcome.to_owned(),
      expanded.parameter_names,
      response,
      expanded.design_matrix,
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
        design_terms,
      })),
      BinaryModel::Probit => EstimationState::Probit(Box::new(BinaryProbitState {
        problem,
        result: result.clone(),
        design_terms,
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
    let predictor_columns =
      binary_continuous_predictor_columns(&state.problem, &state.design_terms)
        .map_err(|message| RuntimeError::EstatMarginsFailed { message })?;
    let result = tabdat_stats::binary_average_marginal_effects_for_columns(
      &state.problem,
      &state.result,
      &predictor_columns,
    )
    .map_err(|e| RuntimeError::EstatMarginsFailed {
      message: e.to_string(),
    })?;
    Ok(ExecutionResult::LogitMarginalEffects(result))
  }

  pub(crate) fn execute_probit_margins(
    &self,
    state: &BinaryProbitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    let predictor_columns =
      binary_continuous_predictor_columns(&state.problem, &state.design_terms)
        .map_err(|message| RuntimeError::EstatMarginsFailed { message })?;
    let result = tabdat_stats::binary_average_marginal_effects_for_columns(
      &state.problem,
      &state.result,
      &predictor_columns,
    )
    .map_err(|e| RuntimeError::EstatMarginsFailed {
      message: e.to_string(),
    })?;
    Ok(ExecutionResult::ProbitMarginalEffects(Box::new(result)))
  }

  pub(crate) fn execute_binary_discrete_margins(
    &self,
    predictor: &str,
  ) -> Result<ExecutionResult, RuntimeError> {
    let Some((problem, result, design_terms)) = (match &self.estimation {
      Some(EstimationState::Logit(state)) => {
        Some((&state.problem, &state.result, &state.design_terms))
      }
      Some(EstimationState::Probit(state)) => {
        Some((&state.problem, &state.result, &state.design_terms))
      }
      _ => None,
    }) else {
      return Err(RuntimeError::EstatMarginsFailed {
        message: "requires a prior logit or probit fit".into(),
      });
    };
    let continuous_columns = binary_continuous_predictor_columns(problem, design_terms)
      .map_err(|message| RuntimeError::EstatMarginsFailed { message })?;
    if let Some(column) = problem
      .predictor_names
      .iter()
      .position(|name| name == predictor)
      && !continuous_columns.contains(&column)
    {
      return Err(RuntimeError::EstatMarginsFailed {
        message: format!(
          "discrete margins require an original continuous predictor; '{predictor}' is not one"
        ),
      });
    }
    let result = tabdat_stats::binary_average_discrete_change(problem, result, predictor).map_err(
      |error| RuntimeError::EstatMarginsFailed {
        message: error.to_string(),
      },
    )?;
    Ok(ExecutionResult::BinaryDiscreteChange(Box::new(result)))
  }

  pub(crate) fn execute_binary_factor_margins(
    &self,
    variable: &str,
  ) -> Result<ExecutionResult, RuntimeError> {
    let Some((problem, result, design_terms)) = (match &self.estimation {
      Some(EstimationState::Logit(state)) => {
        Some((&state.problem, &state.result, &state.design_terms))
      }
      Some(EstimationState::Probit(state)) => {
        Some((&state.problem, &state.result, &state.design_terms))
      }
      _ => None,
    }) else {
      return Err(RuntimeError::EstatMarginsFailed {
        message: "requires a prior logit or probit fit".into(),
      });
    };
    let design = binary_factor_contrast_design(problem, design_terms, variable)
      .map_err(|message| RuntimeError::EstatMarginsFailed { message })?;
    let contrasts = tabdat_stats::binary_average_factor_contrasts(problem, result, &design)
      .map_err(|error| RuntimeError::EstatMarginsFailed {
        message: error.to_string(),
      })?;
    Ok(ExecutionResult::BinaryFactorContrasts(Box::new(contrasts)))
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
    if !matches!(
      command.kind,
      PredictKind::Pr | PredictKind::PrStandardError | PredictKind::Xb
    ) || (command.interval && command.kind != PredictKind::Pr)
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
    if command.kind == PredictKind::PrStandardError || command.interval {
      let result = state.result.clone();
      let design_terms = if state.design_terms.is_empty() {
        result
          .predictor_names
          .iter()
          .map(|name| LinearDesignTerm::Continuous { name: name.clone() })
          .collect()
      } else {
        state.design_terms.clone()
      };
      return self.execute_binary_probability_uncertainty(
        command,
        model,
        result,
        design_terms,
        state.problem.include_intercept,
      );
    }
    if !state.design_terms.is_empty() {
      let design_terms = state.design_terms.clone();
      let include_intercept = state.problem.include_intercept;
      let coefficients = state
        .result
        .coefficients
        .iter()
        .map(|coefficient| coefficient.value)
        .collect();
      return self.execute_binary_design_prediction(
        command,
        model,
        design_terms,
        include_intercept,
        coefficients,
      );
    }
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
      exposure_variable: None,
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

  fn execute_binary_probability_uncertainty(
    &mut self,
    command: &PredictCommand,
    model: BinaryModel,
    result: tabdat_stats::BinaryMleResult,
    design_terms: Vec<LinearDesignTerm>,
    include_intercept: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let error = binary_error(model);
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "predict" })?
      .clone();
    let expected_coefficients = usize::from(include_intercept)
      + design_terms
        .iter()
        .map(|term| match term {
          LinearDesignTerm::Continuous { .. } => 1,
          LinearDesignTerm::Factor {
            reference, levels, ..
          } => {
            if include_intercept && reference.is_some() {
              levels.len().saturating_sub(1)
            } else {
              levels.len()
            }
          }
        })
        .sum::<usize>();
    if result.coefficients.len() != expected_coefficients {
      return Err(RuntimeError::PredictFailed);
    }
    let required = design_terms
      .iter()
      .map(|term| match term {
        LinearDesignTerm::Continuous { name } => name.clone(),
        LinearDesignTerm::Factor { variable, .. } => variable.clone(),
      })
      .collect::<Vec<_>>();
    let missing = required
      .iter()
      .filter(|name| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == name.as_str())
      })
      .cloned()
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for term in &design_terms {
      let (name, factor) = match term {
        LinearDesignTerm::Continuous { name } => (name, false),
        LinearDesignTerm::Factor { variable, .. } => (variable, true),
      };
      let data_type = &dataset
        .columns
        .iter()
        .find(|column| column.name == *name)
        .ok_or(RuntimeError::PredictFailed)?
        .data_type;
      let supported = if factor {
        is_numeric_data_type(data_type) || matches!(data_type.as_str(), "VARCHAR" | "BOOLEAN")
      } else {
        is_numeric_data_type(data_type)
      };
      if !supported {
        return Err(error(format!(
          "current data has an unsupported type for '{name}'"
        )));
      }
    }

    let selected = if required.is_empty() {
      "1".to_owned()
    } else {
      required
        .iter()
        .map(|name| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ")
    };
    let query = format!("SELECT {selected} FROM {ACTIVE_TABLE}");
    let interval_level = if command.interval {
      Some(
        command
          .level
          .parse::<f64>()
          .map_err(|_| error("prediction interval level is invalid".into()))?,
      )
    } else {
      None
    };
    let mut values = Vec::with_capacity(usize::try_from(dataset.row_count).unwrap_or(0));
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let cells = (0..design_terms.len())
            .map(|index| read_owned_cell(row, index, true, "could not read predictor cell"))
            .collect::<Result<Vec<_>, _>>()?;
          let expanded =
            crate::regress_design::expand_current_row(&design_terms, include_intercept, &cells)
              .map_err(|failure| error(failure.to_string()))?;
          let prediction = match expanded {
            None => (None, None, None),
            Some(mut design) => {
              if include_intercept {
                design.insert(0, 1.0);
              }
              let prediction =
                tabdat_stats::binary_probability_prediction(&result, &design, interval_level)
                  .map_err(|failure| error(failure.to_string()))?;
              if command.interval {
                (
                  Some(prediction.estimate),
                  prediction.lower,
                  prediction.upper,
                )
              } else {
                (Some(prediction.standard_error), None, None)
              }
            }
          };
          values.push(prediction);
          Ok(())
        },
        error,
      )?;
    if u64::try_from(values.len()).ok() != Some(dataset.row_count) {
      return Err(RuntimeError::PredictFailed);
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::PredictFailed)?;
    let added = if command.interval {
      let lower_target = format!("{}_lower", command.target_variable);
      let upper_target = format!("{}_upper", command.target_variable);
      let estimates = values.iter().map(|row| row.0).collect::<Vec<_>>();
      let lower = values.iter().map(|row| row.1).collect::<Vec<_>>();
      let upper = values.iter().map(|row| row.2).collect::<Vec<_>>();
      backend.add_numeric_prediction_interval_columns(
        &dataset,
        [
          command.target_variable.as_str(),
          lower_target.as_str(),
          upper_target.as_str(),
        ],
        [&estimates, &lower, &upper],
      )
    } else {
      let standard_errors = values.iter().map(|row| row.0).collect::<Vec<_>>();
      backend.add_numeric_prediction_column(&dataset, &command.target_variable, &standard_errors)
    }
    .map_err(|_| RuntimeError::PredictFailed)?;
    let next = self.sync_active_dataset(added)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }

  fn execute_binary_design_prediction(
    &mut self,
    command: &PredictCommand,
    model: BinaryModel,
    design_terms: Vec<LinearDesignTerm>,
    include_intercept: bool,
    coefficients: Vec<f64>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let error = binary_error(model);
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "predict" })?
      .clone();
    if dataset
      .columns
      .iter()
      .any(|column| column.name == command.target_variable)
    {
      return Err(RuntimeError::PredictTargetExists {
        variable: command.target_variable.clone(),
      });
    }

    let expected_coefficients = usize::from(include_intercept)
      + design_terms
        .iter()
        .map(|term| match term {
          LinearDesignTerm::Continuous { .. } => 1,
          LinearDesignTerm::Factor {
            reference, levels, ..
          } => {
            if include_intercept && reference.is_some() {
              levels.len().saturating_sub(1)
            } else {
              levels.len()
            }
          }
        })
        .sum::<usize>();
    if coefficients.len() != expected_coefficients {
      return Err(RuntimeError::PredictFailed);
    }

    let required = design_terms
      .iter()
      .map(|term| match term {
        LinearDesignTerm::Continuous { name } => name.clone(),
        LinearDesignTerm::Factor { variable, .. } => variable.clone(),
      })
      .collect::<Vec<_>>();
    let missing = required
      .iter()
      .filter(|name| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == name.as_str())
      })
      .cloned()
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for term in &design_terms {
      let name = match term {
        LinearDesignTerm::Continuous { name } => name,
        LinearDesignTerm::Factor { variable, .. } => variable,
      };
      let data_type = &dataset
        .columns
        .iter()
        .find(|column| column.name == *name)
        .ok_or(RuntimeError::PredictFailed)?
        .data_type;
      let supported = match term {
        LinearDesignTerm::Continuous { .. } => is_numeric_data_type(data_type),
        LinearDesignTerm::Factor { .. } => {
          is_numeric_data_type(data_type) || matches!(data_type.as_str(), "VARCHAR" | "BOOLEAN")
        }
      };
      if !supported {
        return Err(error(format!(
          "current data has an unsupported type for '{name}'"
        )));
      }
    }

    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      required
        .iter()
        .map(|name| quote_identifier(name))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut values = Vec::with_capacity(usize::try_from(dataset.row_count).unwrap_or(0));
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          let cells = (0..design_terms.len())
            .map(|index| read_owned_cell(row, index, true, "could not read predictor cell"))
            .collect::<Result<Vec<_>, _>>()?;
          let value = match crate::regress_design::expand_current_row(
            &design_terms,
            include_intercept,
            &cells,
          )
          .map_err(|failure| error(failure.to_string()))?
          {
            None => None,
            Some(columns) => {
              let mut index = 0usize;
              let mut xb = if include_intercept {
                let value = coefficients[0];
                index = 1;
                value
              } else {
                0.0
              };
              for column in columns {
                xb += coefficients[index] * column;
                index += 1;
              }
              if !xb.is_finite() {
                return Err(RuntimeError::PredictFailed);
              }
              let prediction = if command.kind == PredictKind::Pr {
                binary_probability(model, xb)
              } else {
                xb
              };
              Some(prediction)
            }
          };
          if value.is_some_and(|value| !value.is_finite()) {
            return Err(RuntimeError::PredictFailed);
          }
          values.push(value);
          Ok(())
        },
        |message| RuntimeError::RegressFailed { message },
      )?;
    if u64::try_from(values.len()).ok() != Some(dataset.row_count) {
      return Err(RuntimeError::PredictFailed);
    }
    let next_dataset = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_numeric_prediction_column(&dataset, &command.target_variable, &values)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next_dataset = self.sync_active_dataset(next_dataset)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next_dataset,
    }))
  }
}

fn binary_factor_contrast_design(
  problem: &tabdat_stats::EstimationProblem,
  design_terms: &[LinearDesignTerm],
  variable: &str,
) -> Result<tabdat_stats::BinaryFactorContrastDesign, String> {
  let mut predictor_column = 0usize;
  let mut selected = None;
  for term in design_terms {
    match term {
      LinearDesignTerm::Continuous { .. } => predictor_column += 1,
      LinearDesignTerm::Factor {
        variable: term_variable,
        reference,
        levels,
      } => {
        let omitted_index = if problem.include_intercept {
          let reference = reference.as_deref().ok_or_else(|| {
            "retained factor design does not record an intercept reference".to_owned()
          })?;
          Some(
            levels
              .iter()
              .position(|level| level.label() == reference)
              .ok_or_else(|| "retained factor reference is absent from its levels".to_owned())?,
          )
        } else {
          None
        };
        if term_variable == variable {
          if selected.is_some() {
            return Err(format!(
              "factor '{variable}' appears more than once in the retained model"
            ));
          }
          let baseline_index = omitted_index.unwrap_or(0);
          let baseline_level = levels
            .get(baseline_index)
            .ok_or_else(|| format!("factor '{variable}' has no retained levels"))?
            .label()
            .to_owned();
          let mut column = predictor_column;
          let predictor_columns = levels
            .iter()
            .enumerate()
            .map(|(index, _)| {
              if Some(index) == omitted_index {
                None
              } else {
                let current = column;
                column += 1;
                Some(current)
              }
            })
            .collect::<Vec<_>>();
          selected = Some(tabdat_stats::BinaryFactorContrastDesign {
            variable: variable.to_owned(),
            baseline_level,
            levels: levels
              .iter()
              .map(|level| level.label().to_owned())
              .collect(),
            predictor_columns,
          });
        }
        predictor_column += levels.len() - usize::from(omitted_index.is_some());
      }
    }
  }
  if predictor_column != problem.predictor_names.len() {
    return Err("retained factor metadata does not match the binary design".into());
  }
  selected
    .ok_or_else(|| format!("factor '{variable}' is not a categorical term in the retained model"))
}

fn binary_continuous_predictor_columns(
  problem: &tabdat_stats::EstimationProblem,
  design_terms: &[LinearDesignTerm],
) -> Result<Vec<usize>, String> {
  if design_terms.is_empty() {
    return Ok((0..problem.predictor_names.len()).collect());
  }
  let mut predictor_column = 0usize;
  let mut continuous_columns = Vec::new();
  for term in design_terms {
    match term {
      LinearDesignTerm::Continuous { name } => {
        if problem
          .predictor_names
          .get(predictor_column)
          .is_none_or(|retained_name| retained_name != name)
        {
          return Err("retained continuous metadata does not match the binary design".into());
        }
        continuous_columns.push(predictor_column);
        predictor_column += 1;
      }
      LinearDesignTerm::Factor {
        variable,
        reference,
        levels,
      } => {
        if levels.len() < 2 {
          return Err(format!(
            "factor '{variable}' has fewer than two retained levels"
          ));
        }
        let omitted_index = if problem.include_intercept {
          let reference = reference.as_deref().ok_or_else(|| {
            format!("retained factor '{variable}' is missing its intercept reference")
          })?;
          Some(
            levels
              .iter()
              .position(|level| level.label() == reference)
              .ok_or_else(|| {
                format!("retained reference for factor '{variable}' is absent from its levels")
              })?,
          )
        } else {
          if reference.is_some() {
            return Err(format!(
              "retained factor '{variable}' has a reference without an intercept"
            ));
          }
          None
        };
        predictor_column += levels.len() - usize::from(omitted_index.is_some());
      }
    }
  }
  if predictor_column != problem.predictor_names.len() {
    return Err("retained design metadata does not match the binary predictors".into());
  }
  if continuous_columns.is_empty() && !design_terms.is_empty() {
    return Err("margins require at least one continuous predictor in the fitted design".into());
  }
  Ok(continuous_columns)
}

fn binary_probability(model: BinaryModel, xb: f64) -> f64 {
  match model {
    BinaryModel::Logit => {
      if xb >= 0.0 {
        1.0 / (1.0 + (-xb).exp())
      } else {
        let exponential = xb.exp();
        exponential / (1.0 + exponential)
      }
    }
    BinaryModel::Probit => tabdat_stats::normal_cdf(xb),
  }
}
