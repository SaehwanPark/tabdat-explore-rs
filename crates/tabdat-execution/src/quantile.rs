use crate::*;

impl Session {
  pub(crate) fn execute_qreg_plot(
    &self,
    command: &QregPlotCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let error = |message| RuntimeError::QregPlotFailed { message };
    let path = command
      .saving
      .as_deref()
      .map(Path::new)
      .ok_or_else(|| error("requires explicit saving(\"file.svg\") noopen".into()))?;
    if command.open_artifact {
      return Err(error(
        "automatic opening is not installed; add noopen".into(),
      ));
    }
    tabdat_io::validate_svg_path(path).map_err(|e| error(e.to_string()))?;
    let regression = &command.bootstrap.regression;
    if regression.estimator != RegressEstimator::Ols
      || regression.weight_variable.is_some()
      || regression.robust
      || regression.cluster_variable.is_some()
    {
      return Err(error(
        "supports unweighted IID row pairs only; robust, weighted and clustered requests are unsupported"
          .into(),
      ));
    }
    let quantiles = command
      .quantiles
      .iter()
      .map(|value| value.parse::<f64>())
      .collect::<Result<Vec<_>, _>>()
      .map_err(|_| error("quantiles must be finite numeric values".into()))?;
    let problem = self
      .prepare_regression_problem(regression, None, false)
      .map_err(|e| match e {
        RuntimeError::NoActiveDataset { .. } => RuntimeError::NoActiveDataset {
          command: "qregplot",
        },
        other => error(other.to_string()),
      })?;
    let analysis = tabdat_stats::quantile_slope_bands(
      &problem,
      &quantiles,
      &command.coefficient,
      command.bootstrap.replications,
      command.bootstrap.seed,
    )
    .map_err(|e| error(e.to_string()))?;
    let rendered = tabdat_reporting::quantile_slope_svg::render_quantile_slope_svg(
      &analysis,
      command.title.as_deref(),
    )
    .map_err(error)?;
    tabdat_io::publish_svg(path, &rendered.svg).map_err(|e| match e {
      RuntimeError::PlotWriteFailed { path, message } => {
        error(format!("could not publish {}: {message}", path.display()))
      }
      other => error(other.to_string()),
    })?;
    Ok(ExecutionResult::QuantileSlopePlot(Box::new(
      QuantileSlopePlotResult {
        analysis,
        path: path.to_path_buf(),
        format: "svg".into(),
        width: rendered.width,
        height: rendered.height,
      },
    )))
  }

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
      exposure_variable: None,
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
