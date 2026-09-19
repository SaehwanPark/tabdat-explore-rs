#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use duckdb::types::{TimeUnit, Value, ValueRef};
use sha2::{Digest, Sha256};
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

/// The owned duplicate-key aggregate returned by a read-only request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicatesResult {
  /// Key variables in schema or requested order, including repeated requests.
  pub variables: Vec<String>,
  /// The number of rows in the active relation.
  pub total_rows: u64,
  /// The number of distinct key groups, including groups containing NULLs.
  pub unique_groups: u64,
  /// The number of groups containing at least two rows.
  pub duplicate_groups: u64,
  /// The number of rows belonging to duplicate groups.
  pub duplicate_rows: u64,
  /// The surplus rows after retaining one row per duplicate group.
  pub extra_rows: u64,
  /// The largest number of rows in any key group.
  pub max_copies: u64,
}

/// The owned key-uniqueness aggregate returned by a read-only `isid` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsidResult {
  /// Key variables in the requested order, including repeated requests.
  pub variables: Vec<String>,
  /// The number of rows in the active relation.
  pub total_rows: u64,
  /// The number of distinct key groups, including groups containing NULLs.
  pub unique_groups: u64,
  /// The number of rows containing at least one NULL key component.
  pub missing_key_rows: u64,
  /// Whether missing key components were permitted for this request.
  pub missok: bool,
}

