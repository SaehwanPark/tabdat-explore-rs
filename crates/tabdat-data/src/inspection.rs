use crate::*;

impl DuckDbBackend {
  pub fn preview_rows(
    &self,
    limit: i64,
    column_count: usize,
    from_end: bool,
  ) -> Result<Vec<Vec<CellValue>>, DataError> {
    let order = if from_end { "DESC" } else { "ASC" };
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT * FROM (SELECT row_number() OVER () AS __tabdat_preview_order, * FROM {ACTIVE_TABLE}) ORDER BY 1 {order} LIMIT ?"
      ))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([limit]).map_err(|_| DataError)?;
    let mut preview = Vec::new();
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      let mut values = Vec::with_capacity(column_count);
      for index in 0..column_count {
        let value = row.get_ref(index + 1).map_err(|_| DataError)?;
        values.push(cell_value_from_ref(value)?);
      }
      preview.push(values);
    }
    if from_end {
      preview.reverse();
    }
    Ok(preview)
  }
  pub fn summarize_variable(&self, variable: &str) -> Result<SummaryRow, DataError> {
    let quoted_variable = quote_identifier(variable);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT count({quoted_variable}), avg({quoted_variable}), stddev_samp({quoted_variable}), min({quoted_variable}), max({quoted_variable}) FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let count: i64 = row.get(0).map_err(|_| DataError)?;
    let count = u64::try_from(count).map_err(|_| DataError)?;
    let mean: Option<f64> = row.get(1).map_err(|_| DataError)?;
    let std_dev: Option<f64> = row.get(2).map_err(|_| DataError)?;
    let minimum = match row.get_ref(3).map_err(|_| DataError)? {
      ValueRef::Null => None,
      value => Some(cell_value_from_ref(value)?),
    };
    let maximum = match row.get_ref(4).map_err(|_| DataError)? {
      ValueRef::Null => None,
      value => Some(cell_value_from_ref(value)?),
    };
    Ok(SummaryRow {
      variable: variable.to_owned(),
      count,
      mean,
      std_dev,
      minimum,
      maximum,
    })
  }
  pub fn codebook_variable(
    &self,
    variable: &str,
    data_type: &str,
  ) -> Result<CodebookRow, DataError> {
    let quoted_variable = quote_identifier(variable);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT count({quoted_variable}), count(*) - count({quoted_variable}), count(DISTINCT {quoted_variable}) FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let nonmissing =
      u64::try_from(row.get::<_, i64>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let missing =
      u64::try_from(row.get::<_, i64>(1).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let distinct =
      u64::try_from(row.get::<_, i64>(2).map_err(|_| DataError)?).map_err(|_| DataError)?;

    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT {quoted_variable} FROM {ACTIVE_TABLE} WHERE {quoted_variable} IS NOT NULL LIMIT 3"
      ))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let mut examples = Vec::new();
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      examples.push(cell_value_from_ref(row.get_ref(0).map_err(|_| DataError)?)?);
    }
    Ok(CodebookRow {
      variable: variable.to_owned(),
      data_type: data_type.to_owned(),
      nonmissing,
      missing,
      distinct,
      examples,
    })
  }
  pub fn missingness(&self, variables: &[(&str, &str)]) -> Result<Vec<MissingRow>, DataError> {
    let mut select_items = vec!["count(*)".to_owned()];
    for (index, (variable, _)) in variables.iter().enumerate() {
      let quoted_variable = quote_identifier(variable);
      select_items.push(format!(
        "count({quoted_variable}) AS \"__tabdat_missing_nonmissing_{index}\""
      ));
    }
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT {} FROM {ACTIVE_TABLE}",
        select_items.join(", ")
      ))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let total =
      u64::try_from(row.get::<_, i64>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    variables
      .iter()
      .enumerate()
      .map(|(index, (variable, data_type))| {
        let nonmissing = u64::try_from(row.get::<_, i64>(index + 1).map_err(|_| DataError)?)
          .map_err(|_| DataError)?;
        let missing = total.checked_sub(nonmissing).ok_or(DataError)?;
        let missing_percent = if total == 0 {
          0.0
        } else {
          (missing as f64 / total as f64) * 100.0
        };
        Ok(MissingRow {
          variable: (*variable).to_owned(),
          data_type: (*data_type).to_owned(),
          total,
          missing,
          nonmissing,
          missing_percent,
        })
      })
      .collect()
  }
  pub fn duplicates(&self, variables: &[String]) -> Result<DuplicatesResult, DataError> {
    let count_alias = duplicate_count_alias(variables);
    let grouped_query = if variables.is_empty() {
      format!("SELECT COUNT(*) AS {count_alias} FROM {ACTIVE_TABLE}")
    } else {
      let group_columns = variables
        .iter()
        .map(|variable| quote_identifier(variable))
        .collect::<Vec<_>>()
        .join(", ");
      format!("SELECT COUNT(*) AS {count_alias} FROM {ACTIVE_TABLE} GROUP BY {group_columns}")
    };
    let query = format!(
      "SELECT COALESCE(SUM({count_alias}), CAST(0 AS HUGEINT)),\
       COUNT(*),\
       COUNT(*) FILTER (WHERE {count_alias} >= 2),\
       COALESCE(SUM(CASE WHEN {count_alias} >= 2 THEN {count_alias} ELSE 0 END), CAST(0 AS HUGEINT)),\
       COALESCE(SUM(CASE WHEN {count_alias} >= 2 THEN {count_alias} - 1 ELSE 0 END), CAST(0 AS HUGEINT)),\
       COALESCE(MAX({count_alias}), CAST(0 AS HUGEINT))\
       FROM ({grouped_query}) AS \"__tabdat_duplicate_groups\""
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let total_rows =
      u64::try_from(row.get::<_, i128>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let unique_groups =
      u64::try_from(row.get::<_, i64>(1).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let duplicate_groups =
      u64::try_from(row.get::<_, i64>(2).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let duplicate_rows =
      u64::try_from(row.get::<_, i128>(3).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let extra_rows =
      u64::try_from(row.get::<_, i128>(4).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let max_copies =
      u64::try_from(row.get::<_, i128>(5).map_err(|_| DataError)?).map_err(|_| DataError)?;
    Ok(DuplicatesResult {
      variables: variables.to_owned(),
      total_rows,
      unique_groups,
      duplicate_groups,
      duplicate_rows,
      extra_rows,
      max_copies,
    })
  }
  pub fn isid_counts(&self, variables: &[String]) -> Result<(u64, u64, u64, u64, u64), DataError> {
    let count_alias = isid_count_alias(variables);
    let key_sql = variables
      .iter()
      .map(|variable| quote_identifier(variable))
      .collect::<Vec<_>>()
      .join(", ");
    let missing_condition = variables
      .iter()
      .map(|variable| format!("{} IS NULL", quote_identifier(variable)))
      .collect::<Vec<_>>()
      .join(" OR ");
    let grouped_query =
      format!("SELECT {key_sql}, COUNT(*) AS {count_alias} FROM {ACTIVE_TABLE} GROUP BY {key_sql}");
    let query = format!(
      "SELECT COALESCE(SUM({count_alias}), CAST(0 AS HUGEINT)),\
       COUNT(*),\
       COUNT(*) FILTER (WHERE {count_alias} > 1),\
       COALESCE(SUM(CASE WHEN {count_alias} > 1 THEN {count_alias} ELSE 0 END), CAST(0 AS HUGEINT)),\
       COALESCE(SUM(CASE WHEN {missing_condition} THEN {count_alias} ELSE 0 END), CAST(0 AS HUGEINT))\
       FROM ({grouped_query}) AS \"__tabdat_isid_groups\""
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let total_rows =
      u64::try_from(row.get::<_, i128>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let unique_groups =
      u64::try_from(row.get::<_, i64>(1).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let duplicate_groups =
      u64::try_from(row.get::<_, i64>(2).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let duplicate_rows =
      u64::try_from(row.get::<_, i128>(3).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let missing_key_rows =
      u64::try_from(row.get::<_, i128>(4).map_err(|_| DataError)?).map_err(|_| DataError)?;
    Ok((
      total_rows,
      unique_groups,
      duplicate_groups,
      duplicate_rows,
      missing_key_rows,
    ))
  }
  pub fn datasignature(&self, columns: &[ColumnInfo]) -> Result<(String, u64), DataError> {
    let mut digest = Sha256::new();
    digest.update(b"tabdat-datasignature/v1\x00");
    update_signature_token(&mut digest, b"schema");
    for column in columns {
      update_signature_token(&mut digest, b"column");
      update_signature_token(&mut digest, &signature_text(&column.name));
      let canonical_type = canonical_signature_type(&column.data_type);
      update_signature_token(&mut digest, &signature_text(&canonical_type));
    }

    let mut statement = self
      .connection
      .prepare(&format!("SELECT * FROM {ACTIVE_TABLE}"))
      .map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let mut row_count = 0_u64;
    while let Some(row) = rows.next().map_err(|_| DataError)? {
      update_signature_token(&mut digest, b"row");
      for (index, column) in columns.iter().enumerate() {
        let value = row.get_ref(index).map_err(|_| DataError)?.to_owned();
        let encoded = signature_value(&value, Some(&column.data_type)).map_err(|_| DataError)?;
        update_signature_token(&mut digest, &encoded);
      }
      row_count = row_count.checked_add(1).ok_or(DataError)?;
    }
    drop(rows);
    drop(statement);

    Ok((lower_hex(&digest.finalize()), row_count))
  }
  pub fn assert_rows(
    &self,
    expression: &AssertExpression,
    dataset: &DatasetInfo,
  ) -> Result<(u64, u64), DataError> {
    let predicate = compile_assert_expression(expression, dataset)?;
    let query = format!(
      "SELECT COUNT(*), COUNT(*) FILTER (WHERE ({predicate}) IS NULL OR NOT ({predicate})) FROM {ACTIVE_TABLE}"
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let checked =
      u64::try_from(row.get::<_, i64>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let failed =
      u64::try_from(row.get::<_, i64>(1).map_err(|_| DataError)?).map_err(|_| DataError)?;
    Ok((checked, failed))
  }
}
