use crate::*;
use tabdat_stats::{NlsExpression as Math, NlsModel};

fn nls_error(message: String) -> RuntimeError {
  RuntimeError::NlFailed(message)
}

fn compile_mean(
  expression: &GenerateExpression,
  parameters: &[String],
  predictors: &mut Vec<String>,
  depth: usize,
  nodes: &mut usize,
) -> Result<Math, RuntimeError> {
  if depth > tabdat_stats::MAX_NLS_DEPTH || *nodes >= tabdat_stats::MAX_NLS_NODES {
    return Err(nls_error("expression exceeds node/depth bounds".into()));
  }
  *nodes += 1;
  let mut child =
    |e: &GenerateExpression| compile_mean(e, parameters, predictors, depth + 1, nodes);
  Ok(match expression {
    GenerateExpression::Number(spelling) => {
      let value = spelling
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| nls_error("numeric literals must be finite".into()))?;
      Math::Constant(value)
    }
    GenerateExpression::Identifier(name) => {
      if let Some(index) = parameters.iter().position(|p| p == name) {
        Math::Parameter(index)
      } else {
        let index = match predictors.iter().position(|p| p == name) {
          Some(index) => index,
          None => {
            predictors.push(name.clone());
            predictors.len() - 1
          }
        };
        Math::Predictor(index)
      }
    }
    GenerateExpression::UnaryMinus(e) => Math::Negate(Box::new(child(e)?)),
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let left = Box::new(child(left)?);
      let right = Box::new(child(right)?);
      match operator {
        GenerateBinaryOperator::Add => Math::Add(left, right),
        GenerateBinaryOperator::Subtract => Math::Subtract(left, right),
        GenerateBinaryOperator::Multiply => Math::Multiply(left, right),
        GenerateBinaryOperator::Divide => Math::Divide(left, right),
        _ => {
          return Err(nls_error(
            "only numeric +,-,*,/ operators are supported".into(),
          ));
        }
      }
    }
    GenerateExpression::FunctionCall { name, arguments }
      if name.eq_ignore_ascii_case("exp") && arguments.len() == 1 =>
    {
      Math::Exp(Box::new(child(&arguments[0])?))
    }
    _ => {
      return Err(nls_error(
        "mean expressions support numeric arithmetic and one-argument exp only".into(),
      ));
    }
  })
}

