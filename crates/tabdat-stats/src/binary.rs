//! Shared unweighted Bernoulli binary-response maximum likelihood: logit and
//! probit over one scaled Newton/QR core with family-specific link, observed
//! information, and marginal-effect conventions.

use crate::distributions::{chi_square_tail, normal_cdf, normal_pdf, normal_quantile};
use crate::matrix::{
  invert, qr_decompose_with_response, solve_upper_triangular, xtx_inverse_from_r,
};
use crate::{EstimationProblem, EstimationSample, StatsError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Which binary response mechanism the maximum likelihood assumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryModel {
  Logit,
  Probit,
}

/// Minimum number of requested bins for a Hosmer–Lemeshow reference test.
pub const MIN_BINARY_GOODNESS_OF_FIT_GROUPS: usize = 3;
/// Maximum number of requested bins in the bounded binary GOF command.
pub const MAX_BINARY_GOODNESS_OF_FIT_GROUPS: usize = 20;

/// One quantile group in a retained binary Hosmer–Lemeshow report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryHosmerLemeshowGroup {
  pub group: usize,
  /// Upper fitted-probability boundary, inclusive.
  pub probability_upper_bound: f64,
  pub observation_count: usize,
  pub observed_events: usize,
  pub expected_events: f64,
  pub observed_nonevents: usize,
  pub expected_nonevents: f64,
}

/// Quantile-grouped Hosmer–Lemeshow calibration test for a binary fit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryHosmerLemeshowResult {
  pub requested_group_count: usize,
  /// Tied fitted probabilities can make this smaller than requested.
  pub group_count: usize,
  pub statistic: f64,
  pub degrees_of_freedom: usize,
  pub p_value: f64,
  pub groups: Vec<BinaryHosmerLemeshowGroup>,
}

impl BinaryModel {
  fn label(self) -> &'static str {
    match self {
      Self::Logit => "logit",
      Self::Probit => "probit",
    }
  }
}

/// Normal Wald inference, with a fixed 95% confidence interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogitEstimate {
  pub name: String,
  pub value: f64,
  pub standard_error: f64,
  pub statistic: f64,
  pub p_value: f64,
  pub confidence_lower: f64,
  pub confidence_upper: f64,
}

/// Explicit iteration bound; runtime commands use the default bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogitOptions {
  pub max_iterations: usize,
}

impl Default for LogitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// Explicit iteration bound; runtime commands use the default bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbitOptions {
  pub max_iterations: usize,
}

impl Default for ProbitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 100,
    }
  }
}

/// One-way ML score-sandwich metadata. Inference is normal, not Student t;
/// the G/(G-1) correction alone does not ensure reliable few-cluster inference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryCluster {
  pub variable: String,
  pub group_count: usize,
}

/// Joint zero test of all predictor-column coefficients, excluding an intercept.
/// Unavailable tests retain the requested df; no pseudoinverse or reduced-rank
/// reference is substituted. This does not invalidate the fitted model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryOmnibusWald {
  pub statistic: Option<f64>,
  pub degrees_of_freedom: usize,
  pub p_value: Option<f64>,
  pub unavailable_reason: Option<String>,
}

#[derive(Clone, Copy)]
enum BinaryCovariance<'a> {
  ModelBased,
  Robust,
  Cluster(&'a str),
}

/// Unweighted pooled-Bernoulli binary maximum likelihood result (logit
/// or probit). Covariance convention is family-specific: inverse Fisher
/// information for `Logit`, inverse observed information evaluated at the
/// converged coefficients for `Probit`, both without an N-K correction.
/// With `robust`, the covariance is instead the score sandwich with N/(N-1)
/// scaling, or G/(G-1) when `cluster` is present (not linear HC1).
/// Coefficient/AME inference is
/// asymptotic normal, never Student t; ordinary LR inference is suppressed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryMleResult {
  pub model: BinaryModel,
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub include_intercept: bool,
  pub observation_count: usize,
  pub event_count: usize,
  pub coefficients: Vec<LogitEstimate>,
  pub covariance: Vec<Vec<f64>>,
  /// Score sandwich using the converged observed-information bread.
  #[serde(default)]
  pub robust: bool,
  #[serde(default)]
  pub cluster: Option<BinaryCluster>,
  /// Present for robust/clustered fits only; inference is asymptotic chi-square.
  #[serde(default)]
  pub omnibus_wald: Option<BinaryOmnibusWald>,
  pub log_likelihood: f64,
  pub null_log_likelihood: f64,
  pub likelihood_ratio: f64,
  pub likelihood_ratio_df: usize,
  /// None for robust fits or an intercept-only model (zero restrictions).
  pub likelihood_ratio_p_value: Option<f64>,
  pub mcfadden_r_squared: f64,
  /// Information evaluations, including the final convergence check.
  pub iterations: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_probabilities: Vec<f64>,
  /// Stable p*(1-p), even when a probability rounds to exactly one.
  pub fitted_weights: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

/// Test arbitrary linear restrictions on a retained logit or probit fit.
///
/// The selected model-based, independent-row robust or clustered covariance is
/// used as stored. The returned Wald statistic has an asymptotic chi-square
/// reference; no residual-F or few-cluster correction is implied.
pub fn binary_wald_test(
  result: &BinaryMleResult,
  r_matrix: &[Vec<f64>],
  r_vector: &[f64],
) -> Result<crate::WaldChiSquareTestResult, StatsError> {
  let coefficient_names = result
    .coefficients
    .iter()
    .map(|coefficient| coefficient.name.clone())
    .collect::<Vec<_>>();
  crate::post_estimation::validate_mean_coefficient_names(
    &coefficient_names,
    &result.predictor_names,
    result.include_intercept,
  )?;
  let parameters = result
    .coefficients
    .iter()
    .map(|coefficient| coefficient.value)
    .collect::<Vec<_>>();
  crate::post_estimation::wald_chi_square_test(&parameters, &result.covariance, r_matrix, r_vector)
}

