#![forbid(unsafe_code)]

use crate::{
  CoefficientEstimate, CovarianceType, EstimationProblem, LeastSquaresOptions, LeastSquaresResult,
  StatsError, fit_fixed_effects, fit_least_squares, fit_least_squares_with_options, normal_cdf,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Variance-component construction retained with a one-way RE fit.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomEffectsComponentMethod {
  #[default]
  SwamyArora,
  EqualGroupBetween,
  GaussianMarginalMl,
}

/// One-way RE fit (Swamy–Arora/equal-group-between GLS or Gaussian marginal ML).
/// The stored regression is on the
/// quasi-demeaned response/design, including the transformed intercept column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomEffectsResult {
  pub entity_variable: String,
  pub outcome: String,
  pub predictor_names: Vec<String>,
  #[serde(default)]
  pub component_method: RandomEffectsComponentMethod,
  pub gls: LeastSquaresResult,
  pub group_count: usize,
  /// Common retained count, or zero for an unbalanced fit; use group_size_range().
  pub group_size: usize,
  pub sigma_e_squared: f64,
  pub sigma_u_squared: f64,
  pub raw_sigma_u_squared: f64,
  /// Common theta, or theta at the minimum retained group size for an unbalanced fit.
  pub theta: f64,
  pub rho: f64,
  pub confidence_intervals: Vec<[f64; 2]>,
  pub wald_statistic: Option<f64>,
  pub wald_p_value: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub wald_unavailable_reason: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ml: Option<crate::RandomEffectsMlInference>,
}

/// Conventional scalar FE–RE covariance-difference comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassicalPanelHausmanResult {
  pub outcome: String,
  pub predictor: String,
  pub entity_variable: String,
  pub observation_count: usize,
  pub group_count: usize,
  pub group_size: usize,
  pub fe_coefficient: f64,
  pub re_coefficient: f64,
  pub fe_variance: f64,
  pub re_variance: f64,
  pub difference: f64,
  pub variance_difference: f64,
  pub statistic: f64,
  pub df: usize,
  pub p_value: f64,
}

/// Joint slope-only comparison after model-based equal-group-between RE-GLS.
/// Separate native FE/RE residual scales are retained; no rescaling is implied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointPanelHausmanResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub entity_variable: String,
  pub observation_count: usize,
  pub candidate_observation_count: usize,
  pub dropped_observation_count: usize,
  pub group_count: usize,
  pub minimum_group_size: usize,
  pub maximum_group_size: usize,
  pub singleton_groups: usize,
  pub re_component_method: RandomEffectsComponentMethod,
  pub fe_coefficients: Vec<f64>,
  pub re_coefficients: Vec<f64>,
  pub differences: Vec<f64>,
  pub fe_covariance: Vec<Vec<f64>>,
  pub re_covariance: Vec<Vec<f64>>,
  pub covariance_difference: Vec<Vec<f64>>,
  pub fe_residual_df: usize,
  pub re_residual_df: usize,
  pub fe_residual_scale: f64,
  pub re_residual_scale: f64,
  pub minimum_scaled_cholesky_pivot: f64,
  pub statistic: f64,
  pub df: usize,
  pub p_value: f64,
  pub covariance_scale_convention: String,
  pub reference_distribution: String,
  pub assumptions: String,
}

fn scalar_hausman(bf: f64, br: f64, vf: f64, vr: f64) -> Result<(f64, f64, f64, f64), StatsError> {
  let variance_difference = vf - vr;
  let scale = vf.abs().max(vr.abs());
  if !vf.is_finite()
    || !vr.is_finite()
    || vf <= 0.0
    || vr <= 0.0
    || !scale.is_finite()
    || !variance_difference.is_finite()
    || variance_difference <= 128.0 * f64::EPSILON * scale
  {
    return Err(StatsError::InvalidEstimation(
      "FE minus RE covariance is nonpositive or numerically indistinguishable from zero".into(),
    ));
  }
  let difference = bf - br;
  let statistic = difference * difference / variance_difference;
  let p_value = crate::distributions::chi_square_tail(statistic, 1);
  if !difference.is_finite() || !statistic.is_finite() || !p_value.is_finite() {
    return Err(StatsError::InvalidEstimation(
      "classical panel Hausman statistic or probability is not finite".into(),
    ));
  }
  Ok((difference, variance_difference, statistic, p_value))
}

