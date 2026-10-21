use crate::*;

impl Session {
  pub(crate) fn execute_bootstrap(
    &self,
    command: &BootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let r = &command.regression;
    if r.estimator != RegressEstimator::Ols
      || r.weight_variable.is_some()
      || r.robust
      || r.cluster_variable.is_some()
    {
      return Err(RuntimeError::BootstrapFailed(
        "only unweighted IID pairs OLS is supported; no robust/cluster/weighted bootstrap".into(),
      ));
    }
    let problem = self
      .prepare_regression_problem(r, None)
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    tabdat_stats::bootstrap_ols(&problem, command.replications, command.seed)
      .map(|r| ExecutionResult::Bootstrap(Box::new(r)))
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))
  }
}