/// Test grouped calibration on the retained model-based binary estimation sample.
///
/// The requested quantiles are formed over ordered fitted probabilities. Rows
/// with exactly equal fitted probabilities remain together at a cut boundary,
/// so the returned number of nonempty groups can be smaller than requested.
/// Robust, clustered and weighted fits are rejected because the standard
/// Hosmer–Lemeshow chi-square reference does not adjust for them. This follows
/// the grouped `estat gof, group(#)` construction in the Stata Base Reference
/// Manual (<https://www.stata.com/manuals/restatgof.pdf>); a bare
/// covariate-pattern Pearson test is a separate procedure.
pub fn binary_hosmer_lemeshow_test(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
  requested_group_count: usize,
) -> Result<BinaryHosmerLemeshowResult, StatsError> {
  if !(MIN_BINARY_GOODNESS_OF_FIT_GROUPS..=MAX_BINARY_GOODNESS_OF_FIT_GROUPS)
    .contains(&requested_group_count)
  {
    return Err(invalid(
      result.model,
      "Hosmer-Lemeshow group count must be an integer from 3 to 20",
    ));
  }
  problem.validate()?;
  if result.robust
    || result.cluster.is_some()
    || problem.weights.is_some()
    || problem
      .sample
      .as_ref()
      .is_some_and(|sample| sample.weights.is_some() || sample.cluster_groups.is_some())
  {
    return Err(invalid(
      result.model,
      "Hosmer-Lemeshow calibration requires an unweighted model-based fit without clustering",
    ));
  }
  let observation_count = problem.observation_count();
  if result.outcome != problem.outcome
    || result.predictor_names != problem.predictor_names
    || result.include_intercept != problem.include_intercept
    || result.sample != problem.sample
    || result.observation_count != observation_count
    || result
      .coefficients
      .iter()
      .map(|coefficient| coefficient.name.clone())
      .collect::<Vec<_>>()
      != problem.parameter_names()
    || result.fitted_probabilities.len() != observation_count
    || result.fitted_weights.len() != observation_count
    || result
      .fitted_weights
      .iter()
      .any(|weight| !weight.is_finite() || !(0.0..=0.25).contains(weight))
  {
    return Err(invalid(
      result.model,
      "retained problem and fitted result do not describe the same outcome, design and estimation sample",
    ));
  }
  if problem
    .sample
    .as_ref()
    .is_some_and(|sample| sample.retained_indices.len() != observation_count)
  {
    return Err(invalid(
      result.model,
      "retained response, fitted probabilities and estimation sample do not align",
    ));
  }

  let mut ordered = problem
    .response
    .iter()
    .copied()
    .zip(result.fitted_probabilities.iter().copied())
    .zip(result.fitted_weights.iter().copied())
    .map(|((response, probability), fitted_weight)| {
      if !matches!(response, 0.0 | 1.0) {
        return Err(invalid(
          result.model,
          "retained response must contain only exact zero and one values",
        ));
      }
      if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(invalid(
          result.model,
          "retained fitted probabilities must be finite values from zero to one",
        ));
      }
      // Recover the non-event tail when a fitted probability rounds to one;
      // the binary MLE retains p*(1-p) without that subtraction loss.
      let (event_probability, nonevent_probability) = if probability == 1.0 {
        if fitted_weight <= 0.0 || fitted_weight > f64::EPSILON {
          return Err(invalid(
            result.model,
            "a fitted probability rounded to one without a recoverable non-event tail",
          ));
        }
        let nonevent_probability = fitted_weight / probability;
        (1.0 - nonevent_probability, nonevent_probability)
      } else {
        (probability, 1.0 - probability)
      };
      Ok((
        probability,
        response,
        event_probability,
        nonevent_probability,
      ))
    })
    .collect::<Result<Vec<_>, _>>()?;
  if ordered.len() < MIN_BINARY_GOODNESS_OF_FIT_GROUPS {
    return Err(invalid(
      result.model,
      "Hosmer-Lemeshow test requires at least three retained observations",
    ));
  }
  ordered.sort_by(|left, right| left.0.total_cmp(&right.0));

  // Collapse exact probability ties before choosing quantile boundaries. The
  // reference manual keeps tied probabilities together, even when that makes
  // group sizes uneven or reduces the number of groups.
  let mut probability_patterns: Vec<(f64, usize)> = Vec::new();
  for (probability, _, _, _) in &ordered {
    if let Some((last_probability, count)) = probability_patterns.last_mut()
      && *last_probability == *probability
    {
      *count += 1;
    } else {
      probability_patterns.push((*probability, 1));
    }
  }

  let max_probability = probability_patterns
    .last()
    .expect("nonempty retained probabilities")
    .0;
  let mut boundaries = Vec::with_capacity(requested_group_count - 1);
  for quantile in 1..requested_group_count {
    // ceil(quantile * N / requested groups), calculated without multiplying N
    // by the quantile (which could overflow for a very large retained sample).
    let target = (observation_count / requested_group_count) * quantile
      + (observation_count % requested_group_count * quantile).div_ceil(requested_group_count);
    let mut cumulative = 0;
    for (probability, count) in &probability_patterns {
      cumulative += count;
      if cumulative >= target {
        if *probability < max_probability && boundaries.last() != Some(probability) {
          boundaries.push(*probability);
        }
        break;
      }
    }
  }

  #[derive(Default)]
  struct GroupAccumulator {
    observation_count: usize,
    observed_events: usize,
    expected_events: f64,
    expected_nonevents: f64,
    probability_upper_bound: f64,
  }
  let mut accumulators = (0..=boundaries.len())
    .map(|_| GroupAccumulator::default())
    .collect::<Vec<_>>();
  for (probability, response, event_probability, nonevent_probability) in ordered {
    // A probability equal to a cut remains in the lower group: intervals are
    // [p0, p1], (p1, p2], ... as in the reference manual.
    let group_index = boundaries.partition_point(|boundary| *boundary < probability);
    let group = &mut accumulators[group_index];
    group.observation_count += 1;
    group.observed_events += usize::from(response == 1.0);
    group.expected_events += event_probability;
    group.expected_nonevents += nonevent_probability;
    group.probability_upper_bound = probability;
  }

  let group_count = accumulators.len();
  if group_count < MIN_BINARY_GOODNESS_OF_FIT_GROUPS {
    return Err(invalid(
      result.model,
      "fitted-probability ties leave fewer than three nonempty Hosmer-Lemeshow groups",
    ));
  }
  let degrees_of_freedom = group_count - 2;
  let mut statistic = 0.0;
  let mut groups = Vec::with_capacity(group_count);
  for (index, group) in accumulators.into_iter().enumerate() {
    if group.observation_count == 0
      || !group.expected_events.is_finite()
      || !group.expected_nonevents.is_finite()
      || group.expected_events <= 0.0
      || group.expected_nonevents <= 0.0
    {
      return Err(invalid(
        result.model,
        "a Hosmer-Lemeshow group has an empty or zero expected outcome cell",
      ));
    }
    let observed_nonevents = group.observation_count - group.observed_events;
    let variance =
      group.expected_events * group.expected_nonevents / group.observation_count as f64;
    let residual = group.observed_events as f64 - group.expected_events;
    statistic += residual * residual / variance;
    groups.push(BinaryHosmerLemeshowGroup {
      group: index + 1,
      probability_upper_bound: group.probability_upper_bound,
      observation_count: group.observation_count,
      observed_events: group.observed_events,
      expected_events: group.expected_events,
      observed_nonevents,
      expected_nonevents: group.expected_nonevents,
    });
  }
  let p_value = chi_square_tail(statistic, degrees_of_freedom);
  if !statistic.is_finite() || !p_value.is_finite() || !(0.0..=1.0).contains(&p_value) {
    return Err(failed(
      result.model,
      "Hosmer-Lemeshow statistic or chi-square tail is non-finite",
    ));
  }
  Ok(BinaryHosmerLemeshowResult {
    requested_group_count,
    group_count,
    statistic,
    degrees_of_freedom,
    p_value,
    groups,
  })
}

