use crate::count::{CountFamily, CountFixedTerm};
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
    let fixed_term = CountFixedTerm::from_options(
      family,
      command.offset_variable.as_deref(),
      command.exposure_variable.as_deref(),
    )?;
    let (problem, offsets) = self.count_problem(
      family,
      &command.outcome,
      &command.predictors,
      command.include_intercept,
      command.cluster_variable.as_deref(),
      fixed_term,
    )?;
    let result = if let Some(CountFixedTerm::Exposure(exposure_variable)) = fixed_term {
      tabdat_stats::fit_poisson_with_exposure(
        &problem,
        &offsets,
        exposure_variable,
        command.robust,
        command.cluster_variable.as_deref(),
      )
    } else {
      tabdat_stats::fit_poisson_with_offset(
        &problem,
        &offsets,
        command.offset_variable.as_deref(),
        command.robust,
        command.cluster_variable.as_deref(),
      )
    }
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
