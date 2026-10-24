use crate::*;

fn truncreg_error(message: String) -> RuntimeError {
  RuntimeError::TruncregFailed(message)
}

impl Session {
  pub(crate) fn execute_truncreg(
    &mut self,
    command: &TruncregCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // A supported new fit attempt invalidates stale inference, not data/labels.
    self.estimation = None;
    let lower = command
      .lower_limit
      .parse::<f64>()
      .ok()
      .filter(|v| v.is_finite())
      .ok_or_else(|| truncreg_error("ll() must be a finite numeric lower limit".into()))?;
    let k = command.predictors.len() + usize::from(command.include_intercept);
    if !(1..=tabdat_stats::MAX_TRUNCATED_GAUSSIAN_PARAMETERS).contains(&k) {
      return Err(truncreg_error(
        "requires1..32 location parameters including any intercept".into(),
      ));
    }
    let max_rows = tabdat_stats::MAX_TRUNCATED_GAUSSIAN_OBSERVATIONS
      .min(tabdat_stats::MAX_TRUNCATED_GAUSSIAN_WORK / ((k + 1) * (k + 1)));
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "truncreg",
      })?;
    let names: Vec<&str> = std::iter::once(command.outcome.as_str())
      .chain(command.predictors.iter().map(String::as_str))
      .collect();
    for name in &names {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| truncreg_error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(truncreg_error(format!("numeric variable required: {name}")));
      }
    }
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      names
        .iter()
        .map(|n| quote_identifier(n))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let (mut total, mut retained, mut response, mut design) =
      (0, Vec::new(), Vec::new(), Vec::new());
    self.backend.as_ref().ok_or(RuntimeError::NoActiveDataset{command:"truncreg"})?.visit_rows(&query,|row| {
      let index=total;total+=1;let mut values=Vec::with_capacity(names.len());
      // Joint-finite eligibility is decided BEFORE support. A below-limit y on
      // a row with missing X is excluded, not misclassified as a retained error.
      for j in 0..names.len() {
        let cell=row.cell(j).map_err(|_|truncreg_error("could not read estimation cell".into()))?;
        let Some(v)=cell_to_f64(cell).filter(|v|v.is_finite())else{return Ok(());};values.push(v);
      }
      if values[0]<=lower{return Err(truncreg_error("all retained outcomes must be strictly above the truncation limit; no censoring or silent support exclusion".into()));}
      if retained.len()>=max_rows{return Err(truncreg_error(format!("complete finite sample exceeds bounded observation/information work limit ({max_rows} rows)")));}
      retained.push(index);response.push(values[0]);design.push(values[1..].to_vec());Ok(())
    },truncreg_error)?;
    if retained.is_empty() {
      return Err(truncreg_error("no complete finite observations".into()));
    }
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, dropped, None, None)
      .map_err(|e| truncreg_error(e.to_string()))?;
    let problem = tabdat_stats::EstimationProblem::new(
      command.outcome.clone(),
      command.predictors.clone(),
      response,
      design,
    )
    .map_err(|e| truncreg_error(e.to_string()))?
    .with_intercept(command.include_intercept)
    .with_sample(sample);
    let result = tabdat_stats::fit_truncated_gaussian(
      &problem,
      lower,
      &tabdat_stats::TruncatedGaussianOptions::default(),
    )
    .map_err(|e| truncreg_error(e.to_string()))?;
    self.estimation = Some(EstimationState::TruncatedGaussianFit(Box::new(
      result.clone(),
    )));
    Ok(ExecutionResult::TruncatedGaussian(Box::new(result)))
  }

  pub(crate) fn execute_truncreg_margins(
    &self,
    state: &tabdat_stats::TruncatedGaussianResult,
  ) -> Result<ExecutionResult, RuntimeError> {
    tabdat_stats::truncated_gaussian_average_marginal_effects(state)
      .map(|r| ExecutionResult::TruncatedGaussianMarginalEffects(Box::new(r)))
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })
  }

  pub(crate) fn execute_truncreg_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let kind=match command.kind {
      PredictKind::Xb=>tabdat_stats::TruncatedGaussianPredictionKind::Latent,
      PredictKind::Pr=>tabdat_stats::TruncatedGaussianPredictionKind::RetentionProbability,
      PredictKind::TobitMean=>tabdat_stats::TruncatedGaussianPredictionKind::ConditionalMean,
      _=>return Err(truncreg_error("prediction supports only xb, pr (population retention probability) or mean (conditional truncated mean)".into())),
    };
    if command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
      || command.level.parse::<f64>().ok() != Some(95.0)
    {
      return Err(truncreg_error(
        "prediction supports point xb/pr/mean without auxiliary options".into(),
      ));
    }
    let Some(EstimationState::TruncatedGaussianFit(state)) = &self.estimation else {
      return Err(truncreg_error(
        "prediction requires a prior successful lower-truncated Gaussian fit".into(),
      ));
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
    if dataset.row_count > tabdat_stats::MAX_TRUNCATED_GAUSSIAN_OBSERVATIONS as u64 {
      return Err(truncreg_error(
        "prediction supports at most10000 current rows".into(),
      ));
    }
    let names = &state.predictor_names;
    let missing: Vec<_> = names
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in names {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(truncreg_error(format!(
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
              .map_err(|_| truncreg_error("could not read prediction cell".into()))?;
            let Some(v) = cell_to_f64(cell).filter(|v| v.is_finite()) else {
              values.push(None);
              return Ok(());
            };
            predictors.push(v);
          }
          // Evaluate ONLY the requested mode: an underflowed Mills denominator is
          // irrelevant to finite xb and represented population probability0.
          let value = tabdat_stats::truncated_gaussian_predict_row(state, &predictors, kind)
            .map_err(|e| truncreg_error(e.to_string()))?;
          values.push(Some(value));
          Ok(())
        },
        truncreg_error,
      )?;
    // No publication until all scores succeed; the existing typed adapter owns
    // target/length/finiteness checks and transactional column publication.
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