/// Refit scalar FE and balanced model-based Swamy–Arora RE on the same problem.
/// The asymptotic covariance-difference reference requires RE efficiency under H0;
/// it is not robust to residual within-unit dependence.
pub fn classical_panel_hausman(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<ClassicalPanelHausmanResult, StatsError> {
  if problem.predictor_names.len() != 1 || !problem.include_intercept {
    return Err(StatsError::InvalidEstimation(
      "classical panel Hausman requires one slope and an intercept".into(),
    ));
  }
  let re = fit_random_effects(problem, entity_variable)?;
  let fe = fit_fixed_effects(problem, entity_variable, false)?;
  let bf = fe.within.coefficients[0].value;
  let br = re.gls.coefficients[1].value;
  let vf = fe.within.covariance.matrix[0][0];
  let vr = re.gls.covariance.matrix[1][1];
  let (difference, variance_difference, statistic, p_value) = scalar_hausman(bf, br, vf, vr)?;
  Ok(ClassicalPanelHausmanResult {
    outcome: problem.outcome.clone(),
    predictor: problem.predictor_names[0].clone(),
    entity_variable: entity_variable.to_owned(),
    observation_count: problem.response.len(),
    group_count: re.group_count,
    group_size: re.group_size,
    fe_coefficient: bf,
    re_coefficient: br,
    fe_variance: vf,
    re_variance: vr,
    difference,
    variance_difference,
    statistic,
    df: 1,
    p_value,
  })
}

fn joint_hausman_error(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("joint panel Hausman: {message}"))
}

// Scaling by the parent variances makes the cancellation/curvature guard
// invariant to predictor units. Positive diagonal differences alone are not SPD.
fn joint_hausman_quadratic(
  difference: &[f64],
  fe: &[Vec<f64>],
  re: &[Vec<f64>],
) -> Result<(Vec<Vec<f64>>, f64, f64), StatsError> {
  let k = difference.len();
  if k == 0
    || fe.len() != k
    || re.len() != k
    || fe.iter().chain(re).any(|row| row.len() != k)
    || difference
      .iter()
      .chain(fe.iter().flatten())
      .chain(re.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(joint_hausman_error(
      "nonfinite or misaligned coefficient/covariance inputs",
    ));
  }
  let tolerance = 128.0 * f64::EPSILON;
  let scales: Vec<_> = (0..k).map(|i| fe[i][i].max(re[i][i]).sqrt()).collect();
  if (0..k)
    .any(|i| fe[i][i] <= 0.0 || re[i][i] <= 0.0 || !scales[i].is_finite() || scales[i] <= 0.0)
  {
    return Err(joint_hausman_error(
      "parent variances must be finite and positive",
    ));
  }
  let mut covariance = vec![vec![0.0; k]; k];
  let mut normalized = vec![vec![0.0; k]; k];
  for i in 0..k {
    for j in 0..=i {
      let f_asymmetry = (fe[i][j] - fe[j][i]).abs() / scales[i] / scales[j];
      let r_asymmetry = (re[i][j] - re[j][i]).abs() / scales[i] / scales[j];
      if !f_asymmetry.is_finite()
        || !r_asymmetry.is_finite()
        || f_asymmetry > tolerance
        || r_asymmetry > tolerance
      {
        return Err(joint_hausman_error(
          "parent covariance is not numerically symmetric",
        ));
      }
      // Average only roundoff-level asymmetry; do not repair eigenvalues.
      let value = 0.5 * (fe[i][j] - re[i][j]) + 0.5 * (fe[j][i] - re[j][i]);
      let scaled = value / scales[i] / scales[j];
      if !value.is_finite() || !scaled.is_finite() {
        return Err(joint_hausman_error("nonfinite covariance difference"));
      }
      covariance[i][j] = value;
      covariance[j][i] = value;
      normalized[i][j] = scaled;
      normalized[j][i] = scaled;
    }
  }
  let mut lower = vec![vec![0.0; k]; k];
  let mut minimum_pivot = f64::INFINITY;
  for i in 0..k {
    for j in 0..=i {
      let prior: f64 = (0..j).map(|h| lower[i][h] * lower[j][h]).sum();
      let value = normalized[i][j] - prior;
      if i == j {
        if !value.is_finite() || value <= tolerance {
          return Err(joint_hausman_error(
            "FE minus RE covariance is nonpositive definite, rank deficient or cancellation dominated",
          ));
        }
        minimum_pivot = minimum_pivot.min(value);
        lower[i][j] = value.sqrt();
      } else {
        lower[i][j] = value / lower[j][j];
      }
    }
  }
  let mut whitened = vec![0.0; k];
  for i in 0..k {
    let prior: f64 = (0..i).map(|j| lower[i][j] * whitened[j]).sum();
    whitened[i] = (difference[i] / scales[i] - prior) / lower[i][i];
  }
  let statistic: f64 = whitened.iter().map(|v| v * v).sum();
  if !statistic.is_finite() {
    return Err(joint_hausman_error("nonfinite statistic"));
  }
  Ok((covariance, statistic, minimum_pivot))
}

