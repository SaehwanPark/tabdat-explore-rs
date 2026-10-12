use crate::*;

impl DuckDbBackend {
  pub fn save_active_parquet(&self, path: &Path) -> Result<(), DataError> {
    let path_string = path.to_str().ok_or(DataError)?;
    self
      .connection
      .execute(
        &format!("COPY (SELECT * FROM {ACTIVE_TABLE}) TO ? (FORMAT PARQUET)"),
        [path_string],
      )
      .map(|_| ())
      .map_err(|_| DataError)
  }
  pub fn export_active_csv(&self, path: &Path) -> Result<(), DataError> {
    let path_string = path.to_str().ok_or(DataError)?;
    self
      .connection
      .execute(
        &format!("COPY (SELECT * FROM {ACTIVE_TABLE}) TO ? (FORMAT CSV, HEADER)"),
        [path_string],
      )
      .map(|_| ())
      .map_err(|_| DataError)
  }
  pub fn load_eager_input(
    &mut self,
    path: &Path,
    input_format: LocalInputFormat,
    delimiter: Option<&str>,
    has_header: Option<bool>,
  ) -> Result<DatasetInfo, RuntimeError> {
    match input_format {
      LocalInputFormat::Parquet => self.load_eager_parquet(path),
      LocalInputFormat::Csv => self.load_eager_csv(path, delimiter, has_header),
    }
  }
  pub fn load_eager_csv(
    &mut self,
    path: &Path,
    delimiter: Option<&str>,
    has_header: Option<bool>,
  ) -> Result<DatasetInfo, RuntimeError> {
    let path_string = path.to_str().ok_or_else(|| RuntimeError::NonUtf8Path {
      path: path.to_owned(),
    })?;

    self.drop_staging();
    let load_result = match (delimiter, has_header) {
      (None, None) => self.connection.execute(
        &format!("CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * FROM read_csv_auto(?)"),
        [path_string],
      ),
      (Some(delimiter), None) => self.connection.execute(
        &format!("CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * FROM read_csv_auto(?, delim=?)"),
        params![path_string, delimiter],
      ),
      (None, Some(has_header)) => self.connection.execute(
        &format!("CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * FROM read_csv_auto(?, header=?)"),
        params![path_string, has_header],
      ),
      (Some(delimiter), Some(has_header)) => self.connection.execute(
        &format!(
          "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * FROM read_csv_auto(?, delim=?, header=?)"
        ),
        params![path_string, delimiter, has_header],
      ),
    };
    if load_result.is_err() {
      self.drop_staging();
      return Err(RuntimeError::CsvRead {
        path: path.to_owned(),
      });
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::CsvSchemaRead {
          path: path.to_owned(),
        });
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::CsvRowCount {
          path: path.to_owned(),
        });
      }
    };

    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(RuntimeError::CsvTransaction {
        path: path.to_owned(),
      });
    }

    Ok(DatasetInfo {
      source: path.to_owned(),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
  pub fn load_eager_parquet(&mut self, path: &Path) -> Result<DatasetInfo, RuntimeError> {
    let path_string = path.to_str().ok_or_else(|| RuntimeError::NonUtf8Path {
      path: path.to_owned(),
    })?;

    self.drop_staging();
    if self
      .connection
      .execute(
        &format!("CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * FROM read_parquet(?)"),
        [path_string],
      )
      .is_err()
    {
      self.drop_staging();
      return Err(RuntimeError::ParquetRead {
        path: path.to_owned(),
      });
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::SchemaRead {
          path: path.to_owned(),
        });
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(DataError) => {
        self.drop_staging();
        return Err(RuntimeError::RowCount {
          path: path.to_owned(),
        });
      }
    };

    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(RuntimeError::Transaction {
        path: path.to_owned(),
      });
    }

    Ok(DatasetInfo {
      source: path.to_owned(),
      row_count,
      columns,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    })
  }
}