/// The owned reproducibility fingerprint returned by a read-only request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasignatureResult {
  /// The digest algorithm named by the versioned signature protocol.
  pub algorithm: String,
  /// The lowercase hexadecimal digest of the public active relation.
  pub signature: String,
  /// The number of rows consumed by the signature scan.
  pub row_count: u64,
  /// The number of public columns included in the schema prefix.
  pub column_count: u64,
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
  /// Reports duplicate groups for selected key columns.
  Duplicates(DuplicatesResult),
  /// Asserts key uniqueness for selected variables.
  Isid(IsidResult),
  /// Computes a reproducibility fingerprint for the active relation.
  Datasignature(DatasignatureResult),
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
  /// The duplicate request named variables absent from the active schema.
  DuplicatesUnknownVariable { variables: Vec<String> },
  /// DuckDB could not produce the duplicate aggregate.
  DuplicatesFailed,
  /// The isid request named variables absent from the active schema.
  IsidUnknownVariable { variables: Vec<String> },
  /// The isid request did not name any key variables.
  IsidNoVariables,
  /// The active key values violate the requested isid constraints.
  IsidSemanticFailure {
    missing_key_rows: u64,
    duplicate_rows: u64,
    duplicate_groups: u64,
    missok: bool,
  },
  /// DuckDB could not produce the isid aggregate.
  IsidFailed,
  /// DuckDB could not produce the datasignature scan or encoding.
  DatasignatureFailed,
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
      Self::DuplicatesUnknownVariable { variables } => {
        write!(
          formatter,
          "duplicates unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::DuplicatesFailed => formatter.write_str("duplicates failed"),
      Self::IsidUnknownVariable { variables } => {
        write!(formatter, "isid unknown variable: {}", variables.join(", "))
      }
      Self::IsidNoVariables => formatter.write_str("isid expects at least one key variable"),
      Self::IsidSemanticFailure {
        missing_key_rows,
        duplicate_rows,
        duplicate_groups,
        missok,
      } => {
        let mut failures = Vec::new();
        if !missok && *missing_key_rows > 0 {
          failures.push(format!(
            "{missing_key_rows} rows have missing key values (use , missok to permit them)"
          ));
        }
        if *duplicate_groups > 0 {
          failures.push(format!(
            "{duplicate_rows} rows are in {duplicate_groups} duplicate key groups"
          ));
        }
        write!(formatter, "isid failed: {}", failures.join("; "))
      }
      Self::IsidFailed => formatter.write_str("isid failed"),
      Self::DatasignatureFailed => formatter.write_str("datasignature failed"),
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
      Command::Duplicates { variables } => self.execute_duplicates(variables),
      Command::Isid { variables, missok } => self.execute_isid(variables, missok),
      Command::Datasignature => self.execute_datasignature(),
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

  fn execute_duplicates(&self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "duplicates",
      })?;
    let known = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::HashSet<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| !known.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::DuplicatesUnknownVariable { variables: unknown });
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
      .ok_or(RuntimeError::DuplicatesFailed)?;
    backend
      .duplicates(&requested)
      .map(ExecutionResult::Duplicates)
      .map_err(|_| RuntimeError::DuplicatesFailed)
  }

  fn execute_isid(
    &self,
    variables: Vec<String>,
    missok: bool,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "isid" })?;
    if variables.is_empty() {
      return Err(RuntimeError::IsidNoVariables);
    }
    let known = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::HashSet<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| !known.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::IsidUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_ref().ok_or(RuntimeError::IsidFailed)?;
    let (total_rows, unique_groups, duplicate_groups, duplicate_rows, missing_key_rows) = backend
      .isid_counts(&variables)
      .map_err(|_| RuntimeError::IsidFailed)?;
    if (!missok && missing_key_rows > 0) || duplicate_groups > 0 {
      return Err(RuntimeError::IsidSemanticFailure {
        missing_key_rows,
        duplicate_rows,
        duplicate_groups,
        missok,
      });
    }
    Ok(ExecutionResult::Isid(IsidResult {
      variables,
      total_rows,
      unique_groups,
      missing_key_rows,
      missok,
    }))
  }

  fn execute_datasignature(&self) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "datasignature",
      })?;
    let backend = self
      .backend
      .as_ref()
      .ok_or(RuntimeError::DatasignatureFailed)?;
    let (signature, row_count) = backend
      .datasignature(&dataset.columns)
      .map_err(|_| RuntimeError::DatasignatureFailed)?;
    let column_count =
      u64::try_from(dataset.columns.len()).map_err(|_| RuntimeError::DatasignatureFailed)?;
    Ok(ExecutionResult::Datasignature(DatasignatureResult {
      algorithm: "sha256".to_owned(),
      signature,
      row_count,
      column_count,
    }))
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

  fn duplicates(&self, variables: &[String]) -> Result<DuplicatesResult, ()> {
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
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let total_rows = u64::try_from(row.get::<_, i128>(0).map_err(|_| ())?).map_err(|_| ())?;
    let unique_groups = u64::try_from(row.get::<_, i64>(1).map_err(|_| ())?).map_err(|_| ())?;
    let duplicate_groups = u64::try_from(row.get::<_, i64>(2).map_err(|_| ())?).map_err(|_| ())?;
    let duplicate_rows = u64::try_from(row.get::<_, i128>(3).map_err(|_| ())?).map_err(|_| ())?;
    let extra_rows = u64::try_from(row.get::<_, i128>(4).map_err(|_| ())?).map_err(|_| ())?;
    let max_copies = u64::try_from(row.get::<_, i128>(5).map_err(|_| ())?).map_err(|_| ())?;
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

  fn isid_counts(&self, variables: &[String]) -> Result<(u64, u64, u64, u64, u64), ()> {
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
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let total_rows = u64::try_from(row.get::<_, i128>(0).map_err(|_| ())?).map_err(|_| ())?;
    let unique_groups = u64::try_from(row.get::<_, i64>(1).map_err(|_| ())?).map_err(|_| ())?;
    let duplicate_groups = u64::try_from(row.get::<_, i64>(2).map_err(|_| ())?).map_err(|_| ())?;
    let duplicate_rows = u64::try_from(row.get::<_, i128>(3).map_err(|_| ())?).map_err(|_| ())?;
    let missing_key_rows = u64::try_from(row.get::<_, i128>(4).map_err(|_| ())?).map_err(|_| ())?;
    Ok((
      total_rows,
      unique_groups,
      duplicate_groups,
      duplicate_rows,
      missing_key_rows,
    ))
  }

  fn datasignature(&self, columns: &[ColumnInfo]) -> Result<(String, u64), ()> {
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
      .map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let mut row_count = 0_u64;
    while let Some(row) = rows.next().map_err(|_| ())? {
      update_signature_token(&mut digest, b"row");
      for (index, column) in columns.iter().enumerate() {
        let value = row.get_ref(index).map_err(|_| ())?.to_owned();
        let encoded = signature_value(&value, Some(&column.data_type)).map_err(|_| ())?;
        update_signature_token(&mut digest, &encoded);
      }
      row_count = row_count.checked_add(1).ok_or(())?;
    }
    drop(rows);
    drop(statement);

    Ok((lower_hex(&digest.finalize()), row_count))
  }
}

