use crate::count::CountFamily;
use crate::*;

impl Session {
  pub(crate) fn execute_poisson(
    &mut self,
    command: &PoissonCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let family = CountFamily::Poisson;
    if command.robust && command.cluster_variable.is_some() {
      return Err(family.error("cannot combine robust and cluster"));
    }
    let (problem, offsets) = self.count_problem(
      family,
      &command.outcome,
      &command.predictors,
      command.include_intercept,
      command.cluster_variable.as_deref(),
      command.offset_variable.as_deref(),
    )?;
    let mut result = tabdat_stats::fit_poisson_with_offset(
      &problem,
      &offsets,
      command.robust,
      command.cluster_variable.as_deref(),
    )
    .map_err(|e| family.error(e.to_string()))?;
    result.offset_variable = command.offset_variable.clone();
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
