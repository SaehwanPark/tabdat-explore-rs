use crate::*;

fn clogit_error(message: String) -> RuntimeError {
  RuntimeError::ClogitFailed { message }
}

pub(super) struct ChoiceRow {
  pub(super) alternative: i64,
  pub(super) nest: Option<i64>,
  pub(super) predictors: Option<Vec<f64>>,
  pub(super) outcome_finite: bool,
  pub(super) chosen: Option<bool>,
}
pub(super) struct ChoiceFrame {
  pub(super) rows: Vec<ChoiceRow>,
  pub(super) cases: Vec<Vec<usize>>,
  pub(super) levels: Vec<i64>,
  pub(super) alternative_nests: Vec<i64>,
}

impl ChoiceFrame {
  pub(super) fn retained_problem(
    &self,
    outcome: &str,
    group: &str,
    alternative: &str,
    predictors: &[String],
    error: fn(String) -> RuntimeError,
  ) -> Result<tabdat_stats::ConditionalLogitProblem, RuntimeError> {
    let mut keep = vec![false; self.rows.len()];
    let mut retained_cases = Vec::new();
    for case in &self.cases {
      if case
        .iter()
        .any(|&i| self.rows[i].predictors.is_none() || !self.rows[i].outcome_finite)
      {
        continue;
      }
      if case.iter().any(|&i| self.rows[i].chosen.is_none()) {
        return Err(error(
          "chosen outcome must be exactly0 or1 on complete finite choice sets".into(),
        ));
      }
      if case
        .iter()
        .filter(|&&i| self.rows[i].chosen == Some(true))
        .count()
        != 1
      {
        return Err(error(
          "exactly one chosen row is required per retained choice set".into(),
        ));
      }
      for &i in case {
        keep[i] = true;
      }
      retained_cases.push(case);
    }
    let retained = keep
      .iter()
      .enumerate()
      .filter_map(|(i, &keep)| keep.then_some(i))
      .collect::<Vec<_>>();
    if retained.is_empty() {
      return Err(error("no complete finite choice sets".into()));
    }
    let mut retained_position = vec![usize::MAX; self.rows.len()];
    for (position, &original) in retained.iter().enumerate() {
      retained_position[original] = position;
    }
    let cases = retained_cases
      .iter()
      .map(|case| case.iter().map(|&i| retained_position[i]).collect())
      .collect();
    let response = retained
      .iter()
      .map(|&i| f64::from(self.rows[i].chosen.unwrap()))
      .collect();
    let design = retained
      .iter()
      .map(|&i| self.rows[i].predictors.clone().unwrap())
      .collect();
    let alternative_codes = retained.iter().map(|&i| self.rows[i].alternative).collect();
    let total = self.rows.len();
    let excluded = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(retained, total, excluded, None, None)
      .map_err(|e| error(e.to_string()))?;
    let design =
      tabdat_stats::EstimationProblem::new(outcome, predictors.to_vec(), response, design)
        .map_err(|e| error(e.to_string()))?
        .with_intercept(false)
        .with_sample(sample);
    Ok(tabdat_stats::ConditionalLogitProblem {
      design,
      group_variable: group.into(),
      alternative_variable: alternative.into(),
      case_rows: cases,
      alternative_codes,
    })
  }
}

