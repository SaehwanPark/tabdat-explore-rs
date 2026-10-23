use serde::{Deserialize, Serialize};
pub use tabdat_stats::UnitEffectsFTest;
pub use tabdat_stats::{
  BinaryCluster, BinaryOmnibusWald, CountProbabilityLaw, CovarianceType, LogitEstimate,
  MAX_COUNT_POINT_CUTOFF, Nb2Cluster, Nb2Dispersion, Nb2Estimate, Nb2MarginalEffects, Nb2Result,
  PoissonCluster, PoissonEstimate, PoissonMarginalEffects, PoissonOmnibusWald, PoissonResult,
};
pub use tabdat_stats::{
  BootstrapOlsResult, ClusterBootstrapOlsResult, QuantileBootstrapResult, QuantileResult,
};

use crate::{ExecutionMode, LabelValue, LazyEngine};
use std::path::PathBuf;

/// A column in the loaded dataset schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnInfo {
  /// The source column name, in schema order.
  pub name: String,
  /// The DuckDB logical type name.
  pub data_type: String,
}

/// Owned metadata for the active dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// The owned result returned after an eager local Parquet or CSV load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadResult {
  /// Metadata for the newly active dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after projecting columns with `keep`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepResult {
  /// Metadata for the newly active projected dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after projecting out columns with `drop`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropResult {
  /// Metadata for the newly active projected dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after projecting an explicit selection with
/// `select`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectResult {
  /// Metadata for the newly active projected dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned by a read-only `describe` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescribeResult {
  /// The unchanged metadata for the active dataset.
  pub dataset: DatasetInfo,
}

/// Small projection of the retained fit, not its design, predictions or inference arrays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinearModelStatus {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub degrees_of_freedom: usize,
  pub sample_total_observations: Option<usize>,
  pub sample_dropped_observations: Option<usize>,
}

/// Metadata-only session inspection; not a model/current-data compatibility check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusResult {
  pub data_initialized: bool,
  pub dataset: Option<DatasetInfo>,
  pub active_table_name: Option<String>,
  /// Registered names in literal sorted order, without relation data or metadata copies.
  pub named_tables: Vec<String>,
  /// Most recent retained estimator family, including non-linear models.
  pub estimation_family: Option<crate::EstimationFamily>,
  pub linear_model: Option<LinearModelStatus>,
}

/// The owned result after counting current rows, optionally restricted to TRUE predicate rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CountResult {
  /// The selected row count, or all active rows when no predicate was supplied.
  pub row_count: u64,
}

/// An owned descriptive-statistics row for one numeric column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryRow {
  /// The requested column name.
  pub variable: String,
  /// The number of finite non-NULL numeric values used by all summary aggregates.
  pub count: u64,
  /// The arithmetic mean, or `None` when the finite sample is empty.
  pub mean: Option<f64>,
  /// The sample standard deviation, or `None` for fewer than two values.
  pub std_dev: Option<f64>,
  /// The minimum finite value, preserving its owned DuckDB scalar type.
  pub minimum: Option<CellValue>,
  /// The maximum finite value, preserving its owned DuckDB scalar type.
  pub maximum: Option<CellValue>,
}

/// The owned result returned by a read-only `summarize` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummarizeResult {
  /// Rows in requested order, including repeated explicit variables.
  pub rows: Vec<SummaryRow>,
}

/// Fixed percentiles reported by the bounded finite-sample detail workflow.
pub const SUMMARY_PERCENTILES: [u8; 9] = [1, 5, 10, 25, 50, 75, 90, 95, 99];

/// A type-7 linearly interpolated percentile after conversion to DOUBLE.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryPercentile {
  pub percentile: u8,
  /// Undefined for an empty finite sample; not an exact integer/decimal scalar.
  pub value: Option<f64>,
}

/// Ordinary finite statistics and percentiles of the same per-variable sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryDetailRow {
  pub summary: SummaryRow,
  pub percentiles: Vec<SummaryPercentile>,
}

/// Read-only bounded detail rows, retaining request order and duplicates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummarizeDetailResult {
  pub rows: Vec<SummaryDetailRow>,
}

/// An owned codebook profile row for one active column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodebookResult {
  /// Rows in schema or requested order, including repeated explicit variables.
  pub rows: Vec<CodebookRow>,
}