/// Compatibility name for logit results before the shared binary core.
pub type BinaryLogitResult = BinaryMleResult;

/// Conditional-on-observed-design continuous average partial derivatives.
/// These are not discrete changes or chain-rule effects of original variables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogitMarginalEffects {
  #[serde(default)]
  pub robust: bool,
  #[serde(default)]
  pub cluster: Option<BinaryCluster>,
  pub observation_count: usize,
  pub effects: Vec<LogitEstimate>,
}

/// Average fitted-probability change when one binary predictor is set from
/// zero to one for every row in the retained estimation design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryDiscreteChange {
  pub model: BinaryModel,
  pub predictor: String,
  pub from_value: u8,
  pub to_value: u8,
  pub effect: LogitEstimate,
  pub robust: bool,
  pub cluster: Option<BinaryCluster>,
  pub observation_count: usize,
}

fn invalid(model: BinaryModel, message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("{}: {message}", model.label()))
}

fn failed(model: BinaryModel, message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("{}: {message}", model.label()))
}

fn inference(
  model: BinaryModel,
  name: String,
  value: f64,
  standard_error: f64,
) -> Result<LogitEstimate, StatsError> {
  let statistic = value / standard_error;
  // Chi-square(1) survival is the two-sided standard-normal tail. Direct
  // survival evaluation preserves very small representable probabilities.
  let p_value = chi_square_tail(statistic * statistic, 1);
  let confidence_lower = value - NORMAL_95 * standard_error;
  let confidence_upper = value + NORMAL_95 * standard_error;
  if standard_error <= 0.0
    || [
      value,
      standard_error,
      statistic,
      p_value,
      confidence_lower,
      confidence_upper,
    ]
    .iter()
    .any(|v| !v.is_finite())
  {
    return Err(failed(
      model,
      "normal inference is non-finite or has nonpositive variance",
    ));
  }
  Ok(LogitEstimate {
    name,
    value,
    standard_error,
    statistic,
    p_value,
    confidence_lower,
    confidence_upper,
  })
}

struct Evaluation {
  /// Fitted event probability per observation.
  probabilities: Vec<f64>,
  /// Stable p*(1-p) result payload; a probit step may use another diagonal.
  response_variances: Vec<f64>,
  /// Working diagonal for this iteration's information: the per-observation
  /// observed-information diagonal for probit's Newton step while every
  /// entry keeps positivity, the Fisher diagonal otherwise, and logit's
  /// stable p*(1-p) response variance.
  weights: Vec<f64>,
  /// Probit only: per-observation observed-information diagonal, reused for
  /// the converged covariance check.
  curvature_diagonal: Option<Vec<f64>>,
  /// Score residuals X'(these): logit y-p; probit (y-p)*phi/(p(1-p)).
  residuals: Vec<f64>,
  log_likelihood: f64,
}

