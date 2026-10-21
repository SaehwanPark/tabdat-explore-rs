use crate::LabelValue;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

/// Source category of a located script failure, independent of diagnostic wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptErrorKind {
  /// Command execution, recursion, or caller preflight failed.
  Execution,
  /// Script syntax, directives, macro expansion, or control structure is invalid.
  Parse,
  /// Script text could not be loaded, or a command reported a missing data file.
  File,
}

/// Recoverable script parsing or execution error containing source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
  path: PathBuf,
  line: usize,
  message: String,
  kind: ScriptErrorKind,
}

impl ScriptError {
  /// Construct an execution/preflight error with a path and 1-based line number.
  /// Parser and file-loading producers should use `parse` and `file` instead.
  pub fn new(path: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
    Self {
      path: path.into(),
      line,
      message: message.into(),
      kind: ScriptErrorKind::Execution,
    }
  }

  /// Construct an invalid script syntax/directive error without inspecting its message.
  pub fn parse(path: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
    Self {
      kind: ScriptErrorKind::Parse,
      ..Self::new(path, line, message)
    }
  }

  /// Construct a script-loading or missing-data-file error.
  pub fn file(path: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
    Self {
      kind: ScriptErrorKind::File,
      ..Self::new(path, line, message)
    }
  }

  /// Failure source category; presentation text is not a classification protocol.
  pub fn kind(&self) -> ScriptErrorKind {
    self.kind
  }

  /// The path to the script file where the error occurred.
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// The 1-based line number of the error.
  pub fn line(&self) -> usize {
    self.line
  }

  /// The diagnostic error message.
  pub fn message(&self) -> &str {
    &self.message
  }
}

impl fmt::Display for ScriptError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}: {}", self.path.display(), self.line, self.message)
  }
}

