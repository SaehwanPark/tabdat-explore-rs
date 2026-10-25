use serde::{Deserialize, Serialize};

mod endogenous_variables_wire {
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  pub fn serialize<S>(values: &[String], serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    if values.len() == 1 {
      serializer.serialize_str(&values[0])
    } else {
      values.serialize(serializer)
    }
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
      One(String),
      Many(Vec<String>),
    }

    Ok(match OneOrMany::deserialize(deserializer)? {
      OneOrMany::One(value) => vec![value],
      OneOrMany::Many(values) => values,
    })
  }
}

/// The syntax-only command forms currently understood by the language layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum Command {
  /// Show general help or the help topic named by `topic`.
  Help { topic: Option<String> },
  /// Show the current session status.
  Status,
  /// Request termination of the interactive session.
  Exit,
  /// Inspect the active dataset schema (execution is deferred).
  Describe,
  /// Inspect environment and capability health (execution is deferred).
  Doctor,
  /// Compute descriptive statistics for selected columns.
  Summarize { variables: Vec<String> },
  /// Conditional/option-bearing summary syntax. Bounded predicates and one bare
  /// `detail` flag execute; other option-bearing forms remain uninstalled.
  SummarizeQuery {
    variables: Vec<String>,
    condition: Option<GenerateExpression>,
    options: Vec<SyntaxOption>,
  },
  /// Conditional row selection, with a bounded boolean/null expression runtime;
  /// distinct from column keep/drop operations and parsed function-call support.
  RowFilter {
    action: RowFilterAction,
    condition: GenerateExpression,
  },
  /// Compute a signature for the active dataset in the bounded eager runtime.
  Datasignature,
  /// Profile selected columns.
  Codebook { variables: Vec<String> },
  /// Report explicit-null missingness for selected columns.
  Missing { variables: Vec<String> },
  /// Report duplicate key groups in the bounded eager runtime.
  Duplicates { variables: Vec<String> },
  /// Assert key uniqueness for selected variables in the bounded eager runtime.
  Isid {
    variables: Vec<String>,
    missok: bool,
  },
  /// Validate a boolean predicate against every row in the bounded eager runtime.
  Assert { expression: AssertExpression },
  /// Parse a generated-column expression without executing it.
  Generate {
    /// The target column name.
    variable: String,
    /// The owned expression assigned to the target column.
    expression: GenerateExpression,
  },
  /// Parse a replacement expression without executing it.
  Replace {
    /// The existing target column name.
    variable: String,
    /// The owned expression assigned to the target.
    expression: GenerateExpression,
    /// An optional row predicate retained for a later runtime slice.
    condition: Option<GenerateExpression>,
  },
  /// Keep an explicit ordered set of columns in the bounded eager runtime.
  Keep { variables: Vec<String> },
  /// Drop an explicit set of columns in the bounded eager runtime.
  Drop { variables: Vec<String> },
  /// Keep only listed columns (relation execution is deferred).
  Select { variables: Vec<String> },
  /// Sort active rows by listed columns in the bounded eager runtime.
  Sort { variables: Vec<String> },
  /// Sort active rows by explicitly directed keys in the bounded eager runtime.
  Gsort { keys: Vec<SortKey> },
  /// Recode values or ranges in the bounded eager runtime.
  Recode {
    /// The source variables in parser order.
    variables: Vec<String>,
    /// The ordered recode rules.
    rules: Vec<RecodeRule>,
    /// Whether to append generated columns or replace the source columns.
    target: RecodeTarget,
  },
  /// Encode one string column into a new integer-coded column.
  Encode {
    /// The existing string source column.
    source: String,
    /// The new integer-coded target column.
    generate: String,
    /// An optional value-label set name retained for the later label slice.
    label: Option<String>,
  },
  /// Decode an encode-produced numeric column into its original strings.
  Decode {
    /// The existing numeric source column.
    source: String,
    /// The new string target column.
    generate: String,
  },
  /// Manage bounded session-local variable and value labels.
  Label { command: LabelCommand },
  /// Produce bounded eager frequency tables for one or two variables.
  Tabulate { command: TabulateCommand },
  /// Bounded conditional frequencies; multidimensional/value aggregation forms parse only.
  TabulateQuery {
    command: TabulateCommand,
    condition: Option<GenerateExpression>,
    aggregation: Option<TabulateAggregation>,
  },
  /// Join the active dataset with a named table (execution is deferred).
  Join { command: JoinCommand },
  /// Append rows from a named table (execution is deferred).
  Append { table_name: String },
  /// Reshape the active dataset between long and wide layouts (execution is deferred).
  Reshape { command: ReshapeCommand },
  /// Report or declare panel identifiers (execution is deferred).
  Panel { command: PanelCommand },
  /// Apply a panel within/between transform (execution is deferred).
  XtData { command: XtDataCommand },
  /// Fit a bounded instrumental-variables model (2SLS is executable).
  IvRegress { command: IvRegressCommand },
  /// Read-only seeded IID or whole-cluster pairs-bootstrap OLS report on current complete cases.
  Bootstrap { command: BootstrapCommand },
  /// Read-only joint IID pairs-bootstrap quantiles, sharing draws across quantiles.
  QuantileBootstrap { command: QuantileBootstrapCommand },
  /// Fit a fixed- or random-effects panel model (execution is deferred).
  XtReg { command: XtRegCommand },
  /// Fit a dynamic-panel Arellano-Bond model (execution is deferred).
  XtAbond { command: XtAbondCommand },
  /// Fit a fixed-effects panel logit model (execution is deferred).
  XtLogit { command: XtLogitCommand },
  /// Fit a locally weighted regression smoother (execution is deferred).
  Lowess { command: LowessCommand },
  /// Fit a difference-in-differences model (execution is deferred).
  Did { command: DidCommand },
  /// Fit a doubly robust difference-in-differences model (execution is deferred).
  DrDid { command: DrDidCommand },
  /// Fit a double machine learning model (execution is deferred).
  Dml { command: DmlCommand },
  /// Fit a control function regression model (execution is deferred).
  CfRegress { command: CfRegressCommand },
  /// Compute a linear combination of fitted model parameters.
  Lincom { command: LincomCommand },
  /// Test linear hypotheses after estimation.
  Test { command: TestCommand },
  /// Compute a histogram of a variable (visualization execution is deferred).
  Histogram { command: HistogramCommand },
  /// Save a bounded SVG scatter with optional numeric line overlays.
  Scatter { command: ScatterCommand },
  /// Compute a bar chart of a categorical variable (visualization execution is deferred).
  Bar { command: BarCommand },
  /// Diagnostic plot of Bayesian MCMC samples (visualization execution is deferred).
  BayesPlot { command: BayesPlotCommand },
  /// Run a bounded post-estimation diagnostic (execution is deferred).
  Estat { command: EstatCommand },
  /// Store, compare or drop immutable named fit reports, without restoring fits.
  Estimates { command: crate::EstimatesCommand },
  /// Run a one-sample, paired, or independent two-sample t-test.
  Ttest { command: TtestCommand },
  /// Run a bounded grouped read-only child command.
  By { command: ByCommand },
  /// Replace the active dataset with a bounded grouped aggregate relation.
  Collapse { command: CollapseCommand },
  /// Rename one column in the bounded eager runtime.
  Rename { old_name: String, new_name: String },
  /// Execute a script file (script execution is deferred).
  Run { path: String },
  /// Change a runtime setting (configuration execution is deferred).
  Set { name: SettingName, value: String },
  /// Persist the active dataset to a path (filesystem execution is deferred).
  Save { path: String, replace: bool },
  /// Export the active dataset to a path (filesystem execution is deferred).
  Export { path: String, replace: bool },
  /// Select a dataset source and loading options (execution is deferred).
  Use {
    source: DataSource,
    execution_mode: ExecutionMode,
    lazy_engine: Option<LazyEngine>,
    delimiter: Option<String>,
    has_header: Option<bool>,
  },
  /// Count rows in the active dataset.
  Count,
  /// Count TRUE predicate rows without changing the active dataset.
  CountQuery { condition: GenerateExpression },
  /// Preview the first `limit` rows of the active dataset.
  Head { limit: RowLimit },
  /// Preview the last `limit` rows of the active dataset.
  Tail { limit: RowLimit },
  /// Execute a SQL query (execution is deferred).
  Sql { command: SqlCommand },
  /// Fit a linear regression model (execution is deferred).
  Regress { command: RegressCommand },
  /// Fit a linear model on TRUE predicate rows, retaining original sample indices.
  RegressQuery {
    command: RegressCommand,
    condition: GenerateExpression,
  },
  /// Fit a logistic regression model (execution is deferred).
  Logit { command: LogitCommand },
  /// Fit a probit regression model (execution is deferred).
  Probit { command: ProbitCommand },
  /// Unweighted model-based multinomial logit with a fixed reference category.
  Mlogit { command: MlogitCommand },
  /// Single-choice conditional logit on balanced explicit choice sets.
  Clogit { command: ClogitCommand },
  /// Explicit non-normalized two-nest model, not preferred RUM normalization.
  Nlogit { command: NlogitCommand },
  /// Preferred utility normalization with one explicitly selected common scale.
  NlogitNormalizedCommon(NlogitCommand),
  /// Fit unweighted model-based or independent-row robust Poisson regression.
  Poisson { command: PoissonCommand },
  /// Fit bounded unweighted NB2 with jointly estimated log dispersion.
  Nbreg { command: NbregCommand },
  /// Fit a zero-inflated Poisson regression model (execution is deferred).
  Zip { command: ZipCommand },
  /// Fit a zero-inflated negative binomial regression model (execution is deferred).
  Zinb { command: ZinbCommand },
  /// Fit a quantile regression model (execution is deferred).
  Qreg { command: QregCommand },
  /// Fit a tobit (censored) regression model (execution is deferred).
  Tobit { command: TobitCommand },
  /// Fit IID Gaussian regression conditional on outcome above a known limit.
  Truncreg { command: TruncregCommand },
  /// Explicit two-step sample selection; the default ML form remains deferred.
  Heckman { command: HeckmanCommand },
  /// Fit a nonlinear regression model (execution is deferred).
  Nl { command: NlCommand },
  /// Fit bounded entry-zero Weibull PH regression; other parsed variants remain unsupported.
  Streg { command: StregCommand },
  /// Fit entry-zero Cox proportional hazards with Breslow ties and no intercept.
  Stcox { command: StcoxCommand },
  /// Fit a spatial autoregressive regression model (execution is deferred).
  Spregress { command: SpregressCommand },
  /// Fit a lasso regularized regression model (execution is deferred).
  Lasso { command: LassoCommand },
  /// Fit a post-lasso OLS regression model (execution is deferred).
  Postlasso { command: PostlassoCommand },
  /// Fit a ridge regularized regression model (execution is deferred).
  Ridge { command: RidgeCommand },
  /// Fit an elastic net regularized regression model (execution is deferred).
  Elasticnet { command: ElasticnetCommand },
  /// Fit a cross-validated lasso model (execution is deferred).
  Cvlasso { command: CvlassoCommand },
  /// Fit a cross-validated ridge model (execution is deferred).
  Cvridge { command: CvridgeCommand },
  /// Fit a cross-validated elastic net model (execution is deferred).
  Cvelasticnet { command: CvelasticnetCommand },
  /// Fit a direct Bayesian linear regression model (execution is deferred).
  Bayes { command: BayesCommand },
  /// Run a Bayesian estimation model using MCMC sampling (execution is deferred).
  BayesPrefix { command: BayesPrefixCommand },
  /// Predict post-estimation values (execution is deferred).
  Predict { command: PredictCommand },
}