/// Compare all slopes with FE on the retained equal-group-between GLS design.
/// Refit/verify the complete RE result before inference, including its sample,
/// variance components and transformed covariance. Neither current data nor a
/// supplied snapshot's coefficients may silently define a different comparison.
pub fn joint_equal_group_panel_hausman(
  problem: &EstimationProblem,
  retained_re: &RandomEffectsResult,
) -> Result<JointPanelHausmanResult, StatsError> {
  let k = problem.predictor_names.len();
  let n = problem.response.len();
  if !(1..=16).contains(&k)
    || !problem.include_intercept
    || n > 40000
    || n.checked_mul(k * k).is_none_or(|work| work > 20_000_000)
    || retained_re.component_method != RandomEffectsComponentMethod::EqualGroupBetween
    || retained_re.ml.is_some()
    || retained_re.gls.covariance.covariance_type != CovarianceType::NonRobust
  {
    return Err(joint_hausman_error(
      "requires model-based equal-group-between RE-GLS, an intercept, 1..16 slopes, N<=40000 and N*K^2<=20million",
    ));
  }
  let re = fit_random_effects_equal_group_between(problem, &retained_re.entity_variable)?;
  if re != *retained_re {
    return Err(joint_hausman_error(
      "retained RE fit does not match the refitted original design",
    ));
  }
  let fe = fit_fixed_effects(problem, &re.entity_variable, false)?;
  let fe_coefficients: Vec<_> = fe.within.coefficients.iter().map(|c| c.value).collect();
  let re_coefficients: Vec<_> = re.gls.coefficients[1..].iter().map(|c| c.value).collect();
  let differences: Vec<_> = fe_coefficients
    .iter()
    .zip(&re_coefficients)
    .map(|(f, r)| f - r)
    .collect();
  let fe_covariance = fe.within.covariance.matrix.clone();
  let re_covariance: Vec<_> = re.gls.covariance.matrix[1..]
    .iter()
    .map(|row| row[1..].to_vec())
    .collect();
  let (covariance_difference, statistic, minimum_scaled_cholesky_pivot) =
    joint_hausman_quadratic(&differences, &fe_covariance, &re_covariance)?;
  let p_value = crate::distributions::chi_square_tail(statistic, k);
  if !p_value.is_finite() {
    return Err(joint_hausman_error("nonfinite probability"));
  }
  let sample = problem
    .sample
    .as_ref()
    .ok_or_else(|| joint_hausman_error("sample unavailable"))?;
  let mut sizes = BTreeMap::new();
  for group in sample
    .cluster_groups
    .as_ref()
    .ok_or_else(|| joint_hausman_error("groups unavailable"))?
  {
    *sizes.entry(group).or_insert(0_usize) += 1;
  }
  Ok(JointPanelHausmanResult {
    outcome: problem.outcome.clone(), predictor_names: problem.predictor_names.clone(),
    entity_variable: re.entity_variable.clone(), observation_count: n,
    candidate_observation_count: sample.total_observations, dropped_observation_count: sample.dropped_observations,
    group_count: sizes.len(), minimum_group_size: *sizes.values().min().unwrap(),
    maximum_group_size: *sizes.values().max().unwrap(), singleton_groups: sizes.values().filter(|&&n|n==1).count(),
    re_component_method: re.component_method, fe_coefficients,re_coefficients,differences,
    fe_covariance,re_covariance,covariance_difference,
    fe_residual_df: fe.within.fit_statistics.degrees_of_freedom,
    re_residual_df: re.gls.fit_statistics.degrees_of_freedom,
    fe_residual_scale: fe.within.fit_statistics.residual_sum_of_squares.unwrap()/fe.within.fit_statistics.degrees_of_freedom as f64,
    re_residual_scale: re.gls.fit_statistics.residual_sum_of_squares.unwrap()/re.gls.fit_statistics.degrees_of_freedom as f64,
    minimum_scaled_cholesky_pivot,statistic,df:k,p_value,
    covariance_scale_convention: "separate_native_fe_and_re_residual_scales".into(),
    reference_distribution: "chi_square".into(),
    assumptions: "Classical model-based FE/RE comparison requires RE efficiency under H0 and IID homoskedastic idiosyncratic errors; not cluster-robust, survey or causal inference. Non-rejection does not establish exogeneity.".into(),
  })
}