/// An owned missingness row for one active column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissingResult {
  /// Rows in schema or requested order, including repeated explicit variables.
  pub rows: Vec<MissingRow>,
}

/// The owned duplicate-key aggregate returned by a read-only request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertResult {
  /// The number of active rows checked by the predicate.
  pub checked: u64,
  /// The number of rows whose predicate was false or SQL NULL.
  pub failed: u64,
}

/// The owned result returned after appending a generated numeric column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateResult {
  /// Metadata for the newly active generated dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after replacing values in an existing column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceResult {
  /// Metadata for the newly active replaced dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after renaming one column in the active dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameResult {
  /// Metadata for the newly active renamed dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after stably sorting active rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortResult {
  /// Metadata for the newly active sorted dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after stably sorting active rows by directed keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GsortResult {
  /// Metadata for the newly active directed-sort dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after recoding selected columns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecodeResult {
  /// Metadata for the newly active recoded dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after encoding one string column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodeResult {
  /// Metadata for the newly active encoded dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after decoding one encode-produced numeric column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodeResult {
  /// Metadata for the newly active decoded dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after joining the active relation with a named table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinResult {
  /// Metadata for the newly active joined dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after appending rows from a named table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendResult {
  /// Metadata for the newly active combined dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after reshaping the active dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReshapeResult {
  /// Metadata for the newly active reshaped dataset.
  pub dataset: DatasetInfo,
}

/// A normalized named value-label set owned by the active session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueLabelSet {
  /// The set name.
  pub name: String,
  /// Values and their display labels in deterministic order.
  pub mappings: Vec<(LabelValue, String)>,
}

/// Session-local variable and value-label metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelMetadata {
  /// Variable names and their display labels.
  pub variable_labels: Vec<(String, String)>,
  /// Named value-label sets.
  pub value_sets: Vec<ValueLabelSet>,
  /// Variable names and their attached value-label set names.
  pub attachments: Vec<(String, String)>,
}

impl LabelMetadata {
  pub fn is_empty(&self) -> bool {
    self.variable_labels.is_empty() && self.value_sets.is_empty() && self.attachments.is_empty()
  }
}

/// The metadata mutation represented by a successful `label` result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelResult {
  /// The label action that completed.
  pub action: LabelResultAction,
  /// A deterministic human-readable completion message.
  pub message: String,
  /// The resulting metadata, or `None` when the session has no labels.
  pub metadata: Option<LabelMetadata>,
}

/// The owned result returned by a bounded one- or two-way frequency table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabulateResult {
  /// Human-readable table headers in output order.
  pub headers: Vec<String>,
  /// Owned table rows in deterministic category order.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned by a bounded grouped read-only request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ByResult {
  /// Group and aggregate headers in output order.
  pub headers: Vec<String>,
  /// Owned grouped rows in deterministic group order.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned after replacing the active relation with a
/// grouped aggregate relation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseResult {
  /// Metadata for the newly active collapsed dataset.
  pub dataset: DatasetInfo,
}

/// The owned result returned after writing the active dataset to Parquet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveResult {
  /// The output path requested by the caller.
  pub path: PathBuf,
  /// Output-oriented metadata for the dataset that was written.
  pub dataset: DatasetInfo,
}

/// The owned result returned after writing the active dataset to CSV.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportResult {
  /// The output path requested by the caller.
  pub path: PathBuf,
  /// Output-oriented metadata for the dataset that was written.
  pub dataset: DatasetInfo,
}

/// Candidate-row selection for one scatter or overlaid line series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotSeriesSummary {
  pub variable: String,
  pub point_count: usize,
  /// Rows without a complete finite x/series pair, not an estimation exclusion.
  pub omitted_rows: usize,
}

/// Metadata returned only after a complete generated SVG is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotArtifactResult {
  pub path: PathBuf,
  pub format: String,
  pub width: u32,
  pub height: u32,
  pub candidate_rows: usize,
  pub x_variable: String,
  pub scatter: PlotSeriesSummary,
  pub lines: Vec<PlotSeriesSummary>,
  pub jitter_y: f64,
  pub jitter_convention: String,
  /// Numeric coordinates are float64, not lossless wide-integer/decimal IDs.
  pub coordinate_precision: String,
}