/// The bounded Bayesian estimation prefix command form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BayesPrefixCommand {
  /// The inner estimation command (restricted to regress or logit).
  pub command: Box<Command>,
  /// The number of MCMC draws.
  pub draws: Option<i64>,
  /// The number of warmup/burn-in draws.
  pub burnin: Option<i64>,
  /// The number of MCMC chains.
  pub chains: Option<i64>,
  /// The thinning interval.
  pub thin: Option<i64>,
  /// The random seed for the sampler.
  pub seed: Option<i64>,
  /// The custom prior distribution specifications as `(variable, distribution)`.
  pub priors: Vec<(String, String)>,
}

/// The bounded SQL command AST representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqlCommand {
  /// The opaque SQL query text.
  pub query: String,
  /// The optional named-table target.
  pub into: Option<String>,
}

/// A value accepted by a bounded `label define` command.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum LabelValue {
  /// An integer label value.
  Integer(i64),
  /// A non-integer numeric spelling retained without floating-point loss.
  Number(String),
  /// A text label value.
  Text(String),
}

/// The non-I/O session-local forms of the `label` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum LabelCommand {
  /// Set or clear the display label for one variable. `None` clears it.
  Variable {
    /// The target variable.
    variable: String,
    /// The new display label, or `None` for `, clear`.
    text: Option<String>,
  },
  /// Define or replace a named value-label set.
  Define {
    /// The value-label set name.
    name: String,
    /// The value-to-display-text mappings in source order.
    mappings: Vec<(LabelValue, String)>,
    /// Whether an existing set may be replaced.
    replace: bool,
  },
  /// Attach or clear a value-label set for one variable. `None` clears it.
  Values {
    /// The target variable.
    variable: String,
    /// The attached set name, or `None` for `, clear`.
    set_name: Option<String>,
  },
  /// Inspect all or selected value-label sets.
  List {
    /// Optional set names to filter.
    names: Vec<String>,
  },
  /// Drop one or more named value-label sets.
  Drop {
    /// The set names to remove.
    names: Vec<String>,
  },
}

