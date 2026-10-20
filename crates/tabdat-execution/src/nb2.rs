use crate::count::CountFamily;
use crate::*;

impl Session {
  pub(crate) fn execute_nbreg(
    &mut self,
    command: &NbregCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    let family = CountFamily::Nb2;
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
    let result = tabdat_stats::fit_nb2_with_offset(
      &problem,
      &offsets,
      command.offset_variable.as_deref(),
      command.robust,
      command.cluster_variable.as_deref(),
    )
    .map_err(|e| family.error(e.to_string()))?;
    self.estimation = Some(EstimationState::NegativeBinomialFit(Box::new(
      NegativeBinomialFitState {
        problem,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::Nbreg(Box::new(result)))
  }
  pub(crate) fn execute_nbreg_margins(
    &self,
    state: &NegativeBinomialFitState,
  ) -> Result<ExecutionResult, RuntimeError> {
    tabdat_stats::nb2_average_marginal_effects(&state.problem, &state.result)
      .map(|result| ExecutionResult::NbregMarginalEffects(Box::new(result)))
      .map_err(|e| RuntimeError::EstatMarginsFailed {
        message: e.to_string(),
      })
  }
}