fn duplicate_count_alias(variables: &[String]) -> String {
  let base = "__tabdat_duplicate_count";
  let mut suffix = 0_u64;
  loop {
    let candidate = if suffix == 0 {
      base.to_owned()
    } else {
      format!("{base}_{suffix}")
    };
    if variables
      .iter()
      .all(|variable| !variable.eq_ignore_ascii_case(&candidate))
    {
      return quote_identifier(&candidate);
    }
    suffix = suffix
      .checked_add(1)
      .expect("duplicate alias suffix overflow");
  }
}

fn isid_count_alias(variables: &[String]) -> String {
  let base = "__tabdat_isid_count";
  let mut suffix = 0_u64;
  loop {
    let candidate = if suffix == 0 {
      base.to_owned()
    } else {
      format!("{base}_{suffix}")
    };
    if variables
      .iter()
      .all(|variable| !variable.eq_ignore_ascii_case(&candidate))
    {
      return quote_identifier(&candidate);
    }
    suffix = suffix.checked_add(1).expect("isid alias suffix overflow");
  }
}

fn update_signature_token(digest: &mut Sha256, token: &[u8]) {
  let length = (token.len() as u64).to_be_bytes();
  digest.update(length);
  digest.update(token);
}

fn signature_text(value: &str) -> Vec<u8> {
  let mut encoded = Vec::with_capacity(value.len() + 1);
  encoded.push(b'T');
  encoded.extend_from_slice(value.as_bytes());
  encoded
}

fn signature_value(value: &Value, type_hint: Option<&str>) -> Result<Vec<u8>, ()> {
  match value {
    Value::Null => Ok(b"N".to_vec()),
    Value::Boolean(value) => Ok(if *value { b"B1" } else { b"B0" }.to_vec()),
    Value::TinyInt(value) => Ok(integer_value(*value)),
    Value::SmallInt(value) => Ok(integer_value(*value)),
    Value::Int(value) => Ok(integer_value(*value)),
    Value::BigInt(value) => Ok(integer_value(*value)),
    Value::HugeInt(value) => Ok(integer_value(*value)),
    Value::UHugeInt(value) => Ok(integer_value(*value)),
    Value::UTinyInt(value) => Ok(integer_value(*value)),
    Value::USmallInt(value) => Ok(integer_value(*value)),
    Value::UInt(value) => Ok(integer_value(*value)),
    Value::UBigInt(value) => Ok(integer_value(*value)),
    Value::Float(value) => Ok(float_value(f64::from(*value))),
    Value::Double(value) => Ok(float_value(*value)),
    Value::Decimal(value) => Ok(text_value(b'D', &value.to_string())),
    Value::Timestamp(unit, value) => Ok(text_value(
      b'Z',
      &format_timestamp(*unit, *value, is_timezone_type(type_hint)),
    )),
    Value::Text(value) => Ok(text_value(b'S', value)),
    Value::Blob(value) | Value::Geometry(value) => {
      let mut encoded = Vec::with_capacity(value.len() + 1);
      encoded.push(b'Y');
      encoded.extend_from_slice(value);
      Ok(encoded)
    }
    Value::Date32(value) => Ok(text_value(b'A', &format_date(i64::from(*value)))),
    Value::Time64(unit, value) => Ok(text_value(b'H', &format_time(*unit, *value))),
    Value::List(values) | Value::Array(values) => sequence_value(values, type_hint),
    Value::Enum(value) => Ok(text_value(b'S', value)),
    Value::Struct(values) => struct_value(values.iter(), type_hint),
    Value::Map(values) => map_sequence_value(values, type_hint),
    Value::Interval {
      months,
      days,
      nanos,
    } => sequence_value(
      &[
        Value::HugeInt(i128::from(*months)),
        Value::HugeInt(i128::from(*days)),
        Value::HugeInt(i128::from(*nanos)),
      ],
      None,
    ),
    Value::Union(_) => Err(()),
    _ => Err(()),
  }
}