/// The bounded frequency-table forms of `tabulate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabulateCommand {
  /// The one row variable supported by this slice.
  pub row_variables: Vec<String>,
  /// The optional one column variable supported by this slice.
  pub column_variables: Vec<String>,
  /// Include row percentages in a two-way table.
  pub row_percent: bool,
  /// Include column percentages in a two-way table.
  pub column_percent: bool,
  /// Include SQL NULL as an observed category.
  pub include_missing: bool,
  /// Suppress attached value-label display.
  pub nolabel: bool,
}

/// A tabulate value variable and statistic are present together or not at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabulateAggregation {
  pub variable: String,
  pub statistic: CollapseStatistic,
}

/// The finite aggregate functions accepted by bounded `collapse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollapseStatistic {
  /// Count non-NULL values in each aggregate variable.
  Count,
  /// Compute the arithmetic mean of each aggregate variable.
  Mean,
  /// Compute the sum of each aggregate variable.
  Sum,
  /// Compute the minimum of each aggregate variable.
  Min,
  /// Compute the maximum of each aggregate variable.
  Max,
}

/// The bounded grouped aggregate form of `collapse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseCommand {
  /// The aggregate function applied to every requested variable.
  pub statistic: CollapseStatistic,
  /// Aggregate variables in source order.
  pub variables: Vec<String>,
  /// Grouping variables in source order.
  pub groups: Vec<String>,
}

/// The grouped read-only form of `by`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByCommand {
  /// Grouping variables in source order.
  pub groups: Vec<String>,
  /// The child command retained for bounded runtime dispatch.
  pub command: Box<Command>,
}

/// The bounded join modes accepted by the syntax boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinHow {
  /// Keep only active rows with a matching named-table row.
  Inner,
  /// Keep every active row and fill unmatched right-side values with NULL.
  Left,
}

/// The bounded named-table join form used by the execution boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinCommand {
  /// The user-visible named table to join.
  pub table_name: String,
  /// Ordered join keys shared by the active and named-table relations.
  pub keys: Vec<String>,
  /// Whether unmatched active rows are retained.
  pub how: JoinHow,
  /// Suffix used for colliding right-side columns before native name validation.
  pub suffix: String,
}