impl std::error::Error for ScriptError {}

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
  /// The source path is not a local Parquet or CSV file.
  UnsupportedFormat { path: PathBuf },
  /// DuckDB could not read the staged Parquet relation.
  ParquetRead { path: PathBuf },
  /// DuckDB could not inspect the staged schema.
  SchemaRead { path: PathBuf },
  /// DuckDB returned an invalid row count.
  RowCount { path: PathBuf },
  /// DuckDB could not read the staged CSV relation.
  CsvRead { path: PathBuf },
  /// DuckDB could not inspect the staged CSV schema.
  CsvSchemaRead { path: PathBuf },
  /// DuckDB returned an invalid CSV row count.
  CsvRowCount { path: PathBuf },
  /// DuckDB could not atomically publish the staged CSV relation.
  CsvTransaction { path: PathBuf },
  /// DuckDB could not atomically publish the staged relation.
  Transaction { path: PathBuf },
  /// DuckDB could not initialize its in-memory connection.
  BackendInitialization,
  /// The save destination is not a Parquet path.
  SaveUnsupportedFormat { path: PathBuf },
  /// The save destination already exists and replacement was not requested.
  SaveTargetExists { path: PathBuf },
  /// The save destination exists but is not a regular file.
  SaveTargetNotAFile { path: PathBuf },
  /// The save destination's parent directory could not be created.
  SaveParentFailed { path: PathBuf },
  /// DuckDB could not write the active relation to the save destination.
  SaveFailed { path: PathBuf },
  /// The export destination is not a CSV path.
  ExportUnsupportedFormat { path: PathBuf },
  /// The export destination already exists and replacement was not requested.
  ExportTargetExists { path: PathBuf },
  /// The export destination exists but is not a regular file.
  ExportTargetNotAFile { path: PathBuf },
  /// The export destination's parent directory could not be created.
  ExportParentFailed { path: PathBuf },
  /// DuckDB could not write the active relation to the export destination.
  ExportFailed { path: PathBuf },
  /// A named report request failed without changing registry or active state.
  EstimatesFailed { message: String },
  /// The bounded scatter request or selected numeric data could not be rendered.
  ScatterFailed { message: String },
  /// A complete SVG could not be published without replacing a destination.
  PlotWriteFailed { path: PathBuf, message: String },
  /// DuckDB could not produce the requested preview.
  PreviewFailed { command: &'static str },
  /// The count predicate names variables absent from the active schema.
  CountUnknownVariable { variables: Vec<String> },
  /// Conditional count requires a boolean/null predicate.
  CountRequiresBoolean,
  /// The count predicate has incompatible operand domains.
  CountPredicateTypeMismatch { message: String },
  /// Parsed function predicates are outside bounded count execution.
  CountUnsupportedPredicate,
  /// DuckDB could not count the selected rows.
  CountFailed,
  /// The summary request named variables absent from the active schema.
  SummaryUnknownVariable { variables: Vec<String> },
  /// The summary request named variables that are not numeric.
  SummaryRequiresNumeric { variables: Vec<String> },
  /// The active schema contains no numeric columns for a default summary.
  SummaryNoNumericColumns,
  /// Conditional summary requires a boolean/null predicate.
  SummaryRequiresBoolean,
  /// The summary predicate has incompatible operand domains.
  SummaryPredicateTypeMismatch { message: String },
  /// Parsed function predicates are outside bounded summary execution.
  SummaryUnsupportedPredicate,
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
  /// Conditional row selection named absent variables.
  RowFilterUnknownVariable { variables: Vec<String> },
  /// Conditional row selection has incompatible operand domains.
  RowFilterTypeMismatch { message: String },
  /// Row selection requires a boolean or null predicate, not numeric truthiness.
  RowFilterRequiresBoolean,
  /// Parsed function calls are outside bounded row selection.
  RowFilterUnsupportedExpression,
  /// DuckDB could not stage or publish the selected rows.
  RowFilterFailed,
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
  /// The frequency predicate did not evaluate to a boolean or null domain.
  TabulateRequiresBoolean,
  /// The frequency predicate has incompatible operand domains.
  TabulatePredicateTypeMismatch { message: String },
  /// The frequency predicate is outside the bounded expression subset.
  TabulateUnsupportedPredicate,
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
  /// A script parsing or execution error with source file and line diagnostics.
  ScriptError(ScriptError),
  /// The SQL query does not begin with SELECT or WITH.
  SqlNotSelectOrWith,
  /// DuckDB could not execute the SQL query.
  SqlFailed,
  /// The SQL query did not produce any result columns.
  SqlNoTableResult,
  /// Options were provided when activating a registered named table.
  UseOptionsNotSupportedForNamedTable,
  /// The requested named table does not exist in the session registry.
  UnknownTable { name: String },
  /// One or more join keys were not found in the active dataset.
  JoinUnknownVariable { variables: Vec<String> },
  /// One or more join keys were not found in the right named table.
  JoinUnknownVariableInTable {
    table_name: String,
    variables: Vec<String>,
  },
  /// DuckDB could not stage or publish the joined relation.
  JoinFailed,
  /// The table to append contains variables not in the active dataset.
  AppendUnknownVariable { variables: Vec<String> },
  /// The table to append is missing variables present in the active dataset.
  AppendUnknownVariableInTable {
    table_name: String,
    variables: Vec<String>,
  },
  /// An appended variable has a data type incompatible with the active variable.
  AppendTypeMismatch {
    variable: String,
    left_type: String,
    right_type: String,
  },
  /// DuckDB could not stage or publish the appended relation.
  AppendFailed,
  /// One or more variables or identifiers for reshape were not found in the active dataset.
  ReshapeUnknownVariable { variables: Vec<String> },
  /// The long reshape output j-variable already exists in the active dataset.
  ReshapeOutputColumnExists { variable: String },
  /// No columns matched the given stub prefix in a long reshape.
  ReshapeLongFoundNoColumnsForStub { stub: String },
  /// A required column for a discovered j value was missing from a stub in a long reshape.
  ReshapeLongMissingColumn { column: String },
  /// No non-null j values were found for a wide reshape.
  ReshapeWideFoundNoJValues,
  /// A generated wide output column name already exists in non-participating columns.
  ReshapeWideOutputColumnExists { variable: String },
  /// DuckDB could not stage or publish the reshaped relation.
  ReshapeFailed,
  /// One or more variables or identifiers for regress were not found in the active dataset.
  RegressUnknownVariable { variables: Vec<String> },
  /// A conditional fit requires a boolean/null predicate.
  RegressRequiresBoolean,
  /// The fit predicate has incompatible operand domains.
  RegressPredicateTypeMismatch { message: String },
  /// Function calls are outside bounded fit predicate execution.
  RegressUnsupportedPredicate,
  /// The regress request named variables that are not numeric.
  RegressRequiresNumeric { variables: Vec<String> },
  /// The regress sample contained no complete observations.
  RegressNoObservations,
  /// The regress request required positive weight values.
  RegressRequiresPositiveWeights,
  /// The regress request required positive sigma values.
  RegressRequiresPositiveSigma,
  /// The regress estimation failed.
  RegressFailed { message: String },
  /// Invalid one-way within specification, panel keys or numerical fit.
  FixedEffectsFailed { message: String },
  /// Invalid bounded random-effects specification, sample or numerical fit.
  RandomEffectsFailed { message: String },
  /// Unsupported/current-data random-effects prediction.
  RandomEffectsPredictionFailed { message: String },
  /// Unsupported/current-data prediction after within FE, without changing fit.
  FixedEffectsPredictionFailed { message: String },
  /// Invalid/unsupported binary logit specification or a failed finite-MLE fit.
  LogitFailed { message: String },
  /// Invalid/unsupported conditional-quantile request or failed numerical/IID fit.
  QregFailed { message: String },
  /// Continuous logit marginal effects require an owned preceding logit fit.
  EstatMarginsRequiresPriorLogit,
  /// A retained-design delta calculation failed.
  EstatMarginsFailed { message: String },
  /// Count frequency reports require an owned fitted Poisson or NB2 model.
  EstatCountProbRequiresPriorCount,
  /// A bounded count distribution report failed without changing state.
  EstatCountProbFailed { message: String },
  /// Only explicit pr/xb predictions without auxiliary interval options are supported.
  PredictLogitUnsupported,
  /// Invalid/unsupported binary probit specification or a failed finite-MLE fit.
  ProbitFailed { message: String },
  /// Only explicit pr/xb probit predictions without auxiliary interval options are supported.
  PredictBinaryUnsupported,
  /// Invalid/unsupported Poisson request or failed finite numerical fit.
  PoissonFailed { message: String },
  /// Only n/xb without auxiliary options are installed for Poisson.
  PredictPoissonUnsupported,
  /// Existing error identity retained; n now accepts either installed count fit.
  PredictMeanCountRequiresPoisson,
  /// Invalid/unsupported NB2 request or failed joint numerical fit.
  NbregFailed { message: String },
  /// Only n/xb without auxiliary options are installed for NB2.
  PredictNbregUnsupported,
  /// The requested IV estimator is not executed by this runtime slice.
  IvRegressUnsupportedEstimator { estimator: String },
  /// One or more IV variables are absent from the active dataset.
  IvRegressUnknownVariable { variables: Vec<String> },
  /// One or more IV regressors, instruments, or the outcome are not numeric.
  IvRegressRequiresNumeric { variables: Vec<String> },
  /// HC1 and cluster covariance were requested together.
  IvRegressConflictingCovariance,
  /// The IV model has no complete finite observations.
  IvRegressNoObservations,
  /// The IV estimation failed.
  IvRegressFailed { message: String },
  /// VIF requires a preceding supported linear regression.
  EstatRequiresPriorRegression,
  /// DuckDB-independent auxiliary fits for VIF could not be computed.
  EstatVifFailed,
  /// AR requires a retained IV design from a successful 2SLS fit.
  EstatArRequiresPriorIv,
  /// The joint AR null, covariance mode or auxiliary regression is unsupported/invalid.
  EstatArFailed(String),
  /// Invalid bootstrap request or failed original/replicate fit.
  BootstrapFailed(String),
  /// Sargan requires structural residuals and design from a retained 2SLS fit.
  EstatOveridRequiresPriorIv,
  /// Covariance, identification or numerical limits invalidate Sargan inference.
  EstatOveridFailed(String),
  /// Wu-Hausman requires the design/sample of a retained 2SLS fit.
  EstatEndogenousRequiresPriorIv,
  /// Unsupported covariance or an undefined auxiliary endogeneity test.
  EstatEndogenousFailed(String),
  /// `lincom` requires a preceding supported linear regression.
  LincomRequiresPriorEstimation,
  /// An expression or inference error while evaluating `lincom`.
  LincomFailed { message: String },
  /// `test` requires a preceding supported linear regression.
  TestRequiresPriorEstimation,
  /// An expression or inference error while evaluating `test`.
  TestFailed { message: String },
  /// One or more requested `ttest` variables are absent from the active dataset.
  TtestUnknownVariable { variables: Vec<String> },
  /// The tested `ttest` variables are not numeric.
  TtestRequiresNumeric { variables: Vec<String> },
  /// A `ttest` sample contains no finite numeric observations.
  TtestNoValidNumericObservations,
  /// A paired `ttest` contains no complete finite pairs.
  TtestNoValidPairedNumericObservations,
  /// The filtered `by()` variable contains other than exactly two groups.
  TtestInvalidGroupCount { found: usize },
  /// The group key type cannot be stringified with the pinned oracle behavior.
  TtestUnsupportedGroupType { variable: String, data_type: String },
  /// A query, sample conversion, or statistical calculation failed.
  TtestFailed { message: String },
  /// Prediction requires a preceding supported linear regression.
  PredictRequiresPriorRegression,
  /// The prediction target already exists in the active schema.
  PredictTargetExists { variable: String },
  /// One or more required model variables are absent from the active schema.
  PredictUnknownVariable { variables: Vec<String> },
  /// DuckDB could not stage or publish fitted values or residuals.
  PredictFailed,
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
        formatter.write_str("use runtime slice supports only eager local Parquet or CSV loads")
      }
      Self::NonUtf8Path { path } => {
        write!(formatter, "use path is not valid UTF-8: {}", path.display())
      }
      Self::FileNotFound { path } => {
        write!(formatter, "use could not find file: {}", path.display())
      }
      Self::NotAFile { path } => write!(formatter, "use expected a file path: {}", path.display()),
      Self::UnsupportedFormat { path: _ } => {
        formatter.write_str("use runtime slice supports only local .parquet or .csv files")
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
      Self::CsvRead { path } => {
        write!(formatter, "use could not read CSV file: {}", path.display())
      }
      Self::CsvSchemaRead { path } => {
        write!(
          formatter,
          "use could not inspect CSV schema: {}",
          path.display()
        )
      }
      Self::CsvRowCount { path } => {
        write!(
          formatter,
          "use could not count CSV rows: {}",
          path.display()
        )
      }
      Self::Transaction { path } => write!(
        formatter,
        "use could not publish Parquet relation: {}",
        path.display()
      ),
      Self::CsvTransaction { path } => {
        write!(
          formatter,
          "use could not publish CSV relation: {}",
          path.display()
        )
      }
      Self::BackendInitialization => formatter.write_str("use could not initialize DuckDB"),
      Self::SaveUnsupportedFormat { path } => write!(
        formatter,
        "save only supports .parquet files: {}",
        path.display()
      ),
      Self::SaveTargetExists { path } => {
        write!(formatter, "save target already exists: {}", path.display())
      }
      Self::SaveTargetNotAFile { path } => {
        write!(formatter, "save target is not a file: {}", path.display())
      }
      Self::SaveParentFailed { path } => write!(
        formatter,
        "save could not create parent directories: {}",
        path.display()
      ),
      Self::SaveFailed { path } => write!(formatter, "save failed: {}", path.display()),
      Self::ExportUnsupportedFormat { path } => write!(
        formatter,
        "export only supports .csv files: {}",
        path.display()
      ),
      Self::ExportTargetExists { path } => {
        write!(
          formatter,
          "export target already exists: {}",
          path.display()
        )
      }
      Self::ExportTargetNotAFile { path } => {
        write!(formatter, "export target is not a file: {}", path.display())
      }
      Self::ExportParentFailed { path } => write!(
        formatter,
        "export could not create parent directories: {}",
        path.display()
      ),
      Self::ExportFailed { path } => write!(formatter, "export failed: {}", path.display()),
      Self::EstimatesFailed { message } => write!(formatter, "estimates: {message}"),
      Self::ScatterFailed { message } => write!(formatter, "scatter: {message}"),
      Self::PlotWriteFailed { path, message } => write!(
        formatter,
        "scatter could not publish {}: {message}",
        path.display()
      ),
      Self::PreviewFailed { command } => write!(formatter, "{command} failed"),
      Self::CountUnknownVariable { variables } => write!(
        formatter,
        "count unknown variable: {}",
        variables.join(", ")
      ),
      Self::CountRequiresBoolean => {
        formatter.write_str("count predicate requires boolean expression")
      }
      Self::CountPredicateTypeMismatch { message } => formatter.write_str(message),
      Self::CountUnsupportedPredicate => {
        formatter.write_str("count predicate does not support function calls")
      }
      Self::CountFailed => formatter.write_str("count failed"),
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
      Self::SummaryRequiresBoolean => {
        formatter.write_str("summarize predicate requires boolean expression")
      }
      Self::SummaryPredicateTypeMismatch { message } => formatter.write_str(message),
      Self::SummaryUnsupportedPredicate => {
        formatter.write_str("summarize predicate does not support function calls")
      }
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
      Self::RowFilterUnknownVariable { variables } => write!(
        formatter,
        "row filter unknown variable: {}",
        variables.join(", ")
      ),
      Self::RowFilterTypeMismatch { message } => formatter.write_str(message),
      Self::RowFilterRequiresBoolean => {
        formatter.write_str("row filter requires boolean expression")
      }
      Self::RowFilterUnsupportedExpression => {
        formatter.write_str("row filter does not support function calls")
      }
      Self::RowFilterFailed => formatter.write_str("row filter failed"),
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
      Self::TabulateRequiresBoolean => {
        formatter.write_str("tabulate predicate requires boolean expression")
      }
      Self::TabulatePredicateTypeMismatch { message } => formatter.write_str(message),
      Self::TabulateUnsupportedPredicate => {
        formatter.write_str("tabulate predicate does not support function calls")
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
      Self::ScriptError(error) => write!(formatter, "{error}"),
      Self::SqlNotSelectOrWith => {
        formatter.write_str("sql only supports select or with queries in Phase 4")
      }
      Self::SqlFailed => formatter.write_str("sql failed"),
      Self::SqlNoTableResult => formatter.write_str("sql must produce a table result"),
      Self::UseOptionsNotSupportedForNamedTable => {
        formatter.write_str("use options are not supported for named table activation")
      }
      Self::UnknownTable { name } => write!(formatter, "unknown table: {name}"),
      Self::JoinUnknownVariable { variables } => {
        write!(formatter, "join unknown variable: {}", variables.join(", "))
      }
      Self::JoinUnknownVariableInTable {
        table_name,
        variables,
      } => {
        write!(
          formatter,
          "join unknown variable in {table_name}: {}",
          variables.join(", ")
        )
      }
      Self::JoinFailed => formatter.write_str("join failed"),
      Self::AppendUnknownVariable { variables } => {
        write!(
          formatter,
          "append unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::AppendUnknownVariableInTable {
        table_name,
        variables,
      } => {
        write!(
          formatter,
          "append unknown variable in {table_name}: {}",
          variables.join(", ")
        )
      }
      Self::AppendTypeMismatch {
        variable,
        left_type,
        right_type,
      } => {
        write!(
          formatter,
          "append type mismatch for {variable}: {left_type} vs {right_type}"
        )
      }
      Self::AppendFailed => formatter.write_str("append failed"),
      Self::ReshapeUnknownVariable { variables } => {
        write!(
          formatter,
          "reshape unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::ReshapeOutputColumnExists { variable } => {
        write!(
          formatter,
          "reshape output column already exists: {variable}"
        )
      }
      Self::ReshapeLongFoundNoColumnsForStub { stub } => {
        write!(formatter, "reshape long found no columns for stub: {stub}")
      }
      Self::ReshapeLongMissingColumn { column } => {
        write!(formatter, "reshape long missing column: {column}")
      }
      Self::ReshapeWideFoundNoJValues => formatter.write_str("reshape wide found no j values"),
      Self::ReshapeWideOutputColumnExists { variable } => {
        write!(
          formatter,
          "reshape wide output column already exists: {variable}"
        )
      }
      Self::ReshapeFailed => formatter.write_str("reshape failed"),
      Self::RegressRequiresBoolean => {
        formatter.write_str("regress predicate requires boolean expression")
      }
      Self::RegressPredicateTypeMismatch { message } => formatter.write_str(message),
      Self::RegressUnsupportedPredicate => {
        formatter.write_str("regress predicate does not support function calls")
      }
      Self::RegressUnknownVariable { variables } => {
        write!(
          formatter,
          "regress unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::RegressRequiresNumeric { variables } => {
        write!(
          formatter,
          "regress requires numeric variables: {}",
          variables.join(", ")
        )
      }
      Self::RegressNoObservations => {
        formatter.write_str("regress requires at least one complete observation")
      }
      Self::RegressRequiresPositiveWeights => {
        formatter.write_str("regress requires positive weights values")
      }
      Self::RegressRequiresPositiveSigma => {
        formatter.write_str("regress requires positive sigma values")
      }
      Self::RegressFailed { message } => write!(formatter, "regress failed: {message}"),
      Self::RandomEffectsFailed { message } => write!(formatter, "xtreg random-effects failed: {message}"),
      Self::RandomEffectsPredictionFailed { message } => write!(formatter, "predict after xtreg random-effects failed: {message}"),
      Self::FixedEffectsFailed { message } => write!(formatter, "xtreg fe failed: {message}"),
      Self::FixedEffectsPredictionFailed { message } => write!(formatter, "predict after xtreg fe failed: {message}"),
      Self::QregFailed { message } => write!(formatter, "qreg: {message}"),
      Self::LogitFailed { message } => write!(formatter, "logit: {message}"),
      Self::EstatMarginsRequiresPriorLogit => {
        formatter.write_str("estat margins requires a prior logit fit")
      }
      Self::EstatMarginsFailed { message } => write!(formatter, "estat margins: {message}"),
      Self::EstatCountProbRequiresPriorCount => formatter.write_str("estat countprob requires a prior Poisson or NB2 fit"),
      Self::EstatCountProbFailed { message } => write!(formatter, "estat countprob: {message}"),
      Self::PredictLogitUnsupported => formatter.write_str(
        "logit prediction supports only pr or xb, without interval, std or saving options",
      ),
      Self::ProbitFailed { message } => write!(formatter, "probit: {message}"),
      Self::PredictBinaryUnsupported => formatter.write_str(
        "probit prediction supports only pr or xb, without interval, std or saving options",
      ),
      Self::PoissonFailed { message } => write!(formatter, "poisson: {message}"),
      Self::PredictPoissonUnsupported => formatter.write_str(
        "poisson prediction supports only n (mean), xb, pmf(k) or cdf(k), without interval, std or saving options",
      ),
      Self::PredictMeanCountRequiresPoisson => formatter.write_str(
        "predict n requires a prior successful count fit (Poisson or NB2); use pr for binary probabilities or xb for an index",
      ),
      Self::NbregFailed { message } => write!(formatter, "nbreg: {message}"),
      Self::PredictNbregUnsupported => formatter.write_str(
        "nbreg NB2 prediction supports only n (mean), xb, pmf(k) or cdf(k), without interval, std or saving options",
      ),
      Self::IvRegressUnsupportedEstimator { estimator } => write!(
        formatter,
        "ivregress estimator '{estimator}' is not implemented; use 2sls"
      ),
      Self::IvRegressUnknownVariable { variables } => write!(
        formatter,
        "ivregress: unknown variable(s): {}",
        variables.join(", ")
      ),
      Self::IvRegressRequiresNumeric { variables } => write!(
        formatter,
        "ivregress: numeric variables required: {}",
        variables.join(", ")
      ),
      Self::IvRegressConflictingCovariance => {
        formatter.write_str("ivregress cannot combine robust and cluster")
      }
      Self::IvRegressNoObservations => {
        formatter.write_str("ivregress: no complete finite observations")
      }
      Self::IvRegressFailed { message } => write!(formatter, "ivregress failed: {message}"),
      Self::EstatRequiresPriorRegression => {
        formatter.write_str("estat requires a prior regress model")
      }
      Self::EstatVifFailed => formatter.write_str("estat vif failed for current model"),
      Self::EstatArRequiresPriorIv => formatter.write_str("estat ar requires a prior 2SLS fit"),
      Self::BootstrapFailed(message) => write!(formatter,"bootstrap: {message}"),
      Self::EstatArFailed(message) => write!(formatter, "estat ar: {message}"),
      Self::EstatOveridRequiresPriorIv => {
        formatter.write_str("estat overid requires a prior 2SLS fit")
      }
      Self::EstatOveridFailed(message) => write!(formatter, "estat overid: {message}"),
      Self::EstatEndogenousRequiresPriorIv => {
        formatter.write_str("estat endogenous requires a prior 2SLS fit")
      }
      Self::EstatEndogenousFailed(message) => write!(formatter, "estat endogenous: {message}"),
      Self::LincomRequiresPriorEstimation => {
        formatter.write_str("no active estimation results found")
      }
      Self::LincomFailed { message } => formatter.write_str(message),
      Self::TestRequiresPriorEstimation => {
        formatter.write_str("no active estimation results found")
      }
      Self::TestFailed { message } => formatter.write_str(message),
      Self::TtestUnknownVariable { variables } => {
        write!(
          formatter,
          "ttest unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::TtestRequiresNumeric { variables } => {
        write!(
          formatter,
          "ttest requires numeric variables: {}",
          variables.join(", ")
        )
      }
      Self::TtestNoValidNumericObservations => {
        formatter.write_str("ttest: no valid numeric observations found")
      }
      Self::TtestNoValidPairedNumericObservations => {
        formatter.write_str("ttest: no valid paired numeric observations found")
      }
      Self::TtestInvalidGroupCount { found } => write!(
        formatter,
        "ttest: by() variable must define exactly two groups (found {found})"
      ),
      Self::TtestUnsupportedGroupType {
        variable,
        data_type,
      } => write!(
        formatter,
        "ttest: unsupported by() variable type for {variable}: {data_type}"
      ),
      Self::TtestFailed { message } => formatter.write_str(message),
      Self::PredictRequiresPriorRegression => {
        formatter.write_str("predict requires a prior regress model")
      }
      Self::PredictTargetExists { variable } => {
        write!(formatter, "predict target already exists: {variable}")
      }
      Self::PredictUnknownVariable { variables } => {
        write!(
          formatter,
          "predict unknown variable: {}",
          variables.join(", ")
        )
      }
      Self::PredictFailed => formatter.write_str("predict failed"),
    }
  }
}

impl Error for RuntimeError {}

impl From<ScriptError> for RuntimeError {
  fn from(error: ScriptError) -> Self {
    Self::ScriptError(error)
  }
}
