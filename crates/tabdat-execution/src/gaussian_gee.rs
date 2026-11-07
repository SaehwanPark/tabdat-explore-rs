use crate::*;

fn error(message: impl Into<String>) -> RuntimeError {
  RuntimeError::XtGeeFailed {
    message: message.into(),
  }
}

impl Session {
  pub(crate) fn prepare_gee_problem(
    &mut self,
    command: &XtGeeCommand,
  ) -> Result<tabdat_stats::EstimationProblem, RuntimeError> {
    let k = command.predictors.len() + 1;
    if !(2..=33).contains(&k) {
      return Err(error("requires1..32 numeric predictors"));
    }
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "xtgee" })?;
    let group = dataset
      .columns
      .iter()
      .find(|c| c.name == command.entity_variable)
      .ok_or_else(|| error("unknown group variable"))?;
    if !is_numeric_data_type(&group.data_type)
      && !matches!(group.data_type.as_str(), "VARCHAR" | "BOOLEAN")
    {
      return Err(error("group variable must be numeric, text or boolean"));
    }
    // Bound all candidate rows/groups before materializing the joint-finite design.
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
          let key = cluster_key(row.cell(0).map_err(|_| error("could not read group key"))?)
            .map_err(error)?;
          if let Some(key) = key {
            groups.insert(key);
          }
          if groups.len() > 10000 {
            return Err(error("requires at most10000 candidate groups"));
          }
          Ok(())
        },
        error,
      )?;
    let regression = RegressCommand {
      outcome: command.outcome.clone(),
      predictors: command.predictors.clone(),
      estimator: RegressEstimator::Ols,
      weight_variable: None,
      robust: false,
      cluster_variable: Some(command.entity_variable.clone()),
      include_intercept: true,
    };
    let problem = self
      .prepare_regression_problem(&regression, None, true)
      .map_err(|e| error(e.to_string()))?
      .with_intercept_name("_cons");
    Ok(problem)
  }

  pub(crate) fn execute_xtgee(
    &mut self,
    command: &XtGeeCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let problem = self.prepare_gee_problem(command)?;
    let result =
      tabdat_stats::fit_gaussian_gee(&problem, &command.entity_variable, &Default::default())
        .map_err(|e| error(e.to_string()))?;
    // Replace state only after all data, numerical and inference checks succeed.
    self.estimation = Some(EstimationState::GaussianGee(Box::new(result.clone())));
    Ok(ExecutionResult::GaussianGee(Box::new(result)))
  }

  pub(crate) fn execute_gaussian_gee_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || !matches!(command.kind, PredictKind::Xb | PredictKind::Residuals)
    {
      return Err(error(
        "supports marginal mean/xb or residuals only; no intervals, std, count or saved draws",
      ));
    }
    let Some(EstimationState::GaussianGee(result)) = &self.estimation else {
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
    let beta = result
      .working
      .coefficients
      .iter()
      .map(|c| c.value)
      .collect::<Vec<_>>();
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
          let value = if residual {
            cell_to_f64(
              row
                .cell(beta.len() - 1)
                .map_err(|_| error("could not read outcome"))?,
            )
            .filter(|v| v.is_finite())
            .map(|y| y - xb)
          } else {
            Some(xb)
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