fn evaluate(
  x: &[Vec<f64>],
  y: &[f64],
  beta: &[f64],
  model: BinaryModel,
) -> Result<Evaluation, StatsError> {
  let mut probabilities = Vec::with_capacity(y.len());
  let mut response_variances = Vec::with_capacity(y.len());
  let mut residuals = Vec::with_capacity(y.len());
  let mut curvature_diagonal = match model {
    BinaryModel::Logit => None,
    BinaryModel::Probit => Some(Vec::with_capacity(y.len())),
  };
  let mut fisher_diagonal = match model {
    BinaryModel::Logit => None,
    BinaryModel::Probit => Some(Vec::with_capacity(y.len())),
  };
  let mut log_likelihood = 0.0;
  for (row, &outcome) in x.iter().zip(y) {
    let eta: f64 = row.iter().zip(beta).map(|(a, b)| a * b).sum();
    if !eta.is_finite() {
      return Err(failed(model, "linear index is non-finite"));
    }
    match model {
      BinaryModel::Logit => {
        let tail = (-eta.abs()).exp();
        let denominator = 1.0 + tail;
        let (p, q) = if eta >= 0.0 {
          (1.0 / denominator, tail / denominator)
        } else {
          (tail / denominator, 1.0 / denominator)
        };
        probabilities.push(p);
        residuals.push(if outcome == 1.0 { q } else { -p });
        // Neither log(p) at saturation nor eta-softplus(eta) cancellation.
        let signed_eta = if outcome == 1.0 { -eta } else { eta };
        log_likelihood -= signed_eta.max(0.0) + tail.ln_1p();
        response_variances.push(tail / denominator.powi(2));
      }
      BinaryModel::Probit => {
        // Complementary half-tails: p and q sum to exactly one, and a tiny
        // tail holds its own precision instead of 1 - p cancellation.
        let p = normal_cdf(eta);
        if !p.is_finite() {
          return Err(failed(model, "fitted probability is non-finite"));
        }
        let q = normal_cdf(-eta);
        if !q.is_finite() {
          return Err(failed(model, "fitted probability is non-finite"));
        }
        // Density-to-probability ratio a = phi/(p q); the density and a
        // saturated tail underflow together, so a vanishing variance gives
        // a vanishing score contribution with finite, non-NaN pieces.
        let variance = p * q;
        let pdf = normal_pdf(eta);
        let score_slope = if variance > 0.0 { pdf / variance } else { 0.0 };
        let fisher = pdf * score_slope;
        probabilities.push(p);
        response_variances.push(variance);
        residuals.push((outcome - p) * score_slope);
        curvature_diagonal
          .as_mut()
          .unwrap()
          .push(fisher + (outcome - p) * score_slope * (eta + (1.0 - 2.0 * p) * score_slope));
        fisher_diagonal.as_mut().unwrap().push(fisher);
        // p = 1 - q is exact, so the log density is ln1p(-q) for events and
        // ln(q) otherwise; both are negative, and accumulating them keeps
        // no log(p) at saturation and no 1 - p cancellation.
        log_likelihood += if outcome == 1.0 { (-q).ln_1p() } else { q.ln() };
      }
    }
  }
  if !log_likelihood.is_finite() {
    return Err(failed(model, "log likelihood is non-finite"));
  }
  // Logit scores the Fisher information. Probit takes Newton on the observed
  // information while every per-observation diagonal keeps positivity; a
  // negative diagonal is a legitimate nonconcavity of the observed
  // information, so that iteration falls back to Fisher scoring entirely.
  let weights = if let Some(curvature) = &curvature_diagonal {
    if curvature.iter().any(|&j| !j.is_finite() || j < 0.0) {
      fisher_diagonal.as_ref().unwrap().clone()
    } else {
      curvature.clone()
    }
  } else {
    response_variances.clone()
  };
  Ok(Evaluation {
    probabilities,
    response_variances,
    curvature_diagonal,
    residuals,
    weights,
    log_likelihood,
  })
}

fn full_design(problem: &EstimationProblem) -> Vec<Vec<f64>> {
  problem
    .design_matrix
    .iter()
    .map(|row| {
      std::iter::repeat_n(1.0, usize::from(problem.include_intercept))
        .chain(row.iter().copied())
        .collect()
    })
    .collect()
}

fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

/// Fit an unweighted independent-Bernoulli logit MLE using the default
/// iteration bound.
pub fn fit_binary_logit(problem: &EstimationProblem) -> Result<BinaryMleResult, StatsError> {
  fit_binary_logit_with_options(problem, &LogitOptions::default())
}

/// Fit an unweighted independent-Bernoulli probit MLE using the default
/// iteration bound. Covariance is inverse observed information evaluated at
/// the converged coefficients.
pub fn fit_binary_probit(problem: &EstimationProblem) -> Result<BinaryMleResult, StatsError> {
  fit_binary_probit_with_options(problem, &ProbitOptions::default())
}

/// Fit an unweighted independent-Bernoulli logit MLE with an explicit
/// iteration bound.
pub fn fit_binary_logit_with_options(
  problem: &EstimationProblem,
  options: &LogitOptions,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(
    problem,
    BinaryModel::Logit,
    options.max_iterations,
    BinaryCovariance::ModelBased,
  )
}

pub fn fit_binary_probit_with_options(
  problem: &EstimationProblem,
  options: &ProbitOptions,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(
    problem,
    BinaryModel::Probit,
    options.max_iterations,
    BinaryCovariance::ModelBased,
  )
}

/// Fit either binary family with unclustered score-sandwich covariance,
/// corrected by N/(N-1). Fitting and convergence are unchanged from the
/// model-based MLE. Ordinary LR p-values are unavailable under this inference.
pub fn fit_binary_robust(
  problem: &EstimationProblem,
  model: BinaryModel,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(
    problem,
    model,
    LogitOptions::default().max_iterations,
    BinaryCovariance::Robust,
  )
}

/// Fit with one-way clustered individual scores and G/(G-1) correction.
/// The sample must contain a group identifier for every retained row and at
/// least two groups. Coefficient/conditional-X AME inference is asymptotic
/// normal, assuming independent clusters; ordinary LR inference is suppressed.
pub fn fit_binary_clustered(
  problem: &EstimationProblem,
  model: BinaryModel,
  cluster_variable: &str,
) -> Result<BinaryMleResult, StatsError> {
  fit_binary_mle(
    problem,
    model,
    LogitOptions::default().max_iterations,
    BinaryCovariance::Cluster(cluster_variable),
  )
}

fn add_score_outer(matrix: &mut [Vec<f64>], score: &[f64]) {
  for (i, row) in matrix.iter_mut().enumerate() {
    for (j, entry) in row.iter_mut().enumerate() {
      *entry += score[i] * score[j];
    }
  }
}

