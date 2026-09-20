#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use duckdb::Connection;
use duckdb::types::{TimeUnit, Value, ValueRef};
use sha2::{Digest, Sha256};
use tabdat_language::{
  AssertBinaryOperator, AssertExpression, ByCommand, CollapseCommand, CollapseStatistic, Command,
  DataSource, ExecutionMode, GenerateBinaryOperator, GenerateExpression, LabelCommand, LabelValue,
  LazyEngine, RecodeInput, RecodeRangeEndpoint, RecodeRule, RecodeTarget, RecodeValue, RowLimit,
  SortKey, TabulateCommand,
};

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

/// The owned result returned after projecting columns with `keep`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeepResult {
  /// Metadata for the newly active projected dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after projecting out columns with `drop`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropResult {
  /// Metadata for the newly active projected dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after projecting an explicit selection with
/// `select`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectResult {
  /// Metadata for the newly active projected dataset.
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

/// The owned aggregate returned by a read-only row assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertResult {
  /// The number of active rows checked by the predicate.
  pub checked: u64,
  /// The number of rows whose predicate was false or SQL NULL.
  pub failed: u64,
}

/// The owned result returned after appending a generated numeric column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateResult {
  /// Metadata for the newly active generated dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after replacing values in an existing column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceResult {
  /// Metadata for the newly active replaced dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after renaming one column in the active dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameResult {
  /// Metadata for the newly active renamed dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after stably sorting active rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortResult {
  /// Metadata for the newly active sorted dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after stably sorting active rows by directed keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GsortResult {
  /// Metadata for the newly active directed-sort dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after recoding selected columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecodeResult {
  /// Metadata for the newly active recoded dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after encoding one string column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeResult {
  /// Metadata for the newly active encoded dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after decoding one encode-produced numeric column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeResult {
  /// Metadata for the newly active decoded dataset.
  pub dataset: DatasetInfo,
}

/// A normalized named value-label set owned by the active session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueLabelSet {
  /// The set name.
  pub name: String,
  /// Values and their display labels in deterministic order.
  pub mappings: Vec<(LabelValue, String)>,
}

/// Session-local variable and value-label metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelMetadata {
  /// Variable names and their display labels.
  pub variable_labels: Vec<(String, String)>,
  /// Named value-label sets.
  pub value_sets: Vec<ValueLabelSet>,
  /// Variable names and their attached value-label set names.
  pub attachments: Vec<(String, String)>,
}

impl LabelMetadata {
  fn is_empty(&self) -> bool {
    self.variable_labels.is_empty() && self.value_sets.is_empty() && self.attachments.is_empty()
  }
}

/// The metadata mutation represented by a successful `label` result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelResultAction {
  /// A variable display label was set or cleared.
  Variable,
  /// A named value-label set was defined or replaced.
  Define,
  /// A value-label set was attached or cleared.
  Values,
  /// Label metadata was listed.
  List,
  /// Named value-label sets were dropped.
  Drop,
}

/// The owned result returned by a successful session-local `label` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelResult {
  /// The label action that completed.
  pub action: LabelResultAction,
  /// A deterministic human-readable completion message.
  pub message: String,
  /// The resulting metadata, or `None` when the session has no labels.
  pub metadata: Option<LabelMetadata>,
}

