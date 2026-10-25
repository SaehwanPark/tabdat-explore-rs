//! Owned session snapshots and model state; never native connections or handles.

use crate::{DatasetInfo, ExecutionMode, LabelMetadata, LazyEngine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PanelKeys")]
pub struct PanelMetadata {
  id_variable: String,
  time_variable: String,
}

#[derive(Deserialize)]
struct PanelKeys {
  id_variable: String,
  time_variable: String,
}

impl TryFrom<PanelKeys> for PanelMetadata {
  type Error = &'static str;
  fn try_from(keys: PanelKeys) -> Result<Self, Self::Error> {
    Self::new(keys.id_variable, keys.time_variable)
  }
}

impl PanelMetadata {
  pub fn new(id: impl Into<String>, time: impl Into<String>) -> Result<Self, &'static str> {
    let id_variable = id.into();
    let time_variable = time.into();
    if id_variable.is_empty() || time_variable.is_empty() || id_variable == time_variable {
      return Err("panel keys must be nonempty and distinct");
    }
    Ok(Self {
      id_variable,
      time_variable,
    })
  }
  pub fn id_variable(&self) -> &str {
    &self.id_variable
  }
  pub fn time_variable(&self) -> &str {
    &self.time_variable
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelStructureSummary {
  pub observation_count: u64,
  pub entity_count: u64,
  pub time_count: u64,
  pub min_observations_per_entity: u64,
  pub max_observations_per_entity: u64,
}

impl PanelStructureSummary {
  pub fn is_balanced(&self) -> bool {
    self.min_observations_per_entity == self.max_observations_per_entity
  }
}

/// General metadata contract, including an uncomputed lazy row count.
/// DatasetInfo is retained as the existing eagerly measured compatibility type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetMetadata {
  pub source: crate::DataSource,
  pub row_count: Option<u64>,
  pub columns: Vec<crate::ColumnInfo>,
  pub execution_mode: ExecutionMode,
  pub lazy_engine: Option<LazyEngine>,
  pub panel_metadata: Option<PanelMetadata>,
  pub label_metadata: Option<LabelMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationReason {
  PolarsFallback,
  EagerOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimationFamily {
  Linear,
  Lasso,
  Postlasso,
  Ridge,
  Elasticnet,
  Cvlasso,
  Cvridge,
  Cvelasticnet,
  BayesLinear,
  BayesMcmc,
  Spatial,
  Quantile,
  Logit,
  Probit,
  MultinomialLogit,
  ConditionalLogit,
  NonNormalizedNestedLogit,
  NormalizedCommonNestedLogit,
  Tobit,
  TruncatedGaussian,
  Heckman,
  Nonlinear,
  Poisson,
  NegativeBinomial,
  ZeroInflatedPoisson,
  ZeroInflatedNegativeBinomial,
  Survival,
  InstrumentalVariables,
  PanelLinear,
  PanelLogit,
  Did,
  DrDid,
  Dml,
  DynamicPanel,
  ControlFunction,
}

/// Owned post-estimation cache for the executable linear OLS/2SLS paths.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearRegressionState {
  /// Commanded estimator provenance; old snapshots must not be guessed as OLS.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub estimator: Option<String>,
  pub outcome: String,
  pub predictors: Vec<String>,
  pub predictor_design: Vec<Vec<f64>>,
  pub include_intercept: bool,
  pub result: tabdat_stats::LeastSquaresResult,
}

/// Retained complete IV design for diagnostics; independent of subsequent data edits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentalVariablesState {
  pub linear: LinearRegressionState,
  pub problem: tabdat_stats::InstrumentalVariablesProblem,
}

/// Owned within-FE fit and explicit key provenance, independent of current data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedEffectsState {
  pub time_variable: String,
  pub entity_data_type: String,
  pub result: tabdat_stats::FixedEffectsResult,
}

/// Owned balanced/unbalanced one-way RE fit and explicit panel-key provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomEffectsState {
  pub time_variable: String,
  pub entity_data_type: String,
  pub result: tabdat_stats::RandomEffectsResult,
}

/// Owned binary logit design and maximum likelihood fit, independent of current data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryLogitState {
  pub problem: tabdat_stats::EstimationProblem,
  pub result: tabdat_stats::BinaryLogitResult,
}