/// An owned scalar value in a bounded dataset preview.
///
/// Values are copied out of DuckDB before the result leaves the runtime so a
/// caller never depends on a backend row or connection lifetime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum CellValue {
  /// An SQL NULL value.
  Null,
  /// A boolean value.
  Boolean(bool),
  /// A signed integer value, widened without loss of precision.
  SignedInteger(#[serde(with = "crate::wire::signed_decimal")] i128),
  /// An unsigned integer value, widened without loss of precision.
  UnsignedInteger(#[serde(with = "crate::wire::unsigned_decimal")] u128),
  /// A floating-point value.
  Float(#[serde(with = "crate::wire::float_text")] f64),
  /// A decimal value with its declared width, scale, and scaled payload.
  Decimal {
    width: u8,
    scale: u8,
    #[serde(with = "crate::wire::signed_decimal")]
    value: i128,
  },
  /// A UTF-8 text value.
  Text(String),
  /// An owned binary value.
  Bytes(Vec<u8>),
}

/// The owned result returned by a bounded preview request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewResult {
  /// Column names in the active relation's schema order.
  pub columns: Vec<String>,
  /// Rows in relation insertion order, limited to the requested preview.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned after executing a .td script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResult {
  /// The resolved path of the executed script.
  pub path: PathBuf,
  /// Total count of successfully executed commands in the script and nested scripts.
  pub executed_commands: usize,
}

/// The owned result returned by a direct `sql <query>` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableResult {
  /// Column headers in query projection order.
  pub headers: Vec<String>,
  /// Owned table rows in deterministic query result order.
  pub rows: Vec<Vec<CellValue>>,
}

/// The owned result returned by `sql <query> into <table>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqlCreateResult {
  /// The destination named table name.
  pub table_name: String,
  /// Metadata for the newly created and activated relation.
  pub dataset: DatasetInfo,
}

/// The owned result returned after activating a registered named table with `use <table>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivateResult {
  /// The activated table name.
  pub table_name: String,
  /// Metadata for the activated relation.
  pub dataset: DatasetInfo,
}

/// The typed result of executing a linear regression command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionResult {
  /// Outcome (dependent) variable name.
  pub outcome: String,
  /// Ordered predictor variable names.
  pub predictors: Vec<String>,
  /// Estimator type (for example, "ols", "wls", "gls", or "2sls").
  pub estimator: String,
  /// Covariance description (e.g. "nonrobust", "robust", "cluster(<var>)").
  pub covariance: String,
  /// Number of observations in the estimation sample.
  pub observation_count: usize,
  /// Whether an intercept was included.
  pub include_intercept: bool,
  /// R-squared goodness of fit (if computed).
  pub r_squared: Option<f64>,
  /// Adjusted R-squared (if computed).
  pub adjusted_r_squared: Option<f64>,
  /// Root mean squared error (if computed).
  pub root_mse: Option<f64>,
  /// Estimated coefficients and inference.
  pub coefficients: Vec<tabdat_stats::CoefficientEstimate>,
  /// Conventional excluded-instrument first-stage tests for IV fits.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub first_stage: Option<Vec<tabdat_stats::FirstStageDiagnostic>>,
  /// Underlying linear coefficient, covariance, fit-statistics, and sample payload.
  pub least_squares: tabdat_stats::LeastSquaresResult,
}

impl RegressionResult {
  /// Gaussian working-density log likelihood using ML variance RSS/N, not the
  /// residual-df variance used for coefficient covariance. This is descriptive,
  /// not a Bernoulli likelihood or robust/cross-family LR test. Weighted and IV
  /// conventions are deliberately not reported; zero/nonfinite variance is null.
  pub fn gaussian_working_log_likelihood(&self) -> Option<f64> {
    if self.estimator != "ols"
      || self
        .least_squares
        .sample
        .as_ref()
        .is_some_and(|sample| sample.weights.is_some())
    {
      return None;
    }
    self
      .least_squares
      .fit_statistics
      .log_likelihood
      .filter(|value| value.is_finite())
  }
}