impl RandomEffectsResult {
  /// Retained-model counts, independent of any subsequent active-data changes.
  pub fn group_size_range(&self) -> Option<(usize, usize)> {
    if self.group_size > 0 {
      return Some((self.group_size, self.group_size));
    }
    let mut counts = BTreeMap::new();
    for key in self.gls.sample.as_ref()?.cluster_groups.as_ref()? {
      *counts.entry(key.as_str()).or_insert(0_usize) += 1;
    }
    Some((*counts.values().min()?, *counts.values().max()?))
  }

  pub fn theta_range(&self) -> Option<(f64, f64)> {
    if self.group_size > 0 {
      return Some((self.theta, self.theta));
    }
    let (lo, hi) = self.group_size_range()?;
    let theta = |size: usize| {
      1.0
        - (self.sigma_e_squared / (self.sigma_e_squared + size as f64 * self.sigma_u_squared))
          .sqrt()
    };
    Some((theta(lo), theta(hi)))
  }

  /// Selected joint-covariance uncertainty for fitted ML working variance components.
  /// Swamy–Arora GLS results have no likelihood-based component inference.
  pub fn variance_component_inference(
    &self,
  ) -> Result<Option<crate::RandomEffectsMlVarianceComponentInference>, StatsError> {
    self
      .ml
      .as_ref()
      .map(|ml| {
        ml.variance_component_inference(self.sigma_e_squared, self.sigma_u_squared, self.group_size)
      })
      .transpose()
  }
}

/// Joint finite cases must already be selected and sample-aligned unit keys
/// supplied. Equal retained group sizes T>=2 are required. The within and between
/// preliminary regressions must both identify all slopes. No weights are installed.
/// Coefficients use asymptotic normal inference.
pub fn fit_random_effects(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(
    problem,
    entity_variable,
    false,
    false,
    RandomEffectsComponentMethod::SwamyArora,
  )
}

/// Unit-score sandwich of the quasi-demeaned design/residuals, with correction
/// G/(G-1)*(N-1)/(N-K). Components and coefficients are unchanged. Inference is
/// asymptotic normal/chi-square, not G-1 t/F. Singular slope Wald tests are
/// explicitly unavailable while positive marginal coefficient SEs are retained.
pub fn fit_random_effects_clustered(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(
    problem,
    entity_variable,
    true,
    false,
    RandomEffectsComponentMethod::SwamyArora,
  )
}

/// Swamy–Arora GLS with unbiased unbalanced trace correction and per-unit theta.
/// The original balanced-only entry points retain their strict sample contract.
pub fn fit_random_effects_unbalanced(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(
    problem,
    entity_variable,
    false,
    true,
    RandomEffectsComponentMethod::SwamyArora,
  )
}

/// Unit-score covariance for the same unbalanced variance-component estimator.
pub fn fit_random_effects_unbalanced_clustered(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(
    problem,
    entity_variable,
    true,
    true,
    RandomEffectsComponentMethod::SwamyArora,
  )
}

/// Equal-group between MSE minus within variance times mean inverse group size,
/// truncated at zero. Unlike unbalanced Swamy–Arora, every group has equal
/// weight in the preliminary between fit. The same per-size quasi-GLS and
/// model-based normal inference follow; this is not a general Stata mode.
pub fn fit_random_effects_equal_group_between(
  problem: &EstimationProblem,
  entity_variable: &str,
) -> Result<RandomEffectsResult, StatsError> {
  fit_random_effects_impl(
    problem,
    entity_variable,
    false,
    true,
    RandomEffectsComponentMethod::EqualGroupBetween,
  )
}