fn integer_value<T: fmt::Display>(value: T) -> Vec<u8> {
  let mut encoded = String::from("I");
  write!(&mut encoded, "{value}").expect("writing to a String cannot fail");
  encoded.into_bytes()
}

fn text_value(prefix: u8, value: &str) -> Vec<u8> {
  let mut encoded = Vec::with_capacity(value.len() + 1);
  encoded.push(prefix);
  encoded.extend_from_slice(value.as_bytes());
  encoded
}

fn float_value(value: f64) -> Vec<u8> {
  text_value(b'F', &python_float_hex(value))
}

fn python_float_hex(value: f64) -> String {
  if value.is_nan() {
    return "nan".to_owned();
  }
  if value == f64::INFINITY {
    return "inf".to_owned();
  }
  if value == f64::NEG_INFINITY {
    return "-inf".to_owned();
  }

  let bits = value.to_bits();
  let sign = if bits >> 63 == 1 { "-" } else { "" };
  let magnitude = bits & 0x7fff_ffff_ffff_ffff;
  if magnitude == 0 {
    return format!("{sign}0x0.0p+0");
  }

  let exponent_bits = (magnitude >> 52) & 0x7ff;
  let fraction = magnitude & ((1_u64 << 52) - 1);
  if exponent_bits == 0 {
    format!("{sign}0x0.{fraction:013x}p-1022")
  } else {
    let exponent = i32::try_from(exponent_bits).expect("11-bit exponent fits i32") - 1023;
    format!("{sign}0x1.{fraction:013x}p{exponent:+}")
  }
}

fn sequence_value(values: &[Value], type_hint: Option<&str>) -> Result<Vec<u8>, ()> {
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, values.len())?;
  let child_hint = list_value_type(type_hint);
  for value in values {
    let item = signature_value(value, child_hint)?;
    append_signature_part(&mut encoded, &item)?;
  }
  Ok(encoded)
}

fn struct_value<'a, I>(values: I, type_hint: Option<&str>) -> Result<Vec<u8>, ()>
where
  I: IntoIterator<Item = &'a (String, Value)>,
{
  let mut entries = values
    .into_iter()
    .map(|(key, value)| {
      let key_value = Value::Text(key.clone());
      let value_hint = struct_field_type(type_hint, key);
      Ok::<_, ()>((
        signature_value(&key_value, None)?,
        signature_value(value, value_hint.as_deref())?,
      ))
    })
    .collect::<Result<Vec<_>, _>>()?;
  mapping_entries(&mut entries)
}

fn map_sequence_value(
  values: &duckdb::types::OrderedMap<Value, Value>,
  type_hint: Option<&str>,
) -> Result<Vec<u8>, ()> {
  let type_hints = map_value_types(type_hint);
  let key_hint = type_hints.as_ref().map(|(key, _)| key.as_str());
  let value_hint = type_hints.as_ref().map(|(_, value)| value.as_str());
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, values.iter().count())?;
  for (key, value) in values.iter() {
    let pair = sequence_pair_value(key, value, key_hint, value_hint)?;
    append_signature_part(&mut encoded, &pair)?;
  }
  Ok(encoded)
}

fn sequence_pair_value(
  key: &Value,
  value: &Value,
  key_hint: Option<&str>,
  value_hint: Option<&str>,
) -> Result<Vec<u8>, ()> {
  let mut encoded = b"L".to_vec();
  append_ascii_length(&mut encoded, 2)?;
  let key = signature_value(key, key_hint)?;
  let value = signature_value(value, value_hint)?;
  append_signature_part(&mut encoded, &key)?;
  append_signature_part(&mut encoded, &value)?;
  Ok(encoded)
}

fn mapping_entries(entries: &mut Vec<(Vec<u8>, Vec<u8>)>) -> Result<Vec<u8>, ()> {
  entries.sort();

  let mut encoded = b"M".to_vec();
  append_ascii_length(&mut encoded, entries.len())?;
  for (key, value) in entries {
    append_signature_part(&mut encoded, key)?;
    append_signature_part(&mut encoded, value)?;
  }
  Ok(encoded)
}