pub(crate) fn omnibus_wald(
  values: &[f64],
  covariance: &[Vec<f64>],
  include_intercept: bool,
  cluster_group_count: Option<usize>,
) -> BinaryOmnibusWald {
  let start = usize::from(include_intercept);
  let q = values.len() - start;
  let unavailable = |reason: &str| BinaryOmnibusWald {
    statistic: None,
    degrees_of_freedom: q,
    p_value: None,
    unavailable_reason: Some(reason.to_owned()),
  };
  if q == 0 {
    return unavailable("no_non_intercept_coefficients");
  }
  // Cluster score sums add to zero at the fitted estimating equations,
  // bounding meat rank by G-1.
  // Roundoff must not manufacture an invertible q>=G restriction covariance.
  if cluster_group_count.is_some_and(|groups| q >= groups) {
    return unavailable("insufficient_cluster_rank");
  }
  let slopes = &values[start..];
  let errors: Vec<f64> = (start..values.len())
    .map(|i| covariance[i][i].sqrt())
    .collect();
  if errors.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return unavailable("singular_slope_covariance");
  }
  // Inverting the slope correlation matrix avoids a rank decision that depends
  // on the measurement units of any predictor (not just their common scale).
  let correlation: Vec<Vec<f64>> = (0..q)
    .map(|i| {
      (0..q)
        .map(|j| covariance[start + i][start + j] / errors[i] / errors[j])
        .collect()
    })
    .collect();
  let Ok(inverse) = invert(&correlation) else {
    return unavailable("singular_slope_covariance");
  };
  let statistic: f64 = slopes
    .iter()
    .enumerate()
    .map(|(i, a)| {
      slopes
        .iter()
        .enumerate()
        .map(|(j, b)| (a / errors[i]) * inverse[i][j] * (b / errors[j]))
        .sum::<f64>()
    })
    .sum();
  let p_value = chi_square_tail(statistic, q);
  if !statistic.is_finite() || statistic < 0.0 || !p_value.is_finite() {
    return unavailable("nonfinite_wald_inference");
  }
  BinaryOmnibusWald {
    statistic: Some(statistic),
    degrees_of_freedom: q,
    p_value: Some(p_value),
    unavailable_reason: None,
  }
}

#[test]
fn omnibus_wald_does_not_invert_a_rank_deficient_slope_covariance() {
  let wald = omnibus_wald(&[1.0, 2.0], &[vec![1.0, 2.0], vec![2.0, 4.0]], false, None);
  assert_eq!(wald.degrees_of_freedom, 2);
  assert!(wald.statistic.is_none() && wald.p_value.is_none());
  assert_eq!(
    wald.unavailable_reason.as_deref(),
    Some("singular_slope_covariance")
  );
}