/// The two layout directions accepted by the bounded `reshape` syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReshapeDirection {
  /// Convert repeated wide columns into rows.
  Long,
  /// Convert repeated rows into suffixed wide columns.
  Wide,
}

/// The parser-only reshape form retained for a later relation runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReshapeCommand {
  /// Whether the eventual runtime should produce long or wide output.
  pub direction: ReshapeDirection,
  /// Ordered source variables participating in the reshape.
  pub variables: Vec<String>,
  /// Ordered identifier variables retained across reshaped rows or columns.
  pub identifiers: Vec<String>,
  /// The output variable carrying the long-form j values.
  pub j_variable: String,
}

/// The actions accepted by the parser-only `panel` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum PanelAction {
  /// Report the current panel declaration.
  Report,
  /// Clear the current panel declaration.
  Clear,
  /// Declare the entity and time variables for the active relation.
  Set {
    /// The entity identifier variable.
    id_variable: String,
    /// The time variable.
    time_variable: String,
  },
}

/// The parser-only panel form retained for a later relation runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelCommand {
  /// The requested panel action.
  pub action: PanelAction,
}

/// The panel-index transform forms accepted by the parser-only `xtdata`
/// command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XtDataTransform {
  /// Demean each requested variable within its panel entity.
  Within,
  /// Replace each requested variable with its entity-level mean.
  Between,
}

/// The parser-only `xtdata` form retained for a later panel runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XtDataCommand {
  /// Ordered variables selected for transformation.
  pub variables: Vec<String>,
  /// The requested within- or between-entity transform.
  pub transform: XtDataTransform,
}

/// The estimator forms accepted by the parser-only `regress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegressEstimator {
  /// Ordinary least squares.
  Ols,
  /// Weighted least squares.
  Wls,
  /// Generalized least squares.
  Gls,
}

/// Supported linear estimator, covariance, weight and intercept conventions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The requested estimator.
  pub estimator: RegressEstimator,
  /// Optional weight variable for WLS or GLS.
  pub weight_variable: Option<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Binary logit syntax; runtime supports unweighted model-based covariance.
/// Robust and cluster forms remain parsed but are explicitly rejected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `probit` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProbitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Shared person-level design across unordered numeric categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MlogitCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub base_outcome: Option<i64>,
  pub include_intercept: bool,
}

/// Shared utility slopes on alternative-specific numeric columns; no common intercept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClogitCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub group_variable: String,
  pub alternative_variable: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NlogitCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub upper_predictors: Vec<String>,
  pub group_variable: String,
  pub alternative_variable: String,
  pub nest_variable: String,
}

/// Bounded unweighted log-link Poisson with model-based, independent-row
/// robust or one-way clustered score covariance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoissonCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub offset_variable: Option<String>,
  /// Independent-row score sandwich, corrected by N/(N-1).
  pub robust: bool,
  pub cluster_variable: Option<String>,
  pub include_intercept: bool,
}

/// Bounded log-link NB2, variance mu+alpha*mu^2, with full joint observed
/// information and model-based, row-robust or one-way group-score covariance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NbregCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub offset_variable: Option<String>,
  /// Full joint observed-information score sandwich with N/(N-1).
  pub robust: bool,
  pub cluster_variable: Option<String>,
  pub include_intercept: bool,
}

/// The parser-only `zip` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZipCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Ordered zero-inflation predictor variables.
  pub inflate_predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `zinb` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZinbCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Ordered zero-inflation predictor variables.
  pub inflate_predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Unweighted conditional quantile regression; inference options are runtime-validated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QregCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The requested quantile value, retained as string spelling.
  pub quantile: String,
  /// Parsed robust request; the installed IID-only runtime rejects it.
  pub robust: bool,
  /// Whether to include an intercept.
  pub include_intercept: bool,
}

/// Lower-censored IID Gaussian Tobit. Upper/robust/cluster syntax remains
/// accepted but is explicitly rejected by the bounded model-based runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TobitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The lower censoring limit, retained as string spelling.
  pub lower_limit: String,
  /// The optional upper censoring limit, retained as string spelling.
  pub upper_limit: Option<String>,
  /// Request robust covariance (currently rejected, never silently ignored).
  pub robust: bool,
  /// Optional cluster variable (currently rejected).
  pub cluster_variable: Option<String>,
  /// Whether the fitted latent mean includes an intercept.
  pub include_intercept: bool,
}

/// Known fixed lower truncation, not censoring; unsupported covariance variants
/// are not accepted or silently reinterpreted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TruncregCommand {
  pub outcome: String,
  pub predictors: Vec<String>,
  pub lower_limit: String,
  pub include_intercept: bool,
}

/// Sample selection request. Only explicit `twostep` has a native runtime;
/// the historical bare form is not silently reinterpreted as two-step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeckmanCommand {
  /// Explicit two-step method; absent legacy serialized fields remain deferred.
  #[serde(default)]
  pub twostep: bool,
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Selection equation dependent variable.
  pub selection_dependent: String,
  /// Selection equation predictor variables.
  pub selection_predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `nl` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NlCommand {
  /// The dependent variable.
  pub outcome: String,
  /// The nonlinear mathematical expression.
  pub expression: GenerateExpression,
  /// Ordered parameter names.
  pub parameter_names: Vec<String>,
  /// Starting values for the parameters.
  pub start_values: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Entry-zero single-spell Cox PH with Breslow ties and independent-spell inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StcoxCommand {
  pub time_variable: String,
  pub failure_variable: String,
  pub predictors: Vec<String>,
  pub robust: bool,
}

/// Parsed distributions; only entry-zero Weibull has native execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StregDistribution {
  /// Weibull proportional hazards / accelerated failure time model.
  Weibull,
  /// Exponential survival model.
  Exponential,
}

