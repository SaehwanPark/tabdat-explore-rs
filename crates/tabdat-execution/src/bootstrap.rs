use crate::*;

impl Session {
  pub(crate) fn execute_normal_poisson_bootstrap(
    &self,
    command: &PoissonBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let p = command.predictors.len();
    if !(1..=14).contains(&p) {
      return Err(RuntimeError::BootstrapFailed(
        "normal Poisson RE requires 1 to 14 slopes".into(),
      ));
    }
    self
      .validate_xtpoisson_group_structure(&command.cluster_variable, p + 2, 176)
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let (problem, _) = self
      .count_problem(
        crate::count::CountFamily::NormalPoisson,
        &command.outcome,
        &command.predictors,
        true,
        Some(&command.cluster_variable),
        None,
      )
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let result = tabdat_stats::bootstrap_normal_poisson_clustered(
      &problem.with_intercept_name("_cons"),
      &command.cluster_variable,
      command.replications,
      command.seed,
    )
    .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    Ok(ExecutionResult::NormalPoissonBootstrap(Box::new(result)))
  }

  pub(crate) fn execute_gamma_poisson_bootstrap(
    &self,
    command: &PoissonBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let p = command.predictors.len();
    if !(1..=14).contains(&p) {
      return Err(RuntimeError::BootstrapFailed(
        "shared-gamma Poisson RE requires 1 to 14 slopes".into(),
      ));
    }
    self
      .validate_xtpoisson_group_structure(&command.cluster_variable, p + 2, 1)
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let (problem, _) = self
      .count_problem(
        crate::count::CountFamily::GammaPoisson,
        &command.outcome,
        &command.predictors,
        true,
        Some(&command.cluster_variable),
        None,
      )
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let result = tabdat_stats::bootstrap_gamma_poisson_clustered(
      &problem.with_intercept_name("_cons"),
      &command.cluster_variable,
      command.replications,
      command.seed,
    )
    .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    Ok(ExecutionResult::GammaPoissonBootstrap(Box::new(result)))
  }

  pub(crate) fn execute_conditional_poisson_bootstrap(
    &self,
    command: &PoissonBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let k = command.predictors.len();
    if !(1..=16).contains(&k) {
      return Err(RuntimeError::BootstrapFailed(
        "conditional Poisson FE requires 1 to 16 slopes".into(),
      ));
    }
    self
      .validate_xtpoisson_group_structure(&command.cluster_variable, k, 1)
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let (problem, _) = self
      .count_problem(
        crate::count::CountFamily::ConditionalPoisson,
        &command.outcome,
        &command.predictors,
        false,
        Some(&command.cluster_variable),
        None,
      )
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let result = tabdat_stats::bootstrap_conditional_poisson_clustered(
      &problem,
      &command.cluster_variable,
      command.replications,
      command.seed,
    )
    .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    Ok(ExecutionResult::ConditionalPoissonBootstrap(Box::new(
      result,
    )))
  }

  pub(crate) fn execute_poisson_bootstrap(
    &self,
    command: &PoissonBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let (problem, _) = self
      .count_problem(
        crate::count::CountFamily::Poisson,
        &command.outcome,
        &command.predictors,
        true,
        Some(&command.cluster_variable),
        None,
      )
      .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    let mut result =
      tabdat_stats::bootstrap_poisson_clustered(&problem, command.replications, command.seed)
        .map_err(|e| RuntimeError::BootstrapFailed(e.to_string()))?;
    result.cluster_variable = Some(command.cluster_variable.clone());
    Ok(ExecutionResult::ClusterPoissonBootstrap(Box::new(result)))
  }

  pub(crate) fn execute_quantile_bootstrap(
    &self,
    command: &QuantileBootstrapCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let r = &command.bootstrap.regression;
    let error = RuntimeError::BootstrapFailed;
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
    let result = match command.contrast.as_deref() {
      Some(parameter) => tabdat_stats::bootstrap_quantiles_with_contrast(
        &problem,
        &quantiles,
        command.bootstrap.replications,
        command.bootstrap.seed,
        parameter,
      ),
      None => tabdat_stats::bootstrap_quantiles(
        &problem,
        &quantiles,
        command.bootstrap.replications,
        command.bootstrap.seed,
      ),
    };
    result
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