/// The owned result of estimating one affine combination of regression coefficients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LincomResult {
  /// Human-readable form of the requested expression.
  pub label: String,
  /// Estimated combination, including any coefficient-independent constant.
  pub estimate: f64,
  /// Covariance-based standard error.
  pub standard_error: f64,
  /// Student-t statistic against a zero combination, or NaN for zero standard error.
  pub statistic: f64,
  /// Two-sided Student-t p-value, or NaN for zero standard error.
  pub p_value: f64,
  /// Lower endpoint of the 95% confidence interval.
  pub ci_lower: f64,
  /// Upper endpoint of the 95% confidence interval.
  pub ci_upper: f64,
  /// Confidence level in percent.
  pub ci_level: f64,
  /// Residual degrees of freedom used for inference.
  pub df_residual: Option<usize>,
}

/// The owned result returned after testing linear restrictions on a fitted model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
  /// Formatted restrictions in their requested order.
  pub constraints: Vec<String>,
  /// Wald F statistic or chi-square statistic for the tested restrictions.
  pub statistic: f64,
  /// Upper-tail probability for the selected reference distribution.
  pub p_value: f64,
  /// Numerator degrees of freedom (the number of restrictions).
  pub df: usize,
  /// Residual degrees of freedom for an F test, if available.
  pub df_residual: Option<usize>,
  /// Whether `statistic` follows a chi-square rather than F distribution.
  pub is_chi2: bool,
}

/// Descriptive statistics for one sample reported by `ttest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtestGroupStats {
  /// Variable name or group label represented by this sample.
  pub name: String,
  /// Group observation count, or the oracle's residual-df count for an independent difference.
  pub observations: usize,
  /// Arithmetic mean.
  pub mean: f64,
  /// Sample standard error.
  pub standard_error: f64,
  /// Unbiased sample standard deviation; NaN for an independent-sample difference.
  pub standard_deviation: f64,
  /// Lower endpoint of the two-sided 95% Student-t confidence interval.
  pub ci_lower: f64,
  /// Upper endpoint of the two-sided 95% Student-t confidence interval.
  pub ci_upper: f64,
}

/// The owned result of a one-sample, paired, pooled, or Welch `ttest` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtestResult {
  /// First tested variable.
  pub varname1: String,
  /// Second tested variable for a paired test.
  pub varname2: Option<String>,
  /// Grouping variable for an independent-sample test.
  pub by_variable: Option<String>,
  /// Whether the command tested paired observations.
  pub is_paired: bool,
  /// Whether the independent-sample test used Welch's unequal-variance method.
  pub is_welch: bool,
  /// Null mean for a one-sample test; zero for paired and independent tests.
  pub null_value: f64,
  /// Summary for the first sample or lexically first group.
  pub group1_stats: TtestGroupStats,
  /// Summary for the second sample or lexically second group, when present.
  pub group2_stats: Option<TtestGroupStats>,
  /// Combined summary for independent-sample tests.
  pub combined_stats: Option<TtestGroupStats>,
  /// Summary of the tested sample mean or mean difference.
  pub difference_stats: TtestGroupStats,
  /// Student t statistic, or NaN when the standard error is zero or undefined.
  pub t_statistic: f64,
  /// Reference-distribution degrees of freedom.
  pub degrees_of_freedom: f64,
  /// Lower-tail probability under the reference Student-t distribution.
  pub p_left: f64,
  /// Two-sided probability under the reference Student-t distribution.
  pub p_two: f64,
  /// Upper-tail probability under the reference Student-t distribution.
  pub p_right: f64,
}

/// The owned result returned after adding fitted values or residuals to the active dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionResult {
  /// Metadata for the newly active dataset with the appended prediction column.
  pub dataset: DatasetInfo,
}

/// Distributional plug-in frequencies on the original retained count sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountProbabilityResult {
  pub outcome: String,
  pub distribution: tabdat_stats::CountProbabilityLaw,
  pub sample: Option<tabdat_stats::EstimationSample>,
  pub table: tabdat_stats::CountProbabilityTable,
}

