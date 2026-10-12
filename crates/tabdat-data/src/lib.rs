#![forbid(unsafe_code)]
//! Canonical DuckDB adapter with owned public values and a private native connection.

use duckdb::types::{TimeUnit, Value, ValueRef};
use duckdb::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use tabdat_domain::*;
use tabdat_io::LocalInputFormat;
mod owned_rows;
pub use owned_rows::{CellReadError, DataError, DataRow};
pub const ACTIVE_TABLE: &str = "__tabdat_active";
const STAGING_TABLE: &str = "__tabdat_next";
const ACTIVE_VIEW: &str = "active";

pub struct TabulateCount {
  pub row: CellValue,
  pub column: Option<CellValue>,
  pub count: u64,
}

pub struct LinearPrediction<'a> {
  pub target_variable: &'a str,
  pub predictor_names: &'a [String],
  pub predictor_coefficients: &'a [f64],
  pub intercept: Option<f64>,
  pub outcome_variable: &'a str,
  pub residuals: bool,
}

mod expressions;
pub use expressions::*;
mod scalars;
use scalars::*;
pub use scalars::{python_float_hex, validate_sql_query};
mod inspection;
mod labels;
mod loading;
mod relations;
mod transforms;

pub struct DuckDbBackend {
  connection: Connection,
}

impl DuckDbBackend {
  pub fn new() -> Result<Self, RuntimeError> {
    let connection =
      Connection::open_in_memory().map_err(|_| RuntimeError::BackendInitialization)?;
    connection
      .execute_batch("SET preserve_insertion_order = true")
      .map_err(|_| RuntimeError::BackendInitialization)?;
    Ok(Self { connection })
  }
  pub fn table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, DataError> {
    let mut statement = self
      .connection
      .prepare(&format!("DESCRIBE {table_name}"))
      .map_err(|_| DataError)?;
    let rows = statement
      .query_map([], |row| {
        Ok(ColumnInfo {
          name: row.get(0)?,
          data_type: row.get(1)?,
        })
      })
      .map_err(|_| DataError)?;
    rows.map(|row| row.map_err(|_| DataError)).collect()
  }
  pub fn table_row_count(&self, table_name: &str) -> Result<u64, DataError> {
    let row_count: i64 = self
      .connection
      .query_row(&format!("SELECT COUNT(*) FROM {table_name}"), [], |row| {
        row.get(0)
      })
      .map_err(|_| DataError)?;
    u64::try_from(row_count).map_err(|_| DataError)
  }
  pub fn staged_columns(&self) -> Result<Vec<ColumnInfo>, DataError> {
    self.table_columns(STAGING_TABLE)
  }
  pub fn staged_row_count(&self) -> Result<u64, DataError> {
    self.table_row_count(STAGING_TABLE)
  }
  pub fn bind_active_view(&self) -> Result<(), RuntimeError> {
    self
      .connection
      .execute_batch(&format!(
        "CREATE OR REPLACE TEMP VIEW {ACTIVE_VIEW} AS SELECT * FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| RuntimeError::SqlFailed)
  }
  pub fn publish_staging(&mut self) -> Result<(), DataError> {
    self
      .connection
      .execute_batch("BEGIN TRANSACTION")
      .map_err(|_| DataError)?;
    let result = (|| {
      self
        .connection
        .execute_batch(&format!("DROP TABLE IF EXISTS {ACTIVE_TABLE}"))?;
      self.connection.execute_batch(&format!(
        "ALTER TABLE {STAGING_TABLE} RENAME TO {ACTIVE_TABLE}"
      ))?;
      self.connection.execute_batch("COMMIT")?;
      Ok::<(), duckdb::Error>(())
    })();
    if result.is_err() {
      let _ = self.connection.execute_batch("ROLLBACK");
      return Err(DataError);
    }
    Ok(())
  }
  pub fn drop_staging(&mut self) {
    let _ = self
      .connection
      .execute_batch(&format!("DROP TABLE IF EXISTS {STAGING_TABLE}"));
  }
}