/// Scaled Newton/QR with deterministic backtracking for both binary families.
/// A small likelihood change or score alone does not establish convergence
/// under separation: the *full* Newton step must also be small. Singular
/// information or exhaustion fails without publishing coefficients. This is
/// not an analytic separation classifier. Logit scores the Fisher
/// information; probit takes Newton on the observed information with a
/// Fisher-scoring fallback when a per-observation diagonal loses positivity.
fn fit_binary_mle(
  problem: &EstimationProblem,
  model: BinaryModel,
  max_iterations: usize,
  covariance_type: BinaryCovariance<'_>,
) -> Result<BinaryMleResult, StatsError> {
  problem.validate()?;
  if max_iterations == 0 {
    return Err(invalid(model, "max_iterations must be positive"));
  }
  if problem.weights.is_some() || problem.sample.as_ref().is_some_and(|s| s.weights.is_some()) {
    return Err(invalid(model, "weights are not implemented"));
  }
  let groups = problem
    .sample
    .as_ref()
    .and_then(|sample| sample.cluster_groups.as_ref());
  let cluster = match covariance_type {
    BinaryCovariance::Cluster(variable) => {
      if variable.trim().is_empty() {
        return Err(invalid(model, "cluster variable must be named"));
      }
      let groups = groups.ok_or_else(|| invalid(model, "cluster groups are required"))?;
      if groups.len() != problem.observation_count() {
        return Err(invalid(
          model,
          "cluster groups must align with the estimation sample",
        ));
      }
      let group_count = groups.iter().collect::<BTreeSet<_>>().len();
      if group_count < 2 {
        return Err(invalid(
          model,
          "cluster covariance requires at least two retained groups",
        ));
      }
      Some(BinaryCluster {
        variable: variable.to_owned(),
        group_count,
      })
    }
    _ => {
      if groups.is_some() {
        return Err(invalid(model, "cluster groups require a clustered fit"));
      }
      None
    }
  };
  let robust = !matches!(covariance_type, BinaryCovariance::ModelBased);
  if problem.response.iter().any(|v| *v != 0.0 && *v != 1.0) {
    return Err(invalid(
      model,
      "outcome must be exactly 0 or 1 on the estimation sample",
    ));
  }
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if n <= k {
    return Err(StatsError::InsufficientObservations(format!(
      "{} requires more observations than parameters",
      model.label()
    )));
  }
  let events = problem.response.iter().filter(|&&v| v == 1.0).count();
  if events == 0 || events == n {
    return Err(invalid(
      model,
      "this bounded workflow requires both outcome classes",
    ));
  }
  let names = problem.parameter_names();
  if names.iter().collect::<BTreeSet<_>>().len() != k {
    return Err(invalid(
      model,
      "parameter names must be distinct, including the intercept",
    ));
  }
  let mut x = full_design(problem);
  let scales: Vec<f64> = (0..k)
    .map(|j| x.iter().fold(0.0_f64, |norm, row| norm.hypot(row[j])))
    .collect();
  if scales.iter().any(|s| !s.is_finite() || *s <= 0.0) {
    return Err(invalid(
      model,
      "predictor norms must be finite and positive",
    ));
  }
  for row in &mut x {
    for (entry, scale) in row.iter_mut().zip(&scales) {
      *entry /= scale;
    }
  }
  let zero_response = vec![0.0; n];
  // Reject a deficient original design separately from vanishing information.
  qr_decompose_with_response(&x, &zero_response)?;
  let mut beta = vec![0.0; k];
  let proportion = events as f64 / n as f64;
  let null_log_likelihood = if problem.include_intercept {
    // The start inverts the family link at the event proportion, in
    // unit-column-norm coordinates; everything else starts at zero.
    beta[0] = match model {
      BinaryModel::Logit => (proportion / (1.0 - proportion)).ln() * scales[0],
      BinaryModel::Probit => normal_quantile(proportion) * scales[0],
    };
    events as f64 * proportion.ln() + (n - events) as f64 * (-proportion).ln_1p()
  } else {
    -(n as f64) * std::f64::consts::LN_2
  };
  for iteration in 1..=max_iterations {
    let current = evaluate(&x, &problem.response, &beta, model)?;
    let weighted: Vec<Vec<f64>> = x
      .iter()
      .zip(&current.weights)
      .map(|(row, w)| row.iter().map(|entry| entry * w.sqrt()).collect())
      .collect();
    let qr = qr_decompose_with_response(&weighted, &zero_response).map_err(|_| {
      failed(
        model,
        "information is singular or numerically unidentified (possible separation)",
      )
    })?;
    let score: Vec<f64> = (0..k)
      .map(|j| {
        x.iter()
          .zip(&current.residuals)
          .map(|(row, residual)| row[j] * residual)
          .sum()
      })
      .collect();
    // R'R step = score, without forming the normal matrix or dividing a
    // working response by tiny/zero Bernoulli standard deviations.
    let mut intermediate = vec![0.0; k];
    for i in 0..k {
      let preceding: f64 = (0..i).map(|j| qr.r[j][i] * intermediate[j]).sum();
      intermediate[i] = (score[i] - preceding) / qr.r[i][i];
    }
    let step = solve_upper_triangular(&qr.r, &intermediate)?;
    if step.iter().chain(&score).any(|v| !v.is_finite()) {
      return Err(failed(model, "Newton step or score is non-finite"));
    }
    let step_norm = maximum_absolute(&step);
    let score_norm = maximum_absolute(&score);
    // Absolute in unit-column-norm coordinates: another large coefficient
    // must not relax the bound on a still-diverging separation direction.
    if step_norm <= TOLERANCE && score_norm <= TOLERANCE {
      let scaled_covariance = match model {
        BinaryModel::Logit => xtx_inverse_from_r(&qr.r)?,
        BinaryModel::Probit => {
          let curvature = current.curvature_diagonal.as_ref().unwrap();
          if curvature.iter().any(|&j| !j.is_finite() || j < 0.0) {
            return Err(failed(
              model,
              "observed information has nonpositive curvature",
            ));
          }
          // This evaluation's step diagonal is the observed information, so
          // the converged QR is itself R'R = observed information.
          xtx_inverse_from_r(&qr.r)?
        }
      };
      let scaled_covariance = if robust {
        // Sum outer products of bread * individual score. Work in the same
        // unit-column-norm coordinates as the Newton core, then transform
        // back once below; forming an unscaled meat can overflow needlessly.
        let mut sandwich = vec![vec![0.0; k]; k];
        let mut grouped = BTreeMap::<&str, Vec<f64>>::new();
        for (index, (row, residual)) in x.iter().zip(&current.residuals).enumerate() {
          let influence: Vec<f64> = scaled_covariance
            .iter()
            .map(|bread_row| bread_row.iter().zip(row).map(|(a, b)| a * b).sum::<f64>() * residual)
            .collect();
          if cluster.is_some() {
            let group = grouped
              .entry(groups.unwrap()[index].as_str())
              .or_insert_with(|| vec![0.0; k]);
            for (entry, value) in group.iter_mut().zip(&influence) {
              *entry += value;
            }
          } else {
            add_score_outer(&mut sandwich, &influence);
          }
        }
        // Aggregate within cluster before the outer product: individual
        // outer products would discard the within-cluster cross terms.
        for score in grouped.values() {
          add_score_outer(&mut sandwich, score);
        }
        let units = cluster.as_ref().map_or(n, |cluster| cluster.group_count);
        let correction = units as f64 / (units - 1) as f64;
        for row in &mut sandwich {
          for entry in row {
            *entry *= correction;
          }
        }
        sandwich
      } else {
        scaled_covariance
      };
      let covariance: Vec<Vec<f64>> = scaled_covariance
        .iter()
        .enumerate()
        .map(|(i, row)| {
          row
            .iter()
            .enumerate()
            .map(|(j, v)| v / scales[i] / scales[j])
            .collect()
        })
        .collect();
      if covariance.iter().flatten().any(|v| !v.is_finite()) {
        return Err(failed(
          model,
          "covariance cannot be represented in original predictor units",
        ));
      }
      let coefficients: Vec<LogitEstimate> = (0..k)
        .map(|j| {
          inference(
            model,
            names[j].clone(),
            beta[j] / scales[j],
            covariance[j][j].sqrt(),
          )
        })
        .collect::<Result<Vec<_>, _>>()?;
      let likelihood_ratio = 2.0 * (current.log_likelihood - null_log_likelihood);
      if likelihood_ratio < -1e-10 * (1.0 + null_log_likelihood.abs()) {
        return Err(failed(
          model,
          "fitted likelihood is below the nested null model",
        ));
      }
      let likelihood_ratio = likelihood_ratio.max(0.0);
      let df = problem.predictor_names.len();
      let likelihood_ratio_p_value =
        (df > 0 && !robust).then(|| chi_square_tail(likelihood_ratio, df));
      if likelihood_ratio_p_value.is_some_and(|p| !p.is_finite()) {
        return Err(failed(
          model,
          "likelihood-ratio chi-square inference is unavailable",
        ));
      }
      let omnibus_wald = robust.then(|| {
        omnibus_wald(
          &coefficients.iter().map(|c| c.value).collect::<Vec<_>>(),
          &covariance,
          problem.include_intercept,
          cluster.as_ref().map(|c| c.group_count),
        )
      });
      return Ok(BinaryMleResult {
        model,
        outcome: problem.outcome.clone(),
        predictor_names: problem.predictor_names.clone(),
        include_intercept: problem.include_intercept,
        observation_count: n,
        event_count: events,
        coefficients,
        covariance,
        robust,
        cluster,
        omnibus_wald,
        log_likelihood: current.log_likelihood,
        null_log_likelihood,
        likelihood_ratio,
        likelihood_ratio_df: df,
        likelihood_ratio_p_value,
        mcfadden_r_squared: -likelihood_ratio / (2.0 * null_log_likelihood),
        iterations: iteration,
        max_abs_scaled_score: score_norm,
        max_abs_scaled_newton_step: step_norm,
        fitted_probabilities: current.probabilities,
        fitted_weights: current.response_variances,
        sample: problem.sample.clone(),
      });
    }
    let directional: f64 = step.iter().zip(&score).map(|(a, b)| a * b).sum();
    let mut fraction = 1.0;
    let mut accepted = None;
    for _ in 0..50 {
      let candidate: Vec<f64> = beta
        .iter()
        .zip(&step)
        .map(|(b, d)| b + fraction * d)
        .collect();
      if let Ok(evaluation) = evaluate(&x, &problem.response, &candidate, model) {
        // Allow likelihood accumulation roundoff, not a convergence shortcut.
        let slack = 1e-12 * (1.0 + current.log_likelihood.abs());
        if evaluation.log_likelihood
          >= current.log_likelihood + 1e-4 * fraction * directional - slack
        {
          accepted = Some(candidate);
          break;
        }
      }
      fraction *= 0.5;
    }
    beta = accepted
      .ok_or_else(|| failed(model, "backtracking could not find a finite improving step"))?;
  }
  Err(failed(
    model,
    &format!(
      "did not converge within {} information evaluations; possible separation or ill-conditioning",
      max_iterations
    ),
  ))
}

