use crate::*;

#[derive(Clone, Copy)]
enum PredictionLink {
  Linear,
  FiniteLinear,
  Exponential,
  Logistic,
  /// Binary probit inverse link through the registered, engine-validated
  /// Gaussian UDF, so a probability column is bit-identical to the fit's
  /// own probabilities under the same linear index.
  Gaussian,
}

impl DuckDbBackend {
  pub fn filter_rows(
    &mut self,
    dataset: &DatasetInfo,
    predicate: &AssertExpression,
    keep: bool,
  ) -> Result<DatasetInfo, DataError> {
    let predicate = compile_assert_expression(predicate, dataset)?;
    // Drop is the exact complement of keep, including UNKNOWN predicates.
    let selection = if keep {
      format!("COALESCE(({predicate}), FALSE)")
    } else {
      format!("NOT COALESCE(({predicate}), FALSE)")
    };
    let used_names = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<HashSet<_>>();
    let order = quote_identifier(&unique_internal_name(
      "__tabdat_filter_row_order",
      &used_names,
    ));
    self.drop_staging();
    let query = format!(
      "CREATE TEMP TABLE {STAGING_TABLE} AS \
       SELECT * EXCLUDE ({order}) FROM (\
         SELECT row_number() OVER () AS {order}, * FROM {ACTIVE_TABLE}\
       ) AS filter_source WHERE {selection} ORDER BY {order}"
    );
    if self.connection.execute_batch(&query).is_err() {
      self.drop_staging();
      return Err(DataError);
    }
    let next = (|| {
      let columns = self.staged_columns()?;
      let row_count = self.staged_row_count()?;
      if columns != dataset.columns || row_count > dataset.row_count {
        return Err(DataError);
      }
      Ok(DatasetInfo {
        source: dataset.source.clone(),
        row_count,
        columns,
        execution_mode: dataset.execution_mode,
        lazy_engine: dataset.lazy_engine,
      })
    })();
    let dataset = match next {
      Ok(dataset) => dataset,
      Err(error) => {
        self.drop_staging();
        return Err(error);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }
    Ok(dataset)
  }
  pub fn project_columns(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let select_list = variables
      .iter()
      .map(|variable| quote_identifier(variable))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn collapse(
    &mut self,
    dataset: &DatasetInfo,
    statistic: CollapseStatistic,
    variables: &[String],
    groups: &[String],
  ) -> Result<DatasetInfo, DataError> {
    let statistic_name = collapse_statistic_name(statistic);
    let group_sql = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>()
      .join(", ");
    let aggregate_sql = variables
      .iter()
      .map(|variable| {
        let output_name = format!("{statistic_name}_{variable}");
        format!(
          "{statistic_name}({}) AS {}",
          quote_identifier(variable),
          quote_identifier(&output_name)
        )
      })
      .collect::<Vec<_>>()
      .join(", ");
    let order_sql = groups
      .iter()
      .map(|group| format!("{} ASC NULLS LAST", quote_identifier(group)))
      .collect::<Vec<_>>()
      .join(", ");

    self.drop_staging();
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {group_sql}, {aggregate_sql} FROM {ACTIVE_TABLE} GROUP BY {group_sql} ORDER BY {order_sql}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let expected_names = groups
      .iter()
      .cloned()
      .chain(
        variables
          .iter()
          .map(|variable| format!("{statistic_name}_{variable}")),
      )
      .collect::<Vec<_>>();
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(DataError);
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn rename_column(
    &mut self,
    dataset: &DatasetInfo,
    old_name: &str,
    new_name: &str,
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let select_list = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == old_name {
          format!(
            "{} AS {}",
            quote_identifier(&column.name),
            quote_identifier(new_name)
          )
        } else {
          quote_identifier(&column.name)
        }
      })
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let expected_names = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == old_name {
          new_name.to_owned()
        } else {
          column.name.clone()
        }
      })
      .collect::<Vec<_>>();
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(DataError);
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn sort_rows(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
  ) -> Result<DatasetInfo, DataError> {
    let directions = vec![false; variables.len()];
    self.sort_rows_directed(dataset, variables, &directions)
  }
  pub fn sort_rows_directed(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    directions: &[bool],
  ) -> Result<DatasetInfo, DataError> {
    if variables.len() != directions.len() {
      return Err(DataError);
    }
    self.drop_staging();
    let mut ordinal_name = "__tabdat_sort_ordinal".to_owned();
    while dataset
      .columns
      .iter()
      .any(|column| column.name.eq_ignore_ascii_case(&ordinal_name))
    {
      ordinal_name.push('_');
    }
    let quoted_ordinal = quote_identifier(&ordinal_name);
    let order_by = variables
      .iter()
      .zip(directions)
      .map(|(variable, is_descending)| {
        let direction = if *is_descending { "DESC" } else { "ASC" };
        format!("{} {direction} NULLS LAST", quote_identifier(variable))
      })
      .chain(std::iter::once(format!("{quoted_ordinal} ASC")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * EXCLUDE ({quoted_ordinal}) FROM (SELECT row_number() OVER () AS {quoted_ordinal}, * FROM {ACTIVE_TABLE}) AS __tabdat_sort_rows ORDER BY {order_by}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if columns != dataset.columns {
      self.drop_staging();
      return Err(DataError);
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if row_count != dataset.row_count {
      self.drop_staging();
      return Err(DataError);
    }
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn recode_columns(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    rules: &[RecodeRule],
    target: &RecodeTarget,
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();

    let generated = match target {
      RecodeTarget::Generate { variables } => Some(variables),
      RecodeTarget::Replace => None,
    };
    let targets = generated.cloned().unwrap_or_else(|| variables.to_vec());
    let expressions = variables
      .iter()
      .zip(targets.iter())
      .map(|(source, target)| {
        let source_column = dataset
          .columns
          .iter()
          .find(|column| column.name == *source)
          .ok_or(DataError)?;
        let expression = compile_recode_case(source_column, rules)?;
        Ok::<_, DataError>((source.clone(), target.clone(), expression))
      })
      .collect::<Result<Vec<_>, _>>()?;

    let mut select_items = dataset
      .columns
      .iter()
      .map(|column| {
        let expression = expressions
          .iter()
          .find(|(source, _, _)| source == &column.name)
          .map(|(_, _, expression)| expression);
        match (target, expression) {
          (RecodeTarget::Replace, Some(expression)) => {
            format!("{expression} AS {}", quote_identifier(&column.name))
          }
          _ => quote_identifier(&column.name),
        }
      })
      .collect::<Vec<_>>();
    if let Some(generated) = generated {
      select_items.extend(expressions.iter().map(|(_, target, expression)| {
        debug_assert!(generated.iter().any(|variable| variable == target));
        format!("{expression} AS {}", quote_identifier(target))
      }));
    }

    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {} FROM {ACTIVE_TABLE}",
        select_items.join(", ")
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let mut expected_names = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if let Some(generated) = generated {
      expected_names.extend(generated.iter().cloned());
    }
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(DataError);
    }

    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if row_count != dataset.row_count || self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn generate_column(
    &mut self,
    dataset: &DatasetInfo,
    variable: &str,
    expression: &AssertExpression,
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let expression_sql = compile_assert_expression(expression, dataset)?;
    let target_sql = quote_identifier(variable);
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT *, {expression_sql} AS {target_sql} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn add_linear_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    prediction: &LinearPrediction<'_>,
  ) -> Result<DatasetInfo, DataError> {
    self.add_prediction_column(dataset, prediction, PredictionLink::Linear)
  }

  /// Apply a stable sigmoid or finite linear index through the same staging path.
  pub fn add_logit_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    prediction: &LinearPrediction<'_>,
    probability: bool,
  ) -> Result<DatasetInfo, DataError> {
    if prediction.residuals {
      return Err(DataError);
    }
    let link = if probability {
      PredictionLink::Logistic
    } else {
      PredictionLink::FiniteLinear
    };
    self.add_prediction_column(dataset, prediction, link)
  }

  /// Apply the normal inverse link or finite linear index through the same
  /// staging path as the logit prediction.
  pub fn add_probit_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    prediction: &LinearPrediction<'_>,
    probability: bool,
  ) -> Result<DatasetInfo, DataError> {
    if prediction.residuals {
      return Err(DataError);
    }
    let link = if probability {
      PredictionLink::Gaussian
    } else {
      PredictionLink::FiniteLinear
    };
    self.add_prediction_column(dataset, prediction, link)
  }

  /// Mean-count and finite-xb predictions share the established atomic stage.
  pub fn add_poisson_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    prediction: &LinearPrediction<'_>,
    mean_count: bool,
  ) -> Result<DatasetInfo, DataError> {
    if prediction.residuals {
      return Err(DataError);
    }
    let link = if mean_count {
      PredictionLink::Exponential
    } else {
      PredictionLink::FiniteLinear
    };
    self.add_prediction_column(dataset, prediction, link)
  }

  fn add_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    prediction: &LinearPrediction<'_>,
    link: PredictionLink,
  ) -> Result<DatasetInfo, DataError> {
    if prediction.predictor_names.len() != prediction.predictor_coefficients.len() {
      return Err(DataError);
    }

    let mut terms = Vec::with_capacity(
      prediction.predictor_names.len() + usize::from(prediction.intercept.is_some()),
    );
    if let Some(intercept) = prediction.intercept {
      terms.push(format!(
        "CAST({} AS DOUBLE)",
        finite_float_sql_literal(intercept)?
      ));
    }
    for (predictor, coefficient) in prediction
      .predictor_names
      .iter()
      .zip(prediction.predictor_coefficients)
    {
      terms.push(format!(
        "CAST({} AS DOUBLE) * CAST({} AS DOUBLE)",
        finite_float_sql_literal(*coefficient)?,
        quote_identifier(predictor),
      ));
    }
    let prediction_sql = if terms.is_empty() {
      "0.0".to_owned()
    } else {
      terms.join(" + ")
    };
    let expression_sql = if prediction.residuals {
      format!(
        "CAST({} AS DOUBLE) - ({prediction_sql})",
        quote_identifier(prediction.outcome_variable)
      )
    } else {
      prediction_sql
    };
    let expression_sql = match link {
      PredictionLink::Linear => expression_sql,
      PredictionLink::FiniteLinear => {
        format!("CASE WHEN isfinite(({expression_sql})) THEN ({expression_sql}) ELSE NULL END")
      }
      // Guard exp before evaluation: overflowing means are NULL, not an
      // overflowing SQL expression that aborts publication. Underflow is zero.
      PredictionLink::Exponential => format!(
        "CASE WHEN isfinite(({expression_sql})) AND ({expression_sql}) <= {} THEN exp(({expression_sql})) ELSE NULL END",
        finite_float_sql_literal(f64::MAX.ln())?
      ),
      PredictionLink::Logistic => format!(
        "CASE WHEN NOT isfinite(({expression_sql})) THEN NULL WHEN ({expression_sql}) >= 0 THEN 1.0 / (1.0 + exp(-({expression_sql}))) ELSE exp(({expression_sql})) / (1.0 + exp(({expression_sql}))) END"
      ),
      PredictionLink::Gaussian => format!(
        "CASE WHEN NOT isfinite(({expression_sql})) THEN NULL ELSE tabdat_gauss_phi(({expression_sql})) END"
      ),
    };
    let target_sql = quote_identifier(prediction.target_variable);

    self.drop_staging();
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT *, {expression_sql} AS {target_sql} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let expected_names = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .chain(std::iter::once(prediction.target_variable));
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names)
      || columns
        .last()
        .is_none_or(|column| column.data_type != "DOUBLE")
    {
      self.drop_staging();
      return Err(DataError);
    }

    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if row_count != dataset.row_count || self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
  pub fn replace_column(
    &mut self,
    dataset: &DatasetInfo,
    variable: &str,
    target_type: &str,
    expression: &AssertExpression,
    condition: Option<&AssertExpression>,
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let expression_sql = if matches!(expression, AssertExpression::Null) {
      format!("CAST(NULL AS {target_type})")
    } else {
      compile_assert_expression(expression, dataset)?
    };
    let replacement_sql = if let Some(condition) = condition {
      let condition_sql = compile_assert_expression(condition, dataset)?;
      format!(
        "CASE WHEN {condition_sql} THEN {expression_sql} ELSE {} END",
        quote_identifier(variable)
      )
    } else {
      expression_sql
    };
    let select_list = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == variable {
          format!("{replacement_sql} AS {}", quote_identifier(&column.name))
        } else {
          quote_identifier(&column.name)
        }
      })
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(DataError);
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(DataError);
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(DataError);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }
}