fn list_value_type(type_hint: Option<&str>) -> Option<&str> {
  let type_hint = type_hint?.trim();
  if let Some(value) = type_hint
    .strip_prefix("LIST(")
    .and_then(|value| value.strip_suffix(')'))
  {
    return Some(value.trim());
  }
  type_hint.strip_suffix("[]").map(str::trim)
}

fn map_value_types(type_hint: Option<&str>) -> Option<(String, String)> {
  let type_hint = type_hint?;
  let body = type_hint
    .strip_prefix("MAP(")
    .and_then(|value| value.strip_suffix(')'))?;
  let (key, value) = split_top_level_once(body)?;
  Some((key.trim().to_owned(), value.trim().to_owned()))
}

fn struct_field_type(type_hint: Option<&str>, key: &str) -> Option<String> {
  let type_hint = type_hint?;
  let body = type_hint
    .strip_prefix("STRUCT(")
    .and_then(|value| value.strip_suffix(')'))?;
  for field in split_top_level_parts(body) {
    let field = field.trim();
    if field.is_empty() {
      continue;
    }
    let (field_name, field_type) = if let Some(rest) = field.strip_prefix('"') {
      let (field_name, field_type) = parse_quoted_struct_field(rest)?;
      (field_name, field_type)
    } else {
      let (field_name, field_type) = field.split_once(char::is_whitespace)?;
      (field_name.to_owned(), field_type.trim())
    };
    if field_name.eq_ignore_ascii_case(key) {
      return (!field_type.is_empty()).then(|| field_type.to_owned());
    }
  }
  None
}

fn is_timezone_type(type_hint: Option<&str>) -> bool {
  type_hint.is_some_and(|hint| canonical_signature_type(hint).ends_with("_TZ"))
}

fn parse_quoted_struct_field(value: &str) -> Option<(String, &str)> {
  let mut name = String::new();
  let mut offset = 0;
  while offset < value.len() {
    let character = value[offset..].chars().next()?;
    let width = character.len_utf8();
    if character == '"' {
      let after_quote = offset + width;
      if value[after_quote..].starts_with('"') {
        name.push('"');
        offset = after_quote + 1;
        continue;
      }
      return Some((name, value[after_quote..].trim()));
    }
    name.push(character);
    offset += width;
  }
  None
}

fn split_top_level_once(value: &str) -> Option<(&str, &str)> {
  let mut depth = 0_u32;
  let mut quoted = false;
  let mut characters = value.char_indices().peekable();
  while let Some((index, character)) = characters.next() {
    match character {
      '"' => {
        if quoted && characters.peek().is_some_and(|(_, next)| *next == '"') {
          characters.next();
        } else {
          quoted = !quoted;
        }
      }
      '(' if !quoted => depth = depth.checked_add(1)?,
      ')' if !quoted => depth = depth.checked_sub(1)?,
      ',' if !quoted && depth == 0 => return Some((&value[..index], &value[index + 1..])),
      _ => {}
    }
  }
  None
}

fn split_top_level_parts(value: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let mut start = 0;
  let mut depth = 0_u32;
  let mut quoted = false;
  let mut characters = value.char_indices().peekable();
  while let Some((index, character)) = characters.next() {
    match character {
      '"' => {
        if quoted && characters.peek().is_some_and(|(_, next)| *next == '"') {
          characters.next();
        } else {
          quoted = !quoted;
        }
      }
      '(' if !quoted => depth = depth.saturating_add(1),
      ')' if !quoted => depth = depth.saturating_sub(1),
      ',' if !quoted && depth == 0 => {
        parts.push(&value[start..index]);
        start = index + character.len_utf8();
      }
      _ => {}
    }
  }
  parts.push(&value[start..]);
  parts
}

fn append_ascii_length(output: &mut Vec<u8>, length: usize) -> Result<(), ()> {
  let length = u64::try_from(length).map_err(|_| ())?;
  output.extend_from_slice(length.to_string().as_bytes());
  Ok(())
}

fn append_signature_part(output: &mut Vec<u8>, value: &[u8]) -> Result<(), ()> {
  let length = u64::try_from(value.len()).map_err(|_| ())?.to_be_bytes();
  output.extend_from_slice(&length);
  output.extend_from_slice(value);
  Ok(())
}

fn format_date(days_since_epoch: i64) -> String {
  let (year, month, day) = civil_from_days(days_since_epoch);
  format!("{year:04}-{month:02}-{day:02}")
}