/// Results currently exposed by the bounded runtime slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum ExecutionResult {
  EstimateStore(crate::EstimateStoreResult),
  EstimateTable(crate::EstimateTableResult),
  EstimateDrop(crate::EstimateDropResult),
  /// A successfully loaded dataset.
  Load(LoadResult),
  /// The metadata for the currently active dataset.
  Describe(DescribeResult),
  /// The row count for the currently active dataset.
  Count(CountResult),
  /// Metadata-only current data and retained linear-fit inspection.
  Status(StatusResult),
  /// Descriptive statistics for selected numeric columns.
  Summarize(SummarizeResult),
  SummarizeDetail(SummarizeDetailResult),
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
  /// Writes the active relation to a Parquet file without changing state.
  Save(SaveResult),
  /// Writes the active relation to a CSV file without changing state.
  Export(ExportResult),
  /// Publishes a generated SVG without changing data, labels or estimation state.
  Plot(PlotArtifactResult),
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
  /// The result of executing a script file.
  Run(RunResult),
  /// The result of executing a direct SQL query.
  Table(TableResult),
  /// The result of creating a named table from a SQL query.
  SqlCreate(SqlCreateResult),
  /// The result of activating a registered named table.
  Activate(ActivateResult),
  /// Replaces the active relation with a joined dataset from a named table.
  Join(JoinResult),
  /// Replaces the active relation with an appended dataset from a named table.
  Append(AppendResult),
  /// Replaces the active relation with a reshaped dataset.
  Reshape(ReshapeResult),
  /// The result of executing a linear regression command.
  Regression(Box<RegressionResult>),
  /// One-way within FE with unit centers and slope-only covariance/inference.
  FixedEffects(Box<crate::FixedEffectsState>),
  RandomEffects(Box<crate::RandomEffectsState>),
  /// Converged model-based binary logit maximum likelihood fit.
  Logit(Box<tabdat_stats::BinaryLogitResult>),
  /// Conditional-on-retained-X continuous average partial effects and delta inference.
  LogitMarginalEffects(tabdat_stats::LogitMarginalEffects),
  /// Converged model-based binary probit maximum likelihood fit.
  Probit(Box<tabdat_stats::BinaryMleResult>),
  /// Probit conditional-on-retained-X average partial effects and delta inference.
  ProbitMarginalEffects(Box<tabdat_stats::LogitMarginalEffects>),
  /// Log-link Poisson ML or independent-row robust QMLE fit and diagnostics.
  Poisson(Box<PoissonResult>),
  /// Continuous retained-X derivatives with selected-covariance delta inference.
  PoissonMarginalEffects(Box<PoissonMarginalEffects>),
  /// NB2 ML/independent-row robust fit with joint log-dispersion covariance.
  Nbreg(Box<Nb2Result>),
  /// Retained-X continuous effects using the full joint selected covariance.
  NbregMarginalEffects(Box<Nb2MarginalEffects>),
  /// Descriptive observed/fitted count probabilities; no test or uncertainty.
  CountProbabilities(Box<CountProbabilityResult>),
  /// The result of appending fitted values, probabilities or residuals.
  Prediction(PredictionResult),
  /// The result of estimating an affine linear combination of regression coefficients.
  Lincom(LincomResult),
  /// The result of testing linear restrictions on regression coefficients.
  Test(TestResult),
  /// Homoskedastic joint weak-IV-robust test on a retained IV design.
  AndersonRubin(tabdat_stats::AndersonRubinResult),
  /// HC1/clustered joint AR Wald test, with asymptotic chi-square inference.
  RobustAndersonRubin(tabdat_stats::RobustAndersonRubinResult),
  /// Homoskedastic Sargan N*R² overidentification test on retained 2SLS residuals.
  Sargan(tabdat_stats::SarganResult),
  /// Joint homoskedastic residual-inclusion F test of endogenous-regressor exogeneity.
  WuHausman(tabdat_stats::WuHausmanResult),
  Quantile(Box<tabdat_stats::QuantileResult>),
  Bootstrap(Box<tabdat_stats::BootstrapOlsResult>),
  QuantileBootstrap(Box<tabdat_stats::QuantileBootstrapResult>),
  ClusterBootstrap(Box<tabdat_stats::ClusterBootstrapOlsResult>),
  /// Scalar homoskedastic AR confidence set, including unbounded or empty sets.
  AndersonRubinConfidenceSet(tabdat_stats::AndersonRubinConfidenceSet),
  /// The result of a two-sided Student t-test over active dataset columns.
  Ttest(Box<TtestResult>),
}