/// The owned result returned by a bounded one- or two-way frequency table.
#[derive(Debug, Clone, PartialEq)]
pub struct TabulateResult {
  /// Human-readable table headers in output order.
  pub headers: Vec<String>,
  /// Owned table rows in deterministic category order.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned by a bounded grouped read-only request.
#[derive(Debug, Clone, PartialEq)]
pub struct ByResult {
  /// Group and aggregate headers in output order.
  pub headers: Vec<String>,
  /// Owned grouped rows in deterministic group order.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned after replacing the active relation with a
/// grouped aggregate relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapseResult {
  /// Metadata for the newly active collapsed dataset.
  pub dataset: DatasetInfo,
}

struct TabulateCount {
  row: CellValue,
  column: Option<CellValue>,
  count: u64,
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
  /// Checks a typed boolean expression across every active row.
  Assert(AssertResult),
  /// Appends a generated numeric column to the active dataset.
  Generate(GenerateResult),
  /// Replaces values in an existing column in the active dataset.
  Replace(ReplaceResult),
  /// Renames one column in the active dataset.
  Rename(RenameResult),
  /// Stably sorts active rows by ascending columns.
  Sort(SortResult),
  /// Stably sorts active rows by explicitly directed columns.
  Gsort(GsortResult),
  /// Recodes values or ranges in selected active columns.
  Recode(RecodeResult),
  /// Encodes one string column into a new integer-coded column.
  Encode(EncodeResult),
  /// Decodes one encode-produced numeric column into a new string column.
  Decode(DecodeResult),
  /// Manages session-local variable and value-label metadata.
  Label(LabelResult),
  /// Reports a bounded one- or two-way frequency table.
  Tabulate(TabulateResult),
  /// Reports a bounded grouped read-only table.
  By(ByResult),
  /// Replaces the active relation with a bounded grouped aggregate relation.
  Collapse(CollapseResult),
  /// Projects an explicit ordered set of columns in the active dataset.
  Keep(KeepResult),
  /// Removes an explicit set of columns from the active dataset.
  Drop(DropResult),
  /// Projects an explicit ordered set of columns in the active dataset.
  Select(SelectResult),
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
  /// The assertion expression named variables absent from the active schema.
  AssertUnknownVariable { variables: Vec<String> },
  /// The assertion expression did not evaluate to a boolean or null domain.
  AssertRequiresBoolean,
  /// The assertion predicate found false or NULL rows.
  AssertSemanticFailure { checked: u64, failed: u64 },
  /// The assertion expression has incompatible operand domains.
  AssertTypeMismatch { message: String },
  /// DuckDB could not produce the assertion aggregate.
  AssertFailed,
  /// The generate target already exists in the active schema.
  GenerateTargetExists { variable: String },
  /// The generate expression named variables absent from the active schema.
  GenerateUnknownVariable { variables: Vec<String> },
  /// The generate expression has incompatible operand domains.
  GenerateTypeMismatch { message: String },
  /// The generate expression uses a form outside this bounded runtime slice.
  GenerateUnsupportedExpression { message: String },
  /// DuckDB could not stage or publish the generated relation.
  GenerateFailed,
  /// The replace target is absent from the active schema.
  ReplaceTargetUnknownVariable { variable: String },
  /// The replace expression or predicate named variables absent from the active schema.
  ReplaceUnknownVariable { variables: Vec<String> },
  /// The replace expression or target has incompatible domains.
  ReplaceTypeMismatch { message: String },
  /// The replace predicate did not evaluate to a boolean or null domain.
  ReplaceRequiresBoolean,
  /// The replace expression uses a form outside this bounded runtime slice.
  ReplaceUnsupportedExpression { message: String },
  /// DuckDB could not stage or publish the replaced relation.
  ReplaceFailed,
  /// The rename request named a source column absent from the active schema.
  RenameUnknownVariable { variable: String },
  /// The rename request named a target already present in the active schema.
  RenameTargetExists { variable: String },
  /// DuckDB could not stage or publish the renamed relation.
  RenameFailed,
  /// The sort request did not name any variables.
  SortNoVariables,
  /// The sort request named variables absent from the active schema.
  SortUnknownVariable { variables: Vec<String> },
  /// DuckDB could not stage or publish the sorted relation.
  SortFailed,
  /// The gsort request did not name any variables.
  GsortNoVariables,
  /// The gsort request named variables absent from the active schema.
  GsortUnknownVariable { variables: Vec<String> },
  /// DuckDB could not stage or publish the directed sorted relation.
  GsortFailed,
  /// The recode request did not name any source variables.
  RecodeNoVariables,
  /// The recode request did not contain any rules.
  RecodeNoRules,
  /// The recode request named variables absent from the active schema.
  RecodeUnknownVariable { variables: Vec<String> },
  /// Generate mode did not provide one output per source variable.
  RecodeGenerateCountMismatch,
  /// Generate mode repeated an output variable.
  RecodeGenerateDuplicateVariable,
  /// Generate mode named an output already present in the active schema.
  RecodeGenerateTargetExists { variable: String },
  /// A numeric range was applied to a non-numeric source column.
  RecodeRangeRequiresNumeric { variable: String },
  /// DuckDB could not stage or publish the recoded relation.
  RecodeFailed,
  /// The encode request named a source column absent from the active schema.
  EncodeUnknownVariable { variable: String },
  /// The encode source column is not a string variable.
  EncodeRequiresString { variable: String },
  /// The encode target already exists in the active schema.
  EncodeTargetExists { variable: String },
  /// DuckDB could not stage or publish the encoded relation.
  EncodeFailed,
  /// The decode source has no encode-produced value-label map in this session.
  DecodeRequiresAttachedLabels { variable: String },
  /// The decode request named a source column absent from the active schema.
  DecodeUnknownVariable { variable: String },
  /// The decode source column is not numeric.
  DecodeRequiresNumeric { variable: String },
  /// The decode target already exists in the active schema.
  DecodeTargetExists { variable: String },
  /// DuckDB could not stage or publish the decoded relation.
  DecodeFailed,
  /// The label request named a variable absent from the active schema.
  LabelUnknownVariable { variable: String },
  /// A label definition repeated an existing set without `, replace`.
  LabelDefineSetExists { set_name: String },
  /// A label definition repeated one value within a set.
  LabelDefineDuplicateValue { set_name: String, value: LabelValue },
  /// A label definition had no value mappings.
  LabelDefineNoMappings,
  /// A values attachment named a set that does not exist.
  LabelValuesUnknownSet { set_name: String },
  /// A label list request named unknown sets.
  LabelListUnknownSet { names: Vec<String> },
  /// A label drop request named unknown sets.
  LabelDropUnknownSet { names: Vec<String> },
  /// A decode attachment contains values outside the integer code domain.
  DecodeRequiresIntegerLabels { variable: String },
  /// The tabulate request did not name a row variable.
  TabulateNoVariables,
  /// The tabulate request named more than one row or column dimension.
  TabulateUnsupportedDimensions,
  /// The tabulate request repeated a dimension variable.
  TabulateDuplicateVariable { variable: String },
  /// The tabulate request named variables absent from the active schema.
  TabulateUnknownVariable { variables: Vec<String> },
  /// A percentage option was requested for a one-way table.
  TabulatePercentageRequiresTwoWay { option: &'static str },
  /// DuckDB could not produce or own the frequency table.
  TabulateFailed,
  /// The by request did not name any grouping variables.
  ByNoGroups,
  /// The by request named variables absent from the active schema.
  ByUnknownVariable { variables: Vec<String> },
  /// A grouped summarize request named non-numeric variables.
  BySummarizeRequiresNumeric { variables: Vec<String> },
  /// The active schema contains no numeric non-group columns for a default
  /// grouped summarize.
  BySummarizeNoNumericColumns,
  /// The child command is outside the bounded grouped runtime slice.
  ByUnsupportedCommand,
  /// DuckDB could not produce or own a grouped read-only table.
  ByFailed,
  /// The collapse request did not name any aggregate variables.
  CollapseNoVariables,
  /// The collapse request did not name any grouping variables.
  CollapseNoGroups,
  /// The collapse request named variables absent from the active schema.
  CollapseUnknownVariable { variables: Vec<String> },
  /// A non-count collapse statistic was requested for non-numeric variables.
  CollapseRequiresNumeric { variables: Vec<String> },
  /// DuckDB could not stage or publish the collapsed relation.
  CollapseFailed,
  /// The keep request named variables absent from the active schema.
  KeepUnknownVariable { variables: Vec<String> },
  /// The keep request did not name any variables.
  KeepNoVariables,
  /// DuckDB could not stage or publish the projected relation.
  KeepFailed,
  /// The drop request named variables absent from the active schema.
  DropUnknownVariable { variables: Vec<String> },
  /// The drop request did not name any variables.
  DropNoVariables,
  /// The drop request would leave the active relation without columns.
  DropWouldRemoveEveryColumn,
  /// DuckDB could not stage or publish the projected relation.
  DropFailed,
  /// The select request named variables absent from the active schema.
  SelectUnknownVariable { variables: Vec<String> },
  /// The select request did not name any variables.
  SelectNoVariables,
  /// DuckDB could not stage or publish the projected relation.
  SelectFailed,
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
      Self::AssertUnknownVariable { variables } => {
        write!(
          formatter,
          "expression unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::AssertRequiresBoolean => formatter.write_str("predicate requires boolean expression"),
      Self::AssertSemanticFailure { checked, failed } => write!(
        formatter,
        "assertion failed: {failed} of {checked} rows failed"
      ),
      Self::AssertTypeMismatch { message } => formatter.write_str(message),
      Self::AssertFailed => formatter.write_str("assert failed"),
      Self::GenerateTargetExists { variable } => {
        write!(formatter, "generate target already exists: {variable}")
      }
      Self::GenerateUnknownVariable { variables } => {
        write!(
          formatter,
          "expression unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::GenerateTypeMismatch { message } => formatter.write_str(message),
      Self::GenerateUnsupportedExpression { message } => formatter.write_str(message),
      Self::GenerateFailed => formatter.write_str("generate failed"),
      Self::ReplaceTargetUnknownVariable { variable } => {
        write!(formatter, "replace unknown variable: {variable}")
      }
      Self::ReplaceUnknownVariable { variables } => {
        write!(
          formatter,
          "expression unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::ReplaceTypeMismatch { message } => formatter.write_str(message),
      Self::ReplaceRequiresBoolean => formatter.write_str("predicate requires boolean expression"),
      Self::ReplaceUnsupportedExpression { message } => formatter.write_str(message),
      Self::ReplaceFailed => formatter.write_str("replace failed"),
      Self::RenameUnknownVariable { variable } => {
        write!(formatter, "rename unknown variable: {variable}")
      }
      Self::RenameTargetExists { variable } => {
        write!(formatter, "rename target already exists: {variable}")
      }
      Self::RenameFailed => formatter.write_str("rename failed"),
      Self::SortNoVariables => formatter.write_str("sort expects a variable list"),
      Self::SortUnknownVariable { variables } => {
        write!(formatter, "sort unknown variable: {}", variables.join(", "))
      }
      Self::SortFailed => formatter.write_str("sort failed"),
      Self::GsortNoVariables => formatter.write_str("gsort expects at least one variable"),
      Self::GsortUnknownVariable { variables } => {
        write!(
          formatter,
          "gsort unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::GsortFailed => formatter.write_str("gsort failed"),
      Self::RecodeNoVariables => formatter.write_str("recode expects at least one variable"),
      Self::RecodeNoRules => formatter.write_str("recode expects at least one rule"),
      Self::RecodeUnknownVariable { variables } => {
        write!(
          formatter,
          "recode unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::RecodeGenerateCountMismatch => {
        formatter.write_str("number of generate variables must match number of variables to recode")
      }
      Self::RecodeGenerateDuplicateVariable => {
        formatter.write_str("recode generate variables must be unique")
      }
      Self::RecodeGenerateTargetExists { variable } => {
        write!(formatter, "generate variable already exists: {variable}")
      }
      Self::RecodeRangeRequiresNumeric { variable } => write!(
        formatter,
        "range recode rule not allowed on non-numeric column: {variable}"
      ),
      Self::RecodeFailed => formatter.write_str("recode failed"),
      Self::EncodeUnknownVariable { variable } => {
        write!(formatter, "encode unknown variable: {variable}")
      }
      Self::EncodeRequiresString { variable } => {
        write!(formatter, "encode requires a string variable: {variable}")
      }
      Self::EncodeTargetExists { variable } => {
        write!(formatter, "encode target already exists: {variable}")
      }
      Self::EncodeFailed => formatter.write_str("encode failed"),
      Self::DecodeRequiresAttachedLabels { variable } => write!(
        formatter,
        "decode requires attached value labels on {variable}"
      ),
      Self::DecodeUnknownVariable { variable } => {
        write!(formatter, "decode unknown variable: {variable}")
      }
      Self::DecodeRequiresNumeric { variable } => {
        write!(formatter, "decode requires a numeric variable: {variable}")
      }
      Self::DecodeTargetExists { variable } => {
        write!(formatter, "decode target already exists: {variable}")
      }
      Self::DecodeFailed => formatter.write_str("decode failed"),
      Self::DecodeRequiresIntegerLabels { variable } => write!(
        formatter,
        "decode requires integer value labels on {variable}"
      ),
      Self::TabulateNoVariables => formatter.write_str("tabulate expects a row variable"),
      Self::TabulateUnsupportedDimensions => {
        formatter.write_str("tabulate supports at most one row and one column variable")
      }
      Self::TabulateDuplicateVariable { variable } => {
        write!(formatter, "tabulate duplicate variable: {variable}")
      }
      Self::TabulateUnknownVariable { variables } => {
        write!(
          formatter,
          "tabulate unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::TabulatePercentageRequiresTwoWay { option } => {
        write!(
          formatter,
          "tabulate option {option} requires a two-way table"
        )
      }
      Self::TabulateFailed => formatter.write_str("tabulate failed"),
      Self::ByNoGroups => formatter.write_str("by expects at least one grouping variable"),
      Self::ByUnknownVariable { variables } => {
        write!(formatter, "by unknown variable: {}", variables.join(", "))
      }
      Self::BySummarizeRequiresNumeric { variables } => write!(
        formatter,
        "by summarize requires numeric variables: {}",
        variables.join(", ")
      ),
      Self::BySummarizeNoNumericColumns => {
        formatter.write_str("by summarize found no numeric columns")
      }
      Self::ByUnsupportedCommand => {
        formatter.write_str("by only supports summarize and count in the bounded runtime")
      }
      Self::ByFailed => formatter.write_str("by failed"),
      Self::CollapseNoVariables => {
        formatter.write_str("collapse expects at least one aggregate variable")
      }
      Self::CollapseNoGroups => {
        formatter.write_str("collapse expects at least one grouping variable")
      }
      Self::CollapseUnknownVariable { variables } => {
        write!(
          formatter,
          "collapse unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::CollapseRequiresNumeric { variables } => {
        write!(
          formatter,
          "collapse requires numeric variables: {}",
          variables.join(", ")
        )
      }
      Self::CollapseFailed => formatter.write_str("collapse failed"),
      Self::LabelUnknownVariable { variable } => {
        write!(formatter, "label unknown variable: {variable}")
      }
      Self::LabelDefineSetExists { set_name } => write!(
        formatter,
        "label define set already exists: {set_name} (use , replace)"
      ),
      Self::LabelDefineDuplicateValue { set_name, value } => write!(
        formatter,
        "label define duplicate value in set {set_name}: {value:?}"
      ),
      Self::LabelDefineNoMappings => {
        formatter.write_str("label define expects at least one mapping")
      }
      Self::LabelValuesUnknownSet { set_name } => {
        write!(formatter, "label values unknown label set: {set_name}")
      }
      Self::LabelListUnknownSet { names } => {
        write!(
          formatter,
          "label list unknown label set: {}",
          names.join(", ")
        )
      }
      Self::LabelDropUnknownSet { names } => {
        write!(
          formatter,
          "label drop unknown label set: {}",
          names.join(", ")
        )
      }
      Self::KeepUnknownVariable { variables } => {
        write!(formatter, "keep unknown variable: {}", variables.join(", "))
      }
      Self::KeepNoVariables => formatter.write_str("keep expects a variable list or if clause"),
      Self::KeepFailed => formatter.write_str("keep failed"),
      Self::DropUnknownVariable { variables } => {
        write!(formatter, "drop unknown variable: {}", variables.join(", "))
      }
      Self::DropNoVariables => formatter.write_str("drop expects a variable list or if clause"),
      Self::DropWouldRemoveEveryColumn => formatter.write_str("drop would remove every column"),
      Self::DropFailed => formatter.write_str("drop failed"),
      Self::SelectUnknownVariable { variables } => {
        write!(
          formatter,
          "select unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::SelectNoVariables => formatter.write_str("select expects a variable list"),
      Self::SelectFailed => formatter.write_str("select failed"),
    }
  }
}

impl Error for RuntimeError {}

/// A session holding optional active metadata and a private DuckDB adapter.
pub struct Session {
  backend: Option<DuckDbBackend>,
  active_dataset: Option<DatasetInfo>,
  label_metadata: LabelMetadata,
}

impl Session {
  /// Construct a session without initializing DuckDB.
  pub fn new() -> Self {
    Self {
      backend: None,
      active_dataset: None,
      label_metadata: LabelMetadata::default(),
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
      Command::Assert { expression } => self.execute_assert(expression),
      Command::Generate {
        variable,
        expression,
      } => self.execute_generate(variable, expression),
      Command::Replace {
        variable,
        expression,
        condition,
      } => self.execute_replace(variable, expression, condition),
      Command::Rename { old_name, new_name } => self.execute_rename(old_name, new_name),
      Command::Sort { variables } => self.execute_sort(variables),
      Command::Gsort { keys } => self.execute_gsort(keys),
      Command::Recode {
        variables,
        rules,
        target,
      } => self.execute_recode(variables, rules, target),
      Command::Encode {
        source,
        generate,
        label,
      } => self.execute_encode(source, generate, label),
      Command::Decode { source, generate } => self.execute_decode(source, generate),
      Command::Label { command } => self.execute_label(command),
      Command::Tabulate { command } => self.execute_tabulate(command),
      Command::By { command } => self.execute_by(command),
      Command::Collapse { command } => self.execute_collapse(command),
      Command::Keep { variables } => self.execute_keep(variables),
      Command::Drop { variables } => self.execute_drop(variables),
      Command::Select { variables } => self.execute_select(variables),
      Command::Head { limit } => self.execute_head(limit),
      Command::Tail { limit } => self.execute_tail(limit),
      _ => Err(RuntimeError::UnsupportedCommand { name: command_name }),
    }
  }

  /// Return the currently published dataset metadata, if any.
  pub fn active_dataset(&self) -> Option<&DatasetInfo> {
    self.active_dataset.as_ref()
  }

  /// Return the currently published session-local label metadata, if any.
  pub fn active_label_metadata(&self) -> Option<&LabelMetadata> {
    (!self.label_metadata.is_empty()).then_some(&self.label_metadata)
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

  fn execute_assert(&self, expression: AssertExpression) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "assert" })?;
    if let Some(variable) = expression_identifiers(&expression)
      .into_iter()
      .find(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == *variable)
      })
    {
      return Err(RuntimeError::AssertUnknownVariable {
        variables: vec![variable],
      });
    }
    let domain = expression_domain(&expression, dataset)?;
    if !matches!(domain, ExpressionDomain::Boolean | ExpressionDomain::Null) {
      return Err(RuntimeError::AssertRequiresBoolean);
    }
    let backend = self.backend.as_ref().ok_or(RuntimeError::AssertFailed)?;
    let (checked, failed) = backend
      .assert_rows(&expression, dataset)
      .map_err(|_| RuntimeError::AssertFailed)?;
    if failed > 0 {
      return Err(RuntimeError::AssertSemanticFailure { checked, failed });
    }
    Ok(ExecutionResult::Assert(AssertResult { checked, failed }))
  }

  fn execute_generate(
    &mut self,
    variable: String,
    expression: GenerateExpression,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "generate",
      })?
      .clone();
    if dataset.columns.iter().any(|column| column.name == variable) {
      return Err(RuntimeError::GenerateTargetExists { variable });
    }

    let unknown = generate_expression_identifiers(&expression)
      .into_iter()
      .filter(|name| !dataset.columns.iter().any(|column| column.name == *name))
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::GenerateUnknownVariable { variables: unknown });
    }

    let expression = generate_expression_to_assert(&expression)?;
    match expression_domain(&expression, &dataset) {
      Ok(ExpressionDomain::Numeric) => {}
      Ok(_) => {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
      Err(RuntimeError::AssertTypeMismatch { message }) => {
        return Err(RuntimeError::GenerateTypeMismatch { message });
      }
      Err(RuntimeError::AssertUnknownVariable { variables }) => {
        return Err(RuntimeError::GenerateUnknownVariable { variables });
      }
      Err(_) => return Err(RuntimeError::GenerateFailed),
    }
    validate_generate_expression(&expression, &dataset)?;
    let backend = self.backend.as_mut().ok_or(RuntimeError::GenerateFailed)?;
    let next_dataset = backend
      .generate_column(&dataset, &variable, &expression)
      .map_err(|_| RuntimeError::GenerateFailed)?;
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Generate(GenerateResult {
      dataset: next_dataset,
    }))
  }

  fn execute_replace(
    &mut self,
    variable: String,
    expression: GenerateExpression,
    condition: Option<GenerateExpression>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "replace" })?
      .clone();
    if !dataset.columns.iter().any(|column| column.name == variable) {
      return Err(RuntimeError::ReplaceTargetUnknownVariable { variable });
    }

    let replacement = replace_expression_to_assert(&expression)?;
    let condition = condition
      .as_ref()
      .map(replace_expression_to_assert)
      .transpose()?;
    let identifiers = replace_expression_identifiers(&expression)
      .into_iter()
      .chain(
        condition
          .as_ref()
          .map(expression_identifiers)
          .into_iter()
          .flatten(),
      )
      .filter(|name| !dataset.columns.iter().any(|column| column.name == *name))
      .collect::<Vec<_>>();
    if !identifiers.is_empty() {
      return Err(RuntimeError::ReplaceUnknownVariable {
        variables: identifiers,
      });
    }

    let target_type = dataset
      .columns
      .iter()
      .find(|column| column.name == variable)
      .map(|column| column.data_type.as_str())
      .ok_or_else(|| RuntimeError::ReplaceTargetUnknownVariable {
        variable: variable.clone(),
      })?;
    let target_domain = data_type_expression_domain(target_type);
    if !matches!(
      target_domain,
      ExpressionDomain::Numeric | ExpressionDomain::String
    ) {
      return Err(RuntimeError::ReplaceTypeMismatch {
        message: format!(
          "replace target {variable} has unsupported domain: {}",
          expression_domain_name(target_domain)
        ),
      });
    }

    let replacement_domain =
      expression_domain(&replacement, &dataset).map_err(map_replace_error)?;
    if replacement_domain != ExpressionDomain::Null && replacement_domain != target_domain {
      return Err(RuntimeError::ReplaceTypeMismatch {
        message: format!(
          "replace target {variable} is {} but expression is {}",
          expression_domain_name(target_domain),
          expression_domain_name(replacement_domain)
        ),
      });
    }
    if let Some(condition) = condition.as_ref() {
      let condition_domain = expression_domain(condition, &dataset).map_err(map_replace_error)?;
      if !matches!(
        condition_domain,
        ExpressionDomain::Boolean | ExpressionDomain::Null
      ) {
        return Err(RuntimeError::ReplaceRequiresBoolean);
      }
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::ReplaceFailed)?;
    let next_dataset = backend
      .replace_column(
        &dataset,
        &variable,
        target_type,
        &replacement,
        condition.as_ref(),
      )
      .map_err(|_| RuntimeError::ReplaceFailed)?;
    self.remove_value_label_attachment(&variable);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Replace(ReplaceResult {
      dataset: next_dataset,
    }))
  }

  fn execute_rename(
    &mut self,
    old_name: String,
    new_name: String,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "rename" })?
      .clone();
    if !dataset.columns.iter().any(|column| column.name == old_name) {
      return Err(RuntimeError::RenameUnknownVariable { variable: old_name });
    }
    if dataset.columns.iter().any(|column| column.name == new_name) {
      return Err(RuntimeError::RenameTargetExists { variable: new_name });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::RenameFailed)?;
    let next_dataset = backend
      .rename_column(&dataset, &old_name, &new_name)
      .map_err(|_| RuntimeError::RenameFailed)?;
    self.rename_label_metadata(&old_name, &new_name);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Rename(RenameResult {
      dataset: next_dataset,
    }))
  }

  fn execute_sort(&mut self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "sort" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::SortNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SortUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::SortFailed)?;
    let next_dataset = backend
      .sort_rows(&dataset, &variables)
      .map_err(|_| RuntimeError::SortFailed)?;
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Sort(SortResult {
      dataset: next_dataset,
    }))
  }

  fn execute_gsort(&mut self, keys: Vec<SortKey>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "gsort" })?
      .clone();
    if keys.is_empty() {
      return Err(RuntimeError::GsortNoVariables);
    }
    let variables = keys
      .iter()
      .map(|key| key.variable.clone())
      .collect::<Vec<_>>();
    let directions = keys.iter().map(|key| key.descending).collect::<Vec<_>>();
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::GsortUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::GsortFailed)?;
    let next_dataset = backend
      .sort_rows_directed(&dataset, &variables, &directions)
      .map_err(|_| RuntimeError::GsortFailed)?;
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Gsort(GsortResult {
      dataset: next_dataset,
    }))
  }

  fn execute_recode(
    &mut self,
    variables: Vec<String>,
    rules: Vec<RecodeRule>,
    target: RecodeTarget,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "recode" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::RecodeNoVariables);
    }
    if rules.is_empty() {
      return Err(RuntimeError::RecodeNoRules);
    }

    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::RecodeUnknownVariable { variables: unknown });
    }

    if let Some(range_variable) = variables.iter().find(|variable| {
      let data_type = dataset
        .columns
        .iter()
        .find(|column| column.name == **variable)
        .map(|column| column.data_type.as_str())
        .expect("source variable validation precedes range validation");
      !is_numeric_data_type(data_type)
        && rules.iter().any(|rule| {
          rule
            .inputs
            .iter()
            .any(|input| matches!(input, RecodeInput::Range { .. }))
        })
    }) {
      return Err(RuntimeError::RecodeRangeRequiresNumeric {
        variable: (*range_variable).clone(),
      });
    }

    if let RecodeTarget::Generate {
      variables: generated,
    } = &target
    {
      if generated.len() != variables.len() {
        return Err(RuntimeError::RecodeGenerateCountMismatch);
      }
      if generated.is_empty() {
        return Err(RuntimeError::RecodeGenerateCountMismatch);
      }
      if generated
        .iter()
        .enumerate()
        .any(|(index, variable)| generated[..index].contains(variable))
      {
        return Err(RuntimeError::RecodeGenerateDuplicateVariable);
      }
      if let Some(variable) = generated.iter().find(|variable| {
        dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      }) {
        return Err(RuntimeError::RecodeGenerateTargetExists {
          variable: variable.clone(),
        });
      }
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::RecodeFailed)?;
    let next_dataset = backend
      .recode_columns(&dataset, &variables, &rules, &target)
      .map_err(|_| RuntimeError::RecodeFailed)?;
    if matches!(target, RecodeTarget::Replace) {
      for variable in &variables {
        self.remove_value_label_attachment(variable);
      }
    }
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Recode(RecodeResult {
      dataset: next_dataset,
    }))
  }

  fn execute_encode(
    &mut self,
    source: String,
    generate: String,
    label: Option<String>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "encode" })?
      .clone();
    let source_column = dataset
      .columns
      .iter()
      .find(|column| column.name == source)
      .ok_or_else(|| RuntimeError::EncodeUnknownVariable {
        variable: source.clone(),
      })?;
    if dataset.columns.iter().any(|column| column.name == generate) {
      return Err(RuntimeError::EncodeTargetExists { variable: generate });
    }
    if data_type_expression_domain(&source_column.data_type) != ExpressionDomain::String {
      return Err(RuntimeError::EncodeRequiresString { variable: source });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::EncodeFailed)?;
    let values = backend
      .distinct_nonmissing_string_values(&source)
      .map_err(|_| RuntimeError::EncodeFailed)?;
    let mapping = values
      .into_iter()
      .enumerate()
      .map(|(index, value)| {
        let code = index
          .checked_add(1)
          .and_then(|value| i64::try_from(value).ok())
          .ok_or(RuntimeError::EncodeFailed)?;
        Ok((value, code))
      })
      .collect::<Result<Vec<_>, RuntimeError>>()?;
    let next_dataset = backend
      .encode_column(&dataset, &source, &generate, &mapping)
      .map_err(|_| RuntimeError::EncodeFailed)?;
    let set_name = label.unwrap_or_else(|| generate.clone());
    let value_mappings = mapping
      .into_iter()
      .map(|(value, code)| (code, value))
      .collect::<Vec<_>>();
    let mut metadata = self.label_metadata.clone();
    upsert_value_label_set(
      &mut metadata,
      ValueLabelSet {
        name: set_name.clone(),
        mappings: value_mappings
          .into_iter()
          .map(|(code, value)| (LabelValue::Integer(code), value))
          .collect(),
      },
    );
    upsert_attachment(&mut metadata, generate.clone(), set_name);
    if let Some(source_label) = variable_label(&metadata, &source) {
      upsert_variable_label(&mut metadata, generate.clone(), source_label);
    }
    self.label_metadata = normalize_label_metadata(metadata);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Encode(EncodeResult {
      dataset: next_dataset,
    }))
  }

  fn execute_decode(
    &mut self,
    source: String,
    generate: String,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "decode" })?
      .clone();
    let set_name = self
      .label_metadata
      .attachments
      .iter()
      .find(|(variable, _)| variable == &source)
      .map(|(_, set_name)| set_name.clone())
      .ok_or_else(|| RuntimeError::DecodeRequiresAttachedLabels {
        variable: source.clone(),
      })?;
    let value_set = self
      .label_metadata
      .value_sets
      .iter()
      .find(|value_set| value_set.name == set_name)
      .ok_or_else(|| RuntimeError::DecodeRequiresAttachedLabels {
        variable: source.clone(),
      })?;
    let mapping = value_set
      .mappings
      .iter()
      .map(|(value, text)| match value {
        LabelValue::Integer(code) => Ok((*code, text.clone())),
        LabelValue::Number(_) | LabelValue::Text(_) => {
          Err(RuntimeError::DecodeRequiresIntegerLabels {
            variable: source.clone(),
          })
        }
      })
      .collect::<Result<Vec<_>, RuntimeError>>()?;
    let source_column = dataset
      .columns
      .iter()
      .find(|column| column.name == source)
      .ok_or_else(|| RuntimeError::DecodeUnknownVariable {
        variable: source.clone(),
      })?;
    if dataset.columns.iter().any(|column| column.name == generate) {
      return Err(RuntimeError::DecodeTargetExists { variable: generate });
    }
    if data_type_expression_domain(&source_column.data_type) != ExpressionDomain::Numeric {
      return Err(RuntimeError::DecodeRequiresNumeric { variable: source });
    }

    let backend = self.backend.as_mut().ok_or(RuntimeError::DecodeFailed)?;
    let next_dataset = backend
      .decode_column(&dataset, &source, &generate, &mapping)
      .map_err(|_| RuntimeError::DecodeFailed)?;
    if let Some(source_label) = variable_label(&self.label_metadata, &source) {
      let mut metadata = self.label_metadata.clone();
      upsert_variable_label(&mut metadata, generate.clone(), source_label);
      self.label_metadata = normalize_label_metadata(metadata);
    }
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Decode(DecodeResult {
      dataset: next_dataset,
    }))
  }

  fn execute_label(&mut self, command: LabelCommand) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "label" })?
      .clone();
    match command {
      LabelCommand::Variable { variable, text } => {
        ensure_label_variable(&dataset, &variable)?;
        let mut metadata = self.label_metadata.clone();
        metadata
          .variable_labels
          .retain(|(name, _)| name != &variable);
        let (action, message) = if let Some(text) = text {
          metadata.variable_labels.push((variable.clone(), text));
          (
            LabelResultAction::Variable,
            format!("Labeled variable {variable}"),
          )
        } else {
          (
            LabelResultAction::Variable,
            format!("Cleared variable label for {variable}"),
          )
        };
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(action, message, &self.label_metadata))
      }
      LabelCommand::Define {
        name,
        mappings,
        replace,
      } => {
        if mappings.is_empty() {
          return Err(RuntimeError::LabelDefineNoMappings);
        }
        let mut seen = BTreeSet::new();
        for (value, _) in &mappings {
          if !seen.insert(value.clone()) {
            return Err(RuntimeError::LabelDefineDuplicateValue {
              set_name: name,
              value: value.clone(),
            });
          }
        }
        if self
          .label_metadata
          .value_sets
          .iter()
          .any(|value_set| value_set.name == name)
          && !replace
        {
          return Err(RuntimeError::LabelDefineSetExists { set_name: name });
        }
        let mut metadata = self.label_metadata.clone();
        upsert_value_label_set(
          &mut metadata,
          ValueLabelSet {
            name: name.clone(),
            mappings,
          },
        );
        self.label_metadata = normalize_label_metadata(metadata);
        let verb = if replace { "Replaced" } else { "Defined" };
        Ok(self.label_result(
          LabelResultAction::Define,
          format!("{verb} value label set {name}"),
          &self.label_metadata,
        ))
      }
      LabelCommand::Values { variable, set_name } => {
        ensure_label_variable(&dataset, &variable)?;
        let mut metadata = self.label_metadata.clone();
        metadata.attachments.retain(|(name, _)| name != &variable);
        let (message, action) = if let Some(set_name) = set_name {
          if !metadata
            .value_sets
            .iter()
            .any(|value_set| value_set.name == set_name)
          {
            return Err(RuntimeError::LabelValuesUnknownSet { set_name });
          }
          metadata
            .attachments
            .push((variable.clone(), set_name.clone()));
          (
            format!("Attached value label set {set_name} to {variable}"),
            LabelResultAction::Values,
          )
        } else {
          (
            format!("Cleared value labels for {variable}"),
            LabelResultAction::Values,
          )
        };
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(action, message, &self.label_metadata))
      }
      LabelCommand::List { names } => {
        let metadata = if names.is_empty() {
          self.label_metadata.clone()
        } else {
          let available = self
            .label_metadata
            .value_sets
            .iter()
            .map(|value_set| value_set.name.as_str())
            .collect::<BTreeSet<_>>();
          let missing = names
            .iter()
            .filter(|name| !available.contains(name.as_str()))
            .cloned()
            .collect::<BTreeSet<_>>();
          if !missing.is_empty() {
            return Err(RuntimeError::LabelListUnknownSet {
              names: missing.into_iter().collect(),
            });
          }
          let wanted = names.iter().collect::<BTreeSet<_>>();
          LabelMetadata {
            variable_labels: self.label_metadata.variable_labels.clone(),
            value_sets: self
              .label_metadata
              .value_sets
              .iter()
              .filter(|value_set| wanted.contains(&value_set.name))
              .cloned()
              .collect(),
            attachments: self
              .label_metadata
              .attachments
              .iter()
              .filter(|(_, set_name)| wanted.contains(set_name))
              .cloned()
              .collect(),
          }
        };
        Ok(self.label_result(
          LabelResultAction::List,
          "Label dictionary".to_owned(),
          &normalize_label_metadata(metadata),
        ))
      }
      LabelCommand::Drop { names } => {
        let available = self
          .label_metadata
          .value_sets
          .iter()
          .map(|value_set| value_set.name.as_str())
          .collect::<BTreeSet<_>>();
        let missing = names
          .iter()
          .filter(|name| !available.contains(name.as_str()))
          .cloned()
          .collect::<BTreeSet<_>>();
        if !missing.is_empty() {
          return Err(RuntimeError::LabelDropUnknownSet {
            names: missing.into_iter().collect(),
          });
        }
        let dropped = names.iter().collect::<BTreeSet<_>>();
        let mut metadata = self.label_metadata.clone();
        metadata
          .value_sets
          .retain(|value_set| !dropped.contains(&value_set.name));
        metadata
          .attachments
          .retain(|(_, set_name)| !dropped.contains(set_name));
        self.label_metadata = normalize_label_metadata(metadata);
        Ok(self.label_result(
          LabelResultAction::Drop,
          format!("Dropped label set(s): {}", names.join(", ")),
          &self.label_metadata,
        ))
      }
    }
  }

  fn label_result(
    &self,
    action: LabelResultAction,
    message: String,
    metadata: &LabelMetadata,
  ) -> ExecutionResult {
    ExecutionResult::Label(LabelResult {
      action,
      message,
      metadata: (!metadata.is_empty()).then_some(metadata.clone()),
    })
  }

  fn execute_tabulate(&self, command: TabulateCommand) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "tabulate",
      })?;
    if command.row_variables.is_empty() {
      return Err(RuntimeError::TabulateNoVariables);
    }
    if command.row_variables.len() != 1 || command.column_variables.len() > 1 {
      return Err(RuntimeError::TabulateUnsupportedDimensions);
    }

    let row_variable = &command.row_variables[0];
    let column_variable = command.column_variables.first();
    if column_variable.is_some_and(|variable| variable == row_variable) {
      return Err(RuntimeError::TabulateDuplicateVariable {
        variable: row_variable.clone(),
      });
    }
    if column_variable.is_none() && command.row_percent {
      return Err(RuntimeError::TabulatePercentageRequiresTwoWay { option: "row" });
    }
    if column_variable.is_none() && command.column_percent {
      return Err(RuntimeError::TabulatePercentageRequiresTwoWay { option: "col" });
    }

    let known_variables = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<std::collections::BTreeSet<_>>();
    let mut unknown = Vec::new();
    if !known_variables.contains(row_variable.as_str()) {
      unknown.push(row_variable.clone());
    }
    if let Some(variable) = column_variable
      && !known_variables.contains(variable.as_str())
    {
      unknown.push(variable.clone());
    }
    if !unknown.is_empty() {
      return Err(RuntimeError::TabulateUnknownVariable { variables: unknown });
    }

    let backend = self.backend.as_ref().ok_or(RuntimeError::TabulateFailed)?;
    let counts = backend
      .tabulate_counts(
        row_variable,
        column_variable.map(String::as_str),
        command.include_missing,
      )
      .map_err(|_| RuntimeError::TabulateFailed)?;

    if let Some(column_variable) = column_variable {
      let mut row_values = Vec::new();
      let mut column_values = Vec::new();
      for count in &counts {
        push_unique_tabulate_value(&mut row_values, &count.row);
        let Some(column) = count.column.as_ref() else {
          return Err(RuntimeError::TabulateFailed);
        };
        push_unique_tabulate_value(&mut column_values, column);
      }
      row_values.sort_by(tabulate_value_ordering);
      column_values.sort_by(tabulate_value_ordering);

      let mut headers = vec![row_variable.clone()];
      for column in &column_values {
        let displayed = if command.nolabel {
          column.clone()
        } else {
          tabulate_display_value(&self.label_metadata, column_variable, column)
        };
        let label = tabulate_value_text(&displayed);
        headers.push(format!("{label} Count"));
        if command.row_percent {
          headers.push(format!("{label} Row %"));
        }
        if command.column_percent {
          headers.push(format!("{label} Col %"));
        }
      }

      let mut rows = Vec::with_capacity(row_values.len());
      for row_value in &row_values {
        let displayed_row = if command.nolabel {
          row_value.clone()
        } else {
          tabulate_display_value(&self.label_metadata, row_variable, row_value)
        };
        let row_total = counts
          .iter()
          .filter(|entry| tabulate_values_equal(&entry.row, row_value))
          .map(|entry| entry.count)
          .sum::<u64>();
        let mut output = vec![displayed_row];
        for column_value in &column_values {
          let count = counts
            .iter()
            .find(|entry| {
              tabulate_values_equal(&entry.row, row_value)
                && entry
                  .column
                  .as_ref()
                  .is_some_and(|entry_column| tabulate_values_equal(entry_column, column_value))
            })
            .map_or(0, |entry| entry.count);
          output.push(CellValue::SignedInteger(i128::from(count)));
          if command.row_percent {
            output.push(CellValue::Float(if row_total == 0 {
              0.0
            } else {
              100.0 * count as f64 / row_total as f64
            }));
          }
          if command.column_percent {
            let column_total = counts
              .iter()
              .filter(|entry| {
                entry
                  .column
                  .as_ref()
                  .is_some_and(|entry_column| tabulate_values_equal(entry_column, column_value))
              })
              .map(|entry| entry.count)
              .sum::<u64>();
            output.push(CellValue::Float(if column_total == 0 {
              0.0
            } else {
              100.0 * count as f64 / column_total as f64
            }));
          }
        }
        rows.push(output);
      }
      return Ok(ExecutionResult::Tabulate(TabulateResult { headers, rows }));
    }

    let total = counts.iter().map(|entry| entry.count).sum::<u64>();
    let mut rows = Vec::with_capacity(counts.len());
    for entry in counts {
      let displayed = if command.nolabel {
        entry.row
      } else {
        tabulate_display_value(&self.label_metadata, row_variable, &entry.row)
      };
      rows.push(vec![
        displayed,
        CellValue::SignedInteger(i128::from(entry.count)),
        CellValue::Float(if total == 0 {
          0.0
        } else {
          100.0 * entry.count as f64 / total as f64
        }),
      ]);
    }
    Ok(ExecutionResult::Tabulate(TabulateResult {
      headers: vec![
        row_variable.clone(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows,
    }))
  }

  fn execute_by(&self, command: ByCommand) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "by" })?;
    if command.groups.is_empty() {
      return Err(RuntimeError::ByNoGroups);
    }

    let known_variables = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let unknown_groups = command
      .groups
      .iter()
      .filter(|group| !known_variables.contains(group.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown_groups.is_empty() {
      return Err(RuntimeError::ByUnknownVariable {
        variables: unknown_groups,
      });
    }

    match *command.command {
      Command::Summarize { variables } => {
        let unknown_variables = variables
          .iter()
          .filter(|variable| !known_variables.contains(variable.as_str()))
          .cloned()
          .collect::<Vec<_>>();
        if !unknown_variables.is_empty() {
          return Err(RuntimeError::ByUnknownVariable {
            variables: unknown_variables,
          });
        }

        let requested = if variables.is_empty() {
          dataset
            .columns
            .iter()
            .filter(|column| {
              is_numeric_data_type(&column.data_type)
                && !command.groups.iter().any(|group| group == &column.name)
            })
            .map(|column| column.name.clone())
            .collect::<Vec<_>>()
        } else {
          variables
        };
        if requested.is_empty() {
          return Err(RuntimeError::BySummarizeNoNumericColumns);
        }

        let non_numeric = requested
          .iter()
          .filter(|variable| {
            dataset
              .columns
              .iter()
              .find(|column| column.name == **variable)
              .is_none_or(|column| !is_numeric_data_type(&column.data_type))
          })
          .cloned()
          .collect::<Vec<_>>();
        if !non_numeric.is_empty() {
          return Err(RuntimeError::BySummarizeRequiresNumeric {
            variables: non_numeric,
          });
        }

        let backend = self.backend.as_ref().ok_or(RuntimeError::ByFailed)?;
        let rows = backend
          .grouped_summarize(&command.groups, &requested)
          .map_err(|_| RuntimeError::ByFailed)?;
        let headers = command
          .groups
          .iter()
          .cloned()
          .chain(requested.iter().map(|variable| format!("mean_{variable}")))
          .collect();
        Ok(ExecutionResult::By(ByResult { headers, rows }))
      }
      Command::Count => {
        let backend = self.backend.as_ref().ok_or(RuntimeError::ByFailed)?;
        let rows = backend
          .grouped_count(&command.groups)
          .map_err(|_| RuntimeError::ByFailed)?;
        let headers = command
          .groups
          .iter()
          .cloned()
          .chain(std::iter::once("Count".to_owned()))
          .collect();
        Ok(ExecutionResult::By(ByResult { headers, rows }))
      }
      _ => Err(RuntimeError::ByUnsupportedCommand),
    }
  }

  fn execute_collapse(
    &mut self,
    command: CollapseCommand,
  ) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset {
        command: "collapse",
      })?
      .clone();
    if command.variables.is_empty() {
      return Err(RuntimeError::CollapseNoVariables);
    }
    if command.groups.is_empty() {
      return Err(RuntimeError::CollapseNoGroups);
    }

    let known_columns = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let unknown = command
      .groups
      .iter()
      .chain(&command.variables)
      .filter(|variable| !known_columns.contains(variable.as_str()))
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::CollapseUnknownVariable { variables: unknown });
    }

    if command.statistic != CollapseStatistic::Count {
      let non_numeric = command
        .variables
        .iter()
        .filter(|variable| {
          dataset
            .columns
            .iter()
            .find(|column| column.name == **variable)
            .is_some_and(|column| !is_numeric_data_type(&column.data_type))
        })
        .cloned()
        .collect::<Vec<_>>();
      if !non_numeric.is_empty() {
        return Err(RuntimeError::CollapseRequiresNumeric {
          variables: non_numeric,
        });
      }
    }

    let next_dataset = self
      .backend
      .as_mut()
      .ok_or(RuntimeError::CollapseFailed)?
      .collapse(
        &dataset,
        command.statistic,
        &command.variables,
        &command.groups,
      )
      .map_err(|_| RuntimeError::CollapseFailed)?;
    self.retain_label_metadata(&next_dataset);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Collapse(CollapseResult {
      dataset: next_dataset,
    }))
  }

  fn execute_keep(&mut self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "keep" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::KeepNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::KeepUnknownVariable { variables: unknown });
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::KeepFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &variables)
      .map_err(|_| RuntimeError::KeepFailed)?;
    self.retain_label_metadata(&next_dataset);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Keep(KeepResult {
      dataset: next_dataset,
    }))
  }

  fn execute_drop(&mut self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "drop" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::DropNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::DropUnknownVariable { variables: unknown });
    }
    let remaining = dataset
      .columns
      .iter()
      .filter(|column| !variables.iter().any(|variable| variable == &column.name))
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if remaining.is_empty() {
      return Err(RuntimeError::DropWouldRemoveEveryColumn);
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::DropFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &remaining)
      .map_err(|_| RuntimeError::DropFailed)?;
    self.retain_label_metadata(&next_dataset);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Drop(DropResult {
      dataset: next_dataset,
    }))
  }

  fn execute_select(&mut self, variables: Vec<String>) -> Result<ExecutionResult, RuntimeError> {
    let dataset = self
      .active_dataset
      .as_ref()
      .ok_or(RuntimeError::NoActiveDataset { command: "select" })?
      .clone();
    if variables.is_empty() {
      return Err(RuntimeError::SelectNoVariables);
    }
    let unknown = variables
      .iter()
      .filter(|variable| {
        !dataset
          .columns
          .iter()
          .any(|column| column.name == **variable)
      })
      .cloned()
      .collect::<Vec<_>>();
    if !unknown.is_empty() {
      return Err(RuntimeError::SelectUnknownVariable { variables: unknown });
    }
    let backend = self.backend.as_mut().ok_or(RuntimeError::SelectFailed)?;
    let next_dataset = backend
      .project_columns(&dataset, &variables)
      .map_err(|_| RuntimeError::SelectFailed)?;
    self.retain_label_metadata(&next_dataset);
    self.active_dataset = Some(next_dataset.clone());
    Ok(ExecutionResult::Select(SelectResult {
      dataset: next_dataset,
    }))
  }

  fn retain_label_metadata(&mut self, dataset: &DatasetInfo) {
    let columns = dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<BTreeSet<_>>();
    let mut metadata = self.label_metadata.clone();
    metadata
      .variable_labels
      .retain(|(variable, _)| columns.contains(variable.as_str()));
    metadata
      .attachments
      .retain(|(variable, _)| columns.contains(variable.as_str()));
    self.label_metadata = normalize_label_metadata(metadata);
  }

  fn remove_value_label_attachment(&mut self, variable: &str) {
    self
      .label_metadata
      .attachments
      .retain(|(name, _)| name != variable);
    self.label_metadata = normalize_label_metadata(self.label_metadata.clone());
  }

  fn rename_label_metadata(&mut self, old_name: &str, new_name: &str) {
    let mut metadata = self.label_metadata.clone();
    for (name, _) in &mut metadata.variable_labels {
      if name == old_name {
        *name = new_name.to_owned();
      }
    }
    for (name, _) in &mut metadata.attachments {
      if name == old_name {
        *name = new_name.to_owned();
      }
    }
    self.label_metadata = normalize_label_metadata(metadata);
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
    self.label_metadata = LabelMetadata::default();
    self.active_dataset = Some(dataset.clone());
    Ok(ExecutionResult::Load(LoadResult { dataset }))
  }
}

