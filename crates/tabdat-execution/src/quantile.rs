use crate::*;

impl Session {
  pub(crate) fn execute_qreg(
    &mut self,
    command: &QregCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    // Match established new-fit behavior: a failed attempt clears stale inference.
    self.estimation = None;
    let error = |message| RuntimeError::QregFailed { message };
    if command.robust {
      return Err(error(
        "only IID-location-error inference is installed; robust/cluster inference is unsupported"
          .into(),
      ));
    }
    let tau = command
      .quantile
      .parse::<f64>()
      .map_err(|_| error("quantile must be finite and from 0.05 to 0.95".into()))?;
    let regression = RegressCommand {
      outcome: command.outcome.clone(),
      predictors: command.predictors.clone(),
      include_intercept: command.include_intercept,
      estimator: RegressEstimator::Ols,
      weight_variable: None,
      robust: false,
      cluster_variable: None,
    };
    let problem = self
      .prepare_regression_problem(&regression, None, false)
      .map_err(|e| error(e.to_string()))?;
    let result = tabdat_stats::fit_quantile_iid(&problem, tau).map_err(|e| error(e.to_string()))?;
    self.estimation = Some(EstimationState::QuantileFit(Box::new(result.clone())));
    Ok(ExecutionResult::Quantile(Box::new(result)))
  }

  pub(crate) fn execute_qreg_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if !matches!(command.kind, PredictKind::Xb | PredictKind::Residuals)
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(RuntimeError::QregFailed {
        message: "predict supports only xb or residuals, without interval/std/saving".into(),
      });
    }
    let Some(EstimationState::QuantileFit(fit)) = &self.estimation else {
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
    let mut required = fit.predictors.clone();
    if command.kind == PredictKind::Residuals {
      required.push(fit.outcome.clone());
    }
    let missing = required
      .into_iter()
      .filter(|v| !dataset.columns.iter().any(|c| c.name == *v))
      .collect::<Vec<_>>();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    let offset = usize::from(fit.include_intercept);
    let coefficients: Vec<_> = fit.coefficients[offset..].iter().map(|b| b.value).collect();
    let prediction = LinearPrediction {
      target_variable: &command.target_variable,
      predictor_names: &fit.predictors,
      predictor_coefficients: &coefficients,
      intercept: fit.include_intercept.then(|| fit.coefficients[0].value),
      outcome_variable: &fit.outcome,
      residuals: command.kind == PredictKind::Residuals,
    };
    let next = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_linear_prediction_column(&dataset, &prediction)
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next = self.sync_active_dataset(next)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }
}
