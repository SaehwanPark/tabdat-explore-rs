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
    if let Some(time_variable) = &command.time_variable {
      let problem = self.prepare_gee_problem(command)?;
      let sample = problem
        .sample
        .as_ref()
        .ok_or_else(|| error("sample provenance unavailable"))?;
      if sample.dropped_observations > 0 {
        return Err(error(
          "corr(unstructured) requires complete outcome, predictor and entity rows",
        ));
      }
      let dataset = self
        .active_dataset
        .as_ref()
        .ok_or(RuntimeError::NoActiveDataset { command: "xtgee" })?;
      let time_column = dataset
        .columns
        .iter()
        .find(|column| column.name == *time_variable)
        .ok_or_else(|| error("unknown time variable"))?;
      if !is_numeric_data_type(&time_column.data_type) {
        return Err(error("unstructured time variable must be numeric"));
      }
      let query = format!(
        "SELECT {} FROM {ACTIVE_TABLE}",
        quote_identifier(time_variable)
      );
      // Regression preparation and this projection share DuckDB's preserved
      // insertion order, so sample row indices align with the corresponding times.
      let mut all_times = Vec::with_capacity(sample.total_observations);
      self
        .backend
        .as_ref()
        .ok_or_else(|| error("data backend unavailable"))?
        .visit_rows(
          &query,
          |row| {
            if all_times.len() >= 40000 {
              return Err(error("candidate rows<=40000 required"));
            }
            let value = row
              .cell(0)
              .map_err(|_| error("could not read time value"))
              .and_then(|cell| {
                cell_to_f64(cell)
                  .filter(|value| value.is_finite())
                  .ok_or_else(|| error("unstructured time values must be finite and nonmissing"))
              })?;
            // Numeric equality treats -0 and +0 as one period; canonicalize
            // before total-order sorting and binary-searching the shared grid.
            all_times.push(if value == 0.0 { 0.0 } else { value });
            Ok(())
          },
          error,
        )?;
      if all_times.len() != sample.total_observations {
        return Err(error("time rows do not align with the active dataset"));
      }
      let mut time_values = all_times.clone();
      time_values.sort_by(f64::total_cmp);
      time_values.dedup_by(|left, right| *left == *right);
      if !(2..=10).contains(&time_values.len()) {
        return Err(error(
          "corr(unstructured) requires 2..10 shared time values",
        ));
      }
      let period_indices = sample
        .retained_indices
        .iter()
        .map(|&source_row| {
          let value = all_times[source_row];
          time_values
            .binary_search_by(|candidate| candidate.total_cmp(&value))
            .map_err(|_| error("could not map time value to its sorted period"))
        })
        .collect::<Result<Vec<_>, _>>()?;
      let result = tabdat_stats::fit_unstructured_gaussian_gee(
        &problem,
        &command.entity_variable,
        time_variable,
        &time_values,
        &period_indices,
        &Default::default(),
      )
      .map_err(|e| error(e.to_string()))?;
      self.estimation = Some(EstimationState::GaussianGeeUnstructured(Box::new(
        result.clone(),
      )));
      return Ok(ExecutionResult::GaussianGeeUnstructured(Box::new(result)));
    }
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
    let (predictor_names, outcome, beta) = match self.estimation.as_ref() {
      Some(EstimationState::GaussianGee(result)) => (
        result.predictor_names.clone(),
        result.outcome.clone(),
        result
          .working
          .coefficients
          .iter()
          .map(|coefficient| coefficient.value)
          .collect::<Vec<_>>(),
      ),
      Some(EstimationState::GaussianGeeUnstructured(result)) => (
        result.predictor_names.clone(),
        result.outcome.clone(),
        result
          .working
          .coefficients
          .iter()
          .map(|coefficient| coefficient.value)
          .collect::<Vec<_>>(),
      ),
      _ => return Err(RuntimeError::PredictRequiresPriorRegression),
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
    let mut required = predictor_names;
    if residual {
      required.push(outcome);
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