/// Parametric survival specification. Native execution currently supports
/// entry-zero single-spell Weibull, model-based or independent-spell robust;
/// exponential and cluster variants are parsed but remain unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StregCommand {
  /// The survival time variable.
  pub time_variable: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Event / failure indicator variable.
  pub failure_variable: String,
  /// The parametric baseline hazard distribution.
  pub distribution: StregDistribution,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The spatial model types accepted by the parser-only `spregress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpregressModelType {
  /// Spatial autoregressive model (lag of y).
  Lag,
  /// Spatial error model.
  Error,
  /// Spatial autoregressive combined with spatial error (SARAR).
  Sarar,
}

/// The spatial contiguity types accepted by the parser-only `spregress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpregressContiguity {
  /// Queen contiguity (common edge or vertex).
  Queen,
  /// Rook contiguity (common edge only).
  Rook,
}

/// The parser-only `spregress` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpregressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The spatial model specification (`lag`, `error`, or `sarar`).
  pub model_type: SpregressModelType,
  /// Optional latitude/longitude coordinate variable pair for distance-based weights.
  pub coord_variables: Option<(String, String)>,
  /// Number of nearest neighbors when `coord_variables` is used (default: 5).
  pub knn: Option<i64>,
  /// Optional path to external weights file (.gal, .gwt, or shapefile).
  pub weights_file: Option<String>,
  /// Entity ID variable name required when `weights_file` is specified.
  pub id_variable: Option<String>,
  /// Contiguity criterion when `weights_file` is specified (`queen` or `rook`, default: `queen`).
  pub contiguity: Option<SpregressContiguity>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
}

/// The parser-only `lasso` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LassoCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Regularization penalty parameter, retained as string spelling.
  pub alpha: String,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `postlasso` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostlassoCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Regularization penalty parameter, retained as string spelling.
  pub alpha: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `ridge` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RidgeCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Regularization penalty parameter, retained as string spelling.
  pub alpha: String,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `elasticnet` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElasticnetCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Regularization penalty parameter, retained as string spelling.
  pub alpha: String,
  /// Elastic net mixing parameter between 0 and 1 inclusive, retained as string spelling.
  pub l1_ratio: String,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `cvlasso` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvlassoCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Cross-validation folds (must be at least 2, default: 5).
  pub cv: i64,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `cvridge` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvridgeCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Cross-validation folds (must be at least 2, default: 5).
  pub cv: i64,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The elastic net mixing parameter forms accepted by `cvelasticnet`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum CvelasticnetL1Ratio {
  /// Single mixing parameter value retained as string spelling.
  Single(String),
  /// Multiple candidate mixing parameter values retained as string spellings.
  Multiple(Vec<String>),
}

/// The parser-only `cvelasticnet` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvelasticnetCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Cross-validation folds (must be at least 2, default: 5).
  pub cv: i64,
  /// Elastic net mixing parameter or candidate list, retained as string spellings.
  pub l1_ratio: CvelasticnetL1Ratio,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only direct `bayes` linear regression form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BayesCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Maximum number of iterations for evidence maximization (default: 300).
  pub n_iter: i64,
  /// Convergence tolerance for evidence maximization, retained as string spelling (default: "0.001").
  pub tol: String,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Bounded pairs-bootstrap report; does not install or replace a retained fit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapCommand {
  pub regression: RegressCommand,
  pub replications: usize,
  pub seed: u64,
}

/// The quantile bootstrap is a report, not a retained estimator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantileBootstrapCommand {
  pub bootstrap: BootstrapCommand,
  pub quantiles: Vec<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub contrast: Option<String>,
}

/// The requested prediction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictKind {
  /// Linear prediction (fitted values $X\hat{\beta}$).
  Xb,
  /// Residuals ($y - X\hat{\beta}$), including unit intercepts after within FE.
  Residuals,
  /// Conditional FE response including the retained unit intercept (`xbu`).
  FixedEffectsResponse,
  /// Retained normalized unit deviation (`u`) after within FE.
  FixedEffect,
  /// Family-defined probabilities: binary/choice, Tobit uncensoring or selection.
  Pr,
  /// Family-defined Gaussian mean (`mean`): E[max(lower,Y*)|X] for Tobit,
  /// E[Y*|Y*>lower,X] for truncated regression. Retains its legacy wire tag.
  TobitMean,
  /// Selection inverse Mills ratio phi(Z gamma)/Phi(Z gamma).
  SelectionMills,
  /// E[Y*|D=1,X,Z] after Heckman two-step.
  SelectionConditionalMean,
  /// E[D Y*|X,Z], with zero contribution when not selected.
  SelectionObservedContribution,
  /// Multinomial scalar probability for an explicitly selected retained category.
  MultinomialPr(i64),
  /// Multinomial reference-relative index (zero for the base category).
  MultinomialXb(i64),
  /// Conditional mean exp(xb), requested by `n` after Poisson or NB2.
  MeanCount,
  /// Distributional P(Y=k) after Poisson/NB2, k=0..99.
  CountPmf(usize),
  /// Distributional P(Y<=k) after Poisson/NB2, k=0..99.
  CountCdf(usize),
  /// Distributional P(Y=k) after Poisson/NB2 using a current-row cutoff variable.
  CountPmfVariable,
  /// Distributional P(Y<=k) after Poisson/NB2 using a current-row cutoff variable.
  CountCdfVariable,
  /// Current-time cause-specific Weibull hazard (not a probability).
  Hazard,
  /// Current-time integrated cause-specific Weibull or Cox/Breslow hazard.
  CumulativeHazard,
  /// Weibull or Cox net survival exp(-H), not competing-risk incidence.
  Survival,
  /// Predicted spatial lag for spatial models.
  SpatialLag,
  /// Posterior predictive draws for Bayesian models.
  PosteriorPredictive,
}