/// Average continuous predictor-column effects over the *retained training X*.
/// Caller must provide the exact problem used for this fit. Uncertainty is the
/// normal delta method conditional on observed X, through the fitted family's
/// own covariance convention, including intercept/slopes' covariance. It does
/// not estimate the sampling variation of X's distribution.
pub fn binary_average_marginal_effects(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
) -> Result<LogitMarginalEffects, StatsError> {
  problem.validate()?;
  let model = result.model;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  if result.outcome != problem.outcome
    || result.predictor_names != problem.predictor_names
    || result.include_intercept != problem.include_intercept
    || result.sample != problem.sample
    || result.observation_count != n
    || result
      .coefficients
      .iter()
      .map(|c| c.name.clone())
      .collect::<Vec<_>>()
      != problem.parameter_names()
    || result.fitted_probabilities.len() != n
    || result.fitted_weights.len() != n
    || result.covariance.len() != k
    || result.covariance.iter().any(|row| row.len() != k)
  {
    return Err(invalid(
      model,
      "marginal effects require the exact retained fit/design/sample",
    ));
  }
  if result
    .fitted_probabilities
    .iter()
    .any(|p| !p.is_finite() || !(0.0..=1.0).contains(p))
    || result
      .fitted_weights
      .iter()
      .any(|w| !w.is_finite() || !(0.0..=0.25).contains(w))
    || result.covariance.iter().flatten().any(|v| !v.is_finite())
    || result.coefficients.iter().any(|c| !c.value.is_finite())
  {
    return Err(failed(
      model,
      "marginal effects require finite fit quantities",
    ));
  }
  let x = full_design(problem);
  // (link derivative, link second derivative) mean structures.
  let (mean_link, mean_link_change) = match model {
    BinaryModel::Logit => {
      // Stored from the converged fit: stable p*(1-p) stays the link
      // derivative even when a probability rounds to exactly one.
      let mean_link = result.fitted_weights.iter().sum::<f64>() / n as f64;
      let mean_link_change: Vec<f64> = (0..k)
        .map(|j| {
          x.iter()
            .zip(&result.fitted_weights)
            .zip(&result.fitted_probabilities)
            .map(|((row, w), p)| row[j] * w * (1.0 - 2.0 * p))
            .sum::<f64>()
            / n as f64
        })
        .collect();
      (mean_link, mean_link_change)
    }
    BinaryModel::Probit => {
      // Density pieces recomputed from the reported coefficients and the
      // retained design, so saturation in the stored probabilities cannot
      // echo into the delta method.
      let etas: Vec<f64> = x
        .iter()
        .map(|row| {
          row
            .iter()
            .zip(&result.coefficients)
            .map(|(a, c)| a * c.value)
            .sum()
        })
        .collect();
      if etas.iter().any(|z| !z.is_finite()) {
        return Err(failed(
          model,
          "marginal effects require finite fit quantities",
        ));
      }
      let mean_link = etas.iter().map(|&z| normal_pdf(z)).sum::<f64>() / n as f64;
      let mean_link_change: Vec<f64> = (0..k)
        .map(|j| {
          x.iter()
            .zip(&etas)
            .map(|(row, z)| row[j] * (-(z * normal_pdf(*z))))
            .sum::<f64>()
            / n as f64
        })
        .collect();
      (mean_link, mean_link_change)
    }
  };
  let mut effects = Vec::new();
  for (column, name) in problem.predictor_names.iter().enumerate() {
    let index = column + usize::from(problem.include_intercept);
    let beta = result.coefficients[index].value;
    let mut gradient: Vec<f64> = mean_link_change.iter().map(|d| beta * d).collect();
    gradient[index] += mean_link;
    let variance: f64 = (0..k)
      .map(|i| {
        (0..k)
          .map(|j| gradient[i] * result.covariance[i][j] * gradient[j])
          .sum::<f64>()
      })
      .sum();
    effects.push(inference(
      model,
      name.clone(),
      beta * mean_link,
      variance.sqrt(),
    )?);
  }
  Ok(LogitMarginalEffects {
    robust: result.robust,
    cluster: result.cluster.clone(),
    observation_count: n,
    effects,
  })
}