/// Owned binary probit fit; the shared design/fit record with the family
/// encoded by `EstimationState::Probit` and the fitted `result.model`.
pub type BinaryProbitState = BinaryLogitState;

/// Complete finite design retained for continuous MNL effects after data edits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultinomialLogitState {
  pub problem: tabdat_stats::EstimationProblem,
  pub result: tabdat_stats::MultinomialLogitResult,
}

/// Owned complete choice-set design and model-based conditional likelihood fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalLogitState {
  pub problem: tabdat_stats::ConditionalLogitProblem,
  pub result: tabdat_stats::ConditionalLogitResult,
}

/// Owned retained design, structural nest map and non-normalized likelihood fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonNormalizedNestedLogitState {
  pub problem: tabdat_stats::NonNormalizedNestedLogitProblem,
  pub result: tabdat_stats::NonNormalizedNestedLogitResult,
}

/// Retained raw design and natural-utility common-scale normalized fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedCommonNestedLogitState {
  pub problem: tabdat_stats::NestedChoiceProblem,
  pub result: tabdat_stats::NormalizedCommonNestedLogitResult,
}

/// Owned Poisson design/fit, independent of subsequent current-data edits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoissonFitState {
  pub problem: tabdat_stats::EstimationProblem,
  pub result: tabdat_stats::PoissonResult,
}

/// Owned NB2 mean design and joint coefficient/log-dispersion fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeBinomialFitState {
  pub problem: tabdat_stats::EstimationProblem,
  pub result: tabdat_stats::Nb2Result,
}

/// Common normalized non-linear fit state. Backend-specific scoring caches stay
/// behind the adapter, and this record alone does not promise prediction support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelState {
  pub model: tabdat_stats::PostEstimationModel,
  pub sample: tabdat_stats::EstimationSample,
  pub outcome: String,
  pub predictors: Vec<String>,
}

/// Exactly one most-recent family, rather than independently nullable fit slots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "family", content = "state", rename_all = "snake_case")]
pub enum EstimationState {
  Linear(Box<LinearRegressionState>),
  Lasso(ModelState),
  Postlasso(ModelState),
  Ridge(ModelState),
  Elasticnet(ModelState),
  Cvlasso(ModelState),
  Cvridge(ModelState),
  Cvelasticnet(ModelState),
  BayesLinear(ModelState),
  BayesMcmc(ModelState),
  Spatial(ModelState),
  Quantile(ModelState),
  /// Installed certified conditional-quantile fit; legacy Quantile remains readable.
  QuantileFit(Box<tabdat_stats::QuantileResult>),
  Logit(Box<BinaryLogitState>),
  Probit(Box<BinaryProbitState>),
  MultinomialLogit(Box<MultinomialLogitState>),
  ConditionalLogit(Box<ConditionalLogitState>),
  NonNormalizedNestedLogit(Box<NonNormalizedNestedLogitState>),
  NormalizedCommonNestedLogit(Box<NormalizedCommonNestedLogitState>),
  /// Legacy normalized placeholder, retained for serialized-state compatibility.
  Tobit(ModelState),
  /// Owned Gaussian fit/sample and retained-X effect-gradient summaries.
  TobitFit(Box<tabdat_stats::TobitResult>),
  TruncatedGaussianFit(Box<tabdat_stats::TruncatedGaussianResult>),
  /// Owned entry-zero Weibull PH fit with full joint shape inference.
  WeibullFit(Box<tabdat_stats::WeibullResult>),
  Heckman(ModelState),
  HeckmanTwoStepFit(Box<tabdat_stats::HeckmanTwoStepResult>),
  Nonlinear(ModelState),
  /// Legacy normalized placeholder, retained for old serialized state reads.
  Poisson(ModelState),
  PoissonFit(Box<PoissonFitState>),
  /// Legacy normalized placeholder, not an installed fit.
  NegativeBinomial(ModelState),
  NegativeBinomialFit(Box<NegativeBinomialFitState>),
  ZeroInflatedPoisson(ModelState),
  ZeroInflatedNegativeBinomial(ModelState),
  Survival(ModelState),
  InstrumentalVariables(Box<InstrumentalVariablesState>),
  PanelLinear(ModelState),
  FixedEffects(Box<FixedEffectsState>),
  RandomEffects(Box<RandomEffectsState>),
  PanelLogit(ModelState),
  Did(ModelState),
  DrDid(ModelState),
  Dml(ModelState),
  DynamicPanel(ModelState),
  ControlFunction(ModelState),
}

