#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use duckdb::types::ValueRef;
use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine, RowLimit};

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

/// The owned result returned by a read-only `describe` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescribeResult {
  /// The unchanged metadata for the active dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after counting rows in the active dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountResult {
  /// The row count observed in the active relation.
  pub row_count: u64,
}

/// An owned descriptive-statistics row for one numeric column.
#[derive(Debug, Clone, PartialEq)]
pub struct SummaryRow {
  /// The requested column name.
  pub variable: String,
  /// The number of non-null values in the column.
  pub count: u64,
  /// The arithmetic mean, or `None` when there are no non-null values.
  pub mean: Option<f64>,
  /// The sample standard deviation, or `None` for fewer than two values.
  pub std_dev: Option<f64>,
  /// The minimum non-null value, preserving its owned DuckDB scalar type.
  pub minimum: Option<CellValue>,
  /// The maximum non-null value, preserving its owned DuckDB scalar type.
  pub maximum: Option<CellValue>,
}

/// The owned result returned by a read-only `summarize` request.
#[derive(Debug, Clone, PartialEq)]
pub struct SummarizeResult {
  /// Rows in requested order, including repeated explicit variables.
  pub rows: Vec<SummaryRow>,
}

/// An owned codebook profile row for one active column.
#[derive(Debug, Clone, PartialEq)]
pub struct CodebookRow {
  /// The requested column name.
  pub variable: String,
  /// The schema type reported by DuckDB when the relation was loaded.
  pub data_type: String,
  /// The number of non-null values in the column.
  pub nonmissing: u64,
  /// The number of null values in the column.
  pub missing: u64,
  /// The number of distinct non-null values in the column.
  pub distinct: u64,
  /// Up to three non-null values copied into the owned value boundary.
  pub examples: Vec<CellValue>,
}

/// The owned result returned by a read-only `codebook` request.
#[derive(Debug, Clone, PartialEq)]
pub struct CodebookResult {
  /// Rows in schema or requested order, including repeated explicit variables.
  pub rows: Vec<CodebookRow>,
}

/// An owned missingness row for one active column.
#[derive(Debug, Clone, PartialEq)]
pub struct MissingRow {
  /// The requested column name.
  pub variable: String,
  /// The schema type reported by DuckDB when the relation was loaded.
  pub data_type: String,
  /// The total number of rows in the active relation.
  pub total: u64,
  /// The number of SQL NULL values in the column.
  pub missing: u64,
  /// The number of non-NULL values in the column.
  pub nonmissing: u64,
  /// The missing percentage, or zero for an empty relation.
  pub missing_percent: f64,
}

/// The owned result returned by a read-only `missing` request.
#[derive(Debug, Clone, PartialEq)]
pub struct MissingResult {
  /// Rows in schema or requested order, including repeated explicit variables.
  pub rows: Vec<MissingRow>,
}

/// An owned scalar value in a bounded dataset preview.
///
/// Values are copied out of DuckDB before the result leaves the runtime so a
/// caller never depends on a backend row or connection lifetime.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
  /// An SQL NULL value.
  Null,
  /// A boolean value.
  Boolean(bool),
  /// A signed integer value, widened without loss of precision.
  SignedInteger(i128),
  /// An unsigned integer value, widened without loss of precision.
  UnsignedInteger(u128),
  /// A floating-point value.
  Float(f64),
  /// A decimal value with its declared width, scale, and scaled payload.
  Decimal { width: u8, scale: u8, value: i128 },
  /// A UTF-8 text value.
  Text(String),
  /// An owned binary value.
  Bytes(Vec<u8>),
}

/// The owned result returned by a bounded preview request.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewResult {
  /// Column names in the active relation's schema order.
  pub columns: Vec<String>,
  /// Rows in relation insertion order, limited to the requested preview.
  pub rows: Vec<Vec<CellValue>>,
}

/// Results currently exposed by the bounded runtime slice.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionResult {
  /// A successfully loaded dataset.
  Load(LoadResult),
  /// The metadata for the currently active dataset.
  Describe(DescribeResult),
  /// The row count for the currently active dataset.
  Count(CountResult),
  /// Descriptive statistics for selected numeric columns.
  Summarize(SummarizeResult),
  /// Profiles selected columns in the active dataset.
  Codebook(CodebookResult),
  /// Reports SQL-NULL missingness for selected columns.
  Missing(MissingResult),
  /// The requested prefix of rows from the currently active dataset.
  Head(PreviewResult),
  /// The requested suffix of rows from the currently active dataset.
  Tail(PreviewResult),
}