fn fit_random_effects_impl(
  problem: &EstimationProblem,
  entity_variable: &str,
  clustered: bool,
  allow_unbalanced: bool,
  component_method: RandomEffectsComponentMethod,
) -> Result<RandomEffectsResult, StatsError> {
  let within = fit_fixed_effects(problem, entity_variable, false)?;
  let g = within.effects.len();
  let p = problem.predictor_names.len();
  let t = within.minimum_group_size;
  let balanced = within.maximum_group_size == t;
  if !allow_unbalanced && (t < 2 || !balanced) {
    return Err(StatsError::InvalidEstimation(
      "RE-GLS requires equal retained unit counts T >= 2; unbalanced variance estimation is not implemented".into(),
    ));
  }
  if g <= p + 1 {
    return Err(StatsError::ZeroDegreesOfFreedom {
      observations: g,
      parameters: p + 1,
    });
  }
  if problem
    .predictor_names
    .iter()
    .any(|name| name == &problem.intercept_name)
  {
    return Err(StatsError::InvalidParameterName(
      "RE predictor conflicts with intercept name".into(),
    ));
  }
  let mut between_problem = EstimationProblem::new(
    &problem.outcome,
    problem.predictor_names.clone(),
    within.effects.iter().map(|e| e.response_mean).collect(),
    within
      .effects
      .iter()
      .map(|e| e.predictor_means.clone())
      .collect(),
  )?
  .with_intercept_name(&problem.intercept_name);
  if !balanced && component_method == RandomEffectsComponentMethod::SwamyArora {
    between_problem = between_problem.with_weights(
      within
        .effects
        .iter()
        .map(|e| e.observation_count as f64)
        .collect(),
    );
  }
  let between = fit_least_squares(&between_problem)?;
  let sigma_e_squared = within
    .within
    .fit_statistics
    .residual_sum_of_squares
    .unwrap()
    / within.within.fit_statistics.degrees_of_freedom as f64;
  let raw_sigma_u_squared = if component_method == RandomEffectsComponentMethod::EqualGroupBetween {
    let mean_inverse_size = within
      .effects
      .iter()
      .map(|e| 1.0 / e.observation_count as f64)
      .sum::<f64>()
      / g as f64;
    between.fit_statistics.residual_sum_of_squares.unwrap() / (g - p - 1) as f64
      - sigma_e_squared * mean_inverse_size
  } else if balanced {
    between.fit_statistics.residual_sum_of_squares.unwrap() / (g - p - 1) as f64
      - sigma_e_squared / t as f64
  } else {
    // Weighted between projection: A=Z' diag(Ti) Z, C=Z' diag(Ti²) Z.
    // Its residual quadratic has expectation (G-K)*sigma_e² +
    // (N-trace(inv(A)*C))*sigma_u²; a harmonic-T substitution is insufficient.
    let k = p + 1;
    let weighted: Vec<Vec<f64>> = within
      .effects
      .iter()
      .map(|e| {
        let root = (e.observation_count as f64).sqrt();
        std::iter::once(root)
          .chain(e.predictor_means.iter().map(|x| x * root))
          .collect()
      })
      .collect();
    let qr = crate::matrix::qr_decompose_with_response(&weighted, &vec![0.0; g])?;
    let inverse = crate::matrix::xtx_inverse_from_r(&qr.r)?;
    let mut trace = 0.0;
    let mut quadratic = 0.0;
    for (e, residual) in within.effects.iter().zip(&between.residuals) {
      let size = e.observation_count as f64;
      let z: Vec<_> = std::iter::once(1.0)
        .chain(e.predictor_means.iter().copied())
        .collect();
      for i in 0..k {
        for j in 0..k {
          trace += inverse[i][j] * z[i] * z[j] * size * size;
        }
      }
      quadratic += size * residual * residual;
    }
    let denominator = problem.observation_count() as f64 - trace;
    if !denominator.is_finite() || denominator <= 0.0 {
      return Err(StatsError::InvalidEstimation(
        "unbalanced RE component trace has no positive degrees of freedom".into(),
      ));
    }
    (quadratic - (g - p - 1) as f64 * sigma_e_squared) / denominator
  };
  let sigma_u_squared = raw_sigma_u_squared.max(0.0);
  let variance = sigma_e_squared + t as f64 * sigma_u_squared;
  if !sigma_e_squared.is_finite()
    || sigma_e_squared <= 0.0
    || !raw_sigma_u_squared.is_finite()
    || !variance.is_finite()
  {
    return Err(StatsError::InvalidEstimation(
      "RE variance components require finite positive idiosyncratic variance".into(),
    ));
  }
  let theta = 1.0 - (sigma_e_squared / variance).sqrt();
  if theta >= 1.0 {
    return Err(StatsError::InvalidEstimation(
      "RE transformation loses the intercept at machine precision".into(),
    ));
  }
  let centers: BTreeMap<_, _> = within.effects.iter().map(|e| (e.key.as_str(), e)).collect();
  let keys = problem
    .sample
    .as_ref()
    .unwrap()
    .cluster_groups
    .as_ref()
    .unwrap();
  let mut transformed = problem.clone();
  transformed.include_intercept = false;
  transformed
    .predictor_names
    .insert(0, problem.intercept_name.clone());
  for (i, key) in keys.iter().enumerate() {
    let center = centers[key.as_str()];
    let theta = if balanced {
      theta
    } else {
      1.0
        - (sigma_e_squared / (sigma_e_squared + center.observation_count as f64 * sigma_u_squared))
          .sqrt()
    };
    if !theta.is_finite() || theta >= 1.0 {
      return Err(StatsError::InvalidEstimation(
        "RE transformation loses the intercept at machine precision".into(),
      ));
    }
    transformed.response[i] -= theta * center.response_mean;
    for j in 0..p {
      transformed.design_matrix[i][j] -= theta * center.predictor_means[j];
    }
    transformed.design_matrix[i].insert(0, 1.0 - theta);
  }
  let covariance_type = if clustered {
    CovarianceType::Cluster(entity_variable.into())
  } else {
    CovarianceType::NonRobust
  };
  let mut gls =
    fit_least_squares_with_options(&transformed, &LeastSquaresOptions { covariance_type })?;
  if !gls
    .fit_statistics
    .residual_sum_of_squares
    .is_some_and(|v| v.is_finite() && v > 0.0)
    || gls
      .covariance
      .matrix
      .iter()
      .flatten()
      .any(|v| !v.is_finite())
  {
    return Err(StatsError::InvalidEstimation(
      "RE transformed covariance or residual scale is degenerate".into(),
    ));
  }
  let se = gls.covariance.standard_errors();
  if se.iter().any(|se| !se.is_finite() || *se <= 0.0) {
    return Err(StatsError::InvalidEstimation(
      "RE covariance requires finite positive coefficient variances".into(),
    ));
  }
  gls.coefficients = gls
    .coefficients
    .iter()
    .zip(se)
    .map(|(b, se)| {
      let p_value = (se > 0.0).then(|| 2.0 * normal_cdf(-(b.value / se).abs()));
      CoefficientEstimate::new(&b.name, b.value, Some(se), p_value)
    })
    .collect();
  let critical = crate::normal_quantile(0.975);
  let confidence_intervals: Vec<[f64; 2]> = gls
    .coefficients
    .iter()
    .map(|b| {
      let se = b.standard_error.unwrap();
      [b.value - critical * se, b.value + critical * se]
    })
    .collect();
  if confidence_intervals
    .iter()
    .flatten()
    .any(|v| !v.is_finite())
  {
    return Err(StatsError::NonFiniteValue(
      "RE coefficient intervals overflowed".into(),
    ));
  }
  let values: Vec<_> = gls.coefficients.iter().map(|c| c.value).collect();
  let wald = crate::binary::omnibus_wald(
    &values,
    &gls.covariance.matrix,
    true,
    clustered.then_some(g),
  );
  if !clustered && wald.unavailable_reason.is_some() {
    return Err(StatsError::InvalidEstimation(
      "RE slope Wald calculation is singular or nonfinite".into(),
    ));
  }
  // Uncentered no-intercept OLS summaries would describe the transformed
  // implementation, not standard panel within/between/overall R² or likelihood.
  gls.fit_statistics.r_squared = None;
  gls.fit_statistics.adjusted_r_squared = None;
  gls.fit_statistics.log_likelihood = None;
  gls.fit_statistics.f_statistic = None;
  gls.diagnostics.method = if component_method == RandomEffectsComponentMethod::EqualGroupBetween {
    "equal_group_between_random_effects_gls"
  } else if balanced {
    "balanced_swamy_arora_random_effects_gls"
  } else {
    "unbalanced_swamy_arora_random_effects_gls"
  }
  .into();
  Ok(RandomEffectsResult {
    ml: None,
    entity_variable: entity_variable.into(),
    outcome: problem.outcome.clone(),
    predictor_names: problem.predictor_names.clone(),
    component_method,
    gls,
    group_count: g,
    group_size: if balanced { t } else { 0 },
    sigma_e_squared,
    sigma_u_squared,
    raw_sigma_u_squared,
    theta,
    rho: sigma_u_squared / (sigma_e_squared + sigma_u_squared),
    confidence_intervals,
    wald_statistic: wald.statistic,
    wald_p_value: wald.p_value,
    wald_unavailable_reason: wald.unavailable_reason,
  })
}

