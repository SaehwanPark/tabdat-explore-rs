use crate::*;

const PREDICTIONS_TABLE: &str = "__tabdat_prediction_values";

impl DuckDbBackend {
  /// Append finite/NULL Rust-computed predictions in current physical row order.
  /// Typed staging avoids interpolating entity identities or giant CASE expressions.
  pub fn add_numeric_prediction_column(
    &mut self,
    dataset: &DatasetInfo,
    target: &str,
    values: &[Option<f64>],
  ) -> Result<DatasetInfo, DataError> {
    if u64::try_from(values.len()).ok() != Some(dataset.row_count)
      || values.iter().flatten().any(|value| !value.is_finite())
      || dataset.columns.iter().any(|column| column.name == target)
      || self.table_row_count(ACTIVE_TABLE)? != dataset.row_count
    {
      return Err(DataError);
    }
    self.drop_staging();
    let result = (|| {
      self.connection.execute_batch(&format!("DROP TABLE IF EXISTS {PREDICTIONS_TABLE}; CREATE TEMP TABLE {PREDICTIONS_TABLE}(value DOUBLE)"))
        .map_err(|_|DataError)?;
      {
        let mut appender = self
          .connection
          .appender(PREDICTIONS_TABLE)
          .map_err(|_| DataError)?;
        for value in values {
          appender.append_row(params![value]).map_err(|_| DataError)?;
        }
        appender.flush().map_err(|_| DataError)?;
      }
      // DuckDB's positional join preserves both input orders. The connection
      // enables preserve_insertion_order, including the preceding visit_rows scan.
      self.connection.execute_batch(&format!("CREATE TEMP TABLE {STAGING_TABLE} AS SELECT source.*, predictions.value AS {} FROM {ACTIVE_TABLE} AS source POSITIONAL JOIN {PREDICTIONS_TABLE} AS predictions",quote_identifier(target)))
        .map_err(|_|DataError)?;
      let columns = self.staged_columns()?;
      if columns.iter().map(|column| column.name.as_str()).ne(
        dataset
          .columns
          .iter()
          .map(|column| column.name.as_str())
          .chain(std::iter::once(target)),
      ) || columns
        .last()
        .is_none_or(|column| column.data_type != "DOUBLE")
      {
        return Err(DataError);
      }
      let row_count = self.staged_row_count()?;
      if row_count != dataset.row_count {
        return Err(DataError);
      }
      self.publish_staging()?;
      Ok(DatasetInfo {
        source: dataset.source.clone(),
        row_count,
        columns,
        execution_mode: dataset.execution_mode,
        lazy_engine: dataset.lazy_engine,
      })
    })();
    let _ = self
      .connection
      .execute_batch(&format!("DROP TABLE IF EXISTS {PREDICTIONS_TABLE}"));
    if result.is_err() {
      self.drop_staging();
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn metadata(backend: &DuckDbBackend) -> DatasetInfo {
    DatasetInfo {
      source: "input.csv".into(),
      row_count: backend.table_row_count(ACTIVE_TABLE).unwrap(),
      columns: backend.table_columns(ACTIVE_TABLE).unwrap(),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    }
  }
  #[test]
  fn typed_positional_predictions_preserve_wide_ids_order_nulls_and_atomic_refusals() {
    let mut backend = DuckDbBackend::new().unwrap();
    backend.connection.execute_batch(&format!("CREATE TEMP TABLE {ACTIVE_TABLE}(id UBIGINT, label VARCHAR, x DECIMAL(12,2)); INSERT INTO {ACTIVE_TABLE} VALUES (18446744073709551614,'b',1.20),(9223372036854775809,NULL,2.30),(4,'a',NULL)")).unwrap();
    let dataset = metadata(&backend);
    let original = backend
      .run_sql(&format!("SELECT * FROM {ACTIVE_TABLE}"))
      .unwrap();
    for (target, values) in [
      ("x", vec![Some(1.0); 3]),
      ("bad", vec![Some(1.0)]),
      ("bad", vec![Some(f64::INFINITY); 3]),
      ("bad", vec![Some(f64::NAN); 3]),
    ] {
      assert!(
        backend
          .add_numeric_prediction_column(&dataset, target, &values)
          .is_err()
      );
      assert_eq!(
        backend
          .run_sql(&format!("SELECT * FROM {ACTIVE_TABLE}"))
          .unwrap(),
        original
      );
    }
    for count in [2, 4] {
      let mut stale = dataset.clone();
      stale.row_count = count;
      assert!(
        backend
          .add_numeric_prediction_column(&stale, "bad", &vec![Some(1.0); count as usize])
          .is_err()
      );
      assert_eq!(
        backend
          .run_sql(&format!("SELECT * FROM {ACTIVE_TABLE}"))
          .unwrap(),
        original
      );
    }
    let next = backend
      .add_numeric_prediction_column(&dataset, "fitted \" value", &[Some(1.0), None, Some(-2.0)])
      .unwrap();
    assert_eq!(&next.columns[..3], dataset.columns);
    assert_eq!(next.columns[3].data_type, "DOUBLE");
    assert_eq!(next.source, dataset.source);
    assert_eq!(next.row_count, 3);
    let (_, rows) = backend
      .run_sql(&format!("SELECT * FROM {ACTIVE_TABLE}"))
      .unwrap();
    for (i, row) in rows.iter().enumerate() {
      assert_eq!(&row[..3], original.1[i]);
    }
    assert_eq!(rows[0][3], CellValue::Float(1.0));
    assert_eq!(rows[1][3], CellValue::Null);
    assert_eq!(rows[2][3], CellValue::Float(-2.0));
    let helpers: i64 = backend
      .connection
      .query_row(
        "SELECT COUNT(*) FROM duckdb_tables() WHERE table_name = ?",
        [PREDICTIONS_TABLE],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(helpers, 0);
    backend
      .connection
      .execute_batch(&format!("DELETE FROM {ACTIVE_TABLE}"))
      .unwrap();
    let empty = metadata(&backend);
    let next = backend
      .add_numeric_prediction_column(&empty, "empty", &[])
      .unwrap();
    assert_eq!(next.row_count, 0);
    assert_eq!(next.columns.last().unwrap().data_type, "DOUBLE");
  }
}
