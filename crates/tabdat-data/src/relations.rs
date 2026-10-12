use crate::*;

impl DuckDbBackend {
  pub fn run_sql(&self, query: &str) -> Result<(Vec<String>, Vec<Vec<CellValue>>), RuntimeError> {
    self.bind_active_view()?;
    let mut statement = self
      .connection
      .prepare(query)
      .map_err(|_| RuntimeError::SqlFailed)?;
    statement.execute([]).map_err(|_| RuntimeError::SqlFailed)?;
    let headers = statement.column_names();
    if headers.is_empty() {
      return Err(RuntimeError::SqlNoTableResult);
    }
    let column_count = headers.len();
    let mut rows_iter = statement.query([]).map_err(|_| RuntimeError::SqlFailed)?;
    let mut rows = Vec::new();
    while let Some(row) = rows_iter.next().map_err(|_| RuntimeError::SqlFailed)? {
      let mut row_values = Vec::with_capacity(column_count);
      for col_idx in 0..column_count {
        let val_ref = row.get_ref(col_idx).map_err(|_| RuntimeError::SqlFailed)?;
        row_values.push(cell_value_from_ref(val_ref).map_err(|_| RuntimeError::SqlFailed)?);
      }
      rows.push(row_values);
    }
    Ok((headers, rows))
  }
  pub fn create_named_table_from_sql(
    &mut self,
    query: &str,
    table_name: &str,
  ) -> Result<DatasetInfo, RuntimeError> {
    self.bind_active_view()?;
    let internal_name = named_table_identifier(table_name);
    self
      .connection
      .execute_batch(&format!(
        "CREATE OR REPLACE TEMP TABLE {internal_name} AS {query}"
      ))
      .map_err(|_| RuntimeError::SqlFailed)?;
    self
      .connection
      .execute_batch(&format!(
        "CREATE OR REPLACE TEMP TABLE {ACTIVE_TABLE} AS SELECT * FROM {internal_name}"
      ))
      .map_err(|_| RuntimeError::SqlFailed)?;
    let columns = self
      .table_columns(ACTIVE_TABLE)
      .map_err(|_| RuntimeError::SqlFailed)?;
    if columns.is_empty() {
      return Err(RuntimeError::SqlNoTableResult);
    }
    let row_count = self
      .table_row_count(ACTIVE_TABLE)
      .map_err(|_| RuntimeError::SqlFailed)?;
    Ok(DatasetInfo {
      source: PathBuf::from(table_name),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn store_active_as_named_table(&mut self, table_name: &str) -> Result<(), DataError> {
    let internal_name = named_table_identifier(table_name);
    self
      .connection
      .execute_batch(&format!(
        "CREATE OR REPLACE TEMP TABLE {internal_name} AS SELECT * FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| DataError)
  }
  pub fn activate_named_table(&mut self, table_name: &str) -> Result<DatasetInfo, RuntimeError> {
    let internal_name = named_table_identifier(table_name);
    self
      .connection
      .execute_batch(&format!(
        "CREATE OR REPLACE TEMP TABLE {ACTIVE_TABLE} AS SELECT * FROM {internal_name}"
      ))
      .map_err(|_| RuntimeError::BackendInitialization)?;
    let columns = self
      .table_columns(ACTIVE_TABLE)
      .map_err(|_| RuntimeError::BackendInitialization)?;
    let row_count = self
      .table_row_count(ACTIVE_TABLE)
      .map_err(|_| RuntimeError::BackendInitialization)?;
    Ok(DatasetInfo {
      source: PathBuf::from(table_name),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn join_named_table(
    &mut self,
    dataset: &DatasetInfo,
    right_dataset: &DatasetInfo,
    command: &JoinCommand,
  ) -> Result<DatasetInfo, RuntimeError> {
    let active_columns = &dataset.columns;
    let table_name = &command.table_name;
    let keys = &command.keys;
    let how = command.how;
    let suffix = &command.suffix;
    let source = dataset.source.clone();
    let left_names: HashSet<&str> = active_columns.iter().map(|c| c.name.as_str()).collect();
    let missing_left: Vec<String> = keys
      .iter()
      .filter(|key| !left_names.contains(key.as_str()))
      .cloned()
      .collect();
    if !missing_left.is_empty() {
      return Err(RuntimeError::JoinUnknownVariable {
        variables: missing_left,
      });
    }

    let right_names: HashSet<&str> = right_dataset
      .columns
      .iter()
      .map(|c| c.name.as_str())
      .collect();
    let missing_right: Vec<String> = keys
      .iter()
      .filter(|key| !right_names.contains(key.as_str()))
      .cloned()
      .collect();
    if !missing_right.is_empty() {
      return Err(RuntimeError::JoinUnknownVariableInTable {
        table_name: table_name.to_owned(),
        variables: missing_right,
      });
    }

    let mut used_names: HashSet<String> = active_columns.iter().map(|c| c.name.clone()).collect();
    for c in &right_dataset.columns {
      used_names.insert(c.name.clone());
    }
    let left_order_name = unique_internal_name("__tabdat_join_order", &used_names);
    used_names.insert(left_order_name.clone());
    let right_order_name = unique_internal_name("__tabdat_join_right_order", &used_names);

    let mut used_output_names: HashSet<String> =
      active_columns.iter().map(|c| c.name.clone()).collect();
    let mut right_selects = Vec::new();
    for column in &right_dataset.columns {
      if keys.contains(&column.name) {
        continue;
      }
      let base_output_name = if used_output_names.contains(&column.name) {
        format!("{}{suffix}", column.name)
      } else {
        column.name.clone()
      };
      let output_name = unique_output_name(&base_output_name, &used_output_names);
      used_output_names.insert(output_name.clone());
      right_selects.push(format!(
        "right_table.{} AS {}",
        quote_identifier(&column.name),
        quote_identifier(&output_name)
      ));
    }

    let mut select_parts: Vec<String> = active_columns
      .iter()
      .map(|c| format!("left_table.{}", quote_identifier(&c.name)))
      .collect();
    select_parts.extend(right_selects);
    let select_sql = select_parts.join(", ");

    let predicates: Vec<String> = keys
      .iter()
      .map(|key| {
        format!(
          "left_table.{} = right_table.{}",
          quote_identifier(key),
          quote_identifier(key)
        )
      })
      .collect();
    let predicates_sql = predicates.join(" AND ");

    let how_sql = match how {
      JoinHow::Inner => "INNER",
      JoinHow::Left => "LEFT",
    };

    let internal_name = named_table_identifier(table_name);

    let join_query = format!(
      "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_sql} \
       FROM (SELECT row_number() OVER () AS {}, * FROM {ACTIVE_TABLE}) AS left_table \
       {how_sql} JOIN (SELECT row_number() OVER () AS {}, * FROM {}) AS right_table \
       ON {predicates_sql} \
       ORDER BY left_table.{}, right_table.{}",
      quote_identifier(&left_order_name),
      quote_identifier(&right_order_name),
      quote_identifier(&internal_name),
      quote_identifier(&left_order_name),
      quote_identifier(&right_order_name),
    );

    self.drop_staging();
    if self.connection.execute_batch(&join_query).is_err() {
      self.drop_staging();
      return Err(RuntimeError::JoinFailed);
    }

    let columns = match self.staged_columns() {
      Ok(cols) => cols,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::JoinFailed);
      }
    };

    let row_count = match self.staged_row_count() {
      Ok(rc) => rc,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::JoinFailed);
      }
    };

    if self.publish_staging().is_err() {
      return Err(RuntimeError::JoinFailed);
    }

    Ok(DatasetInfo {
      source,
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn append_named_table(
    &mut self,
    dataset: &DatasetInfo,
    append_dataset: &DatasetInfo,
    table_name: &str,
  ) -> Result<DatasetInfo, RuntimeError> {
    let left_names: HashSet<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();
    let right_names: HashSet<&str> = append_dataset
      .columns
      .iter()
      .map(|c| c.name.as_str())
      .collect();

    let mut extra: Vec<String> = Vec::new();
    for c in &append_dataset.columns {
      if !left_names.contains(c.name.as_str()) {
        extra.push(c.name.clone());
      }
    }
    if !extra.is_empty() {
      return Err(RuntimeError::AppendUnknownVariable { variables: extra });
    }

    let mut missing_right: Vec<String> = Vec::new();
    for c in &dataset.columns {
      if !right_names.contains(c.name.as_str()) {
        missing_right.push(c.name.clone());
      }
    }
    if !missing_right.is_empty() {
      return Err(RuntimeError::AppendUnknownVariableInTable {
        table_name: table_name.to_owned(),
        variables: missing_right,
      });
    }

    for left_col in &dataset.columns {
      let right_col = append_dataset
        .columns
        .iter()
        .find(|c| c.name == left_col.name)
        .expect("column existence verified above");
      let left_canonical = canonical_append_type(&left_col.data_type);
      let right_canonical = canonical_append_type(&right_col.data_type);
      if left_canonical.to_uppercase() != right_canonical.to_uppercase() {
        return Err(RuntimeError::AppendTypeMismatch {
          variable: left_col.name.clone(),
          left_type: left_canonical,
          right_type: right_canonical,
        });
      }
    }

    let mut used_names: HashSet<String> = dataset.columns.iter().map(|c| c.name.clone()).collect();
    for c in &append_dataset.columns {
      used_names.insert(c.name.clone());
    }
    let side_col = unique_internal_name("__tabdat_append_side", &used_names);
    used_names.insert(side_col.clone());
    let row_col = unique_internal_name("__tabdat_append_row", &used_names);

    let select_sql = dataset
      .columns
      .iter()
      .map(|c| quote_identifier(&c.name))
      .collect::<Vec<_>>()
      .join(", ");

    let internal_name = named_table_identifier(table_name);

    let append_query = format!(
      "CREATE TEMP TABLE {STAGING_TABLE} AS \
       SELECT * EXCLUDE ({}, {}) \
       FROM ( \
         SELECT 0 AS {}, row_number() OVER () AS {}, {select_sql} \
         FROM {ACTIVE_TABLE} \
         UNION ALL \
         SELECT 1 AS {}, row_number() OVER () AS {}, {select_sql} \
         FROM {} \
       ) AS append_rows \
       ORDER BY {}, {}",
      quote_identifier(&side_col),
      quote_identifier(&row_col),
      quote_identifier(&side_col),
      quote_identifier(&row_col),
      quote_identifier(&side_col),
      quote_identifier(&row_col),
      quote_identifier(&internal_name),
      quote_identifier(&side_col),
      quote_identifier(&row_col),
    );

    self.drop_staging();
    if self.connection.execute_batch(&append_query).is_err() {
      self.drop_staging();
      return Err(RuntimeError::AppendFailed);
    }

    let columns = match self.staged_columns() {
      Ok(cols) => cols,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::AppendFailed);
      }
    };

    let row_count = match self.staged_row_count() {
      Ok(rc) => rc,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::AppendFailed);
      }
    };

    if self.publish_staging().is_err() {
      return Err(RuntimeError::AppendFailed);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn reshape_long(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    identifiers: &[String],
    j_variable: &str,
  ) -> Result<DatasetInfo, RuntimeError> {
    let column_names: HashSet<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();

    let missing_ids: Vec<String> = identifiers
      .iter()
      .filter(|id| !column_names.contains(id.as_str()))
      .cloned()
      .collect();
    if !missing_ids.is_empty() {
      return Err(RuntimeError::ReshapeUnknownVariable {
        variables: missing_ids,
      });
    }

    if column_names.contains(j_variable) {
      return Err(RuntimeError::ReshapeOutputColumnExists {
        variable: j_variable.to_owned(),
      });
    }

    let mut j_values: Vec<String> = Vec::new();
    let mut seen_j: HashSet<String> = HashSet::new();

    for variable in variables {
      let prefix = format!("{variable}_");
      let mut found_any = false;
      for column in &dataset.columns {
        if column.name.starts_with(&prefix) && column.name.len() > prefix.len() {
          found_any = true;
          let suffix = &column.name[prefix.len()..];
          if seen_j.insert(suffix.to_owned()) {
            j_values.push(suffix.to_owned());
          }
        }
      }
      if !found_any {
        return Err(RuntimeError::ReshapeLongFoundNoColumnsForStub {
          stub: variable.clone(),
        });
      }
    }

    for variable in variables {
      for j_val in &j_values {
        let col_name = format!("{variable}_{j_val}");
        if !column_names.contains(col_name.as_str()) {
          return Err(RuntimeError::ReshapeLongMissingColumn { column: col_name });
        }
      }
    }

    let mut used_names: HashSet<String> = dataset.columns.iter().map(|c| c.name.clone()).collect();
    for id in identifiers {
      used_names.insert(id.clone());
    }
    used_names.insert(j_variable.to_owned());
    for v in variables {
      used_names.insert(v.clone());
    }
    let row_order_col = unique_internal_name("__tabdat_reshape_row_order", &used_names);
    used_names.insert(row_order_col.clone());
    let j_order_col = unique_internal_name("__tabdat_reshape_j_order", &used_names);

    let id_sql = identifiers
      .iter()
      .map(|id| format!("reshape_source.{}", quote_identifier(id)))
      .collect::<Vec<_>>()
      .join(", ");

    let mut selects: Vec<String> = Vec::new();
    for (j_index, j_value) in j_values.iter().enumerate() {
      let mut value_parts: Vec<String> = Vec::new();
      for variable in variables {
        let src_col = format!("{variable}_{j_value}");
        value_parts.push(format!(
          "reshape_source.{} AS {}",
          quote_identifier(&src_col),
          quote_identifier(variable),
        ));
      }
      let value_sql = value_parts.join(", ");
      let id_prefix = if id_sql.is_empty() {
        String::new()
      } else {
        format!("{id_sql}, ")
      };
      let value_suffix = if value_sql.is_empty() {
        String::new()
      } else {
        format!(", {value_sql}")
      };
      selects.push(format!(
        "SELECT reshape_source.{}, {id_prefix}{} AS {}{value_suffix}, {} AS {} FROM reshape_source",
        quote_identifier(&row_order_col),
        quote_sql_literal(j_value),
        quote_identifier(j_variable),
        j_index,
        quote_identifier(&j_order_col),
      ));
    }

    let union_query = selects.join(" UNION ALL ");
    let reshape_query = format!(
      "CREATE TEMP TABLE {STAGING_TABLE} AS \
       WITH reshape_source AS ( \
         SELECT row_number() OVER () AS {}, * \
         FROM {ACTIVE_TABLE} \
       ) \
       SELECT * EXCLUDE ({}, {}) \
       FROM ( \
         {union_query} \
       ) AS reshape_long_rows \
       ORDER BY {}, {}",
      quote_identifier(&row_order_col),
      quote_identifier(&row_order_col),
      quote_identifier(&j_order_col),
      quote_identifier(&row_order_col),
      quote_identifier(&j_order_col),
    );

    self.drop_staging();
    if self.connection.execute_batch(&reshape_query).is_err() {
      self.drop_staging();
      return Err(RuntimeError::ReshapeFailed);
    }

    let columns = match self.staged_columns() {
      Ok(cols) => cols,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::ReshapeFailed);
      }
    };

    let row_count = match self.staged_row_count() {
      Ok(rc) => rc,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::ReshapeFailed);
      }
    };

    if self.publish_staging().is_err() {
      return Err(RuntimeError::ReshapeFailed);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn reshape_wide(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    identifiers: &[String],
    j_variable: &str,
  ) -> Result<DatasetInfo, RuntimeError> {
    let column_names: HashSet<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();

    let mut missing: Vec<String> = Vec::new();
    for id in identifiers {
      if !column_names.contains(id.as_str()) && !missing.contains(id) {
        missing.push(id.clone());
      }
    }
    if !column_names.contains(j_variable) && !missing.contains(&j_variable.to_string()) {
      missing.push(j_variable.to_string());
    }
    for var in variables {
      if !column_names.contains(var.as_str()) && !missing.contains(var) {
        missing.push(var.clone());
      }
    }
    if !missing.is_empty() {
      return Err(RuntimeError::ReshapeUnknownVariable { variables: missing });
    }

    let j_sql = format!(
      "SELECT DISTINCT CAST({} AS VARCHAR) AS j_value FROM {ACTIVE_TABLE} WHERE {} IS NOT NULL ORDER BY j_value",
      quote_identifier(j_variable),
      quote_identifier(j_variable),
    );
    let mut stmt = self
      .connection
      .prepare(&j_sql)
      .map_err(|_| RuntimeError::ReshapeFailed)?;
    let mut rows = stmt.query([]).map_err(|_| RuntimeError::ReshapeFailed)?;
    let mut j_values: Vec<String> = Vec::new();
    while let Some(row) = rows.next().map_err(|_| RuntimeError::ReshapeFailed)? {
      let val: String = row.get(0).map_err(|_| RuntimeError::ReshapeFailed)?;
      j_values.push(val);
    }
    if j_values.is_empty() {
      return Err(RuntimeError::ReshapeWideFoundNoJValues);
    }

    let mut source_columns: HashSet<&str> = identifiers.iter().map(|s| s.as_str()).collect();
    source_columns.insert(j_variable);
    for var in variables {
      source_columns.insert(var.as_str());
    }

    for variable in variables {
      for j_value in &j_values {
        let output_name = format!("{variable}_{j_value}");
        if column_names.contains(output_name.as_str())
          && !source_columns.contains(output_name.as_str())
        {
          return Err(RuntimeError::ReshapeWideOutputColumnExists {
            variable: output_name,
          });
        }
      }
    }

    let mut used_names: HashSet<String> = dataset.columns.iter().map(|c| c.name.clone()).collect();
    for id in identifiers {
      used_names.insert(id.clone());
    }
    for var in variables {
      for j_val in &j_values {
        used_names.insert(format!("{var}_{j_val}"));
      }
    }
    let group_order_name = unique_internal_name("__tabdat_reshape_group_order", &used_names);
    used_names.insert(group_order_name.clone());
    let source_order_name = unique_internal_name("__tabdat_reshape_source_order", &used_names);

    let id_sql = identifiers
      .iter()
      .map(|id| format!("reshape_source.{}", quote_identifier(id)))
      .collect::<Vec<_>>()
      .join(", ");

    let mut aggregates: Vec<String> = Vec::new();
    for variable in variables {
      for j_value in &j_values {
        let out_col = format!("{variable}_{j_value}");
        aggregates.push(format!(
          "MAX(CASE WHEN CAST(reshape_source.{} AS VARCHAR) = {} THEN reshape_source.{} END) AS {}",
          quote_identifier(j_variable),
          quote_sql_literal(j_value),
          quote_identifier(variable),
          quote_identifier(&out_col),
        ));
      }
    }
    let aggregate_sql = aggregates.join(", ");

    let reshape_query = format!(
      "CREATE TEMP TABLE {STAGING_TABLE} AS \
       SELECT * EXCLUDE ({}) \
       FROM ( \
         SELECT {id_sql}, {aggregate_sql}, \
           MIN(reshape_source.{}) AS {} \
         FROM ( \
           SELECT row_number() OVER () AS {}, * \
           FROM {ACTIVE_TABLE} \
         ) AS reshape_source \
         GROUP BY {id_sql} \
       ) AS reshape_wide_rows \
       ORDER BY {}",
      quote_identifier(&group_order_name),
      quote_identifier(&source_order_name),
      quote_identifier(&group_order_name),
      quote_identifier(&source_order_name),
      quote_identifier(&group_order_name),
    );

    self.drop_staging();
    if self.connection.execute_batch(&reshape_query).is_err() {
      self.drop_staging();
      return Err(RuntimeError::ReshapeFailed);
    }

    let columns = match self.staged_columns() {
      Ok(cols) => cols,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::ReshapeFailed);
      }
    };

    let row_count = match self.staged_row_count() {
      Ok(rc) => rc,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::ReshapeFailed);
      }
    };

    if self.publish_staging().is_err() {
      return Err(RuntimeError::ReshapeFailed);
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
}