#[cfg(test)]
mod hausman_tests {
  use super::{joint_hausman_quadratic, scalar_hausman};

  #[test]
  fn joint_quadratic_matches_closed_form_and_scaled_permutations() {
    let fe = vec![vec![3.0, 0.4], vec![0.4, 2.0]];
    let re = vec![vec![1.0, 0.2], vec![0.2, 1.0]];
    let delta = [1.0, 2.0];
    let (difference, statistic, pivot) = joint_hausman_quadratic(&delta, &fe, &re).unwrap();
    assert_eq!(difference, vec![vec![2.0, 0.2], vec![0.2, 1.0]]);
    assert!((statistic - 8.2 / 1.96).abs() < 1e-14);
    assert!(pivot > 128.0 * f64::EPSILON);
    for order in [[0, 1], [1, 0]] {
      for scale in [[1e-8, -1e8], [1e8, 1e-8]] {
        let d: Vec<_> = order
          .iter()
          .enumerate()
          .map(|(i, &j)| delta[j] * scale[i])
          .collect();
        let transform = |matrix: &[Vec<f64>]| {
          order
            .iter()
            .enumerate()
            .map(|(i, &a)| {
              order
                .iter()
                .enumerate()
                .map(|(j, &b)| matrix[a][b] * scale[i] * scale[j])
                .collect()
            })
            .collect::<Vec<Vec<f64>>>()
        };
        let (_, actual, _) = joint_hausman_quadratic(&d, &transform(&fe), &transform(&re)).unwrap();
        assert!((actual - statistic).abs() < 1e-13);
      }
    }
  }