/// The parser-only `predict` form retained for a later post-estimation
/// runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictCommand {
  /// Target variable name to store predictions.
  pub target_variable: String,
  /// Prediction kind.
  pub kind: PredictKind,
  /// Current-row count cutoff variable for `CountPmfVariable`/`CountCdfVariable`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub count_cutoff_variable: Option<String>,
  /// Whether interval bounds are requested.
  pub interval: bool,
  /// Credible interval level, retained as string spelling.
  pub level: String,
  /// Whether standard deviation of posterior predictive draws is requested.
  pub std: bool,
  /// Optional file path to save posterior draws.
  pub saving: Option<String>,
}

/// The bounded estimator forms accepted by `ivregress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IvEstimator {
  /// Two-stage least squares.
  TwoStageLeastSquares,
  /// Generalized method of moments.
  GeneralizedMethodOfMoments,
}

/// Bounded `ivregress` syntax; the runtime currently executes 2SLS only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IvRegressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered exogenous regressors.
  pub exogenous: Vec<String>,
  /// Ordered endogenous regressors. The internal wire keeps a single variable
  /// as the pre-list string form and uses an array for multiple variables.
  #[serde(with = "endogenous_variables_wire")]
  pub endogenous: Vec<String>,
  /// Ordered instrumental variables.
  pub instruments: Vec<String>,
  /// Request HC1 covariance; it cannot be combined with clustering.
  pub robust: bool,
  /// Optional variable defining one-way cluster-robust covariance.
  pub cluster_variable: Option<String>,
  /// Whether the fitted model includes an intercept.
  pub include_intercept: bool,
  /// The requested IV estimator; only 2SLS is currently executed.
  pub estimator: IvEstimator,
}

/// Bounded one-way panel estimators accepted by `xtreg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XtRegEstimator {
  /// Fixed-effects panel estimator.
  FixedEffects,
  /// Random-effects panel estimator.
  RandomEffects,
  /// Gaussian marginal maximum likelihood, optionally with interior unit clustering.
  RandomEffectsMl,
}

/// Executable panel estimators require explicit id/time options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XtRegCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The requested fixed- or random-effects estimator.
  pub estimator: XtRegEstimator,
  /// Request unit clustering (not independent-row HC1).
  pub robust: bool,
  /// Panel estimators permit only clustering on the declared entity variable.
  pub cluster_variable: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub id_variable: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub time_variable: Option<String>,
}

/// The parser-only `xtabond` dynamic-panel estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XtAbondCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Maximum lag depth used by the eventual estimator.
  pub lag_depth: i64,
  /// First lag used for the eventual instrument set.
  pub instrument_lag_start: i64,
}

/// The parser-only `xtlogit` fixed-effects panel logit estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XtLogitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
}

/// The parser-only `lowess` locally weighted regression smoother form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowessCommand {
  /// The dependent variable.
  pub outcome: String,
  /// The predictor variable.
  pub predictor: String,
  /// The target smoothed variable name.
  pub target_variable: String,
  /// The bandwidth smoothing parameter, retained as string spelling.
  pub bandwidth: String,
}

/// The parser-only `did` difference-in-differences estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Optional control variables.
  pub controls: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Post-treatment time period indicator variable.
  pub post_variable: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
}

/// The doubly robust difference-in-differences estimator methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrDidMethod {
  /// Outcome regression.
  Or,
  /// Inverse probability weighting.
  Ipw,
  /// Augmented inverse probability weighting (doubly robust).
  Aipw,
}

/// The parser-only `drdid` doubly robust difference-in-differences estimator form retained
/// for a later statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrDidCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Optional covariate control variables.
  pub covariates: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Post-treatment time period indicator variable.
  pub post_variable: String,
  /// Estimation method.
  pub method: DrDidMethod,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Number of bootstrap replications.
  pub bootstrap: Option<i64>,
  /// Random seed for bootstrap.
  pub seed: Option<i64>,
}

/// The parser-only `dml` double machine learning estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmlCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Control variables.
  pub controls: Vec<String>,
  /// Treatment indicator variable.
  pub treatment_variable: String,
  /// Number of cross-fitting folds (default: 5).
  pub folds: i64,
  /// Regularization penalty parameter, retained as string spelling (default: "1.0").
  pub alpha: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Random seed for cross-fitting splits.
  pub seed: Option<i64>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `cfregress` control function estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CfRegressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Exogenous covariate variables.
  pub exogenous: Vec<String>,
  /// Endogenous variable.
  pub endogenous: String,
  /// Instrumental variables.
  pub instruments: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Cluster identifier variable.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// Parsed `lincom` linear combination specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LincomCommand {
  /// The linear combination expression to estimate.
  pub expression: GenerateExpression,
}