fn ensure_label_variable(dataset: &DatasetInfo, variable: &str) -> Result<(), RuntimeError> {
  if dataset.columns.iter().any(|column| column.name == variable) {
    Ok(())
  } else {
    Err(RuntimeError::LabelUnknownVariable {
      variable: variable.to_owned(),
    })
  }
}

fn normalize_label_metadata(mut metadata: LabelMetadata) -> LabelMetadata {
  metadata
    .variable_labels
    .sort_by(|left, right| left.0.cmp(&right.0));
  metadata
    .value_sets
    .sort_by(|left, right| left.name.cmp(&right.name));
  metadata
    .attachments
    .sort_by(|left, right| left.0.cmp(&right.0));
  metadata
}

fn variable_label(metadata: &LabelMetadata, variable: &str) -> Option<String> {
  metadata
    .variable_labels
    .iter()
    .find(|(name, _)| name == variable)
    .map(|(_, text)| text.clone())
}

fn upsert_variable_label(metadata: &mut LabelMetadata, variable: String, text: String) {
  metadata
    .variable_labels
    .retain(|(name, _)| name != &variable);
  metadata.variable_labels.push((variable, text));
}

fn upsert_value_label_set(metadata: &mut LabelMetadata, value_set: ValueLabelSet) {
  metadata
    .value_sets
    .retain(|existing| existing.name != value_set.name);
  metadata.value_sets.push(value_set);
}

