//! Streaming owned row boundary. No DuckDB statement, row or value leaves the adapter.

use crate::{DuckDbBackend, cell_value_from_ref};
use std::{error::Error, fmt};
use tabdat_domain::CellValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError;
impl fmt::Display for DataError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("data operation failed")
  }
}
impl Error for DataError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellReadError {
  Backend(String),
  Unsupported,
  InvalidIndex,
}

pub struct DataRow {
  cells: Vec<Result<CellValue, CellReadError>>,
}
impl DataRow {
  pub fn cell(&self, index: usize) -> Result<&CellValue, CellReadError> {
    self
      .cells
      .get(index)
      .ok_or(CellReadError::InvalidIndex)?
      .as_ref()
      .map_err(Clone::clone)
  }
}

impl DuckDbBackend {
  /// Copy one row at a time. Failed cell conversions are retained until accessed:
  /// sample builders may legitimately skip a row before inspecting another column.
  /// Query failures are normalized through the caller's typed error constructor.
  pub fn visit_rows<E>(
    &self,
    query: &str,
    mut visit: impl FnMut(&DataRow) -> Result<(), E>,
    query_error: impl Fn(String) -> E,
  ) -> Result<(), E> {
    let mut statement = self
      .connection
      .prepare(query)
      .map_err(|error| query_error(error.to_string()))?;
    let mut rows = statement
      .query([])
      .map_err(|error| query_error(error.to_string()))?;
    while let Some(row) = rows
      .next()
      .map_err(|error| query_error(error.to_string()))?
    {
      // duckdb-rs exposes column metadata only after execution, not prepare.
      let count = row.as_ref().column_count();
      let cells = (0..count)
        .map(|index| {
          row
            .get_ref(index)
            .map_err(|error| CellReadError::Backend(error.to_string()))
            .and_then(|value| cell_value_from_ref(value).map_err(|_| CellReadError::Unsupported))
        })
        .collect();
      visit(&DataRow { cells })?;
    }
    Ok(())
  }

  /// Execute adapter-owned internal SQL without exposing a native connection.
  /// This is a trusted low-level operation, not the user-facing SQL policy.
  pub fn execute_batch(&self, sql: &str) -> Result<(), DataError> {
    self.connection.execute_batch(sql).map_err(|_| DataError)
  }

  /// Read one owned integer for adapter diagnostics and contract tests.
  pub fn query_i64(&self, sql: &str) -> Result<i64, DataError> {
    self
      .connection
      .query_row(sql, [], |row| row.get(0))
      .map_err(|_| DataError)
  }
}
