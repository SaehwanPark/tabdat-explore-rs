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
    self.summarize_variable_if(variable, None)
  }
  pub fn summarize_variable_if(
    &self,
    variable: &str,
    predicate: Option<(&AssertExpression, &DatasetInfo)>,
  ) -> Result<SummaryRow, DataError> {
    self
      .summary_variable(variable, predicate, false)
      .map(|row| row.summary)
  }
  pub fn summarize_detail_variable_if(
    &self,
    variable: &str,
    predicate: Option<(&AssertExpression, &DatasetInfo)>,
  ) -> Result<SummaryDetailRow, DataError> {
    self.summary_variable(variable, predicate, true)
  }
  fn summary_variable(
    &self,
    variable: &str,
    predicate: Option<(&AssertExpression, &DatasetInfo)>,
    detail: bool,
  ) -> Result<SummaryDetailRow, DataError> {
    let condition = predicate
      .map(|(expression, dataset)| compile_assert_expression(expression, dataset))
      .transpose()?;
    let selection = condition
      .map(|sql| format!(" AND ({sql})"))
      .unwrap_or_default();
    let quoted_variable = quote_identifier(variable);
    let quantiles = if detail {
      SUMMARY_PERCENTILES
        .iter()
        .map(|percentile| {
          format!(", quantile_cont(CAST({quoted_variable} AS DOUBLE), {percentile}.0 / 100)")
        })
        .collect::<String>()
    } else {
      String::new()
    };
    // One finite sample for every aggregate: NaN/infinity can poison variance.
    // The finiteness test casts to DOUBLE (all supported exact numeric ranges
    // fit); keep aggregate input uncast to retain integer/decimal extrema.
    // The separate row predicate intersects this sample, never a listwise one.
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT count({quoted_variable}), avg({quoted_variable}), stddev_samp({quoted_variable}), min({quoted_variable}), max({quoted_variable}){quantiles} FROM {ACTIVE_TABLE} WHERE isfinite(CAST({quoted_variable} AS DOUBLE)){selection}"
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
    let percentiles = if detail {
      SUMMARY_PERCENTILES
        .iter()
        .enumerate()
        .map(|(index, percentile)| {
          Ok(SummaryPercentile {
            percentile: *percentile,
            value: row.get(5 + index).map_err(|_| DataError)?,
          })
        })
        .collect::<Result<Vec<_>, DataError>>()?
    } else {
      Vec::new()
    };
    Ok(SummaryDetailRow {
      summary: SummaryRow {
        variable: variable.to_owned(),
        count,
        mean,
        std_dev,
        minimum,
        maximum,
      },
      percentiles,
    })
  }
  pub fn xtsum_has_invalid_panel_ids(
    &self,
    id_variable: &str,
    numeric: bool,
  ) -> Result<bool, DataError> {
    let id = quote_identifier(id_variable);
    let invalid = if numeric {
      format!("{id} IS NULL OR NOT isfinite(CAST({id} AS DOUBLE))")
    } else {
      format!("{id} IS NULL")
    };
    self
      .connection
      .query_row(
        &format!("SELECT COUNT(*) FILTER (WHERE {invalid}) > 0 FROM {ACTIVE_TABLE}"),
        [],
        |row| row.get(0),
      )
      .map_err(|_| DataError)
  }
  pub fn xtsum_variable(&self, variable: &str, id_variable: &str) -> Result<XtSumRow, DataError> {
    let value = quote_identifier(variable);
    let id = quote_identifier(id_variable);
    // Keep a variable-specific finite sample. The overall singleton supplies
    // the pooled center for within values, including unbalanced panels.
    let sql = format!(
      r#"
WITH valid AS (
  SELECT {id} AS panel_id, CAST({value} AS DOUBLE) AS x
  FROM {ACTIVE_TABLE}
  WHERE {value} IS NOT NULL AND isfinite(CAST({value} AS DOUBLE))
), overall AS (
  SELECT COUNT(*) AS overall_n, AVG(x) AS overall_mean,
    STDDEV_SAMP(x) AS overall_sd, MIN(x) AS overall_min, MAX(x) AS overall_max
  FROM valid
), panel_means AS (
  SELECT panel_id, COUNT(*) AS observations, AVG(x) AS panel_mean
  FROM valid GROUP BY panel_id
), between_stats AS (
  SELECT COUNT(*) AS between_n, STDDEV_SAMP(panel_mean) AS between_sd,
    MIN(panel_mean) AS between_min, MAX(panel_mean) AS between_max,
    AVG(observations) AS mean_t
  FROM panel_means
), within_values AS (
  SELECT v.x - p.panel_mean + o.overall_mean AS x
  FROM valid AS v
  JOIN panel_means AS p USING (panel_id)
  CROSS JOIN overall AS o
), within_stats AS (
  SELECT COUNT(*) AS within_n, STDDEV_SAMP(x) AS within_sd,
    MIN(x) AS within_min, MAX(x) AS within_max
  FROM within_values
)
SELECT o.overall_n, o.overall_mean, o.overall_sd, o.overall_min, o.overall_max,
  b.between_n, b.between_sd, b.between_min, b.between_max,
  w.within_n, w.within_sd, w.within_min, w.within_max, b.mean_t
FROM overall AS o CROSS JOIN between_stats AS b CROSS JOIN within_stats AS w
"#
    );
    let mut statement = self.connection.prepare(&sql).map_err(|_| DataError)?;
    let mut rows = statement.query([]).map_err(|_| DataError)?;
    let row = rows.next().map_err(|_| DataError)?.ok_or(DataError)?;
    let overall_n =
      u64::try_from(row.get::<_, i64>(0).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let overall_mean: Option<f64> = row.get(1).map_err(|_| DataError)?;
    let overall_sd: Option<f64> = row.get(2).map_err(|_| DataError)?;
    let overall_min: Option<f64> = row.get(3).map_err(|_| DataError)?;
    let overall_max: Option<f64> = row.get(4).map_err(|_| DataError)?;
    let between_n =
      u64::try_from(row.get::<_, i64>(5).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let between_sd: Option<f64> = row.get(6).map_err(|_| DataError)?;
    let between_min: Option<f64> = row.get(7).map_err(|_| DataError)?;
    let between_max: Option<f64> = row.get(8).map_err(|_| DataError)?;
    let within_n =
      u64::try_from(row.get::<_, i64>(9).map_err(|_| DataError)?).map_err(|_| DataError)?;
    let within_sd: Option<f64> = row.get(10).map_err(|_| DataError)?;
    let within_min: Option<f64> = row.get(11).map_err(|_| DataError)?;
    let within_max: Option<f64> = row.get(12).map_err(|_| DataError)?;
    let mean_t: Option<f64> = row.get(13).map_err(|_| DataError)?;
    let numeric = [
      overall_mean,
      overall_sd,
      overall_min,
      overall_max,
      between_sd,
      between_min,
      between_max,
      within_sd,
      within_min,
      within_max,
      mean_t,
    ];
    if numeric
      .into_iter()
      .flatten()
      .any(|value| !value.is_finite())
    {
      return Err(DataError);
    }
    Ok(XtSumRow {
      variable: variable.to_owned(),
      overall_n,
      overall_mean,
      overall_sd,
      overall_min,
      overall_max,
      between_n,
      between_sd,
      between_min,
      between_max,
      within_n,
      within_sd,
      within_min,
      within_max,
      mean_t,
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
  /// Count every TRUE predicate row, without per-variable missingness exclusions.
  /// The caller validates the predicate against the active schema.
  pub fn count_rows_if(
    &self,
    expression: &AssertExpression,
    dataset: &DatasetInfo,
  ) -> Result<u64, DataError> {
    let predicate = compile_assert_expression(expression, dataset)?;
    let query = format!("SELECT COUNT(*) FROM {ACTIVE_TABLE} WHERE ({predicate})");
    let count = self
      .connection
      .query_row(&query, [], |row| row.get::<_, i64>(0))
      .map_err(|_| DataError)?;
    u64::try_from(count).map_err(|_| DataError)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn finite_predicate_keeps_full_128_bit_integer_extrema_without_narrowing() {
    let backend = DuckDbBackend::new().unwrap();
    backend.connection.execute_batch(&format!("CREATE TABLE {ACTIVE_TABLE} AS SELECT CAST('{}' AS UHUGEINT) AS u, CAST('{}' AS HUGEINT) AS i", u128::MAX, i128::MIN)).unwrap();
    for (variable, value) in [
      ("u", CellValue::UnsignedInteger(u128::MAX)),
      ("i", CellValue::SignedInteger(i128::MIN)),
    ] {
      let summary = backend.summarize_variable(variable).unwrap();
      assert_eq!(summary.count, 1);
      assert!(summary.mean.unwrap().is_finite());
      assert_eq!(summary.std_dev, None);
      assert_eq!(summary.minimum, Some(value.clone()));
      assert_eq!(summary.maximum, Some(value));
    }
  }
}