impl Session {
  pub(crate) fn execute_nls(
    &mut self,
    command: &NlCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Like other estimator attempts, failure must not leave an old inference cache.
    self.estimation = None;
    if command.parameter_names.is_empty()
      || command.parameter_names.len() > tabdat_stats::MAX_NLS_PARAMETERS
      || command.start_values.len() != command.parameter_names.len()
    {
      return Err(nls_error(
        "requires 1..32 explicit parameters and matching starts".into(),
      ));
    }
    let start: Vec<f64> = command
      .start_values
      .iter()
      .map(|value| {
        value
          .parse::<f64>()
          .ok()
          .filter(|v| v.is_finite())
          .ok_or_else(|| nls_error("start values must be finite numbers".into()))
      })
      .collect::<Result<_, _>>()?;
    let mut predictors = Vec::new();
    let expression = compile_mean(
      &command.expression,
      &command.parameter_names,
      &mut predictors,
      1,
      &mut 0,
    )?;
    let model = NlsModel {
      outcome: command.outcome.clone(),
      predictor_names: predictors,
      parameter_names: command.parameter_names.clone(),
      expression,
    };
    model.validate().map_err(|e| nls_error(e.to_string()))?;
    // include_intercept is a legacy syntax flag: explicit formula/params ALWAYS
    // control the mean and P; no parameter is inserted or removed by noconstant.
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "nl" })?;
    if dataset.row_count > tabdat_stats::MAX_NLS_OBSERVATIONS as u64 {
      return Err(nls_error(
        "at most10000 candidate rows are supported".into(),
      ));
    }
    let variables: Vec<_> = std::iter::once(model.outcome.as_str())
      .chain(model.predictor_names.iter().map(String::as_str))
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| nls_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(nls_error(format!("numeric variable required: {name}")));
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
    let (mut total, mut retained, mut response, mut data) = (0, Vec::new(), Vec::new(), Vec::new());
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "nl" })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          if total > tabdat_stats::MAX_NLS_OBSERVATIONS {
            return Err(nls_error(
              "at most10000 candidate rows are supported".into(),
            ));
          }
          let mut values = Vec::with_capacity(variables.len());
          for j in 0..variables.len() {
            let cell = row
              .cell(j)
              .map_err(|_| nls_error("could not read numeric estimation cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              return Ok(());
            };
            values.push(value);
          }
          retained.push(index);
          response.push(values[0]);
          data.push(values[1..].to_vec());
          Ok(())
        },
        nls_error,
      )?;
    let sample =
      tabdat_stats::EstimationSample::new(retained, total, total - response.len(), None, None)
        .map_err(|e| nls_error(e.to_string()))?;
    let result = tabdat_stats::fit_nls(&model, &response, &data, &start, &sample, command.robust)
      .map_err(|e| nls_error(e.to_string()))?;
    self.estimation = Some(EstimationState::NonlinearFit(Box::new(result.clone())));
    Ok(ExecutionResult::Nl(Box::new(result)))
  }

  pub(crate) fn execute_nls_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(
      command.kind,
      PredictKind::Xb | PredictKind::TobitMean | PredictKind::Residuals
    ) || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(nls_error("prediction supports only default/xb or mean (nonlinear fitted response), or residuals; no intervals/std/auxiliary modes".into()));
    }
    let Some(EstimationState::NonlinearFit(fit)) = &self.estimation else {
      return Err(nls_error(
        "prediction requires a successful nonlinear fit".into(),
      ));
    };
    let nodes = fit.model.validate().map_err(|e| nls_error(e.to_string()))?;
    if fit.coefficients.len() != fit.model.parameter_names.len()
      || fit
        .coefficients
        .iter()
        .zip(&fit.model.parameter_names)
        .any(|(c, name)| c.name != *name || !c.value.is_finite())
    {
      return Err(nls_error("invalid retained nonlinear parameters".into()));
    }
    let parameters: Vec<_> = fit.coefficients.iter().map(|c| c.value).collect();
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "predict" })?
      .clone();
    if command.target_variable.trim().is_empty() {
      return Err(nls_error("prediction target must be nonempty".into()));
    }
    if dataset
      .columns
      .iter()
      .any(|c| c.name.eq_ignore_ascii_case(&command.target_variable))
    {
      return Err(RuntimeError::PredictTargetExists {
        variable: command.target_variable.clone(),
      });
    }
    if dataset.row_count > tabdat_stats::MAX_NLS_OBSERVATIONS as u64
      || dataset.row_count.saturating_mul(nodes as u64) > tabdat_stats::MAX_NLS_WORK as u64
    {
      return Err(nls_error(
        "prediction current-row/work bounds exceeded".into(),
      ));
    }
    let residuals = command.kind == PredictKind::Residuals;
    let mut names = fit.model.predictor_names.clone();
    if residuals {
      names.push(fit.model.outcome.clone());
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
        return Err(nls_error(format!(
          "prediction requires numeric variable: {name}"
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
    let mut output = Vec::new();
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PredictFailed)?
      .visit_rows(
        &query,
        |row| {
          if output.len() >= tabdat_stats::MAX_NLS_OBSERVATIONS {
            return Err(nls_error(
              "prediction supports at most10000 current rows".into(),
            ));
          }
          let mut values = Vec::with_capacity(names.len());
          for j in 0..names.len() {
            let cell = row
              .cell(j)
              .map_err(|_| nls_error("could not read prediction cell".into()))?;
            let Some(value) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              output.push(None);
              return Ok(());
            };
            values.push(value);
          }
          let y = if residuals { values.pop() } else { None };
          let mean = fit
            .model
            .predict(&parameters, &values)
            .map_err(|e| nls_error(e.to_string()))?;
          let value = y.map_or(mean, |y| y - mean);
          if !value.is_finite() {
            return Err(nls_error("nonrepresentable current residual".into()));
          }
          output.push(Some(value));
          Ok(())
        },
        nls_error,
      )?;
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