fn upsert_attachment(metadata: &mut LabelMetadata, variable: String, set_name: String) {
  metadata.attachments.retain(|(name, _)| name != &variable);
  metadata.attachments.push((variable, set_name));
}

fn push_unique_tabulate_value(values: &mut Vec<CellValue>, value: &CellValue) {
  if !values
    .iter()
    .any(|existing| tabulate_values_equal(existing, value))
  {
    values.push(value.clone());
  }
}

fn tabulate_values_equal(left: &CellValue, right: &CellValue) -> bool {
  match (left, right) {
    (CellValue::Float(left), CellValue::Float(right)) => {
      left == right || (left.is_nan() && right.is_nan())
    }
    _ => left == right,
  }
}

fn tabulate_value_ordering(left: &CellValue, right: &CellValue) -> std::cmp::Ordering {
  use std::cmp::Ordering;

  match (left, right) {
    (CellValue::Null, CellValue::Null) => Ordering::Equal,
    (CellValue::Null, _) => Ordering::Greater,
    (_, CellValue::Null) => Ordering::Less,
    (CellValue::Boolean(left), CellValue::Boolean(right)) => left.cmp(right),
    (CellValue::SignedInteger(left), CellValue::SignedInteger(right)) => left.cmp(right),
    (CellValue::UnsignedInteger(left), CellValue::UnsignedInteger(right)) => left.cmp(right),
    (CellValue::Float(left), CellValue::Float(right)) => left.total_cmp(right),
    (
      CellValue::Decimal {
        scale: left_scale,
        value: left,
        ..
      },
      CellValue::Decimal {
        scale: right_scale,
        value: right,
        ..
      },
    ) if left_scale == right_scale => left.cmp(right),
    (CellValue::Text(left), CellValue::Text(right)) => left.cmp(right),
    (CellValue::Bytes(left), CellValue::Bytes(right)) => left.cmp(right),
    _ => tabulate_value_text(left).cmp(&tabulate_value_text(right)),
  }
}