impl Session {
  // Keys and the available-alternative universe are structural: validate them
  // even in sets subsequently excluded for a nonfinite outcome/predictor.
  pub(super) fn read_choice_frame(
    &self,
    group: &str,
    alternative: &str,
    predictors: &[String],
    outcome: Option<&str>,
    nest: Option<&str>,
  ) -> Result<ChoiceFrame, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "clogit" })?;
    let variables = [group, alternative]
      .into_iter()
      .chain(nest)
      .chain(outcome)
      .chain(predictors.iter().map(String::as_str))
      .collect::<Vec<_>>();
    for (index, name) in variables.iter().enumerate() {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == **name)
        .ok_or_else(|| clogit_error(format!("unknown variable: {name}")))?;
      let permitted = is_numeric_data_type(&column.data_type)
        || (index == 0 && matches!(column.data_type.as_str(), "VARCHAR" | "BOOLEAN"));
      if !permitted {
        return Err(clogit_error(if index == 0 {
          format!("group variable must be numeric, text or boolean: {name}")
        } else {
          format!("numeric variable required: {name}")
        }));
      }
    }
    let query = format!(
      "SELECT {} FROM {ACTIVE_TABLE}",
      variables
        .iter()
        .map(|n| quote_identifier(n))
        .collect::<Vec<_>>()
        .join(", ")
    );
    let mut rows = Vec::new();
    let mut cases: Vec<Vec<usize>> = Vec::new();
    let mut keys = BTreeMap::new();
    let mut levels = BTreeSet::new();
    let mut alternative_nests = BTreeMap::new();
    self.backend.as_ref().ok_or(RuntimeError::NoActiveDataset { command: "clogit" })?.visit_rows(
      &query,
      |row| {
        if rows.len() >= 40_000 { return Err(clogit_error("requires at most40000 candidate choice rows".into())); }
        let group_cell = row.cell(0).map_err(|_| clogit_error("could not read group key".into()))?;
        let key = cluster_key(group_cell).map_err(|m| clogit_error(m.into()))?
          .ok_or_else(|| clogit_error("group keys must be nonmissing and finite; a whole choice set cannot otherwise be identified".into()))?;
        let code = exact_integer_code(row.cell(1).map_err(|_| clogit_error("could not read alternative key".into()))?)
          .ok_or_else(|| clogit_error("alternative keys must be exact nonmissing integers within +/- (2^53-1)".into()))?;
        levels.insert(code);
        if levels.len() > 16 { return Err(clogit_error("at most16 alternatives are supported".into())); }
        let nest_code = if nest.is_some() {
          let value = exact_integer_code(row.cell(2).map_err(|_| clogit_error("could not read nest key".into()))?)
            .ok_or_else(|| clogit_error("nest keys must be exact nonmissing integers within +/- (2^53-1)".into()))?;
          if alternative_nests.insert(code,value).is_some_and(|previous| previous != value) {
            return Err(clogit_error("alternative-to-nest mapping must be identical in every candidate set, including excluded sets".into()));
          }
          Some(value)
        } else { None };
        let (outcome_finite, chosen) = if outcome.is_some() {
          let cell = row.cell(2 + usize::from(nest.is_some())).map_err(|_| clogit_error("could not read chosen outcome".into()))?;
          (cell_to_f64(cell).is_some_and(|v| v.is_finite()), exact_integer_code(cell).filter(|v| matches!(v,0|1)).map(|v| v == 1))
        } else { (true, None) };
        let mut values = Vec::with_capacity(predictors.len());
        let mut complete = true;
        for j in 0..predictors.len() {
          let cell = row.cell(2 + usize::from(nest.is_some()) + usize::from(outcome.is_some()) + j)
            .map_err(|_| clogit_error("could not read numeric predictor".into()))?;
          match cell_to_f64(cell).filter(|v| v.is_finite()) {
            Some(value) => values.push(value),
            None => complete = false,
          }
        }
        let case = if let Some(&index) = keys.get(&key) { index } else {
          if cases.len() >= 10_000 { return Err(clogit_error("requires at most10000 candidate choice sets".into())); }
          let index = cases.len(); keys.insert(key,index); cases.push(Vec::new()); index
        };
        cases[case].push(rows.len());
        rows.push(ChoiceRow { alternative: code, nest: nest_code, predictors: complete.then_some(values), outcome_finite, chosen });
        Ok(())
      },
      clogit_error,
    )?;
    if rows.is_empty() {
      return Err(clogit_error("no choice rows are available".into()));
    }
    let levels = levels.into_iter().collect::<Vec<_>>();
    if levels.len() < 2 {
      return Err(clogit_error(
        "requires at least two alternatives per choice set".into(),
      ));
    }
    for case in &cases {
      let codes = case
        .iter()
        .map(|&i| rows[i].alternative)
        .collect::<BTreeSet<_>>();
      if codes.len() != case.len() {
        return Err(clogit_error("duplicate group/alternative key pair".into()));
      }
      if codes.iter().copied().collect::<Vec<_>>() != levels {
        return Err(clogit_error("every choice set must contain the identical complete alternative universe; ragged sets are not supported".into()));
      }
    }
    if nest.is_some() {
      let nests = alternative_nests.values().copied().collect::<BTreeSet<_>>();
      if nests.len() != 2
        || nests
          .iter()
          .any(|n| alternative_nests.values().filter(|code| *code == n).count() < 2)
      {
        return Err(clogit_error(
          "requires exactly two nests with at least two alternatives each".into(),
        ));
      }
    }
    let alternative_nests = alternative_nests.into_values().collect();
    Ok(ChoiceFrame {
      rows,
      cases,
      levels,
      alternative_nests,
    })
  }

  pub(crate) fn execute_clogit(
    &mut self,
    command: &ClogitCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    self.estimation = None;
    if command.predictors.is_empty() || command.predictors.len() > 32 {
      return Err(clogit_error(
        "requires1..32 numeric utility predictors, without a common intercept".into(),
      ));
    }
    let frame = self.read_choice_frame(
      &command.group_variable,
      &command.alternative_variable,
      &command.predictors,
      Some(&command.outcome),
      None,
    )?;
    let problem = frame.retained_problem(
      &command.outcome,
      &command.group_variable,
      &command.alternative_variable,
      &command.predictors,
      clogit_error,
    )?;
    let result = tabdat_stats::fit_conditional_logit(&problem, &Default::default())
      .map_err(|e| clogit_error(e.to_string()))?;
    self.estimation = Some(EstimationState::ConditionalLogit(Box::new(
      ConditionalLogitState {
        problem,
        result: result.clone(),
      },
    )));
    Ok(ExecutionResult::ConditionalLogit(Box::new(result)))
  }

  pub(crate) fn execute_clogit_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    if command.kind != PredictKind::Pr
      || command.interval
      || command.std
      || command.saving.is_some()
      || command.count_cutoff_variable.is_some()
    {
      return Err(clogit_error("conditional-choice prediction supports only predict newvar, pr on complete current choice sets".into()));
    }
    let Some(EstimationState::ConditionalLogit(state)) = &self.estimation else {
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
    let result = &state.result;
    let frame = self.read_choice_frame(
      &result.group_variable,
      &result.alternative_variable,
      &result.predictor_names,
      None,
      None,
    )?;
    if frame.levels != result.alternative_levels {
      return Err(clogit_error(
        "prediction requires the complete retained alternative universe".into(),
      ));
    }
    let mut values = vec![None; frame.rows.len()];
    for case in &frame.cases {
      if case.iter().any(|&i| frame.rows[i].predictors.is_none()) {
        continue;
      }
      let x = case
        .iter()
        .map(|&i| frame.rows[i].predictors.clone().unwrap())
        .collect::<Vec<_>>();
      let codes = case
        .iter()
        .map(|&i| frame.rows[i].alternative)
        .collect::<Vec<_>>();
      let probabilities = tabdat_stats::conditional_logit_predict_set(result, &x, &codes)
        .map_err(|e| clogit_error(e.to_string()))?;
      for (&i, probability) in case.iter().zip(probabilities) {
        values[i] = Some(probability);
      }
    }
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