fn binary_link_and_derivative(model: BinaryModel, eta: f64) -> (f64, f64) {
  match model {
    BinaryModel::Logit => {
      let tail = (-eta.abs()).exp();
      let denominator = 1.0 + tail;
      let probability = if eta >= 0.0 {
        1.0 / denominator
      } else {
        tail / denominator
      };
      (probability, tail / denominator.powi(2))
    }
    BinaryModel::Probit => (normal_cdf(eta), normal_pdf(eta)),
  }
}

/// Estimate the average discrete probability change for one numeric 0/1
/// predictor over the retained fit design. Other predictors stay at their
/// observed row values. The full gradient is evaluated against the fitted
/// model-based, robust or clustered covariance and inference is asymptotic
/// normal conditional on observed X.
pub fn binary_average_discrete_change(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
  predictor: &str,
) -> Result<BinaryDiscreteChange, StatsError> {
  problem.validate()?;
  let model = result.model;
  let n = problem.observation_count();
  let k = problem.parameter_count();
  let parameter_names = problem.parameter_names();
  if result.outcome != problem.outcome
    || result.predictor_names != problem.predictor_names
    || result.include_intercept != problem.include_intercept
    || result.sample != problem.sample
    || result.observation_count != n
    || result
      .coefficients
      .iter()
      .map(|coefficient| coefficient.name.clone())
      .collect::<Vec<_>>()
      != parameter_names
    || result.covariance.len() != k
    || result.covariance.iter().any(|row| row.len() != k)
    || result.fitted_probabilities.len() != n
  {
    return Err(invalid(
      model,
      "discrete changes require the exact retained fit/design/sample",
    ));
  }
  if result
    .coefficients
    .iter()
    .any(|coefficient| !coefficient.value.is_finite())
    || result
      .covariance
      .iter()
      .flatten()
      .any(|value| !value.is_finite())
  {
    return Err(failed(
      model,
      "discrete changes require finite fit quantities",
    ));
  }
  let Some(predictor_index) = problem
    .predictor_names
    .iter()
    .position(|name| name == predictor)
  else {
    return Err(invalid(
      model,
      &format!("discrete change predictor '{predictor}' is not in the retained model"),
    ));
  };
  let observed_values = problem
    .design_matrix
    .iter()
    .map(|row| row[predictor_index])
    .collect::<Vec<_>>();
  if observed_values
    .iter()
    .any(|value| *value != 0.0 && *value != 1.0)
  {
    return Err(invalid(
      model,
      &format!(
        "discrete change predictor '{predictor}' must contain only 0 and 1 in the retained estimation sample"
      ),
    ));
  }
  if !observed_values.contains(&0.0) || !observed_values.contains(&1.0) {
    return Err(invalid(
      model,
      &format!(
        "discrete change predictor '{predictor}' requires both 0 and 1 in the retained estimation sample"
      ),
    ));
  }

  let coefficient_values = result
    .coefficients
    .iter()
    .map(|coefficient| coefficient.value)
    .collect::<Vec<_>>();
  let column_index = predictor_index + usize::from(problem.include_intercept);
  let linear_index = |row: &[f64]| {
    row
      .iter()
      .zip(&coefficient_values)
      .map(|(value, coefficient)| value * coefficient)
      .sum::<f64>()
  };
  let mut effect_sum = 0.0;
  let mut gradient_sum = vec![0.0; k];
  for row in full_design(problem) {
    let mut from = row.clone();
    let mut to = row;
    from[column_index] = 0.0;
    to[column_index] = 1.0;
    let eta_from = linear_index(&from);
    let eta_to = linear_index(&to);
    if !eta_from.is_finite() || !eta_to.is_finite() {
      return Err(failed(model, "discrete change linear index is non-finite"));
    }
    let (probability_from, derivative_from) = binary_link_and_derivative(model, eta_from);
    let (probability_to, derivative_to) = binary_link_and_derivative(model, eta_to);
    if [
      probability_from,
      derivative_from,
      probability_to,
      derivative_to,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
      return Err(failed(model, "discrete change probability is non-finite"));
    }
    effect_sum += probability_to - probability_from;
    for column in 0..k {
      gradient_sum[column] += derivative_to * to[column] - derivative_from * from[column];
    }
  }
  let effect_value = effect_sum / n as f64;
  let gradient = gradient_sum
    .into_iter()
    .map(|value| value / n as f64)
    .collect::<Vec<_>>();
  let variance = (0..k)
    .map(|i| {
      (0..k)
        .map(|j| gradient[i] * result.covariance[i][j] * gradient[j])
        .sum::<f64>()
    })
    .sum::<f64>();
  if !effect_value.is_finite() || !variance.is_finite() || variance <= 0.0 {
    return Err(failed(
      model,
      "discrete change delta variance is non-finite or nonpositive",
    ));
  }
  let effect = inference(model, predictor.to_owned(), effect_value, variance.sqrt())?;
  Ok(BinaryDiscreteChange {
    model,
    predictor: predictor.to_owned(),
    from_value: 0,
    to_value: 1,
    effect,
    robust: result.robust,
    cluster: result.cluster.clone(),
    observation_count: n,
  })
}

/// Compatibility name for the logit variant of the shared binary core. It
/// only admits logit results, so prior callers stay name-checked.
pub fn logit_average_marginal_effects(
  problem: &EstimationProblem,
  result: &BinaryMleResult,
) -> Result<LogitMarginalEffects, StatsError> {
  if result.model != BinaryModel::Logit {
    return Err(invalid(
      BinaryModel::Logit,
      "the logit compatibility alias requires a logit result",
    ));
  }
  binary_average_marginal_effects(problem, result)
}