/// Errors produced by the bounded runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
  /// The runtime does not yet execute the named language command.
  UnsupportedCommand { name: &'static str },
  /// The command requires a dataset, but the session has not loaded one.
  NoActiveDataset { command: &'static str },
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
  /// DuckDB could not produce the requested preview.
  PreviewFailed { command: &'static str },
  /// The summary request named variables absent from the active schema.
  SummaryUnknownVariable { variables: Vec<String> },
  /// The summary request named variables that are not numeric.
  SummaryRequiresNumeric { variables: Vec<String> },
  /// The active schema contains no numeric columns for a default summary.
  SummaryNoNumericColumns,
  /// DuckDB could not produce or own one summary row.
  SummaryFailed { variable: String },
  /// The codebook request named variables absent from the active schema.
  CodebookUnknownVariable { variables: Vec<String> },
  /// DuckDB could not produce or own one codebook row.
  CodebookFailed { variable: String },
  /// The missingness request named variables absent from the active schema.
  MissingUnknownVariable { variables: Vec<String> },
  /// DuckDB could not produce the missingness aggregate.
  MissingFailed,
}

impl fmt::Display for RuntimeError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnsupportedCommand { name } => {
        write!(formatter, "runtime does not execute command: {name}")
      }
      Self::NoActiveDataset { command } => write!(
        formatter,
        "{command} requires an active dataset; run use <path> first"
      ),
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
      Self::PreviewFailed { command } => write!(formatter, "{command} failed"),
      Self::SummaryUnknownVariable { variables } => write!(
        formatter,
        "summarize unknown variable: {}",
        variables.join(", ")
      ),
      Self::SummaryRequiresNumeric { variables } => write!(
        formatter,
        "summarize requires numeric variables: {}",
        variables.join(", ")
      ),
      Self::SummaryNoNumericColumns => formatter.write_str("summarize found no numeric columns"),
      Self::SummaryFailed { variable } => {
        write!(formatter, "summarize failed for variable: {variable}")
      }
      Self::CodebookUnknownVariable { variables } => {
        write!(
          formatter,
          "codebook unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::CodebookFailed { variable } => {
        write!(formatter, "codebook failed for variable: {variable}")
      }
      Self::MissingUnknownVariable { variables } => {
        write!(
          formatter,
          "missing unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::MissingFailed => formatter.write_str("missing failed"),
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
      Command::Describe => self.execute_describe(),
      Command::Count => self.execute_count(),
      Command::Summarize { variables } => self.execute_summarize(variables),
      Command::Codebook { variables } => self.execute_codebook(variables),
      Command::Missing { variables } => self.execute_missing(variables),
      Command::Head { limit } => self.execute_head(limit),
      Command::Tail { limit } => self.execute_tail(limit),
      _ => Err(RuntimeError::UnsupportedCommand { name: command_name }),
    }
  }

  /// Return the currently published dataset metadata, if any.
  pub fn active_dataset(&self) -> Option<&DatasetInfo> {
    self.active_dataset.as_ref()
  }

  fn execute_describe(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "describe",
      })?;
    Ok(ExecutionResult::Describe(DescribeResult {
      dataset: dataset.clone(),
    }))
  }

  fn execute_count(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "count" })?
      .clone();
    Ok(ExecutionResult::Count(CountResult {
      row_count: dataset.row_count,
    }))
  }

  fn execute_summarize(&self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "summarize",
      })?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SummaryUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .filter(|column| is_numeric_data_type(&column.data_type))
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    if requested.is_empty() {
      return Err(RuntimeError::SummaryNoNumericColumns);
    }

    let non_numeric = requested
      .iter()
      .filter(|variable| {
        column_types
          .get(variable.as_str())
          .is_none_or(|data_type| !is_numeric_data_type(data_type))
      })
      .cloned()
      .collect::<Vec<_>>();
    if !non_numeric.is_empty() {
      return Err(RuntimeError::SummaryRequiresNumeric {
        variables: non_numeric,
      });
    }

    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::SummaryFailed {
        variable: requested[0].clone(),
      })?;
    let rows = requested
      .iter()
      .map(|variable| {
        backend
          .summarize_variable(variable)
          .map_err(|_| RuntimeError::SummaryFailed {
            variable: variable.clone(),
          })
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecutionResult::Summarize(SummarizeResult { rows }))
  }

  fn execute_codebook(&self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "codebook",
      })?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::CodebookUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| RuntimeError::CodebookFailed {
        variable: requested
          .first()
          .cloned()
          .unwrap_or_else(|| "<none>".to_owned()),
      })?;
    let rows = requested
      .iter()
      .map(|variable| {
        let data_type = column_types
          .get(variable.as_str())
          .copied()
          .ok_or_else(|| RuntimeError::CodebookFailed {
            variable: variable.clone(),
          })?;
        backend
          .codebook_variable(variable, data_type)
          .map_err(|_| RuntimeError::CodebookFailed {
            variable: variable.clone(),
          })
      })
      .collect::<Result<Vec<_>, _>>()?;
    Ok(ExecutionResult::Codebook(CodebookResult { rows }))
  }

  fn execute_missing(&self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "missing" })?;
    let column_types = dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<std::collections::HashMap<_, _>>();
    let unknown = variables
      .iter()
      .filter(|variable| !column_types.contains_key(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::MissingUnknownVariable { variables: unknown });
    }

    let requested = if variables.is_empty() {
      dataset
        .columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<Vec<_>>()
    } else {
      variables
    };
    let typed_requested = requested
      .iter()
      .map(|variable| {
        Ok((
          variable.as_str(),
          column_types
            .get(variable.as_str())
            .copied()
            .ok_or(RuntimeError::MissingFailed)?,
        ))
      })
      .collect::<Result<Vec<_>, _>>()?;
    let backend = self.backend.as_ref().ok_or(RuntimeError::MissingFailed)?;
    let rows = backend
      .missingness(&typed_requested)
      .map_err(|_| RuntimeError::MissingFailed)?;
    Ok(ExecutionResult::Missing(MissingResult { rows }))
  }

  fn execute_head(&self, limit: RowLimit) -> Result<ExecutionResult, RuntimeError> {
    self
      .execute_preview(limit, "head", false)
      .map(ExecutionResult::Head)
  }

  fn execute_tail(&self, limit: RowLimit) -> Result<ExecutionResult, RuntimeError> {
    self
      .execute_preview(limit, "tail", true)
      .map(ExecutionResult::Tail)
  }

  fn execute_preview(
    &self,
    limit: RowLimit,
    command: &'static str,
    from_end: bool,
  ) -> Result<PreviewResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command })?;
    let limit = limit
      .as_decimal()
      .parse::<i64>()
      .map_err(|_| RuntimeError::PreviewFailed { command })?;
    let columns = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if limit == 0 {
      return Ok(PreviewResult {
        columns,
        rows: Vec::new(),
      });
    }

    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::PreviewFailed { command })?;
    let rows = backend
      .preview_rows(limit, dataset.columns.len(), from_end)
      .map_err(|_| RuntimeError::PreviewFailed { command })?;
    Ok(PreviewResult { columns, rows })
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
    Command::Codebook { .. } => "codebook",
    Command::Datasignature => "datasignature",
    Command::Missing { .. } => "missing",
    Command::Duplicates { .. } => "duplicates",
    Command::Isid { .. } => "isid",
    Command::Select { .. } => "select",
    Command::Sort { .. } => "sort",
    Command::Gsort { .. } => "gsort",
    Command::Rename { .. } => "rename",
    Command::Run { .. } => "run",
    Command::Set { .. } => "set",
    Command::Save { .. } => "save",
    Command::Export { .. } => "export",
    Command::Use { .. } => "use",
    Command::Count => "count",
    Command::Head { .. } => "head",
    Command::Tail { .. } => "tail",
  }
}