fn tabulate_display_value(
  metadata: &LabelMetadata,
  variable: &str,
  value: &CellValue,
) -> CellValue {
  let Some((_, set_name)) = metadata
    .attachments
    .iter()
    .find(|(name, _)| name == variable)
  else {
    return value.clone();
  };
  let Some(value_set) = metadata
    .value_sets
    .iter()
    .find(|value_set| &value_set.name == set_name)
  else {
    return value.clone();
  };
  value_set
    .mappings
    .iter()
    .find(|(label_value, _)| tabulate_label_matches_cell(label_value, value))
    .map_or_else(|| value.clone(), |(_, text)| CellValue::Text(text.clone()))
}

fn tabulate_label_matches_cell(label: &LabelValue, value: &CellValue) -> bool {
  match (label, value) {
    (LabelValue::Integer(label), CellValue::SignedInteger(value)) => i128::from(*label) == *value,
    (LabelValue::Integer(label), CellValue::UnsignedInteger(value)) => {
      *label >= 0 && u128::try_from(*label).is_ok_and(|label| label == *value)
    }
    (LabelValue::Number(label), CellValue::Float(value)) => label
      .parse::<f64>()
      .is_ok_and(|label| label == *value || (label.is_nan() && value.is_nan())),
    (LabelValue::Text(label), CellValue::Text(value)) => label == value,
    _ => false,
  }
}

fn tabulate_value_text(value: &CellValue) -> String {
  match value {
    CellValue::Null => "missing".to_owned(),
    CellValue::Boolean(value) => value.to_string(),
    CellValue::SignedInteger(value) => value.to_string(),
    CellValue::UnsignedInteger(value) => value.to_string(),
    CellValue::Float(value) => value.to_string(),
    CellValue::Decimal { value, scale, .. } => {
      if *scale == 0 {
        return value.to_string();
      }
      let negative = *value < 0;
      let digits = value.unsigned_abs().to_string();
      let scale = usize::from(*scale);
      let rendered = if digits.len() <= scale {
        format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
      } else {
        let split = digits.len() - scale;
        format!("{}.{}", &digits[..split], &digits[split..])
      };
      if negative {
        format!("-{rendered}")
      } else {
        rendered
      }
    }
    CellValue::Text(value) => value.clone(),
    CellValue::Bytes(value) => format!("0x{}", hex_bytes(value)),
  }
}

fn hex_bytes(bytes: &[u8]) -> String {
  bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
    Command::Assert { .. } => "assert",
    Command::Generate { .. } => "generate",
    Command::Replace { .. } => "replace",
    Command::Keep { .. } => "keep",
    Command::Drop { .. } => "drop",
    Command::Missing { .. } => "missing",
    Command::Duplicates { .. } => "duplicates",
    Command::Isid { .. } => "isid",
    Command::Select { .. } => "select",
    Command::Sort { .. } => "sort",
    Command::Gsort { .. } => "gsort",
    Command::Recode { .. } => "recode",
    Command::Encode { .. } => "encode",
    Command::Decode { .. } => "decode",
    Command::Label { .. } => "label",
    Command::Tabulate { .. } => "tabulate",
    Command::Join { .. } => "join",
    Command::By { .. } => "by",
    Command::Collapse { .. } => "collapse",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpressionDomain {
  Numeric,
  String,
  Boolean,
  Null,
  Other,
}

fn expression_identifiers(expression: &AssertExpression) -> Vec<String> {
  match expression {
    AssertExpression::Identifier(name) => vec![name.clone()],
    AssertExpression::Number(_) | AssertExpression::String(_) | AssertExpression::Null => {
      Vec::new()
    }
    AssertExpression::UnaryMinus(operand) => expression_identifiers(operand),
    AssertExpression::Binary { left, right, .. } => {
      let mut identifiers = expression_identifiers(left);
      identifiers.extend(expression_identifiers(right));
      identifiers
    }
  }
}

fn generate_expression_identifiers(expression: &GenerateExpression) -> Vec<String> {
  match expression {
    GenerateExpression::Identifier(name) => vec![name.clone()],
    GenerateExpression::Number(_) | GenerateExpression::String(_) | GenerateExpression::Null => {
      Vec::new()
    }
    GenerateExpression::UnaryMinus(operand) => generate_expression_identifiers(operand),
    GenerateExpression::Binary { left, right, .. } => {
      let mut identifiers = generate_expression_identifiers(left);
      identifiers.extend(generate_expression_identifiers(right));
      identifiers
    }
    GenerateExpression::FunctionCall { arguments, .. } => arguments
      .iter()
      .flat_map(generate_expression_identifiers)
      .collect(),
  }
}

fn replace_expression_identifiers(expression: &GenerateExpression) -> Vec<String> {
  match expression {
    GenerateExpression::Identifier(name) => vec![name.clone()],
    GenerateExpression::Number(_) | GenerateExpression::String(_) | GenerateExpression::Null => {
      Vec::new()
    }
    GenerateExpression::UnaryMinus(operand) => replace_expression_identifiers(operand),
    GenerateExpression::Binary { left, right, .. } => {
      let mut identifiers = replace_expression_identifiers(left);
      identifiers.extend(replace_expression_identifiers(right));
      identifiers
    }
    GenerateExpression::FunctionCall { arguments, .. } => arguments
      .iter()
      .flat_map(replace_expression_identifiers)
      .collect(),
  }
}

fn replace_expression_to_assert(
  expression: &GenerateExpression,
) -> Result<AssertExpression, RuntimeError> {
  match expression {
    GenerateExpression::Identifier(name) => Ok(AssertExpression::Identifier(name.clone())),
    GenerateExpression::Number(value) => Ok(AssertExpression::Number(value.clone())),
    GenerateExpression::String(value) => Ok(AssertExpression::String(value.clone())),
    GenerateExpression::Null => Ok(AssertExpression::Null),
    GenerateExpression::UnaryMinus(operand) => Ok(AssertExpression::UnaryMinus(Box::new(
      replace_expression_to_assert(operand)?,
    ))),
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let operator = match operator {
        GenerateBinaryOperator::Add => AssertBinaryOperator::Add,
        GenerateBinaryOperator::Subtract => AssertBinaryOperator::Subtract,
        GenerateBinaryOperator::Multiply => AssertBinaryOperator::Multiply,
        GenerateBinaryOperator::Divide => AssertBinaryOperator::Divide,
        GenerateBinaryOperator::Equal => AssertBinaryOperator::Equal,
        GenerateBinaryOperator::NotEqual => AssertBinaryOperator::NotEqual,
        GenerateBinaryOperator::Less => AssertBinaryOperator::Less,
        GenerateBinaryOperator::LessOrEqual => AssertBinaryOperator::LessOrEqual,
        GenerateBinaryOperator::Greater => AssertBinaryOperator::Greater,
        GenerateBinaryOperator::GreaterOrEqual => AssertBinaryOperator::GreaterOrEqual,
      };
      Ok(AssertExpression::Binary {
        left: Box::new(replace_expression_to_assert(left)?),
        operator,
        right: Box::new(replace_expression_to_assert(right)?),
      })
    }
    GenerateExpression::FunctionCall { .. } => Err(RuntimeError::ReplaceUnsupportedExpression {
      message: "replace does not support function calls".to_owned(),
    }),
  }
}

fn map_replace_error(error: RuntimeError) -> RuntimeError {
  match error {
    RuntimeError::AssertUnknownVariable { variables } => {
      RuntimeError::ReplaceUnknownVariable { variables }
    }
    RuntimeError::AssertTypeMismatch { message } => RuntimeError::ReplaceTypeMismatch { message },
    other => other,
  }
}

fn generate_expression_to_assert(
  expression: &GenerateExpression,
) -> Result<AssertExpression, RuntimeError> {
  match expression {
    GenerateExpression::Identifier(name) => Ok(AssertExpression::Identifier(name.clone())),
    GenerateExpression::Number(value) => Ok(AssertExpression::Number(value.clone())),
    GenerateExpression::String(_) => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support string expressions".to_owned(),
    }),
    GenerateExpression::Null => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support NULL expressions".to_owned(),
    }),
    GenerateExpression::UnaryMinus(operand) => Ok(AssertExpression::UnaryMinus(Box::new(
      generate_expression_to_assert(operand)?,
    ))),
    GenerateExpression::Binary {
      left,
      operator,
      right,
    } => {
      let operator = match operator {
        GenerateBinaryOperator::Add => AssertBinaryOperator::Add,
        GenerateBinaryOperator::Subtract => AssertBinaryOperator::Subtract,
        GenerateBinaryOperator::Multiply => AssertBinaryOperator::Multiply,
        GenerateBinaryOperator::Divide => AssertBinaryOperator::Divide,
        GenerateBinaryOperator::Equal
        | GenerateBinaryOperator::NotEqual
        | GenerateBinaryOperator::Less
        | GenerateBinaryOperator::LessOrEqual
        | GenerateBinaryOperator::Greater
        | GenerateBinaryOperator::GreaterOrEqual => {
          return Err(RuntimeError::GenerateUnsupportedExpression {
            message: "generate does not support comparison expressions".to_owned(),
          });
        }
      };
      Ok(AssertExpression::Binary {
        left: Box::new(generate_expression_to_assert(left)?),
        operator,
        right: Box::new(generate_expression_to_assert(right)?),
      })
    }
    GenerateExpression::FunctionCall { .. } => Err(RuntimeError::GenerateUnsupportedExpression {
      message: "generate does not support function calls".to_owned(),
    }),
  }
}