/// Parsed `test` linear hypothesis testing specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCommand {
  /// The linear constraints to test.
  pub constraints: Vec<GenerateExpression>,
}

/// Parsed `histogram` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistogramCommand {
  /// The single variable to plot.
  pub variable: String,
  /// Optional bin count.
  pub bins: Option<i64>,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// Parsed `scatter` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScatterCommand {
  /// The y-axis variable to plot.
  pub y_variable: String,
  /// The x-axis variable to plot.
  pub x_variable: String,
  /// Up to three columns to connect in stable x/row order, without changing data.
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub line_variables: Vec<String>,
  /// Optional finite nonnegative y-display amplitude, retained as a numeric literal.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub jitter_y: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// Parsed `bar` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarCommand {
  /// The variable to plot frequency categories for.
  pub variable: String,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to include missing values as a category in the bar chart.
  pub include_missing: bool,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// The diagnostic plot kinds supported by `bayesplot`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BayesPlotKind {
  /// Trace plot of MCMC iterations.
  Trace,
  /// Density plot of posterior draws.
  Density,
  /// Autocorrelation plot across MCMC lags.
  Autocorrelation,
}

/// Parsed `bayesplot` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BayesPlotCommand {
  /// The MCMC diagnostic plot kind (trace, density, or autocorrelation).
  pub kind: BayesPlotKind,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// The direct comparison forms accepted by `ttest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtestCommand {
  /// The first variable in the comparison.
  pub varname1: String,
  /// The optional second variable for a paired comparison.
  pub varname2: Option<String>,
  /// The optional numeric value, retained as source spelling.
  pub value: Option<String>,
  /// The optional grouping variable for a two-sample comparison.
  pub by_variable: Option<String>,
  /// Whether unequal-variance inference was requested.
  pub welch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowFilterAction {
  Keep,
  Drop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxOption {
  pub name: String,
  pub value: SyntaxOptionValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum SyntaxOptionValue {
  Flag,
  Number(String),
  String(String),
  Boolean(bool),
  Identifiers(Vec<String>),
  Numbers(Vec<String>),
  Prior(String, String),
}

/// Recognize the single bare detail flag shared by execution and JSON admission.
/// This does not admit grouped summaries or validate a row predicate.
pub fn summarize_detail_requested(options: &[SyntaxOption]) -> bool {
  matches!(options, [SyntaxOption { name, value: SyntaxOptionValue::Flag }] if name == "detail")
}

/// Post-estimation diagnostic syntax; options exist only on applicable variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstatSubcommand {
  Residuals,
  Ovtest,
  Margins,
  /// Descriptive observed/fitted retained-count bins, ending in >= tail_from.
  CountProbabilities {
    tail_from: usize,
  },
  Gof,
  Did,
  Drdid,
  Dml,
  Bayes,
  Report {
    saving: Option<String>,
    open_artifact: bool,
  },
  SpatialCoordinates {
    latitude: String,
    longitude: String,
    knn: i64,
  },
  SpatialWeights {
    path: String,
    id_variable: String,
    contiguity: SpregressContiguity,
  },
  /// Report variance inflation factors for a linear regression model.
  Vif,
  /// Scalar homoskedastic AR confidence set; level is a percentage.
  AndersonRubinConfidenceSet {
    level: String,
  },
  /// Project a multi-endogenous joint homoskedastic AR region onto one coefficient.
  AndersonRubinProjectedConfidenceSet {
    coefficient: String,
    level: String,
  },
  /// Joint homoskedastic Anderson-Rubin null, in endogenous fit order.
  AndersonRubin {
    null_values: Vec<String>,
  },
  /// Report the first-stage diagnostic for an IV model.
  FirstStage,
  /// Report the overidentification diagnostic for an IV model.
  Overid,
  /// Report the endogeneity diagnostic for an IV model.
  Endogenous,
  /// Compare fixed- and random-effects panel models.
  Hausman,
}

/// An `estat` form accepted by the language layer; runtime support is
/// subcommand-specific.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstatCommand {
  /// The requested diagnostic subcommand.
  pub subcommand: EstatSubcommand,
}

/// An expression accepted by the bounded eager `assert` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum AssertExpression {
  /// A dataset column reference.
  Identifier(String),
  /// A validated numeric literal preserved as SQL-safe source text.
  Number(String),
  /// A quoted string literal.
  String(String),
  /// An explicit SQL NULL literal.
  Null,
  /// Unary numeric negation.
  UnaryMinus(Box<Self>),
  /// A binary arithmetic or comparison expression.
  Binary {
    /// Left operand.
    left: Box<Self>,
    /// Operator between operands.
    operator: AssertBinaryOperator,
    /// Right operand.
    right: Box<Self>,
  },
}