  #[test]
  fn joint_quadratic_rejects_indefinite_singular_cancellation_and_invalid_inputs() {
    let identity = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    // Each parent is SPD and the difference has positive diagonal entries.
    for fe in [
      vec![vec![2.0, 1.2], vec![1.2, 2.0]],
      vec![vec![2.0, 1.0], vec![1.0, 2.0]],
      vec![vec![1.0 + 64.0 * f64::EPSILON, 0.0], vec![0.0, 2.0]],
      vec![vec![2.0, 0.1], vec![0.2, 2.0]],
      vec![vec![f64::NAN, 0.0], vec![0.0, 2.0]],
    ] {
      assert!(joint_hausman_quadratic(&[1.0, 2.0], &fe, &identity).is_err());
    }
    assert!(joint_hausman_quadratic(&[f64::INFINITY, 0.0], &identity, &identity).is_err());
    assert!(joint_hausman_quadratic(&[], &[], &[]).is_err());
    assert!(joint_hausman_quadratic(&[1.0], &identity, &identity).is_err());
    assert!(joint_hausman_quadratic(&[f64::MAX], &[vec![1.0]], &[vec![0.5]]).is_err());
    let (_, zero, _) = joint_hausman_quadratic(&[0.0], &[vec![2.0]], &[vec![1.0]]).unwrap();
    assert_eq!(zero, 0.0);
  }

  #[test]
  fn rejects_nonpositive_and_cancellation_dominated_covariance_difference() {
    assert!(scalar_hausman(1.0, 0.0, 1.0, 2.0).is_err());
    assert!(scalar_hausman(1.0, 0.0, 1.0, 1.0).is_err());
    assert!(scalar_hausman(1.0, 0.0, 1.0, 1.0 - 64.0 * f64::EPSILON).is_err());
    assert!(scalar_hausman(1.0, 0.0, -1.0, -2.0).is_err());
    assert!(scalar_hausman(f64::MAX, -f64::MAX, 2.0, 1.0).is_err());
    assert!(scalar_hausman(1.0, 1.0, 2.0, 1.0).is_ok());
  }
}