fn format_timestamp(unit: TimeUnit, value: i64, timezone: bool) -> String {
  let (seconds, nanos) = split_time(unit, value);
  let days = seconds.div_euclid(86_400);
  let seconds_of_day = seconds.rem_euclid(86_400);
  let hour = seconds_of_day / 3_600;
  let minute = (seconds_of_day % 3_600) / 60;
  let second = seconds_of_day % 60;
  let (year, month, day) = civil_from_days(days);
  let micros = nanos / 1_000;
  let mut formatted = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
  if nanos != 0 {
    if unit == TimeUnit::Nanosecond && nanos % 1_000 != 0 {
      write!(&mut formatted, ".{nanos:09}").expect("writing to a String cannot fail");
    } else if micros != 0 {
      write!(&mut formatted, ".{micros:06}").expect("writing to a String cannot fail");
    }
  }
  if timezone {
    formatted.push_str("+00:00");
  }
  formatted
}

fn format_time(unit: TimeUnit, value: i64) -> String {
  let (seconds, nanos) = split_time(unit, value);
  let total_micros =
    (i128::from(seconds) * 1_000_000 + i128::from(nanos) / 1_000).rem_euclid(86_400_000_000);
  let total_micros = i64::try_from(total_micros).expect("time microseconds fit i64");
  let hour = total_micros / 3_600_000_000;
  let minute = (total_micros % 3_600_000_000) / 60_000_000;
  let second = (total_micros % 60_000_000) / 1_000_000;
  let micros = total_micros % 1_000_000;
  if micros == 0 {
    format!("{hour:02}:{minute:02}:{second:02}")
  } else {
    format!("{hour:02}:{minute:02}:{second:02}.{micros:06}")
  }
}

