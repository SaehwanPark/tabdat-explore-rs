use crate::count::CountFamily;
use crate::*;

impl Session {
  pub(crate) fn execute_poisson(
    &mut self,
    command: &PoissonCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let family = CountFamily::Poisson;
    if command.cluster_variable.is_some() {
      return Err(
        family.error("clustered Poisson is not implemented; robust assumes independent rows"),
      );
    }
    let problem = self.count_problem(
      family,
      &command.outcome,
      &command.predictors,
      command.include_intercept,
    )?;
    let result = tabdat_stats::fit_poisson(&problem, command.robust)
      .map_err(|e| family.error(e.to_string()))?;
    self.estimation = Some(EstimationState::PoissonFit(Box::new(PoissonFitState {
      problem,
      result: result.clone(),
    })));
    Ok(ExecutionResult::Poisson(Box::new(result)))
  }
  pub(crate) fn execute_poisson_margins(
    &self,
    state: &PoissonFitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    tabdat_stats::poisson_average_marginal_effects(&state.problem, &state.result)
      .map(|result| ExecutionResult::PoissonMarginalEffects(Box::new(result)))
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })
  }
}
