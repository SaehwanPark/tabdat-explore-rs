use crate::*;

impl DuckDbBackend {
  pub fn distinct_nonmissing_string_values(&self, source: &str) -> Result<Vec<String>, DataError> {
    let quoted_source = quote_identifier(source);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT DISTINCT CAST({quoted_source} AS VARCHAR) AS value FROM {ACTIVE_TABLE} WHERE {quoted_source} IS NOT NULL ORDER BY value"
      ))
      .map_err(|_| DataError)?;
    let rows = statement
      .query_map([], |row| row.get::<_, String>(0))
      .map_err(|_| DataError)?;
    rows.map(|row| row.map_err(|_| DataError)).collect()
  }
  pub fn grouped_summarize(
    &self,
    groups: &[String],
    variables: &[String],
  ) -> Result<Vec<Vec<CellValue>>, DataError> {
    if groups.is_empty() || variables.is_empty() {
      return Err(DataError);
    }
    let group_columns = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>();
    let aggregate_columns = variables
      .iter()
      .map(|variable| {
        let quoted_variable = quote_identifier(variable);
        format!(
          "avg({quoted_variable}) AS {}",
          quote_identifier(&format!("mean_{variable}"))
        )
      })
      .collect::<Vec<_>>();
    let order_columns = group_columns
      .iter()
      .map(|group| format!("{group} ASC NULLS LAST"))
      .collect::<Vec<_>>();
    let query = format!(
      "SELECT {}, {} FROM {ACTIVE_TABLE} GROUP BY {} ORDER BY {}",
      group_columns.join(", "),
      aggregate_columns.join(", "),
      group_columns.join(", "),
      order_columns.join(", ")
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let mut grouped = Vec::new();
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      let mut values = Vec::with_capacity(groups.len() + variables.len());
      for index in 0..(groups.len() + variables.len()) {
        values.push(cell_value_from_ref(
          row.get_ref(index).map_err(|_| DataError)?,
        )?);
      }
      grouped.push(values);
    }
    Ok(grouped)
  }
  pub fn grouped_count(&self, groups: &[String]) -> Result<Vec<Vec<CellValue>>, DataError> {
    if groups.is_empty() {
      return Err(DataError);
    }
    let group_columns = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>();
    let order_columns = group_columns
      .iter()
      .map(|group| format!("{group} ASC NULLS LAST"))
      .collect::<Vec<_>>();
    let query = format!(
      "SELECT {}, COUNT(*) FROM {ACTIVE_TABLE} GROUP BY {} ORDER BY {}",
      group_columns.join(", "),
      group_columns.join(", "),
      order_columns.join(", ")
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let mut grouped = Vec::new();
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      let mut values = Vec::with_capacity(groups.len() + 1);
      for index in 0..(groups.len() + 1) {
        values.push(cell_value_from_ref(
          row.get_ref(index).map_err(|_| DataError)?,
        )?);
      }
      grouped.push(values);
    }
    Ok(grouped)
  }
  pub fn tabulate_counts(
    &self,
    row_variable: &str,
    column_variable: Option<&str>,
    include_missing: bool,
  ) -> Result<Vec<TabulateCount>, DataError> {
    self.tabulate_counts_if(row_variable, column_variable, include_missing, None)
  }

  /// Select TRUE predicate rows, then apply category NULL exclusions before grouping.
  /// The caller validates the predicate domain against the active dataset schema.
  pub fn tabulate_counts_if(
    &self,
    row_variable: &str,
    column_variable: Option<&str>,
    include_missing: bool,
    condition: Option<(&AssertExpression, &DatasetInfo)>,
  ) -> Result<Vec<TabulateCount>, DataError> {
    let quoted_row = quote_identifier(row_variable);
    let quoted_column = column_variable.map(quote_identifier);
    let select_columns = quoted_column.as_ref().map_or_else(
      || quoted_row.clone(),
      |column| format!("{quoted_row}, {column}"),
    );
    let group_columns = select_columns.clone();
    let order_columns = quoted_column.as_ref().map_or_else(
      || format!("{quoted_row} ASC NULLS LAST"),
      |column| format!("{quoted_row} ASC NULLS LAST, {column} ASC NULLS LAST"),
    );
    let mut predicates = Vec::new();
    if let Some((expression, dataset)) = condition {
      predicates.push(format!(
        "({})",
        compile_assert_expression(expression, dataset)?
      ));
    }
    if !include_missing {
      predicates.push(format!("{quoted_row} IS NOT NULL"));
      if let Some(column) = quoted_column.as_ref() {
        predicates.push(format!("{column} IS NOT NULL"));
      }
    }
    let where_sql = if predicates.is_empty() {
      String::new()
    } else {
      format!(" WHERE {}", predicates.join(" AND "))
    };
    let query = format!(
      "SELECT {select_columns}, COUNT(*) FROM {ACTIVE_TABLE}{where_sql} GROUP BY {group_columns} ORDER BY {order_columns}"
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let mut counts = Vec::new();
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      let row_value = cell_value_from_ref(row.get_ref(0).map_err(|_| DataError)?)?;
      let column = if quoted_column.is_some() {
        Some(cell_value_from_ref(row.get_ref(1).map_err(|_| DataError)?)?)
      } else {
        None
      };
      let count = u64::try_from(
        row
          .get::<_, i64>(if quoted_column.is_some() { 2 } else { 1 })
          .map_err(|_| DataError)?,
      )
      .map_err(|_| DataError)?;
      counts.push(TabulateCount {
        row: row_value,
        column,
        count,
      });
    }
    Ok(counts)
  }
  pub fn encode_column(
    &mut self,
    dataset: &DatasetInfo,
    source: &str,
    target: &str,
    mapping: &[(String, i64)],
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let quoted_source = quote_identifier(source);
    let quoted_target = quote_identifier(target);
    let case_sql = if mapping.is_empty() {
      "CAST(NULL AS BIGINT)".to_owned()
    } else {
      let whens = mapping
        .iter()
        .map(|(value, code)| {
          format!(
            "WHEN {quoted_source} = {} THEN {code}",
            quote_recode_literal(value)
          )
        })
        .collect::<Vec<_>>()
        .join(" ");
      format!("CASE {whens} ELSE NULL END")
    };
    let select_items = dataset
      .columns
      .iter()
      .map(|column| quote_identifier(&column.name))
      .chain(std::iter::once(format!("{case_sql} AS {quoted_target}")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_items} FROM {ACTIVE_TABLE}"
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
    expected_names.push(target.to_owned());
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
  pub fn decode_column(
    &mut self,
    dataset: &DatasetInfo,
    source: &str,
    target: &str,
    mapping: &[(i64, String)],
  ) -> Result<DatasetInfo, DataError> {
    self.drop_staging();
    let quoted_source = quote_identifier(source);
    let quoted_target = quote_identifier(target);
    let case_sql = if mapping.is_empty() {
      "CAST(NULL AS VARCHAR)".to_owned()
    } else {
      let whens = mapping
        .iter()
        .map(|(code, value)| {
          format!(
            "WHEN {quoted_source} = {code} THEN {}",
            quote_recode_literal(value)
          )
        })
        .collect::<Vec<_>>()
        .join(" ");
      format!("CASE {whens} ELSE NULL END")
    };
    let select_items = dataset
      .columns
      .iter()
      .map(|column| quote_identifier(&column.name))
      .chain(std::iter::once(format!("{case_sql} AS {quoted_target}")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_items} FROM {ACTIVE_TABLE}"
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
    expected_names.push(target.to_owned());
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
}