fn split_time(unit: TimeUnit, value: i64) -> (i64, i64) {
  let (divisor, nanos_per_unit) = match unit {
    TimeUnit::Second => (1_i128, 1_000_000_000_i128),
    TimeUnit::Millisecond => (1_000_i128, 1_000_000_i128),
    TimeUnit::Microsecond => (1_000_000_i128, 1_000_i128),
    TimeUnit::Nanosecond => (1_000_000_000_i128, 1_i128),
  };
  let value = i128::from(value);
  let seconds = value.div_euclid(divisor);
  let remainder = value.rem_euclid(divisor);
  (
    i64::try_from(seconds).expect("DuckDB timestamp seconds fit i64"),
    i64::try_from(remainder * nanos_per_unit).expect("timestamp nanoseconds fit i64"),
  )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
  let shifted = days_since_epoch + 719_468;
  let era = if shifted >= 0 {
    shifted / 146_097
  } else {
    (shifted - 146_096) / 146_097
  };
  let day_of_era = shifted - era * 146_097;
  let year_of_era =
    (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
  let year = year_of_era + era * 400;
  let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
  let month_part = (5 * day_of_year + 2) / 153;
  let day = day_of_year - (153 * month_part + 2) / 5 + 1;
  let month = month_part + if month_part < 10 { 3 } else { -9 };
  let year = year + if month <= 2 { 1 } else { 0 };
  (
    year,
    u32::try_from(month).expect("civil month is positive"),
    u32::try_from(day).expect("civil day is positive"),
  )
}

fn canonical_signature_type(data_type: &str) -> String {
  let normalized = data_type
    .chars()
    .filter(|character| !character.is_whitespace())
    .flat_map(|character| character.to_uppercase())
    .collect::<String>();
  let alias = match normalized.as_str() {
    "TINYINT" => Some("INT8"),
    "SMALLINT" => Some("INT16"),
    "INTEGER" => Some("INT32"),
    "BIGINT" => Some("INT64"),
    "HUGEINT" => Some("INT128"),
    "UTINYINT" => Some("UINT8"),
    "USMALLINT" => Some("UINT16"),
    "UINTEGER" => Some("UINT32"),
    "UBIGINT" => Some("UINT64"),
    "UHUGEINT" => Some("UINT128"),
    "FLOAT" | "REAL" => Some("FLOAT32"),
    "DOUBLE" => Some("FLOAT64"),
    "BOOLEAN" | "BOOL" => Some("BOOL"),
    "VARCHAR" | "TEXT" | "CATEGORICAL" | "ENUM" => Some("STRING"),
    "DATETIME" | "TIMESTAMP" => Some("TIMESTAMP_US"),
    "TIMESTAMPTZ" | "TIMESTAMPWITHTIMEZONE" => Some("TIMESTAMP_US_TZ"),
    _ => None,
  };
  if let Some(alias) = alias {
    return alias.to_owned();
  }
  if normalized.starts_with("DECIMAL(PRECISION=")
    && let Some((precision, scale)) = parse_decimal_precision_scale(&normalized)
  {
    return format!("DECIMAL({precision},{scale})");
  }
  if normalized.starts_with("DECIMAL(") && normalized.ends_with(')') {
    return normalized;
  }
  if normalized.starts_with("DATETIME(TIME_UNIT=") {
    let unit_start = "DATETIME(TIME_UNIT=".len();
    if let Some(unit_end) = normalized[unit_start..].find([',', ')']) {
      let unit = &normalized[unit_start..unit_start + unit_end];
      let timezone = if normalized
        .split_once("TIME_ZONE=")
        .is_some_and(|(_, value)| !value.starts_with("NONE"))
      {
        "_TZ"
      } else {
        ""
      };
      return format!("TIMESTAMP_{unit}{timezone}");
    }
  }
  if normalized.starts_with("LIST(") && normalized.ends_with(')') {
    return format!(
      "LIST({})",
      canonical_signature_type(&normalized[5..normalized.len() - 1])
    );
  }
  if normalized.ends_with("[]") {
    return format!(
      "LIST({})",
      canonical_signature_type(&normalized[..normalized.len() - 2])
    );
  }
  normalized
}

fn parse_decimal_precision_scale(data_type: &str) -> Option<(&str, &str)> {
  let body = data_type
    .strip_prefix("DECIMAL(PRECISION=")?
    .strip_suffix(')')?;
  let (precision, scale) = body.split_once(",SCALE=")?;
  if precision
    .chars()
    .all(|character| character.is_ascii_digit())
    && scale.chars().all(|character| character.is_ascii_digit())
  {
    Some((precision, scale))
  } else {
    None
  }
}

fn lower_hex(bytes: &[u8]) -> String {
  const HEX: &[u8; 16] = b"0123456789abcdef";
  let mut encoded = String::with_capacity(bytes.len() * 2);
  for byte in bytes {
    encoded.push(char::from(HEX[usize::from(byte >> 4)]));
    encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
  }
  encoded
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
  fn datasignature_float_hex_matches_python_for_edge_values() {
    assert_eq!(python_float_hex(0.0), "0x0.0p+0");
    assert_eq!(python_float_hex(-0.0), "-0x0.0p+0");
    assert_eq!(python_float_hex(1.0), "0x1.0000000000000p+0");
    assert_eq!(
      python_float_hex(f64::from_bits(1)),
      "0x0.0000000000001p-1022"
    );
    assert_eq!(python_float_hex(f64::NAN), "nan");
    assert_eq!(python_float_hex(f64::INFINITY), "inf");
    assert_eq!(python_float_hex(f64::NEG_INFINITY), "-inf");
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
  fn datasignature_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session.execute(Command::Datasignature).unwrap_err(),
      RuntimeError::NoActiveDataset {
        command: "datasignature"
      }
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
  fn duplicates_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Duplicates {
          variables: Vec::new(),
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset {
        command: "duplicates"
      }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn isid_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Isid {
          variables: vec!["id".to_owned()],
          missok: false,
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "isid" }
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
  fn failed_duplicates_keeps_the_published_dataset_metadata() {
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
        .execute(Command::Duplicates {
          variables: vec!["value".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::DuplicatesFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  }

  #[test]
  fn failed_isid_keeps_the_published_dataset_metadata() {
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
        .execute(Command::Isid {
          variables: vec!["value".to_owned()],
          missok: false,
        })
        .unwrap_err(),
      RuntimeError::IsidFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
  }

  #[test]
  fn failed_datasignature_keeps_the_published_dataset_metadata() {
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
      session.execute(Command::Datasignature).unwrap_err(),
      RuntimeError::DatasignatureFailed
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
