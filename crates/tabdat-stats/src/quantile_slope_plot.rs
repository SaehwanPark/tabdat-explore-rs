use crate::{
  EstimationProblem, EstimationSample, LeastSquaresResult, StatsError, bootstrap_quantiles,
  fit_least_squares,
};
use serde::{Deserialize, Serialize};

const BAND_CRITICAL_VALUE: f64 = 1.96;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileSlopeBandPoint {
  /// Requested conditional quantile.
  pub quantile: f64,
  /// Original-sample coefficient for the selected predictor.
  pub estimate: f64,
  /// Square root of that coefficient's diagonal in the joint bootstrap covariance.
  pub standard_error: f64,
  /// Fixed `estimate - 1.96 × standard_error` pointwise endpoint.
  pub lower: f64,
  /// Fixed `estimate + 1.96 × standard_error` pointwise endpoint.
  pub upper: f64,
}

/// Pointwise coefficient bands and provenance from one joint IID pairs bootstrap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantileSlopeBandResult {
  /// Outcome used to prepare the shared estimation sample.
  pub outcome: String,
  /// Predictor whose slope is reported at each quantile.
  pub coefficient: String,
  /// Quantiles in the requested, strictly increasing order.
  pub quantiles: Vec<f64>,
  /// Rows retained after joint finite-case selection.
  pub observation_count: usize,
  /// Number of common row-pairs bootstrap draws.
  pub replications: usize,
  /// Explicit unsigned seed passed to the common-draw bootstrap.
  pub seed: u64,
  /// Versioned identifier for the bootstrap random-number stream.
  pub rng: String,
  /// Nominal level corresponding to the fixed normal critical value.
  pub confidence_level: f64,
  /// Machine-readable reference distribution label.
  pub reference_distribution: String,
  /// Fixed critical value used for both endpoints.
  pub critical_value: f64,
  /// Machine-readable pointwise interval method label.
  pub band_method: String,
  /// Machine-readable common-draw covariance and denominator convention.
  pub covariance_convention: String,
  /// False because marginal pointwise intervals do not guarantee joint coverage.
  pub simultaneous_coverage: bool,
  /// OLS slope for the same retained sample and design.
  pub ols_coefficient: f64,
  /// One estimate, standard error and pair of pointwise endpoints per quantile.
  pub points: Vec<QuantileSlopeBandPoint>,
  /// Optional fit-time row positions and candidate/dropped counts.
  pub sample: Option<EstimationSample>,
}

fn invalid(message: impl Into<String>) -> StatsError {
  StatsError::InvalidEstimation(format!("quantile slope bands: {}", message.into()))
}

/// Joint IID row-bootstrap coefficient bands for one selected quantile slope.
/// The displayed endpoints use the original author's fixed `estimate ± 1.96 SE`
/// convention. They are pointwise normal-approximation intervals, not bands
/// with a simultaneous-coverage guarantee.
///
/// `coefficient` must name one predictor, and `quantiles` must contain at least
/// two strictly increasing values in `[0.05, 0.95]`. The input must be an
/// unweighted design; the underlying fit and bootstrap enforce rank, replication
/// and work limits. The problem is not mutated. Errors cover invalid requests,
/// failed OLS or quantile fits, bootstrap failures and nonrepresentable results.
pub fn quantile_slope_bands(
  problem: &EstimationProblem,
  quantiles: &[f64],
  coefficient: &str,
  replications: usize,
  seed: u64,
) -> Result<QuantileSlopeBandResult, StatsError> {
  if quantiles.len() < 2 {
    return Err(invalid(
      "at least two quantiles are required for a line plot",
    ));
  }
  let predictor_position = problem
    .predictor_names
    .iter()
    .position(|name| name == coefficient)
    .ok_or_else(|| invalid(format!("unknown coefficient: {coefficient}")))?;
  let parameter_position = predictor_position + usize::from(problem.include_intercept);

  let ols = fit_least_squares(problem)?;
  let ols_coefficient = coefficient_value(&ols, parameter_position, coefficient)?;
  let bootstrap = bootstrap_quantiles(problem, quantiles, replications, seed)?;
  let parameters_per_quantile = problem.parameter_count();
  let points = bootstrap
    .quantiles
    .iter()
    .enumerate()
    .map(|(quantile_index, &quantile)| {
      let index = quantile_index * parameters_per_quantile + parameter_position;
      let estimate = bootstrap.coefficients[index].estimate;
      let variance = bootstrap.covariance_matrix[index][index];
      if !variance.is_finite() || variance < 0.0 {
        return Err(invalid(format!(
          "bootstrap variance for quantile {quantile} is not finite and nonnegative"
        )));
      }
      let standard_error = variance.sqrt();
      let width = BAND_CRITICAL_VALUE * standard_error;
      let lower = estimate - width;
      let upper = estimate + width;
      if [estimate, standard_error, lower, upper]
        .iter()
        .any(|value| !value.is_finite())
      {
        return Err(invalid(format!(
          "band for quantile {quantile} is not representable"
        )));
      }
      Ok(QuantileSlopeBandPoint {
        quantile,
        estimate,
        standard_error,
        lower,
        upper,
      })
    })
    .collect::<Result<Vec<_>, _>>()?;

  Ok(QuantileSlopeBandResult {
    outcome: bootstrap.outcome,
    coefficient: coefficient.to_owned(),
    quantiles: bootstrap.quantiles,
    observation_count: bootstrap.observation_count,
    replications: bootstrap.replications,
    seed: bootstrap.seed,
    rng: bootstrap.rng,
    confidence_level: 95.0,
    reference_distribution: "asymptotic_standard_normal".into(),
    critical_value: BAND_CRITICAL_VALUE,
    band_method: "pointwise_normal_fixed_1.96".into(),
    covariance_convention: "common_iid_pairs_b_minus_1".into(),
    simultaneous_coverage: false,
    ols_coefficient,
    points,
    sample: bootstrap.sample,
  })
}

fn coefficient_value(
  result: &LeastSquaresResult,
  position: usize,
  name: &str,
) -> Result<f64, StatsError> {
  result
    .coefficients
    .get(position)
    .filter(|coefficient| coefficient.name == name)
    .map(|coefficient| coefficient.value)
    .filter(|value| value.is_finite())
    .ok_or_else(|| invalid(format!("OLS coefficient is unavailable: {name}")))
}
