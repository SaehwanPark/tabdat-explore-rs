use crate::*;

#[derive(Clone, Copy)]
pub(crate) enum CountFamily {
  Poisson,
  Nb2,
}
impl CountFamily {
  fn command(self) -> &'static str {
    match self {
      Self::Poisson => "poisson",
      Self::Nb2 => "nbreg",
    }
  }
  fn maximum(self) -> i128 {
    match self {
      Self::Poisson => 9_007_199_254_740_991,
      Self::Nb2 => 100_000,
    }
  }
  pub(crate) fn error(self, message: impl Into<String>) -> RuntimeError {
    match self {
      Self::Poisson => RuntimeError::PoissonFailed {
        message: message.into(),
      },
      Self::Nb2 => RuntimeError::NbregFailed {
        message: message.into(),
      },
    }
  }
  fn count_error(self) -> RuntimeError {
    self.error(match self {
      Self::Poisson => "outcome must be an exact nonnegative integer at most 2^53-1 on the complete finite estimation sample",
      Self::Nb2 => "NB2 outcome must be an exact nonnegative integer in 0..100000 on the complete finite estimation sample",
    })
  }
  fn prediction_error(self) -> RuntimeError {
    match self {
      Self::Poisson => RuntimeError::PredictPoissonUnsupported,
      Self::Nb2 => RuntimeError::PredictNbregUnsupported,
    }
  }
}

