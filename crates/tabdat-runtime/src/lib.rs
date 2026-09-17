#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine};

const ACTIVE_TABLE: &str = "__tabdat_active";
const STAGING_TABLE: &str = "__tabdat_next";

/// A column in the loaded dataset schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
  /// The source column name, in schema order.
  pub name: String,
  /// The DuckDB logical type name.
  pub data_type: String,
}

/// Owned metadata for the active dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetInfo {
  /// The local source path used for the load.
  pub source: PathBuf,
  /// The number of rows observed while staging the relation.
  pub row_count: u64,
  /// The ordered schema observed while staging the relation.
  pub columns: Vec<ColumnInfo>,
  /// The execution mode used to load the relation.
  pub execution_mode: ExecutionMode,
  /// The lazy engine, if one was selected. Eager loads have no lazy engine.
  pub lazy_engine: Option<LazyEngine>,
}

/// The owned result returned after an eager Parquet load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadResult {
  /// Metadata for the newly active dataset.
  pub dataset: DatasetInfo,
}

/// Results currently exposed by the bounded runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResult {
  /// A successfully loaded dataset.
  Load(LoadResult),
}

/// Errors produced by the bounded runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
  /// The runtime does not yet execute the named language command.
  UnsupportedCommand { name: &'static str },
  /// The request uses a source, mode, or option outside this slice.
  UnsupportedUseConfiguration,
  /// The source path does not have a UTF-8 representation accepted by DuckDB.
  NonUtf8Path { path: PathBuf },
  /// The source path does not exist.
  FileNotFound { path: PathBuf },
  /// The source path exists but is not a regular file.
  NotAFile { path: PathBuf },
  /// The source path is not a local Parquet file.
  UnsupportedFormat { path: PathBuf },
  /// DuckDB could not read the staged Parquet relation.
  ParquetRead { path: PathBuf },
  /// DuckDB could not inspect the staged schema.
  SchemaRead { path: PathBuf },
  /// DuckDB returned an invalid row count.
  RowCount { path: PathBuf },
  /// DuckDB could not atomically publish the staged relation.
  Transaction { path: PathBuf },
  /// DuckDB could not initialize its in-memory connection.
  BackendInitialization,
}

impl fmt::Display for RuntimeError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnsupportedCommand { name } => {
        write!(formatter, "runtime does not execute command: {name}")
      }
      Self::UnsupportedUseConfiguration => {
        formatter.write_str("use runtime slice supports only eager local Parquet loads")
      }
      Self::NonUtf8Path { path } => {
        write!(formatter, "use path is not valid UTF-8: {}", path.display())
      }
      Self::FileNotFound { path } => {
        write!(formatter, "use could not find file: {}", path.display())
      }
      Self::NotAFile { path } => write!(formatter, "use expected a file path: {}", path.display()),
      Self::UnsupportedFormat { path: _ } => {
        formatter.write_str("use runtime slice supports only local .parquet files")
      }
      Self::ParquetRead { path } => {
        write!(
          formatter,
          "use could not read Parquet file: {}",
          path.display()
        )
      }
      Self::SchemaRead { path } => {
        write!(
          formatter,
          "use could not inspect Parquet schema: {}",
          path.display()
        )
      }
      Self::RowCount { path } => {
        write!(
          formatter,
          "use could not count Parquet rows: {}",
          path.display()
        )
      }
      Self::Transaction { path } => write!(
        formatter,
        "use could not publish Parquet relation: {}",
        path.display()
      ),
      Self::BackendInitialization => formatter.write_str("use could not initialize DuckDB"),
    }
  }
}

impl Error for RuntimeError {}

/// A session holding optional active metadata and a private DuckDB adapter.
pub struct Session {
  backend: Option<DuckDbBackend>,
  active_dataset: Option<DatasetInfo>,
}

impl Session {
  /// Construct a session without initializing DuckDB.
  pub fn new() -> Self {
    Self {
      backend: None,
      active_dataset: None,
    }
  }

  /// Execute one command supported by this bounded runtime slice.
  pub fn execute(&mut self, command: Command) -> Result<ExecutionResult, RuntimeError> {
    let command_name = command_name(&command);
    match command {
      Command::Use {
        source,
        execution_mode,
        lazy_engine,
        delimiter,
        has_header,
      } => self.execute_use(source, execution_mode, lazy_engine, delimiter, has_header),
      _ => Err(RuntimeError::UnsupportedCommand { name: command_name }),
    }
  }

  /// Return the currently published dataset metadata, if any.
  pub fn active_dataset(&self) -> Option<&DatasetInfo> {
    self.active_dataset.as_ref()
  }