impl EstimationState {
  pub fn linear(&self) -> Option<&LinearRegressionState> {
    match self {
      Self::Linear(state) => Some(state),
      Self::InstrumentalVariables(state) => Some(&state.linear),
      _ => None,
    }
  }
}

/// Explicit backend-free snapshot. Reading this does not initialize capabilities.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExecutionState {
  pub active_dataset: Option<DatasetInfo>,
  pub active_table_name: Option<String>,
  pub named_tables: BTreeMap<String, DatasetInfo>,
  pub labels: LabelMetadata,
  pub panel: Option<PanelMetadata>,
  pub last_operation: Option<String>,
  pub last_materialization_reason: Option<MaterializationReason>,
  pub estimation: Option<EstimationState>,
  /// Reporting snapshots only; independent of the active estimator and data.
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub stored_estimates: BTreeMap<String, crate::EstimateSnapshot>,
}

impl EstimationState {
  pub fn family(&self) -> EstimationFamily {
    match self {
      Self::Linear(..) => EstimationFamily::Linear,
      Self::Lasso(..) => EstimationFamily::Lasso,
      Self::Postlasso(..) => EstimationFamily::Postlasso,
      Self::Ridge(..) => EstimationFamily::Ridge,
      Self::Elasticnet(..) => EstimationFamily::Elasticnet,
      Self::Cvlasso(..) => EstimationFamily::Cvlasso,
      Self::Cvridge(..) => EstimationFamily::Cvridge,
      Self::Cvelasticnet(..) => EstimationFamily::Cvelasticnet,
      Self::BayesLinear(..) => EstimationFamily::BayesLinear,
      Self::BayesMcmc(..) => EstimationFamily::BayesMcmc,
      Self::Spatial(..) => EstimationFamily::Spatial,
      Self::Quantile(..) | Self::QuantileFit(..) => EstimationFamily::Quantile,
      Self::Logit(..) => EstimationFamily::Logit,
      Self::Probit(..) => EstimationFamily::Probit,
      Self::MultinomialLogit(..) => EstimationFamily::MultinomialLogit,
      Self::ConditionalLogit(..) => EstimationFamily::ConditionalLogit,
      Self::NonNormalizedNestedLogit(..) => EstimationFamily::NonNormalizedNestedLogit,
      Self::NormalizedCommonNestedLogit(..) => EstimationFamily::NormalizedCommonNestedLogit,
      Self::Tobit(..) | Self::TobitFit(..) => EstimationFamily::Tobit,
      Self::TruncatedGaussianFit(..) => EstimationFamily::TruncatedGaussian,
      Self::Heckman(..) | Self::HeckmanTwoStepFit(..) => EstimationFamily::Heckman,
      Self::Nonlinear(..) => EstimationFamily::Nonlinear,
      Self::Poisson(..) | Self::PoissonFit(..) => EstimationFamily::Poisson,
      Self::NegativeBinomial(..) | Self::NegativeBinomialFit(..) => {
        EstimationFamily::NegativeBinomial
      }
      Self::ZeroInflatedPoisson(..) => EstimationFamily::ZeroInflatedPoisson,
      Self::ZeroInflatedNegativeBinomial(..) => EstimationFamily::ZeroInflatedNegativeBinomial,
      Self::Survival(..) | Self::WeibullFit(..) => EstimationFamily::Survival,
      Self::InstrumentalVariables(..) => EstimationFamily::InstrumentalVariables,
      Self::PanelLinear(..) | Self::FixedEffects(..) | Self::RandomEffects(..) => {
        EstimationFamily::PanelLinear
      }
      Self::PanelLogit(..) => EstimationFamily::PanelLogit,
      Self::Did(..) => EstimationFamily::Did,
      Self::DrDid(..) => EstimationFamily::DrDid,
      Self::Dml(..) => EstimationFamily::Dml,
      Self::DynamicPanel(..) => EstimationFamily::DynamicPanel,
      Self::ControlFunction(..) => EstimationFamily::ControlFunction,
    }
  }
}
