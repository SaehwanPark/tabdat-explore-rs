use crate::*;

impl Session {
  pub(crate) fn execute_quantile_bootstrap(
    &self,
    command: &QuantileBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let r = &command.bootstrap.regression;
    let error = |e: String| RuntimeError::BootstrapFailed(e);
    if r.estimator != RegressEstimator::Ols
      || r.weight_variable.is_some()
      || r.robust
      || r.cluster_variable.is_some()
    {
      return Err(error(
        "quantile bootstrap supports only unweighted IID pairs".into(),
      ));
    }
    let quantiles = command
      .quantiles
      .iter()
      .map(|q| q.parse::<f64>())
      .collect::<Result<Vec<_>, _>>()
      .map_err(|_| error("invalid quantiles".into()))?;
    let problem = self
      .prepare_regression_problem(r, None, false)
      .map_err(|e| error(e.to_string()))?;
    tabdat_stats::bootstrap_quantiles(
      &problem,
      &quantiles,
      command.bootstrap.replications,
      command.bootstrap.seed,
    )
    .map(|r| ExecutionResult::QuantileBootstrap(Box::new(r)))
    .map_err(|e| error(e.to_string()))
  }

  pub(crate) fn execute_bootstrap(
    &self,
    command: &BootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let r = &command.regression;
    if r.estimator != RegressEstimator::Ols || r.weight_variable.is_some() || r.robust {
      return Err(RuntimeError::BootstrapFailed(
        "only unweighted pairs OLS is supported; no robust or weighted bootstrap".into(),
      ));
    }
    let problem = self
      .prepare_regression_problem(r, None, r.cluster_variable.is_some())
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    if r.cluster_variable.is_some() {
      return tabdat_stats::bootstrap_ols_clustered(&problem, command.replications, command.seed)
        .map(|r| ExecutionResult::ClusterBootstrap(Box::new(r)))
        .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()));
    }
    tabdat_stats::bootstrap_ols(&problem, command.replications, command.seed)
      .map(|r| ExecutionResult::Bootstrap(Box::new(r)))
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))
  }
}