fn is_numeric_data_type(data_type: &str) -> bool {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  matches!(
    base,
    "TINYINT"
      | "SMALLINT"
      | "INTEGER"
      | "BIGINT"
      | "HUGEINT"
      | "UHUGEINT"
      | "UTINYINT"
      | "USMALLINT"
      | "UINTEGER"
      | "UBIGINT"
      | "INT8"
      | "INT16"
      | "INT32"
      | "INT64"
      | "UINT8"
      | "UINT16"
      | "UINT32"
      | "UINT64"
      | "UINT128"
      | "FLOAT32"
      | "FLOAT64"
      | "FLOAT"
      | "REAL"
      | "DOUBLE"
      | "DECIMAL"
  )
}

fn quote_identifier(identifier: &str) -> String {
  format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn validate_local_parquet_path(path: &Path) -> Result<(), RuntimeError> {
  let extension_is_parquet = path
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| extension.eq_ignore_ascii_case("parquet"));
  if !extension_is_parquet {
    return Err(RuntimeError::UnsupportedFormat {
      path: path.to_owned(),
    });
  }
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

  fn preview_rows(
    &self,
    limit: i64,
    column_count: usize,
    from_end: bool,
  ) -> Result<Vec<Vec<CellValue>>, ()> {
    let order = if from_end { "DESC" } else { "ASC" };
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT * FROM (SELECT row_number() OVER () AS __tabdat_preview_order, * FROM {ACTIVE_TABLE}) ORDER BY 1 {order} LIMIT ?"
      ))
      .map_err(|_| ())?;
    let mut rows = statement.query([limit]).map_err(|_| ())?;
    let mut preview = Vec::new();
    while let Some(row) = rows.next().map_err(|_| ())? {
      let mut values = Vec::with_capacity(column_count);
      for index in 0..column_count {
        let value = row.get_ref(index + 1).map_err(|_| ())?;
        values.push(cell_value_from_ref(value)?);
      }
      preview.push(values);
    }
    if from_end {
      preview.reverse();
    }
    Ok(preview)
  }

  fn summarize_variable(&self, variable: &str) -> Result<SummaryRow, ()> {
    let quoted_variable = quote_identifier(variable);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT count({quoted_variable}), avg({quoted_variable}), stddev_samp({quoted_variable}), min({quoted_variable}), max({quoted_variable}) FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let count: i64 = row.get(0).map_err(|_| ())?;
    let count = u64::try_from(count).map_err(|_| ())?;
    let mean: Option<f64> = row.get(1).map_err(|_| ())?;
    let std_dev: Option<f64> = row.get(2).map_err(|_| ())?;
    let minimum = match row.get_ref(3).map_err(|_| ())? {
      ValueRef::Null => None,
      value => Some(cell_value_from_ref(value)?),
    };
    let maximum = match row.get_ref(4).map_err(|_| ())? {
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

  fn codebook_variable(&self, variable: &str, data_type: &str) -> Result<CodebookRow, ()> {
    let quoted_variable = quote_identifier(variable);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT count({quoted_variable}), count(*) - count({quoted_variable}), count(DISTINCT {quoted_variable}) FROM {ACTIVE_TABLE}"
      ))
      .map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let nonmissing = u64::try_from(row.get::<_, i64>(0).map_err(|_| ())?).map_err(|_| ())?;
    let missing = u64::try_from(row.get::<_, i64>(1).map_err(|_| ())?).map_err(|_| ())?;
    let distinct = u64::try_from(row.get::<_, i64>(2).map_err(|_| ())?).map_err(|_| ())?;

    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT {quoted_variable} FROM {ACTIVE_TABLE} WHERE {quoted_variable} IS NOT NULL LIMIT 3"
      ))
      .map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let mut examples = Vec::new();
    while let Some(row) = rows.next().map_err(|_| ())? {
      examples.push(cell_value_from_ref(row.get_ref(0).map_err(|_| ())?)?);
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

  fn missingness(&self, variables: &[(&str, &str)]) -> Result<Vec<MissingRow>, ()> {
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
      .map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let total = u64::try_from(row.get::<_, i64>(0).map_err(|_| ())?).map_err(|_| ())?;
    variables
      .iter()
      .enumerate()
      .map(|(index, (variable, data_type))| {
        let nonmissing =
          u64::try_from(row.get::<_, i64>(index + 1).map_err(|_| ())?).map_err(|_| ())?;
        let missing = total.checked_sub(nonmissing).ok_or(())?;
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
}

fn cell_value_from_ref(value: ValueRef<'_>) -> Result<CellValue, ()> {
  match value {
    ValueRef::Null => Ok(CellValue::Null),
    ValueRef::Boolean(value) => Ok(CellValue::Boolean(value)),
    ValueRef::TinyInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::SmallInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::Int(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::BigInt(value) => Ok(CellValue::SignedInteger(i128::from(value))),
    ValueRef::HugeInt(value) => Ok(CellValue::SignedInteger(value)),
    ValueRef::UTinyInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::USmallInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UBigInt(value) => Ok(CellValue::UnsignedInteger(u128::from(value))),
    ValueRef::UHugeInt(value) => Ok(CellValue::UnsignedInteger(value)),
    ValueRef::Float(value) => Ok(CellValue::Float(f64::from(value))),
    ValueRef::Double(value) => Ok(CellValue::Float(value)),
    ValueRef::Decimal(value) => Ok(CellValue::Decimal {
      width: value.width(),
      scale: value.scale(),
      value: value.value(),
    }),
    ValueRef::Text(value) => String::from_utf8(value.to_vec())
      .map(CellValue::Text)
      .map_err(|_| ()),
    ValueRef::Blob(value) => Ok(CellValue::Bytes(value.to_vec())),
    _ => Err(()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicU64, Ordering};
  use std::time::{SystemTime, UNIX_EPOCH};

  static NEXT_MISSING_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

  #[test]
  fn describe_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session.execute(Command::Describe).unwrap_err(),
      RuntimeError::NoActiveDataset {
        command: "describe"
      }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn count_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session.execute(Command::Count).unwrap_err(),
      RuntimeError::NoActiveDataset { command: "count" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn head_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Head {
          limit: RowLimit::default(),
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "head" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn tail_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Tail {
          limit: RowLimit::default(),
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "tail" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn codebook_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Codebook {
          variables: Vec::new()
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset {
        command: "codebook"
      }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn missing_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Missing {
          variables: Vec::new(),
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "missing" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn failed_preview_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![ColumnInfo {
        name: "value".to_owned(),
        data_type: "INTEGER".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    session.active_dataset = Some(dataset.clone());
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
      ))
      .expect("the test active relation should be created");
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
      .expect("the test active relation should be dropped");

    assert_eq!(
      session
        .execute(Command::Head {
          limit: RowLimit::default(),
        })
        .unwrap_err(),
      RuntimeError::PreviewFailed { command: "head" }
    );
    assert_eq!(
      session
        .execute(Command::Tail {
          limit: RowLimit::default(),
        })
        .unwrap_err(),
      RuntimeError::PreviewFailed { command: "tail" }
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  }

  #[test]
  fn failed_summary_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![ColumnInfo {
        name: "value".to_owned(),
        data_type: "INTEGER".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    session.active_dataset = Some(dataset.clone());

    assert_eq!(
      session
        .execute(Command::Summarize {
          variables: vec!["value".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::SummaryFailed {
        variable: "value".to_owned(),
      }
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  }

  #[test]
  fn failed_codebook_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![ColumnInfo {
        name: "value".to_owned(),
        data_type: "INTEGER".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    session.active_dataset = Some(dataset.clone());
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
      ))
      .expect("the test active relation should be created");
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
      .expect("the test active relation should be dropped");

    assert_eq!(
      session
        .execute(Command::Codebook {
          variables: vec!["value".to_owned()]
        })
        .unwrap_err(),
      RuntimeError::CodebookFailed {
        variable: "value".to_owned()
      }
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  }

  #[test]
  fn failed_missing_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![ColumnInfo {
        name: "value".to_owned(),
        data_type: "INTEGER".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    session.active_dataset = Some(dataset.clone());
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS value"
      ))
      .expect("the test active relation should be created");
    session
      .backend
      .as_mut()
      .expect("test backend should exist")
      .connection
      .execute_batch(&format!("DROP TABLE {ACTIVE_TABLE}"))
      .expect("the test active relation should be dropped");

    assert_eq!(
      session
        .execute(Command::Missing {
          variables: vec!["value".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::MissingFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
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

    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_MISSING_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
      "tabdat-runtime-missing-{}-{nonce}-{fixture_id}.parquet",
      std::process::id()
    ));
    assert!(!path.exists());
    assert!(matches!(
      backend.load_eager_parquet(&path),
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