fn validate_generate_expression(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<(), RuntimeError> {
  match expression {
    AssertExpression::Identifier(name) => {
      let is_numeric = dataset
        .columns
        .iter()
        .find(|column| column.name == *name)
        .is_some_and(|column| is_numeric_data_type(&column.data_type));
      if !is_numeric {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
    }
    AssertExpression::Number(value) => {
      if !value.parse::<f64>().is_ok_and(f64::is_finite) {
        return Err(RuntimeError::GenerateTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
    }
    AssertExpression::UnaryMinus(operand) => validate_generate_expression(operand, dataset)?,
    AssertExpression::Binary { left, right, .. } => {
      validate_generate_expression(left, dataset)?;
      validate_generate_expression(right, dataset)?;
    }
    AssertExpression::String(_) | AssertExpression::Null => {
      return Err(RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support non-numeric expressions".to_owned(),
      });
    }
  }
  Ok(())
}

fn expression_domain(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<ExpressionDomain, RuntimeError> {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .find(|column| column.name == *name)
      .map(|column| data_type_expression_domain(&column.data_type))
      .ok_or_else(|| RuntimeError::AssertUnknownVariable {
        variables: vec![name.clone()],
      }),
    AssertExpression::Number(_) => Ok(ExpressionDomain::Numeric),
    AssertExpression::UnaryMinus(operand) => {
      if expression_contains_unsigned_identifier(operand, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message:
            "expression type mismatch: unsigned numeric values do not support subtraction or unary minus"
              .to_owned(),
        });
      }
      let operand_domain = expression_domain(operand, dataset)?;
      if operand_domain == ExpressionDomain::Null {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "null literal only supports equality and inequality comparisons".to_owned(),
        });
      }
      if operand_domain != ExpressionDomain::Numeric {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: unary minus requires numeric operand".to_owned(),
        });
      }
      Ok(ExpressionDomain::Numeric)
    }
    AssertExpression::String(_) => Ok(ExpressionDomain::String),
    AssertExpression::Null => Ok(ExpressionDomain::Null),
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      if expression_has_unsafe_unsigned_arithmetic(expression, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message:
            "expression type mismatch: unsigned numeric values do not support subtraction or unary minus"
              .to_owned(),
        });
      }
      if expression_has_unsafe_unsigned_numeric_pair(expression, dataset) {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: unsigned numeric values cannot be combined with negative numeric literals"
            .to_owned(),
        });
      }
      let left_domain = expression_domain(left, dataset)?;
      let right_domain = expression_domain(right, dataset)?;
      if operator.is_comparison() {
        if left_domain == ExpressionDomain::Null || right_domain == ExpressionDomain::Null {
          if !matches!(
            operator,
            AssertBinaryOperator::Equal | AssertBinaryOperator::NotEqual
          ) {
            return Err(RuntimeError::AssertTypeMismatch {
              message: "null literal only supports equality and inequality comparisons".to_owned(),
            });
          }
          return Ok(ExpressionDomain::Boolean);
        }
        let compatible = left_domain == right_domain
          || (left_domain == ExpressionDomain::Numeric
            && right_domain == ExpressionDomain::Numeric);
        if !compatible {
          return Err(RuntimeError::AssertTypeMismatch {
            message: format!(
              "expression type mismatch: cannot compare {} and {} values",
              expression_domain_name(left_domain),
              expression_domain_name(right_domain)
            ),
          });
        }
        return Ok(ExpressionDomain::Boolean);
      }
      if expression_contains_null_literal(left) || expression_contains_null_literal(right) {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "null literal only supports equality and inequality comparisons".to_owned(),
        });
      }
      if left_domain != ExpressionDomain::Numeric || right_domain != ExpressionDomain::Numeric {
        return Err(RuntimeError::AssertTypeMismatch {
          message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
        });
      }
      Ok(ExpressionDomain::Numeric)
    }
  }
}

fn data_type_expression_domain(data_type: &str) -> ExpressionDomain {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  if is_numeric_data_type(data_type) {
    ExpressionDomain::Numeric
  } else if matches!(base, "BOOLEAN" | "BOOL") {
    ExpressionDomain::Boolean
  } else if matches!(base, "VARCHAR" | "TEXT" | "STRING" | "CHAR") {
    ExpressionDomain::String
  } else {
    ExpressionDomain::Other
  }
}

fn expression_domain_name(domain: ExpressionDomain) -> &'static str {
  match domain {
    ExpressionDomain::Numeric => "numeric",
    ExpressionDomain::String => "string",
    ExpressionDomain::Boolean => "boolean",
    ExpressionDomain::Null => "null",
    ExpressionDomain::Other => "unsupported",
  }
}

fn expression_contains_null_literal(expression: &AssertExpression) -> bool {
  match expression {
    AssertExpression::Null => true,
    AssertExpression::UnaryMinus(operand) => expression_contains_null_literal(operand),
    AssertExpression::Binary { left, right, .. } => {
      expression_contains_null_literal(left) || expression_contains_null_literal(right)
    }
    AssertExpression::Identifier(_) | AssertExpression::Number(_) | AssertExpression::String(_) => {
      false
    }
  }
}

fn assert_operator_sql_symbol(operator: AssertBinaryOperator) -> &'static str {
  match operator {
    AssertBinaryOperator::Add => "+",
    AssertBinaryOperator::Subtract => "-",
    AssertBinaryOperator::Multiply => "*",
    AssertBinaryOperator::Divide => "/",
    AssertBinaryOperator::Equal => "=",
    AssertBinaryOperator::NotEqual => "!=",
    AssertBinaryOperator::Less => "<",
    AssertBinaryOperator::LessOrEqual => "<=",
    AssertBinaryOperator::Greater => ">",
    AssertBinaryOperator::GreaterOrEqual => ">=",
  }
}