/// An expression AST shared by data and statistical command forms.
///
/// The parser preserves a broader syntax than each runtime executes. This remains
/// separate from [`AssertExpression`], which models only the bounded predicate forms
/// accepted by `assert` and does not represent function calls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum GenerateExpression {
  /// A dataset column reference.
  Identifier(String),
  /// A validated numeric literal preserved as source text.
  Number(String),
  /// A quoted string literal.
  String(String),
  /// An explicit SQL NULL literal.
  Null,
  /// Unary numeric negation.
  UnaryMinus(Box<Self>),
  /// A binary arithmetic or comparison expression.
  Binary {
    /// Left operand.
    left: Box<Self>,
    /// Operator between operands.
    operator: GenerateBinaryOperator,
    /// Right operand.
    right: Box<Self>,
  },
  /// A function call with arguments in source order.
  FunctionCall {
    /// Function name as written by the caller.
    name: String,
    /// Arguments in source order.
    arguments: Vec<Self>,
  },
}

/// Operators accepted by [`GenerateExpression::Binary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerateBinaryOperator {
  /// Addition.
  Add,
  /// Subtraction.
  Subtract,
  /// Multiplication.
  Multiply,
  /// Division.
  Divide,
  /// Equality.
  Equal,
  /// Inequality.
  NotEqual,
  /// Less-than comparison.
  Less,
  /// Less-than-or-equal comparison.
  LessOrEqual,
  /// Greater-than comparison.
  Greater,
  /// Greater-than-or-equal comparison.
  GreaterOrEqual,
}

/// Operators accepted by [`AssertExpression::Binary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertBinaryOperator {
  /// Addition.
  Add,
  /// Subtraction.
  Subtract,
  /// Multiplication.
  Multiply,
  /// Division.
  Divide,
  /// Equality.
  Equal,
  /// Inequality.
  NotEqual,
  /// Less-than comparison.
  Less,
  /// Less-than-or-equal comparison.
  LessOrEqual,
  /// Greater-than comparison.
  Greater,
  /// Greater-than-or-equal comparison.
  GreaterOrEqual,
}

/// A local path or an unvalidated remote URI supplied to `use`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum DataSource {
  LocalPath(String),
  Uri(String),
}

/// Whether a `use` request should load eagerly or build a lazy plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
  Eager,
  Lazy,
}

/// The lazy engine named by a `use` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LazyEngine {
  DuckDb,
  Polars,
}

/// The finite setting names accepted by the syntax-only `set` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingName {
  /// Select the artifact image format (value validation is deferred).
  GraphFormat,
  /// Select the directory used for generated artifacts (validation is deferred).
  ArtifactDir,
  /// Select whether generated graphs open automatically (validation is deferred).
  GraphOpen,
}

/// One owned `gsort` key and its requested direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortKey {
  /// The variable spelling after an optional unquoted direction prefix.
  pub variable: String,
  /// Whether the key requests descending order.
  pub descending: bool,
}

/// A value accepted on either side of a bounded `recode` rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RecodeValue {
  /// A numeric literal preserved as source text.
  Number(String),
  /// A text value, including a parsed unquoted keyword outside its structural
  /// role.
  Text(String),
}

/// One endpoint of an inclusive numeric `recode` range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RecodeRangeEndpoint {
  /// The lower unbounded endpoint.
  Min,
  /// The upper unbounded endpoint.
  Max,
  /// A numeric endpoint preserved as source text.
  Number(String),
}

/// One input accepted by a bounded `recode` rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RecodeInput {
  /// A single scalar value.
  Value(RecodeValue),
  /// An inclusive numeric range.
  Range {
    /// The lower endpoint.
    start: RecodeRangeEndpoint,
    /// The upper endpoint.
    end: RecodeRangeEndpoint,
  },
  /// SQL NULL values.
  Missing,
  /// Non-NULL values.
  NonMissing,
  /// The unmatched fallback value.
  Else,
}

/// One ordered bounded `recode` rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecodeRule {
  /// The input values or ranges matched by this rule.
  pub inputs: Vec<RecodeInput>,
  /// The value emitted for a matching row.
  pub output: RecodeValue,
}

/// The mutually exclusive write target for a bounded `recode` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum RecodeTarget {
  /// Append one new output column per source variable.
  Generate { variables: Vec<String> },
  /// Replace the selected source variables in place.
  Replace,
}

impl AssertBinaryOperator {
  /// Return whether this operator produces a boolean comparison.
  pub fn is_comparison(self) -> bool {
    matches!(
      self,
      Self::Equal
        | Self::NotEqual
        | Self::Less
        | Self::LessOrEqual
        | Self::Greater
        | Self::GreaterOrEqual
    )
  }
}

/// A validated, canonical non-negative decimal row limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RowLimit(Box<str>);

impl RowLimit {
  /// Validate an ASCII non-negative integer and remove redundant leading zeros.
  /// No machine-integer conversion is required, so arbitrarily large limits survive.
  pub fn new(value: &str) -> Result<Self, &'static str> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
      return Err("row limit must be a non-negative ASCII decimal integer");
    }
    let canonical = value.trim_start_matches('0');
    Ok(Self(
      if canonical.is_empty() { "0" } else { canonical }.into(),
    ))
  }

  /// Return the canonical ASCII decimal representation without parsing it into
  /// a bounded machine integer.
  pub fn as_decimal(&self) -> &str {
    &self.0
  }
}

impl Default for RowLimit {
  fn default() -> Self {
    Self("5".into())
  }
}

impl TryFrom<String> for RowLimit {
  type Error = &'static str;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::new(&value)
  }
}
impl From<RowLimit> for String {
  fn from(value: RowLimit) -> Self {
    value.0.into()
  }
}