  fn execute_use(
    &mut self,
    source: DataSource,
    execution_mode: ExecutionMode,
    lazy_engine: Option<LazyEngine>,
    delimiter: Option<String>,
    has_header: Option<bool>,
  ) -> Result<ExecutionResult, RuntimeError> {
    if execution_mode != ExecutionMode::Eager
      || lazy_engine.is_some()
      || delimiter.is_some()
      || has_header.is_some()
    {
      return Err(RuntimeError::UnsupportedUseConfiguration);
    }

    let DataSource::LocalPath(raw_path) = source else {
      return Err(RuntimeError::UnsupportedUseConfiguration);
    };
    let path = PathBuf::from(raw_path);
    validate_local_parquet_path(&path)?;

    let dataset = if let Some(backend) = self.backend.as_mut() {
      backend.load_eager_parquet(&path)?
    } else {
      let mut backend = DuckDbBackend::new()?;
      let dataset = backend.load_eager_parquet(&path)?;
      self.backend = Some(backend);
      dataset
    };
    self.active_dataset = Some(dataset.clone());
    Ok(ExecutionResult::Load(LoadResult { dataset }))
  }
}

impl Default for Session {
  fn default() -> Self {
    Self::new()
  }
}

fn command_name(command: &Command) -> &'static str {
  match command {
    Command::Help { .. } => "help",
    Command::Status => "status",
    Command::Exit => "exit",
    Command::Describe => "describe",
    Command::Doctor => "doctor",
    Command::Summarize { .. } => "summarize",
    Command::Datasignature => "datasignature",
    Command::Codebook { .. } => "codebook",
    Command::Missing { .. } => "missing",
    Command::Duplicates { .. } => "duplicates",
    Command::Set { .. } => "set",
    Command::Use { .. } => "use",
    Command::Count => "count",
    Command::Head { .. } => "head",
    Command::Tail { .. } => "tail",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_session_defers_backend_initialization() {
    let session = Session::new();

    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn failed_staged_read_keeps_the_private_active_relation() {
    let mut backend = DuckDbBackend::new().expect("test backend should initialize");
    backend
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
      ))
      .expect("the test active relation should be created");

    let path = Path::new("__tabdat_runtime_missing_fixture__.parquet");
    assert!(matches!(
      backend.load_eager_parquet(path),
      Err(RuntimeError::ParquetRead { .. })
    ));

    let value: i32 = backend
      .connection
      .query_row(&format!("SELECT value FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
  }
}

fn validate_local_parquet_path(path: &Path) -> Result<(), RuntimeError> {
  if !path.exists() {
    return Err(RuntimeError::FileNotFound {
      path: path.to_owned(),
    });
  }
  if !path.is_file() {
    return Err(RuntimeError::NotAFile {
      path: path.to_owned(),
    });
  }
  let extension_is_parquet = path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| extension.eq_ignore_ascii_case("parquet"));
  if !extension_is_parquet {
    return Err(RuntimeError::UnsupportedFormat {
      path: path.to_owned(),
    });
  }
  Ok(())
}

struct DuckDbBackend {
  connection: Connection,
}

impl DuckDbBackend {
  fn new() -> Result<Self, RuntimeError> {
    let connection =
      Connection::open_in_memory().map_err(|_| RuntimeError::BackendInitialization)?;
    connection
      .execute_batch("SET preserve_insertion_order = true")
      .map_err(|_| RuntimeError::BackendInitialization)?;
    Ok(Self { connection })
  }

  fn load_eager_parquet(&mut self, path: &Path) -> Result<DatasetInfo, RuntimeError> {
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
      Err(()) => {
        self.drop_staging();
        return Err(RuntimeError::SchemaRead {
          path: path.to_owned(),
        });
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
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

  fn staged_columns(&self) -> Result<Vec<ColumnInfo>, ()> {
    let mut statement = self
      .connection
      .prepare(&format!("DESCRIBE {STAGING_TABLE}"))
      .map_err(|_| ())?;
    let rows = statement
      .query_map([], |row| {
        Ok(ColumnInfo {
          name: row.get(0)?,
          data_type: row.get(1)?,
        })
      })
      .map_err(|_| ())?;
    rows.map(|row| row.map_err(|_| ())).collect()
  }

  fn staged_row_count(&self) -> Result<u64, ()> {
    let row_count: i64 = self
      .connection
      .query_row(
        &format!("SELECT COUNT(*) FROM {STAGING_TABLE}"),
        [],
        |row| row.get(0),
      )
      .map_err(|_| ())?;
    u64::try_from(row_count).map_err(|_| ())
  }

  fn publish_staging(&mut self) -> Result<(), ()> {
    self
      .connection
      .execute_batch("BEGIN TRANSACTION")
      .map_err(|_| ())?;
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
      return Err(());
    }
    Ok(())
  }

  fn drop_staging(&mut self) {
    let _ = self
      .connection
      .execute_batch(&format!("DROP TABLE IF EXISTS {STAGING_TABLE}"));
  }
}