// Validate loaded integer identity BEFORE f64 rounding, jointly with model
// missingness. This does not recover digits a loader already rounded to DOUBLE.
fn exact_count(cell: &CellValue, maximum: i128) -> Option<f64> {
  match cell {
    CellValue::Float(y)
      if y.is_finite() && *y >= 0.0 && *y <= maximum as f64 && y.fract() == 0.0 =>
    {
      Some(if *y == 0.0 { 0.0 } else { *y })
    }
    CellValue::SignedInteger(y) if (0..=maximum).contains(y) => Some(*y as f64),
    CellValue::UnsignedInteger(y) if *y <= maximum as u128 => Some(*y as f64),
    CellValue::Decimal { value, scale, .. } if *value >= 0 => {
      let divisor = 10_i128.checked_pow(u32::from(*scale))?;
      (*value % divisor == 0 && *value / divisor <= maximum).then(|| (*value / divisor) as f64)
    }
    _ => None,
  }
}
impl Session {
  pub(crate) fn count_problem(
    &self,
    family: CountFamily,
    outcome: &str,
    predictors: &[String],
    include_intercept: bool,
    cluster_variable: Option<&str>,
    offset_variable: Option<&str>,
  ) -> Result<(tabdat_stats::EstimationProblem, Vec<f64>), RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: family.command(),
      })?;
    let variables: Vec<&str> = std::iter::once(outcome)
      .chain(predictors.iter().map(String::as_str))
      .chain(offset_variable)
      .collect();
    for name in &variables {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == *name)
        .ok_or_else(|| family.error(format!("unknown variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type) {
        return Err(family.error(format!("numeric variable required: {name}")));
      }
    }
    let mut columns: Vec<String> = variables
      .iter()
      .map(|name| quote_identifier(name))
      .collect();
    if let Some(name) = cluster_variable {
      let column = dataset
        .columns
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| family.error(format!("unknown cluster variable: {name}")))?;
      if !is_numeric_data_type(&column.data_type)
        && !matches!(column.data_type.as_str(), "VARCHAR" | "BOOLEAN")
      {
        return Err(family.error("cluster variable must be numeric, text or boolean"));
      }
      columns.push(quote_identifier(name));
    }
    let query = format!("SELECT {} FROM {ACTIVE_TABLE}", columns.join(", "));
    let (mut total, mut retained, mut response, mut design, mut cluster_groups, mut offsets) = (
      0,
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
    );
    self
      .backend
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: family.command(),
      })?
      .visit_rows(
        &query,
        |row| {
          let index = total;
          total += 1;
          let mut values = Vec::with_capacity(variables.len());
          let mut count = None;
          for j in 0..variables.len() {
            let cell = row.cell(j).map_err(|e| match e {
              CellReadError::Backend(message) => family.error(message),
              _ => family.error("could not read count-model numeric cell"),
            })?;
            if j == 0 {
              count = exact_count(cell, family.maximum());
            }
            match cell_to_f64(cell) {
              Some(value) if value.is_finite() => values.push(value),
              _ => return Ok(()),
            }
          }
          let group = if cluster_variable.is_some() {
            let cell = row
              .cell(variables.len())
              .map_err(|_| family.error("could not read cluster cell"))?;
            let Some(group) = cluster_key(cell).map_err(|message| family.error(message))? else {
              return Ok(());
            };
            Some(group)
          } else {
            None
          };
          // Joint model/cluster exclusions precede finite count-domain rejection.
          let count = count.ok_or_else(|| family.count_error())?;
          retained.push(index);
          response.push(count);
          design.push(values[1..1 + predictors.len()].to_vec());
          offsets.push(if offset_variable.is_some() {
            *values.last().unwrap()
          } else {
            0.0
          });
          if let Some(group) = group {
            cluster_groups.push(group);
          }
          Ok(())
        },
        |message| family.error(message),
      )?;
    if retained.is_empty() {
      return Err(family.error("no complete finite observations"));
    }
    let dropped = total - retained.len();
    let sample = tabdat_stats::EstimationSample::new(
      retained,
      total,
      dropped,
      None,
      cluster_variable.is_some().then_some(cluster_groups),
    )
    .map_err(|e| family.error(e.to_string()))?;
    Ok((
      tabdat_stats::EstimationProblem::new(outcome, predictors.to_vec(), response, design)
        .map_err(|e| family.error(e.to_string()))?
        .with_intercept(include_intercept)
        .with_sample(sample),
      offsets,
    ))
  }

  pub(crate) fn execute_count_probability_table(
    &self,
    tail_from: usize,
  ) -> Result<ExecutionResult, RuntimeError> {
    use tabdat_stats::CountProbabilityLaw;
    let (distribution, problem, means) = match &self.estimation {
      Some(EstimationState::PoissonFit(state)) => (
        CountProbabilityLaw::Poisson,
        &state.problem,
        &state.result.fitted_means,
      ),
      Some(EstimationState::NegativeBinomialFit(state)) => (
        CountProbabilityLaw::Nb2 {
          alpha: state.result.dispersion.alpha,
        },
        &state.problem,
        &state.result.fitted_means,
      ),
      _ => return Err(RuntimeError::EstatCountProbRequiresPriorCount),
    };
    let table =
      tabdat_stats::count_probability_table(distribution, &problem.response, means, tail_from)
        .map_err(|error| RuntimeError::EstatCountProbFailed {
          message: error.to_string(),
        })?;
    Ok(ExecutionResult::CountProbabilities(Box::new(
      CountProbabilityResult {
        outcome: problem.outcome.clone(),
        distribution,
        sample: problem.sample.clone(),
        table,
      },
    )))
  }

  pub(crate) fn execute_count_prediction(
    &mut self,
    command: &PredictCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let (family, problem, estimates) = match &self.estimation {
      Some(EstimationState::PoissonFit(state)) => (
        CountFamily::Poisson,
        &state.problem,
        &state.result.coefficients,
      ),
      Some(EstimationState::NegativeBinomialFit(state)) => {
        (CountFamily::Nb2, &state.problem, &state.result.coefficients)
      }
      _ => return Err(RuntimeError::PredictRequiresPriorRegression),
    };
    if !matches!(command.kind, PredictKind::Xb | PredictKind::MeanCount)
      || command.interval
      || command.std
      || command.saving.is_some()
    {
      return Err(family.prediction_error());
    }
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
    let mut predictors = problem.predictor_names.clone();
    let offset_variable = match &self.estimation {
      Some(EstimationState::PoissonFit(state)) => state.result.offset_variable.as_ref(),
      _ => None,
    };
    if let Some(name) = offset_variable {
      predictors.push(name.clone());
    }
    let missing: Vec<_> = predictors
      .iter()
      .filter(|name| !dataset.columns.iter().any(|c| c.name == **name))
      .cloned()
      .collect();
    if !missing.is_empty() {
      return Err(RuntimeError::PredictUnknownVariable { variables: missing });
    }
    for name in &predictors {
      if dataset
        .columns
        .iter()
        .any(|c| c.name == *name && !is_numeric_data_type(&c.data_type))
      {
        return Err(family.error(format!("prediction requires numeric predictor: {name}")));
      }
    }
    let offset = usize::from(problem.include_intercept);
    let mut coefficients: Vec<_> = estimates[offset..].iter().map(|c| c.value).collect();
    if offset_variable.is_some() {
      coefficients.push(1.0);
    }
    let prediction = LinearPrediction {
      target_variable: &command.target_variable,
      predictor_names: &predictors,
      predictor_coefficients: &coefficients,
      intercept: problem.include_intercept.then(|| estimates[0].value),
      outcome_variable: &problem.outcome,
      residuals: false,
    };
    let added = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::PredictFailed)?
      .add_count_prediction_column(
        &dataset,
        &prediction,
        command.kind == PredictKind::MeanCount,
      )
      .map_err(|_| RuntimeError::PredictFailed)?;
    let next = self.sync_active_dataset(added)?;
    Ok(ExecutionResult::Prediction(PredictionResult {
      dataset: next,
    }))
  }
}
#[test]
fn exact_counts_reject_fractional_decimal_and_wide_integer_rounding_for_each_family() {
  for family in [CountFamily::Poisson, CountFamily::Nb2] {
    let maximum = family.maximum();
    assert_eq!(exact_count(&CellValue::Float(-0.0), maximum), Some(0.0));
    assert_eq!(exact_count(&CellValue::Float(0.5), maximum), None);
    assert_eq!(
      exact_count(&CellValue::SignedInteger(maximum + 1), maximum),
      None
    );
    assert_eq!(
      exact_count(&CellValue::UnsignedInteger(maximum as u128 + 1), maximum),
      None
    );
    assert_eq!(
      exact_count(
        &CellValue::Decimal {
          width: 38,
          scale: 20,
          value: 100_000_000_000_000_000_001
        },
        maximum
      ),
      None
    );
    assert_eq!(
      exact_count(
        &CellValue::Decimal {
          width: 38,
          scale: 20,
          value: 300_000_000_000_000_000_000
        },
        maximum
      ),
      Some(3.0)
    );
  }
}