fn compile_assert_expression(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, ()> {
  match expression {
    AssertExpression::Binary {
      left,
      operator,
      right,
    } if operator.is_comparison() => {
      let left_is_null = matches!(left.as_ref(), AssertExpression::Null);
      let right_is_null = matches!(right.as_ref(), AssertExpression::Null);
      if left_is_null || right_is_null {
        if left_is_null && right_is_null {
          return Ok(if *operator == AssertBinaryOperator::Equal {
            "TRUE".to_owned()
          } else if *operator == AssertBinaryOperator::NotEqual {
            "FALSE".to_owned()
          } else {
            return Err(());
          });
        }
        let other = if left_is_null { right } else { left };
        let other_sql = compile_assert_expression_operand(other, dataset)?;
        return Ok(format!(
          "({other_sql} {})",
          if *operator == AssertBinaryOperator::Equal {
            "IS NULL"
          } else if *operator == AssertBinaryOperator::NotEqual {
            "IS NOT NULL"
          } else {
            return Err(());
          }
        ));
      }
      let left_sql = compile_assert_expression_operand(left, dataset)?;
      let right_sql = compile_assert_expression_operand(right, dataset)?;
      Ok(format!(
        "({left_sql} {} {right_sql})",
        assert_operator_sql_symbol(*operator)
      ))
    }
    _ if expression_is_numeric_result(expression) => {
      let raw = compile_assert_expression_raw(expression, dataset)?;
      Ok(safe_numeric_sql(&raw))
    }
    _ => compile_assert_expression_raw(expression, dataset),
  }
}

fn compile_assert_expression_operand(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, ()> {
  if expression_is_numeric_result(expression) {
    let raw = compile_assert_expression_raw(expression, dataset)?;
    Ok(safe_numeric_sql(&raw))
  } else {
    compile_assert_expression_raw(expression, dataset)
  }
}

fn compile_assert_expression_raw(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> Result<String, ()> {
  match expression {
    AssertExpression::Identifier(name) => Ok(quote_identifier(name)),
    AssertExpression::Number(value) => {
      if value.parse::<f64>().is_err() {
        return Err(());
      }
      Ok(value.clone())
    }
    AssertExpression::String(value) => Ok(format!("'{}'", value.replace('\'', "''"))),
    AssertExpression::Null => Ok("NULL".to_owned()),
    AssertExpression::UnaryMinus(operand) => {
      let mut operand_sql = compile_assert_expression_raw(operand, dataset)?;
      if expression_is_integral(expression, dataset) {
        operand_sql = cast_exact_integer_sql(&operand_sql);
      }
      Ok(format!("-({operand_sql})"))
    }
    AssertExpression::Binary { operator, .. } if operator.is_comparison() => {
      compile_assert_expression(expression, dataset)
    }
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      let mut left_sql = compile_assert_expression_raw(left, dataset)?;
      let mut right_sql = compile_assert_expression_raw(right, dataset)?;
      if expression_is_integral(expression, dataset) {
        left_sql = cast_exact_integer_sql(&left_sql);
        right_sql = cast_exact_integer_sql(&right_sql);
      }
      Ok(format!(
        "({left_sql} {} {right_sql})",
        assert_operator_sql_symbol(*operator)
      ))
    }
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

fn is_integer_data_type(data_type: &str) -> bool {
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
  )
}

fn is_unsigned_data_type(data_type: &str) -> bool {
  let normalized = data_type.trim().to_ascii_uppercase();
  let base = normalized.split('(').next().unwrap_or_default().trim();
  matches!(
    base,
    "UTINYINT"
      | "USMALLINT"
      | "UINTEGER"
      | "UBIGINT"
      | "UHUGEINT"
      | "UINT8"
      | "UINT16"
      | "UINT32"
      | "UINT64"
      | "UINT128"
  )
}

fn expression_contains_unsigned_identifier(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .any(|column| column.name == *name && is_unsigned_data_type(&column.data_type)),
    AssertExpression::UnaryMinus(operand) => {
      expression_contains_unsigned_identifier(operand, dataset)
    }
    AssertExpression::Binary { left, right, .. } => {
      expression_contains_unsigned_identifier(left, dataset)
        || expression_contains_unsigned_identifier(right, dataset)
    }
    AssertExpression::Number(_) | AssertExpression::String(_) | AssertExpression::Null => false,
  }
}

fn expression_has_unsafe_unsigned_arithmetic(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  match expression {
    AssertExpression::UnaryMinus(operand) => {
      expression_contains_unsigned_identifier(operand, dataset)
    }
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      (*operator == AssertBinaryOperator::Subtract
        && (expression_contains_unsigned_identifier(left, dataset)
          || expression_contains_unsigned_identifier(right, dataset)))
        || expression_has_unsafe_unsigned_arithmetic(left, dataset)
        || expression_has_unsafe_unsigned_arithmetic(right, dataset)
    }
    AssertExpression::Identifier(_)
    | AssertExpression::Number(_)
    | AssertExpression::String(_)
    | AssertExpression::Null => false,
  }
}

fn expression_contains_negative_numeric_literal(expression: &AssertExpression) -> bool {
  match expression {
    AssertExpression::Number(value) => value.parse::<f64>().is_ok_and(|value| value < 0.0),
    AssertExpression::UnaryMinus(operand) => matches!(
      operand.as_ref(),
      AssertExpression::Number(value) if value.parse::<f64>().is_ok_and(|value| value != 0.0)
    ),
    AssertExpression::Identifier(_)
    | AssertExpression::String(_)
    | AssertExpression::Null
    | AssertExpression::Binary { .. } => false,
  }
}

fn expression_has_unsafe_unsigned_numeric_pair(
  expression: &AssertExpression,
  dataset: &DatasetInfo,
) -> bool {
  let AssertExpression::Binary { left, right, .. } = expression else {
    return false;
  };
  (expression_contains_unsigned_identifier(left, dataset)
    && expression_contains_negative_numeric_literal(right))
    || (expression_contains_unsigned_identifier(right, dataset)
      && expression_contains_negative_numeric_literal(left))
}

fn expression_is_integral(expression: &AssertExpression, dataset: &DatasetInfo) -> bool {
  match expression {
    AssertExpression::Identifier(name) => dataset
      .columns
      .iter()
      .any(|column| column.name == *name && is_integer_data_type(&column.data_type)),
    AssertExpression::Number(value) => !value.contains('.'),
    AssertExpression::UnaryMinus(operand) => expression_is_integral(operand, dataset),
    AssertExpression::Binary {
      left,
      operator,
      right,
    } => {
      matches!(
        operator,
        AssertBinaryOperator::Add | AssertBinaryOperator::Subtract | AssertBinaryOperator::Multiply
      ) && expression_is_integral(left, dataset)
        && expression_is_integral(right, dataset)
    }
    AssertExpression::String(_) | AssertExpression::Null => false,
  }
}

fn expression_is_numeric_result(expression: &AssertExpression) -> bool {
  matches!(
    expression,
    AssertExpression::UnaryMinus(_)
      | AssertExpression::Binary {
        operator: AssertBinaryOperator::Add
          | AssertBinaryOperator::Subtract
          | AssertBinaryOperator::Multiply
          | AssertBinaryOperator::Divide,
        ..
      }
  )
}

fn cast_exact_integer_sql(expression: &str) -> String {
  format!("CAST({expression} AS DECIMAL(38,0))")
}

fn safe_numeric_sql(expression: &str) -> String {
  format!(
    "(SELECT CASE WHEN isfinite(CAST(__tabdat_numeric_value AS DOUBLE)) THEN __tabdat_numeric_value ELSE NULL END FROM (SELECT try({expression}) AS __tabdat_numeric_value) AS __tabdat_numeric_result)"
  )
}

fn quote_identifier(identifier: &str) -> String {
  format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn collapse_statistic_name(statistic: CollapseStatistic) -> &'static str {
  match statistic {
    CollapseStatistic::Count => "count",
    CollapseStatistic::Mean => "mean",
    CollapseStatistic::Sum => "sum",
    CollapseStatistic::Min => "min",
    CollapseStatistic::Max => "max",
  }
}

fn quote_recode_literal(value: &str) -> String {
  format!("'{}'", value.replace('\'', "''"))
}

fn validate_recode_number(value: &str) -> Result<(), ()> {
  value
    .parse::<f64>()
    .is_ok_and(f64::is_finite)
    .then_some(())
    .ok_or(())
}

fn compile_recode_case(source: &ColumnInfo, rules: &[RecodeRule]) -> Result<String, ()> {
  let source_is_numeric = is_numeric_data_type(&source.data_type);
  let output_is_text = !source_is_numeric
    || rules
      .iter()
      .any(|rule| matches!(&rule.output, RecodeValue::Text(_)));
  let source_sql = quote_identifier(&source.name);
  let mut clauses = Vec::new();
  let mut else_sql = None;

  for rule in rules {
    let mut conditions = Vec::new();
    let mut is_else = false;
    for input in &rule.inputs {
      match input {
        RecodeInput::Value(value) => {
          conditions.push(compile_recode_value_condition(
            &source_sql,
            source_is_numeric,
            value,
          )?);
        }
        RecodeInput::Range { start, end } => {
          if !source_is_numeric {
            return Err(());
          }
          conditions.push(compile_recode_range_condition(&source_sql, start, end)?);
        }
        RecodeInput::Missing => conditions.push(format!("{source_sql} IS NULL")),
        RecodeInput::NonMissing => conditions.push(format!("{source_sql} IS NOT NULL")),
        RecodeInput::Else => is_else = true,
      }
    }
    let output_sql = compile_recode_output(&rule.output, output_is_text)?;
    if is_else {
      else_sql = Some(output_sql);
    } else if !conditions.is_empty() {
      let condition_sql = conditions
        .into_iter()
        .map(|condition| format!("({condition})"))
        .collect::<Vec<_>>()
        .join(" OR ");
      clauses.push(format!("WHEN {condition_sql} THEN {output_sql}"));
    }
  }

  let fallback_sql = else_sql.unwrap_or_else(|| {
    if output_is_text {
      format!("CAST({source_sql} AS VARCHAR)")
    } else {
      source_sql.clone()
    }
  });
  if clauses.is_empty() {
    return Ok(fallback_sql);
  }
  Ok(format!(
    "CASE {} ELSE {fallback_sql} END",
    clauses.join(" ")
  ))
}

fn compile_recode_value_condition(
  source_sql: &str,
  source_is_numeric: bool,
  value: &RecodeValue,
) -> Result<String, ()> {
  match value {
    RecodeValue::Number(value) => {
      validate_recode_number(value)?;
      if source_is_numeric {
        Ok(format!("{source_sql} = {value}"))
      } else {
        Ok(format!(
          "CAST({source_sql} AS VARCHAR) = {}",
          quote_recode_literal(value)
        ))
      }
    }
    RecodeValue::Text(value) => Ok(format!(
      "CAST({source_sql} AS VARCHAR) = {}",
      quote_recode_literal(value)
    )),
  }
}

fn compile_recode_range_condition(
  source_sql: &str,
  start: &RecodeRangeEndpoint,
  end: &RecodeRangeEndpoint,
) -> Result<String, ()> {
  let start_sql = recode_range_endpoint_sql(start)?;
  let end_sql = recode_range_endpoint_sql(end)?;
  match (start_sql, end_sql) {
    (None, None) => Ok(format!("{source_sql} IS NOT NULL")),
    (None, Some(end)) => Ok(format!("{source_sql} <= {end}")),
    (Some(start), None) => Ok(format!("{source_sql} >= {start}")),
    (Some(start), Some(end)) => Ok(format!("{source_sql} >= {start} AND {source_sql} <= {end}")),
  }
}

fn recode_range_endpoint_sql(endpoint: &RecodeRangeEndpoint) -> Result<Option<String>, ()> {
  match endpoint {
    RecodeRangeEndpoint::Min => Ok(None),
    RecodeRangeEndpoint::Max => Ok(None),
    RecodeRangeEndpoint::Number(value) => {
      validate_recode_number(value)?;
      Ok(Some(value.clone()))
    }
  }
}

fn compile_recode_output(value: &RecodeValue, output_is_text: bool) -> Result<String, ()> {
  match value {
    RecodeValue::Number(value) => {
      validate_recode_number(value)?;
      if output_is_text {
        Ok(quote_recode_literal(value))
      } else {
        Ok(value.clone())
      }
    }
    RecodeValue::Text(value) => Ok(quote_recode_literal(value)),
  }
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

  fn project_columns(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let select_list = variables
      .iter()
      .map(|variable| quote_identifier(variable))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn collapse(
    &mut self,
    dataset: &DatasetInfo,
    statistic: CollapseStatistic,
    variables: &[String],
    groups: &[String],
  ) -> Result<DatasetInfo, ()> {
    let statistic_name = collapse_statistic_name(statistic);
    let group_sql = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>()
      .join(", ");
    let aggregate_sql = variables
      .iter()
      .map(|variable| {
        let output_name = format!("{statistic_name}_{variable}");
        format!(
          "{statistic_name}({}) AS {}",
          quote_identifier(variable),
          quote_identifier(&output_name)
        )
      })
      .collect::<Vec<_>>()
      .join(", ");
    let order_sql = groups
      .iter()
      .map(|group| format!("{} ASC NULLS LAST", quote_identifier(group)))
      .collect::<Vec<_>>()
      .join(", ");

    self.drop_staging();
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {group_sql}, {aggregate_sql} FROM {ACTIVE_TABLE} GROUP BY {group_sql} ORDER BY {order_sql}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let expected_names = groups
      .iter()
      .cloned()
      .chain(
        variables
          .iter()
          .map(|variable| format!("{statistic_name}_{variable}")),
      )
      .collect::<Vec<_>>();
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(());
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn rename_column(
    &mut self,
    dataset: &DatasetInfo,
    old_name: &str,
    new_name: &str,
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let select_list = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == old_name {
          format!(
            "{} AS {}",
            quote_identifier(&column.name),
            quote_identifier(new_name)
          )
        } else {
          quote_identifier(&column.name)
        }
      })
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let expected_names = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == old_name {
          new_name.to_owned()
        } else {
          column.name.clone()
        }
      })
      .collect::<Vec<_>>();
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(());
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn sort_rows(&mut self, dataset: &DatasetInfo, variables: &[String]) -> Result<DatasetInfo, ()> {
    let directions = vec![false; variables.len()];
    self.sort_rows_directed(dataset, variables, &directions)
  }

  fn sort_rows_directed(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    directions: &[bool],
  ) -> Result<DatasetInfo, ()> {
    if variables.len() != directions.len() {
      return Err(());
    }
    self.drop_staging();
    let mut ordinal_name = "__tabdat_sort_ordinal".to_owned();
    while dataset
      .columns
      .iter()
      .any(|column| column.name.eq_ignore_ascii_case(&ordinal_name))
    {
      ordinal_name.push('_');
    }
    let quoted_ordinal = quote_identifier(&ordinal_name);
    let order_by = variables
      .iter()
      .zip(directions)
      .map(|(variable, is_descending)| {
        let direction = if *is_descending { "DESC" } else { "ASC" };
        format!("{} {direction} NULLS LAST", quote_identifier(variable))
      })
      .chain(std::iter::once(format!("{quoted_ordinal} ASC")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT * EXCLUDE ({quoted_ordinal}) FROM (SELECT row_number() OVER () AS {quoted_ordinal}, * FROM {ACTIVE_TABLE}) AS __tabdat_sort_rows ORDER BY {order_by}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if columns != dataset.columns {
      self.drop_staging();
      return Err(());
    }
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if row_count != dataset.row_count {
      self.drop_staging();
      return Err(());
    }
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn distinct_nonmissing_string_values(&self, source: &str) -> Result<Vec<String>, ()> {
    let quoted_source = quote_identifier(source);
    let mut statement = self
      .connection
      .prepare(&format!(
        "SELECT DISTINCT CAST({quoted_source} AS VARCHAR) AS value FROM {ACTIVE_TABLE} WHERE {quoted_source} IS NOT NULL ORDER BY value"
      ))
      .map_err(|_| ())?;
    let rows = statement
      .query_map([], |row| row.get::<_, String>(0))
      .map_err(|_| ())?;
    rows.map(|row| row.map_err(|_| ())).collect()
  }

  fn grouped_summarize(
    &self,
    groups: &[String],
    variables: &[String],
  ) -> Result<Vec<Vec<CellValue>>, ()> {
    if groups.is_empty() || variables.is_empty() {
      return Err(());
    }
    let group_columns = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>();
    let aggregate_columns = variables
      .iter()
      .map(|variable| {
        let quoted_variable = quote_identifier(variable);
        format!(
          "avg({quoted_variable}) AS {}",
          quote_identifier(&format!("mean_{variable}"))
        )
      })
      .collect::<Vec<_>>();
    let order_columns = group_columns
      .iter()
      .map(|group| format!("{group} ASC NULLS LAST"))
      .collect::<Vec<_>>();
    let query = format!(
      "SELECT {}, {} FROM {ACTIVE_TABLE} GROUP BY {} ORDER BY {}",
      group_columns.join(", "),
      aggregate_columns.join(", "),
      group_columns.join(", "),
      order_columns.join(", ")
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let mut grouped = Vec::new();
    while let Some(row) = rows.next().map_err(|_| ())? {
      let mut values = Vec::with_capacity(groups.len() + variables.len());
      for index in 0..(groups.len() + variables.len()) {
        values.push(cell_value_from_ref(row.get_ref(index).map_err(|_| ())?)?);
      }
      grouped.push(values);
    }
    Ok(grouped)
  }

  fn grouped_count(&self, groups: &[String]) -> Result<Vec<Vec<CellValue>>, ()> {
    if groups.is_empty() {
      return Err(());
    }
    let group_columns = groups
      .iter()
      .map(|group| quote_identifier(group))
      .collect::<Vec<_>>();
    let order_columns = group_columns
      .iter()
      .map(|group| format!("{group} ASC NULLS LAST"))
      .collect::<Vec<_>>();
    let query = format!(
      "SELECT {}, COUNT(*) FROM {ACTIVE_TABLE} GROUP BY {} ORDER BY {}",
      group_columns.join(", "),
      group_columns.join(", "),
      order_columns.join(", ")
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let mut grouped = Vec::new();
    while let Some(row) = rows.next().map_err(|_| ())? {
      let mut values = Vec::with_capacity(groups.len() + 1);
      for index in 0..(groups.len() + 1) {
        values.push(cell_value_from_ref(row.get_ref(index).map_err(|_| ())?)?);
      }
      grouped.push(values);
    }
    Ok(grouped)
  }

  fn tabulate_counts(
    &self,
    row_variable: &str,
    column_variable: Option<&str>,
    include_missing: bool,
  ) -> Result<Vec<TabulateCount>, ()> {
    let quoted_row = quote_identifier(row_variable);
    let quoted_column = column_variable.map(quote_identifier);
    let select_columns = quoted_column.as_ref().map_or_else(
      || quoted_row.clone(),
      |column| format!("{quoted_row}, {column}"),
    );
    let group_columns = select_columns.clone();
    let order_columns = quoted_column.as_ref().map_or_else(
      || format!("{quoted_row} ASC NULLS LAST"),
      |column| format!("{quoted_row} ASC NULLS LAST, {column} ASC NULLS LAST"),
    );
    let where_sql = if include_missing {
      String::new()
    } else {
      let mut predicates = vec![format!("{quoted_row} IS NOT NULL")];
      if let Some(column) = quoted_column.as_ref() {
        predicates.push(format!("{column} IS NOT NULL"));
      }
      format!(" WHERE {}", predicates.join(" AND "))
    };
    let query = format!(
      "SELECT {select_columns}, COUNT(*) FROM {ACTIVE_TABLE}{where_sql} GROUP BY {group_columns} ORDER BY {order_columns}"
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let mut counts = Vec::new();
    while let Some(row) = rows.next().map_err(|_| ())? {
      let row_value = cell_value_from_ref(row.get_ref(0).map_err(|_| ())?)?;
      let column = if quoted_column.is_some() {
        Some(cell_value_from_ref(row.get_ref(1).map_err(|_| ())?)?)
      } else {
        None
      };
      let count = u64::try_from(
        row
          .get::<_, i64>(if quoted_column.is_some() { 2 } else { 1 })
          .map_err(|_| ())?,
      )
      .map_err(|_| ())?;
      counts.push(TabulateCount {
        row: row_value,
        column,
        count,
      });
    }
    Ok(counts)
  }

  fn encode_column(
    &mut self,
    dataset: &DatasetInfo,
    source: &str,
    target: &str,
    mapping: &[(String, i64)],
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let quoted_source = quote_identifier(source);
    let quoted_target = quote_identifier(target);
    let case_sql = if mapping.is_empty() {
      "CAST(NULL AS BIGINT)".to_owned()
    } else {
      let whens = mapping
        .iter()
        .map(|(value, code)| {
          format!(
            "WHEN {quoted_source} = {} THEN {code}",
            quote_recode_literal(value)
          )
        })
        .collect::<Vec<_>>()
        .join(" ");
      format!("CASE {whens} ELSE NULL END")
    };
    let select_items = dataset
      .columns
      .iter()
      .map(|column| quote_identifier(&column.name))
      .chain(std::iter::once(format!("{case_sql} AS {quoted_target}")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_items} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let mut expected_names = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    expected_names.push(target.to_owned());
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(());
    }

    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if row_count != dataset.row_count || self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn decode_column(
    &mut self,
    dataset: &DatasetInfo,
    source: &str,
    target: &str,
    mapping: &[(i64, String)],
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let quoted_source = quote_identifier(source);
    let quoted_target = quote_identifier(target);
    let case_sql = if mapping.is_empty() {
      "CAST(NULL AS VARCHAR)".to_owned()
    } else {
      let whens = mapping
        .iter()
        .map(|(code, value)| {
          format!(
            "WHEN {quoted_source} = {code} THEN {}",
            quote_recode_literal(value)
          )
        })
        .collect::<Vec<_>>()
        .join(" ");
      format!("CASE {whens} ELSE NULL END")
    };
    let select_items = dataset
      .columns
      .iter()
      .map(|column| quote_identifier(&column.name))
      .chain(std::iter::once(format!("{case_sql} AS {quoted_target}")))
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_items} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let mut expected_names = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    expected_names.push(target.to_owned());
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(());
    }

    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if row_count != dataset.row_count || self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn recode_columns(
    &mut self,
    dataset: &DatasetInfo,
    variables: &[String],
    rules: &[RecodeRule],
    target: &RecodeTarget,
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();

    let generated = match target {
      RecodeTarget::Generate { variables } => Some(variables),
      RecodeTarget::Replace => None,
    };
    let targets = generated.cloned().unwrap_or_else(|| variables.to_vec());
    let expressions = variables
      .iter()
      .zip(targets.iter())
      .map(|(source, target)| {
        let source_column = dataset
          .columns
          .iter()
          .find(|column| column.name == *source)
          .ok_or(())?;
        let expression = compile_recode_case(source_column, rules)?;
        Ok::<_, ()>((source.clone(), target.clone(), expression))
      })
      .collect::<Result<Vec<_>, _>>()?;

    let mut select_items = dataset
      .columns
      .iter()
      .map(|column| {
        let expression = expressions
          .iter()
          .find(|(source, _, _)| source == &column.name)
          .map(|(_, _, expression)| expression);
        match (target, expression) {
          (RecodeTarget::Replace, Some(expression)) => {
            format!("{expression} AS {}", quote_identifier(&column.name))
          }
          _ => quote_identifier(&column.name),
        }
      })
      .collect::<Vec<_>>();
    if let Some(generated) = generated {
      select_items.extend(expressions.iter().map(|(_, target, expression)| {
        debug_assert!(generated.iter().any(|variable| variable == target));
        format!("{expression} AS {}", quote_identifier(target))
      }));
    }

    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {} FROM {ACTIVE_TABLE}",
        select_items.join(", ")
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let mut expected_names = dataset
      .columns
      .iter()
      .map(|column| column.name.clone())
      .collect::<Vec<_>>();
    if let Some(generated) = generated {
      expected_names.extend(generated.iter().cloned());
    }
    if columns
      .iter()
      .map(|column| column.name.as_str())
      .ne(expected_names.iter().map(String::as_str))
    {
      self.drop_staging();
      return Err(());
    }

    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if row_count != dataset.row_count || self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn generate_column(
    &mut self,
    dataset: &DatasetInfo,
    variable: &str,
    expression: &AssertExpression,
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let expression_sql = compile_assert_expression(expression, dataset)?;
    let target_sql = quote_identifier(variable);
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT *, {expression_sql} AS {target_sql} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
    })
  }

  fn replace_column(
    &mut self,
    dataset: &DatasetInfo,
    variable: &str,
    target_type: &str,
    expression: &AssertExpression,
    condition: Option<&AssertExpression>,
  ) -> Result<DatasetInfo, ()> {
    self.drop_staging();
    let expression_sql = if matches!(expression, AssertExpression::Null) {
      format!("CAST(NULL AS {target_type})")
    } else {
      compile_assert_expression(expression, dataset)?
    };
    let replacement_sql = if let Some(condition) = condition {
      let condition_sql = compile_assert_expression(condition, dataset)?;
      format!(
        "CASE WHEN {condition_sql} THEN {expression_sql} ELSE {} END",
        quote_identifier(variable)
      )
    } else {
      expression_sql
    };
    let select_list = dataset
      .columns
      .iter()
      .map(|column| {
        if column.name == variable {
          format!("{replacement_sql} AS {}", quote_identifier(&column.name))
        } else {
          quote_identifier(&column.name)
        }
      })
      .collect::<Vec<_>>()
      .join(", ");
    if self
      .connection
      .execute_batch(&format!(
        "CREATE TEMP TABLE {STAGING_TABLE} AS SELECT {select_list} FROM {ACTIVE_TABLE}"
      ))
      .is_err()
    {
      self.drop_staging();
      return Err(());
    }

    let columns = match self.staged_columns() {
      Ok(columns) => columns,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    let row_count = match self.staged_row_count() {
      Ok(row_count) => row_count,
      Err(()) => {
        self.drop_staging();
        return Err(());
      }
    };
    if self.publish_staging().is_err() {
      self.drop_staging();
      return Err(());
    }

    Ok(DatasetInfo {
      source: dataset.source.clone(),
      row_count,
      columns,
      execution_mode: dataset.execution_mode,
      lazy_engine: dataset.lazy_engine,
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

  fn assert_rows(
    &self,
    expression: &AssertExpression,
    dataset: &DatasetInfo,
  ) -> Result<(u64, u64), ()> {
    let predicate = compile_assert_expression(expression, dataset)?;
    let query = format!(
      "SELECT COUNT(*), COUNT(*) FILTER (WHERE ({predicate}) IS NULL OR NOT ({predicate})) FROM {ACTIVE_TABLE}"
    );
    let mut statement = self.connection.prepare(&query).map_err(|_| ())?;
    let mut rows = statement.query([]).map_err(|_| ())?;
    let row = rows.next().map_err(|_| ())?.ok_or(())?;
    let checked = u64::try_from(row.get::<_, i64>(0).map_err(|_| ())?).map_err(|_| ())?;
    let failed = u64::try_from(row.get::<_, i64>(1).map_err(|_| ())?).map_err(|_| ())?;
    Ok((checked, failed))
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
  fn assert_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Assert {
          expression: AssertExpression::Binary {
            left: Box::new(AssertExpression::Identifier("age".to_owned())),
            operator: AssertBinaryOperator::Greater,
            right: Box::new(AssertExpression::Number("0".to_owned())),
          },
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "assert" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn keep_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Keep {
          variables: vec!["age".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "keep" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn select_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Select {
          variables: vec!["age".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "select" }
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
  fn replace_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Replace {
          variable: "age".to_owned(),
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("age".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Number("1".to_owned())),
          },
          condition: None,
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "replace" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn rename_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Rename {
          old_name: "old_name".to_owned(),
          new_name: "new_name".to_owned(),
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "rename" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn sort_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Sort {
          variables: vec!["value".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "sort" }
    );
    assert!(session.backend.is_none());
    assert!(session.active_dataset.is_none());
  }

  #[test]
  fn gsort_does_not_initialize_backend_for_a_new_session() {
    let mut session = Session::new();

    assert_eq!(
      session
        .execute(Command::Gsort {
          keys: vec![SortKey {
            variable: "value".to_owned(),
            descending: true,
          }],
        })
        .unwrap_err(),
      RuntimeError::NoActiveDataset { command: "gsort" }
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
  fn failed_keep_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![ColumnInfo {
        name: "age".to_owned(),
        data_type: "INTEGER".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    session.active_dataset = Some(dataset.clone());

    assert_eq!(
      session
        .execute(Command::Keep {
          variables: vec!["age".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::KeepFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    assert!(session.backend.is_some());
  }

  #[test]
  fn failed_drop_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![
        ColumnInfo {
          name: "age".to_owned(),
          data_type: "INTEGER".to_owned(),
        },
        ColumnInfo {
          name: "sex".to_owned(),
          data_type: "VARCHAR".to_owned(),
        },
      ],
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS age"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Drop {
          variables: vec!["age".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::DropFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let age: i64 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT age FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the pre-existing active relation should remain available");
    assert_eq!(age, 7);
  }

  #[test]
  fn failed_select_keeps_the_published_dataset_metadata() {
    let mut session = Session::new();
    session.backend = Some(DuckDbBackend::new().expect("test backend should initialize"));
    let dataset = DatasetInfo {
      source: PathBuf::from("fixture.parquet"),
      row_count: 1,
      columns: vec![
        ColumnInfo {
          name: "age".to_owned(),
          data_type: "INTEGER".to_owned(),
        },
        ColumnInfo {
          name: "sex".to_owned(),
          data_type: "VARCHAR".to_owned(),
        },
      ],
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS age"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Select {
          variables: vec!["sex".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::SelectFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let age: i64 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT age FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the pre-existing active relation should remain available");
    assert_eq!(age, 7);
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
  fn failed_generate_keeps_metadata_and_private_active_relation() {
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Generate {
          variable: "value2".to_owned(),
          expression: GenerateExpression::Identifier("value".to_owned()),
        })
        .unwrap_err(),
      RuntimeError::GenerateFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let value: i32 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT other FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
  }

  #[test]
  fn failed_replace_keeps_metadata_and_private_active_relation() {
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Replace {
          variable: "value".to_owned(),
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("value".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Number("1".to_owned())),
          },
          condition: None,
        })
        .unwrap_err(),
      RuntimeError::ReplaceFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let value: i32 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT other FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
  }

  #[test]
  fn failed_rename_keeps_metadata_and_private_active_relation() {
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Rename {
          old_name: "value".to_owned(),
          new_name: "renamed".to_owned(),
        })
        .unwrap_err(),
      RuntimeError::RenameFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let value: i32 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT other FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
  }

  #[test]
  fn failed_sort_keeps_metadata_and_private_active_relation() {
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Sort {
          variables: vec!["value".to_owned()],
        })
        .unwrap_err(),
      RuntimeError::SortFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let value: i32 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT other FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
  }

  #[test]
  fn failed_gsort_keeps_metadata_and_private_active_relation() {
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
        "CREATE TEMP TABLE {ACTIVE_TABLE} AS SELECT 7 AS other"
      ))
      .expect("the mismatched active relation should be created");

    assert_eq!(
      session
        .execute(Command::Gsort {
          keys: vec![SortKey {
            variable: "value".to_owned(),
            descending: true,
          }],
        })
        .unwrap_err(),
      RuntimeError::GsortFailed
    );
    assert_eq!(session.active_dataset.as_ref(), Some(&dataset));
    let value: i32 = session
      .backend
      .as_ref()
      .expect("test backend should exist")
      .connection
      .query_row(&format!("SELECT other FROM {ACTIVE_TABLE}"), [], |row| {
        row.get(0)
      })
      .expect("the prior active relation should remain queryable");
    assert_eq!(value, 7);
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
