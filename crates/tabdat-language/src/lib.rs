#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

/// The syntax-only command forms currently understood by the language layer.
#[derive(Debug, Clone, PartialEq, Eq)]
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
  /// Fit an instrumental-variables model (execution is deferred).
  IvRegress { command: IvRegressCommand },
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
  /// Compute linear combination of model parameters (execution is deferred).
  Lincom { command: LincomCommand },
  /// Test linear hypotheses after estimation (execution is deferred).
  Test { command: TestCommand },
  /// Compute a histogram of a variable (visualization execution is deferred).
  Histogram { command: HistogramCommand },
  /// Compute a scatter plot of two variables (visualization execution is deferred).
  Scatter { command: ScatterCommand },
  /// Compute a bar chart of a categorical variable (visualization execution is deferred).
  Bar { command: BarCommand },
  /// Diagnostic plot of Bayesian MCMC samples (visualization execution is deferred).
  BayesPlot { command: BayesPlotCommand },
  /// Run a bounded post-estimation diagnostic (execution is deferred).
  Estat { command: EstatCommand },
  /// Run a bounded two-sample test (execution is deferred).
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
  /// Preview the first `limit` rows of the active dataset.
  Head { limit: RowLimit },
  /// Preview the last `limit` rows of the active dataset.
  Tail { limit: RowLimit },
  /// Execute a SQL query (execution is deferred).
  Sql { command: SqlCommand },
  /// Fit a linear regression model (execution is deferred).
  Regress { command: RegressCommand },
  /// Fit a logistic regression model (execution is deferred).
  Logit { command: LogitCommand },
  /// Fit a probit regression model (execution is deferred).
  Probit { command: ProbitCommand },
  /// Fit a Poisson regression model (execution is deferred).
  Poisson { command: PoissonCommand },
  /// Fit a negative binomial regression model (execution is deferred).
  Nbreg { command: NbregCommand },
  /// Fit a zero-inflated Poisson regression model (execution is deferred).
  Zip { command: ZipCommand },
  /// Fit a zero-inflated negative binomial regression model (execution is deferred).
  Zinb { command: ZinbCommand },
  /// Fit a quantile regression model (execution is deferred).
  Qreg { command: QregCommand },
  /// Fit a tobit (censored) regression model (execution is deferred).
  Tobit { command: TobitCommand },
  /// Fit a Heckman sample-selection regression model (execution is deferred).
  Heckman { command: HeckmanCommand },
  /// Fit a nonlinear regression model (execution is deferred).
  Nl { command: NlCommand },
  /// Fit a parametric survival regression model (execution is deferred).
  Streg { command: StregCommand },
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlCommand {
  /// The opaque SQL query text.
  pub query: String,
  /// The optional named-table target.
  pub into: Option<String>,
}

/// A value accepted by a bounded `label define` command.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LabelValue {
  /// An integer label value.
  Integer(i64),
  /// A non-integer numeric spelling retained without floating-point loss.
  Number(String),
  /// A text label value.
  Text(String),
}

/// The non-I/O session-local forms of the `label` command.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The finite aggregate functions accepted by bounded `collapse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapseCommand {
  /// The aggregate function applied to every requested variable.
  pub statistic: CollapseStatistic,
  /// Aggregate variables in source order.
  pub variables: Vec<String>,
  /// Grouping variables in source order.
  pub groups: Vec<String>,
}

/// The grouped read-only form of `by`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByCommand {
  /// Grouping variables in source order.
  pub groups: Vec<String>,
  /// The child command retained for bounded runtime dispatch.
  pub command: Box<Command>,
}

/// The bounded join modes accepted by the syntax boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinHow {
  /// Keep only active rows with a matching named-table row.
  Inner,
  /// Keep every active row and fill unmatched right-side values with NULL.
  Left,
}

/// The named-table join form retained for a later runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinCommand {
  /// The user-visible named table to join.
  pub table_name: String,
  /// Ordered join keys shared by the active and named-table relations.
  pub keys: Vec<String>,
  /// Whether unmatched active rows are retained.
  pub how: JoinHow,
  /// Suffix used by the eventual runtime for colliding right-side columns.
  pub suffix: String,
}

/// The two layout directions accepted by the bounded `reshape` syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReshapeDirection {
  /// Convert repeated wide columns into rows.
  Long,
  /// Convert repeated rows into suffixed wide columns.
  Wide,
}

/// The parser-only reshape form retained for a later relation runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelCommand {
  /// The requested panel action.
  pub action: PanelAction,
}

/// The panel-index transform forms accepted by the parser-only `xtdata`
/// command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XtDataTransform {
  /// Demean each requested variable within its panel entity.
  Within,
  /// Replace each requested variable with its entity-level mean.
  Between,
}

/// The parser-only `xtdata` form retained for a later panel runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XtDataCommand {
  /// Ordered variables selected for transformation.
  pub variables: Vec<String>,
  /// The requested within- or between-entity transform.
  pub transform: XtDataTransform,
}

/// The estimator forms accepted by the parser-only `regress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressEstimator {
  /// Ordinary least squares.
  Ols,
  /// Weighted least squares.
  Wls,
  /// Generalized least squares.
  Gls,
}

/// The parser-only `regress` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The parser-only `logit` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The parser-only `poisson` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoissonCommand {
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

/// The parser-only `nbreg` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbregCommand {
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

/// The parser-only `zip` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The parser-only `qreg` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QregCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The requested quantile value, retained as string spelling.
  pub quantile: String,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `tobit` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TobitCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The lower censoring limit, retained as string spelling.
  pub lower_limit: String,
  /// The optional upper censoring limit, retained as string spelling.
  pub upper_limit: Option<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
}

/// The parser-only `heckman` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeckmanCommand {
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The parametric distributions accepted by the parser-only `streg` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StregDistribution {
  /// Weibull proportional hazards / accelerated failure time model.
  Weibull,
  /// Exponential survival model.
  Exponential,
}

/// The parser-only `streg` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpregressModelType {
  /// Spatial autoregressive model (lag of y).
  Lag,
  /// Spatial error model.
  Error,
  /// Spatial autoregressive combined with spatial error (SARAR).
  Sarar,
}

/// The spatial contiguity types accepted by the parser-only `spregress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpregressContiguity {
  /// Queen contiguity (common edge or vertex).
  Queen,
  /// Rook contiguity (common edge only).
  Rook,
}

/// The parser-only `spregress` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CvelasticnetL1Ratio {
  /// Single mixing parameter value retained as string spelling.
  Single(String),
  /// Multiple candidate mixing parameter values retained as string spellings.
  Multiple(Vec<String>),
}

/// The parser-only `cvelasticnet` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The requested prediction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictKind {
  /// Linear prediction (fitted values $X\hat{\beta}$).
  Xb,
  /// Residuals ($y - X\hat{\beta}$).
  Residuals,
  /// Predicted probabilities for binary choice models.
  Pr,
  /// Predicted spatial lag for spatial models.
  SpatialLag,
  /// Posterior predictive draws for Bayesian models.
  PosteriorPredictive,
}

/// The parser-only `predict` form retained for a later post-estimation
/// runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredictCommand {
  /// Target variable name to store predictions.
  pub target_variable: String,
  /// Prediction kind.
  pub kind: PredictKind,
  /// Whether interval bounds are requested.
  pub interval: bool,
  /// Credible interval level, retained as string spelling.
  pub level: String,
  /// Whether standard deviation of posterior predictive draws is requested.
  pub std: bool,
  /// Optional file path to save posterior draws.
  pub saving: Option<String>,
}

/// The estimator forms accepted by the parser-only `ivregress` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IvEstimator {
  /// Two-stage least squares.
  TwoStageLeastSquares,
  /// Generalized method of moments.
  GeneralizedMethodOfMoments,
}

/// The parser-only `ivregress` form retained for a later statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IvRegressCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered exogenous regressors.
  pub exogenous: Vec<String>,
  /// The single endogenous regressor supported by this syntax slice.
  pub endogenous: String,
  /// Ordered instrumental variables.
  pub instruments: Vec<String>,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
  /// Whether the eventual runtime should include an intercept.
  pub include_intercept: bool,
  /// The requested IV estimator.
  pub estimator: IvEstimator,
}

/// The estimator forms accepted by the parser-only `xtreg` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XtRegEstimator {
  /// Fixed-effects panel estimator.
  FixedEffects,
  /// Random-effects panel estimator.
  RandomEffects,
}

/// The parser-only `xtreg` form retained for a later panel/statistical runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XtRegCommand {
  /// The dependent variable.
  pub outcome: String,
  /// Ordered predictor variables.
  pub predictors: Vec<String>,
  /// The requested fixed- or random-effects estimator.
  pub estimator: XtRegEstimator,
  /// Request robust covariance in the eventual runtime.
  pub robust: bool,
  /// Optional cluster variable for the eventual runtime.
  pub cluster_variable: Option<String>,
}

/// The parser-only `xtabond` dynamic-panel estimator form retained for a later
/// statistical runtime slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LincomCommand {
  /// The linear combination expression to estimate.
  pub expression: GenerateExpression,
}

/// Parsed `test` linear hypothesis testing specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCommand {
  /// The linear constraints to test.
  pub constraints: Vec<GenerateExpression>,
}

/// Parsed `histogram` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScatterCommand {
  /// The y-axis variable to plot.
  pub y_variable: String,
  /// The x-axis variable to plot.
  pub x_variable: String,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// Parsed `bar` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BayesPlotKind {
  /// Trace plot of MCMC iterations.
  Trace,
  /// Density plot of posterior draws.
  Density,
  /// Autocorrelation plot across MCMC lags.
  Autocorrelation,
}

/// Parsed `bayesplot` visualization specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BayesPlotCommand {
  /// The MCMC diagnostic plot kind (trace, density, or autocorrelation).
  pub kind: BayesPlotKind,
  /// Optional file path to save the generated plot.
  pub saving: Option<String>,
  /// Whether to open the generated artifact in the browser/viewer (default true).
  pub open_artifact: bool,
}

/// The parser-only direct comparison forms accepted by `ttest`.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// The no-option post-estimation diagnostics accepted by this parser-only
/// `estat` slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstatSubcommand {
  /// Report the first-stage diagnostic for an IV model.
  FirstStage,
  /// Report the overidentification diagnostic for an IV model.
  Overid,
  /// Report the endogeneity diagnostic for an IV model.
  Endogenous,
  /// Compare fixed- and random-effects panel models.
  Hausman,
}

/// The parser-only `estat` form retained for a later post-estimation runtime
/// slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstatCommand {
  /// The requested diagnostic subcommand.
  pub subcommand: EstatSubcommand,
}

/// An expression accepted by the bounded eager `assert` command.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// An expression retained by the syntax-only `generate` and `replace` commands.
///
/// This deliberately remains separate from [`AssertExpression`]: the bounded
/// assert runtime does not claim function-call support, while the language
/// layer must preserve calls for a later generate execution slice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSource {
  LocalPath(String),
  Uri(String),
}

/// Whether a `use` request should load eagerly or build a lazy plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
  Eager,
  Lazy,
}

/// The lazy engine named by a `use` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LazyEngine {
  DuckDb,
  Polars,
}

/// The finite setting names accepted by the syntax-only `set` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingName {
  /// Select the artifact image format (value validation is deferred).
  GraphFormat,
  /// Select the directory used for generated artifacts (validation is deferred).
  ArtifactDir,
  /// Select whether generated graphs open automatically (validation is deferred).
  GraphOpen,
}

/// One owned `gsort` key and its requested direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
  /// The variable spelling after an optional unquoted direction prefix.
  pub variable: String,
  /// Whether the key requests descending order.
  pub descending: bool,
}

/// A value accepted on either side of a bounded `recode` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecodeValue {
  /// A numeric literal preserved as source text.
  Number(String),
  /// A text value, including a parsed unquoted keyword outside its structural
  /// role.
  Text(String),
}

/// One endpoint of an inclusive numeric `recode` range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecodeRangeEndpoint {
  /// The lower unbounded endpoint.
  Min,
  /// The upper unbounded endpoint.
  Max,
  /// A numeric endpoint preserved as source text.
  Number(String),
}

/// One input accepted by a bounded `recode` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecodeRule {
  /// The input values or ranges matched by this rule.
  pub inputs: Vec<RecodeInput>,
  /// The value emitted for a matching row.
  pub output: RecodeValue,
}

/// The mutually exclusive write target for a bounded `recode` command.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowLimit(Box<str>);

impl RowLimit {
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

/// A deterministic error produced while parsing a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  message: String,
}

/// The lexical categories emitted by [`tokenize`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
  /// An identifier, optionally decoded from backtick quoting.
  Identifier { quoted: bool },
  /// A single- or double-quoted string.
  String,
  /// A decimal number spelling.
  Number,
  /// A command or expression operator.
  Symbol,
}

/// An owned lexical token from a command line.
///
/// Offsets mirror the Python oracle's `_Token` fields. In particular, a
/// quoted-string token's `start` is the offset immediately after its opening
/// quote because that is the recovered oracle contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
  /// The token category.
  pub kind: TokenKind,
  /// The decoded token text.
  pub text: String,
  /// The recorded Unicode-scalar start offset in the source text.
  pub start: usize,
  /// The exclusive Unicode-scalar end offset in the source text.
  pub end: usize,
}

fn is_command_whitespace(character: char) -> bool {
  character.is_whitespace() || matches!(character, '\u{1c}'..='\u{1f}')
}

fn leading_quote_error(command: &str, quote: u8) -> &'static str {
  let identifier = quote == b'`';
  let mut index = 1;
  let mut content_nonempty = false;
  let bytes = command.as_bytes();
  while index < bytes.len() {
    if bytes[index] == quote {
      if bytes.get(index + 1) == Some(&quote) {
        content_nonempty = true;
        index += 2;
        continue;
      }
      if identifier && !content_nonempty {
        return "quoted identifier cannot be empty";
      }
      return "command must start with an unquoted command name";
    }
    content_nonempty = true;
    index += 1;
  }
  if identifier {
    "unterminated quoted identifier"
  } else {
    "unterminated quoted string"
  }
}

impl ParseError {
  fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
    }
  }

  /// Return the stable human-readable parse diagnostic.
  pub fn message(&self) -> &str {
    &self.message
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.message)
  }
}

impl Error for ParseError {}

/// Parse one syntax-only command without executing it or initializing a backend.
pub fn parse_command(input: &str) -> Result<Command, ParseError> {
  let command = input.trim_matches(is_command_whitespace);
  if command.is_empty() {
    return Err(ParseError::new("empty command"));
  }

  if let Some(quote @ (b'`' | b'\'' | b'"')) = command.as_bytes().first().copied() {
    return Err(ParseError::new(leading_quote_error(command, quote)));
  }

  if let Some(help_body) = command.strip_prefix('?') {
    return parse_help(help_body.trim());
  }

  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"use"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"rename"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"sort"))
    && command.as_bytes().get(4) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"select"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"run"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"sql"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"regress"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"logit"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"probit"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"poisson"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"nbreg"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"zip"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"zinb"))
    && command.as_bytes().get(4) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"qreg"))
    && command.as_bytes().get(4) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"tobit"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"heckman"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..2)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"nl"))
    && command.as_bytes().get(2) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"streg"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..9)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"spregress"))
    && command.as_bytes().get(9) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"lasso"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..9)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"postlasso"))
    && command.as_bytes().get(9) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"ridge"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..10)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"elasticnet"))
    && command.as_bytes().get(10) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"cvlasso"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"cvridge"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..12)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"cvelasticnet"))
    && command.as_bytes().get(12) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"predict"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"xtlogit"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"lowess"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"did"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..5)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"drdid"))
    && command.as_bytes().get(5) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"dml"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..9)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"cfregress"))
    && command.as_bytes().get(9) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"lincom"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"test"))
    && command.as_bytes().get(4) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..9)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"histogram"))
    && command.as_bytes().get(9) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..7)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"scatter"))
    && command.as_bytes().get(7) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"bar"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..9)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"bayesplot"))
    && command.as_bytes().get(9) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }

  let first_word = command
    .split(is_command_whitespace)
    .next()
    .unwrap_or("")
    .to_ascii_lowercase();

  if first_word == "bayes:"
    || first_word.starts_with("bayes,")
    || (first_word == "bayes" && first_unquoted_colon(command).is_some())
  {
    return parse_bayes_prefix_command(command);
  }

  // `status` keeps the Python tokenizer's punctuation diagnostics for
  // attached unary-sign forms such as `status-1` and `status+1`.
  if command.len() > 6
    && command
      .as_bytes()
      .get(..6)
      .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"status"))
    && command
      .get(6..)
      .and_then(|suffix| suffix.chars().next())
      .is_some_and(|character| matches!(character, '-' | '+'))
  {
    return parse_named_command("status", &command[6..]);
  }

  // `gsort` permits attached symbolic key text (for example `gsort-age` and
  // `gsort:age`) in the pinned Python tokenizer. Split only this command's
  // non-identifier suffix before the generic command-name boundary scan.
  if command.len() > 5
    && command
      .as_bytes()
      .get(..5)
      .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"gsort"))
    && command
      .get(5..)
      .and_then(|suffix| suffix.chars().next())
      .is_some_and(|character| !character.is_alphanumeric() && character != '_')
  {
    return parse_named_command("gsort", &command[5..]);
  }

  for (name, prefix_length) in [("save", 4usize), ("export", 6usize)] {
    if command.len() > prefix_length
      && command
        .as_bytes()
        .get(..prefix_length)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name.as_bytes()))
      && command
        .get(prefix_length..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|character| !character.is_alphanumeric() && character != '_')
    {
      return parse_named_command(name, &command[prefix_length..]);
    }
  }

  let Some(command_end) = command
    .find(|character: char| is_command_whitespace(character) || matches!(character, ',' | '='))
  else {
    return parse_named_command(command, "");
  };
  let delimiter = command[command_end..]
    .chars()
    .next()
    .expect("command_end always points to a character");
  let name = &command[..command_end];
  let body = command[command_end..].trim_matches(is_command_whitespace);
  if name.eq_ignore_ascii_case("use") && delimiter == ',' {
    parse_use_options(command[command_end + 1..].trim_matches(is_command_whitespace))?;
    return Err(ParseError::new("unknown command: use"));
  }
  if name.eq_ignore_ascii_case("use") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("use assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("run") && delimiter == ',' {
    if command[command_end + 1..]
      .trim_matches(is_command_whitespace)
      .is_empty()
    {
      return Err(ParseError::new(
        "comma must be followed by at least one option",
      ));
    }
    return Err(ParseError::new("unknown command: run"));
  }
  if name.eq_ignore_ascii_case("run") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("run assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("sql") && delimiter == ',' {
    return Err(ParseError::new("unknown command: sql"));
  }
  if name.eq_ignore_ascii_case("lincom") && delimiter == ',' {
    if command[command_end + 1..]
      .trim_matches(is_command_whitespace)
      .is_empty()
    {
      return Err(ParseError::new(
        "comma must be followed by at least one option",
      ));
    }
    return Err(ParseError::new("unknown command: lincom"));
  }
  if name.eq_ignore_ascii_case("test") && delimiter == ',' {
    if command[command_end + 1..]
      .trim_matches(is_command_whitespace)
      .is_empty()
    {
      return Err(ParseError::new(
        "comma must be followed by at least one option",
      ));
    }
    return Err(ParseError::new("unknown command: test"));
  }
  if name.eq_ignore_ascii_case("sql") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("sql assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("regress") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "regress assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("logit") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "logit assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("probit") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "probit assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("bayes") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "bayes assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("poisson") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "poisson assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("nbreg") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "nbreg assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("zip") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("zip assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("zinb") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "zinb assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("qreg") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "qreg assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("tobit") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "tobit assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("heckman") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "heckman assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("nl") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("nl assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("streg") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "streg assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("spregress") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "spregress assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("lasso") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "lasso assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("postlasso") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "postlasso assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("ridge") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "ridge assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("elasticnet") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "elasticnet assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("cvlasso") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "cvlasso assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("cvridge") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "cvridge assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("cvelasticnet") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "cvelasticnet assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("predict") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "predict assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("xtlogit") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "xtlogit assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("lowess") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "lowess assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("did") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("did assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("drdid") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "drdid assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("dml") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("dml assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("cfregress") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "cfregress assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("lincom") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "lincom assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("test") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "test assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("histogram") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "histogram assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("scatter") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "scatter assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("bar") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("bar assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("bayesplot") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new(
      "bayesplot assignment requires a target before =",
    ));
  }
  if name.eq_ignore_ascii_case("help") && !is_command_whitespace(delimiter) {
    return Err(ParseError::new("unknown command: help"));
  }
  parse_named_command(name, body)
}

fn parse_named_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let normalized_name = name.to_lowercase();
  match normalized_name.as_str() {
    "help" => parse_help(body),
    "status" => parse_status_command(body),
    "describe" => {
      if body.is_empty() {
        Ok(Command::Describe)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "describe assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else if body.starts_with('-') {
        Err(ParseError::new("unsupported token in command: -"))
      } else if body.starts_with('+') {
        Err(ParseError::new("unsupported token in command: +"))
      } else {
        Err(ParseError::new(
          "describe does not accept arguments, if clauses, or options",
        ))
      }
    }
    "doctor" => {
      if body.is_empty() {
        Ok(Command::Doctor)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "doctor assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else if body.starts_with('-') {
        Err(ParseError::new("unsupported token in command: -"))
      } else if body.starts_with('+') {
        Err(ParseError::new("unsupported token in command: +"))
      } else if body.eq_ignore_ascii_case("if") {
        Err(ParseError::new("missing expression after if"))
      } else {
        Err(ParseError::new(
          "doctor does not accept arguments, if clauses, options, or assignment syntax",
        ))
      }
    }
    "summarize" => parse_summarize_command(body),
    "datasignature" => parse_datasignature_command(body),
    "assert" => parse_assert_command(body),
    "codebook" => parse_codebook_command(body),
    "missing" => parse_missing_command(body),
    "duplicates" => parse_duplicates_command(body),
    "isid" => parse_isid_command(body),
    "keep" => parse_keep_command(body),
    "drop" => parse_drop_command(body),
    "select" => parse_select_command(body),
    "sort" => parse_sort_command(body),
    "gsort" => parse_gsort_command(body),
    "recode" => parse_recode_command(body),
    "encode" => parse_encode_command(body),
    "decode" => parse_decode_command(body),
    "label" => parse_label_command(body),
    "tabulate" => parse_tabulate_command(body),
    "join" => parse_join_command(body),
    "append" => parse_append_command(body),
    "reshape" => parse_reshape_command(body),
    "panel" => parse_panel_command(body),
    "xtdata" => parse_xtdata_command(body),
    "ivregress" => parse_ivregress_command(body),
    "xtreg" => parse_xtreg_command(body),
    "xtabond" => parse_xtabond_command(body),
    "xtlogit" => parse_xtlogit_command(body),
    "lowess" => parse_lowess_command(body),
    "did" => parse_did_command(body),
    "drdid" => parse_drdid_command(body),
    "dml" => parse_dml_command(body),
    "cfregress" => parse_cfregress_command(body),
    "lincom" => parse_lincom_command(body),
    "test" => parse_test_command(body),
    "histogram" => parse_histogram_command(body),
    "scatter" => parse_scatter_command(body),
    "bar" => parse_bar_command(body),
    "bayesplot" => parse_bayesplot_command(body),
    "estat" => parse_estat_command(body),
    "ttest" => parse_ttest_command(body),
    "by" => parse_by_command(body),
    "collapse" => parse_collapse_command(body),
    "rename" => parse_rename_command(body),
    "generate" => parse_generate_command(body),
    "replace" => parse_replace_command(body),
    "run" => parse_run_command(body),
    "set" => parse_set_command(body),
    "save" | "export" => parse_save_export_command(normalized_name.as_str(), body),
    "use" => parse_use_command(body),
    "count" | "head" | "tail" => parse_inspection_command(normalized_name.as_str(), body),
    "sql" => parse_sql_command(body),
    "regress" => parse_regress_command(body),
    "logit" => parse_binary_or_count_response_command("logit", body),
    "probit" => parse_binary_or_count_response_command("probit", body),
    "poisson" => parse_binary_or_count_response_command("poisson", body),
    "nbreg" => parse_binary_or_count_response_command("nbreg", body),
    "zip" => parse_zero_inflated_count_command("zip", body),
    "zinb" => parse_zero_inflated_count_command("zinb", body),
    "qreg" => parse_qreg_command(body),
    "tobit" => parse_tobit_command(body),
    "heckman" => parse_heckman_command(body),
    "nl" => parse_nl_command(body),
    "streg" => parse_streg_command(body),
    "spregress" => parse_spregress_command(body),
    "lasso" => parse_lasso_command(body),
    "postlasso" => parse_postlasso_command(body),
    "ridge" => parse_ridge_command(body),
    "elasticnet" => parse_elasticnet_command(body),
    "cvlasso" => parse_cvlasso_command(body),
    "cvridge" => parse_cvridge_command(body),
    "cvelasticnet" => parse_cvelasticnet_command(body),
    "bayes" => parse_bayes_command(body),
    "predict" => parse_predict_command(body),
    "exit" | "quit" => {
      if body.is_empty() {
        Ok(Command::Exit)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(format!(
          "{normalized_name} assignment requires a target before ="
        )))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else {
        Err(ParseError::new(format!(
          "{normalized_name} does not accept arguments, if clauses, or options"
        )))
      }
    }
    other => Err(ParseError::new(format!("unknown command: {other}"))),
  }
}

fn parse_status_command(body: &str) -> Result<Command, ParseError> {
  if body.is_empty() {
    return Ok(Command::Status);
  }
  if body.starts_with('-') {
    return Err(ParseError::new("unsupported token in command: -"));
  }
  if body.starts_with('+') {
    return Err(ParseError::new("unsupported token in command: +"));
  }
  if body.eq_ignore_ascii_case("if") {
    return Err(ParseError::new("missing expression after if"));
  }
  if body.trim_matches(is_command_whitespace).ends_with(',') {
    return Err(ParseError::new(
      "comma must be followed by at least one option",
    ));
  }
  if body.starts_with('=') && !body.starts_with("==") {
    return Err(ParseError::new(
      "status assignment requires a target before =",
    ));
  }
  if body.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  Err(ParseError::new(
    "status does not accept arguments, if clauses, options, or assignment syntax",
  ))
}

#[derive(Debug)]
struct SimpleArgument {
  text: String,
  backtick_quoted: bool,
}

#[derive(Debug, Default)]
struct SimpleBody {
  arguments: Vec<SimpleArgument>,
  has_options: bool,
  has_assignment: bool,
  assignment_target_missing: bool,
  has_condition: bool,
  missing_condition_expression: bool,
}

fn parse_inspection_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(format!(
      "{name} assignment requires a target before ="
    )));
  }

  if name == "count" {
    if parts.arguments.is_empty()
      && !parts.has_options
      && !parts.has_assignment
      && !parts.has_condition
    {
      return Ok(Command::Count);
    }
    return Err(ParseError::new(
      "count does not accept arguments, if clauses, options, or assignment syntax",
    ));
  }

  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new(format!(
      "{name} does not accept if clauses, options, or assignment syntax"
    )));
  }
  if parts.arguments.len() > 1 {
    return Err(ParseError::new(format!(
      "{name} accepts at most one row limit"
    )));
  }
  let limit = parts
    .arguments
    .first()
    .map(|argument| parse_row_limit(&argument.text, name))
    .transpose()?
    .unwrap_or_default();
  Ok(match name {
    "head" => Command::Head { limit },
    "tail" => Command::Tail { limit },
    _ => unreachable!("parse_inspection_command only handles inspection names"),
  })
}

fn parse_set_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new("set assignment requires a target before ="));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition || parts.arguments.len() != 2
  {
    return Err(ParseError::new("set expects syntax: set name value"));
  }

  let setting_name = parts.arguments[0].text.to_lowercase();
  let name = match (setting_name.as_str(), parts.arguments[0].backtick_quoted) {
    ("graph_format", false) => SettingName::GraphFormat,
    ("artifact_dir", false) => SettingName::ArtifactDir,
    ("graph_open", false) => SettingName::GraphOpen,
    _ => {
      return Err(ParseError::new(format!(
        "unknown setting: {}",
        parts.arguments[0].text
      )));
    }
  };
  Ok(Command::Set {
    name,
    value: parts.arguments[1].text.clone(),
  })
}

fn parse_datasignature_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "datasignature assignment requires a target before =",
    ));
  }
  if parts.arguments.is_empty()
    && !parts.has_options
    && !parts.has_assignment
    && !parts.has_condition
  {
    return Ok(Command::Datasignature);
  }
  Err(ParseError::new(
    "datasignature does not accept arguments, if clauses, options, or assignment syntax",
  ))
}

fn parse_assert_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new("assert expects a boolean expression"));
  }

  let mut depth = 0_i32;
  for token in &tokens {
    match (&token.kind, token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => {
        depth -= 1;
        if depth < 0 {
          return Err(ParseError::new("unsupported token in expression: )"));
        }
      }
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        return Err(ParseError::new("assert does not accept options"));
      }
      (UseTokenKind::Identifier { quoted: false }, name)
        if depth == 0 && name.eq_ignore_ascii_case("if") =>
      {
        return Err(ParseError::new("assert does not accept if clauses"));
      }
      _ => {}
    }
  }
  if depth != 0 {
    return Err(ParseError::new("unsupported token in expression: ("));
  }
  if tokens
    .iter()
    .any(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  {
    return Err(ParseError::new("assert does not accept assignment syntax"));
  }

  let expression = AssertExpressionParser::new(tokens).parse()?;
  Ok(Command::Assert { expression })
}

fn parse_generate_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "generate expects syntax: generate new = expression",
    ));
  }
  if tokens
    .first()
    .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  {
    return Err(ParseError::new(
      "generate assignment requires a target before =",
    ));
  }
  if tokens.first().is_some_and(|token| {
    matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
  }) {
    match tokens.get(1) {
      None => {
        return Err(ParseError::new("missing expression after if"));
      }
      Some(token) if token.kind == UseTokenKind::Symbol && token.text == "," => {
        return Err(ParseError::new("missing expression after if"));
      }
      Some(token) if token.kind == UseTokenKind::Symbol && token.text == "=" => {
        return Err(ParseError::new("unsupported token in expression: ="));
      }
      _ => {
        return Err(ParseError::new(
          "generate does not accept if clauses or options",
        ));
      }
    }
  }

  let Some(equal_index) = tokens
    .iter()
    .position(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  else {
    if let Some(token) = tokens.iter().find(|token| {
      token.kind == UseTokenKind::Symbol
        && matches!(token.text.as_str(), "==" | "+" | "-" | "!" | "@" | ":")
    }) {
      return Err(ParseError::new(format!(
        "unsupported token in command: {}",
        token.text
      )));
    }
    return Err(ParseError::new(
      "generate expects syntax: generate new = expression",
    ));
  };

  let target = tokens[..equal_index].iter().collect::<Vec<_>>();
  if target.len() != 1 || !matches!(target[0].kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(
      "generate expects syntax: generate new = expression",
    ));
  }
  let variable = target[0].text.clone();
  let expression_tokens = &tokens[equal_index + 1..];
  if expression_tokens.is_empty() {
    return Err(ParseError::new(
      "generate assignment requires an expression after =",
    ));
  }

  let mut depth = 0_i32;
  let mut option_start = None;
  for (index, token) in expression_tokens.iter().enumerate() {
    match (&token.kind, token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => depth -= 1,
      (UseTokenKind::Identifier { quoted: false }, name)
        if depth == 0 && name.eq_ignore_ascii_case("if") =>
      {
        return Err(ParseError::new("duplicate if clause"));
      }
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        option_start = Some(index);
        break;
      }
      _ => {}
    }
  }
  let expression_end = option_start.unwrap_or(expression_tokens.len());
  if expression_end == 0 {
    return Err(ParseError::new(
      "generate assignment requires an expression after =",
    ));
  }
  if option_start.is_some_and(|index| index + 1 < expression_tokens.len()) {
    return Err(ParseError::new(
      "generate does not accept if clauses or options",
    ));
  }
  let expression =
    GenerateExpressionParser::new(expression_tokens[..expression_end].to_vec()).parse()?;
  Ok(Command::Generate {
    variable,
    expression,
  })
}

fn parse_replace_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "replace expects syntax: replace existing = expression",
    ));
  }
  if tokens
    .first()
    .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  {
    return Err(ParseError::new(
      "replace assignment requires a target before =",
    ));
  }
  if tokens.first().is_some_and(|token| {
    matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
  }) {
    match tokens.get(1) {
      None => return Err(ParseError::new("missing expression after if")),
      Some(token)
        if token.kind == UseTokenKind::Symbol
          && matches!(token.text.as_str(), "," | "=" | "==") =>
      {
        if token.text == "," {
          return Err(ParseError::new("missing expression after if"));
        }
        return Err(ParseError::new(format!(
          "unsupported token in expression: {}",
          token.text
        )));
      }
      _ => {
        return Err(ParseError::new(
          "replace expects syntax: replace existing = expression",
        ));
      }
    }
  }

  let Some(equal_index) = tokens
    .iter()
    .position(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  else {
    if tokens.last().is_some_and(|token| {
      matches!(token.kind, UseTokenKind::Identifier { quoted: false })
        && token.text.eq_ignore_ascii_case("if")
    }) {
      return Err(ParseError::new("missing expression after if"));
    }
    if let Some(token) = tokens.iter().find(|token| {
      token.kind == UseTokenKind::Symbol
        && matches!(token.text.as_str(), "==" | "+" | "-" | "!" | "@" | ":")
    }) {
      return Err(ParseError::new(format!(
        "unsupported token in command: {}",
        token.text
      )));
    }
    return Err(ParseError::new(
      "replace expects syntax: replace existing = expression",
    ));
  };

  let target = tokens[..equal_index].iter().collect::<Vec<_>>();
  if target.len() != 1 || !matches!(target[0].kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(
      "replace expects syntax: replace existing = expression",
    ));
  }
  let variable = target[0].text.clone();
  let expression_tokens = &tokens[equal_index + 1..];
  if expression_tokens.is_empty() {
    return Err(ParseError::new(
      "replace assignment requires an expression after =",
    ));
  }

  let mut depth = 0_i32;
  let mut expression_end = expression_tokens.len();
  let mut condition_tokens = &expression_tokens[expression_tokens.len()..];
  let mut has_options = false;
  for (index, token) in expression_tokens.iter().enumerate() {
    match (&token.kind, token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => depth -= 1,
      (UseTokenKind::Identifier { quoted: false }, name)
        if depth == 0 && name.eq_ignore_ascii_case("if") =>
      {
        expression_end = index;
        let tail = &expression_tokens[index + 1..];
        let mut tail_depth = 0_i32;
        let mut condition_end = tail.len();
        for (tail_index, tail_token) in tail.iter().enumerate() {
          match (&tail_token.kind, tail_token.text.as_str()) {
            (UseTokenKind::Symbol, "(") => tail_depth += 1,
            (UseTokenKind::Symbol, ")") => tail_depth -= 1,
            (UseTokenKind::Identifier { quoted: false }, name)
              if tail_depth == 0 && name.eq_ignore_ascii_case("if") =>
            {
              return Err(ParseError::new("duplicate if clause"));
            }
            (UseTokenKind::Symbol, ",") if tail_depth == 0 => {
              condition_end = tail_index;
              has_options = true;
              break;
            }
            _ => {}
          }
        }
        condition_tokens = &tail[..condition_end];
        break;
      }
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        expression_end = index;
        has_options = true;
        break;
      }
      _ => {}
    }
  }

  let condition = if condition_tokens.is_empty() {
    None
  } else {
    Some(GenerateExpressionParser::new(condition_tokens.to_vec()).parse()?)
  };
  if expression_end == 0 {
    return Err(ParseError::new(
      "replace assignment requires an expression after =",
    ));
  }
  if has_options {
    return Err(ParseError::new("replace does not accept options"));
  }
  let expression =
    GenerateExpressionParser::new(expression_tokens[..expression_end].to_vec()).parse()?;
  Ok(Command::Replace {
    variable,
    expression,
    condition,
  })
}

struct GenerateExpressionParser {
  tokens: Vec<UseToken>,
  index: usize,
}

impl GenerateExpressionParser {
  fn new(tokens: Vec<UseToken>) -> Self {
    Self { tokens, index: 0 }
  }

  fn parse(mut self) -> Result<GenerateExpression, ParseError> {
    let expression = self.parse_comparison()?;
    if let Some(token) = self.peek() {
      return Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      )));
    }
    Ok(expression)
  }

  fn parse_comparison(&mut self) -> Result<GenerateExpression, ParseError> {
    let mut expression = self.parse_additive()?;
    while let Some(operator) = self.peek().and_then(generate_comparison_operator) {
      self.index += 1;
      if self.peek().is_none() {
        return Err(ParseError::new(format!(
          "incomplete expression after {}",
          generate_operator_text(operator)
        )));
      }
      let right = self.parse_additive()?;
      expression = GenerateExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
    Ok(expression)
  }

  fn parse_additive(&mut self) -> Result<GenerateExpression, ParseError> {
    let mut expression = self.parse_multiplicative()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "+" => Some(GenerateBinaryOperator::Add),
        "-" => Some(GenerateBinaryOperator::Subtract),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      if self.peek().is_none() {
        return Err(ParseError::new(format!(
          "incomplete expression after {}",
          generate_operator_text(operator)
        )));
      }
      let right = self.parse_multiplicative()?;
      expression = GenerateExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_multiplicative(&mut self) -> Result<GenerateExpression, ParseError> {
    let mut expression = self.parse_unary()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "*" => Some(GenerateBinaryOperator::Multiply),
        "/" => Some(GenerateBinaryOperator::Divide),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      if self.peek().is_none() {
        return Err(ParseError::new(format!(
          "incomplete expression after {}",
          generate_operator_text(operator)
        )));
      }
      let right = self.parse_unary()?;
      expression = GenerateExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_unary(&mut self) -> Result<GenerateExpression, ParseError> {
    if self.peek().is_some_and(|token| token.text == "-") {
      self.index += 1;
      if self.peek().is_none() {
        return Err(ParseError::new("incomplete expression after -"));
      }
      return Ok(GenerateExpression::UnaryMinus(Box::new(
        self.parse_unary()?,
      )));
    }
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> Result<GenerateExpression, ParseError> {
    let Some(token) = self.consume() else {
      return Err(ParseError::new("missing expression"));
    };
    match token.kind {
      UseTokenKind::Number => {
        if token.text.parse::<f64>().is_err() {
          return Err(ParseError::new(format!("malformed number: {}", token.text)));
        }
        Ok(GenerateExpression::Number(token.text))
      }
      UseTokenKind::String => Ok(GenerateExpression::String(token.text)),
      UseTokenKind::Identifier { quoted } => {
        if !quoted && token.text.eq_ignore_ascii_case("null") {
          return Ok(GenerateExpression::Null);
        }
        if !quoted
          && self
            .peek()
            .is_some_and(|next| next.kind == UseTokenKind::Symbol && next.text == "(")
        {
          return self.parse_function_call(token.text);
        }
        Ok(GenerateExpression::Identifier(token.text))
      }
      UseTokenKind::Symbol if token.text == "(" => {
        let expression = self.parse_comparison()?;
        let Some(closing) = self.consume() else {
          return Err(ParseError::new("missing closing ) in expression"));
        };
        if closing.kind != UseTokenKind::Symbol || closing.text != ")" {
          return Err(ParseError::new(format!(
            "unsupported token in expression: {}",
            closing.text
          )));
        }
        Ok(expression)
      }
      UseTokenKind::Symbol => Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      ))),
    }
  }

  fn parse_function_call(&mut self, name: String) -> Result<GenerateExpression, ParseError> {
    self.index += 1;
    let mut arguments = Vec::new();
    if self
      .peek()
      .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == ")")
    {
      self.index += 1;
      return Ok(GenerateExpression::FunctionCall { name, arguments });
    }
    loop {
      arguments.push(self.parse_comparison()?);
      let Some(separator) = self.consume() else {
        return Err(ParseError::new(format!(
          "missing closing ) in function call: {name}"
        )));
      };
      if separator.kind == UseTokenKind::Symbol && separator.text == ")" {
        break;
      }
      if separator.kind != UseTokenKind::Symbol || separator.text != "," {
        return Err(ParseError::new(format!(
          "function call {name} arguments must be separated by commas"
        )));
      }
    }
    Ok(GenerateExpression::FunctionCall { name, arguments })
  }

  fn peek(&self) -> Option<&UseToken> {
    self.tokens.get(self.index)
  }

  fn consume(&mut self) -> Option<UseToken> {
    let token = self.tokens.get(self.index).cloned();
    self.index += usize::from(token.is_some());
    token
  }
}

fn generate_comparison_operator(token: &UseToken) -> Option<GenerateBinaryOperator> {
  match token.text.as_str() {
    "==" => Some(GenerateBinaryOperator::Equal),
    "!=" => Some(GenerateBinaryOperator::NotEqual),
    "<" => Some(GenerateBinaryOperator::Less),
    "<=" => Some(GenerateBinaryOperator::LessOrEqual),
    ">" => Some(GenerateBinaryOperator::Greater),
    ">=" => Some(GenerateBinaryOperator::GreaterOrEqual),
    _ => None,
  }
}

fn generate_operator_text(operator: GenerateBinaryOperator) -> &'static str {
  match operator {
    GenerateBinaryOperator::Add => "+",
    GenerateBinaryOperator::Subtract => "-",
    GenerateBinaryOperator::Multiply => "*",
    GenerateBinaryOperator::Divide => "/",
    GenerateBinaryOperator::Equal => "==",
    GenerateBinaryOperator::NotEqual => "!=",
    GenerateBinaryOperator::Less => "<",
    GenerateBinaryOperator::LessOrEqual => "<=",
    GenerateBinaryOperator::Greater => ">",
    GenerateBinaryOperator::GreaterOrEqual => ">=",
  }
}

struct AssertExpressionParser {
  tokens: Vec<UseToken>,
  index: usize,
}

impl AssertExpressionParser {
  fn new(tokens: Vec<UseToken>) -> Self {
    Self { tokens, index: 0 }
  }

  fn parse(mut self) -> Result<AssertExpression, ParseError> {
    let expression = self.parse_comparison()?;
    if let Some(token) = self.peek() {
      return Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      )));
    }
    Ok(expression)
  }

  fn parse_comparison(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_additive()?;
    let Some(operator) = self.peek().and_then(assert_comparison_operator) else {
      return Ok(expression);
    };
    self.index += 1;
    let right = self.parse_additive()?;
    expression = AssertExpression::Binary {
      left: Box::new(expression),
      operator,
      right: Box::new(right),
    };
    if let Some(token) = self.peek()
      && assert_comparison_operator(token).is_some()
    {
      return Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      )));
    }
    Ok(expression)
  }

  fn parse_additive(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_multiplicative()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "+" => Some(AssertBinaryOperator::Add),
        "-" => Some(AssertBinaryOperator::Subtract),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      let right = self.parse_multiplicative()?;
      expression = AssertExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_multiplicative(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_unary()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "*" => Some(AssertBinaryOperator::Multiply),
        "/" => Some(AssertBinaryOperator::Divide),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      let right = self.parse_unary()?;
      expression = AssertExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_unary(&mut self) -> Result<AssertExpression, ParseError> {
    if self.peek().is_some_and(|token| token.text == "-") {
      self.index += 1;
      return Ok(AssertExpression::UnaryMinus(Box::new(self.parse_unary()?)));
    }
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> Result<AssertExpression, ParseError> {
    let Some(token) = self.consume() else {
      return Err(ParseError::new("assert expects a boolean expression"));
    };
    match token.kind {
      UseTokenKind::Number => {
        if token.text.parse::<f64>().is_err() {
          return Err(ParseError::new(format!("malformed number: {}", token.text)));
        }
        Ok(AssertExpression::Number(token.text))
      }
      UseTokenKind::String => Ok(AssertExpression::String(token.text)),
      UseTokenKind::Identifier { quoted } => {
        if !quoted && token.text.eq_ignore_ascii_case("null") {
          Ok(AssertExpression::Null)
        } else {
          Ok(AssertExpression::Identifier(token.text))
        }
      }
      UseTokenKind::Symbol if token.text == "(" => {
        let expression = self.parse_comparison()?;
        let Some(closing) = self.consume() else {
          return Err(ParseError::new("unsupported token in expression: ("));
        };
        if closing.kind != UseTokenKind::Symbol || closing.text != ")" {
          return Err(ParseError::new(format!(
            "unsupported token in expression: {}",
            closing.text
          )));
        }
        Ok(expression)
      }
      UseTokenKind::Symbol => Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      ))),
    }
  }

  fn peek(&self) -> Option<&UseToken> {
    self.tokens.get(self.index)
  }

  fn consume(&mut self) -> Option<UseToken> {
    let token = self.tokens.get(self.index).cloned();
    self.index += usize::from(token.is_some());
    token
  }
}

fn assert_comparison_operator(token: &UseToken) -> Option<AssertBinaryOperator> {
  if token.kind != UseTokenKind::Symbol {
    return None;
  }
  match token.text.as_str() {
    "==" => Some(AssertBinaryOperator::Equal),
    "!=" => Some(AssertBinaryOperator::NotEqual),
    "<" => Some(AssertBinaryOperator::Less),
    "<=" => Some(AssertBinaryOperator::LessOrEqual),
    ">" => Some(AssertBinaryOperator::Greater),
    ">=" => Some(AssertBinaryOperator::GreaterOrEqual),
    _ => None,
  }
}

fn parse_summarize_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "summarize assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "summarize does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "summarize does not accept if clauses or options",
    ));
  }
  Ok(Command::Summarize {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_codebook_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "codebook assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "codebook does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "codebook does not accept if clauses or options",
    ));
  }
  Ok(Command::Codebook {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_missing_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "missing assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new("missing does not accept assignment syntax"));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "missing does not accept if clauses or options",
    ));
  }
  Ok(Command::Missing {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_duplicates_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "duplicates assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "duplicates does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "duplicates does not accept if clauses or options",
    ));
  }

  let mut arguments = parts.arguments;
  if arguments.first().is_some_and(|argument| {
    !argument.backtick_quoted && argument.text.eq_ignore_ascii_case("report")
  }) {
    arguments.remove(0);
  }
  Ok(Command::Duplicates {
    variables: arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_isid_command(body: &str) -> Result<Command, ParseError> {
  let (variable_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(variable_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "isid assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    if variable_body
      .trim_matches(is_command_whitespace)
      .ends_with('=')
    {
      return Err(ParseError::new(
        "isid assignment requires an expression after =",
      ));
    }
    return Err(ParseError::new(
      "isid only accepts a variable list and missok option",
    ));
  }
  if parts.has_condition {
    return Err(ParseError::new(
      "isid only accepts a variable list and missok option",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  if parts.arguments.is_empty() {
    return Err(ParseError::new("isid expects at least one key variable"));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| option.name != "missok")
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "isid unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  if options
    .iter()
    .any(|option| option.name == "missok" && option.value != UseOptionValue::Flag)
  {
    return Err(ParseError::new(
      "isid option missok does not accept a value",
    ));
  }

  Ok(Command::Isid {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
    missok: options.iter().any(|option| option.name == "missok"),
  })
}

fn parse_select_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.has_condition
    && let Some(error) = condition_syntax_error(body)
  {
    return Err(error);
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "select assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "select assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("select only accepts a variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("select expects at least one variable"));
  }
  Ok(Command::Select {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_keep_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  let condition_has_options = parts.has_condition && keep_condition_has_options(body);
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "keep assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "keep assignment requires an expression after =",
    ));
  }
  if condition_has_options || parts.has_options || parts.has_assignment {
    return Err(ParseError::new(
      "keep does not accept options or assignment syntax",
    ));
  }
  if parts.has_condition {
    if parts.arguments.is_empty() {
      return Err(ParseError::new(
        "keep if execution is deferred in the bounded runtime",
      ));
    }
    return Err(ParseError::new(
      "keep cannot combine a variable list with an if clause",
    ));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("keep expects a variable list or if clause"));
  }
  Ok(Command::Keep {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_drop_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  let condition_has_options = parts.has_condition && keep_condition_has_options(body);
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.has_condition
    && let Some(error) = condition_syntax_error(body)
  {
    return Err(error);
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "drop assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "drop assignment requires an expression after =",
    ));
  }
  if condition_has_options || parts.has_options || parts.has_assignment {
    return Err(ParseError::new(
      "drop does not accept options or assignment syntax",
    ));
  }
  if parts.has_condition {
    if parts.arguments.is_empty() {
      return Err(ParseError::new(
        "drop if execution is deferred in the bounded runtime",
      ));
    }
    return Err(ParseError::new(
      "drop cannot combine a variable list with an if clause",
    ));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("drop expects a variable list or if clause"));
  }
  Ok(Command::Drop {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn condition_syntax_error(body: &str) -> Option<ParseError> {
  let tokens = match tokenize_use_options(body) {
    Ok(tokens) => tokens,
    Err(error) => return Some(error),
  };
  let condition_start = tokens.iter().position(|token| {
    matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
  })?;
  let mut depth = 0_i32;
  let mut saw_operand = false;
  let mut last_condition_token = None;
  for token in tokens.into_iter().skip(condition_start + 1) {
    if token.kind == UseTokenKind::Symbol {
      match token.text.as_str() {
        "(" => depth += 1,
        ")" => depth -= 1,
        "," if depth == 0 => break,
        _ => {}
      }
      if depth == 0 && token.text.eq_ignore_ascii_case("if") {
        return Some(ParseError::new("duplicate if clause"));
      }
      last_condition_token = Some(token);
      continue;
    }
    if depth == 0
      && matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
    {
      return Some(ParseError::new("duplicate if clause"));
    }
    saw_operand = true;
    last_condition_token = Some(token);
  }
  let last = last_condition_token?;
  if saw_operand
    && matches!(
      last.text.as_str(),
      "+" | "-" | "*" | "/" | "==" | "!=" | "<" | "<=" | ">" | ">="
    )
  {
    return Some(ParseError::new(format!(
      "incomplete expression after {}",
      last.text
    )));
  }
  None
}

fn keep_condition_has_options(body: &str) -> bool {
  let characters: Vec<char> = body.chars().collect();
  let mut index = 0;
  let mut condition_seen = false;
  while index < characters.len() {
    if matches!(characters[index], '\'' | '"' | '`') {
      let quote = characters[index];
      index += 1;
      while index < characters.len() {
        if characters[index] == quote {
          if quote == '`' && characters.get(index + 1) == Some(&'`') {
            index += 2;
            continue;
          }
          index += 1;
          break;
        }
        index += 1;
      }
      continue;
    }
    if characters[index].is_alphabetic() || characters[index] == '_' {
      let start = index;
      index += 1;
      while index < characters.len()
        && (characters[index].is_alphanumeric() || characters[index] == '_')
      {
        index += 1;
      }
      if !condition_seen
        && characters[start..index]
          .iter()
          .collect::<String>()
          .eq_ignore_ascii_case("if")
      {
        condition_seen = true;
      }
      continue;
    }
    if condition_seen && characters[index] == ',' {
      let mut lookahead = index + 1;
      while characters
        .get(lookahead)
        .is_some_and(|character| is_command_whitespace(*character))
      {
        lookahead += 1;
      }
      return characters.get(lookahead).is_some();
    }
    index += 1;
  }
  false
}

fn parse_sort_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "sort assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "sort assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("sort only accepts a variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("sort expects at least one variable"));
  }
  Ok(Command::Sort {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_recode_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "recode command: missing variable list and rules",
    ));
  }

  let mut depth = 0_i32;
  let mut comma_index = None;
  for (index, token) in tokens.iter().enumerate() {
    match (token.kind.clone(), token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => depth -= 1,
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        comma_index = Some(index);
        break;
      }
      _ => {}
    }
  }

  let (body_tokens, option_tokens) = match comma_index {
    Some(index) => (&tokens[..index], Some(&tokens[index + 1..])),
    None => (&tokens[..], None),
  };
  let target = parse_recode_target(option_tokens)?;

  let Some(first_rule_index) = body_tokens
    .iter()
    .position(|token| token.kind == UseTokenKind::Symbol && token.text == "(")
  else {
    return Err(ParseError::new("recode command: no rules specified"));
  };
  if first_rule_index == 0 {
    return Err(ParseError::new("recode command: no variables specified"));
  }

  let variables = body_tokens[..first_rule_index]
    .iter()
    .map(|token| match &token.kind {
      UseTokenKind::Identifier { .. } => Ok(token.text.clone()),
      _ => Err(ParseError::new(format!(
        "invalid variable name in recode list: {}",
        token.text
      ))),
    })
    .collect::<Result<Vec<_>, _>>()?;

  let mut rules = Vec::new();
  let mut index = first_rule_index;
  while index < body_tokens.len() {
    if body_tokens[index].kind != UseTokenKind::Symbol || body_tokens[index].text != "(" {
      return Err(ParseError::new(format!(
        "expected '(' at start of rule, got: {}",
        body_tokens[index].text
      )));
    }

    let mut rule_depth = 1_i32;
    let mut end = index + 1;
    while end < body_tokens.len() && rule_depth > 0 {
      match (
        body_tokens[end].kind.clone(),
        body_tokens[end].text.as_str(),
      ) {
        (UseTokenKind::Symbol, "(") => rule_depth += 1,
        (UseTokenKind::Symbol, ")") => rule_depth -= 1,
        _ => {}
      }
      if rule_depth > 0 {
        end += 1;
      }
    }
    if rule_depth > 0 {
      return Err(ParseError::new("unterminated rule parenthesis"));
    }

    rules.push(parse_recode_rule(&body_tokens[index + 1..end])?);
    index = end + 1;
  }

  Ok(Command::Recode {
    variables,
    rules,
    target,
  })
}

fn parse_recode_target(tokens: Option<&[UseToken]>) -> Result<RecodeTarget, ParseError> {
  let Some(tokens) = tokens else {
    return Err(ParseError::new(
      "recode command requires either generate() or replace option",
    ));
  };
  if tokens.is_empty() {
    return Err(ParseError::new(
      "comma must be followed by at least one option",
    ));
  }

  let options = parse_use_option_tokens(tokens.to_vec())?;
  let mut generate = None;
  let mut replace = false;
  for option in options {
    let name = option.name.to_ascii_lowercase();
    match name.as_str() {
      "generate" => {
        if generate.is_some() {
          return Err(ParseError::new("recode option specified more than once"));
        }
        let UseOptionValue::Identifiers(variables) = option.value else {
          return Err(ParseError::new(
            "recode generate option expects variable names",
          ));
        };
        generate = Some(variables);
      }
      "replace" => {
        if replace {
          return Err(ParseError::new("recode option specified more than once"));
        }
        if option.value != UseOptionValue::Flag {
          return Err(ParseError::new(
            "recode replace option does not accept a value",
          ));
        }
        replace = true;
      }
      _ => {
        return Err(ParseError::new(format!(
          "recode unsupported option: {name}"
        )));
      }
    }
  }

  match (generate, replace) {
    (Some(variables), false) => Ok(RecodeTarget::Generate { variables }),
    (None, true) => Ok(RecodeTarget::Replace),
    (Some(_), true) => Err(ParseError::new(
      "recode command: cannot specify both generate() and replace",
    )),
    (None, false) => Err(ParseError::new(
      "recode command requires either generate() or replace option",
    )),
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RecodeAtom {
  Value(RecodeValue),
  Min,
  Max,
  Missing,
  NonMissing,
  Else,
  Slash,
}

fn parse_recode_rule(tokens: &[UseToken]) -> Result<RecodeRule, ParseError> {
  let equals = tokens
    .iter()
    .enumerate()
    .filter(|(_, token)| token.kind == UseTokenKind::Symbol && token.text == "=")
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
  let Some(&equal_index) = equals.first() else {
    return Err(ParseError::new(
      "recode rule expects '=' between inputs and output",
    ));
  };
  if equals.len() > 1 {
    return Err(ParseError::new("recode rule contains multiple '=' symbols"));
  }
  let lhs_tokens = &tokens[..equal_index];
  let rhs_tokens = &tokens[equal_index + 1..];
  if lhs_tokens.is_empty() {
    return Err(ParseError::new("recode rule: missing inputs before '='"));
  }
  if rhs_tokens.is_empty() {
    return Err(ParseError::new("recode rule: missing output after '='"));
  }

  let lhs_atoms = parse_recode_atoms(lhs_tokens)?;
  let mut inputs = Vec::new();
  let mut index = 0;
  while index < lhs_atoms.len() {
    if lhs_atoms[index] == RecodeAtom::Slash {
      return Err(ParseError::new("invalid slash positioning in recode rule"));
    }
    if lhs_atoms.get(index + 1) == Some(&RecodeAtom::Slash) {
      let Some(end) = lhs_atoms.get(index + 2) else {
        return Err(ParseError::new("unterminated range in recode rule"));
      };
      let start = recode_range_endpoint(&lhs_atoms[index]).ok_or_else(|| {
        ParseError::new(format!(
          "invalid range start in recode rule: {}",
          recode_atom_text(&lhs_atoms[index])
        ))
      })?;
      let end = recode_range_endpoint(end).ok_or_else(|| {
        ParseError::new(format!(
          "invalid range end in recode rule: {}",
          recode_atom_text(end)
        ))
      })?;
      inputs.push(RecodeInput::Range { start, end });
      index += 3;
      continue;
    }
    inputs.push(recode_input(lhs_atoms[index].clone()));
    index += 1;
  }
  if inputs
    .iter()
    .filter(|input| matches!(input, RecodeInput::Else))
    .count()
    > 0
    && inputs.len() > 1
  {
    return Err(ParseError::new(
      "else rule must not be combined with other inputs",
    ));
  }

  let output_atoms = parse_recode_atoms(rhs_tokens)?;
  if output_atoms.len() != 1 {
    return Err(ParseError::new("recode rule output must be a single value"));
  }
  let output = match output_atoms.into_iter().next().expect("length checked") {
    RecodeAtom::Value(value) => value,
    atom => RecodeValue::Text(recode_atom_text(&atom)),
  };
  Ok(RecodeRule { inputs, output })
}

fn parse_recode_atoms(tokens: &[UseToken]) -> Result<Vec<RecodeAtom>, ParseError> {
  let mut atoms = Vec::new();
  let mut index = 0;
  while index < tokens.len() {
    let token = &tokens[index];
    if token.kind == UseTokenKind::Symbol && token.text == "/" {
      atoms.push(RecodeAtom::Slash);
      index += 1;
      continue;
    }
    if token.kind == UseTokenKind::Symbol
      && matches!(token.text.as_str(), "+" | "-")
      && tokens
        .get(index + 1)
        .is_some_and(|next| next.kind == UseTokenKind::Number)
    {
      let number = format!("{}{}", token.text, tokens[index + 1].text);
      atoms.push(RecodeAtom::Value(RecodeValue::Number(number)));
      index += 2;
      continue;
    }
    atoms.push(match &token.kind {
      UseTokenKind::Number => RecodeAtom::Value(RecodeValue::Number(token.text.clone())),
      UseTokenKind::String => RecodeAtom::Value(RecodeValue::Text(token.text.clone())),
      UseTokenKind::Identifier { .. } => match token.text.to_ascii_lowercase().as_str() {
        "min" => RecodeAtom::Min,
        "max" => RecodeAtom::Max,
        "missing" => RecodeAtom::Missing,
        "nonmissing" => RecodeAtom::NonMissing,
        "else" => RecodeAtom::Else,
        _ => RecodeAtom::Value(RecodeValue::Text(token.text.to_ascii_lowercase())),
      },
      UseTokenKind::Symbol => {
        return Err(ParseError::new(format!(
          "unexpected token in recode rule input: {}",
          token.text
        )));
      }
    });
    index += 1;
  }
  Ok(atoms)
}

fn recode_range_endpoint(atom: &RecodeAtom) -> Option<RecodeRangeEndpoint> {
  match atom {
    RecodeAtom::Min => Some(RecodeRangeEndpoint::Min),
    RecodeAtom::Max => Some(RecodeRangeEndpoint::Max),
    RecodeAtom::Value(RecodeValue::Number(value)) => {
      Some(RecodeRangeEndpoint::Number(value.clone()))
    }
    RecodeAtom::Value(RecodeValue::Text(_))
    | RecodeAtom::Missing
    | RecodeAtom::NonMissing
    | RecodeAtom::Else
    | RecodeAtom::Slash => None,
  }
}

fn recode_input(atom: RecodeAtom) -> RecodeInput {
  match atom {
    RecodeAtom::Value(value) => RecodeInput::Value(value),
    RecodeAtom::Missing => RecodeInput::Missing,
    RecodeAtom::NonMissing => RecodeInput::NonMissing,
    RecodeAtom::Else => RecodeInput::Else,
    RecodeAtom::Min => RecodeInput::Value(RecodeValue::Text("min".to_owned())),
    RecodeAtom::Max => RecodeInput::Value(RecodeValue::Text("max".to_owned())),
    RecodeAtom::Slash => unreachable!("slash is handled before scalar conversion"),
  }
}

fn recode_atom_text(atom: &RecodeAtom) -> String {
  match atom {
    RecodeAtom::Value(RecodeValue::Number(value)) => value.clone(),
    RecodeAtom::Value(RecodeValue::Text(value)) => value.clone(),
    RecodeAtom::Min => "min".to_owned(),
    RecodeAtom::Max => "max".to_owned(),
    RecodeAtom::Missing => "missing".to_owned(),
    RecodeAtom::NonMissing => "nonmissing".to_owned(),
    RecodeAtom::Else => "else".to_owned(),
    RecodeAtom::Slash => "/".to_owned(),
  }
}

fn parse_gsort_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "gsort assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "gsort assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("gsort only accepts a signed variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("gsort expects at least one variable"));
  }

  let mut keys = Vec::with_capacity(parts.arguments.len());
  for argument in parts.arguments {
    let mut variable = argument.text;
    let mut descending = false;
    if !argument.backtick_quoted
      && let Some(prefix) = variable
        .chars()
        .next()
        .filter(|prefix| matches!(prefix, '+' | '-'))
    {
      descending = prefix == '-';
      variable.remove(0);
      if variable.is_empty() {
        return Err(ParseError::new(
          "gsort expects a variable after each direction prefix",
        ));
      }
      if variable
        .chars()
        .next()
        .is_some_and(|prefix| matches!(prefix, '+' | '-'))
      {
        return Err(ParseError::new(
          "gsort keys must use at most one + or - prefix",
        ));
      }
    }
    if variable.is_empty() {
      return Err(ParseError::new(
        "gsort expects a variable after each direction prefix",
      ));
    }
    keys.push(SortKey {
      variable,
      descending,
    });
  }
  Ok(Command::Gsort { keys })
}

fn parse_append_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "append expects syntax: append <table>";
  let parts = parse_simple_body(body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() != 1
  {
    return Err(ParseError::new(syntax));
  }

  let table_name = parts
    .arguments
    .into_iter()
    .next()
    .expect("append arity checked before extracting the table")
    .text;
  validate_named_table_name(&table_name)?;
  Ok(Command::Append { table_name })
}

fn parse_reshape_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "reshape expects syntax: reshape long|wide varlist, i(id_vars) j(name)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let argument_body = argument_body.trim_matches(is_command_whitespace);
  let simple_parts = parse_simple_body(argument_body, false)?;
  if simple_parts.has_condition
    || simple_parts.has_assignment
    || simple_parts.missing_condition_expression
  {
    return Err(ParseError::new(syntax));
  }

  let argument_tokens = tokenize_use_options(argument_body)?;
  if argument_tokens.len() < 2 {
    return Err(ParseError::new(syntax));
  }
  let direction = match &argument_tokens[0].kind {
    UseTokenKind::Identifier { quoted: true } | UseTokenKind::Symbol => None,
    _ => Some(argument_tokens[0].text.to_ascii_lowercase()),
  };
  let direction = match direction.as_deref() {
    Some("long") => ReshapeDirection::Long,
    Some("wide") => ReshapeDirection::Wide,
    _ => return Err(ParseError::new("reshape direction must be long or wide")),
  };

  let variables = argument_tokens[1..]
    .iter()
    .map(|token| {
      if matches!(token.kind, UseTokenKind::Symbol) {
        Err(ParseError::new(syntax))
      } else {
        Ok(token.text.clone())
      }
    })
    .collect::<Result<Vec<_>, _>>()?;
  if variables.iter().enumerate().any(|(index, variable)| {
    variables[..index]
      .iter()
      .any(|previous| previous == variable)
  }) {
    return Err(ParseError::new("reshape variable list contains duplicates"));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .map(|option| option.name.clone())
    .filter(|name| name != "i" && name != "j")
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "reshape unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  let identifiers = reshape_identifier_option(&options, "i")?;
  let Some(identifiers) = identifiers.filter(|identifiers| !identifiers.is_empty()) else {
    return Err(ParseError::new(
      "reshape expects exactly one i(id_vars) option",
    ));
  };
  if identifiers.iter().enumerate().any(|(index, identifier)| {
    identifiers[..index]
      .iter()
      .any(|previous| previous == identifier)
  }) {
    return Err(ParseError::new("reshape variable list contains duplicates"));
  }

  let j_values = reshape_identifier_option(&options, "j")?;
  let Some(j_values) = j_values.filter(|values| values.len() == 1) else {
    return Err(ParseError::new(
      "reshape expects exactly one j(name) option",
    ));
  };
  let j_variable = j_values
    .into_iter()
    .next()
    .expect("reshape j option arity checked before extracting the name");

  if variables
    .iter()
    .any(|variable| identifiers.iter().any(|identifier| identifier == variable))
    || variables.iter().any(|variable| variable == &j_variable)
    || identifiers
      .iter()
      .any(|identifier| identifier == &j_variable)
  {
    return Err(ParseError::new(
      "reshape variables, i(), and j() names must be distinct",
    ));
  }

  Ok(Command::Reshape {
    command: ReshapeCommand {
      direction,
      variables,
      identifiers,
      j_variable,
    },
  })
}

fn parse_panel_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "panel expects syntax: panel [<id_var> <time_var>|clear]";
  let parts = parse_simple_body(body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
  {
    return Err(ParseError::new(syntax));
  }

  if parts.arguments.is_empty() {
    return Ok(Command::Panel {
      command: PanelCommand {
        action: PanelAction::Report,
      },
    });
  }

  let first = &parts.arguments[0];
  if !first.backtick_quoted && first.text.eq_ignore_ascii_case("clear") {
    if parts.arguments.len() == 1 {
      return Ok(Command::Panel {
        command: PanelCommand {
          action: PanelAction::Clear,
        },
      });
    }
    return Err(ParseError::new(syntax));
  }

  if parts.arguments.len() != 2 {
    return Err(ParseError::new(syntax));
  }
  let time_variable = &parts.arguments[1].text;
  if first.text == *time_variable {
    return Err(ParseError::new(
      "panel id and time variables must be distinct",
    ));
  }

  Ok(Command::Panel {
    command: PanelCommand {
      action: PanelAction::Set {
        id_variable: first.text.clone(),
        time_variable: time_variable.clone(),
      },
    },
  })
}

fn parse_xtdata_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "xtdata expects syntax: xtdata <varlist>, within|between";
  let (variable_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(variable_body, false)?;
  if parts.arguments.is_empty()
    || parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| option.name != "within" && option.name != "between")
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "xtdata unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.value != UseOptionValue::Flag {
      return Err(ParseError::new(format!(
        "xtdata option {} does not accept a value",
        option.name
      )));
    }
  }

  let has_within = options.iter().any(|option| option.name == "within");
  let has_between = options.iter().any(|option| option.name == "between");
  if has_within == has_between {
    return Err(ParseError::new(
      "xtdata requires exactly one of within or between",
    ));
  }
  let transform = if has_within {
    XtDataTransform::Within
  } else {
    XtDataTransform::Between
  };

  Ok(Command::XtData {
    command: XtDataCommand {
      variables: parts
        .arguments
        .into_iter()
        .map(|argument| argument.text)
        .collect(),
      transform,
    },
  })
}

fn regress_identifier_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<Vec<String>>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "regress option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::Identifiers(values) => Ok(Some(values.clone())),
    _ => Err(ParseError::new(format!(
      "regress option {name} expects variables"
    ))),
  }
}

fn parse_regress_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "regress expects syntax: regress <y> <xvars>";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "robust" | "cluster" | "noconstant" | "wls" | "gls"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "regress unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "regress option {} does not accept a value",
        option.name
      )));
    }
  }

  let cluster_values = regress_identifier_option(&options, "cluster")?;
  if cluster_values
    .as_ref()
    .is_some_and(|values| values.len() != 1)
  {
    return Err(ParseError::new(
      "regress option cluster expects one variable",
    ));
  }
  let cluster_variable = cluster_values.and_then(|mut values| values.pop());

  let wls_values = regress_identifier_option(&options, "wls")?;
  if wls_values.as_ref().is_some_and(|values| values.len() != 1) {
    return Err(ParseError::new("regress option wls expects one variable"));
  }

  let gls_values = regress_identifier_option(&options, "gls")?;
  if gls_values.as_ref().is_some_and(|values| values.len() != 1) {
    return Err(ParseError::new("regress option gls expects one variable"));
  }

  if wls_values.is_some() && gls_values.is_some() {
    return Err(ParseError::new("regress cannot combine wls and gls"));
  }

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new("regress cannot combine robust and cluster"));
  }

  let (estimator, weight_variable) = if let Some(mut wls) = wls_values {
    (RegressEstimator::Wls, wls.pop())
  } else if let Some(mut gls) = gls_values {
    (RegressEstimator::Gls, gls.pop())
  } else {
    (RegressEstimator::Ols, None)
  };

  Ok(Command::Regress {
    command: RegressCommand {
      outcome: parts.arguments[0].text.clone(),
      predictors: parts.arguments[1..]
        .iter()
        .map(|argument| argument.text.clone())
        .collect(),
      estimator,
      weight_variable,
      robust,
      cluster_variable,
      include_intercept: !options.iter().any(|option| option.name == "noconstant"),
    },
  })
}

fn parse_binary_or_count_response_command(
  command_name: &str,
  body: &str,
) -> Result<Command, ParseError> {
  let syntax = format!("{command_name} expects syntax: {command_name} <y> <xvars>");
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "robust" | "cluster" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "{command_name} unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "{command_name} option {} does not accept a value",
        option.name
      )));
    }
  }

  let cluster_matches = options
    .iter()
    .filter(|option| option.name == "cluster")
    .collect::<Vec<_>>();
  if cluster_matches.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option cluster may only be supplied once"
    )));
  }
  let cluster_values = match cluster_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new(format!(
            "{command_name} option cluster expects one variable"
          )));
        }
        Some(values[0].clone())
      }
      _ => {
        return Err(ParseError::new(format!(
          "{command_name} option cluster expects variables"
        )));
      }
    },
    None => None,
  };

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_values.is_some() {
    return Err(ParseError::new(format!(
      "{command_name} cannot combine robust and cluster"
    )));
  }

  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  match command_name {
    "logit" => Ok(Command::Logit {
      command: LogitCommand {
        outcome,
        predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    "probit" => Ok(Command::Probit {
      command: ProbitCommand {
        outcome,
        predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    "poisson" => Ok(Command::Poisson {
      command: PoissonCommand {
        outcome,
        predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    "nbreg" => Ok(Command::Nbreg {
      command: NbregCommand {
        outcome,
        predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    _ => unreachable!("unsupported binary or count response command: {command_name}"),
  }
}

fn parse_zero_inflated_count_command(
  command_name: &str,
  body: &str,
) -> Result<Command, ParseError> {
  let syntax =
    format!("{command_name} expects syntax: {command_name} <y> <xvars>, inflate(<zvars>)");
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "inflate" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "{command_name} unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "{command_name} option {} does not accept a value",
        option.name
      )));
    }
  }

  let inflate_matches = options
    .iter()
    .filter(|option| option.name == "inflate")
    .collect::<Vec<_>>();
  if inflate_matches.is_empty() {
    return Err(ParseError::new(format!(
      "{command_name} option inflate expects one-or-more variables"
    )));
  }
  if inflate_matches.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option inflate may only be supplied once"
    )));
  }
  let inflate_predictors = match &inflate_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.is_empty() {
        return Err(ParseError::new(format!(
          "{command_name} option inflate expects one-or-more variables"
        )));
      }
      values.clone()
    }
    _ => {
      return Err(ParseError::new(format!(
        "{command_name} option inflate expects variables"
      )));
    }
  };

  let cluster_matches = options
    .iter()
    .filter(|option| option.name == "cluster")
    .collect::<Vec<_>>();
  if cluster_matches.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option cluster may only be supplied once"
    )));
  }
  let cluster_values = match cluster_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new(format!(
            "{command_name} option cluster expects one variable"
          )));
        }
        Some(values[0].clone())
      }
      _ => {
        return Err(ParseError::new(format!(
          "{command_name} option cluster expects variables"
        )));
      }
    },
    None => None,
  };

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_values.is_some() {
    return Err(ParseError::new(format!(
      "{command_name} cannot combine robust and cluster"
    )));
  }

  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  match command_name {
    "zip" => Ok(Command::Zip {
      command: ZipCommand {
        outcome,
        predictors,
        inflate_predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    "zinb" => Ok(Command::Zinb {
      command: ZinbCommand {
        outcome,
        predictors,
        inflate_predictors,
        robust,
        cluster_variable: cluster_values,
        include_intercept,
      },
    }),
    _ => unreachable!("unsupported zero-inflated count command: {command_name}"),
  }
}

fn parse_qreg_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "qreg expects syntax: qreg <y> <xvars>";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "quantile" | "robust" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "qreg unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "qreg option {} does not accept a value",
        option.name
      )));
    }
  }

  let quantile_matches = options
    .iter()
    .filter(|option| option.name == "quantile")
    .collect::<Vec<_>>();
  if quantile_matches.len() > 1 {
    return Err(ParseError::new(
      "qreg option quantile may only be supplied once",
    ));
  }
  let quantile = match quantile_matches.first() {
    Some(option) => {
      let num_str = match &option.value {
        UseOptionValue::Number(text) => text.as_str(),
        UseOptionValue::String(text) => text.as_str(),
        _ => {
          return Err(ParseError::new(
            "qreg option quantile expects a numeric value",
          ));
        }
      };
      let Ok(val) = num_str.parse::<f64>() else {
        return Err(ParseError::new(
          "qreg option quantile expects a numeric value",
        ));
      };
      if val <= 0.0 || val >= 1.0 {
        return Err(ParseError::new(
          "qreg option quantile must be between 0 and 1",
        ));
      }
      num_str.to_owned()
    }
    None => "0.5".to_owned(),
  };

  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();
  let robust = options.iter().any(|option| option.name == "robust");
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Qreg {
    command: QregCommand {
      outcome,
      predictors,
      quantile,
      robust,
      include_intercept,
    },
  })
}

fn parse_tobit_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "ll" | "ul" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "tobit unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "tobit option {} does not accept a value",
        option.name
      )));
    }
  }

  let ll_matches = options
    .iter()
    .filter(|option| option.name == "ll")
    .collect::<Vec<_>>();
  if ll_matches.is_empty() {
    return Err(ParseError::new("tobit option ll expects one numeric value"));
  }
  if ll_matches.len() > 1 {
    return Err(ParseError::new("tobit option ll may only be supplied once"));
  }
  let lower_limit = match &ll_matches[0].value {
    UseOptionValue::Number(text) => {
      if text.parse::<f64>().is_err() {
        return Err(ParseError::new("tobit option ll expects a numeric value"));
      }
      text.clone()
    }
    UseOptionValue::String(text) => {
      if text.parse::<f64>().is_err() {
        return Err(ParseError::new("tobit option ll expects a numeric value"));
      }
      text.clone()
    }
    _ => {
      return Err(ParseError::new("tobit option ll expects a numeric value"));
    }
  };

  let ul_matches = options
    .iter()
    .filter(|option| option.name == "ul")
    .collect::<Vec<_>>();
  if ul_matches.len() > 1 {
    return Err(ParseError::new("tobit option ul may only be supplied once"));
  }
  let upper_limit = match ul_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Number(text) => {
        if text.parse::<f64>().is_err() {
          return Err(ParseError::new("tobit option ul expects a numeric value"));
        }
        Some(text.clone())
      }
      UseOptionValue::String(text) => {
        if text.parse::<f64>().is_err() {
          return Err(ParseError::new("tobit option ul expects a numeric value"));
        }
        Some(text.clone())
      }
      _ => {
        return Err(ParseError::new("tobit option ul expects a numeric value"));
      }
    },
    None => None,
  };

  let cluster_matches = options
    .iter()
    .filter(|option| option.name == "cluster")
    .collect::<Vec<_>>();
  if cluster_matches.len() > 1 {
    return Err(ParseError::new(
      "tobit option cluster may only be supplied once",
    ));
  }
  let cluster_variable = match cluster_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new("tobit option cluster expects one variable"));
        }
        Some(values[0].clone())
      }
      _ => {
        return Err(ParseError::new("tobit option cluster expects variables"));
      }
    },
    None => None,
  };

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new("tobit cannot combine robust and cluster"));
  }

  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Tobit {
    command: TobitCommand {
      outcome,
      predictors,
      lower_limit,
      upper_limit,
      robust,
      cluster_variable,
      include_intercept,
    },
  })
}

fn parse_heckman_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "selectdep" | "select" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "heckman unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "heckman option {} does not accept a value",
        option.name
      )));
    }
  }

  let selectdep_matches = options
    .iter()
    .filter(|option| option.name == "selectdep")
    .collect::<Vec<_>>();
  if selectdep_matches.is_empty() {
    return Err(ParseError::new(
      "heckman option selectdep expects one variable",
    ));
  }
  if selectdep_matches.len() > 1 {
    return Err(ParseError::new(
      "heckman option selectdep may only be supplied once",
    ));
  }
  let selection_dependent = match &selectdep_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.len() != 1 {
        return Err(ParseError::new(
          "heckman option selectdep expects one variable",
        ));
      }
      values[0].clone()
    }
    _ => {
      return Err(ParseError::new(
        "heckman option selectdep expects variables",
      ));
    }
  };

  let select_matches = options
    .iter()
    .filter(|option| option.name == "select")
    .collect::<Vec<_>>();
  if select_matches.is_empty() {
    return Err(ParseError::new(
      "heckman option select expects at least one variable",
    ));
  }
  if select_matches.len() > 1 {
    return Err(ParseError::new(
      "heckman option select may only be supplied once",
    ));
  }
  let selection_predictors = match &select_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.is_empty() {
        return Err(ParseError::new(
          "heckman option select expects at least one variable",
        ));
      }
      values.clone()
    }
    _ => {
      return Err(ParseError::new("heckman option select expects variables"));
    }
  };

  let cluster_matches = options
    .iter()
    .filter(|option| option.name == "cluster")
    .collect::<Vec<_>>();
  if cluster_matches.len() > 1 {
    return Err(ParseError::new(
      "heckman option cluster may only be supplied once",
    ));
  }
  let cluster_variable = match cluster_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new(
            "heckman option cluster expects one variable",
          ));
        }
        Some(values[0].clone())
      }
      _ => {
        return Err(ParseError::new("heckman option cluster expects variables"));
      }
    },
    None => None,
  };

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new("heckman cannot combine robust and cluster"));
  }

  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Heckman {
    command: HeckmanCommand {
      outcome,
      predictors,
      selection_dependent,
      selection_predictors,
      robust,
      cluster_variable,
      include_intercept,
    },
  })
}

fn parse_nl_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)";
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new(syntax));
  }
  if tokens
    .iter()
    .any(|token| token.kind == UseTokenKind::Symbol && token.text == "==")
  {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if tokens
    .first()
    .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  {
    return Err(ParseError::new("nl assignment requires a target before ="));
  }

  let Some(equal_index) = tokens
    .iter()
    .position(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  else {
    return Err(ParseError::new(syntax));
  };

  let target_tokens = &tokens[..equal_index];
  if target_tokens.is_empty() {
    return Err(ParseError::new("nl assignment requires a target before ="));
  }
  if target_tokens.len() != 1 || !matches!(target_tokens[0].kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(syntax));
  }
  let outcome = target_tokens[0].text.clone();

  let remaining = &tokens[equal_index + 1..];
  if remaining.is_empty() {
    return Err(ParseError::new(
      "nl assignment requires an expression after =",
    ));
  }

  let mut depth = 0_i32;
  let mut option_start = None;
  for (index, token) in remaining.iter().enumerate() {
    match (&token.kind, token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => depth -= 1,
      (UseTokenKind::Identifier { quoted: false }, name)
        if depth == 0 && name.eq_ignore_ascii_case("if") =>
      {
        return Err(ParseError::new("duplicate if clause"));
      }
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        option_start = Some(index);
        break;
      }
      _ => {}
    }
  }

  let (expression_tokens, option_tokens) = match option_start {
    Some(index) => (&remaining[..index], &remaining[index + 1..]),
    None => (remaining, &[][..]),
  };

  if expression_tokens.is_empty() {
    return Err(ParseError::new(
      "nl assignment requires an expression after =",
    ));
  }

  let expression = GenerateExpressionParser::new(expression_tokens.to_vec()).parse()?;

  let options = if option_tokens.is_empty() {
    Vec::new()
  } else {
    parse_use_option_tokens(option_tokens.to_vec())?
  };

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "params" | "start" | "robust" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "nl unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "nl option {} does not accept a value",
        option.name
      )));
    }
  }

  let params_matches = options
    .iter()
    .filter(|option| option.name == "params")
    .collect::<Vec<_>>();
  if params_matches.is_empty() {
    return Err(ParseError::new(
      "nl option params expects one-or-more parameter names",
    ));
  }
  if params_matches.len() > 1 {
    return Err(ParseError::new(
      "nl option params may only be supplied once",
    ));
  }
  let parameter_names = match &params_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.is_empty() {
        return Err(ParseError::new(
          "nl option params expects one-or-more parameter names",
        ));
      }
      let mut seen = std::collections::HashSet::new();
      for name in values {
        if !seen.insert(name) {
          return Err(ParseError::new(
            "nl option params must not repeat parameter names",
          ));
        }
      }
      values.clone()
    }
    _ => {
      return Err(ParseError::new("nl option params expects variables"));
    }
  };

  let start_matches = options
    .iter()
    .filter(|option| option.name == "start")
    .collect::<Vec<_>>();
  if start_matches.is_empty() {
    return Err(ParseError::new(
      "nl option start expects one-or-more numeric values",
    ));
  }
  if start_matches.len() > 1 {
    return Err(ParseError::new("nl option start may only be supplied once"));
  }
  let start_values = match &start_matches[0].value {
    UseOptionValue::Numbers(values) => {
      if values.is_empty() {
        return Err(ParseError::new(
          "nl option start expects one-or-more numeric values",
        ));
      }
      for val in values {
        if val.parse::<f64>().is_err() {
          return Err(ParseError::new(
            "nl option start expects one-or-more numeric values",
          ));
        }
      }
      values.clone()
    }
    _ => {
      return Err(ParseError::new("nl option start expects variables"));
    }
  };

  if start_values.len() != parameter_names.len() {
    return Err(ParseError::new(
      "nl option start count must match params count",
    ));
  }

  let robust = options.iter().any(|option| option.name == "robust");
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Nl {
    command: NlCommand {
      outcome,
      expression,
      parameter_names,
      start_values,
      robust,
      include_intercept,
    },
  })
}

fn parse_streg_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "streg assignment requires a target before =",
    ));
  }

  let syntax = "streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "failure" | "dist" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "streg unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "streg option {} does not accept a value",
        option.name
      )));
    }
  }

  let failure_matches = options
    .iter()
    .filter(|option| option.name == "failure")
    .collect::<Vec<_>>();
  if failure_matches.is_empty() {
    return Err(ParseError::new("streg option failure expects one variable"));
  }
  if failure_matches.len() > 1 {
    return Err(ParseError::new(
      "streg option failure may only be supplied once",
    ));
  }
  let failure_variable = match &failure_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.len() != 1 {
        return Err(ParseError::new("streg option failure expects one variable"));
      }
      values[0].clone()
    }
    _ => {
      return Err(ParseError::new("streg option failure expects variables"));
    }
  };

  let dist_matches = options
    .iter()
    .filter(|option| option.name == "dist")
    .collect::<Vec<_>>();
  if dist_matches.is_empty() {
    return Err(ParseError::new("streg option dist expects one value"));
  }
  if dist_matches.len() > 1 {
    return Err(ParseError::new(
      "streg option dist may only be supplied once",
    ));
  }
  let distribution = match &dist_matches[0].value {
    UseOptionValue::Identifiers(values) => {
      if values.len() != 1 {
        return Err(ParseError::new("streg option dist expects one value"));
      }
      match values[0].to_ascii_lowercase().as_str() {
        "weibull" => StregDistribution::Weibull,
        "exponential" => StregDistribution::Exponential,
        _ => {
          return Err(ParseError::new(
            "streg option dist must be weibull or exponential",
          ));
        }
      }
    }
    _ => {
      return Err(ParseError::new("streg option dist expects variables"));
    }
  };

  let cluster_matches = options
    .iter()
    .filter(|option| option.name == "cluster")
    .collect::<Vec<_>>();
  if cluster_matches.len() > 1 {
    return Err(ParseError::new(
      "streg option cluster may only be supplied once",
    ));
  }
  let cluster_variable = match cluster_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new("streg option cluster expects one variable"));
        }
        Some(values[0].clone())
      }
      _ => {
        return Err(ParseError::new("streg option cluster expects variables"));
      }
    },
    None => None,
  };

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new("streg cannot combine robust and cluster"));
  }

  let include_intercept = !options.iter().any(|option| option.name == "noconstant");
  let time_variable = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();

  Ok(Command::Streg {
    command: StregCommand {
      time_variable,
      predictors,
      failure_variable,
      distribution,
      robust,
      cluster_variable,
      include_intercept,
    },
  })
}

fn parse_spregress_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "spregress assignment requires a target before =",
    ));
  }

  let syntax = "spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "coord" | "model" | "knn" | "robust" | "weights" | "id" | "contiguity"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "spregress unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "robust" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "spregress option robust does not accept a value",
      ));
    }
  }

  let has_coord = options.iter().any(|option| option.name == "coord");
  let has_weights = options.iter().any(|option| option.name == "weights");

  if has_coord && has_weights {
    return Err(ParseError::new(
      "spregress option coord and weights are mutually exclusive",
    ));
  } else if !has_coord && !has_weights {
    return Err(ParseError::new(
      "spregress requires either coord() or weights() option",
    ));
  }

  let model_matches = options
    .iter()
    .filter(|option| option.name == "model")
    .collect::<Vec<_>>();
  if model_matches.len() > 1 {
    return Err(ParseError::new(
      "spregress option model may only be supplied once",
    ));
  }
  let model_type = match model_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new("spregress option model expects one value"));
        }
        match values[0].as_str() {
          "lag" => SpregressModelType::Lag,
          "error" => SpregressModelType::Error,
          "sarar" => SpregressModelType::Sarar,
          _ => {
            return Err(ParseError::new(
              "spregress option model must be 'lag', 'error', or 'sarar'",
            ));
          }
        }
      }
      _ => {
        return Err(ParseError::new("spregress option model expects a value"));
      }
    },
    None => SpregressModelType::Lag,
  };

  let (coord_variables, knn, weights_file, id_variable, contiguity) = if has_coord {
    if options.iter().any(|option| option.name == "id") {
      return Err(ParseError::new(
        "spregress option id can only be used with weights() option",
      ));
    }
    if options.iter().any(|option| option.name == "contiguity") {
      return Err(ParseError::new(
        "spregress option contiguity can only be used with weights() option",
      ));
    }

    let coord_matches = options
      .iter()
      .filter(|option| option.name == "coord")
      .collect::<Vec<_>>();
    if coord_matches.len() > 1 {
      return Err(ParseError::new(
        "spregress option coord may only be supplied once",
      ));
    }
    let coord_vars = match &coord_matches[0].value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 2 {
          return Err(ParseError::new(
            "spregress option coord expects exactly two variables representing latitude and longitude coordinates",
          ));
        }
        (values[0].clone(), values[1].clone())
      }
      _ => {
        return Err(ParseError::new("spregress option coord expects variables"));
      }
    };

    let knn_matches = options
      .iter()
      .filter(|option| option.name == "knn")
      .collect::<Vec<_>>();
    if knn_matches.len() > 1 {
      return Err(ParseError::new(
        "spregress option knn may only be supplied once",
      ));
    }
    let knn_val = match knn_matches.first() {
      Some(option) => match &option.value {
        UseOptionValue::Number(text) => {
          let Ok(val) = text.parse::<i64>() else {
            return Err(ParseError::new(
              "spregress option knn expects an integer value",
            ));
          };
          if val < 1 {
            return Err(ParseError::new("spregress option knn must be at least 1"));
          }
          val
        }
        _ => {
          return Err(ParseError::new(
            "spregress option knn expects an integer value",
          ));
        }
      },
      None => 5,
    };

    (Some(coord_vars), Some(knn_val), None, None, None)
  } else {
    if options.iter().any(|option| option.name == "knn") {
      return Err(ParseError::new(
        "spregress option knn/coord can only be used with coord() option",
      ));
    }

    let weights_matches = options
      .iter()
      .filter(|option| option.name == "weights")
      .collect::<Vec<_>>();
    if weights_matches.len() > 1 {
      return Err(ParseError::new(
        "spregress option weights may only be supplied once",
      ));
    }
    let weights_path = match &weights_matches[0].value {
      UseOptionValue::String(path) => path.clone(),
      _ => {
        return Err(ParseError::new("spregress option weights expects a path"));
      }
    };

    let id_matches = options
      .iter()
      .filter(|option| option.name == "id")
      .collect::<Vec<_>>();
    if id_matches.is_empty() {
      return Err(ParseError::new(
        "spregress option id() is required when weights() is specified",
      ));
    }
    if id_matches.len() > 1 {
      return Err(ParseError::new(
        "spregress option id may only be supplied once",
      ));
    }
    let id_var = match &id_matches[0].value {
      UseOptionValue::Identifiers(values) => {
        if values.len() != 1 {
          return Err(ParseError::new("spregress option id expects one value"));
        }
        values[0].clone()
      }
      _ => {
        return Err(ParseError::new("spregress option id expects a value"));
      }
    };

    let contiguity_matches = options
      .iter()
      .filter(|option| option.name == "contiguity")
      .collect::<Vec<_>>();
    if contiguity_matches.len() > 1 {
      return Err(ParseError::new(
        "spregress option contiguity may only be supplied once",
      ));
    }
    let contiguity_val = match contiguity_matches.first() {
      Some(option) => match &option.value {
        UseOptionValue::Identifiers(values) => {
          if values.len() != 1 {
            return Err(ParseError::new(
              "spregress option contiguity expects one value",
            ));
          }
          match values[0].as_str() {
            "queen" => SpregressContiguity::Queen,
            "rook" => SpregressContiguity::Rook,
            _ => {
              return Err(ParseError::new(
                "spregress option contiguity must be 'queen' or 'rook'",
              ));
            }
          }
        }
        _ => {
          return Err(ParseError::new(
            "spregress option contiguity expects a value",
          ));
        }
      },
      None => SpregressContiguity::Queen,
    };

    (
      None,
      None,
      Some(weights_path),
      Some(id_var),
      Some(contiguity_val),
    )
  };

  let robust = options.iter().any(|option| option.name == "robust");
  let outcome = parts.arguments[0].text.clone();
  let predictors = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();

  Ok(Command::Spregress {
    command: SpregressCommand {
      outcome,
      predictors,
      model_type,
      coord_variables,
      knn,
      weights_file,
      id_variable,
      contiguity,
      robust,
    },
  })
}

fn parse_regularized_linear_command(
  command_name: &str,
  body: &str,
) -> Result<(String, Vec<String>, Vec<UseOption>), ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(format!(
      "{command_name} assignment requires a target before ="
    )));
  }

  let syntax = format!("{command_name} expects syntax: {command_name} linear <y> <xvars>");
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 3
  {
    return Err(ParseError::new(syntax));
  }

  let model_spec = &parts.arguments[0];
  if model_spec.backtick_quoted || !model_spec.text.eq_ignore_ascii_case("linear") {
    return Err(ParseError::new(format!(
      "{command_name} model must be linear"
    )));
  }

  let outcome = parts.arguments[1].text.clone();
  let predictors = parts.arguments[2..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect();

  Ok((outcome, predictors, options))
}

fn extract_alpha_option(command_name: &str, options: &[UseOption]) -> Result<String, ParseError> {
  let alpha_matches = options
    .iter()
    .filter(|option| option.name == "alpha")
    .collect::<Vec<_>>();
  if alpha_matches.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option alpha may only be supplied once"
    )));
  }
  match alpha_matches.first() {
    Some(option) => {
      let num_str = match &option.value {
        UseOptionValue::Number(text) => text.as_str(),
        UseOptionValue::String(text) => text.as_str(),
        _ => {
          return Err(ParseError::new(format!(
            "{command_name} option alpha expects a numeric value"
          )));
        }
      };
      let Ok(val) = num_str.parse::<f64>() else {
        return Err(ParseError::new(format!(
          "{command_name} option alpha expects a numeric value"
        )));
      };
      if val <= 0.0 {
        return Err(ParseError::new(format!(
          "{command_name} option alpha must be positive"
        )));
      }
      Ok(num_str.to_owned())
    }
    None => Ok("1.0".to_owned()),
  }
}

fn parse_lasso_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("lasso", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "alpha" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "lasso unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "lasso option noconstant does not accept a value",
      ));
    }
  }

  let alpha = extract_alpha_option("lasso", &options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Lasso {
    command: LassoCommand {
      outcome,
      predictors,
      alpha,
      include_intercept,
    },
  })
}

fn parse_postlasso_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("postlasso", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "alpha" | "robust" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "postlasso unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "postlasso option {} does not accept a value",
        option.name
      )));
    }
  }

  let alpha = extract_alpha_option("postlasso", &options)?;
  let robust = options.iter().any(|option| option.name == "robust");
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Postlasso {
    command: PostlassoCommand {
      outcome,
      predictors,
      alpha,
      robust,
      include_intercept,
    },
  })
}

fn parse_ridge_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("ridge", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "alpha" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "ridge unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "ridge option noconstant does not accept a value",
      ));
    }
  }

  let alpha = extract_alpha_option("ridge", &options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Ridge {
    command: RidgeCommand {
      outcome,
      predictors,
      alpha,
      include_intercept,
    },
  })
}

fn parse_elasticnet_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("elasticnet", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "alpha" | "l1_ratio" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "elasticnet unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "elasticnet option noconstant does not accept a value",
      ));
    }
  }

  let alpha = extract_alpha_option("elasticnet", &options)?;

  let l1_matches = options
    .iter()
    .filter(|option| option.name == "l1_ratio")
    .collect::<Vec<_>>();
  if l1_matches.len() > 1 {
    return Err(ParseError::new(
      "elasticnet option l1_ratio may only be supplied once",
    ));
  }
  let l1_ratio = match l1_matches.first() {
    Some(option) => {
      let num_str = match &option.value {
        UseOptionValue::Number(text) => text.as_str(),
        UseOptionValue::Numbers(values) => {
          if values.len() != 1 {
            return Err(ParseError::new(
              "elasticnet option l1_ratio expects one value",
            ));
          }
          values[0].as_str()
        }
        _ => {
          return Err(ParseError::new(
            "elasticnet option l1_ratio expects a numeric value",
          ));
        }
      };
      let Ok(val) = num_str.parse::<f64>() else {
        return Err(ParseError::new(
          "elasticnet option l1_ratio expects a numeric value",
        ));
      };
      if !(0.0..=1.0).contains(&val) {
        return Err(ParseError::new(
          "elasticnet option l1_ratio must be between 0 and 1 inclusive",
        ));
      }
      num_str.to_owned()
    }
    None => "0.5".to_owned(),
  };

  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Elasticnet {
    command: ElasticnetCommand {
      outcome,
      predictors,
      alpha,
      l1_ratio,
      include_intercept,
    },
  })
}

fn extract_cv_option(command_name: &str, options: &[UseOption]) -> Result<i64, ParseError> {
  let cv_matches = options
    .iter()
    .filter(|option| option.name == "cv")
    .collect::<Vec<_>>();
  if cv_matches.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option cv may only be supplied once"
    )));
  }
  match cv_matches.first() {
    Some(option) => {
      let UseOptionValue::Number(value) = &option.value else {
        return Err(ParseError::new(format!(
          "{command_name} option cv expects an integer value"
        )));
      };
      let parsed = value.parse::<f64>().ok().filter(|val| val.is_finite());
      let Some(parsed) = parsed.filter(|val| val.fract() == 0.0) else {
        return Err(ParseError::new(format!(
          "{command_name} option cv expects an integer value"
        )));
      };
      if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
        return Err(ParseError::new(format!(
          "{command_name} option cv expects an integer value"
        )));
      }
      let parsed = parsed as i64;
      if parsed < 2 {
        return Err(ParseError::new(format!(
          "{command_name} option cv must be at least 2"
        )));
      }
      Ok(parsed)
    }
    None => Ok(5),
  }
}

fn extract_cvelasticnet_l1_ratio(options: &[UseOption]) -> Result<CvelasticnetL1Ratio, ParseError> {
  let l1_matches = options
    .iter()
    .filter(|option| option.name == "l1_ratio")
    .collect::<Vec<_>>();
  if l1_matches.len() > 1 {
    return Err(ParseError::new(
      "cvelasticnet option l1_ratio may only be supplied once",
    ));
  }
  match l1_matches.first() {
    Some(option) => match &option.value {
      UseOptionValue::Number(text) => {
        let Ok(val) = text.parse::<f64>() else {
          return Err(ParseError::new(
            "cvelasticnet option l1_ratio expects a numeric value or list of numeric values",
          ));
        };
        if !(0.0..=1.0).contains(&val) {
          return Err(ParseError::new(
            "cvelasticnet option l1_ratio values must be between 0 and 1 inclusive",
          ));
        }
        Ok(CvelasticnetL1Ratio::Single(text.clone()))
      }
      UseOptionValue::Numbers(values) => {
        if values.is_empty() {
          return Err(ParseError::new(
            "cvelasticnet option l1_ratio cannot be empty",
          ));
        }
        for item in values {
          let Ok(val) = item.parse::<f64>() else {
            return Err(ParseError::new(
              "cvelasticnet option l1_ratio values must be numeric",
            ));
          };
          if !(0.0..=1.0).contains(&val) {
            return Err(ParseError::new(
              "cvelasticnet option l1_ratio values must be between 0 and 1 inclusive",
            ));
          }
        }
        Ok(CvelasticnetL1Ratio::Multiple(values.clone()))
      }
      _ => Err(ParseError::new(
        "cvelasticnet option l1_ratio expects a numeric value or list of numeric values",
      )),
    },
    None => Ok(CvelasticnetL1Ratio::Multiple(vec![
      "0.1".to_owned(),
      "0.5".to_owned(),
      "0.7".to_owned(),
      "0.9".to_owned(),
      "0.95".to_owned(),
      "0.99".to_owned(),
      "1.0".to_owned(),
    ])),
  }
}

fn parse_cvlasso_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("cvlasso", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "cv" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "cvlasso unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "cvlasso option noconstant does not accept a value",
      ));
    }
  }

  let cv = extract_cv_option("cvlasso", &options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Cvlasso {
    command: CvlassoCommand {
      outcome,
      predictors,
      cv,
      include_intercept,
    },
  })
}

fn parse_cvridge_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("cvridge", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "cv" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "cvridge unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "cvridge option noconstant does not accept a value",
      ));
    }
  }

  let cv = extract_cv_option("cvridge", &options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Cvridge {
    command: CvridgeCommand {
      outcome,
      predictors,
      cv,
      include_intercept,
    },
  })
}

fn parse_cvelasticnet_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("cvelasticnet", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "cv" | "l1_ratio" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "cvelasticnet unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "cvelasticnet option noconstant does not accept a value",
      ));
    }
  }

  let cv = extract_cv_option("cvelasticnet", &options)?;
  let l1_ratio = extract_cvelasticnet_l1_ratio(&options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Cvelasticnet {
    command: CvelasticnetCommand {
      outcome,
      predictors,
      cv,
      l1_ratio,
      include_intercept,
    },
  })
}

fn extract_n_iter_option(options: &[UseOption]) -> Result<i64, ParseError> {
  let matches = options
    .iter()
    .filter(|option| option.name == "n_iter")
    .collect::<Vec<_>>();
  if matches.len() > 1 {
    return Err(ParseError::new(
      "bayes option n_iter may only be supplied once",
    ));
  }
  match matches.first() {
    Some(option) => {
      let UseOptionValue::Number(value) = &option.value else {
        return Err(ParseError::new(
          "bayes option n_iter expects an integer value",
        ));
      };
      let parsed = value.parse::<f64>().ok().filter(|val| val.is_finite());
      let Some(parsed) = parsed.filter(|val| val.fract() == 0.0) else {
        return Err(ParseError::new(
          "bayes option n_iter expects an integer value",
        ));
      };
      if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
        return Err(ParseError::new(
          "bayes option n_iter expects an integer value",
        ));
      }
      let parsed = parsed as i64;
      if parsed < 1 {
        return Err(ParseError::new("bayes option n_iter must be at least 1"));
      }
      Ok(parsed)
    }
    None => Ok(300),
  }
}

fn extract_tol_option(options: &[UseOption]) -> Result<String, ParseError> {
  let matches = options
    .iter()
    .filter(|option| option.name == "tol")
    .collect::<Vec<_>>();
  if matches.len() > 1 {
    return Err(ParseError::new(
      "bayes option tol may only be supplied once",
    ));
  }
  match matches.first() {
    Some(option) => {
      let UseOptionValue::Number(value) = &option.value else {
        return Err(ParseError::new("bayes option tol expects a numeric value"));
      };
      let Some(parsed) = value.parse::<f64>().ok().filter(|val| val.is_finite()) else {
        return Err(ParseError::new("bayes option tol expects a numeric value"));
      };
      if parsed <= 0.0 {
        return Err(ParseError::new("bayes option tol must be positive"));
      }
      Ok(value.clone())
    }
    None => Ok("0.001".to_string()),
  }
}

fn parse_bayes_command(body: &str) -> Result<Command, ParseError> {
  let (outcome, predictors, options) = parse_regularized_linear_command("bayes", body)?;

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "n_iter" | "tol" | "noconstant"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "bayes unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noconstant" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "bayes option noconstant does not accept a value",
      ));
    }
  }

  let n_iter = extract_n_iter_option(&options)?;
  let tol = extract_tol_option(&options)?;
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Bayes {
    command: BayesCommand {
      outcome,
      predictors,
      n_iter,
      tol,
      include_intercept,
    },
  })
}

fn parse_predict_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "predict assignment requires a target before =",
    ));
  }

  let syntax = "predict expects syntax: predict <newvar>";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let argument_body = argument_body.trim_matches(is_command_whitespace);
  let simple_parts = parse_simple_body(argument_body, false)?;
  if simple_parts.has_condition
    || simple_parts.has_assignment
    || simple_parts.missing_condition_expression
    || simple_parts.arguments.len() != 1
  {
    return Err(ParseError::new(syntax));
  }

  let target_variable = simple_parts
    .arguments
    .into_iter()
    .next()
    .expect("argument count verified")
    .text;

  let options = match option_body {
    Some(options_str) => parse_use_options(options_str)?,
    None => Vec::new(),
  };

  let mut unsupported = options
    .iter()
    .filter(|opt| {
      !matches!(
        opt.name.as_str(),
        "xb"
          | "residuals"
          | "pr"
          | "spatial_lag"
          | "posterior_predictive"
          | "interval"
          | "level"
          | "std"
          | "saving"
      )
    })
    .map(|opt| opt.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "predict unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(
      option.name.as_str(),
      "xb" | "residuals" | "pr" | "spatial_lag" | "posterior_predictive" | "interval" | "std"
    ) && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "predict option {} does not accept a value",
        option.name
      )));
    }
  }

  let mut kinds_present = Vec::new();
  for opt in &options {
    if matches!(
      opt.name.as_str(),
      "xb" | "residuals" | "pr" | "spatial_lag" | "posterior_predictive"
    ) && !kinds_present.contains(&opt.name.as_str())
    {
      kinds_present.push(opt.name.as_str());
    }
  }
  if kinds_present.len() > 1 {
    return Err(ParseError::new(
      "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
    ));
  }

  let kind = if options.iter().any(|opt| opt.name == "residuals") {
    PredictKind::Residuals
  } else if options.iter().any(|opt| opt.name == "pr") {
    PredictKind::Pr
  } else if options.iter().any(|opt| opt.name == "spatial_lag") {
    PredictKind::SpatialLag
  } else if options.iter().any(|opt| opt.name == "posterior_predictive") {
    PredictKind::PosteriorPredictive
  } else {
    PredictKind::Xb
  };

  let interval = options.iter().any(|opt| opt.name == "interval");

  let level_options = options
    .iter()
    .filter(|opt| opt.name == "level")
    .collect::<Vec<_>>();
  if level_options.len() > 1 {
    return Err(ParseError::new(
      "predict option level may only be supplied once",
    ));
  }
  let level_opt = level_options.into_iter().next();
  let (level_str, level_supplied) = match level_opt {
    Some(opt) => match &opt.value {
      UseOptionValue::Flag => {
        return Err(ParseError::new(
          "predict option level expects a numeric value",
        ));
      }
      UseOptionValue::Number(num) => (num.clone(), true),
      _ => {
        return Err(ParseError::new(
          "predict option level expects a numeric value",
        ));
      }
    },
    None => ("95.0".to_string(), false),
  };

  let saving_options = options
    .iter()
    .filter(|opt| opt.name == "saving")
    .collect::<Vec<_>>();
  if saving_options.len() > 1 {
    return Err(ParseError::new(
      "predict option saving may only be supplied once",
    ));
  }
  let saving_opt = saving_options.into_iter().next();
  let saving = match saving_opt {
    Some(opt) => match &opt.value {
      UseOptionValue::Flag => {
        return Err(ParseError::new("predict option saving expects a path"));
      }
      UseOptionValue::String(path) => Some(path.clone()),
      _ => return Err(ParseError::new("predict option saving expects a path")),
    },
    None => None,
  };

  if (interval || level_supplied) && kind != PredictKind::PosteriorPredictive {
    return Err(ParseError::new(
      "predict interval options require posterior_predictive",
    ));
  }
  if level_supplied && !interval {
    return Err(ParseError::new("predict option level requires interval"));
  }
  if level_supplied {
    let parsed_level = level_str
      .parse::<f64>()
      .map_err(|_| ParseError::new("predict option level expects a numeric value"))?;
    if parsed_level <= 0.0 || parsed_level >= 100.0 {
      return Err(ParseError::new(
        "predict option level must be between 0 and 100",
      ));
    }
  }

  let std = options.iter().any(|opt| opt.name == "std");

  if (std || saving.is_some()) && kind != PredictKind::PosteriorPredictive {
    return Err(ParseError::new(
      "predict std and saving options require posterior_predictive",
    ));
  }

  if saving.is_some() && (std || interval) {
    return Err(ParseError::new(
      "predict saving option cannot be combined with std or interval options",
    ));
  }

  Ok(Command::Predict {
    command: PredictCommand {
      target_variable,
      kind,
      interval,
      level: level_str,
      std,
      saving,
    },
  })
}

fn parse_bayes_prefix_command(command: &str) -> Result<Command, ParseError> {
  let Some(colon_index) = first_unquoted_colon(command) else {
    return Err(ParseError::new(
      "bayes prefix expects syntax: bayes [, options]: command",
    ));
  };

  let before = command[..colon_index].trim_matches(is_command_whitespace);
  let after = command[colon_index + 1..].trim_matches(is_command_whitespace);

  let (draws, burnin, chains, thin, seed, priors) = if before.eq_ignore_ascii_case("bayes") {
    (None, None, None, None, None, Vec::new())
  } else {
    if !before.to_ascii_lowercase().starts_with("bayes") {
      return Err(ParseError::new("invalid bayes prefix"));
    }
    let options_part = before[5..].trim_matches(is_command_whitespace);
    if !options_part.starts_with(',') {
      return Err(ParseError::new(
        "bayes prefix options must start with a comma",
      ));
    }

    let parsed_options = parse_use_options(options_part[1..].trim_matches(is_command_whitespace))?;

    let mut draws = None;
    let mut burnin = None;
    let mut chains = None;
    let mut thin = None;
    let mut seed = None;
    let mut priors = Vec::new();

    for option in parsed_options {
      match option.name.as_str() {
        "draws" => {
          let value = parse_bayes_numeric_option(&option.name, &option.value, "draws")?;
          draws = Some(value);
        }
        "burnin" | "tune" => {
          let value = parse_bayes_numeric_option(&option.name, &option.value, "burnin")?;
          burnin = Some(value);
        }
        "chains" => {
          let value = parse_bayes_numeric_option(&option.name, &option.value, "chains")?;
          chains = Some(value);
        }
        "thin" => {
          let value = parse_bayes_numeric_option(&option.name, &option.value, "thin")?;
          thin = Some(value);
        }
        "seed" | "rseed" => {
          let value = parse_bayes_numeric_option(&option.name, &option.value, "seed")?;
          seed = Some(value);
        }
        "prior" => match option.value {
          UseOptionValue::Prior(var, dist) => {
            priors.push((var, dist));
          }
          _ => {
            return Err(ParseError::new("prior expects (variable, distribution)"));
          }
        },
        other => {
          return Err(ParseError::new(format!(
            "unsupported bayes option: {other}"
          )));
        }
      }
    }

    (draws, burnin, chains, thin, seed, priors)
  };

  if after.is_empty() {
    return Err(ParseError::new("bayes expects a command after :"));
  }

  let inner_command = parse_command(after)?;
  match &inner_command {
    Command::Regress { .. } | Command::Logit { .. } => {}
    _ => {
      return Err(ParseError::new(
        "bayes prefix only supports regress and logit commands",
      ));
    }
  }

  Ok(Command::BayesPrefix {
    command: BayesPrefixCommand {
      command: Box::new(inner_command),
      draws,
      burnin,
      chains,
      thin,
      seed,
      priors,
    },
  })
}

fn parse_bayes_numeric_option(
  _option_name: &str,
  value: &UseOptionValue,
  error_name: &str,
) -> Result<i64, ParseError> {
  match value {
    UseOptionValue::Number(text) => {
      if let Ok(num) = text.parse::<f64>() {
        Ok(num as i64)
      } else {
        Err(ParseError::new(format!(
          "{error_name} must be a numeric value"
        )))
      }
    }
    UseOptionValue::String(text) => {
      if let Ok(num) = text.parse::<f64>() {
        Ok(num as i64)
      } else {
        Err(ParseError::new(format!(
          "{error_name} must be a numeric value"
        )))
      }
    }
    _ => Err(ParseError::new(format!(
      "{error_name} must be a numeric value"
    ))),
  }
}

fn ivregress_identifier_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<Vec<String>>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "ivregress option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::Identifiers(values) => Ok(Some(values.clone())),
    _ => Err(ParseError::new(format!(
      "ivregress option {name} expects variables"
    ))),
  }
}

fn parse_ivregress_command(body: &str) -> Result<Command, ParseError> {
  let syntax =
    "ivregress expects syntax: ivregress 2sls|gmm <y> [exog_vars], endog(<var>) iv(<vars>)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let estimator = match (
    parts.arguments[0].backtick_quoted,
    parts.arguments[0].text.to_ascii_lowercase().as_str(),
  ) {
    (false, "2sls") => IvEstimator::TwoStageLeastSquares,
    (false, "gmm") => IvEstimator::GeneralizedMethodOfMoments,
    _ => return Err(ParseError::new("ivregress estimator must be 2sls or gmm")),
  };

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "endog" | "iv" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "ivregress unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "ivregress option {} does not accept a value",
        option.name
      )));
    }
  }

  let endog_values = ivregress_identifier_option(&options, "endog")?;
  let Some(endog_values) = endog_values.filter(|values| values.len() == 1) else {
    return Err(ParseError::new(
      "ivregress option endog expects one variable",
    ));
  };
  let endogenous = endog_values
    .into_iter()
    .next()
    .expect("ivregress endog option arity checked before extracting the variable");

  let instrument_values = ivregress_identifier_option(&options, "iv")?;
  let Some(instruments) = instrument_values.filter(|values| !values.is_empty()) else {
    return Err(ParseError::new(
      "ivregress option iv expects at least one variable",
    ));
  };

  let cluster_values = ivregress_identifier_option(&options, "cluster")?;
  if cluster_values
    .as_ref()
    .is_some_and(|values| values.len() != 1)
  {
    return Err(ParseError::new(
      "ivregress option cluster expects one variable",
    ));
  }
  let cluster_variable = cluster_values.and_then(|mut values| values.pop());
  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new(
      "ivregress cannot combine robust and cluster",
    ));
  }

  let outcome = parts.arguments[1].text.clone();
  let exogenous = parts.arguments[2..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect::<Vec<_>>();
  if exogenous.iter().any(|variable| variable == &endogenous) {
    return Err(ParseError::new(
      "ivregress endog variable must not appear in exogenous variables",
    ));
  }

  Ok(Command::IvRegress {
    command: IvRegressCommand {
      outcome,
      exogenous,
      endogenous,
      instruments,
      robust,
      cluster_variable,
      include_intercept: !options.iter().any(|option| option.name == "noconstant"),
      estimator,
    },
  })
}

fn xtreg_identifier_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<Vec<String>>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "xtreg option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::Identifiers(values) => Ok(Some(values.clone())),
    _ => Err(ParseError::new(format!(
      "xtreg option {name} expects variables"
    ))),
  }
}

fn parse_xtreg_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "xtreg expects syntax: xtreg <y> <xvars>, fe|re";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "fe" | "re" | "robust" | "cluster"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "xtreg unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "fe" | "re" | "robust")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "xtreg option {} does not accept a value",
        option.name
      )));
    }
  }

  let cluster_values = xtreg_identifier_option(&options, "cluster")?;
  if cluster_values
    .as_ref()
    .is_some_and(|values| values.len() != 1)
  {
    return Err(ParseError::new("xtreg option cluster expects one variable"));
  }
  let cluster_variable = cluster_values.and_then(|mut values| values.pop());

  let has_fe = options.iter().any(|option| option.name == "fe");
  let has_re = options.iter().any(|option| option.name == "re");
  if has_fe == has_re {
    return Err(ParseError::new("xtreg requires exactly one of fe or re"));
  }

  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new("xtreg cannot combine robust and cluster"));
  }

  Ok(Command::XtReg {
    command: XtRegCommand {
      outcome: parts.arguments[0].text.clone(),
      predictors: parts.arguments[1..]
        .iter()
        .map(|argument| argument.text.clone())
        .collect(),
      estimator: if has_fe {
        XtRegEstimator::FixedEffects
      } else {
        XtRegEstimator::RandomEffects
      },
      robust,
      cluster_variable,
    },
  })
}

fn xtabond_integer_option(
  options: &[UseOption],
  name: &str,
  minimum: i64,
) -> Result<Option<i64>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "xtabond option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };

  let UseOptionValue::Number(value) = &option.value else {
    return Err(ParseError::new(format!(
      "xtabond option {name} expects an integer value"
    )));
  };
  let parsed = value.parse::<f64>().ok().filter(|value| value.is_finite());
  let Some(parsed) = parsed.filter(|value| value.fract() == 0.0) else {
    return Err(ParseError::new(format!(
      "xtabond option {name} expects an integer value"
    )));
  };
  if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
    return Err(ParseError::new(format!(
      "xtabond option {name} expects an integer value"
    )));
  }
  let parsed = parsed as i64;
  if parsed < minimum {
    return Err(ParseError::new(format!(
      "xtabond option {name} must be at least {minimum}"
    )));
  }
  Ok(Some(parsed))
}

fn parse_xtabond_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "xtabond expects syntax: xtabond <y> [xvars] [, robust lags(#) instlag(#)]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.is_empty()
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "robust" | "lags" | "instlag"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "xtabond unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "robust" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "xtabond option robust does not accept a value",
      ));
    }
  }

  let lag_depth = xtabond_integer_option(&options, "lags", 1)?.unwrap_or(1);
  let instrument_lag_start = xtabond_integer_option(&options, "instlag", 2)?.unwrap_or(2);
  if instrument_lag_start <= lag_depth {
    return Err(ParseError::new(
      "xtabond option instlag must be greater than option lags",
    ));
  }

  Ok(Command::XtAbond {
    command: XtAbondCommand {
      outcome: parts.arguments[0].text.clone(),
      predictors: parts.arguments[1..]
        .iter()
        .map(|argument| argument.text.clone())
        .collect(),
      robust: options.iter().any(|option| option.name == "robust"),
      lag_depth,
      instrument_lag_start,
    },
  })
}

fn parse_xtlogit_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "xtlogit assignment requires a target before =",
    ));
  }

  let syntax = "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 2
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "fe" | "robust"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "xtlogit unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "fe" | "robust") && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(format!(
        "xtlogit option {} does not accept a value",
        option.name
      )));
    }
  }

  let has_fe = options.iter().any(|option| option.name == "fe");
  if !has_fe {
    return Err(ParseError::new("xtlogit requires option fe"));
  }

  let robust = options.iter().any(|option| option.name == "robust");

  Ok(Command::XtLogit {
    command: XtLogitCommand {
      outcome: parts.arguments[0].text.clone(),
      predictors: parts.arguments[1..]
        .iter()
        .map(|argument| argument.text.clone())
        .collect(),
      robust,
    },
  })
}

fn parse_lowess_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "lowess assignment requires a target before =",
    ));
  }

  let syntax = "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() != 2
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "gen" | "bandwidth"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "lowess unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  let gen_matches = options
    .iter()
    .filter(|option| option.name == "gen")
    .collect::<Vec<_>>();
  if gen_matches.len() > 1 {
    return Err(ParseError::new(
      "lowess option gen may only be supplied once",
    ));
  }
  let target_variable = match gen_matches.first() {
    None => return Err(ParseError::new("lowess option gen expects one variable")),
    Some(option) => match &option.value {
      UseOptionValue::Identifiers(vars) => {
        if vars.len() != 1 {
          return Err(ParseError::new("lowess option gen expects one variable"));
        }
        vars[0].clone()
      }
      _ => return Err(ParseError::new("lowess option gen expects variables")),
    },
  };

  let bandwidth_matches = options
    .iter()
    .filter(|option| option.name == "bandwidth")
    .collect::<Vec<_>>();
  if bandwidth_matches.len() > 1 {
    return Err(ParseError::new(
      "lowess option bandwidth may only be supplied once",
    ));
  }
  let bandwidth = match bandwidth_matches.first() {
    Some(option) => {
      let num_str = match &option.value {
        UseOptionValue::Number(text) => text.as_str(),
        UseOptionValue::String(text) => text.as_str(),
        UseOptionValue::Identifiers(vars) => {
          if vars.len() != 1 {
            return Err(ParseError::new("lowess option bandwidth expects one value"));
          }
          vars[0].as_str()
        }
        _ => {
          return Err(ParseError::new(
            "lowess option bandwidth expects a numeric value",
          ));
        }
      };
      let Ok(val) = num_str.parse::<f64>() else {
        return Err(ParseError::new(
          "lowess option bandwidth expects a numeric value",
        ));
      };
      if !val.is_finite() || val <= 0.0 || val >= 1.0 {
        return Err(ParseError::new(
          "lowess option bandwidth must be between 0 and 1",
        ));
      }
      num_str.to_string()
    }
    None => (2.0f64 / 3.0f64).to_string(),
  };

  Ok(Command::Lowess {
    command: LowessCommand {
      outcome: parts.arguments[0].text.clone(),
      predictor: parts.arguments[1].text.clone(),
      target_variable,
      bandwidth,
    },
  })
}

fn parse_did_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new("did assignment requires a target before ="));
  }

  let syntax = "did expects syntax: did <y> [controls], treat(<var>) post(<var>)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.is_empty()
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "treat" | "post" | "robust"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "did unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "robust" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new("did option robust does not accept a value"));
    }
  }

  let parse_var_option = |opt_name: &str| -> Result<String, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == opt_name)
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(format!(
        "did option {opt_name} may only be supplied once"
      )));
    }
    let Some(option) = matches.first() else {
      return Err(ParseError::new(format!(
        "did option {opt_name} expects one variable"
      )));
    };
    match &option.value {
      UseOptionValue::Identifiers(vars) => {
        if vars.len() != 1 {
          return Err(ParseError::new(format!(
            "did option {opt_name} expects one variable"
          )));
        }
        Ok(vars[0].clone())
      }
      _ => Err(ParseError::new(format!(
        "did option {opt_name} expects variables"
      ))),
    }
  };

  let treatment_variable = parse_var_option("treat")?;
  let post_variable = parse_var_option("post")?;

  if treatment_variable == post_variable {
    return Err(ParseError::new(
      "did treatment and post variables must be distinct",
    ));
  }

  let outcome = parts.arguments[0].text.clone();
  let controls = parts.arguments[1..]
    .iter()
    .map(|arg| arg.text.clone())
    .collect::<Vec<_>>();

  if treatment_variable == outcome || post_variable == outcome {
    return Err(ParseError::new(
      "did treatment and post variables must differ from outcome",
    ));
  }

  if controls.contains(&treatment_variable) || controls.contains(&post_variable) {
    return Err(ParseError::new(
      "did treatment and post variables must not appear in controls",
    ));
  }

  let robust = options.iter().any(|option| option.name == "robust");

  Ok(Command::Did {
    command: DidCommand {
      outcome,
      controls,
      treatment_variable,
      post_variable,
      robust,
    },
  })
}

fn parse_drdid_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "drdid assignment requires a target before =",
    ));
  }

  let syntax = "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.is_empty()
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "treat" | "post" | "method" | "robust" | "bootstrap" | "seed"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "drdid unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "robust" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "drdid option robust does not accept a value",
      ));
    }
  }

  let parse_var_option = |opt_name: &str| -> Result<String, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == opt_name)
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} may only be supplied once"
      )));
    }
    let Some(option) = matches.first() else {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} expects one variable"
      )));
    };
    match &option.value {
      UseOptionValue::Identifiers(vars) => {
        if vars.len() != 1 {
          return Err(ParseError::new(format!(
            "drdid option {opt_name} expects one variable"
          )));
        }
        Ok(vars[0].clone())
      }
      _ => Err(ParseError::new(format!(
        "drdid option {opt_name} expects variables"
      ))),
    }
  };

  let treatment_variable = parse_var_option("treat")?;
  let post_variable = parse_var_option("post")?;

  if treatment_variable == post_variable {
    return Err(ParseError::new(
      "drdid treatment and post variables must be distinct",
    ));
  }

  let outcome = parts.arguments[0].text.clone();
  let covariates = parts.arguments[1..]
    .iter()
    .map(|arg| arg.text.clone())
    .collect::<Vec<_>>();

  if treatment_variable == outcome || post_variable == outcome {
    return Err(ParseError::new(
      "drdid treatment and post variables must differ from outcome",
    ));
  }

  if covariates.contains(&treatment_variable) || covariates.contains(&post_variable) {
    return Err(ParseError::new(
      "drdid treatment and post variables must not appear in covariates",
    ));
  }

  let method_matches = options
    .iter()
    .filter(|option| option.name == "method")
    .collect::<Vec<_>>();
  if method_matches.len() > 1 {
    return Err(ParseError::new(
      "drdid option method may only be supplied once",
    ));
  }
  let method = match method_matches.first() {
    None => DrDidMethod::Aipw,
    Some(option) => {
      let raw_val = match &option.value {
        UseOptionValue::Identifiers(vars) => {
          if vars.len() != 1 {
            return Err(ParseError::new("drdid option method expects one value"));
          }
          vars[0].as_str()
        }
        UseOptionValue::String(s) => s.as_str(),
        _ => {
          return Err(ParseError::new("drdid option method expects a value"));
        }
      };
      match raw_val {
        "or" => DrDidMethod::Or,
        "ipw" => DrDidMethod::Ipw,
        "aipw" => DrDidMethod::Aipw,
        _ => {
          return Err(ParseError::new(
            "drdid option method must be one of: or, ipw, aipw",
          ));
        }
      }
    }
  };

  let parse_int_option = |opt_name: &str, minimum: i64| -> Result<Option<i64>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == opt_name)
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} may only be supplied once"
      )));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    let UseOptionValue::Number(value) = &option.value else {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} expects an integer value"
      )));
    };
    let parsed = value.parse::<f64>().ok().filter(|val| val.is_finite());
    let Some(parsed) = parsed.filter(|val| val.fract() == 0.0) else {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} expects an integer value"
      )));
    };
    if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} expects an integer value"
      )));
    }
    let parsed = parsed as i64;
    if parsed < minimum {
      return Err(ParseError::new(format!(
        "drdid option {opt_name} must be at least {minimum}"
      )));
    }
    Ok(Some(parsed))
  };

  let bootstrap = parse_int_option("bootstrap", 1)?;
  let seed = parse_int_option("seed", 0)?;

  if seed.is_some() && bootstrap.is_none() {
    return Err(ParseError::new(
      "drdid option seed requires option bootstrap",
    ));
  }

  let robust = options.iter().any(|option| option.name == "robust");

  Ok(Command::DrDid {
    command: DrDidCommand {
      outcome,
      covariates,
      treatment_variable,
      post_variable,
      method,
      robust,
      bootstrap,
      seed,
    },
  })
}

fn parse_dml_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new("dml assignment requires a target before ="));
  }

  let syntax = "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.len() < 3
  {
    return Err(ParseError::new(syntax));
  }

  if parts.arguments[0].backtick_quoted || !parts.arguments[0].text.eq_ignore_ascii_case("linear") {
    return Err(ParseError::new("dml model must be linear"));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "treat" | "folds" | "alpha" | "robust" | "seed" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "dml unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "dml option {} does not accept a value",
        option.name
      )));
    }
  }

  let treat_matches = options
    .iter()
    .filter(|option| option.name == "treat")
    .collect::<Vec<_>>();
  if treat_matches.len() > 1 {
    return Err(ParseError::new(
      "dml option treat may only be supplied once",
    ));
  }
  let Some(treat_option) = treat_matches.first() else {
    return Err(ParseError::new("dml option treat expects one variable"));
  };
  let treatment_variable = match &treat_option.value {
    UseOptionValue::Identifiers(vars) => {
      if vars.len() != 1 {
        return Err(ParseError::new("dml option treat expects one variable"));
      }
      vars[0].clone()
    }
    _ => {
      return Err(ParseError::new("dml option treat expects variables"));
    }
  };

  let outcome = parts.arguments[1].text.clone();
  let controls = parts.arguments[2..]
    .iter()
    .map(|arg| arg.text.clone())
    .collect::<Vec<_>>();

  if treatment_variable == outcome {
    return Err(ParseError::new(
      "dml treatment variable must differ from outcome",
    ));
  }

  if controls.contains(&treatment_variable) {
    return Err(ParseError::new(
      "dml treatment variable must not appear in controls",
    ));
  }

  let parse_int_option = |opt_name: &str, minimum: i64| -> Result<Option<i64>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == opt_name)
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(format!(
        "dml option {opt_name} may only be supplied once"
      )));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    let UseOptionValue::Number(value) = &option.value else {
      return Err(ParseError::new(format!(
        "dml option {opt_name} expects an integer value"
      )));
    };
    let parsed = value.parse::<f64>().ok().filter(|val| val.is_finite());
    let Some(parsed) = parsed.filter(|val| val.fract() == 0.0) else {
      return Err(ParseError::new(format!(
        "dml option {opt_name} expects an integer value"
      )));
    };
    if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
      return Err(ParseError::new(format!(
        "dml option {opt_name} expects an integer value"
      )));
    }
    let parsed = parsed as i64;
    if parsed < minimum {
      return Err(ParseError::new(format!(
        "dml option {opt_name} must be at least {minimum}"
      )));
    }
    Ok(Some(parsed))
  };

  let folds = parse_int_option("folds", 2)?.unwrap_or(5);
  let alpha = extract_alpha_option("dml", &options)?;
  let seed = parse_int_option("seed", 0)?;

  let robust = options.iter().any(|option| option.name == "robust");
  let include_intercept = !options.iter().any(|option| option.name == "noconstant");

  Ok(Command::Dml {
    command: DmlCommand {
      outcome,
      controls,
      treatment_variable,
      folds,
      alpha,
      robust,
      seed,
      include_intercept,
    },
  })
}

fn cfregress_identifier_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<Vec<String>>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "cfregress option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::Identifiers(values) => Ok(Some(values.clone())),
    _ => Err(ParseError::new(format!(
      "cfregress option {name} expects variables"
    ))),
  }
}

fn parse_cfregress_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_start();
  if trimmed.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  if trimmed.starts_with('=') {
    return Err(ParseError::new(
      "cfregress assignment requires a target before =",
    ));
  }

  let syntax = "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(argument_body, false)?;
  if parts.has_condition
    || parts.has_options
    || parts.has_assignment
    || parts.missing_condition_expression
    || parts.arguments.is_empty()
  {
    return Err(ParseError::new(syntax));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .filter(|option| {
      !matches!(
        option.name.as_str(),
        "endog" | "iv" | "robust" | "cluster" | "noconstant"
      )
    })
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "cfregress unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if matches!(option.name.as_str(), "robust" | "noconstant")
      && option.value != UseOptionValue::Flag
    {
      return Err(ParseError::new(format!(
        "cfregress option {} does not accept a value",
        option.name
      )));
    }
  }

  let endog_values = cfregress_identifier_option(&options, "endog")?;
  let Some(endog_values) = endog_values.filter(|values| values.len() == 1) else {
    return Err(ParseError::new(
      "cfregress option endog expects one variable",
    ));
  };
  let endogenous = endog_values
    .into_iter()
    .next()
    .expect("cfregress endog option arity checked before extracting the variable");

  let instrument_values = cfregress_identifier_option(&options, "iv")?;
  let Some(instruments) = instrument_values.filter(|values| !values.is_empty()) else {
    return Err(ParseError::new(
      "cfregress option iv expects at least one variable",
    ));
  };

  let cluster_values = cfregress_identifier_option(&options, "cluster")?;
  if cluster_values
    .as_ref()
    .is_some_and(|values| values.len() != 1)
  {
    return Err(ParseError::new(
      "cfregress option cluster expects one variable",
    ));
  }
  let cluster_variable = cluster_values.and_then(|mut values| values.pop());
  let robust = options.iter().any(|option| option.name == "robust");
  if robust && cluster_variable.is_some() {
    return Err(ParseError::new(
      "cfregress cannot combine robust and cluster",
    ));
  }

  let outcome = parts.arguments[0].text.clone();
  let exogenous = parts.arguments[1..]
    .iter()
    .map(|argument| argument.text.clone())
    .collect::<Vec<_>>();
  if exogenous.iter().any(|variable| variable == &endogenous) {
    return Err(ParseError::new(
      "cfregress endog variable must not appear in exogenous variables",
    ));
  }

  Ok(Command::CfRegress {
    command: CfRegressCommand {
      outcome,
      exogenous,
      endogenous,
      instruments,
      robust,
      cluster_variable,
      include_intercept: !options.iter().any(|option| option.name == "noconstant"),
    },
  })
}

fn parse_estat_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "estat expects syntax: estat <residuals|ovtest|vif|firststage|overid|hausman|endogenous|margins|gof|did|drdid|dml|bayes|spatial|report>";
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "estat assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "estat assignment requires an expression after =",
    ));
  }
  if parts.has_condition || parts.arguments.len() != 1 {
    return Err(ParseError::new(syntax));
  }

  let argument = &parts.arguments[0];
  let normalized_subcommand = argument.text.to_ascii_lowercase();
  let subcommand = match (normalized_subcommand.as_str(), argument.backtick_quoted) {
    ("firststage", false) => EstatSubcommand::FirstStage,
    ("overid", false) => EstatSubcommand::Overid,
    ("endogenous", false) => EstatSubcommand::Endogenous,
    ("hausman", false) => EstatSubcommand::Hausman,
    _ => {
      return Err(ParseError::new(
        "estat subcommand must be residuals, ovtest, vif, firststage, overid, hausman, endogenous, margins, gof, did, drdid, dml, bayes, spatial, or report",
      ));
    }
  };
  if parts.has_options {
    return Err(ParseError::new(format!(
      "estat {} does not support options",
      normalized_subcommand
    )));
  }

  Ok(Command::Estat {
    command: EstatCommand { subcommand },
  })
}

fn parse_lincom_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_matches(is_command_whitespace);
  if trimmed.is_empty() {
    return Err(ParseError::new(
      "lincom command expects a linear combination expression",
    ));
  }

  let tokens = tokenize(trimmed)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "lincom command expects a linear combination expression",
    ));
  }

  let expression = GenerateExpressionParser::new(tokens).parse()?;
  Ok(Command::Lincom {
    command: LincomCommand { expression },
  })
}

fn parse_test_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_matches(is_command_whitespace);
  if trimmed.is_empty() {
    return Err(ParseError::new(
      "test command expects a list of variables or constraints",
    ));
  }

  let tokens = tokenize(trimmed)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "test command expects a list of variables or constraints",
    ));
  }

  let has_parens = tokens
    .iter()
    .any(|t| matches!(t.kind, TokenKind::Symbol) && t.text == "(");
  let mut constraints = Vec::new();

  if has_parens {
    let mut groups: Vec<Vec<Token>> = Vec::new();
    let mut current_group: Vec<Token> = Vec::new();
    let mut depth: isize = 0;

    for token in tokens {
      if matches!(token.kind, TokenKind::Symbol) && token.text == "(" {
        if depth == 0 {
          current_group.clear();
        } else {
          current_group.push(token);
        }
        depth += 1;
      } else if matches!(token.kind, TokenKind::Symbol) && token.text == ")" {
        depth -= 1;
        if depth == 0 {
          groups.push(std::mem::take(&mut current_group));
        } else {
          current_group.push(token);
        }
      } else if depth > 0 {
        current_group.push(token);
      } else {
        return Err(ParseError::new(
          "test command: unexpected tokens outside parentheses",
        ));
      }
    }

    if depth != 0 {
      return Err(ParseError::new("test command: mismatched parentheses"));
    }

    for group_tokens in groups {
      if group_tokens.is_empty() {
        return Err(ParseError::new(
          "test command: empty constraint inside parentheses",
        ));
      }
      constraints.push(parse_single_constraint(group_tokens)?);
    }
  } else {
    let equal_indices: Vec<usize> = tokens
      .iter()
      .enumerate()
      .filter(|(_, t)| matches!(t.kind, TokenKind::Symbol) && (t.text == "=" || t.text == "=="))
      .map(|(i, _)| i)
      .collect();

    if !equal_indices.is_empty() {
      if equal_indices.len() > 1 {
        return Err(ParseError::new(
          "test command: multiple '=' in a single constraint (use parentheses for multiple constraints)",
        ));
      }
      let eq_idx = equal_indices[0];
      let lhs_tokens = tokens[..eq_idx].to_vec();
      let rhs_tokens = tokens[eq_idx + 1..].to_vec();
      if lhs_tokens.is_empty() {
        return Err(ParseError::new(
          "test command: missing left-hand side of constraint",
        ));
      }
      if rhs_tokens.is_empty() {
        return Err(ParseError::new(
          "test command: missing right-hand side of constraint",
        ));
      }
      let lhs = GenerateExpressionParser::new(lhs_tokens).parse()?;
      let rhs = GenerateExpressionParser::new(rhs_tokens).parse()?;
      constraints.push(GenerateExpression::Binary {
        left: Box::new(lhs),
        operator: GenerateBinaryOperator::Subtract,
        right: Box::new(rhs),
      });
    } else {
      for token in tokens {
        if !matches!(token.kind, TokenKind::Identifier { .. }) {
          return Err(ParseError::new(format!(
            "test command: expected variable name, got '{}'",
            token.text
          )));
        }
        constraints.push(GenerateExpression::Identifier(token.text));
      }
    }
  }

  Ok(Command::Test {
    command: TestCommand { constraints },
  })
}

fn parse_single_constraint(tokens: Vec<Token>) -> Result<GenerateExpression, ParseError> {
  let equal_indices: Vec<usize> = tokens
    .iter()
    .enumerate()
    .filter(|(_, t)| matches!(t.kind, TokenKind::Symbol) && (t.text == "=" || t.text == "=="))
    .map(|(i, _)| i)
    .collect();

  if !equal_indices.is_empty() {
    if equal_indices.len() > 1 {
      return Err(ParseError::new(
        "test command: multiple '=' in a constraint",
      ));
    }
    let eq_idx = equal_indices[0];
    let lhs_tokens = tokens[..eq_idx].to_vec();
    let rhs_tokens = tokens[eq_idx + 1..].to_vec();
    if lhs_tokens.is_empty() || rhs_tokens.is_empty() {
      return Err(ParseError::new("test command: malformed constraint"));
    }
    let lhs = GenerateExpressionParser::new(lhs_tokens).parse()?;
    let rhs = GenerateExpressionParser::new(rhs_tokens).parse()?;
    Ok(GenerateExpression::Binary {
      left: Box::new(lhs),
      operator: GenerateBinaryOperator::Subtract,
      right: Box::new(rhs),
    })
  } else {
    GenerateExpressionParser::new(tokens).parse()
  }
}

fn parse_histogram_command(body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "histogram assignment requires a target before =",
    ));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "histogram assignment requires an expression after =",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(
      "histogram does not accept if clauses or assignment syntax",
    ));
  }
  if parts.arguments.len() != 1 {
    return Err(ParseError::new("histogram expects exactly one variable"));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "bins" | "saving" | "noopen"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "histogram unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noopen" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "histogram option noopen does not accept a value",
      ));
    }
  }

  let parse_bins_option = || -> Result<Option<i64>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == "bins")
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(
        "histogram option bins may only be supplied once",
      ));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    let parsed = match &option.value {
      UseOptionValue::Number(value) => {
        let parsed = value.parse::<f64>().ok().filter(|val| val.is_finite());
        let Some(parsed) = parsed.filter(|val| val.fract() == 0.0) else {
          return Err(ParseError::new(
            "histogram option bins expects an integer value",
          ));
        };
        if parsed < i64::MIN as f64 || parsed > i64::MAX as f64 {
          return Err(ParseError::new(
            "histogram option bins expects an integer value",
          ));
        }
        parsed as i64
      }
      UseOptionValue::Identifiers(ids) => {
        if ids.len() != 1 {
          return Err(ParseError::new(
            "histogram option bins expects one integer value",
          ));
        }
        if !ids[0].is_ascii() || !ids[0].chars().all(|c| c.is_ascii_digit()) {
          return Err(ParseError::new(
            "histogram option bins expects an integer value",
          ));
        }
        let Ok(val) = ids[0].parse::<i64>() else {
          return Err(ParseError::new(
            "histogram option bins expects an integer value",
          ));
        };
        val
      }
      _ => {
        return Err(ParseError::new(
          "histogram option bins expects an integer value",
        ));
      }
    };
    if parsed < 1 {
      return Err(ParseError::new("histogram option bins must be at least 1"));
    }
    Ok(Some(parsed))
  };

  let parse_saving_option = || -> Result<Option<String>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == "saving")
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(
        "histogram option saving may only be supplied once",
      ));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    match &option.value {
      UseOptionValue::String(s) => Ok(Some(s.clone())),
      _ => Err(ParseError::new("histogram option saving expects a path")),
    }
  };

  let bins = parse_bins_option()?;
  let saving = parse_saving_option()?;
  let open_artifact = !options.iter().any(|option| option.name == "noopen");

  let variable = parts.arguments.into_iter().next().unwrap().text;

  Ok(Command::Histogram {
    command: HistogramCommand {
      variable,
      bins,
      saving,
      open_artifact,
    },
  })
}

fn parse_scatter_command(body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "scatter assignment requires a target before =",
    ));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "scatter assignment requires an expression after =",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(
      "scatter does not accept if clauses or assignment syntax",
    ));
  }
  if parts.arguments.len() != 2 {
    return Err(ParseError::new(
      "scatter expects syntax: scatter y_var x_var",
    ));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "saving" | "noopen"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "scatter unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noopen" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "scatter option noopen does not accept a value",
      ));
    }
  }

  let parse_saving_option = || -> Result<Option<String>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == "saving")
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(
        "scatter option saving may only be supplied once",
      ));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    match &option.value {
      UseOptionValue::String(s) => Ok(Some(s.clone())),
      _ => Err(ParseError::new("scatter option saving expects a path")),
    }
  };

  let saving = parse_saving_option()?;
  let open_artifact = !options.iter().any(|option| option.name == "noopen");

  let mut args = parts.arguments.into_iter();
  let y_variable = args.next().unwrap().text;
  let x_variable = args.next().unwrap().text;

  Ok(Command::Scatter {
    command: ScatterCommand {
      y_variable,
      x_variable,
      saving,
      open_artifact,
    },
  })
}

fn parse_bar_command(body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new("bar assignment requires a target before ="));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "bar assignment requires an expression after =",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(
      "bar does not accept if clauses or assignment syntax",
    ));
  }
  if parts.arguments.len() != 1 {
    return Err(ParseError::new("bar expects exactly one variable"));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "saving" | "missing" | "noopen"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "bar unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "missing" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "bar option missing does not accept a value",
      ));
    }
    if option.name == "noopen" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new("bar option noopen does not accept a value"));
    }
  }

  let parse_saving_option = || -> Result<Option<String>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == "saving")
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(
        "bar option saving may only be supplied once",
      ));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    match &option.value {
      UseOptionValue::String(s) => Ok(Some(s.clone())),
      _ => Err(ParseError::new("bar option saving expects a path")),
    }
  };

  let saving = parse_saving_option()?;
  let include_missing = options.iter().any(|option| option.name == "missing");
  let open_artifact = !options.iter().any(|option| option.name == "noopen");

  let variable = parts.arguments.into_iter().next().unwrap().text;

  Ok(Command::Bar {
    command: BarCommand {
      variable,
      saving,
      include_missing,
      open_artifact,
    },
  })
}

fn parse_bayesplot_command(body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "bayesplot assignment requires a target before =",
    ));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "bayesplot assignment requires an expression after =",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(
      "bayesplot does not accept if clauses or assignment syntax",
    ));
  }
  if parts.arguments.len() != 1 {
    return Err(ParseError::new(
      "bayesplot expects syntax: bayesplot <trace|density|autocorrelation>",
    ));
  }

  let kind_str = &parts.arguments[0].text;
  let kind = match kind_str.as_str() {
    "trace" => BayesPlotKind::Trace,
    "density" => BayesPlotKind::Density,
    "autocorrelation" => BayesPlotKind::Autocorrelation,
    _ => {
      return Err(ParseError::new(
        "bayesplot kind must be trace, density, or autocorrelation",
      ));
    }
  };

  let mut unsupported = options
    .iter()
    .filter(|option| !matches!(option.name.as_str(), "saving" | "noopen"))
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "bayesplot unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  for option in &options {
    if option.name == "noopen" && option.value != UseOptionValue::Flag {
      return Err(ParseError::new(
        "bayesplot option noopen does not accept a value",
      ));
    }
  }

  let parse_saving_option = || -> Result<Option<String>, ParseError> {
    let matches = options
      .iter()
      .filter(|option| option.name == "saving")
      .collect::<Vec<_>>();
    if matches.len() > 1 {
      return Err(ParseError::new(
        "bayesplot option saving may only be supplied once",
      ));
    }
    let Some(option) = matches.first() else {
      return Ok(None);
    };
    match &option.value {
      UseOptionValue::String(s) => Ok(Some(s.clone())),
      _ => Err(ParseError::new("bayesplot option saving expects a path")),
    }
  };

  let saving = parse_saving_option()?;
  let open_artifact = !options.iter().any(|option| option.name == "noopen");

  Ok(Command::BayesPlot {
    command: BayesPlotCommand {
      kind,
      saving,
      open_artifact,
    },
  })
}

fn parse_ttest_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body.trim_matches(is_command_whitespace))?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "ttest command expects a variable comparison or a variable with by() option",
    ));
  }

  let comma_indices = tokens
    .iter()
    .enumerate()
    .filter(|(_, token)| token.kind == UseTokenKind::Symbol && token.text == ",")
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
  if comma_indices.len() > 1 {
    return Err(ParseError::new("ttest command: duplicate comma"));
  }

  if let Some(&comma_index) = comma_indices.first() {
    let variable_tokens = &tokens[..comma_index];
    if variable_tokens.len() != 1
      || !matches!(variable_tokens[0].kind, UseTokenKind::Identifier { .. })
    {
      return Err(ParseError::new(
        "ttest command: expects a single variable name before comma",
      ));
    }

    let options = parse_use_option_tokens(tokens[comma_index + 1..].to_vec())?;
    let by_options = options
      .iter()
      .filter(|option| option.name == "by")
      .collect::<Vec<_>>();
    if by_options.len() > 1 {
      return Err(ParseError::new("ttest option by may only be supplied once"));
    }
    let Some(by_option) = by_options.first() else {
      return Err(ParseError::new(
        "ttest command requires option by(<variable>)",
      ));
    };
    let by_variable = match &by_option.value {
      UseOptionValue::Identifiers(values) if values.len() == 1 => values[0].clone(),
      UseOptionValue::Identifiers(_) => {
        return Err(ParseError::new(
          "ttest command requires option by(<variable>)",
        ));
      }
      _ => return Err(ParseError::new("ttest option by expects variables")),
    };

    let mut unsupported = options
      .iter()
      .filter(|option| !matches!(option.name.as_str(), "by" | "welch" | "unequal"))
      .map(|option| option.name.clone())
      .collect::<Vec<_>>();
    unsupported.sort_unstable();
    unsupported.dedup();
    if !unsupported.is_empty() {
      return Err(ParseError::new(format!(
        "ttest unsupported option: {}",
        unsupported.join(", ")
      )));
    }

    for option in &options {
      if matches!(option.name.as_str(), "welch" | "unequal") && option.value != UseOptionValue::Flag
      {
        return Err(ParseError::new(format!(
          "ttest option {} does not accept a value",
          option.name
        )));
      }
    }

    return Ok(Command::Ttest {
      command: TtestCommand {
        varname1: variable_tokens[0].text.clone(),
        varname2: None,
        value: None,
        by_variable: Some(by_variable),
        welch: options
          .iter()
          .any(|option| matches!(option.name.as_str(), "welch" | "unequal")),
      },
    });
  }

  let comparison_indices = tokens
    .iter()
    .enumerate()
    .filter(|(_, token)| {
      token.kind == UseTokenKind::Symbol && matches!(token.text.as_str(), "=" | "==")
    })
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
  if comparison_indices.is_empty() {
    return Err(ParseError::new(
      "ttest command expects comparison (e.g. ttest var == value) or by() option",
    ));
  }
  if comparison_indices.len() > 1 {
    return Err(ParseError::new("ttest command: multiple comparisons"));
  }

  let comparison_index = comparison_indices[0];
  let left = &tokens[..comparison_index];
  if left.len() != 1 || !matches!(left[0].kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(
      "ttest command: LHS must be a single variable name",
    ));
  }

  let right = &tokens[comparison_index + 1..];
  let (varname2, value) = match right {
    [token] if matches!(token.kind, UseTokenKind::Number) => (None, Some(token.text.clone())),
    [token] if matches!(token.kind, UseTokenKind::Identifier { .. }) => {
      (Some(token.text.clone()), None)
    }
    [token]
      if !matches!(
        token.kind,
        UseTokenKind::Identifier { .. } | UseTokenKind::Number
      ) =>
    {
      return Err(ParseError::new(
        "ttest command: RHS must be a variable name or a numeric value",
      ));
    }
    [sign, token]
      if sign.kind == UseTokenKind::Symbol
        && matches!(sign.text.as_str(), "+" | "-")
        && matches!(token.kind, UseTokenKind::Number) =>
    {
      (None, Some(format!("{}{}", sign.text, token.text)))
    }
    _ => {
      return Err(ParseError::new(
        "ttest command: RHS must be a single variable name or a numeric value",
      ));
    }
  };

  Ok(Command::Ttest {
    command: TtestCommand {
      varname1: left[0].text.clone(),
      varname2,
      value,
      by_variable: None,
      welch: false,
    },
  })
}

fn reshape_identifier_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<Vec<String>>, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name == name)
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "reshape option {name} may only be supplied once"
    )));
  }
  let Some(option) = matching.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::Identifiers(values) => Ok(Some(values.clone())),
    _ => Err(ParseError::new(format!(
      "reshape option {name} expects variables"
    ))),
  }
}

fn parse_join_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "join expects syntax: join <table> on <keylist>";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let argument_body = argument_body.trim_matches(is_command_whitespace);
  let simple_parts = parse_simple_body(argument_body, false)?;
  if simple_parts.has_condition
    || simple_parts.has_assignment
    || simple_parts.missing_condition_expression
  {
    return Err(ParseError::new(syntax));
  }

  let tokens = tokenize_use_options(argument_body)?;
  if tokens.len() < 3 {
    return Err(ParseError::new(syntax));
  }
  if !is_join_argument_token(&tokens[0])
    || !matches!(&tokens[1].kind, UseTokenKind::Identifier { quoted: false })
    || !tokens[1].text.eq_ignore_ascii_case("on")
  {
    return Err(ParseError::new(syntax));
  }

  let table_name = tokens[0].text.clone();
  validate_named_table_name(&table_name)?;
  let keys = tokens[2..]
    .iter()
    .map(|token| {
      if is_join_argument_token(token) {
        Ok(token.text.clone())
      } else {
        Err(ParseError::new(syntax))
      }
    })
    .collect::<Result<Vec<_>, _>>()?;
  if keys
    .iter()
    .enumerate()
    .any(|(index, key)| keys[..index].iter().any(|previous| previous == key))
  {
    return Err(ParseError::new("join key list contains duplicates"));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let mut unsupported = options
    .iter()
    .map(|option| option.name.to_ascii_lowercase())
    .filter(|name| name != "how" && name != "suffix")
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "join unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  let how = match join_single_text_option(&options, "how")?.as_deref() {
    None | Some("inner") => JoinHow::Inner,
    Some("left") => JoinHow::Left,
    Some(_) => return Err(ParseError::new("join how must be inner or left")),
  };
  let suffix = join_single_text_option(&options, "suffix")?.unwrap_or_else(|| "_right".to_owned());
  if suffix.is_empty() {
    return Err(ParseError::new("join suffix cannot be empty"));
  }

  Ok(Command::Join {
    command: JoinCommand {
      table_name,
      keys,
      how,
      suffix,
    },
  })
}

fn is_join_argument_token(token: &UseToken) -> bool {
  !matches!(token.kind, UseTokenKind::Symbol)
}

fn validate_named_table_name(table_name: &str) -> Result<(), ParseError> {
  let mut characters = table_name.chars();
  let valid = characters
    .next()
    .is_some_and(|character| character.is_alphabetic() || character == '_')
    && characters.all(|character| character.is_alphanumeric() || character == '_');
  if !valid {
    return Err(ParseError::new("sql into table name must be an identifier"));
  }
  let normalized = table_name.to_ascii_lowercase();
  if normalized == "active" || normalized.starts_with("__tabdat_") {
    return Err(ParseError::new(format!(
      "sql into cannot use reserved table name: {table_name}"
    )));
  }
  Ok(())
}

fn parse_sql_command(body: &str) -> Result<Command, ParseError> {
  let trimmed = body.trim_matches(is_command_whitespace);
  if trimmed.is_empty() {
    return Err(ParseError::new("sql expects a query"));
  }

  let (query, remainder) = if trimmed.starts_with("\"\"\"") {
    parse_triple_quoted_sql(trimmed)?
  } else {
    split_sql_into(trimmed)?
  };

  let normalized_query = query.trim_matches(is_command_whitespace);
  if normalized_query.is_empty() {
    return Err(ParseError::new("sql expects a query"));
  }

  let into = parse_sql_into_remainder(remainder)?;
  if let Some(ref target) = into {
    validate_named_table_name(target)?;
  }

  Ok(Command::Sql {
    command: SqlCommand {
      query: normalized_query.to_owned(),
      into,
    },
  })
}

fn parse_triple_quoted_sql(body: &str) -> Result<(&str, &str), ParseError> {
  let closing = body[3..]
    .find("\"\"\"")
    .map(|offset| offset + 3)
    .ok_or_else(|| ParseError::new("sql multiline query is missing closing \"\"\""))?;
  let query = &body[3..closing];
  let remainder = body[closing + 3..].trim_matches(is_command_whitespace);
  Ok((query, remainder))
}

fn split_sql_into(body: &str) -> Result<(&str, &str), ParseError> {
  let stripped_body = body.trim_end_matches(is_command_whitespace);
  let words = split_whitespace_words(stripped_body);
  if words
    .last()
    .is_some_and(|last| last.text.eq_ignore_ascii_case("into"))
  {
    return Err(ParseError::new(
      "sql into expects syntax: sql <query> into <table>",
    ));
  }
  if words.len() >= 3 {
    let second_to_last = &words[words.len() - 2];
    if second_to_last.text.eq_ignore_ascii_case("into") {
      let into_start = second_to_last.start;
      let query = stripped_body[..into_start].trim_end_matches(is_command_whitespace);
      let remainder = &stripped_body[into_start..];
      return Ok((query, remainder));
    }
  }
  Ok((body, ""))
}

fn parse_sql_into_remainder(remainder: &str) -> Result<Option<String>, ParseError> {
  let trimmed = remainder.trim_matches(is_command_whitespace);
  if trimmed.is_empty() {
    return Ok(None);
  }
  let parts: Vec<&str> = trimmed
    .split(is_command_whitespace)
    .filter(|part| !part.is_empty())
    .collect();
  if parts.len() != 2 || !parts[0].eq_ignore_ascii_case("into") {
    return Err(ParseError::new(
      "sql into expects syntax: sql <query> into <table>",
    ));
  }
  Ok(Some(parts[1].to_owned()))
}

#[derive(Debug)]
struct WordSpan<'a> {
  text: &'a str,
  start: usize,
  #[allow(dead_code)]
  end: usize,
}

fn split_whitespace_words(text: &str) -> Vec<WordSpan<'_>> {
  let mut spans = Vec::new();
  let mut start = None;
  for (index, character) in text.char_indices() {
    if is_command_whitespace(character) {
      if let Some(word_start) = start.take() {
        spans.push(WordSpan {
          text: &text[word_start..index],
          start: word_start,
          end: index,
        });
      }
    } else if start.is_none() {
      start = Some(index);
    }
  }
  if let Some(word_start) = start {
    spans.push(WordSpan {
      text: &text[word_start..],
      start: word_start,
      end: text.len(),
    });
  }
  spans
}

fn join_single_text_option(
  options: &[UseOption],
  name: &str,
) -> Result<Option<String>, ParseError> {
  let matches = options
    .iter()
    .filter(|option| option.name.eq_ignore_ascii_case(name))
    .collect::<Vec<_>>();
  if matches.len() > 1 {
    return Err(ParseError::new(format!(
      "join option {name} may only be supplied once"
    )));
  }
  let Some(option) = matches.first() else {
    return Ok(None);
  };
  match &option.value {
    UseOptionValue::String(value) => Ok(Some(value.clone())),
    UseOptionValue::Identifiers(values) if values.len() == 1 => Ok(Some(values[0].clone())),
    _ => Err(ParseError::new(format!(
      "join option {name} expects a value"
    ))),
  }
}

fn parse_by_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "by expects syntax: by group_vars: command";
  let Some(colon_index) = first_unquoted_colon(body) else {
    return Err(ParseError::new(syntax));
  };

  let group_body = body[..colon_index].trim_matches(is_command_whitespace);
  let group_tokens = tokenize_use_options(group_body)?;
  if group_tokens.is_empty()
    || group_tokens
      .iter()
      .any(|token| !matches!(token.kind, UseTokenKind::Identifier { .. }))
  {
    return Err(ParseError::new("by expects at least one grouping variable"));
  }
  let groups = group_tokens
    .into_iter()
    .map(|token| token.text)
    .collect::<Vec<_>>();

  let command_body = body[colon_index + 1..].trim_matches(is_command_whitespace);
  if command_body.is_empty() {
    return Err(ParseError::new("by expects a command after :"));
  }
  let command = parse_command(command_body)?;
  match &command {
    Command::By { .. } => return Err(ParseError::new("nested by commands are not supported")),
    Command::Help { .. } => {
      return Err(ParseError::new("help is not supported inside by commands"));
    }
    Command::Status => {
      return Err(ParseError::new(
        "status is not supported inside by commands",
      ));
    }
    Command::Doctor => {
      return Err(ParseError::new(
        "doctor is not supported inside by commands",
      ));
    }
    _ => {}
  }

  Ok(Command::By {
    command: ByCommand {
      groups,
      command: Box::new(command),
    },
  })
}

fn parse_collapse_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "collapse expects syntax: collapse stat varlist, by(group_vars)";
  let Some(comma_index) = first_unquoted_comma(body) else {
    return Err(ParseError::new(
      "collapse expects exactly one by(group_vars) option",
    ));
  };
  let variable_body = body[..comma_index].trim_matches(is_command_whitespace);
  let simple_parts = parse_simple_body(variable_body, false)?;
  if simple_parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if simple_parts.has_condition || simple_parts.has_assignment {
    return Err(ParseError::new(
      "collapse does not accept if clauses or assignment syntax",
    ));
  }
  let variable_tokens = tokenize_use_options(variable_body)?;
  if variable_tokens.len() < 2 {
    return Err(ParseError::new(syntax));
  }

  let statistic_text = variable_tokens[0].text.clone();
  if !matches!(
    &variable_tokens[0].kind,
    UseTokenKind::Identifier { quoted: false }
  ) {
    return Err(ParseError::new(format!(
      "collapse unsupported statistic: {statistic_text}"
    )));
  }
  let statistic = match statistic_text.to_ascii_lowercase().as_str() {
    "count" => CollapseStatistic::Count,
    "mean" => CollapseStatistic::Mean,
    "sum" => CollapseStatistic::Sum,
    "min" => CollapseStatistic::Min,
    "max" => CollapseStatistic::Max,
    _ => {
      return Err(ParseError::new(format!(
        "collapse unsupported statistic: {statistic_text}"
      )));
    }
  };

  let variables = variable_tokens[1..]
    .iter()
    .map(|token| {
      if matches!(&token.kind, UseTokenKind::Identifier { .. }) {
        Ok(token.text.clone())
      } else {
        Err(ParseError::new(syntax))
      }
    })
    .collect::<Result<Vec<_>, _>>()?;

  let options = parse_use_options(&body[comma_index + 1..])?;
  let by_options = options
    .iter()
    .filter(|option| option.name.eq_ignore_ascii_case("by"))
    .collect::<Vec<_>>();
  if options.len() != 1 || by_options.len() != 1 {
    return Err(ParseError::new(
      "collapse expects exactly one by(group_vars) option",
    ));
  }
  let groups = match &by_options[0].value {
    UseOptionValue::Identifiers(groups) if !groups.is_empty() => groups.clone(),
    _ => {
      return Err(ParseError::new(
        "collapse by() expects at least one grouping variable",
      ));
    }
  };

  Ok(Command::Collapse {
    command: CollapseCommand {
      statistic,
      variables,
      groups,
    },
  })
}

fn parse_tabulate_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "tabulate expects one or two variables";
  let (variable_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let variable_tokens = tokenize_use_options(variable_body.trim_matches(is_command_whitespace))?;
  let variables = variable_tokens
    .iter()
    .map(|token| match &token.kind {
      UseTokenKind::Identifier { .. } => Ok(token.text.clone()),
      _ => Err(ParseError::new(syntax)),
    })
    .collect::<Result<Vec<_>, _>>()?;
  if !matches!(variables.len(), 1 | 2) {
    return Err(ParseError::new(syntax));
  }

  for (index, variable) in variables.iter().enumerate() {
    if variables[..index]
      .iter()
      .any(|previous| previous == variable)
    {
      return Err(ParseError::new(format!(
        "tabulate duplicate variable: {variable}"
      )));
    }
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  let supported = ["row", "col", "missing", "nolabel"];
  let mut unsupported = options
    .iter()
    .map(|option| option.name.to_ascii_lowercase())
    .filter(|name| !supported.contains(&name.as_str()))
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "tabulate unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  let mut option_names = Vec::with_capacity(options.len());
  for option in &options {
    let name = option.name.to_ascii_lowercase();
    if option_names.iter().any(|previous| previous == &name) {
      return Err(ParseError::new(format!(
        "tabulate option {name} can only be specified once"
      )));
    }
    if option.value != UseOptionValue::Flag {
      return Err(ParseError::new(format!(
        "tabulate option {name} does not accept a value"
      )));
    }
    option_names.push(name);
  }

  let row_percent = option_names.iter().any(|name| name == "row");
  let column_percent = option_names.iter().any(|name| name == "col");
  if variables.len() == 1 && (row_percent || column_percent) {
    return Err(ParseError::new(
      "tabulate one-way tables do not accept row or col options",
    ));
  }

  Ok(Command::Tabulate {
    command: TabulateCommand {
      row_variables: vec![variables[0].clone()],
      column_variables: variables.get(1).cloned().into_iter().collect(),
      row_percent,
      column_percent,
      include_missing: option_names.iter().any(|name| name == "missing"),
      nolabel: option_names.iter().any(|name| name == "nolabel"),
    },
  })
}

fn parse_label_command(body: &str) -> Result<Command, ParseError> {
  let syntax = "label expects syntax: label variable|define|values|list|drop|save|use ...";
  let (argument_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let tokens = tokenize_use_options(argument_body.trim_matches(is_command_whitespace))?;
  let Some(action_token) = tokens.first() else {
    return Err(ParseError::new(syntax));
  };
  let UseTokenKind::Identifier { quoted: false } = action_token.kind else {
    return Err(ParseError::new(
      "label subcommand must be an unquoted identifier",
    ));
  };
  let action = action_token.text.to_ascii_lowercase();
  let arguments = &tokens[1..];
  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  match action.as_str() {
    "variable" => parse_label_variable(arguments, &options),
    "define" => parse_label_define(arguments, &options),
    "values" => parse_label_values(arguments, &options),
    "list" => {
      if !options.is_empty() {
        return Err(ParseError::new("label list does not accept options"));
      }
      let names = arguments
        .iter()
        .map(|token| label_identifier(token, "label list"))
        .collect::<Result<Vec<_>, _>>()?;
      Ok(Command::Label {
        command: LabelCommand::List { names },
      })
    }
    "drop" => {
      if !options.is_empty() {
        return Err(ParseError::new("label drop does not accept options"));
      }
      if arguments.is_empty() {
        return Err(ParseError::new(
          "label drop expects at least one label set name",
        ));
      }
      let names = arguments
        .iter()
        .map(|token| label_identifier(token, "label drop"))
        .collect::<Result<Vec<_>, _>>()?;
      Ok(Command::Label {
        command: LabelCommand::Drop { names },
      })
    }
    _ => Err(ParseError::new(syntax)),
  }
}

fn parse_label_variable(tokens: &[UseToken], options: &[UseOption]) -> Result<Command, ParseError> {
  let clear = label_flag_option(options, "clear", "label variable")?;
  if options
    .iter()
    .any(|option| !option.name.eq_ignore_ascii_case("clear"))
  {
    return Err(ParseError::new(format!(
      "label variable unsupported option: {}",
      unsupported_label_options(options, &["clear"])
    )));
  }
  if clear {
    if tokens.len() != 1 {
      return Err(ParseError::new(
        "label variable, clear expects syntax: label variable <varname>, clear",
      ));
    }
    return Ok(Command::Label {
      command: LabelCommand::Variable {
        variable: label_identifier(&tokens[0], "label variable")?,
        text: None,
      },
    });
  }
  if tokens.len() != 2 || !matches!(tokens[1].kind, UseTokenKind::String) {
    return Err(ParseError::new(
      "label variable expects syntax: label variable <varname> \"text\"",
    ));
  }
  Ok(Command::Label {
    command: LabelCommand::Variable {
      variable: label_identifier(&tokens[0], "label variable")?,
      text: Some(tokens[1].text.clone()),
    },
  })
}

fn parse_label_define(tokens: &[UseToken], options: &[UseOption]) -> Result<Command, ParseError> {
  if options
    .iter()
    .any(|option| !option.name.eq_ignore_ascii_case("replace"))
  {
    return Err(ParseError::new(format!(
      "label define unsupported option: {}",
      unsupported_label_options(options, &["replace"])
    )));
  }
  let replace = label_flag_option(options, "replace", "label define")?;
  let Some(name_token) = tokens.first() else {
    return Err(ParseError::new(
      "label define expects syntax: label define <lblname> <value> \"text\" [<value> \"text\" ...]",
    ));
  };
  let name = label_identifier(name_token, "label define")?;
  let mut mappings = Vec::new();
  let mut index = 1;
  while index < tokens.len() {
    let (value, consumed) = parse_label_value(&tokens[index..])?;
    index += consumed;
    let Some(text_token) = tokens.get(index) else {
      return Err(ParseError::new("label define text must be a quoted string"));
    };
    if !matches!(text_token.kind, UseTokenKind::String) {
      return Err(ParseError::new("label define text must be a quoted string"));
    }
    index += 1;
    if mappings.iter().any(|(existing, _)| existing == &value) {
      return Err(ParseError::new(format!(
        "label define duplicate value: {}",
        format_label_value(&value)
      )));
    }
    mappings.push((value, text_token.text.clone()));
  }
  if mappings.is_empty() {
    return Err(ParseError::new(
      "label define expects syntax: label define <lblname> <value> \"text\" [<value> \"text\" ...]",
    ));
  }
  Ok(Command::Label {
    command: LabelCommand::Define {
      name,
      mappings,
      replace,
    },
  })
}

fn parse_label_values(tokens: &[UseToken], options: &[UseOption]) -> Result<Command, ParseError> {
  if options
    .iter()
    .any(|option| !option.name.eq_ignore_ascii_case("clear"))
  {
    return Err(ParseError::new(format!(
      "label values unsupported option: {}",
      unsupported_label_options(options, &["clear"])
    )));
  }
  let clear = label_flag_option(options, "clear", "label values")?;
  if tokens.len() != if clear { 1 } else { 2 } {
    if clear {
      return Err(ParseError::new(
        "label values, clear expects syntax: label values <varname>, clear",
      ));
    }
    return Err(ParseError::new(
      "label values expects syntax: label values <varname> <lblname>",
    ));
  }
  let variable = label_identifier(&tokens[0], "label values")?;
  let set_name = if clear {
    None
  } else {
    Some(label_identifier(&tokens[1], "label values")?)
  };
  Ok(Command::Label {
    command: LabelCommand::Values { variable, set_name },
  })
}

fn label_identifier(token: &UseToken, command_name: &str) -> Result<String, ParseError> {
  if !matches!(token.kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(format!(
      "{command_name} expects identifier arguments"
    )));
  }
  Ok(token.text.clone())
}

fn label_flag_option(
  options: &[UseOption],
  name: &str,
  command_name: &str,
) -> Result<bool, ParseError> {
  let matching = options
    .iter()
    .filter(|option| option.name.eq_ignore_ascii_case(name))
    .collect::<Vec<_>>();
  if matching.len() > 1 {
    return Err(ParseError::new(format!(
      "{command_name} option {name} can only be specified once"
    )));
  }
  if let Some(option) = matching.first()
    && option.value != UseOptionValue::Flag
  {
    return Err(ParseError::new(format!(
      "{command_name} option {name} does not accept a value"
    )));
  }
  Ok(!matching.is_empty())
}

fn unsupported_label_options(options: &[UseOption], allowed: &[&str]) -> String {
  let mut names = options
    .iter()
    .filter(|option| {
      !allowed
        .iter()
        .any(|allowed| option.name.eq_ignore_ascii_case(allowed))
    })
    .map(|option| option.name.to_ascii_lowercase())
    .collect::<Vec<_>>();
  names.sort_unstable();
  names.dedup();
  names.join(", ")
}

fn parse_label_value(tokens: &[UseToken]) -> Result<(LabelValue, usize), ParseError> {
  let Some(token) = tokens.first() else {
    return Err(ParseError::new(
      "label define expects a value before each quoted label",
    ));
  };
  match &token.kind {
    UseTokenKind::String | UseTokenKind::Identifier { .. } => {
      Ok((LabelValue::Text(token.text.clone()), 1))
    }
    UseTokenKind::Number => Ok((parse_label_number(&token.text)?, 1)),
    UseTokenKind::Symbol if token.text == "+" || token.text == "-" => {
      let Some(number) = tokens.get(1) else {
        return Err(ParseError::new(
          "label define expects a value before each quoted label",
        ));
      };
      if !matches!(number.kind, UseTokenKind::Number) {
        return Err(ParseError::new(
          "label define expects a value before each quoted label",
        ));
      }
      Ok((
        parse_label_number(&format!("{}{}", token.text, number.text))?,
        2,
      ))
    }
    _ => Err(ParseError::new(
      "label define expects a value before each quoted label",
    )),
  }
}

fn parse_label_number(text: &str) -> Result<LabelValue, ParseError> {
  if text
    .chars()
    .any(|character| matches!(character, '.' | 'e' | 'E'))
  {
    if text.parse::<f64>().is_err() {
      return Err(ParseError::new("label define expects numeric values"));
    }
    return Ok(LabelValue::Number(text.to_owned()));
  }
  text
    .parse::<i64>()
    .map(LabelValue::Integer)
    .map_err(|_| ParseError::new("label define expects numeric values"))
}

fn format_label_value(value: &LabelValue) -> String {
  match value {
    LabelValue::Integer(value) => value.to_string(),
    LabelValue::Number(value) => value.clone(),
    LabelValue::Text(value) => format!("\"{value}\""),
  }
}

fn parse_encode_command(body: &str) -> Result<Command, ParseError> {
  let (source_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let source_tokens = tokenize_use_options(source_body.trim_matches(is_command_whitespace))?;
  let Some(source_token) = source_tokens.first() else {
    return Err(ParseError::new(
      "encode expects syntax: encode <strvar>, generate(<newvar>)",
    ));
  };
  if source_tokens.len() != 1 || !matches!(source_token.kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(
      "encode expects syntax: encode <strvar>, generate(<newvar>)",
    ));
  }

  let Some(option_body) = option_body else {
    return Err(ParseError::new("encode requires generate(<newvar>)"));
  };
  let options = parse_use_options(option_body)?;
  let mut generate = None;
  let mut label = None;
  let mut unsupported = Vec::new();
  for option in options {
    let name = option.name.clone();
    match name.as_str() {
      "generate" => {
        if generate.is_some() {
          return Err(ParseError::new(
            "encode option generate can only be specified once",
          ));
        }
        generate = Some(parse_encode_identifier_option(&name, &option.value)?);
      }
      "label" => {
        if label.is_some() {
          return Err(ParseError::new(
            "encode option label can only be specified once",
          ));
        }
        label = Some(parse_encode_identifier_option(&name, &option.value)?);
      }
      _ => unsupported.push(name),
    }
  }
  if !unsupported.is_empty() {
    unsupported.sort_unstable();
    unsupported.dedup();
    return Err(ParseError::new(format!(
      "encode unsupported option: {}",
      unsupported.join(", ")
    )));
  }
  let Some(generate) = generate else {
    return Err(ParseError::new("encode requires generate(<newvar>)"));
  };
  Ok(Command::Encode {
    source: source_token.text.clone(),
    generate,
    label,
  })
}

fn parse_encode_identifier_option(
  name: &str,
  value: &UseOptionValue,
) -> Result<String, ParseError> {
  let UseOptionValue::Identifiers(values) = value else {
    return Err(ParseError::new(format!(
      "encode option {name} expects identifiers in parentheses"
    )));
  };
  if values.len() != 1 {
    return Err(ParseError::new(format!(
      "encode option {name} expects exactly one variable"
    )));
  }
  Ok(values[0].clone())
}

fn parse_decode_command(body: &str) -> Result<Command, ParseError> {
  let (source_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let source_tokens = tokenize_use_options(source_body.trim_matches(is_command_whitespace))?;
  let Some(source_token) = source_tokens.first() else {
    return Err(ParseError::new(
      "decode expects syntax: decode <numvar>, generate(<newvar>)",
    ));
  };
  if source_tokens.len() != 1 || !matches!(source_token.kind, UseTokenKind::Identifier { .. }) {
    return Err(ParseError::new(
      "decode expects syntax: decode <numvar>, generate(<newvar>)",
    ));
  }

  let Some(option_body) = option_body else {
    return Err(ParseError::new("decode requires generate(<newvar>)"));
  };
  let options = parse_use_options(option_body)?;
  let mut generate = None;
  let mut unsupported = Vec::new();
  for option in options {
    let name = option.name.clone();
    match name.as_str() {
      "generate" => {
        if generate.is_some() {
          return Err(ParseError::new(
            "decode option generate can only be specified once",
          ));
        }
        generate = Some(parse_decode_identifier_option(&name, &option.value)?);
      }
      _ => unsupported.push(name),
    }
  }
  if !unsupported.is_empty() {
    unsupported.sort_unstable();
    unsupported.dedup();
    return Err(ParseError::new(format!(
      "decode unsupported option: {}",
      unsupported.join(", ")
    )));
  }
  let Some(generate) = generate else {
    return Err(ParseError::new("decode requires generate(<newvar>)"));
  };
  Ok(Command::Decode {
    source: source_token.text.clone(),
    generate,
  })
}

fn parse_decode_identifier_option(
  name: &str,
  value: &UseOptionValue,
) -> Result<String, ParseError> {
  let UseOptionValue::Identifiers(values) = value else {
    return Err(ParseError::new(format!(
      "decode option {name} expects identifiers in parentheses"
    )));
  };
  if values.len() != 1 {
    return Err(ParseError::new(format!(
      "decode option {name} expects exactly one variable"
    )));
  }
  Ok(values[0].clone())
}

fn parse_run_command(body: &str) -> Result<Command, ParseError> {
  let mut path_parts = body
    .trim_matches(is_command_whitespace)
    .split(is_command_whitespace)
    .filter(|part| !part.is_empty());
  let Some(path) = path_parts.next() else {
    return Err(ParseError::new(
      "run expects exactly one path: run <script>",
    ));
  };
  if path_parts.next().is_some() {
    return Err(ParseError::new(
      "run expects exactly one path: run <script>",
    ));
  }
  Ok(Command::Run {
    path: path.to_owned(),
  })
}

fn parse_save_export_command(command_name: &str, body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(format!(
      "{command_name} assignment requires a target before ="
    )));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(format!(
      "{command_name} assignment requires an expression after ="
    )));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(format!(
      "{command_name} does not accept if clauses or assignment syntax"
    )));
  }
  if parts.arguments.len() != 1 {
    return Err(ParseError::new(format!(
      "{command_name} expects exactly one path"
    )));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| option.name != "replace")
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "{command_name} unsupported option: {}",
      unsupported.join(", ")
    )));
  }
  if options
    .iter()
    .any(|option| option.name == "replace" && option.value != UseOptionValue::Flag)
  {
    return Err(ParseError::new(format!(
      "{command_name} option replace does not accept a value"
    )));
  }

  let path = parts
    .arguments
    .into_iter()
    .next()
    .expect("save/export arity checked before extracting the path")
    .text;
  let replace = options.iter().any(|option| option.name == "replace");
  Ok(match command_name {
    "save" => Command::Save { path, replace },
    "export" => Command::Export { path, replace },
    _ => unreachable!("save/export parser only handles save and export"),
  })
}

fn parse_rename_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "rename assignment requires a target before =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition || parts.arguments.len() != 2
  {
    return Err(ParseError::new(
      "rename expects exactly two variables: rename old new",
    ));
  }
  let mut arguments = parts.arguments.into_iter();
  let old_name = arguments
    .next()
    .expect("rename arity checked before extracting arguments")
    .text;
  let new_name = arguments
    .next()
    .expect("rename arity checked before extracting arguments")
    .text;
  Ok(Command::Rename { old_name, new_name })
}

fn first_unquoted_comma(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  let mut quote = None;
  let mut index = 0;
  while index < bytes.len() {
    let character = bytes[index];
    if let Some(active_quote) = quote {
      if character == active_quote {
        if active_quote == b'`' && bytes.get(index + 1) == Some(&active_quote) {
          index += 2;
          continue;
        }
        quote = None;
      }
      index += 1;
      continue;
    }
    match character {
      b'\'' | b'"' | b'`' => {
        quote = Some(character);
        index += 1;
      }
      b',' => return Some(index),
      _ => index += 1,
    }
  }
  None
}

fn first_unquoted_colon(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  let mut quote = None;
  let mut index = 0;
  while index < bytes.len() {
    let character = bytes[index];
    if let Some(active_quote) = quote {
      if character == active_quote {
        if active_quote == b'`' && bytes.get(index + 1) == Some(&active_quote) {
          index += 2;
          continue;
        }
        quote = None;
      }
      index += 1;
      continue;
    }
    match character {
      b'\'' | b'"' | b'`' => {
        quote = Some(character);
        index += 1;
      }
      b':' => return Some(index),
      _ => index += 1,
    }
  }
  None
}

type UseTokenKind = TokenKind;
type UseToken = Token;

#[derive(Debug, Clone, PartialEq, Eq)]
enum UseOptionValue {
  Flag,
  String(String),
  Number(String),
  Boolean(bool),
  Identifiers(Vec<String>),
  Numbers(Vec<String>),
  Prior(String, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UseOption {
  name: String,
  value: UseOptionValue,
}

fn parse_use_command(body: &str) -> Result<Command, ParseError> {
  let body = body.trim_matches(is_command_whitespace);
  if body.is_empty() {
    return Err(ParseError::new("use expects exactly one path: use <path>"));
  }

  let (path_text, option_text) = match body.split_once(',') {
    Some((path, options)) => (path, Some(options)),
    None => (body, None),
  };
  let path_parts: Vec<&str> = path_text
    .split(is_command_whitespace)
    .filter(|part| !part.is_empty())
    .collect();
  if path_parts.len() != 1 {
    return Err(ParseError::new("use expects exactly one path: use <path>"));
  }

  let source_text = path_parts[0].to_owned();
  let source = if source_text.contains("://") {
    DataSource::Uri(source_text)
  } else {
    DataSource::LocalPath(source_text)
  };

  let Some(option_text) = option_text else {
    return Ok(Command::Use {
      source,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    });
  };

  let options = parse_use_options(option_text)?;
  let mut names: Vec<&str> = Vec::with_capacity(options.len());
  for option in &options {
    if names.contains(&option.name.as_str()) {
      return Err(ParseError::new("use option specified more than once"));
    }
    names.push(option.name.as_str());
  }

  let mut is_lazy = false;
  let mut engine: Option<String> = None;
  let mut delimiter: Option<String> = None;
  let mut has_header: Option<bool> = None;

  for option in options {
    match option.name.as_str() {
      "lazy" => {
        if option.value != UseOptionValue::Flag {
          return Err(ParseError::new("use lazy option does not accept a value"));
        }
        is_lazy = true;
      }
      "engine" => match option.value {
        UseOptionValue::String(value) => engine = Some(value.to_lowercase()),
        _ => {
          return Err(ParseError::new("use engine option expects a string value"));
        }
      },
      "delimiter" => match option.value {
        UseOptionValue::String(value) => delimiter = Some(value),
        _ => {
          return Err(ParseError::new(
            "use delimiter option expects a string value",
          ));
        }
      },
      "has_header" => match option.value {
        UseOptionValue::Boolean(value) => has_header = Some(value),
        UseOptionValue::Flag => has_header = Some(true),
        _ => {
          return Err(ParseError::new(
            "use has_header option expects a boolean value",
          ));
        }
      },
      name => return Err(ParseError::new(format!("unknown use option: {name}"))),
    }
  }

  let lazy_engine = if let Some(engine) = engine {
    let engine = match engine.as_str() {
      "duckdb" => LazyEngine::DuckDb,
      "polars" => LazyEngine::Polars,
      _ => return Err(ParseError::new("use engine must be duckdb or polars")),
    };
    if !is_lazy {
      return Err(ParseError::new("use engine option requires lazy mode"));
    }
    Some(engine)
  } else if is_lazy {
    Some(LazyEngine::DuckDb)
  } else {
    None
  };

  Ok(Command::Use {
    source,
    execution_mode: if is_lazy {
      ExecutionMode::Lazy
    } else {
      ExecutionMode::Eager
    },
    lazy_engine,
    delimiter,
    has_header,
  })
}

fn parse_use_options(text: &str) -> Result<Vec<UseOption>, ParseError> {
  let tokens = tokenize_use_options(text)?;
  parse_use_option_tokens(tokens)
}

fn parse_use_option_tokens(tokens: Vec<UseToken>) -> Result<Vec<UseOption>, ParseError> {
  if tokens.is_empty() {
    return Err(ParseError::new(
      "comma must be followed by at least one option",
    ));
  }

  let mut stream = UseTokenStream { tokens, index: 0 };
  let mut options = Vec::new();
  while !stream.at_end() {
    let token = stream.consume().expect("stream is not at end");
    let UseTokenKind::Identifier { quoted: false } = token.kind else {
      return Err(ParseError::new("option names must be identifiers"));
    };
    let name = token.text;
    let mut value = UseOptionValue::Flag;

    if stream.peek_is_symbol("(") {
      stream.consume();
      let mut value_tokens = Vec::new();
      let mut depth = 1;
      while !stream.at_end() && depth > 0 {
        let token = stream.consume().expect("stream is not at end");
        if token.kind == UseTokenKind::Symbol && token.text == "(" {
          depth += 1;
        } else if token.kind == UseTokenKind::Symbol && token.text == ")" {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        value_tokens.push(token);
      }
      if depth > 0 {
        return Err(ParseError::new(format!(
          "option {name} is missing closing )"
        )));
      }
      if value_tokens.is_empty() && !name.eq_ignore_ascii_case("by") && name != "i" && name != "j" {
        return Err(ParseError::new(format!(
          "option {name} expects at least one value"
        )));
      }
      value = if value_tokens.is_empty()
        && (name.eq_ignore_ascii_case("by") || name == "i" || name == "j")
      {
        UseOptionValue::Identifiers(Vec::new())
      } else {
        parse_use_parenthesized_value(&name, value_tokens)?
      };
    }

    if stream.peek_is_symbol("=") {
      stream.consume();
      let Some(value_token) = stream.consume() else {
        return Err(ParseError::new(format!(
          "option {name} requires a value after ="
        )));
      };
      if value_token.kind == UseTokenKind::Symbol
        && matches!(value_token.text.as_str(), "," | "=" | "(" | ")")
      {
        return Err(ParseError::new(format!(
          "option {name} has malformed value"
        )));
      }
      value = match value_token.kind {
        UseTokenKind::Number => UseOptionValue::Number(value_token.text),
        _ => UseOptionValue::String(value_token.text),
      };
    } else if stream.peek_is_kind(&UseTokenKind::Number)
      || stream.peek_is_kind(&UseTokenKind::String)
    {
      return Err(ParseError::new(format!(
        "option {name} value must use option=value syntax"
      )));
    }

    options.push(UseOption { name, value });
  }
  Ok(options)
}

fn parse_use_parenthesized_value(
  name: &str,
  tokens: Vec<UseToken>,
) -> Result<UseOptionValue, ParseError> {
  if name == "delimiter" {
    if tokens.len() != 1
      || !matches!(
        tokens[0].kind,
        UseTokenKind::String | UseTokenKind::Identifier { .. }
      )
    {
      return Err(ParseError::new(
        "option delimiter expects a single string or identifier value",
      ));
    }
    return Ok(UseOptionValue::String(tokens[0].text.clone()));
  }

  if name == "has_header" {
    if tokens.len() != 1 {
      return Err(ParseError::new("option has_header expects true or false"));
    }
    let UseTokenKind::Identifier { quoted: false } = tokens[0].kind else {
      return Err(ParseError::new("option has_header expects true or false"));
    };
    return match tokens[0].text.to_lowercase().as_str() {
      "true" => Ok(UseOptionValue::Boolean(true)),
      "false" => Ok(UseOptionValue::Boolean(false)),
      _ => Err(ParseError::new("option has_header expects true or false")),
    };
  }

  if matches!(name, "saving" | "weights") {
    return Ok(UseOptionValue::String(
      tokens.into_iter().map(|token| token.text).collect(),
    ));
  }

  if matches!(
    name,
    "alpha"
      | "ll"
      | "ul"
      | "quantile"
      | "lags"
      | "instlag"
      | "n_iter"
      | "tol"
      | "knn"
      | "cv"
      | "bootstrap"
      | "seed"
      | "rseed"
      | "folds"
      | "level"
      | "draws"
      | "burnin"
      | "tune"
      | "chains"
      | "thin"
  ) {
    let numeric_text: String = tokens.iter().map(|token| token.text.as_str()).collect();
    if numeric_text.parse::<f64>().is_ok() {
      return Ok(UseOptionValue::Number(numeric_text));
    }
    return Err(ParseError::new(format!(
      "option {name} expects a numeric value"
    )));
  }

  if name == "prior" {
    let comma_index = tokens
      .iter()
      .position(|token| token.kind == UseTokenKind::Symbol && token.text == ",");
    let Some(comma_index) = comma_index else {
      return Err(ParseError::new(
        "prior option expects prior(variable, distribution) syntax",
      ));
    };
    if comma_index == 0 || comma_index + 1 == tokens.len() {
      return Err(ParseError::new(
        "prior option expects prior(variable, distribution) syntax",
      ));
    }
    let var_name = tokens[..comma_index]
      .iter()
      .map(|token| token.text.as_str())
      .collect::<String>()
      .trim()
      .to_string();
    let dist_expr = tokens[comma_index + 1..]
      .iter()
      .map(|token| token.text.as_str())
      .collect::<String>()
      .trim()
      .to_string();
    return Ok(UseOptionValue::Prior(var_name, dist_expr));
  }

  if name == "l1_ratio" {
    let mut values = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
      if tokens[index].kind == UseTokenKind::Number {
        values.push(tokens[index].text.clone());
        index += 1;
        continue;
      }
      if tokens[index].kind == UseTokenKind::Symbol
        && matches!(tokens[index].text.as_str(), "-" | "+")
        && tokens
          .get(index + 1)
          .is_some_and(|token| token.kind == UseTokenKind::Number)
      {
        values.push(format!("{}{}", tokens[index].text, tokens[index + 1].text));
        index += 2;
        continue;
      }
      return Err(ParseError::new("option l1_ratio values must be numeric"));
    }
    if values.len() == 1 {
      return Ok(UseOptionValue::Number(values.remove(0)));
    }
    return Ok(UseOptionValue::Numbers(values));
  }

  if name == "start" {
    let mut values = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
      if tokens[index].kind == UseTokenKind::Number {
        values.push(tokens[index].text.clone());
        index += 1;
        continue;
      }
      if tokens[index].kind == UseTokenKind::Symbol
        && matches!(tokens[index].text.as_str(), "-" | "+")
        && tokens
          .get(index + 1)
          .is_some_and(|token| token.kind == UseTokenKind::Number)
      {
        values.push(format!("{}{}", tokens[index].text, tokens[index + 1].text));
        index += 2;
        continue;
      }
      return Err(ParseError::new("option start values must be numeric"));
    }
    return Ok(UseOptionValue::Numbers(values));
  }

  if tokens
    .iter()
    .all(|token| matches!(token.kind, UseTokenKind::Identifier { .. }))
  {
    return Ok(UseOptionValue::Identifiers(
      tokens.into_iter().map(|token| token.text).collect(),
    ));
  }
  Err(ParseError::new(format!(
    "option {name} values must be identifiers"
  )))
}

#[derive(Debug)]
struct UseTokenStream {
  tokens: Vec<UseToken>,
  index: usize,
}

impl UseTokenStream {
  fn at_end(&self) -> bool {
    self.index >= self.tokens.len()
  }

  fn peek_is_symbol(&self, symbol: &str) -> bool {
    self
      .tokens
      .get(self.index)
      .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == symbol)
  }

  fn peek_is_kind(&self, kind: &UseTokenKind) -> bool {
    self
      .tokens
      .get(self.index)
      .is_some_and(|token| &token.kind == kind)
  }

  fn consume(&mut self) -> Option<UseToken> {
    let token = self.tokens.get(self.index).cloned();
    self.index += usize::from(token.is_some());
    token
  }
}

/// Tokenize a command using the pinned Python parser's lexical rules.
///
/// Token offsets are Unicode-scalar offsets, matching Python string indexing
/// rather than Rust UTF-8 byte offsets.
pub fn tokenize(text: &str) -> Result<Vec<Token>, ParseError> {
  let characters: Vec<char> = text.chars().collect();
  let mut tokens = Vec::new();
  let mut index = 0;
  while index < characters.len() {
    let character = characters[index];
    if is_command_whitespace(character) {
      index += 1;
      continue;
    }
    if character.is_alphabetic() || character == '_' {
      let start = index;
      index += 1;
      while index < characters.len()
        && (characters[index].is_alphanumeric() || characters[index] == '_')
      {
        index += 1;
      }
      tokens.push(Token {
        kind: TokenKind::Identifier { quoted: false },
        text: characters[start..index].iter().collect(),
        start,
        end: index,
      });
      continue;
    }
    if character == '`' {
      let start = index;
      index += 1;
      let mut value = String::new();
      let mut content_nonempty = false;
      let mut closed = false;
      while index < characters.len() {
        if characters[index] != '`' {
          value.push(characters[index]);
          content_nonempty = true;
          index += 1;
          continue;
        }
        if characters.get(index + 1) == Some(&'`') {
          value.push('`');
          content_nonempty = true;
          index += 2;
          continue;
        }
        index += 1;
        closed = true;
        break;
      }
      if !closed {
        return Err(ParseError::new("unterminated quoted identifier"));
      }
      if !content_nonempty {
        return Err(ParseError::new("quoted identifier cannot be empty"));
      }
      tokens.push(Token {
        kind: TokenKind::Identifier { quoted: true },
        text: value,
        start,
        end: index,
      });
      continue;
    }
    if character.is_numeric()
      || (character == '.'
        && characters
          .get(index + 1)
          .is_some_and(|next| next.is_numeric()))
    {
      let start = index;
      index += 1;
      while index < characters.len() && (characters[index].is_numeric() || characters[index] == '.')
      {
        index += 1;
      }
      let text: String = characters[start..index].iter().collect();
      if text.chars().filter(|character| *character == '.').count() > 1 {
        return Err(ParseError::new(format!("malformed number: {text}")));
      }
      tokens.push(Token {
        kind: TokenKind::Number,
        text,
        start,
        end: index,
      });
      continue;
    }
    if matches!(character, '\'' | '"') {
      let quote = character;
      index += 1;
      let value_start = index;
      while index < characters.len() && characters[index] != quote {
        index += 1;
      }
      if index >= characters.len() {
        return Err(ParseError::new("unterminated quoted string"));
      }
      let text: String = characters[value_start..index].iter().collect();
      index += 1;
      tokens.push(Token {
        kind: TokenKind::String,
        text,
        // Preserve the Python tokenizer's recovered string-token start,
        // which points just after the opening quote.
        start: value_start,
        end: index,
      });
      continue;
    }
    let two_char: String = characters[index..].iter().take(2).collect();
    if matches!(two_char.as_str(), "==" | "!=" | "<=" | ">=") {
      tokens.push(Token {
        kind: TokenKind::Symbol,
        text: two_char,
        start: index,
        end: index + 2,
      });
      index += 2;
      continue;
    }
    if matches!(
      character,
      ',' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '(' | ')' | ':' | '.'
    ) {
      tokens.push(Token {
        kind: TokenKind::Symbol,
        text: character.to_string(),
        start: index,
        end: index + 1,
      });
      index += 1;
      continue;
    }
    return Err(ParseError::new(format!(
      "unsupported token in command: {character}"
    )));
  }
  Ok(tokens)
}

fn tokenize_use_options(text: &str) -> Result<Vec<UseToken>, ParseError> {
  tokenize(text)
}

fn parse_row_limit(text: &str, name: &str) -> Result<RowLimit, ParseError> {
  if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
    return Err(ParseError::new(format!(
      "{name} row limit must be a non-negative integer"
    )));
  }
  let canonical = text.trim_start_matches('0');
  let canonical = if canonical.is_empty() { "0" } else { canonical };
  Ok(RowLimit(canonical.into()))
}

fn parse_simple_body(body: &str, allow_symbols: bool) -> Result<SimpleBody, ParseError> {
  let characters: Vec<char> = body.chars().collect();
  let mut parts = SimpleBody::default();
  let mut index = 0;
  while index < characters.len() {
    while characters
      .get(index)
      .is_some_and(|character| is_command_whitespace(*character))
    {
      index += 1;
    }
    if index >= characters.len() {
      break;
    }

    match characters[index] {
      ',' => {
        index += 1;
        while characters
          .get(index)
          .is_some_and(|character| is_command_whitespace(*character))
        {
          index += 1;
        }
        if index >= characters.len() {
          return Err(ParseError::new(
            "comma must be followed by at least one option",
          ));
        }
        parts.has_options = true;
        break;
      }
      '=' if !(allow_symbols && characters.get(index + 1) == Some(&'=')) => {
        if characters.get(index + 1) == Some(&'=') {
          return Err(ParseError::new("unsupported token in command: =="));
        }
        parts.has_assignment = true;
        parts.assignment_target_missing = parts.arguments.is_empty();
        break;
      }
      _ if is_unsupported_simple_symbol(characters[index], allow_symbols) => {
        return Err(ParseError::new(format!(
          "unsupported token in command: {}",
          characters[index]
        )));
      }
      _ => {
        let mut text = String::new();
        let mut quoted = false;
        let mut backtick_quoted = false;
        loop {
          if index >= characters.len() || is_command_whitespace(characters[index]) {
            break;
          }
          if characters[index] == ',' {
            break;
          }
          if characters[index] == '=' {
            if allow_symbols && characters.get(index + 1) == Some(&'=') {
              text.push_str("==");
              index += 2;
              continue;
            }
            if allow_symbols && matches!(text.chars().last(), Some('<' | '>')) {
              text.push('=');
              index += 1;
              continue;
            }
            break;
          }
          if characters[index] == '!' && allow_symbols && characters.get(index + 1) == Some(&'=') {
            text.push_str("!=");
            index += 2;
            continue;
          }
          if characters[index] == '!' {
            return Err(ParseError::new("unsupported token in command: !"));
          }
          if characters[index].is_alphabetic() || characters[index] == '_' {
            let identifier_start = index;
            index += 1;
            while index < characters.len()
              && (characters[index].is_alphanumeric() || characters[index] == '_')
            {
              index += 1;
            }
            let identifier_is_if = index - identifier_start == 2
              && characters[identifier_start].eq_ignore_ascii_case(&'i')
              && characters[identifier_start + 1].eq_ignore_ascii_case(&'f');
            if identifier_is_if {
              if !text.is_empty() || quoted {
                index = identifier_start;
                break;
              }
              text.push_str("if");
              continue;
            }
            text.extend(&characters[identifier_start..index]);
            continue;
          }
          if is_unsupported_simple_symbol(characters[index], allow_symbols) {
            return Err(ParseError::new(format!(
              "unsupported token in command: {}",
              characters[index]
            )));
          }
          if matches!(
            characters[index],
            '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>'
          ) {
            if allow_symbols {
              text.push(characters[index]);
              index += 1;
              continue;
            }
            return Err(ParseError::new(format!(
              "unsupported token in command: {}",
              characters[index]
            )));
          }
          if characters[index] == '.' && !allow_symbols {
            let numeric_start = text.is_empty()
              && characters
                .get(index + 1)
                .is_some_and(|character| character.is_numeric());
            let numeric_continuation = !text.is_empty()
              && text
                .chars()
                .next()
                .is_some_and(|character| character.is_numeric());
            if !numeric_start && !numeric_continuation {
              return Err(ParseError::new("unsupported token in command: ."));
            }
          }
          if matches!(characters[index], '\'' | '"' | '`') {
            let if_boundary_before_backtick =
              characters[index] == '`' && !quoted && text.eq_ignore_ascii_case("if");
            if ((!text.is_empty() || quoted) && characters[index] != '`')
              || if_boundary_before_backtick
            {
              break;
            }
            let quote = characters[index];
            quoted = true;
            backtick_quoted = backtick_quoted || quote == '`';
            let piece = parse_quoted_piece(&characters, &mut index, quote)?;
            text.push_str(&piece);
            if allow_symbols
              && quote != '`'
              && characters
                .get(index)
                .is_some_and(|next| matches!(next, '\'' | '"'))
            {
              break;
            }
          } else {
            text.push(characters[index]);
            index += 1;
          }
        }
        if !text.is_empty() || quoted {
          let is_if = !quoted && text.eq_ignore_ascii_case("if");
          if is_if {
            parts.has_condition = true;
            let mut lookahead = index;
            while characters
              .get(lookahead)
              .is_some_and(|character| is_command_whitespace(*character))
            {
              lookahead += 1;
            }
            parts.missing_condition_expression = characters
              .get(lookahead)
              .is_none_or(|character| matches!(character, ',' | '='));
            break;
          }
          parts.arguments.push(SimpleArgument {
            text,
            backtick_quoted,
          });
        }
      }
    }
  }
  Ok(parts)
}

fn is_unsupported_simple_symbol(character: char, allow_symbols: bool) -> bool {
  if !allow_symbols {
    return !character.is_alphanumeric()
      && character != '_'
      && character != '.'
      && !matches!(character, '\'' | '"' | '`' | ',' | '=');
  }
  if allow_symbols
    && !character.is_alphanumeric()
    && character != '_'
    && character != '.'
    && !matches!(character, '\'' | '"' | '`' | ',' | '=')
    && !matches!(
      character,
      '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>' | '!'
    )
  {
    return true;
  }
  let always_unsupported = matches!(
    character,
    '?' | '[' | ']' | '{' | '}' | '%' | '&' | '|' | '^' | '~' | '#'
  );
  let symbol_allowed_for_set = matches!(
    character,
    '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>'
  );
  let unsupported_known_symbol = matches!(
    character,
    '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>' | '='
  );
  always_unsupported
    || (unsupported_known_symbol && character != '=' && !(allow_symbols && symbol_allowed_for_set))
    || (character == '=' && !allow_symbols)
}

fn parse_quoted_piece(
  characters: &[char],
  index: &mut usize,
  quote: char,
) -> Result<String, ParseError> {
  *index += 1;
  let mut text = String::new();
  let mut content_nonempty = false;
  while *index < characters.len() {
    if characters[*index] == quote {
      if characters.get(*index + 1) == Some(&quote) && quote == '`' {
        if quote == '`' && !content_nonempty && *index + 2 == characters.len() {
          return Err(ParseError::new("quoted identifier cannot be empty"));
        }
        text.push(quote);
        content_nonempty = true;
        *index += 2;
        continue;
      }
      *index += 1;
      if quote == '`' && !content_nonempty {
        return Err(ParseError::new("quoted identifier cannot be empty"));
      }
      return Ok(text);
    }
    text.push(characters[*index]);
    content_nonempty = true;
    *index += 1;
  }
  Err(ParseError::new(if quote == '`' {
    "unterminated quoted identifier"
  } else {
    "unterminated quoted string"
  }))
}

fn parse_help(body: &str) -> Result<Command, ParseError> {
  let mut words = body
    .split(is_command_whitespace)
    .filter(|word| !word.is_empty());
  let topic = words.next().map(str::to_lowercase);
  if words.next().is_some() {
    return Err(ParseError::new(
      "help expects at most one command name: help <command>",
    ));
  }
  Ok(Command::Help { topic })
}

#[cfg(test)]
mod tests {
  use super::{
    BarCommand, BayesCommand, BayesPlotCommand, BayesPlotKind, BayesPrefixCommand, ByCommand,
    CfRegressCommand, Command, CvelasticnetCommand, CvelasticnetL1Ratio, CvlassoCommand,
    CvridgeCommand, DataSource, DidCommand, DmlCommand, DrDidCommand, DrDidMethod,
    ElasticnetCommand, ExecutionMode, GenerateBinaryOperator, GenerateExpression, HeckmanCommand,
    HistogramCommand, LassoCommand, LazyEngine, LincomCommand, LogitCommand, LowessCommand,
    NbregCommand, NlCommand, ParseError, PoissonCommand, PostlassoCommand, PredictCommand,
    PredictKind, ProbitCommand, QregCommand, RegressCommand, RegressEstimator, RidgeCommand,
    RowLimit, ScatterCommand, SettingName, SortKey, SpregressCommand, SpregressContiguity,
    SpregressModelType, SqlCommand, StregCommand, StregDistribution, TabulateCommand, TestCommand,
    TobitCommand, XtLogitCommand, ZinbCommand, ZipCommand, parse_command,
  };

  #[test]
  fn parses_help_aliases_and_topics() {
    assert_eq!(
      parse_command("help").unwrap(),
      Command::Help { topic: None }
    );
    assert_eq!(parse_command("?").unwrap(), Command::Help { topic: None });
    assert_eq!(
      parse_command("help summarize").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(
      parse_command("? SUMMARIZE").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(
      parse_command("?foo").unwrap(),
      Command::Help {
        topic: Some("foo".to_owned()),
      }
    );
  }

  #[test]
  fn normalizes_command_case_and_whitespace() {
    assert_eq!(
      parse_command("  HELP   SUMMARIZE  ").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(parse_command("\tSTATUS\n").unwrap(), Command::Status);
    assert_eq!(parse_command(" qUiT ").unwrap(), Command::Exit);
  }

  #[test]
  fn parses_status_and_exit_aliases() {
    assert_eq!(parse_command("status").unwrap(), Command::Status);
    assert_eq!(parse_command("exit").unwrap(), Command::Exit);
    assert_eq!(parse_command("quit").unwrap(), Command::Exit);
  }

  #[test]
  fn parses_describe_with_case_and_whitespace_normalization() {
    assert_eq!(parse_command("describe").unwrap(), Command::Describe);
    assert_eq!(
      parse_command("  DESCRIBE\u{1c}").unwrap(),
      Command::Describe
    );
  }

  #[test]
  fn parses_doctor_with_case_and_whitespace_normalization() {
    assert_eq!(parse_command("doctor").unwrap(), Command::Doctor);
    assert_eq!(parse_command("\tDOCTOR\u{1c}").unwrap(), Command::Doctor);
  }

  #[test]
  fn parses_summarize_variables_without_execution() {
    assert_eq!(
      parse_command(" SUMMARIZE ").unwrap(),
      Command::Summarize { variables: vec![] }
    );
    assert_eq!(
      parse_command("summarize age bmi").unwrap(),
      Command::Summarize {
        variables: vec!["age".to_owned(), "bmi".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize\u{1c}`bmi-zscore`\u{1d}\"value col\"").unwrap(),
      Command::Summarize {
        variables: vec!["bmi-zscore".to_owned(), "value col".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize `x``y` \"report\"").unwrap(),
      Command::Summarize {
        variables: vec!["x`y".to_owned(), "report".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize age age").unwrap(),
      Command::Summarize {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_summarize_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "summarize age if age > 0",
        "summarize does not accept if clauses or options",
      ),
      (
        "summarize age, detail",
        "summarize does not accept if clauses or options",
      ),
      (
        "summarize age = other",
        "summarize does not accept assignment syntax",
      ),
      (
        "summarize = age",
        "summarize assignment requires a target before =",
      ),
      (
        "summarize age,",
        "comma must be followed by at least one option",
      ),
      (
        "summarize,",
        "comma must be followed by at least one option",
      ),
      ("summarize if", "missing expression after if"),
      ("summarize age if", "missing expression after if"),
      ("summarize age==x", "unsupported token in command: =="),
      ("summarize age-1", "unsupported token in command: -"),
      ("summarize age+1", "unsupported token in command: +"),
      ("summarize age!x", "unsupported token in command: !"),
      ("summarize age@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_datasignature_with_case_and_whitespace_normalization() {
    assert_eq!(
      parse_command("datasignature").unwrap(),
      Command::Datasignature
    );
    assert_eq!(
      parse_command("\tDATASIGNATURE\u{1c}").unwrap(),
      Command::Datasignature
    );
  }

  #[test]
  fn parses_codebook_variables_without_execution() {
    assert_eq!(
      parse_command(" CODEBOOK ").unwrap(),
      Command::Codebook { variables: vec![] }
    );
    assert_eq!(
      parse_command("codebook age sex").unwrap(),
      Command::Codebook {
        variables: vec!["age".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook `bmi-zscore` `cost.2024` `x/y`").unwrap(),
      Command::Codebook {
        variables: vec![
          "bmi-zscore".to_owned(),
          "cost.2024".to_owned(),
          "x/y".to_owned(),
        ],
      }
    );
    assert_eq!(
      parse_command("codebook\u{1c}`x y`\u{1d}sex").unwrap(),
      Command::Codebook {
        variables: vec!["x y".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook foo\"bar\"").unwrap(),
      Command::Codebook {
        variables: vec!["foo".to_owned(), "bar".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook \"foo\"\"bar\"").unwrap(),
      Command::Codebook {
        variables: vec!["foo".to_owned(), "bar".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook \"\"\"\"").unwrap(),
      Command::Codebook {
        variables: vec![String::new(), String::new()],
      }
    );
  }

  #[test]
  fn parses_missing_variables_without_execution() {
    assert_eq!(
      parse_command(" MISSING ").unwrap(),
      Command::Missing { variables: vec![] }
    );
    assert_eq!(
      parse_command("missing cost age").unwrap(),
      Command::Missing {
        variables: vec!["cost".to_owned(), "age".to_owned()],
      }
    );
    assert_eq!(
      parse_command("missing\u{1c}`bmi-zscore`\u{1d}\"value col\"").unwrap(),
      Command::Missing {
        variables: vec!["bmi-zscore".to_owned(), "value col".to_owned()],
      }
    );
  }

  #[test]
  fn parses_duplicates_variables_without_execution() {
    assert_eq!(
      parse_command(" DUPLICATES ").unwrap(),
      Command::Duplicates { variables: vec![] }
    );
    assert_eq!(
      parse_command("duplicates report id label").unwrap(),
      Command::Duplicates {
        variables: vec!["id".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("duplicates id label").unwrap(),
      Command::Duplicates {
        variables: vec!["id".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("duplicates \"report\"").unwrap(),
      Command::Duplicates { variables: vec![] }
    );
    assert_eq!(
      parse_command("duplicates\u{1c}`report`\u{1d}cost").unwrap(),
      Command::Duplicates {
        variables: vec!["report".to_owned(), "cost".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_duplicates_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "duplicates id if id > 0",
        "duplicates does not accept if clauses or options",
      ),
      (
        "duplicates id, missing",
        "duplicates does not accept if clauses or options",
      ),
      (
        "duplicates id = other",
        "duplicates does not accept assignment syntax",
      ),
      (
        "duplicates = id",
        "duplicates assignment requires a target before =",
      ),
      (
        "duplicates id,",
        "comma must be followed by at least one option",
      ),
      (
        "duplicates,",
        "comma must be followed by at least one option",
      ),
      ("duplicates if", "missing expression after if"),
      ("duplicates id if", "missing expression after if"),
      ("duplicates id==x", "unsupported token in command: =="),
      ("duplicates id-1", "unsupported token in command: -"),
      ("duplicates id+1", "unsupported token in command: +"),
      ("duplicates id!x", "unsupported token in command: !"),
      ("duplicates id@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_isid_variables_and_missok_without_execution() {
    assert_eq!(
      parse_command(" ISID patient_id visit ").unwrap(),
      Command::Isid {
        variables: vec!["patient_id".to_owned(), "visit".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid\u{1c}`a,b`\u{1d}visit, missok missok").unwrap(),
      Command::Isid {
        variables: vec!["a,b".to_owned(), "visit".to_owned()],
        missok: true,
      }
    );
    assert_eq!(
      parse_command("isid a a").unwrap(),
      Command::Isid {
        variables: vec!["a".to_owned(), "a".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid `a``b`").unwrap(),
      Command::Isid {
        variables: vec!["a`b".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid \"\"").unwrap(),
      Command::Isid {
        variables: vec![String::new()],
        missok: false,
      }
    );
  }

  #[test]
  fn rejects_invalid_isid_syntax_with_exact_diagnostics() {
    let cases = [
      ("isid", "isid expects at least one key variable"),
      ("isid, missok", "isid expects at least one key variable"),
      ("isid,", "comma must be followed by at least one option"),
      (
        "isid patient_id if visit > 0",
        "isid only accepts a variable list and missok option",
      ),
      (
        "isid patient_id = other",
        "isid only accepts a variable list and missok option",
      ),
      (
        "isid patient_id =",
        "isid assignment requires an expression after =",
      ),
      (
        "isid = patient_id",
        "isid assignment requires a target before =",
      ),
      ("isid patient_id, report", "isid unsupported option: report"),
      (
        "isid patient_id, foo bar",
        "isid unsupported option: bar, foo",
      ),
      (
        "isid patient_id, missok(true)",
        "isid option missok does not accept a value",
      ),
      (
        "isid patient_id, missok 1",
        "option missok value must use option=value syntax",
      ),
      ("isid patient_id if", "missing expression after if"),
      ("isid patient_id==x", "unsupported token in command: =="),
      ("isid patient_id-x", "unsupported token in command: -"),
      ("isid patient_id+x", "unsupported token in command: +"),
      ("isid patient_id!x", "unsupported token in command: !"),
      ("isid patient_id@x", "unsupported token in command: @"),
      ("isid patient_id, MISSOK", "isid unsupported option: MISSOK"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_select_variables_without_execution() {
    assert_eq!(
      parse_command(" SELECT age sex ").unwrap(),
      Command::Select {
        variables: vec!["age".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("select\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
      Command::Select {
        variables: vec!["a,b".to_owned(), "old name".to_owned()],
      }
    );
    assert_eq!(
      parse_command("select age age").unwrap(),
      Command::Select {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_select_syntax_with_exact_diagnostics() {
    let cases = [
      ("select", "select expects at least one variable"),
      (
        "select age if age > 0",
        "select only accepts a variable list",
      ),
      ("select age, stable", "select only accepts a variable list"),
      ("select age = x", "select only accepts a variable list"),
      ("select = x", "select assignment requires a target before ="),
      (
        "select age =",
        "select assignment requires an expression after =",
      ),
      (
        "select age,",
        "comma must be followed by at least one option",
      ),
      ("select if", "missing expression after if"),
      ("select age==x", "unsupported token in command: =="),
      ("select age-1", "unsupported token in command: -"),
      ("select age+1", "unsupported token in command: +"),
      ("select age!x", "unsupported token in command: !"),
      ("select age@x", "unsupported token in command: @"),
      ("select:age", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_sort_variables_without_execution() {
    assert_eq!(
      parse_command(" SORT age label ").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
      Command::Sort {
        variables: vec!["a,b".to_owned(), "old name".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort age age").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort age label now").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "label".to_owned(), "now".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_sort_syntax_with_exact_diagnostics() {
    let cases = [
      ("sort", "sort expects at least one variable"),
      ("sort age if age > 0", "sort only accepts a variable list"),
      ("sort age, stable", "sort only accepts a variable list"),
      ("sort age = x", "sort only accepts a variable list"),
      ("sort = x", "sort assignment requires a target before ="),
      (
        "sort age =",
        "sort assignment requires an expression after =",
      ),
      ("sort age,", "comma must be followed by at least one option"),
      ("sort if", "missing expression after if"),
      ("sort age==x", "unsupported token in command: =="),
      ("sort age-1", "unsupported token in command: -"),
      ("sort age+1", "unsupported token in command: +"),
      ("sort age!x", "unsupported token in command: !"),
      ("sort age@x", "unsupported token in command: @"),
      ("sort:age", "unsupported token in command: :"),
      ("sort age:label", "unsupported token in command: :"),
      ("sort age/label", "unsupported token in command: /"),
      ("sort age.label", "unsupported token in command: ."),
      ("sort +age", "unsupported token in command: +"),
      ("sort -age", "unsupported token in command: -"),
      ("sort !age", "unsupported token in command: !"),
      ("sort @age", "unsupported token in command: @"),
      ("sort ``", "quoted identifier cannot be empty"),
      ("sort \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_gsort_keys_without_execution() {
    assert_eq!(
      parse_command(" GSORT group_id -label ").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "label".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort\u{1c}+group_id\u{1d}-label").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "label".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort `-score` \"old name\"").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "-score".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "old name".to_owned(),
            descending: false,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort \"-score\"").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "score".to_owned(),
          descending: true,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort group_id group_id").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort+age").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "age".to_owned(),
          descending: false,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort-age").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "age".to_owned(),
          descending: true,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort +age -age").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "age".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "age".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort:age age==x").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: ":age".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "age==x".to_owned(),
            descending: false,
          },
        ],
      }
    );
  }

  #[test]
  fn rejects_invalid_gsort_syntax_with_exact_diagnostics() {
    let cases = [
      ("gsort", "gsort expects at least one variable"),
      (
        "gsort group_id if x > 0",
        "gsort only accepts a signed variable list",
      ),
      (
        "gsort group_id, stable",
        "gsort only accepts a signed variable list",
      ),
      (
        "gsort group_id = x",
        "gsort only accepts a signed variable list",
      ),
      ("gsort = x", "gsort assignment requires a target before ="),
      (
        "gsort group_id =",
        "gsort assignment requires an expression after =",
      ),
      (
        "gsort group_id,",
        "comma must be followed by at least one option",
      ),
      ("gsort,", "comma must be followed by at least one option"),
      ("gsort if", "missing expression after if"),
      (
        "gsort --group_id",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort ++label",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort +-label",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort -",
        "gsort expects a variable after each direction prefix",
      ),
      (
        "gsort +",
        "gsort expects a variable after each direction prefix",
      ),
      (
        "gsort group_id -",
        "gsort expects a variable after each direction prefix",
      ),
      ("gsort age!x", "unsupported token in command: !"),
      ("gsort age@x", "unsupported token in command: @"),
      ("gsort!age", "unsupported token in command: !"),
      ("gsort@age", "unsupported token in command: @"),
      ("gsort?age", "unsupported token in command: ?"),
      ("gsort ``", "quoted identifier cannot be empty"),
      ("gsort \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_rename_names_without_execution() {
    assert_eq!(
      parse_command(" rename sex gender ").unwrap(),
      Command::Rename {
        old_name: "sex".to_owned(),
        new_name: "gender".to_owned(),
      }
    );
    assert_eq!(
      parse_command("RENAME\u{1c}  `old-name`\u{1d}\"new name\"").unwrap(),
      Command::Rename {
        old_name: "old-name".to_owned(),
        new_name: "new name".to_owned(),
      }
    );
    assert_eq!(
      parse_command("rename old old").unwrap(),
      Command::Rename {
        old_name: "old".to_owned(),
        new_name: "old".to_owned(),
      }
    );
  }

  #[test]
  fn rejects_invalid_rename_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "rename",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new now",
        "rename expects exactly two variables: rename old new",
      ),
      ("rename if", "missing expression after if"),
      ("rename old if", "missing expression after if"),
      ("rename old new if", "missing expression after if"),
      (
        "rename old if x > 0",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new if x > 0",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new, replace",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new,",
        "comma must be followed by at least one option",
      ),
      (
        "rename=old new",
        "rename assignment requires a target before =",
      ),
      (
        "rename = old",
        "rename assignment requires a target before =",
      ),
      ("rename==old new", "unsupported token in command: =="),
      ("rename:old new", "unsupported token in command: :"),
      ("rename old-new new", "unsupported token in command: -"),
      ("rename old+new new", "unsupported token in command: +"),
      ("rename old@new new", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_run_path_without_execution() {
    assert_eq!(
      parse_command(" run analysis.td ").unwrap(),
      Command::Run {
        path: "analysis.td".to_owned(),
      }
    );
    assert_eq!(
      parse_command("RUN\u{1c}analysis.td").unwrap(),
      Command::Run {
        path: "analysis.td".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run \"analysis.td\"").unwrap(),
      Command::Run {
        path: "\"analysis.td\"".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run `analysis.td`").unwrap(),
      Command::Run {
        path: "`analysis.td`".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run analysis.td,").unwrap(),
      Command::Run {
        path: "analysis.td,".to_owned(),
      }
    );
  }

  #[test]
  fn rejects_invalid_run_syntax_with_exact_diagnostics() {
    let cases = [
      ("run", "run expects exactly one path: run <script>"),
      ("run   ", "run expects exactly one path: run <script>"),
      (
        "run a.td b.td",
        "run expects exactly one path: run <script>",
      ),
      (
        "run \"a b.td\"",
        "run expects exactly one path: run <script>",
      ),
      (
        "run a.td if x > 0",
        "run expects exactly one path: run <script>",
      ),
      ("run,", "comma must be followed by at least one option"),
      ("run,foo", "unknown command: run"),
      ("run=foo", "run assignment requires a target before ="),
      ("run==foo", "unsupported token in command: =="),
      ("run:foo", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_save_and_export_paths_without_execution() {
    assert_eq!(
      parse_command(" SAVE output.parquet ").unwrap(),
      Command::Save {
        path: "output.parquet".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("export \"my output.parquet\", replace").unwrap(),
      Command::Export {
        path: "my output.parquet".to_owned(),
        replace: true,
      }
    );
    assert_eq!(
      parse_command("save `a,b`, replace replace").unwrap(),
      Command::Save {
        path: "a,b".to_owned(),
        replace: true,
      }
    );
    assert_eq!(
      parse_command("export a==b").unwrap(),
      Command::Export {
        path: "a==b".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("save:out.parquet").unwrap(),
      Command::Save {
        path: ":out.parquet".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("export/out.csv").unwrap(),
      Command::Export {
        path: "/out.csv".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("save \"\"").unwrap(),
      Command::Save {
        path: String::new(),
        replace: false,
      }
    );
  }

  #[test]
  fn rejects_invalid_save_and_export_syntax_with_exact_diagnostics() {
    let cases = [
      ("save", "save expects exactly one path"),
      ("save one two", "save expects exactly one path"),
      ("export", "export expects exactly one path"),
      ("export one two", "export expects exactly one path"),
      (
        "save out if x > 0",
        "save does not accept if clauses or assignment syntax",
      ),
      (
        "export out = x",
        "export does not accept if clauses or assignment syntax",
      ),
      ("save if", "missing expression after if"),
      ("save = out", "save assignment requires a target before ="),
      (
        "export out =",
        "export assignment requires an expression after =",
      ),
      ("save out,", "comma must be followed by at least one option"),
      ("save out, force", "save unsupported option: force"),
      ("export out, REPLACE", "export unsupported option: REPLACE"),
      (
        "save out, replace=true",
        "save option replace does not accept a value",
      ),
      (
        "export out, replace(foo)",
        "export option replace does not accept a value",
      ),
      ("save out@x", "unsupported token in command: @"),
      ("export@out", "unsupported token in command: @"),
      (
        "save out, replace, replace",
        "option names must be identifiers",
      ),
      ("save ``, replace", "quoted identifier cannot be empty"),
      ("export \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_set_values_without_executing_configuration() {
    assert_eq!(
      parse_command(" SET GRAPH_FORMAT PnG ").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "PnG".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir artifacts/custom").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: "artifacts/custom".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_open \"Off\"").unwrap(),
      Command::Set {
        name: SettingName::GraphOpen,
        value: "Off".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir 'my plots'").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: "my plots".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_format `svg`").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "svg".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir \"\"").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: String::new(),
      }
    );
    assert_eq!(
      parse_command("set graph_open maybe").unwrap(),
      Command::Set {
        name: SettingName::GraphOpen,
        value: "maybe".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_format foo==bar").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "foo==bar".to_owned(),
      }
    );
    for value in ["foo<=bar", "foo>=bar", "<=foo", ">=foo"] {
      assert_eq!(
        parse_command(&format!("set graph_format {value}")).unwrap(),
        Command::Set {
          name: SettingName::GraphFormat,
          value: value.to_owned(),
        },
        "{value:?}"
      );
    }
    for (input, value) in [
      ("set graph_format foo`bar`", "foobar"),
      ("set graph_format \"a\"`b`", "ab"),
      ("set graph_format foo`bar`+x", "foobar+x"),
      ("set graph_format `foo`bar`baz`", "foobarbaz"),
    ] {
      assert_eq!(
        parse_command(input).unwrap(),
        Command::Set {
          name: SettingName::GraphFormat,
          value: value.to_owned(),
        },
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_inspection_commands_and_canonical_limits() {
    assert_eq!(parse_command("count").unwrap(), Command::Count);
    assert_eq!(parse_command(" COUNT ").unwrap(), Command::Count);
    assert_eq!(
      parse_command("head").unwrap(),
      Command::Head {
        limit: RowLimit::default(),
      }
    );
    assert_eq!(
      parse_command("TAIL 000").unwrap(),
      Command::Tail {
        limit: RowLimit("0".into()),
      }
    );
    let head = parse_command("head 00018446744073709551616").unwrap();
    assert_eq!(
      head,
      Command::Head {
        limit: RowLimit("18446744073709551616".into()),
      }
    );
    let huge = "9".repeat(100);
    let tail = parse_command(&format!("tail {huge}")).unwrap();
    assert_eq!(
      tail,
      Command::Tail {
        limit: RowLimit(huge.into_boxed_str()),
      }
    );
    assert_eq!(
      match parse_command("head 0007").unwrap() {
        Command::Head { limit } => limit.as_decimal().to_owned(),
        other => panic!("unexpected command: {other:?}"),
      },
      "7"
    );
  }

  #[test]
  fn parses_use_sources_modes_and_options() {
    assert_eq!(
      parse_command("use data.parquet").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("data.parquet".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: None,
        has_header: None,
      }
    );
    assert_eq!(
      parse_command("  USE\u{1c}s3://bucket/data.parquet, lazy  ").unwrap(),
      Command::Use {
        source: DataSource::Uri("s3://bucket/data.parquet".to_owned()),
        execution_mode: ExecutionMode::Lazy,
        lazy_engine: Some(LazyEngine::DuckDb),
        delimiter: None,
        has_header: None,
      }
    );
    assert_eq!(
      parse_command("use file.csv, lazy engine=POLARS delimiter=\";\" has_header(false)").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Lazy,
        lazy_engine: Some(LazyEngine::Polars),
        delimiter: Some(";".to_owned()),
        has_header: Some(false),
      }
    );
    assert_eq!(
      parse_command("use file.csv, has_header(TRUE) delimiter(\"\")").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: Some(String::new()),
        has_header: Some(true),
      }
    );
    assert_eq!(
      parse_command("use file.csv, has_header").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: None,
        has_header: Some(true),
      }
    );
  }

  #[test]
  fn rejects_invalid_use_syntax_with_exact_diagnostics() {
    let cases = [
      ("use", "use expects exactly one path: use <path>"),
      (
        "use one.parquet two.parquet",
        "use expects exactly one path: use <path>",
      ),
      (
        "use data.parquet,",
        "comma must be followed by at least one option",
      ),
      (
        "use data.parquet, lazy=true",
        "use lazy option does not accept a value",
      ),
      (
        "use data.parquet, lazy lazy",
        "use option specified more than once",
      ),
      (
        "use data.parquet, engine=duckdb",
        "use engine option requires lazy mode",
      ),
      (
        "use data.parquet, lazy engine=spark",
        "use engine must be duckdb or polars",
      ),
      (
        "use data.parquet, engine",
        "use engine option expects a string value",
      ),
      (
        "use data.parquet, engine=١",
        "use engine option expects a string value",
      ),
      (
        "use data.parquet, delimiter",
        "use delimiter option expects a string value",
      ),
      (
        "use data.parquet, has_header(1)",
        "option has_header expects true or false",
      ),
      ("use data.parquet, unknown", "unknown use option: unknown"),
      (
        "use data.parquet, delimiter()",
        "option delimiter expects at least one value",
      ),
      (
        "use data.parquet, engine()",
        "option engine expects at least one value",
      ),
      (
        "use data.parquet, has_header=",
        "option has_header requires a value after =",
      ),
      (
        "use data.parquet, delimiter(,)",
        "option delimiter expects a single string or identifier value",
      ),
      ("use data.parquet, alpha(1)", "unknown use option: alpha"),
      (
        "use data.parquet, alpha(foo)",
        "option alpha expects a numeric value",
      ),
      ("use data.parquet, saving(1)", "unknown use option: saving"),
      (
        "use data.parquet, prior(x)",
        "prior option expects prior(variable, distribution) syntax",
      ),
      (
        "use data.parquet, prior(x,normal)",
        "unknown use option: prior",
      ),
      (
        "use data.parquet, l1_ratio(foo)",
        "option l1_ratio values must be numeric",
      ),
      (
        "use data.parquet, unknown(1)",
        "option unknown values must be identifiers",
      ),
      (
        "use data.parquet, delimiter=;",
        "unsupported token in command: ;",
      ),
      (
        "use data.parquet, lazy,",
        "option names must be identifiers",
      ),
      ("use, lazy", "unknown command: use"),
      ("use,", "comma must be followed by at least one option"),
      ("use,,", "option names must be identifiers"),
      ("use=data", "use assignment requires a target before ="),
      ("use==data", "unsupported token in command: =="),
      ("use:data", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn accepts_quoted_numeric_limits() {
    for (input, expected) in [
      ("head \"10\"", "10"),
      ("tail '0010'", "10"),
      ("head `0`", "0"),
    ] {
      let command = parse_command(input).unwrap();
      let actual = match command {
        Command::Head { limit } | Command::Tail { limit } => limit.as_decimal().to_owned(),
        other => panic!("unexpected command: {other:?}"),
      };
      assert_eq!(actual, expected);
    }
  }

  #[test]
  fn rejects_invalid_inspection_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "count 1",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "count if x",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "count, detail",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("count = 1", "count assignment requires a target before ="),
      ("count == 1", "unsupported token in command: =="),
      ("count -1", "unsupported token in command: -"),
      ("head 5 6", "head accepts at most one row limit"),
      ("head 1.0", "head row limit must be a non-negative integer"),
      ("head 1e2", "head row limit must be a non-negative integer"),
      ("head foo", "head row limit must be a non-negative integer"),
      ("head ١", "head row limit must be a non-negative integer"),
      ("head -1", "unsupported token in command: -"),
      ("head +1", "unsupported token in command: +"),
      (
        "head 1 if x",
        "head does not accept if clauses, options, or assignment syntax",
      ),
      (
        "head 1, detail",
        "head does not accept if clauses, options, or assignment syntax",
      ),
      ("head = 1", "head assignment requires a target before ="),
      ("head == 1", "unsupported token in command: =="),
      ("head,", "comma must be followed by at least one option"),
      ("head 5,", "comma must be followed by at least one option"),
      ("tail 5 6", "tail accepts at most one row limit"),
      ("tail 1.0", "tail row limit must be a non-negative integer"),
      ("tail -1", "unsupported token in command: -"),
      ("tail +1", "unsupported token in command: +"),
      (
        "tail 1 if x",
        "tail does not accept if clauses, options, or assignment syntax",
      ),
      (
        "tail 1, detail",
        "tail does not accept if clauses, options, or assignment syntax",
      ),
      ("tail = 1", "tail assignment requires a target before ="),
      ("tail == 1", "unsupported token in command: =="),
      ("tail,", "comma must be followed by at least one option"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
    assert_eq!(
      parse_command("head \"\"").unwrap_err().to_string(),
      "head row limit must be a non-negative integer"
    );
    assert_eq!(
      parse_command("head ``").unwrap_err().to_string(),
      "quoted identifier cannot be empty"
    );
    assert_eq!(
      parse_command("head \"unterminated")
        .unwrap_err()
        .to_string(),
      "unterminated quoted string"
    );
  }

  #[test]
  fn rejects_invalid_describe_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "describe age",
        "describe does not accept arguments, if clauses, or options",
      ),
      (
        "describe if age > 18",
        "describe does not accept arguments, if clauses, or options",
      ),
      (
        "describe, detail",
        "describe does not accept arguments, if clauses, or options",
      ),
      ("describe,", "comma must be followed by at least one option"),
      (
        "describe age,",
        "comma must be followed by at least one option",
      ),
      (
        "describe=now",
        "describe assignment requires a target before =",
      ),
      ("describe == now", "unsupported token in command: =="),
      ("describe -1", "unsupported token in command: -"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_doctor_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "doctor foo",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "doctor if age > 18",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "doctor, detail",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("doctor if", "missing expression after if"),
      ("doctor,", "comma must be followed by at least one option"),
      (
        "doctor foo,",
        "comma must be followed by at least one option",
      ),
      ("doctor=now", "doctor assignment requires a target before ="),
      ("doctor == now", "unsupported token in command: =="),
      ("doctor -1", "unsupported token in command: -"),
      ("doctor +1", "unsupported token in command: +"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_datasignature_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "datasignature age",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age > 0",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age==x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age+x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age-x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("datasignature age if", "missing expression after if"),
      ("datasignature if, fast", "missing expression after if"),
      ("datasignature if,", "missing expression after if"),
      (
        "datasignature, fast",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("datasignature if", "missing expression after if"),
      (
        "datasignature,",
        "comma must be followed by at least one option",
      ),
      (
        "datasignature age,",
        "comma must be followed by at least one option",
      ),
      (
        "datasignature = value",
        "datasignature assignment requires a target before =",
      ),
      (
        "datasignature=value",
        "datasignature assignment requires a target before =",
      ),
      ("datasignature == now", "unsupported token in command: =="),
      ("datasignature -1", "unsupported token in command: -"),
      ("datasignature +1", "unsupported token in command: +"),
      ("datasignature age==x", "unsupported token in command: =="),
      ("datasignature age-1", "unsupported token in command: -"),
      ("datasignature age+1", "unsupported token in command: +"),
      ("datasignature age!x", "unsupported token in command: !"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_codebook_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "codebook age if age > 18",
        "codebook does not accept if clauses or options",
      ),
      (
        "codebook age, detail",
        "codebook does not accept if clauses or options",
      ),
      (
        "codebook age = 1",
        "codebook does not accept assignment syntax",
      ),
      (
        "codebook = 1",
        "codebook assignment requires a target before =",
      ),
      (
        "codebook age,",
        "comma must be followed by at least one option",
      ),
      ("codebook if", "missing expression after if"),
      ("codebook -1", "unsupported token in command: -"),
      ("codebook +1", "unsupported token in command: +"),
      ("codebook .", "unsupported token in command: ."),
      ("codebook age@x", "unsupported token in command: @"),
      ("codebook age#x", "unsupported token in command: #"),
      (
        "codebook if`foo`",
        "codebook does not accept if clauses or options",
      ),
      ("codebook age==x", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_missing_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "missing age if age > 0",
        "missing does not accept if clauses or options",
      ),
      (
        "missing age, detail",
        "missing does not accept if clauses or options",
      ),
      (
        "missing age = other",
        "missing does not accept assignment syntax",
      ),
      (
        "missing = age",
        "missing assignment requires a target before =",
      ),
      (
        "missing age,",
        "comma must be followed by at least one option",
      ),
      ("missing,", "comma must be followed by at least one option"),
      ("missing if", "missing expression after if"),
      ("missing age==x", "unsupported token in command: =="),
      ("missing age-1", "unsupported token in command: -"),
      ("missing age+1", "unsupported token in command: +"),
      ("missing age!x", "unsupported token in command: !"),
      ("missing age@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_set_syntax_with_exact_diagnostics() {
    let cases = [
      ("set", "set expects syntax: set name value"),
      ("set graph_format", "set expects syntax: set name value"),
      (
        "set graph_format png extra",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format png, detail",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format, detail",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format png,",
        "comma must be followed by at least one option",
      ),
      ("set unknown on", "unknown setting: unknown"),
      ("set `graph_format` png", "unknown setting: graph_format"),
      ("set = png", "set assignment requires a target before ="),
      ("set=png", "set assignment requires a target before ="),
      (
        "set graph_format = png",
        "set expects syntax: set name value",
      ),
      ("set if", "missing expression after if"),
      ("set graph_format if", "missing expression after if"),
      (
        "set graph_format foo?bar",
        "unsupported token in command: ?",
      ),
      (
        "set graph_format foo\\bar",
        "unsupported token in command: \\",
      ),
      (
        "set graph_format foo;bar",
        "unsupported token in command: ;",
      ),
      (
        "set graph_format foo@bar",
        "unsupported token in command: @",
      ),
      (
        "set graph_format foo$bar",
        "unsupported token in command: $",
      ),
      (
        "set graph_format foo😀bar",
        "unsupported token in command: 😀",
      ),
      (
        "set graph_format \"a\"\"b\"",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format 'a''b'",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format foo\"bar\"",
        "set expects syntax: set name value",
      ),
      ("set graph_format foo-if", "missing expression after if"),
      ("set graph_format foo.if", "missing expression after if"),
      ("set graph_format \"foo\"if", "missing expression after if"),
      ("set graph_format `foo`if", "missing expression after if"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_empty_unknown_and_quoted_commands() {
    assert_eq!(
      parse_command("  ").unwrap_err().to_string(),
      "empty command"
    );
    assert_eq!(
      parse_command("unknown").unwrap_err().to_string(),
      "unknown command: unknown"
    );
    assert_eq!(
      parse_command("ÄBC").unwrap_err().to_string(),
      "unknown command: äbc"
    );
    assert_eq!(
      parse_command("ΣTATUS").unwrap_err().to_string(),
      "unknown command: σtatus"
    );
    assert_eq!(
      parse_command("`help`").unwrap_err().to_string(),
      "command must start with an unquoted command name"
    );
  }

  #[test]
  fn rejects_extra_help_words() {
    assert_eq!(
      parse_command("help one two").unwrap_err(),
      ParseError::new("help expects at most one command name: help <command>")
    );
    assert_eq!(
      parse_command("? one two").unwrap_err(),
      ParseError::new("help expects at most one command name: help <command>")
    );
  }

  #[test]
  fn rejects_arguments_for_read_only_and_exit_commands() {
    assert_eq!(
      parse_command("status now").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    assert_eq!(
      parse_command("status, verbose").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    for (input, expected) in [
      ("status -1", "unsupported token in command: -"),
      ("status +1", "unsupported token in command: +"),
      ("status-1", "unsupported token in command: -"),
      ("status+1", "unsupported token in command: +"),
      ("status -", "unsupported token in command: -"),
      ("status +", "unsupported token in command: +"),
      ("status --1", "unsupported token in command: -"),
      ("status ++1", "unsupported token in command: +"),
      ("status -1,", "unsupported token in command: -"),
      ("status +1,", "unsupported token in command: +"),
      (
        "status if x",
        "status does not accept arguments, if clauses, options, or assignment syntax",
      ),
    ] {
      assert_eq!(parse_command(input).unwrap_err().to_string(), expected);
    }
    assert_eq!(
      parse_command("status if").unwrap_err().to_string(),
      "missing expression after if"
    );
    assert_eq!(
      parse_command("exit foo").unwrap_err().to_string(),
      "exit does not accept arguments, if clauses, or options"
    );
    assert_eq!(
      parse_command("quit, now").unwrap_err().to_string(),
      "quit does not accept arguments, if clauses, or options"
    );
    assert_eq!(
      parse_command("status=now").unwrap_err().to_string(),
      "status assignment requires a target before ="
    );
    assert_eq!(
      parse_command("exit=now").unwrap_err().to_string(),
      "exit assignment requires a target before ="
    );
    assert_eq!(
      parse_command("quit=now").unwrap_err().to_string(),
      "quit assignment requires a target before ="
    );
    assert_eq!(
      parse_command("status == now").unwrap_err().to_string(),
      "unsupported token in command: =="
    );
    assert_eq!(
      parse_command("exit == now").unwrap_err().to_string(),
      "unsupported token in command: =="
    );
    for input in ["status,", "status now,", "exit,", "quit ,"] {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        "comma must be followed by at least one option"
      );
    }
    assert_eq!(
      parse_command("help,verbose").unwrap_err().to_string(),
      "unknown command: help"
    );
    assert_eq!(
      parse_command("help , verbose").unwrap_err().to_string(),
      "help expects at most one command name: help <command>"
    );
  }

  #[test]
  fn matches_python_whitespace_and_malformed_quote_diagnostics() {
    assert_eq!(
      parse_command("\u{1c}").unwrap_err().to_string(),
      "empty command"
    );
    assert_eq!(
      parse_command("status\u{1c}now").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    assert_eq!(
      parse_command("help\u{1c},foo").unwrap(),
      Command::Help {
        topic: Some(",foo".to_owned()),
      }
    );
    assert_eq!(
      parse_command("\"").unwrap_err().to_string(),
      "unterminated quoted string"
    );
    assert_eq!(
      parse_command("'").unwrap_err().to_string(),
      "unterminated quoted string"
    );
    assert_eq!(
      parse_command("`").unwrap_err().to_string(),
      "unterminated quoted identifier"
    );
    assert_eq!(
      parse_command("``").unwrap_err().to_string(),
      "quoted identifier cannot be empty"
    );
    assert_eq!(
      parse_command("`foo``").unwrap_err().to_string(),
      "unterminated quoted identifier"
    );
    assert_eq!(
      parse_command("\"foo\"\"").unwrap_err().to_string(),
      "unterminated quoted string"
    );
  }

  #[test]
  fn parses_by_grouped_read_only_children() {
    assert_eq!(
      parse_command("BY sex: summarize age cost").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["sex".to_owned()],
          command: Box::new(Command::Summarize {
            variables: vec!["age".to_owned(), "cost".to_owned()],
          }),
        },
      }
    );
    assert_eq!(
      parse_command("by sex age: count").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["sex".to_owned(), "age".to_owned()],
          command: Box::new(Command::Count),
        },
      }
    );
    assert_eq!(
      parse_command("by `group:key`: summarize `value col`").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["group:key".to_owned()],
          command: Box::new(Command::Summarize {
            variables: vec!["value col".to_owned()],
          }),
        },
      }
    );
    assert_eq!(
      parse_command("by sex: tabulate outcome").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["sex".to_owned()],
          command: Box::new(Command::Tabulate {
            command: TabulateCommand {
              row_variables: vec!["outcome".to_owned()],
              column_variables: Vec::new(),
              row_percent: false,
              column_percent: false,
              include_missing: false,
              nolabel: false,
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_by_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "by sex summarize age",
        "by expects syntax: by group_vars: command",
      ),
      (
        "by : summarize age",
        "by expects at least one grouping variable",
      ),
      (
        "by sex, age: count",
        "by expects at least one grouping variable",
      ),
      ("by sex:", "by expects a command after :"),
      (
        "by sex: by age: count",
        "nested by commands are not supported",
      ),
      ("by sex: help", "help is not supported inside by commands"),
      (
        "by sex: status",
        "status is not supported inside by commands",
      ),
      (
        "by sex: doctor",
        "doctor is not supported inside by commands",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_sql_syntax() {
    assert_eq!(
      parse_command("sql select sex, avg(bmi) as mean_bmi from active group by sex").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select sex, avg(bmi) as mean_bmi from active group by sex".to_owned(),
          into: None,
        },
      }
    );
    assert_eq!(
      parse_command("sql select sex, avg(bmi) from active group by sex into summary").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select sex, avg(bmi) from active group by sex".to_owned(),
          into: Some("summary".to_owned()),
        },
      }
    );
    assert_eq!(
      parse_command("sql select * from active   into summary").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select * from active".to_owned(),
          into: Some("summary".to_owned()),
        },
      }
    );
    assert_eq!(
      parse_command("sql \"\"\"\nselect sex, count(*) as n\nfrom active\n\"\"\"").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select sex, count(*) as n\nfrom active".to_owned(),
          into: None,
        },
      }
    );
    assert_eq!(
      parse_command("sql \"\"\"\nselect sex, count(*) as n\nfrom active\n\"\"\" into grouped")
        .unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select sex, count(*) as n\nfrom active".to_owned(),
          into: Some("grouped".to_owned()),
        },
      }
    );
    assert_eq!(
      parse_command(
        "sql \"\"\"\nselect value, label\nfrom active\norder by value desc, label\n\"\"\" into ordered"
      )
      .unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select value, label\nfrom active\norder by value desc, label".to_owned(),
          into: Some("ordered".to_owned()),
        },
      }
    );
    assert_eq!(
      parse_command("sql select * from active INTO summary").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select * from active".to_owned(),
          into: Some("summary".to_owned()),
        },
      }
    );
    assert_eq!(
      parse_command("sql into active").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "into active".to_owned(),
          into: None,
        },
      }
    );
    assert_eq!(
      parse_command("sql select 1 - 1").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select 1 - 1".to_owned(),
          into: None,
        },
      }
    );
    assert_eq!(
      parse_command("sql select 1; into foo").unwrap(),
      Command::Sql {
        command: SqlCommand {
          query: "select 1;".to_owned(),
          into: Some("foo".to_owned()),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_sql_syntax_with_exact_diagnostics() {
    let cases = [
      ("sql", "sql expects a query"),
      ("sql   ", "sql expects a query"),
      (
        "sql \"\"\"select * from active",
        "sql multiline query is missing closing \"\"\"",
      ),
      ("sql \"\"\"\"\"\"", "sql expects a query"),
      ("sql   \"\"\"   \"\"\"", "sql expects a query"),
      (
        "sql select * from active into",
        "sql into expects syntax: sql <query> into <table>",
      ),
      (
        "sql select * from into",
        "sql into expects syntax: sql <query> into <table>",
      ),
      (
        "sql select * from active into active",
        "sql into cannot use reserved table name: active",
      ),
      (
        "sql select * from active into __tabdat_next",
        "sql into cannot use reserved table name: __tabdat_next",
      ),
      (
        "sql select * from active into bad-name",
        "sql into table name must be an identifier",
      ),
      ("sql:select 1", "unsupported token in command: :"),
      ("sql,foo", "unknown command: sql"),
      ("sql=1", "sql assignment requires a target before ="),
      ("sql==1", "unsupported token in command: =="),
      (
        "sql \"\"\"select * from active\"\"\" extra",
        "sql into expects syntax: sql <query> into <table>",
      ),
      (
        "sql \"\"\"select * from active\"\"\" into",
        "sql into expects syntax: sql <query> into <table>",
      ),
      (
        "sql \"\"\"select * from active\"\"\" into foo bar",
        "sql into expects syntax: sql <query> into <table>",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_regress_syntax() {
    assert_eq!(
      parse_command("regress cost age bmi").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          estimator: RegressEstimator::Ols,
          weight_variable: None,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("regress cost age, robust").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          estimator: RegressEstimator::Ols,
          weight_variable: None,
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("regress cost age, cluster(sex)").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          estimator: RegressEstimator::Ols,
          weight_variable: None,
          robust: false,
          cluster_variable: Some("sex".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("regress cost age, noconstant").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          estimator: RegressEstimator::Ols,
          weight_variable: None,
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("regress cost age, wls(weight) cluster(firm)").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          estimator: RegressEstimator::Wls,
          weight_variable: Some("weight".to_owned()),
          robust: false,
          cluster_variable: Some("firm".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("regress cost age, gls(sigma) robust").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          estimator: RegressEstimator::Gls,
          weight_variable: Some("sigma".to_owned()),
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("REGRESS `total cost` `age value` 'bmi value', noconstant robust").unwrap(),
      Command::Regress {
        command: RegressCommand {
          outcome: "total cost".to_owned(),
          predictors: vec!["age value".to_owned(), "bmi value".to_owned()],
          estimator: RegressEstimator::Ols,
          weight_variable: None,
          robust: true,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_regress_syntax_with_exact_diagnostics() {
    let cases = [
      ("regress", "regress expects syntax: regress <y> <xvars>"),
      (
        "regress cost",
        "regress expects syntax: regress <y> <xvars>",
      ),
      (
        "regress cost age if age > 18",
        "regress expects syntax: regress <y> <xvars>",
      ),
      (
        "regress cost age, robust cluster(sex)",
        "regress cannot combine robust and cluster",
      ),
      (
        "regress cost age, cluster",
        "regress option cluster expects variables",
      ),
      (
        "regress cost age, cluster()",
        "option cluster expects at least one value",
      ),
      (
        "regress cost age, cluster(sex firm)",
        "regress option cluster expects one variable",
      ),
      (
        "regress cost age, cluster(a) cluster(b)",
        "regress option cluster may only be supplied once",
      ),
      (
        "regress cost age, wls",
        "regress option wls expects variables",
      ),
      (
        "regress cost age, wls()",
        "option wls expects at least one value",
      ),
      (
        "regress cost age, wls(age bmi)",
        "regress option wls expects one variable",
      ),
      (
        "regress cost age, wls(a) wls(b)",
        "regress option wls may only be supplied once",
      ),
      (
        "regress cost age, gls",
        "regress option gls expects variables",
      ),
      (
        "regress cost age, gls()",
        "option gls expects at least one value",
      ),
      (
        "regress cost age, gls(age bmi)",
        "regress option gls expects one variable",
      ),
      (
        "regress cost age, gls(a) gls(b)",
        "regress option gls may only be supplied once",
      ),
      (
        "regress cost age, wls(age) gls(sigma)",
        "regress cannot combine wls and gls",
      ),
      (
        "regress cost age, robust=true",
        "regress option robust does not accept a value",
      ),
      (
        "regress cost age, noconstant=true",
        "regress option noconstant does not accept a value",
      ),
      (
        "regress cost age, invalid",
        "regress unsupported option: invalid",
      ),
      (
        "regress cost age,",
        "comma must be followed by at least one option",
      ),
      ("regress,", "comma must be followed by at least one option"),
      ("regress=", "regress assignment requires a target before ="),
      ("regress==", "unsupported token in command: =="),
      ("regress:cost age", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_logit_and_probit_syntax() {
    assert_eq!(
      parse_command("logit outcome x1 x2").unwrap(),
      Command::Logit {
        command: LogitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("logit outcome x1, robust").unwrap(),
      Command::Logit {
        command: LogitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("LOGIT outcome x1, cluster(group_id)").unwrap(),
      Command::Logit {
        command: LogitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("logit outcome x1, noconstant").unwrap(),
      Command::Logit {
        command: LogitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("probit outcome x1 x2").unwrap(),
      Command::Probit {
        command: ProbitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("probit outcome x1, robust").unwrap(),
      Command::Probit {
        command: ProbitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("PROBIT outcome x1, cluster(group_id)").unwrap(),
      Command::Probit {
        command: ProbitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("probit outcome x1, noconstant").unwrap(),
      Command::Probit {
        command: ProbitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_logit_and_probit_syntax_with_exact_diagnostics() {
    let cases = [
      ("logit", "logit expects syntax: logit <y> <xvars>"),
      ("logit y", "logit expects syntax: logit <y> <xvars>"),
      (
        "logit y x if y == 1",
        "logit expects syntax: logit <y> <xvars>",
      ),
      (
        "logit y x, robust cluster(group)",
        "logit cannot combine robust and cluster",
      ),
      (
        "logit y x, cluster",
        "logit option cluster expects variables",
      ),
      (
        "logit y x, cluster()",
        "option cluster expects at least one value",
      ),
      (
        "logit y x, cluster(group firm)",
        "logit option cluster expects one variable",
      ),
      (
        "logit y x, cluster(a) cluster(b)",
        "logit option cluster may only be supplied once",
      ),
      (
        "logit y x, robust=true",
        "logit option robust does not accept a value",
      ),
      (
        "logit y x, noconstant=true",
        "logit option noconstant does not accept a value",
      ),
      ("logit y x, invalid", "logit unsupported option: invalid"),
      (
        "logit y x,",
        "comma must be followed by at least one option",
      ),
      ("logit,", "comma must be followed by at least one option"),
      ("logit=", "logit assignment requires a target before ="),
      ("logit==", "unsupported token in command: =="),
      ("logit:y x", "unsupported token in command: :"),
      ("probit", "probit expects syntax: probit <y> <xvars>"),
      ("probit y", "probit expects syntax: probit <y> <xvars>"),
      (
        "probit y x if y == 1",
        "probit expects syntax: probit <y> <xvars>",
      ),
      (
        "probit y x, robust cluster(group)",
        "probit cannot combine robust and cluster",
      ),
      (
        "probit y x, cluster",
        "probit option cluster expects variables",
      ),
      (
        "probit y x, cluster()",
        "option cluster expects at least one value",
      ),
      (
        "probit y x, cluster(group firm)",
        "probit option cluster expects one variable",
      ),
      (
        "probit y x, cluster(a) cluster(b)",
        "probit option cluster may only be supplied once",
      ),
      (
        "probit y x, robust=true",
        "probit option robust does not accept a value",
      ),
      (
        "probit y x, noconstant=true",
        "probit option noconstant does not accept a value",
      ),
      ("probit y x, invalid", "probit unsupported option: invalid"),
      (
        "probit y x,",
        "comma must be followed by at least one option",
      ),
      ("probit,", "comma must be followed by at least one option"),
      ("probit=", "probit assignment requires a target before ="),
      ("probit==", "unsupported token in command: =="),
      ("probit:y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_poisson_and_nbreg_syntax() {
    assert_eq!(
      parse_command("poisson outcome x1 x2").unwrap(),
      Command::Poisson {
        command: PoissonCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("poisson outcome x1, robust").unwrap(),
      Command::Poisson {
        command: PoissonCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("POISSON outcome x1, cluster(group_id)").unwrap(),
      Command::Poisson {
        command: PoissonCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("poisson outcome x1, noconstant").unwrap(),
      Command::Poisson {
        command: PoissonCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("poisson `y var` 'x var', cluster(`firm id`)").unwrap(),
      Command::Poisson {
        command: PoissonCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          robust: false,
          cluster_variable: Some("firm id".to_owned()),
          include_intercept: true,
        },
      }
    );

    assert_eq!(
      parse_command("nbreg outcome x1 x2").unwrap(),
      Command::Nbreg {
        command: NbregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("nbreg outcome x1, robust").unwrap(),
      Command::Nbreg {
        command: NbregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("NBREG outcome x1, cluster(group_id)").unwrap(),
      Command::Nbreg {
        command: NbregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("nbreg outcome x1, noconstant").unwrap(),
      Command::Nbreg {
        command: NbregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("nbreg `y var` 'x var', cluster(`firm id`)").unwrap(),
      Command::Nbreg {
        command: NbregCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          robust: false,
          cluster_variable: Some("firm id".to_owned()),
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_poisson_and_nbreg_syntax_with_exact_diagnostics() {
    let cases = [
      ("poisson", "poisson expects syntax: poisson <y> <xvars>"),
      ("poisson y", "poisson expects syntax: poisson <y> <xvars>"),
      (
        "poisson y x if y > 0",
        "poisson expects syntax: poisson <y> <xvars>",
      ),
      (
        "poisson y x, robust cluster(group)",
        "poisson cannot combine robust and cluster",
      ),
      (
        "poisson y x, cluster",
        "poisson option cluster expects variables",
      ),
      (
        "poisson y x, cluster()",
        "option cluster expects at least one value",
      ),
      (
        "poisson y x, cluster(group firm)",
        "poisson option cluster expects one variable",
      ),
      (
        "poisson y x, cluster(a) cluster(b)",
        "poisson option cluster may only be supplied once",
      ),
      (
        "poisson y x, robust=true",
        "poisson option robust does not accept a value",
      ),
      (
        "poisson y x, noconstant=true",
        "poisson option noconstant does not accept a value",
      ),
      (
        "poisson y x, invalid",
        "poisson unsupported option: invalid",
      ),
      (
        "poisson y x,",
        "comma must be followed by at least one option",
      ),
      ("poisson,", "comma must be followed by at least one option"),
      ("poisson=", "poisson assignment requires a target before ="),
      ("poisson==", "unsupported token in command: =="),
      ("poisson:y x", "unsupported token in command: :"),
      ("nbreg", "nbreg expects syntax: nbreg <y> <xvars>"),
      ("nbreg y", "nbreg expects syntax: nbreg <y> <xvars>"),
      (
        "nbreg y x if y > 0",
        "nbreg expects syntax: nbreg <y> <xvars>",
      ),
      (
        "nbreg y x, robust cluster(group)",
        "nbreg cannot combine robust and cluster",
      ),
      (
        "nbreg y x, cluster",
        "nbreg option cluster expects variables",
      ),
      (
        "nbreg y x, cluster()",
        "option cluster expects at least one value",
      ),
      (
        "nbreg y x, cluster(group firm)",
        "nbreg option cluster expects one variable",
      ),
      (
        "nbreg y x, cluster(a) cluster(b)",
        "nbreg option cluster may only be supplied once",
      ),
      (
        "nbreg y x, robust=true",
        "nbreg option robust does not accept a value",
      ),
      (
        "nbreg y x, noconstant=true",
        "nbreg option noconstant does not accept a value",
      ),
      ("nbreg y x, invalid", "nbreg unsupported option: invalid"),
      (
        "nbreg y x,",
        "comma must be followed by at least one option",
      ),
      ("nbreg,", "comma must be followed by at least one option"),
      ("nbreg=", "nbreg assignment requires a target before ="),
      ("nbreg==", "unsupported token in command: =="),
      ("nbreg:y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_zip_and_zinb_syntax() {
    assert_eq!(
      parse_command("zip outcome x1 x2, inflate(z1 z2)").unwrap(),
      Command::Zip {
        command: ZipCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          inflate_predictors: vec!["z1".to_owned(), "z2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("zip outcome x1, inflate(z1) robust").unwrap(),
      Command::Zip {
        command: ZipCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("ZIP outcome x1, inflate(z1) cluster(group_id)").unwrap(),
      Command::Zip {
        command: ZipCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("zip outcome x1, inflate(z1) noconstant").unwrap(),
      Command::Zip {
        command: ZipCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("zip `y var` 'x var', inflate(`z var`) cluster(`firm id`)").unwrap(),
      Command::Zip {
        command: ZipCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          inflate_predictors: vec!["z var".to_owned()],
          robust: false,
          cluster_variable: Some("firm id".to_owned()),
          include_intercept: true,
        },
      }
    );

    assert_eq!(
      parse_command("zinb outcome x1 x2, inflate(z1 z2)").unwrap(),
      Command::Zinb {
        command: ZinbCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          inflate_predictors: vec!["z1".to_owned(), "z2".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("zinb outcome x1, inflate(z1) robust").unwrap(),
      Command::Zinb {
        command: ZinbCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("ZINB outcome x1, inflate(z1) cluster(group_id)").unwrap(),
      Command::Zinb {
        command: ZinbCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("zinb outcome x1, inflate(z1) noconstant").unwrap(),
      Command::Zinb {
        command: ZinbCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          inflate_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("zinb `y var` 'x var', inflate(`z var`) cluster(`firm id`)").unwrap(),
      Command::Zinb {
        command: ZinbCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          inflate_predictors: vec!["z var".to_owned()],
          robust: false,
          cluster_variable: Some("firm id".to_owned()),
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_zip_and_zinb_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "zip",
        "zip expects syntax: zip <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zip y",
        "zip expects syntax: zip <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zip y x",
        "zip option inflate expects one-or-more variables",
      ),
      (
        "zip y x if y > 0",
        "zip expects syntax: zip <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zip y x if y > 0, inflate(z)",
        "zip expects syntax: zip <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zip y x = 1",
        "zip expects syntax: zip <y> <xvars>, inflate(<zvars>)",
      ),
      ("zip y x, inflate", "zip option inflate expects variables"),
      (
        "zip y x, inflate()",
        "option inflate expects at least one value",
      ),
      (
        "zip y x, inflate(z1) inflate(z2)",
        "zip option inflate may only be supplied once",
      ),
      (
        "zip y x, inflate(z) robust cluster(group)",
        "zip cannot combine robust and cluster",
      ),
      (
        "zip y x, inflate(z) cluster",
        "zip option cluster expects variables",
      ),
      (
        "zip y x, inflate(z) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "zip y x, inflate(z) cluster(group firm)",
        "zip option cluster expects one variable",
      ),
      (
        "zip y x, inflate(z) cluster(a) cluster(b)",
        "zip option cluster may only be supplied once",
      ),
      (
        "zip y x, inflate(z) robust=true",
        "zip option robust does not accept a value",
      ),
      (
        "zip y x, inflate(z) noconstant=true",
        "zip option noconstant does not accept a value",
      ),
      (
        "zip y x, inflate(z) invalid",
        "zip unsupported option: invalid",
      ),
      ("zip y x,", "comma must be followed by at least one option"),
      ("zip,", "comma must be followed by at least one option"),
      ("zip=", "zip assignment requires a target before ="),
      ("zip==", "unsupported token in command: =="),
      ("zip:y x", "unsupported token in command: :"),
      (
        "zinb",
        "zinb expects syntax: zinb <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zinb y",
        "zinb expects syntax: zinb <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zinb y x",
        "zinb option inflate expects one-or-more variables",
      ),
      (
        "zinb y x if y > 0",
        "zinb expects syntax: zinb <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zinb y x if y > 0, inflate(z)",
        "zinb expects syntax: zinb <y> <xvars>, inflate(<zvars>)",
      ),
      (
        "zinb y x = 1",
        "zinb expects syntax: zinb <y> <xvars>, inflate(<zvars>)",
      ),
      ("zinb y x, inflate", "zinb option inflate expects variables"),
      (
        "zinb y x, inflate()",
        "option inflate expects at least one value",
      ),
      (
        "zinb y x, inflate(z1) inflate(z2)",
        "zinb option inflate may only be supplied once",
      ),
      (
        "zinb y x, inflate(z) robust cluster(group)",
        "zinb cannot combine robust and cluster",
      ),
      (
        "zinb y x, inflate(z) cluster",
        "zinb option cluster expects variables",
      ),
      (
        "zinb y x, inflate(z) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "zinb y x, inflate(z) cluster(group firm)",
        "zinb option cluster expects one variable",
      ),
      (
        "zinb y x, inflate(z) cluster(a) cluster(b)",
        "zinb option cluster may only be supplied once",
      ),
      (
        "zinb y x, inflate(z) robust=true",
        "zinb option robust does not accept a value",
      ),
      (
        "zinb y x, inflate(z) noconstant=true",
        "zinb option noconstant does not accept a value",
      ),
      (
        "zinb y x, inflate(z) invalid",
        "zinb unsupported option: invalid",
      ),
      ("zinb y x,", "comma must be followed by at least one option"),
      ("zinb,", "comma must be followed by at least one option"),
      ("zinb=", "zinb assignment requires a target before ="),
      ("zinb==", "unsupported token in command: =="),
      ("zinb:y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_qreg_syntax() {
    assert_eq!(
      parse_command("qreg outcome x1 x2").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          quantile: "0.5".to_owned(),
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("qreg outcome x1, quantile(0.25)").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          quantile: "0.25".to_owned(),
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("qreg outcome x1, quantile=0.75").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          quantile: "0.75".to_owned(),
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("qreg outcome x1, robust").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          quantile: "0.5".to_owned(),
          robust: true,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("qreg outcome x1, noconstant").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          quantile: "0.5".to_owned(),
          robust: false,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("QREG `y var` 'x var', quantile(0.1) robust noconstant").unwrap(),
      Command::Qreg {
        command: QregCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          quantile: "0.1".to_owned(),
          robust: true,
          include_intercept: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_qreg_syntax_with_exact_diagnostics() {
    let cases = [
      ("qreg", "qreg expects syntax: qreg <y> <xvars>"),
      ("qreg y", "qreg expects syntax: qreg <y> <xvars>"),
      ("qreg y x if y > 0", "qreg expects syntax: qreg <y> <xvars>"),
      ("qreg y x = 1", "qreg expects syntax: qreg <y> <xvars>"),
      (
        "qreg y x, cluster(group)",
        "qreg unsupported option: cluster",
      ),
      (
        "qreg y x, quantile()",
        "option quantile expects at least one value",
      ),
      (
        "qreg y x, quantile",
        "qreg option quantile expects a numeric value",
      ),
      (
        "qreg y x, quantile(abc)",
        "option quantile expects a numeric value",
      ),
      (
        "qreg y x, quantile=abc",
        "qreg option quantile expects a numeric value",
      ),
      (
        "qreg y x, quantile(0)",
        "qreg option quantile must be between 0 and 1",
      ),
      (
        "qreg y x, quantile(1)",
        "qreg option quantile must be between 0 and 1",
      ),
      (
        "qreg y x, quantile(1.2)",
        "qreg option quantile must be between 0 and 1",
      ),
      (
        "qreg y x, quantile(-0.1)",
        "qreg option quantile must be between 0 and 1",
      ),
      (
        "qreg y x, quantile=1.2",
        "qreg option quantile must be between 0 and 1",
      ),
      (
        "qreg y x, quantile(0.25) quantile(0.5)",
        "qreg option quantile may only be supplied once",
      ),
      (
        "qreg y x, robust=true",
        "qreg option robust does not accept a value",
      ),
      (
        "qreg y x, noconstant=true",
        "qreg option noconstant does not accept a value",
      ),
      ("qreg y x, invalid", "qreg unsupported option: invalid"),
      ("qreg y x,", "comma must be followed by at least one option"),
      ("qreg,", "comma must be followed by at least one option"),
      ("qreg=", "qreg assignment requires a target before ="),
      ("qreg==", "unsupported token in command: =="),
      ("qreg:y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_tobit_syntax() {
    assert_eq!(
      parse_command("tobit outcome x1, ll(0)").unwrap(),
      Command::Tobit {
        command: TobitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          lower_limit: "0".to_owned(),
          upper_limit: None,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("tobit outcome x1 x2, ll(-1) ul(10) robust").unwrap(),
      Command::Tobit {
        command: TobitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          lower_limit: "-1".to_owned(),
          upper_limit: Some("10".to_owned()),
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("tobit outcome x1, ll(0) cluster(group_id) noconstant").unwrap(),
      Command::Tobit {
        command: TobitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          lower_limit: "0".to_owned(),
          upper_limit: None,
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("tobit outcome x1, ll=0 ul=10").unwrap(),
      Command::Tobit {
        command: TobitCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          lower_limit: "0".to_owned(),
          upper_limit: Some("10".to_owned()),
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("TOBIT `y var` 'x var', ll(0.5) ul(9.5) robust noconstant").unwrap(),
      Command::Tobit {
        command: TobitCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          lower_limit: "0.5".to_owned(),
          upper_limit: Some("9.5".to_owned()),
          robust: true,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_tobit_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "tobit",
        "tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]",
      ),
      (
        "tobit y",
        "tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]",
      ),
      ("tobit y x", "tobit option ll expects one numeric value"),
      (
        "tobit y x if y > 0",
        "tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]",
      ),
      (
        "tobit y x = 1",
        "tobit expects syntax: tobit <y> <xvars>, ll(<num>) [ul(<num>)]",
      ),
      ("tobit y x, ll()", "option ll expects at least one value"),
      ("tobit y x, ll(low)", "option ll expects a numeric value"),
      (
        "tobit y x, ul(1)",
        "tobit option ll expects one numeric value",
      ),
      (
        "tobit y x, ll(0) robust cluster(group)",
        "tobit cannot combine robust and cluster",
      ),
      (
        "tobit y x, ll(0) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "tobit y x, ll(0) cluster(group firm)",
        "tobit option cluster expects one variable",
      ),
      (
        "tobit y x, ll(0) robust=true",
        "tobit option robust does not accept a value",
      ),
      (
        "tobit y x, ll(0) noconstant=true",
        "tobit option noconstant does not accept a value",
      ),
      ("tobit y x, ll", "tobit option ll expects a numeric value"),
      (
        "tobit y x, ll(0) ul",
        "tobit option ul expects a numeric value",
      ),
      (
        "tobit y x, ll(0) ul()",
        "option ul expects at least one value",
      ),
      (
        "tobit y x, ll(0) ul(high)",
        "option ul expects a numeric value",
      ),
      (
        "tobit y x, ll(0) ll(1)",
        "tobit option ll may only be supplied once",
      ),
      (
        "tobit y x, ll(0) ul(1) ul(2)",
        "tobit option ul may only be supplied once",
      ),
      (
        "tobit y x, ll(0) cluster(c1) cluster(c2)",
        "tobit option cluster may only be supplied once",
      ),
      (
        "tobit y x, ll(0) cluster",
        "tobit option cluster expects variables",
      ),
      ("tobit y x, ll(0) foo", "tobit unsupported option: foo"),
      ("tobit:", "unsupported token in command: :"),
      ("tobit=", "tobit assignment requires a target before ="),
      ("tobit==", "unsupported token in command: =="),
      ("tobit,", "comma must be followed by at least one option"),
      (
        "tobit y x,",
        "comma must be followed by at least one option",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_heckman_syntax() {
    assert_eq!(
      parse_command("heckman outcome x1, selectdep(selected) select(z1)").unwrap(),
      Command::Heckman {
        command: HeckmanCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          selection_dependent: "selected".to_owned(),
          selection_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("heckman outcome x1 x2, selectdep(selected) select(z1 z2) robust").unwrap(),
      Command::Heckman {
        command: HeckmanCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          selection_dependent: "selected".to_owned(),
          selection_predictors: vec!["z1".to_owned(), "z2".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command(
        "heckman outcome x1, selectdep(selected) select(z1) cluster(group_id) noconstant"
      )
      .unwrap(),
      Command::Heckman {
        command: HeckmanCommand {
          outcome: "outcome".to_owned(),
          predictors: vec!["x1".to_owned()],
          selection_dependent: "selected".to_owned(),
          selection_predictors: vec!["z1".to_owned()],
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command(
        "HECKMAN `y var` 'x var', selectdep(`sel var`) select(`z var`) robust noconstant"
      )
      .unwrap(),
      Command::Heckman {
        command: HeckmanCommand {
          outcome: "y var".to_owned(),
          predictors: vec!["x var".to_owned()],
          selection_dependent: "sel var".to_owned(),
          selection_predictors: vec!["z var".to_owned()],
          robust: true,
          cluster_variable: None,
          include_intercept: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_heckman_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "heckman",
        "heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)",
      ),
      (
        "heckman y",
        "heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)",
      ),
      (
        "heckman y x",
        "heckman option selectdep expects one variable",
      ),
      (
        "heckman y x if y > 0",
        "heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)",
      ),
      (
        "heckman y x = 1",
        "heckman expects syntax: heckman <y> <xvars>, selectdep(<var>) select(<vars>)",
      ),
      (
        "heckman y x, selectdep() select(z)",
        "option selectdep expects at least one value",
      ),
      (
        "heckman y x, selectdep(s t) select(z)",
        "heckman option selectdep expects one variable",
      ),
      (
        "heckman y x, selectdep(s)",
        "heckman option select expects at least one variable",
      ),
      (
        "heckman y x, select(s)",
        "heckman option selectdep expects one variable",
      ),
      (
        "heckman y x, selectdep(s) select()",
        "option select expects at least one value",
      ),
      (
        "heckman y x, selectdep(s) select(z) robust cluster(group)",
        "heckman cannot combine robust and cluster",
      ),
      (
        "heckman y x, selectdep(s) select(z) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "heckman y x, selectdep(s) select(z) cluster(group firm)",
        "heckman option cluster expects one variable",
      ),
      (
        "heckman y x, selectdep(s) select(z) robust=true",
        "heckman option robust does not accept a value",
      ),
      (
        "heckman y x, selectdep(s) select(z) noconstant=true",
        "heckman option noconstant does not accept a value",
      ),
      (
        "heckman y x, selectdep select(z)",
        "heckman option selectdep expects variables",
      ),
      (
        "heckman y x, selectdep(s) select",
        "heckman option select expects variables",
      ),
      (
        "heckman y x, selectdep(s) select(z) cluster",
        "heckman option cluster expects variables",
      ),
      (
        "heckman y x, selectdep(s) selectdep(s2) select(z)",
        "heckman option selectdep may only be supplied once",
      ),
      (
        "heckman y x, selectdep(s) select(z) select(z2)",
        "heckman option select may only be supplied once",
      ),
      (
        "heckman y x, selectdep(s) select(z) cluster(c1) cluster(c2)",
        "heckman option cluster may only be supplied once",
      ),
      (
        "heckman y x, selectdep(s) select(z) foo",
        "heckman unsupported option: foo",
      ),
      (
        "heckman y x, selectdep('s var') select(z)",
        "option selectdep values must be identifiers",
      ),
      (
        "heckman y x, selectdep(s) select('z var')",
        "option select values must be identifiers",
      ),
      (
        "heckman y x, selectdep(s) select(z) cluster('c var')",
        "option cluster values must be identifiers",
      ),
      ("heckman:", "unsupported token in command: :"),
      ("heckman=", "heckman assignment requires a target before ="),
      ("heckman==", "unsupported token in command: =="),
      ("heckman,", "comma must be followed by at least one option"),
      (
        "heckman y x,",
        "comma must be followed by at least one option",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_nl_syntax() {
    assert_eq!(
      parse_command("nl y = a + b*x, params(a b) start(1 2)").unwrap(),
      Command::Nl {
        command: NlCommand {
          outcome: "y".to_owned(),
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("a".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("b".to_owned())),
              operator: GenerateBinaryOperator::Multiply,
              right: Box::new(GenerateExpression::Identifier("x".to_owned())),
            }),
          },
          parameter_names: vec!["a".to_owned(), "b".to_owned()],
          start_values: vec!["1".to_owned(), "2".to_owned()],
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("nl outcome = -b0 + b1*x1, params(b0 b1) start(-0.5 1.5) robust noconstant")
        .unwrap(),
      Command::Nl {
        command: NlCommand {
          outcome: "outcome".to_owned(),
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::UnaryMinus(Box::new(
              GenerateExpression::Identifier("b0".to_owned())
            ))),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("b1".to_owned())),
              operator: GenerateBinaryOperator::Multiply,
              right: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            }),
          },
          parameter_names: vec!["b0".to_owned(), "b1".to_owned()],
          start_values: vec!["-0.5".to_owned(), "1.5".to_owned()],
          robust: true,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("NL `y var` = exp(a + b*`x var`), params(a b) start(1 2) robust").unwrap(),
      Command::Nl {
        command: NlCommand {
          outcome: "y var".to_owned(),
          expression: GenerateExpression::FunctionCall {
            name: "exp".to_owned(),
            arguments: vec![GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("a".to_owned())),
              operator: GenerateBinaryOperator::Add,
              right: Box::new(GenerateExpression::Binary {
                left: Box::new(GenerateExpression::Identifier("b".to_owned())),
                operator: GenerateBinaryOperator::Multiply,
                right: Box::new(GenerateExpression::Identifier("x var".to_owned())),
              }),
            }],
          },
          parameter_names: vec!["a".to_owned(), "b".to_owned()],
          start_values: vec!["1".to_owned(), "2".to_owned()],
          robust: true,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("nl y = a, start(1) params(a)").unwrap(),
      Command::Nl {
        command: NlCommand {
          outcome: "y".to_owned(),
          expression: GenerateExpression::Identifier("a".to_owned()),
          parameter_names: vec!["a".to_owned()],
          start_values: vec!["1".to_owned()],
          robust: false,
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_nl_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "nl",
        "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)",
      ),
      (
        "nl y x",
        "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)",
      ),
      (
        "nl y",
        "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)",
      ),
      (
        "nl if y > 0",
        "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)",
      ),
      (
        "nl y if y > 0",
        "nl expects syntax: nl <y> = <expr>, params(<params>) start(<values>)",
      ),
      (
        "nl y = x",
        "nl option params expects one-or-more parameter names",
      ),
      (
        "nl y = a + b*x, params(a b)",
        "nl option start expects one-or-more numeric values",
      ),
      (
        "nl y = a + b*x, start(1 2)",
        "nl option params expects one-or-more parameter names",
      ),
      (
        "nl y = a + b*x, params(a b) start(1)",
        "nl option start count must match params count",
      ),
      (
        "nl y = a + b*x, params(a b) start(1 two)",
        "option start values must be numeric",
      ),
      (
        "nl y = a + b*x, params(a a) start(1 2)",
        "nl option params must not repeat parameter names",
      ),
      (
        "nl y = a + b*x if y > 0, params(a b) start(1 2)",
        "duplicate if clause",
      ),
      (
        "nl y = a + b*x, robust=true params(a b) start(1 2)",
        "nl option robust does not accept a value",
      ),
      (
        "nl y = a + b*x, noconstant=true params(a b) start(1 2)",
        "nl option noconstant does not accept a value",
      ),
      (
        "nl y = a + b*x, params() start(1)",
        "option params expects at least one value",
      ),
      (
        "nl y = a + b*x, params(a) start()",
        "option start expects at least one value",
      ),
      (
        "nl y = a + b*x, params(a) start(1) cluster(c)",
        "nl unsupported option: cluster",
      ),
      (
        "nl y = a + b*x, params(a) start(1) params(b)",
        "nl option params may only be supplied once",
      ),
      (
        "nl y = a + b*x, params(a) start(1) start(2)",
        "nl option start may only be supplied once",
      ),
      ("nl:", "unsupported token in command: :"),
      ("nl=", "nl assignment requires a target before ="),
      ("nl==", "unsupported token in command: =="),
      ("nl = x", "nl assignment requires a target before ="),
      ("nl y =", "nl assignment requires an expression after ="),
      ("nl == x", "unsupported token in command: =="),
      ("nl y == x", "unsupported token in command: =="),
      (
        "nl y = a + b*x,",
        "nl option params expects one-or-more parameter names",
      ),
      (
        "nl y = a + b*x, params",
        "nl option params expects variables",
      ),
      (
        "nl y = a + b*x, params=a start=1",
        "nl option params expects variables",
      ),
      (
        "nl y = a + b*x, params(a) start",
        "nl option start expects variables",
      ),
      (
        "nl y = a + b*x, params('a') start(1)",
        "option params values must be identifiers",
      ),
      (
        "nl y = a + b*x, params(a) start(1) foo",
        "nl unsupported option: foo",
      ),
      (
        "nl y = a + b*x, params(a) start(1) foo bar",
        "nl unsupported option: bar, foo",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_streg_syntax() {
    assert_eq!(
      parse_command("streg time age income, failure(died) dist(weibull)").unwrap(),
      Command::Streg {
        command: StregCommand {
          time_variable: "time".to_owned(),
          predictors: vec!["age".to_owned(), "income".to_owned()],
          failure_variable: "died".to_owned(),
          distribution: StregDistribution::Weibull,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("streg time age, failure(died) dist(exponential) robust").unwrap(),
      Command::Streg {
        command: StregCommand {
          time_variable: "time".to_owned(),
          predictors: vec!["age".to_owned()],
          failure_variable: "died".to_owned(),
          distribution: StregDistribution::Exponential,
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("streg time age, failure(died) dist(weibull) cluster(group_id) noconstant")
        .unwrap(),
      Command::Streg {
        command: StregCommand {
          time_variable: "time".to_owned(),
          predictors: vec!["age".to_owned()],
          failure_variable: "died".to_owned(),
          distribution: StregDistribution::Weibull,
          robust: false,
          cluster_variable: Some("group_id".to_owned()),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("STREG `time var` `age var`, failure(`event var`) dist(Weibull)").unwrap(),
      Command::Streg {
        command: StregCommand {
          time_variable: "time var".to_owned(),
          predictors: vec!["age var".to_owned()],
          failure_variable: "event var".to_owned(),
          distribution: StregDistribution::Weibull,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("streg time age, dist(EXPONENTIAL) failure(died)").unwrap(),
      Command::Streg {
        command: StregCommand {
          time_variable: "time".to_owned(),
          predictors: vec!["age".to_owned()],
          failure_variable: "died".to_owned(),
          distribution: StregDistribution::Exponential,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_streg_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "streg",
        "streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)",
      ),
      (
        "streg time",
        "streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)",
      ),
      (
        "streg time age",
        "streg option failure expects one variable",
      ),
      (
        "streg time age if time > 0, failure(event) dist(weibull)",
        "streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)",
      ),
      (
        "streg time = age",
        "streg expects syntax: streg <time_var> <xvars>, failure(<event>) dist(...)",
      ),
      (
        "streg time age, failure() dist(weibull)",
        "option failure expects at least one value",
      ),
      (
        "streg time age, failure(event event2) dist(weibull)",
        "streg option failure expects one variable",
      ),
      (
        "streg time age, failure(event) dist()",
        "option dist expects at least one value",
      ),
      (
        "streg time age, failure(event) dist(loglogistic)",
        "streg option dist must be weibull or exponential",
      ),
      (
        "streg time age, failure(event) dist(weibull) robust cluster(group)",
        "streg cannot combine robust and cluster",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster(group firm)",
        "streg option cluster expects one variable",
      ),
      (
        "streg time age, failure(event) dist(weibull) robust=true",
        "streg option robust does not accept a value",
      ),
      (
        "streg time age, failure(event) dist(weibull) noconstant=true",
        "streg option noconstant does not accept a value",
      ),
      (
        "streg time age, failure(event) dist(weibull) failure(event2)",
        "streg option failure may only be supplied once",
      ),
      (
        "streg time age, failure(event) dist(weibull) dist(exponential)",
        "streg option dist may only be supplied once",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster(a) cluster(b)",
        "streg option cluster may only be supplied once",
      ),
      (
        "streg time age, failure(event) dist(weibull) foo",
        "streg unsupported option: foo",
      ),
      (
        "streg time age, failure(event) dist(weibull) foo bar",
        "streg unsupported option: bar, foo",
      ),
      ("streg:", "unsupported token in command: :"),
      ("streg=", "streg assignment requires a target before ="),
      ("streg==", "unsupported token in command: =="),
      ("streg=foo", "streg assignment requires a target before ="),
      ("streg = foo", "streg assignment requires a target before ="),
      ("streg,", "comma must be followed by at least one option"),
      (
        "streg time age,",
        "comma must be followed by at least one option",
      ),
      (
        "streg time age, failure",
        "streg option failure expects variables",
      ),
      (
        "streg time age, failure=event dist=weibull",
        "streg option failure expects variables",
      ),
      (
        "streg time age, failure(event) dist",
        "streg option dist expects variables",
      ),
      (
        "streg time age, failure(event) dist=weibull",
        "streg option dist expects variables",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster",
        "streg option cluster expects variables",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster=group",
        "streg option cluster expects variables",
      ),
      (
        "streg time age, failure('event') dist(weibull)",
        "option failure values must be identifiers",
      ),
      (
        "streg time age, failure(event) dist('weibull')",
        "option dist values must be identifiers",
      ),
      (
        "streg time age, failure(event) dist(weibull) cluster('group')",
        "option cluster values must be identifiers",
      ),
      (
        "streg time age, failure(event)",
        "streg option dist expects one value",
      ),
      (
        "streg time age, dist(weibull)",
        "streg option failure expects one variable",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_spregress_syntax() {
    assert_eq!(
      parse_command("spregress y x1 x2, coord(lat lon)").unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "y".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          model_type: SpregressModelType::Lag,
          coord_variables: Some(("lat".to_owned(), "lon".to_owned())),
          knn: Some(5),
          weights_file: None,
          id_variable: None,
          contiguity: None,
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("spregress y x, coord(lat lon) model(error) knn(3) robust").unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "y".to_owned(),
          predictors: vec!["x".to_owned()],
          model_type: SpregressModelType::Error,
          coord_variables: Some(("lat".to_owned(), "lon".to_owned())),
          knn: Some(3),
          weights_file: None,
          id_variable: None,
          contiguity: None,
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("spregress y x, coord(lat lon) model(sarar) knn(3) robust").unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "y".to_owned(),
          predictors: vec!["x".to_owned()],
          model_type: SpregressModelType::Sarar,
          coord_variables: Some(("lat".to_owned(), "lon".to_owned())),
          knn: Some(3),
          weights_file: None,
          id_variable: None,
          contiguity: None,
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("spregress y x1 x2, weights(path/to/w.gal) id(station)").unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "y".to_owned(),
          predictors: vec!["x1".to_owned(), "x2".to_owned()],
          model_type: SpregressModelType::Lag,
          coord_variables: None,
          knn: None,
          weights_file: Some("path/to/w.gal".to_owned()),
          id_variable: Some("station".to_owned()),
          contiguity: Some(SpregressContiguity::Queen),
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command(
        "spregress y x, weights(w.shp) id(station) contiguity(rook) model(error) robust"
      )
      .unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "y".to_owned(),
          predictors: vec!["x".to_owned()],
          model_type: SpregressModelType::Error,
          coord_variables: None,
          knn: None,
          weights_file: Some("w.shp".to_owned()),
          id_variable: Some("station".to_owned()),
          contiguity: Some(SpregressContiguity::Rook),
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("SPREGRESS `outcome var` `pred var`, coord(`lat var` `lon var`)").unwrap(),
      Command::Spregress {
        command: SpregressCommand {
          outcome: "outcome var".to_owned(),
          predictors: vec!["pred var".to_owned()],
          model_type: SpregressModelType::Lag,
          coord_variables: Some(("lat var".to_owned(), "lon var".to_owned())),
          knn: Some(5),
          weights_file: None,
          id_variable: None,
          contiguity: None,
          robust: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_spregress_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "spregress",
        "spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]",
      ),
      (
        "spregress y",
        "spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]",
      ),
      (
        "spregress y x",
        "spregress requires either coord() or weights() option",
      ),
      (
        "spregress y x if y > 0, coord(lat lon)",
        "spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]",
      ),
      (
        "spregress y = x, coord(lat lon)",
        "spregress expects syntax: spregress <y> <xvars>, [coord(<lat_var> <lon_var>) [knn(<k>)] | weights(<path_to_file>) id(<id_var>) [contiguity(queen|rook)]] [model(<lag|error|sarar>) robust]",
      ),
      ("spregress:", "unsupported token in command: :"),
      (
        "spregress=",
        "spregress assignment requires a target before =",
      ),
      ("spregress==", "unsupported token in command: =="),
      (
        "spregress=foo",
        "spregress assignment requires a target before =",
      ),
      (
        "spregress = foo",
        "spregress assignment requires a target before =",
      ),
      (
        "spregress,",
        "comma must be followed by at least one option",
      ),
      (
        "spregress y x,",
        "comma must be followed by at least one option",
      ),
      (
        "spregress y x, coord(lat lon) weights(w.gal) id(station)",
        "spregress option coord and weights are mutually exclusive",
      ),
      (
        "spregress y x, coord(lat)",
        "spregress option coord expects exactly two variables representing latitude and longitude coordinates",
      ),
      (
        "spregress y x, coord(lat lon alt)",
        "spregress option coord expects exactly two variables representing latitude and longitude coordinates",
      ),
      (
        "spregress y x, coord",
        "spregress option coord expects variables",
      ),
      (
        "spregress y x, coord()",
        "option coord expects at least one value",
      ),
      (
        "spregress y x, coord(lat lon) coord(lat2 lon2)",
        "spregress option coord may only be supplied once",
      ),
      (
        "spregress y x, coord(lat lon) id(station)",
        "spregress option id can only be used with weights() option",
      ),
      (
        "spregress y x, coord(lat lon) contiguity(queen)",
        "spregress option contiguity can only be used with weights() option",
      ),
      (
        "spregress y x, coord(lat lon) knn(-1)",
        "spregress option knn must be at least 1",
      ),
      (
        "spregress y x, coord(lat lon) knn(0)",
        "spregress option knn must be at least 1",
      ),
      (
        "spregress y x, coord(lat lon) knn(3.5)",
        "spregress option knn expects an integer value",
      ),
      (
        "spregress y x, coord(lat lon) knn",
        "spregress option knn expects an integer value",
      ),
      (
        "spregress y x, coord(lat lon) knn()",
        "option knn expects at least one value",
      ),
      (
        "spregress y x, coord(lat lon) knn(3) knn(4)",
        "spregress option knn may only be supplied once",
      ),
      (
        "spregress y x, coord(lat lon) model(invalid)",
        "spregress option model must be 'lag', 'error', or 'sarar'",
      ),
      (
        "spregress y x, coord(lat lon) model(LAG)",
        "spregress option model must be 'lag', 'error', or 'sarar'",
      ),
      (
        "spregress y x, coord(lat lon) model",
        "spregress option model expects a value",
      ),
      (
        "spregress y x, coord(lat lon) model()",
        "option model expects at least one value",
      ),
      (
        "spregress y x, coord(lat lon) model(lag error)",
        "spregress option model expects one value",
      ),
      (
        "spregress y x, coord(lat lon) model(lag) model(error)",
        "spregress option model may only be supplied once",
      ),
      (
        "spregress y x, weights(w.shp)",
        "spregress option id() is required when weights() is specified",
      ),
      (
        "spregress y x, weights(w.shp) id(station) knn(3)",
        "spregress option knn/coord can only be used with coord() option",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity(invalid)",
        "spregress option contiguity must be 'queen' or 'rook'",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity(QUEEN)",
        "spregress option contiguity must be 'queen' or 'rook'",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity",
        "spregress option contiguity expects a value",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity()",
        "option contiguity expects at least one value",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity(queen rook)",
        "spregress option contiguity expects one value",
      ),
      (
        "spregress y x, weights(w.shp) id(station) contiguity(queen) contiguity(rook)",
        "spregress option contiguity may only be supplied once",
      ),
      (
        "spregress y x, weights(w.shp) id",
        "spregress option id expects a value",
      ),
      (
        "spregress y x, weights(w.shp) id()",
        "option id expects at least one value",
      ),
      (
        "spregress y x, weights(w.shp) id(station1 station2)",
        "spregress option id expects one value",
      ),
      (
        "spregress y x, weights(w.shp) id(station1) id(station2)",
        "spregress option id may only be supplied once",
      ),
      (
        "spregress y x, weights id(station)",
        "spregress option weights expects a path",
      ),
      (
        "spregress y x, weights() id(station)",
        "option weights expects at least one value",
      ),
      (
        "spregress y x, weights(w1) weights(w2) id(station)",
        "spregress option weights may only be supplied once",
      ),
      (
        "spregress y x, coord(lat lon) robust=true",
        "spregress option robust does not accept a value",
      ),
      (
        "spregress y x, coord(lat lon) robust(foo)",
        "spregress option robust does not accept a value",
      ),
      (
        "spregress y x, coord(lat lon) invalid_opt",
        "spregress unsupported option: invalid_opt",
      ),
      (
        "spregress y x, coord(lat lon) invalid_opt2 invalid_opt1",
        "spregress unsupported option: invalid_opt1, invalid_opt2",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_bayes_prefix_syntax() {
    assert_eq!(
      parse_command("bayes: regress y x").unwrap(),
      Command::BayesPrefix {
        command: BayesPrefixCommand {
          command: Box::new(Command::Regress {
            command: RegressCommand {
              outcome: "y".to_string(),
              predictors: vec!["x".to_string()],
              estimator: RegressEstimator::Ols,
              weight_variable: None,
              robust: false,
              cluster_variable: None,
              include_intercept: true,
            },
          }),
          draws: None,
          burnin: None,
          chains: None,
          thin: None,
          seed: None,
          priors: Vec::new(),
        },
      }
    );

    let cmd1 = "bayes, draws(500) burnin(200) chains(2) thin(2) seed(123): regress y x";
    assert_eq!(
      parse_command(cmd1).unwrap(),
      Command::BayesPrefix {
        command: BayesPrefixCommand {
          command: Box::new(Command::Regress {
            command: RegressCommand {
              outcome: "y".to_string(),
              predictors: vec!["x".to_string()],
              estimator: RegressEstimator::Ols,
              weight_variable: None,
              robust: false,
              cluster_variable: None,
              include_intercept: true,
            },
          }),
          draws: Some(500),
          burnin: Some(200),
          chains: Some(2),
          thin: Some(2),
          seed: Some(123),
          priors: Vec::new(),
        },
      }
    );

    let cmd2 = "bayes, prior(x, normal(0, 10)) prior(intercept, uniform(-5, 5)): logit y x";
    assert_eq!(
      parse_command(cmd2).unwrap(),
      Command::BayesPrefix {
        command: BayesPrefixCommand {
          command: Box::new(Command::Logit {
            command: LogitCommand {
              outcome: "y".to_string(),
              predictors: vec!["x".to_string()],
              robust: false,
              cluster_variable: None,
              include_intercept: true,
            },
          }),
          draws: None,
          burnin: None,
          chains: None,
          thin: None,
          seed: None,
          priors: vec![
            ("x".to_string(), "normal(0,10)".to_string()),
            ("intercept".to_string(), "uniform(-5,5)".to_string()),
          ],
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_bayes_prefix_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "bayes: codebook",
        "bayes prefix only supports regress and logit commands",
      ),
      (
        "bayes: summarize",
        "bayes prefix only supports regress and logit commands",
      ),
      (
        "bayes: probit y x",
        "bayes prefix only supports regress and logit commands",
      ),
      ("bayes:", "bayes expects a command after :"),
      ("bayes: ", "bayes expects a command after :"),
      (
        "bayes, draws(100)",
        "bayes prefix expects syntax: bayes [, options]: command",
      ),
      (
        "bayes draws(100): regress y x",
        "bayes prefix options must start with a comma",
      ),
      (
        "bayes, invalid(1): regress y x",
        "option invalid values must be identifiers",
      ),
      (
        "bayes, invalid: regress y x",
        "unsupported bayes option: invalid",
      ),
      (
        "bayes, draws(abc): regress y x",
        "option draws expects a numeric value",
      ),
      (
        "bayes, draws=abc: regress y x",
        "draws must be a numeric value",
      ),
      (
        "bayes, prior(x): regress y x",
        "prior option expects prior(variable, distribution) syntax",
      ),
      (
        "bayes, prior: regress y x",
        "prior expects (variable, distribution)",
      ),
      (
        "bayes, prior(x, normal): codebook",
        "bayes prefix only supports regress and logit commands",
      ),
      (
        "bayes,",
        "bayes prefix expects syntax: bayes [, options]: command",
      ),
      (
        "bayes, : regress y x",
        "comma must be followed by at least one option",
      ),
      ("bayes", "bayes expects syntax: bayes linear <y> <xvars>"),
      (
        "bayes linear",
        "bayes expects syntax: bayes linear <y> <xvars>",
      ),
      ("bayes = 1", "bayes assignment requires a target before ="),
      ("bayes == 1", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_regularized_regression_syntax() {
    assert_eq!(
      parse_command("lasso linear cost age bmi").unwrap(),
      Command::Lasso {
        command: LassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          alpha: "1.0".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("lasso linear cost age, alpha(0.25)").unwrap(),
      Command::Lasso {
        command: LassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "0.25".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("lasso linear cost age, noconstant").unwrap(),
      Command::Lasso {
        command: LassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("LASSO linear `cost var` `age var`, alpha(0.5) noconstant").unwrap(),
      Command::Lasso {
        command: LassoCommand {
          outcome: "cost var".to_owned(),
          predictors: vec!["age var".to_owned()],
          alpha: "0.5".to_owned(),
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("postlasso linear cost age bmi").unwrap(),
      Command::Postlasso {
        command: PostlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          alpha: "1.0".to_owned(),
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("postlasso linear cost age, alpha(0.25)").unwrap(),
      Command::Postlasso {
        command: PostlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "0.25".to_owned(),
          robust: false,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("postlasso linear cost age, robust").unwrap(),
      Command::Postlasso {
        command: PostlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          robust: true,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("postlasso linear cost age, noconstant").unwrap(),
      Command::Postlasso {
        command: PostlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          robust: false,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("postlasso linear cost age, robust alpha(0.1) noconstant").unwrap(),
      Command::Postlasso {
        command: PostlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "0.1".to_owned(),
          robust: true,
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("ridge linear cost age bmi").unwrap(),
      Command::Ridge {
        command: RidgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          alpha: "1.0".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("ridge linear cost age, alpha(0.25)").unwrap(),
      Command::Ridge {
        command: RidgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "0.25".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("ridge linear cost age, noconstant").unwrap(),
      Command::Ridge {
        command: RidgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("elasticnet linear cost age bmi").unwrap(),
      Command::Elasticnet {
        command: ElasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          alpha: "1.0".to_owned(),
          l1_ratio: "0.5".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("elasticnet linear cost age, alpha(0.25) l1_ratio(0.75)").unwrap(),
      Command::Elasticnet {
        command: ElasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "0.25".to_owned(),
          l1_ratio: "0.75".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("elasticnet linear cost age, noconstant").unwrap(),
      Command::Elasticnet {
        command: ElasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          l1_ratio: "0.5".to_owned(),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("elasticnet linear cost age, l1_ratio(0.0)").unwrap(),
      Command::Elasticnet {
        command: ElasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          l1_ratio: "0.0".to_owned(),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("elasticnet linear cost age, l1_ratio(1.0)").unwrap(),
      Command::Elasticnet {
        command: ElasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          alpha: "1.0".to_owned(),
          l1_ratio: "1.0".to_owned(),
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_regularized_regression_syntax_with_exact_diagnostics() {
    let cases = [
      ("lasso", "lasso expects syntax: lasso linear <y> <xvars>"),
      (
        "lasso linear",
        "lasso expects syntax: lasso linear <y> <xvars>",
      ),
      (
        "lasso linear y",
        "lasso expects syntax: lasso linear <y> <xvars>",
      ),
      ("lasso logistic y x", "lasso model must be linear"),
      ("lasso `linear` y x", "lasso model must be linear"),
      (
        "lasso linear y x if y > 0",
        "lasso expects syntax: lasso linear <y> <xvars>",
      ),
      (
        "lasso linear y x, alpha()",
        "option alpha expects at least one value",
      ),
      (
        "lasso linear y x, alpha(0)",
        "lasso option alpha must be positive",
      ),
      (
        "lasso linear y x, alpha(-1)",
        "lasso option alpha must be positive",
      ),
      (
        "lasso linear y x, alpha(foo)",
        "option alpha expects a numeric value",
      ),
      (
        "lasso linear y x, alpha",
        "lasso option alpha expects a numeric value",
      ),
      (
        "lasso linear y x, alpha(1) alpha(2)",
        "lasso option alpha may only be supplied once",
      ),
      (
        "lasso linear y x, robust",
        "lasso unsupported option: robust",
      ),
      (
        "lasso linear y x, noconstant(1)",
        "option noconstant values must be identifiers",
      ),
      (
        "lasso linear y x, noconstant(foo)",
        "lasso option noconstant does not accept a value",
      ),
      ("lasso:", "unsupported token in command: :"),
      ("lasso=", "lasso assignment requires a target before ="),
      ("lasso==", "unsupported token in command: =="),
      ("lasso = y x", "lasso assignment requires a target before ="),
      (
        "postlasso",
        "postlasso expects syntax: postlasso linear <y> <xvars>",
      ),
      (
        "postlasso linear",
        "postlasso expects syntax: postlasso linear <y> <xvars>",
      ),
      (
        "postlasso linear y",
        "postlasso expects syntax: postlasso linear <y> <xvars>",
      ),
      ("postlasso logistic y x", "postlasso model must be linear"),
      (
        "postlasso linear y x if y > 0",
        "postlasso expects syntax: postlasso linear <y> <xvars>",
      ),
      (
        "postlasso linear y x, alpha()",
        "option alpha expects at least one value",
      ),
      (
        "postlasso linear y x, alpha(-1)",
        "postlasso option alpha must be positive",
      ),
      (
        "postlasso linear y x, robust(1)",
        "option robust values must be identifiers",
      ),
      (
        "postlasso linear y x, robust(foo)",
        "postlasso option robust does not accept a value",
      ),
      (
        "postlasso linear y x, cv(5)",
        "postlasso unsupported option: cv",
      ),
      ("postlasso:", "unsupported token in command: :"),
      (
        "postlasso=",
        "postlasso assignment requires a target before =",
      ),
      ("postlasso==", "unsupported token in command: =="),
      ("ridge", "ridge expects syntax: ridge linear <y> <xvars>"),
      (
        "ridge linear",
        "ridge expects syntax: ridge linear <y> <xvars>",
      ),
      (
        "ridge linear y",
        "ridge expects syntax: ridge linear <y> <xvars>",
      ),
      ("ridge logistic y x", "ridge model must be linear"),
      (
        "ridge linear y x if y > 0",
        "ridge expects syntax: ridge linear <y> <xvars>",
      ),
      (
        "ridge linear y x, alpha()",
        "option alpha expects at least one value",
      ),
      (
        "ridge linear y x, alpha(-1)",
        "ridge option alpha must be positive",
      ),
      (
        "ridge linear y x, robust",
        "ridge unsupported option: robust",
      ),
      ("ridge:", "unsupported token in command: :"),
      ("ridge=", "ridge assignment requires a target before ="),
      ("ridge==", "unsupported token in command: =="),
      (
        "elasticnet",
        "elasticnet expects syntax: elasticnet linear <y> <xvars>",
      ),
      (
        "elasticnet linear",
        "elasticnet expects syntax: elasticnet linear <y> <xvars>",
      ),
      (
        "elasticnet linear y",
        "elasticnet expects syntax: elasticnet linear <y> <xvars>",
      ),
      ("elasticnet logistic y x", "elasticnet model must be linear"),
      (
        "elasticnet linear y x if y > 0",
        "elasticnet expects syntax: elasticnet linear <y> <xvars>",
      ),
      (
        "elasticnet linear y x, alpha()",
        "option alpha expects at least one value",
      ),
      (
        "elasticnet linear y x, alpha(-1)",
        "elasticnet option alpha must be positive",
      ),
      (
        "elasticnet linear y x, l1_ratio()",
        "option l1_ratio expects at least one value",
      ),
      (
        "elasticnet linear y x, l1_ratio(-0.1)",
        "elasticnet option l1_ratio must be between 0 and 1 inclusive",
      ),
      (
        "elasticnet linear y x, l1_ratio(1.1)",
        "elasticnet option l1_ratio must be between 0 and 1 inclusive",
      ),
      (
        "elasticnet linear y x, l1_ratio(foo)",
        "option l1_ratio values must be numeric",
      ),
      (
        "elasticnet linear y x, l1_ratio",
        "elasticnet option l1_ratio expects a numeric value",
      ),
      (
        "elasticnet linear y x, l1_ratio(0.2) l1_ratio(0.4)",
        "elasticnet option l1_ratio may only be supplied once",
      ),
      (
        "elasticnet linear y x, l1_ratio(0.2 0.5)",
        "elasticnet option l1_ratio expects one value",
      ),
      (
        "elasticnet linear y x, robust",
        "elasticnet unsupported option: robust",
      ),
      ("elasticnet:", "unsupported token in command: :"),
      (
        "elasticnet=",
        "elasticnet assignment requires a target before =",
      ),
      ("elasticnet==", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_cv_regularized_regression_syntax() {
    assert_eq!(
      parse_command("cvlasso linear cost age bmi").unwrap(),
      Command::Cvlasso {
        command: CvlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          cv: 5,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvlasso linear cost age, cv(10)").unwrap(),
      Command::Cvlasso {
        command: CvlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 10,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvlasso linear cost age, cv(2.0)").unwrap(),
      Command::Cvlasso {
        command: CvlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 2,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvlasso linear cost age, noconstant").unwrap(),
      Command::Cvlasso {
        command: CvlassoCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 5,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("CVLASSO linear `cost var` `age var`, cv(3) noconstant").unwrap(),
      Command::Cvlasso {
        command: CvlassoCommand {
          outcome: "cost var".to_owned(),
          predictors: vec!["age var".to_owned()],
          cv: 3,
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("cvridge linear cost age bmi").unwrap(),
      Command::Cvridge {
        command: CvridgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          cv: 5,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvridge linear cost age, cv(4)").unwrap(),
      Command::Cvridge {
        command: CvridgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 4,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvridge linear cost age, noconstant").unwrap(),
      Command::Cvridge {
        command: CvridgeCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 5,
          include_intercept: false,
        },
      }
    );

    assert_eq!(
      parse_command("cvelasticnet linear cost age bmi").unwrap(),
      Command::Cvelasticnet {
        command: CvelasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned(), "bmi".to_owned()],
          cv: 5,
          l1_ratio: CvelasticnetL1Ratio::Multiple(vec![
            "0.1".to_owned(),
            "0.5".to_owned(),
            "0.7".to_owned(),
            "0.9".to_owned(),
            "0.95".to_owned(),
            "0.99".to_owned(),
            "1.0".to_owned(),
          ]),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvelasticnet linear cost age, cv(8) l1_ratio(0.5)").unwrap(),
      Command::Cvelasticnet {
        command: CvelasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 8,
          l1_ratio: CvelasticnetL1Ratio::Single("0.5".to_owned()),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvelasticnet linear cost age, l1_ratio(0.2 0.4 0.6 0.8)").unwrap(),
      Command::Cvelasticnet {
        command: CvelasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 5,
          l1_ratio: CvelasticnetL1Ratio::Multiple(vec![
            "0.2".to_owned(),
            "0.4".to_owned(),
            "0.6".to_owned(),
            "0.8".to_owned(),
          ]),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cvelasticnet linear cost age, l1_ratio(0.0) noconstant").unwrap(),
      Command::Cvelasticnet {
        command: CvelasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 5,
          l1_ratio: CvelasticnetL1Ratio::Single("0.0".to_owned()),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("cvelasticnet linear cost age, l1_ratio(1.0)").unwrap(),
      Command::Cvelasticnet {
        command: CvelasticnetCommand {
          outcome: "cost".to_owned(),
          predictors: vec!["age".to_owned()],
          cv: 5,
          l1_ratio: CvelasticnetL1Ratio::Single("1.0".to_owned()),
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_cv_regularized_regression_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "cvlasso",
        "cvlasso expects syntax: cvlasso linear <y> <xvars>",
      ),
      (
        "cvlasso linear",
        "cvlasso expects syntax: cvlasso linear <y> <xvars>",
      ),
      (
        "cvlasso linear y",
        "cvlasso expects syntax: cvlasso linear <y> <xvars>",
      ),
      ("cvlasso logistic y x", "cvlasso model must be linear"),
      ("cvlasso `linear` y x", "cvlasso model must be linear"),
      (
        "cvlasso linear y x if y > 0",
        "cvlasso expects syntax: cvlasso linear <y> <xvars>",
      ),
      (
        "cvlasso linear y x, cv()",
        "option cv expects at least one value",
      ),
      (
        "cvlasso linear y x, cv(1)",
        "cvlasso option cv must be at least 2",
      ),
      (
        "cvlasso linear y x, cv(0)",
        "cvlasso option cv must be at least 2",
      ),
      (
        "cvlasso linear y x, cv(-1)",
        "cvlasso option cv must be at least 2",
      ),
      (
        "cvlasso linear y x, cv(2.5)",
        "cvlasso option cv expects an integer value",
      ),
      (
        "cvlasso linear y x, cv(foo)",
        "option cv expects a numeric value",
      ),
      (
        "cvlasso linear y x, cv",
        "cvlasso option cv expects an integer value",
      ),
      (
        "cvlasso linear y x, cv(3) cv(5)",
        "cvlasso option cv may only be supplied once",
      ),
      (
        "cvlasso linear y x, noconstant(1)",
        "option noconstant values must be identifiers",
      ),
      (
        "cvlasso linear y x, noconstant(foo)",
        "cvlasso option noconstant does not accept a value",
      ),
      (
        "cvlasso linear y x, alpha(1)",
        "cvlasso unsupported option: alpha",
      ),
      ("cvlasso:", "unsupported token in command: :"),
      ("cvlasso=", "cvlasso assignment requires a target before ="),
      ("cvlasso==", "unsupported token in command: =="),
      (
        "cvlasso = y x",
        "cvlasso assignment requires a target before =",
      ),
      (
        "cvridge",
        "cvridge expects syntax: cvridge linear <y> <xvars>",
      ),
      (
        "cvridge linear",
        "cvridge expects syntax: cvridge linear <y> <xvars>",
      ),
      (
        "cvridge linear y",
        "cvridge expects syntax: cvridge linear <y> <xvars>",
      ),
      ("cvridge logistic y x", "cvridge model must be linear"),
      (
        "cvridge linear y x if y > 0",
        "cvridge expects syntax: cvridge linear <y> <xvars>",
      ),
      (
        "cvridge linear y x, cv(1)",
        "cvridge option cv must be at least 2",
      ),
      (
        "cvridge linear y x, cv(2.5)",
        "cvridge option cv expects an integer value",
      ),
      (
        "cvridge linear y x, cv",
        "cvridge option cv expects an integer value",
      ),
      (
        "cvridge linear y x, cv(3) cv(5)",
        "cvridge option cv may only be supplied once",
      ),
      (
        "cvridge linear y x, alpha(1)",
        "cvridge unsupported option: alpha",
      ),
      ("cvridge:", "unsupported token in command: :"),
      ("cvridge=", "cvridge assignment requires a target before ="),
      ("cvridge==", "unsupported token in command: =="),
      (
        "cvelasticnet",
        "cvelasticnet expects syntax: cvelasticnet linear <y> <xvars>",
      ),
      (
        "cvelasticnet linear",
        "cvelasticnet expects syntax: cvelasticnet linear <y> <xvars>",
      ),
      (
        "cvelasticnet linear y",
        "cvelasticnet expects syntax: cvelasticnet linear <y> <xvars>",
      ),
      (
        "cvelasticnet logistic y x",
        "cvelasticnet model must be linear",
      ),
      (
        "cvelasticnet linear y x if y > 0",
        "cvelasticnet expects syntax: cvelasticnet linear <y> <xvars>",
      ),
      (
        "cvelasticnet linear y x, cv(1)",
        "cvelasticnet option cv must be at least 2",
      ),
      (
        "cvelasticnet linear y x, cv(2.5)",
        "cvelasticnet option cv expects an integer value",
      ),
      (
        "cvelasticnet linear y x, cv",
        "cvelasticnet option cv expects an integer value",
      ),
      (
        "cvelasticnet linear y x, cv(3) cv(5)",
        "cvelasticnet option cv may only be supplied once",
      ),
      (
        "cvelasticnet linear y x, l1_ratio()",
        "option l1_ratio expects at least one value",
      ),
      (
        "cvelasticnet linear y x, l1_ratio(-0.1)",
        "cvelasticnet option l1_ratio values must be between 0 and 1 inclusive",
      ),
      (
        "cvelasticnet linear y x, l1_ratio(1.1)",
        "cvelasticnet option l1_ratio values must be between 0 and 1 inclusive",
      ),
      (
        "cvelasticnet linear y x, l1_ratio(0.2 1.5)",
        "cvelasticnet option l1_ratio values must be between 0 and 1 inclusive",
      ),
      (
        "cvelasticnet linear y x, l1_ratio(foo)",
        "option l1_ratio values must be numeric",
      ),
      (
        "cvelasticnet linear y x, l1_ratio",
        "cvelasticnet option l1_ratio expects a numeric value or list of numeric values",
      ),
      (
        "cvelasticnet linear y x, l1_ratio(0.2) l1_ratio(0.4)",
        "cvelasticnet option l1_ratio may only be supplied once",
      ),
      (
        "cvelasticnet linear y x, alpha(1)",
        "cvelasticnet unsupported option: alpha",
      ),
      ("cvelasticnet:", "unsupported token in command: :"),
      (
        "cvelasticnet=",
        "cvelasticnet assignment requires a target before =",
      ),
      ("cvelasticnet==", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_bayes_linear_command_syntax() {
    assert_eq!(
      parse_command("bayes linear cost age bmi").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "cost".to_string(),
          predictors: vec!["age".to_string(), "bmi".to_string()],
          n_iter: 300,
          tol: "0.001".to_string(),
          include_intercept: true,
        },
      }
    );

    assert_eq!(
      parse_command("bayes linear cost age, n_iter(500)").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "cost".to_string(),
          predictors: vec!["age".to_string()],
          n_iter: 500,
          tol: "0.001".to_string(),
          include_intercept: true,
        },
      }
    );

    assert_eq!(
      parse_command("bayes linear cost age, tol(1e-4)").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "cost".to_string(),
          predictors: vec!["age".to_string()],
          n_iter: 300,
          tol: "1e-4".to_string(),
          include_intercept: true,
        },
      }
    );

    assert_eq!(
      parse_command("bayes linear cost age, n_iter(100) tol(1e-5) noconstant").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "cost".to_string(),
          predictors: vec!["age".to_string()],
          n_iter: 100,
          tol: "1e-5".to_string(),
          include_intercept: false,
        },
      }
    );

    // Case insensitivity
    assert_eq!(
      parse_command("BAYES LINEAR cost age").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "cost".to_string(),
          predictors: vec!["age".to_string()],
          n_iter: 300,
          tol: "0.001".to_string(),
          include_intercept: true,
        },
      }
    );

    // Backtick quoting
    assert_eq!(
      parse_command("bayes linear `total cost` `patient age`").unwrap(),
      Command::Bayes {
        command: BayesCommand {
          outcome: "total cost".to_string(),
          predictors: vec!["patient age".to_string()],
          n_iter: 300,
          tol: "0.001".to_string(),
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_bayes_linear_syntax() {
    let cases = [
      ("bayes", "bayes expects syntax: bayes linear <y> <xvars>"),
      (
        "bayes linear",
        "bayes expects syntax: bayes linear <y> <xvars>",
      ),
      (
        "bayes linear y",
        "bayes expects syntax: bayes linear <y> <xvars>",
      ),
      ("bayes logistic y x", "bayes model must be linear"),
      (
        "bayes linear y x if y > 0",
        "bayes expects syntax: bayes linear <y> <xvars>",
      ),
      (
        "bayes linear y x, n_iter()",
        "option n_iter expects at least one value",
      ),
      (
        "bayes linear y x, n_iter(-10)",
        "bayes option n_iter must be at least 1",
      ),
      (
        "bayes linear y x, n_iter(0)",
        "bayes option n_iter must be at least 1",
      ),
      (
        "bayes linear y x, n_iter(1.5)",
        "bayes option n_iter expects an integer value",
      ),
      (
        "bayes linear y x, n_iter(foo)",
        "option n_iter expects a numeric value",
      ),
      (
        "bayes linear y x, n_iter",
        "bayes option n_iter expects an integer value",
      ),
      (
        "bayes linear y x, n_iter(100) n_iter(200)",
        "bayes option n_iter may only be supplied once",
      ),
      (
        "bayes linear y x, tol()",
        "option tol expects at least one value",
      ),
      (
        "bayes linear y x, tol(-0.5)",
        "bayes option tol must be positive",
      ),
      (
        "bayes linear y x, tol(0)",
        "bayes option tol must be positive",
      ),
      (
        "bayes linear y x, tol(foo)",
        "option tol expects a numeric value",
      ),
      (
        "bayes linear y x, tol",
        "bayes option tol expects a numeric value",
      ),
      (
        "bayes linear y x, tol(0.01) tol(0.02)",
        "bayes option tol may only be supplied once",
      ),
      (
        "bayes linear y x, noconstant(1)",
        "option noconstant values must be identifiers",
      ),
      (
        "bayes linear y x, noconstant=true",
        "bayes option noconstant does not accept a value",
      ),
      (
        "bayes linear y x, alpha(1)",
        "bayes unsupported option: alpha",
      ),
      (
        "bayes linear y x, robust",
        "bayes unsupported option: robust",
      ),
      ("bayes=", "bayes assignment requires a target before ="),
      ("bayes = 1", "bayes assignment requires a target before ="),
      ("bayes==", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_predict_command_syntax() {
    assert_eq!(
      parse_command("predict cost_hat").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "cost_hat".to_string(),
          kind: PredictKind::Xb,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict resid, residuals").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "resid".to_string(),
          kind: PredictKind::Residuals,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict p_hat, pr").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "p_hat".to_string(),
          kind: PredictKind::Pr,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict spatial_hat, spatial_lag").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "spatial_hat".to_string(),
          kind: PredictKind::SpatialLag,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive interval").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: true,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive interval level(90)").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: true,
          level: "90".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive interval level(95.5)").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: true,
          level: "95.5".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive std").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: false,
          level: "95.0".to_string(),
          std: true,
          saving: None,
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive saving(draws.parquet)").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: Some("draws.parquet".to_string()),
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive saving(subdir/draws.parquet)").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: Some("subdir/draws.parquet".to_string()),
        },
      }
    );

    assert_eq!(
      parse_command("predict y_pp, posterior_predictive std interval level(90)").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "y_pp".to_string(),
          kind: PredictKind::PosteriorPredictive,
          interval: true,
          level: "90".to_string(),
          std: true,
          saving: None,
        },
      }
    );

    // Case insensitivity
    assert_eq!(
      parse_command("PREDICT cost_hat").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "cost_hat".to_string(),
          kind: PredictKind::Xb,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    // Backtick quoting
    assert_eq!(
      parse_command("predict `cost hat`, residuals").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "cost hat".to_string(),
          kind: PredictKind::Residuals,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    // Double quotes
    assert_eq!(
      parse_command("predict \"cost hat\", pr").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "cost hat".to_string(),
          kind: PredictKind::Pr,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );

    // Repeated allowed flags
    assert_eq!(
      parse_command("predict cost_hat, xb xb").unwrap(),
      Command::Predict {
        command: PredictCommand {
          target_variable: "cost_hat".to_string(),
          kind: PredictKind::Xb,
          interval: false,
          level: "95.0".to_string(),
          std: false,
          saving: None,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_predict_syntax() {
    let cases = [
      ("predict", "predict expects syntax: predict <newvar>"),
      ("predict a b", "predict expects syntax: predict <newvar>"),
      (
        "predict cost_hat if age > 18",
        "predict expects syntax: predict <newvar>",
      ),
      (
        "predict cost_hat if age > 18, residuals",
        "predict expects syntax: predict <newvar>",
      ),
      (
        "predict cost_hat = 1",
        "predict expects syntax: predict <newvar>",
      ),
      ("predict,", "predict expects syntax: predict <newvar>"),
      (
        "predict, residuals",
        "predict expects syntax: predict <newvar>",
      ),
      (
        "predict cost_hat,",
        "comma must be followed by at least one option",
      ),
      ("predict=", "predict assignment requires a target before ="),
      (
        "predict = 1",
        "predict assignment requires a target before =",
      ),
      ("predict==", "unsupported token in command: =="),
      ("predict == 1", "unsupported token in command: =="),
      ("predict:", "unsupported token in command: :"),
      ("predict: cost_hat", "unsupported token in command: :"),
      ("predict cost_hat, foo", "predict unsupported option: foo"),
      (
        "predict cost_hat, foo bar",
        "predict unsupported option: bar, foo",
      ),
      (
        "predict cost_hat, RESIDUALS",
        "predict unsupported option: RESIDUALS",
      ),
      (
        "predict cost_hat, xb=true",
        "predict option xb does not accept a value",
      ),
      (
        "predict cost_hat, residuals=true",
        "predict option residuals does not accept a value",
      ),
      (
        "predict cost_hat, pr=true",
        "predict option pr does not accept a value",
      ),
      (
        "predict cost_hat, spatial_lag=true",
        "predict option spatial_lag does not accept a value",
      ),
      (
        "predict cost_hat, posterior_predictive=true",
        "predict option posterior_predictive does not accept a value",
      ),
      (
        "predict cost_hat, interval=true",
        "predict option interval does not accept a value",
      ),
      (
        "predict cost_hat, std=true",
        "predict option std does not accept a value",
      ),
      (
        "predict cost_hat, xb residuals",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, xb spatial_lag",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, xb posterior_predictive",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, pr residuals",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, pr spatial_lag",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, pr posterior_predictive",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, residuals spatial_lag",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, residuals posterior_predictive",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, spatial_lag posterior_predictive",
        "predict options xb, residuals, pr, spatial_lag, and posterior_predictive cannot be combined",
      ),
      (
        "predict cost_hat, xb interval",
        "predict interval options require posterior_predictive",
      ),
      (
        "predict cost_hat, xb level(90)",
        "predict interval options require posterior_predictive",
      ),
      (
        "predict cost_hat, residuals interval",
        "predict interval options require posterior_predictive",
      ),
      (
        "predict cost_hat, posterior_predictive level(90)",
        "predict option level requires interval",
      ),
      (
        "predict cost_hat, posterior_predictive interval level(0)",
        "predict option level must be between 0 and 100",
      ),
      (
        "predict cost_hat, posterior_predictive interval level(100)",
        "predict option level must be between 0 and 100",
      ),
      (
        "predict cost_hat, posterior_predictive interval level(-5)",
        "predict option level must be between 0 and 100",
      ),
      (
        "predict cost_hat, posterior_predictive interval level",
        "predict option level expects a numeric value",
      ),
      (
        "predict cost_hat, posterior_predictive interval level(abc)",
        "option level expects a numeric value",
      ),
      (
        "predict cost_hat, posterior_predictive interval level()",
        "option level expects at least one value",
      ),
      (
        "predict cost_hat, posterior_predictive interval level(90) level(95)",
        "predict option level may only be supplied once",
      ),
      (
        "predict cost_hat, xb std",
        "predict std and saving options require posterior_predictive",
      ),
      (
        "predict cost_hat, xb saving(draws.parquet)",
        "predict std and saving options require posterior_predictive",
      ),
      (
        "predict cost_hat, residuals saving(draws.parquet)",
        "predict std and saving options require posterior_predictive",
      ),
      (
        "predict cost_hat, posterior_predictive saving",
        "predict option saving expects a path",
      ),
      (
        "predict cost_hat, posterior_predictive saving()",
        "option saving expects at least one value",
      ),
      (
        "predict cost_hat, posterior_predictive saving(a) saving(b)",
        "predict option saving may only be supplied once",
      ),
      (
        "predict cost_hat, posterior_predictive std saving(draws.parquet)",
        "predict saving option cannot be combined with std or interval options",
      ),
      (
        "predict cost_hat, posterior_predictive interval saving(draws.parquet)",
        "predict saving option cannot be combined with std or interval options",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_xtlogit_syntax() {
    assert_eq!(
      parse_command("xtlogit y x, fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x".to_string()],
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit y x1 x2, fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x1".to_string(), "x2".to_string()],
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit y x, fe robust").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x".to_string()],
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit y x, robust fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x".to_string()],
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit y x, fe fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x".to_string()],
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("XTLOGIT y x, fe robust").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y".to_string(),
          predictors: vec!["x".to_string()],
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit `y var` `x var`, fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y var".to_string(),
          predictors: vec!["x var".to_string()],
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("xtlogit \"y var\" \"x var\", fe").unwrap(),
      Command::XtLogit {
        command: XtLogitCommand {
          outcome: "y var".to_string(),
          predictors: vec!["x var".to_string()],
          robust: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_xtlogit_syntax() {
    let cases = [
      (
        "xtlogit",
        "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
      ),
      (
        "xtlogit, fe",
        "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
      ),
      (
        "xtlogit y, fe",
        "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
      ),
      ("xtlogit y x", "xtlogit requires option fe"),
      ("xtlogit y x, robust", "xtlogit requires option fe"),
      (
        "xtlogit y x,",
        "comma must be followed by at least one option",
      ),
      (
        "xtlogit y x if y > 0, fe",
        "xtlogit expects syntax: xtlogit <y> <xvars>, fe [robust]",
      ),
      ("xtlogit y x, fe extra", "xtlogit unsupported option: extra"),
      (
        "xtlogit y x, fe foo bar",
        "xtlogit unsupported option: bar, foo",
      ),
      (
        "xtlogit y x, FE ROBUST",
        "xtlogit unsupported option: FE, ROBUST",
      ),
      ("xtlogit y x, fe(1)", "option fe values must be identifiers"),
      (
        "xtlogit y x, fe(a)",
        "xtlogit option fe does not accept a value",
      ),
      (
        "xtlogit y x, fe=1",
        "xtlogit option fe does not accept a value",
      ),
      (
        "xtlogit y x, fe=a",
        "xtlogit option fe does not accept a value",
      ),
      (
        "xtlogit y x, fe robust(1)",
        "option robust values must be identifiers",
      ),
      (
        "xtlogit y x, fe robust(a)",
        "xtlogit option robust does not accept a value",
      ),
      (
        "xtlogit y x, fe robust=1",
        "xtlogit option robust does not accept a value",
      ),
      ("xtlogit=", "xtlogit assignment requires a target before ="),
      (
        "xtlogit = 1",
        "xtlogit assignment requires a target before =",
      ),
      ("xtlogit==", "unsupported token in command: =="),
      ("xtlogit == 1", "unsupported token in command: =="),
      ("xtlogit:", "unsupported token in command: :"),
      ("xtlogit: regress y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_lowess_syntax() {
    assert_eq!(
      parse_command("lowess y x, gen(y_hat)").unwrap(),
      Command::Lowess {
        command: LowessCommand {
          outcome: "y".to_string(),
          predictor: "x".to_string(),
          target_variable: "y_hat".to_string(),
          bandwidth: (2.0f64 / 3.0f64).to_string(),
        },
      }
    );
    assert_eq!(
      parse_command("lowess y x, gen(y_hat) bandwidth=0.5").unwrap(),
      Command::Lowess {
        command: LowessCommand {
          outcome: "y".to_string(),
          predictor: "x".to_string(),
          target_variable: "y_hat".to_string(),
          bandwidth: "0.5".to_string(),
        },
      }
    );
    assert_eq!(
      parse_command("LOWESS y x, gen(y_hat) bandwidth=0.8").unwrap(),
      Command::Lowess {
        command: LowessCommand {
          outcome: "y".to_string(),
          predictor: "x".to_string(),
          target_variable: "y_hat".to_string(),
          bandwidth: "0.8".to_string(),
        },
      }
    );
    assert_eq!(
      parse_command("lowess `y var` `x var`, gen(y_hat)").unwrap(),
      Command::Lowess {
        command: LowessCommand {
          outcome: "y var".to_string(),
          predictor: "x var".to_string(),
          target_variable: "y_hat".to_string(),
          bandwidth: (2.0f64 / 3.0f64).to_string(),
        },
      }
    );
    assert_eq!(
      parse_command("lowess \"y var\" \"x var\", gen(y_hat)").unwrap(),
      Command::Lowess {
        command: LowessCommand {
          outcome: "y var".to_string(),
          predictor: "x var".to_string(),
          target_variable: "y_hat".to_string(),
          bandwidth: (2.0f64 / 3.0f64).to_string(),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_lowess_syntax() {
    let cases = [
      (
        "lowess",
        "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
      ),
      (
        "lowess y",
        "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
      ),
      (
        "lowess y x z, gen(y_hat)",
        "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
      ),
      (
        "lowess y x if y > 0, gen(y_hat)",
        "lowess expects syntax: lowess <y> <x>, gen(<newvar>) [bandwidth=<0,1>]",
      ),
      ("lowess y x", "lowess option gen expects one variable"),
      (
        "lowess y x,",
        "comma must be followed by at least one option",
      ),
      ("lowess y x, gen", "lowess option gen expects variables"),
      ("lowess y x, gen()", "option gen expects at least one value"),
      (
        "lowess y x, gen(y1 y2)",
        "lowess option gen expects one variable",
      ),
      (
        "lowess y x, gen(y1) gen(y2)",
        "lowess option gen may only be supplied once",
      ),
      (
        "lowess y x, gen(y_hat) extra",
        "lowess unsupported option: extra",
      ),
      (
        "lowess y x, GEN(y_hat) BANDWIDTH=0.8",
        "lowess unsupported option: BANDWIDTH, GEN",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth",
        "lowess option bandwidth expects a numeric value",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth=0",
        "lowess option bandwidth must be between 0 and 1",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth=1",
        "lowess option bandwidth must be between 0 and 1",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth=1.5",
        "lowess option bandwidth must be between 0 and 1",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth=abc",
        "lowess option bandwidth expects a numeric value",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth(0.5)",
        "option bandwidth values must be identifiers",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth(abc)",
        "lowess option bandwidth expects a numeric value",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth(a b)",
        "lowess option bandwidth expects one value",
      ),
      (
        "lowess y x, gen(y_hat) bandwidth=0.5 bandwidth=0.6",
        "lowess option bandwidth may only be supplied once",
      ),
      ("lowess=", "lowess assignment requires a target before ="),
      ("lowess = 1", "lowess assignment requires a target before ="),
      ("lowess==", "unsupported token in command: =="),
      ("lowess == 1", "unsupported token in command: =="),
      ("lowess:", "unsupported token in command: :"),
      ("lowess: regress y x", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_did_syntax() {
    assert_eq!(
      parse_command("did y, treat(d) post(t)").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y".to_string(),
          controls: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("did y x1 x2, treat(d) post(t) robust").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string(), "x2".to_string()],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          robust: true,
        },
      }
    );
    assert_eq!(
      parse_command("did y, post(t) treat(d)").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y".to_string(),
          controls: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("DID y, treat(d) post(t)").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y".to_string(),
          controls: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("did `y var` `x var`, treat(`d var`) post(`t var`)").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y var".to_string(),
          controls: vec!["x var".to_string()],
          treatment_variable: "d var".to_string(),
          post_variable: "t var".to_string(),
          robust: false,
        },
      }
    );
    assert_eq!(
      parse_command("did \"y var\" \"x var\", treat(d) post(t)").unwrap(),
      Command::Did {
        command: DidCommand {
          outcome: "y var".to_string(),
          controls: vec!["x var".to_string()],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          robust: false,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_did_syntax() {
    let cases = [
      (
        "did",
        "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
      ),
      (
        "did if x > 0, treat(d) post(t)",
        "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
      ),
      (
        "did = 1, treat(d) post(t)",
        "did assignment requires a target before =",
      ),
      (
        "did y = 1, treat(d) post(t)",
        "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
      ),
      (
        "did y if x > 0, treat(d) post(t)",
        "did expects syntax: did <y> [controls], treat(<var>) post(<var>)",
      ),
      ("did y", "did option treat expects one variable"),
      ("did y,", "comma must be followed by at least one option"),
      ("did y, treat", "did option treat expects variables"),
      ("did y, treat()", "option treat expects at least one value"),
      (
        "did y, treat(d1 d2) post(t)",
        "did option treat expects one variable",
      ),
      (
        "did y, treat(d) treat(d2) post(t)",
        "did option treat may only be supplied once",
      ),
      ("did y, treat(d)", "did option post expects one variable"),
      ("did y, treat(d) post", "did option post expects variables"),
      (
        "did y, treat(d) post()",
        "option post expects at least one value",
      ),
      (
        "did y, treat(d) post(t1 t2)",
        "did option post expects one variable",
      ),
      (
        "did y, treat(d) post(t) post(t2)",
        "did option post may only be supplied once",
      ),
      (
        "did y, treat(d) post(t) extra",
        "did unsupported option: extra",
      ),
      (
        "did y, TREAT(d) POST(t)",
        "did unsupported option: POST, TREAT",
      ),
      (
        "did y, treat(d) post(t) robust=1",
        "did option robust does not accept a value",
      ),
      (
        "did y, treat(d) post(t) robust(1)",
        "option robust values must be identifiers",
      ),
      (
        "did y, treat(d) post(t) robust(foo)",
        "did option robust does not accept a value",
      ),
      (
        "did y, treat(d) post(d)",
        "did treatment and post variables must be distinct",
      ),
      (
        "did y, treat(y) post(t)",
        "did treatment and post variables must differ from outcome",
      ),
      (
        "did y, treat(d) post(y)",
        "did treatment and post variables must differ from outcome",
      ),
      (
        "did y d, treat(d) post(t)",
        "did treatment and post variables must not appear in controls",
      ),
      (
        "did y t, treat(d) post(t)",
        "did treatment and post variables must not appear in controls",
      ),
      ("did=", "did assignment requires a target before ="),
      ("did = 1", "did assignment requires a target before ="),
      ("did==", "unsupported token in command: =="),
      ("did == 1", "unsupported token in command: =="),
      ("did:", "unsupported token in command: :"),
      (
        "did: y, treat(d) post(t)",
        "unsupported token in command: :",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_drdid_syntax() {
    assert_eq!(
      parse_command("drdid y, treat(d) post(t)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y x1 x2, treat(d) post(t)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec!["x1".to_string(), "x2".to_string()],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) method(or)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Or,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) method(ipw)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Ipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) method(aipw)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) method=or").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Or,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) robust").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: true,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) bootstrap(100)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: Some(100),
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid y, treat(d) post(t) bootstrap(100) seed(42)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y".to_string(),
          covariates: vec![],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: Some(100),
          seed: Some(42),
        },
      }
    );
    assert_eq!(
      parse_command("drdid `y var` `x var`, treat(`d var`) post(`t var`)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y var".to_string(),
          covariates: vec!["x var".to_string()],
          treatment_variable: "d var".to_string(),
          post_variable: "t var".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
    assert_eq!(
      parse_command("drdid \"y var\" \"x var\", treat(d) post(t)").unwrap(),
      Command::DrDid {
        command: DrDidCommand {
          outcome: "y var".to_string(),
          covariates: vec!["x var".to_string()],
          treatment_variable: "d".to_string(),
          post_variable: "t".to_string(),
          method: DrDidMethod::Aipw,
          robust: false,
          bootstrap: None,
          seed: None,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_drdid_syntax() {
    let cases = [
      (
        "drdid",
        "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
      ),
      (
        "drdid if x > 0, treat(d) post(t)",
        "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
      ),
      (
        "drdid = 1, treat(d) post(t)",
        "drdid assignment requires a target before =",
      ),
      (
        "drdid y = 1, treat(d) post(t)",
        "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
      ),
      (
        "drdid y if x > 0, treat(d) post(t)",
        "drdid expects syntax: drdid <y> [covariates], treat(<var>) post(<var>) [method(or|ipw|aipw) robust bootstrap(<n>) seed(<n>)]",
      ),
      ("drdid y", "drdid option treat expects one variable"),
      ("drdid y,", "comma must be followed by at least one option"),
      ("drdid y, treat", "drdid option treat expects variables"),
      (
        "drdid y, treat()",
        "option treat expects at least one value",
      ),
      (
        "drdid y, treat(d1 d2) post(t)",
        "drdid option treat expects one variable",
      ),
      (
        "drdid y, treat(d) treat(d2) post(t)",
        "drdid option treat may only be supplied once",
      ),
      (
        "drdid y, treat(d)",
        "drdid option post expects one variable",
      ),
      (
        "drdid y, treat(d) post",
        "drdid option post expects variables",
      ),
      (
        "drdid y, treat(d) post()",
        "option post expects at least one value",
      ),
      (
        "drdid y, treat(d) post(t1 t2)",
        "drdid option post expects one variable",
      ),
      (
        "drdid y, treat(d) post(t) post(t2)",
        "drdid option post may only be supplied once",
      ),
      (
        "drdid y, treat(d) post(t) method(bad)",
        "drdid option method must be one of: or, ipw, aipw",
      ),
      (
        "drdid y, treat(d) post(t) method",
        "drdid option method expects a value",
      ),
      (
        "drdid y, treat(d) post(t) method()",
        "option method expects at least one value",
      ),
      (
        "drdid y, treat(d) post(t) method(or ipw)",
        "drdid option method expects one value",
      ),
      (
        "drdid y, treat(d) post(t) method(or) method(ipw)",
        "drdid option method may only be supplied once",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(0)",
        "drdid option bootstrap must be at least 1",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(-1)",
        "drdid option bootstrap must be at least 1",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(abc)",
        "option bootstrap expects a numeric value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap",
        "drdid option bootstrap expects an integer value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap()",
        "option bootstrap expects at least one value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) bootstrap(20)",
        "drdid option bootstrap may only be supplied once",
      ),
      (
        "drdid y, treat(d) post(t) seed(42)",
        "drdid option seed requires option bootstrap",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) seed(-1)",
        "drdid option seed must be at least 0",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) seed(abc)",
        "option seed expects a numeric value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) seed",
        "drdid option seed expects an integer value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) seed()",
        "option seed expects at least one value",
      ),
      (
        "drdid y, treat(d) post(t) bootstrap(10) seed(10) seed(20)",
        "drdid option seed may only be supplied once",
      ),
      (
        "drdid y, treat(d) post(t) robust=1",
        "drdid option robust does not accept a value",
      ),
      (
        "drdid y, treat(d) post(t) robust(1)",
        "option robust values must be identifiers",
      ),
      (
        "drdid y, treat(d) post(t) robust(foo)",
        "drdid option robust does not accept a value",
      ),
      (
        "drdid y, treat(d) post(t) extra",
        "drdid unsupported option: extra",
      ),
      (
        "drdid y, TREAT(d) POST(t)",
        "drdid unsupported option: POST, TREAT",
      ),
      (
        "drdid y, treat(d) post(d)",
        "drdid treatment and post variables must be distinct",
      ),
      (
        "drdid y, treat(y) post(t)",
        "drdid treatment and post variables must differ from outcome",
      ),
      (
        "drdid y, treat(d) post(y)",
        "drdid treatment and post variables must differ from outcome",
      ),
      (
        "drdid y d, treat(d) post(t)",
        "drdid treatment and post variables must not appear in covariates",
      ),
      (
        "drdid y t, treat(d) post(t)",
        "drdid treatment and post variables must not appear in covariates",
      ),
      ("drdid=", "drdid assignment requires a target before ="),
      ("drdid = 1", "drdid assignment requires a target before ="),
      ("drdid==", "unsupported token in command: =="),
      ("drdid == 1", "unsupported token in command: =="),
      ("drdid:", "unsupported token in command: :"),
      (
        "drdid: y, treat(d) post(t)",
        "unsupported token in command: :",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_dml_syntax() {
    assert_eq!(
      parse_command("dml linear y x1 x2, treat(d)").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string(), "x2".to_string()],
          treatment_variable: "d".to_string(),
          folds: 5,
          alpha: "1.0".to_string(),
          robust: false,
          seed: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear y x1, treat(d) folds(3)").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string()],
          treatment_variable: "d".to_string(),
          folds: 3,
          alpha: "1.0".to_string(),
          robust: false,
          seed: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear y x1 x2, treat(d) alpha(0.5)").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string(), "x2".to_string()],
          treatment_variable: "d".to_string(),
          folds: 5,
          alpha: "0.5".to_string(),
          robust: false,
          seed: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear y x1, treat(d) robust").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string()],
          treatment_variable: "d".to_string(),
          folds: 5,
          alpha: "1.0".to_string(),
          robust: true,
          seed: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear y x1, treat(d) seed(42)").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string()],
          treatment_variable: "d".to_string(),
          folds: 5,
          alpha: "1.0".to_string(),
          robust: false,
          seed: Some(42),
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear y x1, treat(d) noconstant").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string()],
          treatment_variable: "d".to_string(),
          folds: 5,
          alpha: "1.0".to_string(),
          robust: false,
          seed: None,
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command(
        "dml LINEAR y x1 x2, treat(d) folds(10) alpha(0.1) robust seed(123) noconstant"
      )
      .unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y".to_string(),
          controls: vec!["x1".to_string(), "x2".to_string()],
          treatment_variable: "d".to_string(),
          folds: 10,
          alpha: "0.1".to_string(),
          robust: true,
          seed: Some(123),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("dml linear `y var` `x var`, treat(`d var`)").unwrap(),
      Command::Dml {
        command: DmlCommand {
          outcome: "y var".to_string(),
          controls: vec!["x var".to_string()],
          treatment_variable: "d var".to_string(),
          folds: 5,
          alpha: "1.0".to_string(),
          robust: false,
          seed: None,
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_dml_syntax() {
    let cases = [
      (
        "dml",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      (
        "dml linear",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      (
        "dml linear y",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      (
        "dml linear y, treat(d)",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      (
        "dml linear y x if y > 0, treat(d)",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      ("dml logistic y x, treat(d)", "dml model must be linear"),
      ("dml `linear` y x, treat(d)", "dml model must be linear"),
      (
        "dml linear y x = 1, treat(d)",
        "dml expects syntax: dml linear <y> <controls>, treat(<var>) [folds(<int>) alpha(<num>) robust seed(<int>) noconstant]",
      ),
      (
        "dml linear y x == 1, treat(d)",
        "unsupported token in command: ==",
      ),
      (
        "dml linear y x,",
        "comma must be followed by at least one option",
      ),
      ("dml linear y x", "dml option treat expects one variable"),
      (
        "dml linear y x, treat",
        "dml option treat expects variables",
      ),
      (
        "dml linear y x, treat()",
        "option treat expects at least one value",
      ),
      (
        "dml linear y x, treat(d1 d2)",
        "dml option treat expects one variable",
      ),
      (
        "dml linear y x, treat(d) treat(d2)",
        "dml option treat may only be supplied once",
      ),
      (
        "dml linear y x, treat(y)",
        "dml treatment variable must differ from outcome",
      ),
      (
        "dml linear y x, treat(x)",
        "dml treatment variable must not appear in controls",
      ),
      (
        "dml linear y x, treat(d) folds(1)",
        "dml option folds must be at least 2",
      ),
      (
        "dml linear y x, treat(d) folds(0)",
        "dml option folds must be at least 2",
      ),
      (
        "dml linear y x, treat(d) folds(-1)",
        "dml option folds must be at least 2",
      ),
      (
        "dml linear y x, treat(d) folds(1.5)",
        "dml option folds expects an integer value",
      ),
      (
        "dml linear y x, treat(d) folds(abc)",
        "option folds expects a numeric value",
      ),
      (
        "dml linear y x, treat(d) folds",
        "dml option folds expects an integer value",
      ),
      (
        "dml linear y x, treat(d) folds()",
        "option folds expects at least one value",
      ),
      (
        "dml linear y x, treat(d) folds(5) folds(10)",
        "dml option folds may only be supplied once",
      ),
      (
        "dml linear y x, treat(d) alpha(-1)",
        "dml option alpha must be positive",
      ),
      (
        "dml linear y x, treat(d) alpha(0)",
        "dml option alpha must be positive",
      ),
      (
        "dml linear y x, treat(d) alpha(abc)",
        "option alpha expects a numeric value",
      ),
      (
        "dml linear y x, treat(d) alpha",
        "dml option alpha expects a numeric value",
      ),
      (
        "dml linear y x, treat(d) alpha()",
        "option alpha expects at least one value",
      ),
      (
        "dml linear y x, treat(d) alpha(1) alpha(2)",
        "dml option alpha may only be supplied once",
      ),
      (
        "dml linear y x, treat(d) seed(-1)",
        "dml option seed must be at least 0",
      ),
      (
        "dml linear y x, treat(d) seed(1.5)",
        "dml option seed expects an integer value",
      ),
      (
        "dml linear y x, treat(d) seed(abc)",
        "option seed expects a numeric value",
      ),
      (
        "dml linear y x, treat(d) seed",
        "dml option seed expects an integer value",
      ),
      (
        "dml linear y x, treat(d) seed()",
        "option seed expects at least one value",
      ),
      (
        "dml linear y x, treat(d) seed(1) seed(2)",
        "dml option seed may only be supplied once",
      ),
      (
        "dml linear y x, treat(d) robust=true",
        "dml option robust does not accept a value",
      ),
      (
        "dml linear y x, treat(d) robust(foo)",
        "dml option robust does not accept a value",
      ),
      (
        "dml linear y x, treat(d) noconstant=true",
        "dml option noconstant does not accept a value",
      ),
      (
        "dml linear y x, treat(d) noconstant(foo)",
        "dml option noconstant does not accept a value",
      ),
      (
        "dml linear y x, treat(d) extra",
        "dml unsupported option: extra",
      ),
      (
        "dml linear y x, treat(d) extra2 extra1",
        "dml unsupported option: extra1, extra2",
      ),
      ("dml linear y x, TREAT(d)", "dml unsupported option: TREAT"),
      ("dml=", "dml assignment requires a target before ="),
      ("dml = 1", "dml assignment requires a target before ="),
      ("dml==", "unsupported token in command: =="),
      ("dml == 1", "unsupported token in command: =="),
      ("dml:", "unsupported token in command: :"),
      (
        "dml: linear y x, treat(d)",
        "unsupported token in command: :",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_cfregress_syntax() {
    assert_eq!(
      parse_command("cfregress cost age bmi, endog(hours) iv(distance policy)").unwrap(),
      Command::CfRegress {
        command: CfRegressCommand {
          outcome: "cost".to_string(),
          exogenous: vec!["age".to_string(), "bmi".to_string()],
          endogenous: "hours".to_string(),
          instruments: vec!["distance".to_string(), "policy".to_string()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cfregress cost, endog(hours) iv(distance) robust").unwrap(),
      Command::CfRegress {
        command: CfRegressCommand {
          outcome: "cost".to_string(),
          exogenous: vec![],
          endogenous: "hours".to_string(),
          instruments: vec!["distance".to_string()],
          robust: true,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cfregress cost age, endog(hours) iv(distance) cluster(group_id) noconstant")
        .unwrap(),
      Command::CfRegress {
        command: CfRegressCommand {
          outcome: "cost".to_string(),
          exogenous: vec!["age".to_string()],
          endogenous: "hours".to_string(),
          instruments: vec!["distance".to_string()],
          robust: false,
          cluster_variable: Some("group_id".to_string()),
          include_intercept: false,
        },
      }
    );
    assert_eq!(
      parse_command("cfregress `cost var` `age var`, endog(`hours var`) iv(`dist var`)").unwrap(),
      Command::CfRegress {
        command: CfRegressCommand {
          outcome: "cost var".to_string(),
          exogenous: vec!["age var".to_string()],
          endogenous: "hours var".to_string(),
          instruments: vec!["dist var".to_string()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
    assert_eq!(
      parse_command("cfregress \"cost var\" \"age var\", endog(hours) iv(distance)").unwrap(),
      Command::CfRegress {
        command: CfRegressCommand {
          outcome: "cost var".to_string(),
          exogenous: vec!["age var".to_string()],
          endogenous: "hours".to_string(),
          instruments: vec!["distance".to_string()],
          robust: false,
          cluster_variable: None,
          include_intercept: true,
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_cfregress_syntax() {
    let cases = [
      (
        "cfregress",
        "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
      ),
      (
        "cfregress if x > 0, endog(d) iv(z)",
        "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
      ),
      (
        "cfregress y = 1, endog(d) iv(z)",
        "cfregress expects syntax: cfregress <y> [exog_vars], endog(<var>) iv(<vars>)",
      ),
      (
        "cfregress y == 1, endog(d) iv(z)",
        "unsupported token in command: ==",
      ),
      (
        "cfregress y,",
        "comma must be followed by at least one option",
      ),
      ("cfregress y", "cfregress option endog expects one variable"),
      (
        "cfregress y, endog(d)",
        "cfregress option iv expects at least one variable",
      ),
      (
        "cfregress y, iv(z)",
        "cfregress option endog expects one variable",
      ),
      (
        "cfregress y, endog() iv(z)",
        "option endog expects at least one value",
      ),
      (
        "cfregress y, endog(d1 d2) iv(z)",
        "cfregress option endog expects one variable",
      ),
      (
        "cfregress y, endog iv(z)",
        "cfregress option endog expects variables",
      ),
      (
        "cfregress y, endog(d) iv()",
        "option iv expects at least one value",
      ),
      (
        "cfregress y, endog(d) iv",
        "cfregress option iv expects variables",
      ),
      (
        "cfregress y, endog(d) iv(z) cluster",
        "cfregress option cluster expects variables",
      ),
      (
        "cfregress y, endog(d) iv(z) cluster()",
        "option cluster expects at least one value",
      ),
      (
        "cfregress y, endog(d) iv(z) cluster(c1 c2)",
        "cfregress option cluster expects one variable",
      ),
      (
        "cfregress y, endog(d) iv(z) robust cluster(c)",
        "cfregress cannot combine robust and cluster",
      ),
      (
        "cfregress y, endog(d) endog(d2) iv(z)",
        "cfregress option endog may only be supplied once",
      ),
      (
        "cfregress y, endog(d) iv(z) iv(z2)",
        "cfregress option iv may only be supplied once",
      ),
      (
        "cfregress y, endog(d) iv(z) cluster(c1) cluster(c2)",
        "cfregress option cluster may only be supplied once",
      ),
      (
        "cfregress y d, endog(d) iv(z)",
        "cfregress endog variable must not appear in exogenous variables",
      ),
      (
        "cfregress y, endog(d) iv(z) robust=true",
        "cfregress option robust does not accept a value",
      ),
      (
        "cfregress y, endog(d) iv(z) robust(foo)",
        "cfregress option robust does not accept a value",
      ),
      (
        "cfregress y, endog(d) iv(z) noconstant=true",
        "cfregress option noconstant does not accept a value",
      ),
      (
        "cfregress y, endog(d) iv(z) noconstant(foo)",
        "cfregress option noconstant does not accept a value",
      ),
      (
        "cfregress y, endog(d) iv(z) extra",
        "cfregress unsupported option: extra",
      ),
      (
        "cfregress y, endog(d) iv(z) extra2 extra1",
        "cfregress unsupported option: extra1, extra2",
      ),
      (
        "cfregress y, ENDOG(d) IV(z)",
        "cfregress unsupported option: ENDOG, IV",
      ),
      (
        "cfregress=",
        "cfregress assignment requires a target before =",
      ),
      (
        "cfregress = 1",
        "cfregress assignment requires a target before =",
      ),
      ("cfregress==", "unsupported token in command: =="),
      ("cfregress == 1", "unsupported token in command: =="),
      ("cfregress:", "unsupported token in command: :"),
      (
        "cfregress: y, endog(d) iv(z)",
        "unsupported token in command: :",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_lincom_command_forms() {
    assert_eq!(
      parse_command("lincom x1 - x2").unwrap(),
      Command::Lincom {
        command: LincomCommand {
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          },
        },
      }
    );

    assert_eq!(
      parse_command("lincom x1 + 2 * x2").unwrap(),
      Command::Lincom {
        command: LincomCommand {
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Number("2".to_owned())),
              operator: GenerateBinaryOperator::Multiply,
              right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
            }),
          },
        },
      }
    );

    assert_eq!(
      parse_command("lincom (x1 + x2) * 3").unwrap(),
      Command::Lincom {
        command: LincomCommand {
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
              operator: GenerateBinaryOperator::Add,
              right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
            }),
            operator: GenerateBinaryOperator::Multiply,
            right: Box::new(GenerateExpression::Number("3".to_owned())),
          },
        },
      }
    );

    assert_eq!(
      parse_command("lincom -x1 + x2").unwrap(),
      Command::Lincom {
        command: LincomCommand {
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::UnaryMinus(Box::new(
              GenerateExpression::Identifier("x1".to_owned())
            ))),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          },
        },
      }
    );

    assert_eq!(
      parse_command("LINCOM `wage rate` + 2 * `hours worked`").unwrap(),
      Command::Lincom {
        command: LincomCommand {
          expression: GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("wage rate".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Number("2".to_owned())),
              operator: GenerateBinaryOperator::Multiply,
              right: Box::new(GenerateExpression::Identifier("hours worked".to_owned())),
            }),
          },
        },
      }
    );

    assert_eq!(
      parse_command("by group: lincom x1 + x2").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["group".to_owned()],
          command: Box::new(Command::Lincom {
            command: LincomCommand {
              expression: GenerateExpression::Binary {
                left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
                operator: GenerateBinaryOperator::Add,
                right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
              },
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_lincom_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "lincom",
        "lincom command expects a linear combination expression",
      ),
      (
        "lincom   ",
        "lincom command expects a linear combination expression",
      ),
      ("lincom:", "unsupported token in command: :"),
      ("lincom: x1 + x2", "unsupported token in command: :"),
      ("lincom;", "unknown command: lincom;"),
      ("lincom x1;", "unsupported token in command: ;"),
      ("lincom=", "lincom assignment requires a target before ="),
      ("lincom=1", "lincom assignment requires a target before ="),
      ("lincom==", "unsupported token in command: =="),
      ("lincom==1", "unsupported token in command: =="),
      ("lincom,", "comma must be followed by at least one option"),
      ("lincom, level(95)", "unknown command: lincom"),
      ("lincom ,", "unsupported token in expression: ,"),
      ("lincom , level(95)", "unsupported token in expression: ,"),
      ("lincom = 1", "unsupported token in expression: ="),
      ("lincom == 1", "unsupported token in expression: =="),
      ("lincom x1 +", "incomplete expression after +"),
      ("lincom x1 *", "incomplete expression after *"),
      ("lincom (x1 + x2", "missing closing ) in expression"),
      ("lincom x1 + x2)", "unsupported token in expression: )"),
      ("lincom x1 + + x2", "unsupported token in expression: +"),
      ("lincom x1.x2", "unsupported token in expression: ."),
      ("lincom x1[0]", "unsupported token in command: ["),
      ("lincom x1 if x2 > 0", "unsupported token in expression: if"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_test_command_forms() {
    assert_eq!(
      parse_command("test x1").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Identifier("x1".to_owned())],
        },
      }
    );

    assert_eq!(
      parse_command("test x1 x2").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![
            GenerateExpression::Identifier("x1".to_owned()),
            GenerateExpression::Identifier("x2".to_owned()),
          ],
        },
      }
    );

    assert_eq!(
      parse_command("test x1 = x2").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test x1 == x2").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test x1 + 2 * x2 = 0").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
              operator: GenerateBinaryOperator::Add,
              right: Box::new(GenerateExpression::Binary {
                left: Box::new(GenerateExpression::Number("2".to_owned())),
                operator: GenerateBinaryOperator::Multiply,
                right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
              }),
            }),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Number("0".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test (x1 = x2) (x3 = 0)").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![
            GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
              operator: GenerateBinaryOperator::Subtract,
              right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
            },
            GenerateExpression::Binary {
              left: Box::new(GenerateExpression::Identifier("x3".to_owned())),
              operator: GenerateBinaryOperator::Subtract,
              right: Box::new(GenerateExpression::Number("0".to_owned())),
            },
          ],
        },
      }
    );

    assert_eq!(
      parse_command("test (x1 = x2)").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test (x1)").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Identifier("x1".to_owned())],
        },
      }
    );

    assert_eq!(
      parse_command("test (x1 + x2)").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Add,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test (x1 == 1)").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Number("1".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("test ( (x1) = 0 )").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Number("0".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("TEST x1 = x2").unwrap(),
      Command::Test {
        command: TestCommand {
          constraints: vec![GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
          }],
        },
      }
    );

    assert_eq!(
      parse_command("by group: test x1 = x2").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["group".to_owned()],
          command: Box::new(Command::Test {
            command: TestCommand {
              constraints: vec![GenerateExpression::Binary {
                left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
                operator: GenerateBinaryOperator::Subtract,
                right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
              }],
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_test_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "test",
        "test command expects a list of variables or constraints",
      ),
      (
        "test   ",
        "test command expects a list of variables or constraints",
      ),
      ("test:", "unsupported token in command: :"),
      ("test: x1", "unsupported token in command: :"),
      ("test;", "unknown command: test;"),
      ("test x1;", "unsupported token in command: ;"),
      ("test=", "test assignment requires a target before ="),
      ("test=1", "test assignment requires a target before ="),
      ("test==", "unsupported token in command: =="),
      ("test==1", "unsupported token in command: =="),
      ("test,", "comma must be followed by at least one option"),
      ("test, replace", "unknown command: test"),
      (
        "test ()",
        "test command: empty constraint inside parentheses",
      ),
      ("test (x1 = )", "test command: malformed constraint"),
      ("test ( = x1)", "test command: malformed constraint"),
      (
        "test (x1 = x2 = x3)",
        "test command: multiple '=' in a constraint",
      ),
      (
        "test x1 = x2 = x3",
        "test command: multiple '=' in a single constraint (use parentheses for multiple constraints)",
      ),
      (
        "test x1 =",
        "test command: missing right-hand side of constraint",
      ),
      (
        "test = x2",
        "test command: missing left-hand side of constraint",
      ),
      (
        "test (x1) extra",
        "test command: unexpected tokens outside parentheses",
      ),
      ("test (x1) (x2", "test command: mismatched parentheses"),
      ("test (x1))", "test command: mismatched parentheses"),
      (
        "test 123",
        "test command: expected variable name, got '123'",
      ),
      (
        "test x1, replace",
        "test command: expected variable name, got ','",
      ),
      (
        "test x1 if y > 0",
        "test command: expected variable name, got '>'",
      ),
      (
        "test (x1 = x2, replace)",
        "unsupported token in expression: ,",
      ),
      ("test ((x1 = 0))", "missing closing ) in expression"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_histogram_syntax() {
    assert_eq!(
      parse_command("histogram x").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: None,
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("histogram price, bins=20").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "price".into(),
          bins: Some(20),
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("histogram weight, bins = 15").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "weight".into(),
          bins: Some(15),
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("histogram x, saving(plot.png)").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: None,
          saving: Some("plot.png".into()),
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("histogram x, saving(\"my plot.png\")").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: None,
          saving: Some("my plot.png".into()),
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("histogram x, noopen").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: None,
          saving: None,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("histogram x, bins=25 saving(out.png) noopen").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: Some(25),
          saving: Some("out.png".into()),
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("histogram x, noopen bins=10 saving(\"path/to/plot.png\")").unwrap(),
      Command::Histogram {
        command: HistogramCommand {
          variable: "x".into(),
          bins: Some(10),
          saving: Some("path/to/plot.png".into()),
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("by foreign: histogram mpg, bins=10").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["foreign".into()],
          command: Box::new(Command::Histogram {
            command: HistogramCommand {
              variable: "mpg".into(),
              bins: Some(10),
              saving: None,
              open_artifact: true,
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_histogram_syntax_with_exact_diagnostics() {
    let cases = [
      ("histogram", "histogram expects exactly one variable"),
      ("histogram   ", "histogram expects exactly one variable"),
      ("histogram x y", "histogram expects exactly one variable"),
      ("histogram x y z", "histogram expects exactly one variable"),
      ("histogram:", "unsupported token in command: :"),
      ("histogram: x", "unsupported token in command: :"),
      (
        "histogram=",
        "histogram assignment requires a target before =",
      ),
      (
        "histogram=1",
        "histogram assignment requires a target before =",
      ),
      (
        "histogram = 1",
        "histogram assignment requires a target before =",
      ),
      ("histogram==", "unsupported token in command: =="),
      ("histogram==1", "unsupported token in command: =="),
      (
        "histogram,",
        "comma must be followed by at least one option",
      ),
      (
        "histogram, bins=10",
        "histogram expects exactly one variable",
      ),
      (
        "histogram x = 2",
        "histogram does not accept if clauses or assignment syntax",
      ),
      (
        "histogram x =",
        "histogram assignment requires an expression after =",
      ),
      (
        "histogram x if x > 0",
        "histogram does not accept if clauses or assignment syntax",
      ),
      ("histogram x, foo", "histogram unsupported option: foo"),
      (
        "histogram x, zebra apple",
        "histogram unsupported option: apple, zebra",
      ),
      (
        "histogram x, bins=0",
        "histogram option bins must be at least 1",
      ),
      (
        "histogram x, bins=1.5",
        "histogram option bins expects an integer value",
      ),
      (
        "histogram x, bins=abc",
        "histogram option bins expects an integer value",
      ),
      (
        "histogram x, bins=10 bins=20",
        "histogram option bins may only be supplied once",
      ),
      (
        "histogram x, saving",
        "histogram option saving expects a path",
      ),
      (
        "histogram x, saving(a) saving(b)",
        "histogram option saving may only be supplied once",
      ),
      (
        "histogram x, noopen=1",
        "histogram option noopen does not accept a value",
      ),
      (
        "histogram x, noopen(true)",
        "histogram option noopen does not accept a value",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_scatter_syntax() {
    assert_eq!(
      parse_command("scatter y x").unwrap(),
      Command::Scatter {
        command: ScatterCommand {
          y_variable: "y".into(),
          x_variable: "x".into(),
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("scatter price weight, saving(plot.png)").unwrap(),
      Command::Scatter {
        command: ScatterCommand {
          y_variable: "price".into(),
          x_variable: "weight".into(),
          saving: Some("plot.png".into()),
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("scatter price weight, saving(\"my plot.png\")").unwrap(),
      Command::Scatter {
        command: ScatterCommand {
          y_variable: "price".into(),
          x_variable: "weight".into(),
          saving: Some("my plot.png".into()),
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("scatter price weight, noopen").unwrap(),
      Command::Scatter {
        command: ScatterCommand {
          y_variable: "price".into(),
          x_variable: "weight".into(),
          saving: None,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("scatter price weight, saving(out.png) noopen").unwrap(),
      Command::Scatter {
        command: ScatterCommand {
          y_variable: "price".into(),
          x_variable: "weight".into(),
          saving: Some("out.png".into()),
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("by foreign: scatter price weight, noopen").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["foreign".into()],
          command: Box::new(Command::Scatter {
            command: ScatterCommand {
              y_variable: "price".into(),
              x_variable: "weight".into(),
              saving: None,
              open_artifact: false,
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_scatter_syntax_with_exact_diagnostics() {
    let cases = [
      ("scatter", "scatter expects syntax: scatter y_var x_var"),
      (
        "scatter price",
        "scatter expects syntax: scatter y_var x_var",
      ),
      (
        "scatter price weight extra",
        "scatter expects syntax: scatter y_var x_var",
      ),
      (
        "scatter, noopen",
        "scatter expects syntax: scatter y_var x_var",
      ),
      ("scatter:", "unsupported token in command: :"),
      ("scatter: price weight", "unsupported token in command: :"),
      ("scatter=", "scatter assignment requires a target before ="),
      ("scatter=1", "scatter assignment requires a target before ="),
      (
        "scatter = 1",
        "scatter assignment requires a target before =",
      ),
      ("scatter==", "unsupported token in command: =="),
      ("scatter==1", "unsupported token in command: =="),
      ("scatter,", "comma must be followed by at least one option"),
      (
        "scatter price weight = 2",
        "scatter does not accept if clauses or assignment syntax",
      ),
      (
        "scatter price weight =",
        "scatter assignment requires an expression after =",
      ),
      (
        "scatter price weight if price > 0",
        "scatter does not accept if clauses or assignment syntax",
      ),
      (
        "scatter price weight, foo",
        "scatter unsupported option: foo",
      ),
      (
        "scatter price weight, zebra apple",
        "scatter unsupported option: apple, zebra",
      ),
      (
        "scatter price weight, saving",
        "scatter option saving expects a path",
      ),
      (
        "scatter price weight, saving(a) saving(b)",
        "scatter option saving may only be supplied once",
      ),
      (
        "scatter price weight, noopen=1",
        "scatter option noopen does not accept a value",
      ),
      (
        "scatter price weight, noopen(true)",
        "scatter option noopen does not accept a value",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_valid_bar_syntax() {
    assert_eq!(
      parse_command("bar sex").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: false,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, missing noopen").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: true,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, saving(out.png)").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: Some("out.png".into()),
          include_missing: false,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, saving(\"my bar.png\")").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: Some("my bar.png".into()),
          include_missing: false,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, missing").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: true,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, noopen").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: false,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, saving(out.png) missing noopen").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: Some("out.png".into()),
          include_missing: true,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, missing missing").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: true,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bar sex, noopen noopen").unwrap(),
      Command::Bar {
        command: BarCommand {
          variable: "sex".into(),
          saving: None,
          include_missing: false,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("by foreign: bar sex, missing noopen").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["foreign".into()],
          command: Box::new(Command::Bar {
            command: BarCommand {
              variable: "sex".into(),
              saving: None,
              include_missing: true,
              open_artifact: false,
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_bar_syntax_with_exact_diagnostics() {
    let cases = [
      ("bar", "bar expects exactly one variable"),
      ("bar sex age", "bar expects exactly one variable"),
      ("bar, missing", "bar expects exactly one variable"),
      ("bar:", "unsupported token in command: :"),
      ("bar: sex", "unsupported token in command: :"),
      ("bar=", "bar assignment requires a target before ="),
      ("bar=1", "bar assignment requires a target before ="),
      ("bar = 1", "bar assignment requires a target before ="),
      ("bar==", "unsupported token in command: =="),
      ("bar==1", "unsupported token in command: =="),
      ("bar,", "comma must be followed by at least one option"),
      (
        "bar sex = 1",
        "bar does not accept if clauses or assignment syntax",
      ),
      ("bar sex =", "bar assignment requires an expression after ="),
      (
        "bar sex if age > 18",
        "bar does not accept if clauses or assignment syntax",
      ),
      ("bar sex, foo", "bar unsupported option: foo"),
      (
        "bar sex, zebra apple",
        "bar unsupported option: apple, zebra",
      ),
      ("bar sex, bins=20", "bar unsupported option: bins"),
      ("bar sex, saving", "bar option saving expects a path"),
      (
        "bar sex, saving(a) saving(b)",
        "bar option saving may only be supplied once",
      ),
      (
        "bar sex, missing=true",
        "bar option missing does not accept a value",
      ),
      (
        "bar sex, missing(a)",
        "bar option missing does not accept a value",
      ),
      (
        "bar sex, noopen=1",
        "bar option noopen does not accept a value",
      ),
      (
        "bar sex, noopen(true)",
        "bar option noopen does not accept a value",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_supported_bayesplot_commands() {
    assert_eq!(
      parse_command("bayesplot trace").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Trace,
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bayesplot density").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Density,
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bayesplot autocorrelation").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Autocorrelation,
          saving: None,
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bayesplot trace, noopen").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Trace,
          saving: None,
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("bayesplot density, saving(figures/posterior.svg)").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Density,
          saving: Some("figures/posterior.svg".into()),
          open_artifact: true,
        },
      }
    );
    assert_eq!(
      parse_command("bayesplot autocorrelation, saving(\"my plot.png\") noopen").unwrap(),
      Command::BayesPlot {
        command: BayesPlotCommand {
          kind: BayesPlotKind::Autocorrelation,
          saving: Some("my plot.png".into()),
          open_artifact: false,
        },
      }
    );
    assert_eq!(
      parse_command("by foreign: bayesplot trace, noopen").unwrap(),
      Command::By {
        command: ByCommand {
          groups: vec!["foreign".into()],
          command: Box::new(Command::BayesPlot {
            command: BayesPlotCommand {
              kind: BayesPlotKind::Trace,
              saving: None,
              open_artifact: false,
            },
          }),
        },
      }
    );
  }

  #[test]
  fn rejects_invalid_bayesplot_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "bayesplot",
        "bayesplot expects syntax: bayesplot <trace|density|autocorrelation>",
      ),
      (
        "bayesplot trace density",
        "bayesplot expects syntax: bayesplot <trace|density|autocorrelation>",
      ),
      (
        "bayesplot, noopen",
        "bayesplot expects syntax: bayesplot <trace|density|autocorrelation>",
      ),
      ("bayesplot:", "unsupported token in command: :"),
      ("bayesplot: trace", "unsupported token in command: :"),
      (
        "bayesplot=",
        "bayesplot assignment requires a target before =",
      ),
      (
        "bayesplot=1",
        "bayesplot assignment requires a target before =",
      ),
      (
        "bayesplot = 1",
        "bayesplot assignment requires a target before =",
      ),
      ("bayesplot==", "unsupported token in command: =="),
      ("bayesplot==1", "unsupported token in command: =="),
      (
        "bayesplot,",
        "comma must be followed by at least one option",
      ),
      (
        "bayesplot foo",
        "bayesplot kind must be trace, density, or autocorrelation",
      ),
      (
        "bayesplot TRACE",
        "bayesplot kind must be trace, density, or autocorrelation",
      ),
      (
        "bayesplot trace = 1",
        "bayesplot does not accept if clauses or assignment syntax",
      ),
      (
        "bayesplot trace =",
        "bayesplot assignment requires an expression after =",
      ),
      (
        "bayesplot trace if x > 0",
        "bayesplot does not accept if clauses or assignment syntax",
      ),
      ("bayesplot trace, foo", "bayesplot unsupported option: foo"),
      (
        "bayesplot trace, zebra apple",
        "bayesplot unsupported option: apple, zebra",
      ),
      (
        "bayesplot trace, bins=20",
        "bayesplot unsupported option: bins",
      ),
      (
        "bayesplot trace, saving",
        "bayesplot option saving expects a path",
      ),
      (
        "bayesplot trace, saving(a) saving(b)",
        "bayesplot option saving may only be supplied once",
      ),
      (
        "bayesplot trace, noopen=1",
        "bayesplot option noopen does not accept a value",
      ),
      (
        "bayesplot trace, noopen(yes)",
        "bayesplot option noopen does not accept a value",
      ),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }
}
