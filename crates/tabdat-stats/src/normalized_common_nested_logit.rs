//! Preferred common-utility normalization with one shared positive nest scale.
//! In b_tilde=beta/lambda coordinates this is exactly the earlier likelihood
//! with equal scales. Reuse its derivatives, but fit the projected system and
//! transform the complete covariance back to natural utilities and scale.

use crate::conditional_logit::alternative_order;
use crate::distributions::chi_square_tail;
use crate::matrix::choice_information_inverse;
use crate::nested_logit::{
  Evaluation, NestedLayout, evaluate, inference, optimize_with, terms, upper_constants,
  validate_nested_design,
};
use crate::{
  EstimationSample, LogitEstimate, NestedChoiceProblem, NestedLogitOptions, StatsError,
  fit_conditional_logit,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedCommonNestedLogitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub lower_predictor_count: usize,
  pub group_variable: String,
  pub alternative_variable: String,
  pub nest_variable: String,
  pub alternative_levels: Vec<i64>,
  pub nest_levels: Vec<i64>,
  pub alternative_nests: Vec<i64>,
  pub chosen_counts: Vec<usize>,
  pub observation_count: usize,
  pub choice_set_count: usize,
  pub choice_set_size: usize,
  pub candidate_choice_set_count: usize,
  pub excluded_choice_set_count: usize,
  pub coefficients: Vec<LogitEstimate>,
  /// One common natural scale, not two separately estimated nest parameters.
  pub dissimilarity: LogitEstimate,
  pub scale_test_null: f64,
  /// Lower/upper coefficients, then iv_common; includes all cross terms.
  pub covariance_parameter_names: Vec<String>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub restricted_log_likelihood: f64,
  pub independence_likelihood_ratio: f64,
  pub independence_df: usize,
  pub independence_p_value: f64,
  /// Descriptive (0,1] check, not a test of the composite consistency null.
  pub random_utility_iv_bounds_satisfied: bool,
  pub iterations: usize,
  pub converged_starts: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_probabilities: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("nlogit normalized common: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("nlogit normalized common: {message}"))
}
// Shared private routines retain their existing error context for old callers.
// Their equal-scale use here must not mislabel the fitted normalization.
fn context(error: StatsError) -> StatsError {
  match error {
    StatsError::InvalidEstimation(message) => invalid(
      message
        .strip_prefix("nlogit nonnormalized: ")
        .unwrap_or(&message),
    ),
    StatsError::EstimationFailed(message) => failed(
      message
        .strip_prefix("nlogit nonnormalized: ")
        .unwrap_or(&message),
    ),
    other => other,
  }
}

fn project_matrix(matrix: &[Vec<f64>], k: usize) -> Vec<Vec<f64>> {
  (0..=k)
    .map(|r| {
      (0..=k)
        .map(|s| match (r == k, s == k) {
          (false, false) => matrix[r][s],
          (true, false) => matrix[k][s] + matrix[k + 1][s],
          (false, true) => matrix[r][k] + matrix[r][k + 1],
          (true, true) => matrix[k][k] + matrix[k][k + 1] + matrix[k + 1][k] + matrix[k + 1][k + 1],
        })
        .collect()
    })
    .collect()
}
fn evaluate_common(
  x: &[Vec<f64>],
  layout: &NestedLayout,
  lower: usize,
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let k = theta.len() - 1;
  let expanded = theta[..k]
    .iter()
    .copied()
    .chain([theta[k]; 2])
    .collect::<Vec<_>>();
  let mut value = evaluate(x, layout, lower, &expanded).map_err(context)?;
  let last_scale_score = value.score.pop().unwrap();
  value.score[k] += last_scale_score;
  value.observed = project_matrix(&value.observed, k);
  value.fisher = project_matrix(&value.fisher, k);
  Ok(value)
}

fn centered_design(
  problem: &NestedChoiceProblem,
  layout: &NestedLayout,
) -> Result<Vec<Vec<f64>>, StatsError> {
  let mut x = problem.choice.design.design_matrix.clone();
  // Equal scales make lower case-common shifts cancel. Upper case-common
  // shifts also cancel; validate nest constancy on the raw design first.
  for case in &layout.choice.cases {
    let baseline = x[case[0]].clone();
    for &i in case {
      for (v, base) in x[i].iter_mut().zip(&baseline) {
        *v -= base;
        if !v.is_finite() {
          return Err(failed("nonfinite case-centered predictor"));
        }
      }
    }
  }
  Ok(x)
}

/// ML with one shared positive scale under the preferred utility normalization.
/// The scale is not constrained above by one: fitted RUM-bound violations are
/// explicit diagnostics. Joint observed covariance is transformed in full;
/// independence uses a locally identified lambda=1 restriction, chi-square1.
pub fn fit_normalized_common_nested_logit(
  problem: &NestedChoiceProblem,
  options: &NestedLogitOptions,
) -> Result<NormalizedCommonNestedLogitResult, StatsError> {
  let layout = validate_nested_design(problem, true).map_err(context)?;
  let d = &problem.choice.design;
  let k = d.predictor_names.len();
  let n = d.response.len();
  let g = layout.choice.cases.len();
  let lower = problem.lower_predictor_count;
  let restricted = fit_conditional_logit(&problem.choice, &Default::default())?;
  let mut x = centered_design(problem, &layout)?;
  let mut scales = Vec::with_capacity(k);
  for r in 0..k {
    let max = x.iter().map(|row| row[r].abs()).fold(0.0, f64::max);
    if max == 0.0 {
      return Err(invalid("a predictor has no within-case variation"));
    }
    let scale = max * (x.iter().map(|row| (row[r] / max).powi(2)).sum::<f64>() / n as f64).sqrt();
    if !scale.is_finite() || scale <= 0.0 {
      return Err(failed("undefined centered predictor scale"));
    }
    scales.push(scale);
    for row in &mut x {
      row[r] /= scale;
    }
  }
  let restriction_theta = restricted
    .coefficients
    .iter()
    .zip(&scales)
    .map(|(b, s)| b.value * s)
    .chain([0.0])
    .collect::<Vec<_>>();
  let restriction = evaluate_common(&x, &layout, lower, &restriction_theta)?;
  choice_information_inverse(&restriction.fisher).map_err(|_| invalid(
    "common scale is not locally identified at the independence restriction; a regular LR reference is unavailable",
  ))?;
  let mut best = None;
  let mut converged_starts = 0;
  let mut last_error = String::new();
  for lambda in [1.0_f64, 2.0] {
    let theta = restricted
      .coefficients
      .iter()
      .zip(&scales)
      .enumerate()
      .map(|(r, (b, s))| b.value * s / if r < lower { lambda } else { 1.0 })
      .chain([lambda.ln()])
      .collect::<Vec<_>>();
    match optimize_with(theta, options, |theta| {
      evaluate_common(&x, &layout, lower, theta)
    }) {
      Ok(solution) => {
        converged_starts += 1;
        if best
          .as_ref()
          .is_none_or(|b: &crate::nested_logit::Solution| {
            solution.evaluation.log_likelihood > b.evaluation.log_likelihood
          })
        {
          best = Some(solution);
        }
      }
      Err(error) => last_error = context(error).to_string(),
    }
  }
  let solution = best.ok_or_else(|| failed(&last_error))?;
  let ll = solution.evaluation.log_likelihood;
  if ll < restricted.log_likelihood - 1e-12 * (1.0 + restricted.log_likelihood.abs()) {
    return Err(failed(
      "fitted likelihood is below its conditional restriction",
    ));
  }
  let lambda = solution.theta[k].exp();
  let values = (0..k)
    .map(|r| solution.theta[r] / scales[r] * if r < lower { lambda } else { 1.0 })
    .chain([lambda])
    .collect::<Vec<_>>();
  let mut jacobian = vec![vec![0.0; k + 1]; k + 1];
  for r in 0..k {
    jacobian[r][r] = if r < lower { lambda } else { 1.0 } / scales[r];
    if r < lower {
      jacobian[r][k] = values[r];
    }
  }
  jacobian[k][k] = lambda;
  // beta=lambda*b_tilde: diagonal rescaling alone would omit the scale and
  // coefficient/scale contributions, even though upper gamma is unchanged.
  let temp = jacobian
    .iter()
    .map(|row| {
      (0..=k)
        .map(|s| {
          row
            .iter()
            .enumerate()
            .map(|(j, v)| v * solution.inverse[j][s])
            .sum::<f64>()
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let covariance = temp
    .iter()
    .map(|row| {
      jacobian
        .iter()
        .map(|other| row.iter().zip(other).map(|(a, b)| a * b).sum::<f64>() / g as f64)
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  if covariance.iter().flatten().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite joint covariance"));
  }
  let coefficients = (0..k)
    .map(|r| {
      inference(
        d.predictor_names[r].clone(),
        values[r],
        covariance[r][r],
        0.0,
      )
      .map_err(context)
    })
    .collect::<Result<Vec<_>, _>>()?;
  let dissimilarity =
    inference("iv_common".into(), lambda, covariance[k][k], 1.0).map_err(context)?;
  let mut chosen_counts = vec![0; layout.choice.levels.len()];
  for &i in &layout.choice.chosen {
    chosen_counts[layout
      .choice
      .levels
      .binary_search(&problem.choice.alternative_codes[i])
      .unwrap()] += 1;
  }
  let lr = (2.0 * (ll - restricted.log_likelihood)).max(0.0);
  Ok(NormalizedCommonNestedLogitResult {
    outcome: d.outcome.clone(),
    predictor_names: d.predictor_names.clone(),
    lower_predictor_count: lower,
    group_variable: problem.choice.group_variable.clone(),
    alternative_variable: problem.choice.alternative_variable.clone(),
    nest_variable: problem.nest_variable.clone(),
    alternative_levels: layout.choice.levels,
    nest_levels: layout.nest_levels,
    alternative_nests: layout.alternative_nests,
    chosen_counts,
    observation_count: n,
    choice_set_count: g,
    choice_set_size: layout.choice.cases[0].len(),
    candidate_choice_set_count: layout.choice.candidate_cases,
    excluded_choice_set_count: layout.choice.candidate_cases - g,
    coefficients,
    dissimilarity,
    scale_test_null: 1.0,
    covariance_parameter_names: d
      .predictor_names
      .iter()
      .cloned()
      .chain(["iv_common".into()])
      .collect(),
    covariance,
    log_likelihood: ll,
    restricted_log_likelihood: restricted.log_likelihood,
    independence_likelihood_ratio: lr,
    independence_df: 1,
    independence_p_value: chi_square_tail(lr, 1),
    random_utility_iv_bounds_satisfied: lambda <= 1.0,
    iterations: solution.iterations,
    converged_starts,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_probabilities: solution.evaluation.probabilities,
    sample: d.sample.clone(),
  })
}

/// Current complete-set probabilities in input order, independent of outcomes.
/// Upper features remain constant within nests; keys match the retained map.
pub fn normalized_common_nested_predict_set(
  result: &NormalizedCommonNestedLogitResult,
  rows: &[Vec<f64>],
  codes: &[i64],
  nest_codes: &[i64],
) -> Result<Vec<f64>, StatsError> {
  let order = alternative_order(codes)?;
  let k = result.predictor_names.len();
  if rows.len() != codes.len()
    || nest_codes.len() != codes.len()
    || order.iter().map(|&i| codes[i]).collect::<Vec<_>>() != result.alternative_levels
    || order.iter().map(|&i| nest_codes[i]).collect::<Vec<_>>() != result.alternative_nests
  {
    return Err(invalid(
      "prediction requires the complete retained alternative/nest map",
    ));
  }
  let lambda = result.dissimilarity.value;
  if k > 31
    || result.lower_predictor_count == 0
    || result.lower_predictor_count > k
    || result.coefficients.len() != k
    || result.nest_levels.len() != 2
    || result.nest_levels[0] >= result.nest_levels[1]
    || !lambda.is_finite()
    || lambda <= 0.0
    || rows
      .iter()
      .any(|row| row.len() != k || row.iter().any(|v| !v.is_finite()))
    || result.coefficients.iter().any(|b| !b.value.is_finite())
  {
    return Err(invalid(
      "prediction requires finite retained parameters and current predictors",
    ));
  }
  let nests = nest_codes
    .iter()
    .map(|code| {
      result
        .nest_levels
        .binary_search(code)
        .map_err(|_| invalid("unknown nest in prediction"))
    })
    .collect::<Result<Vec<_>, _>>()?;
  let refs = rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
  upper_constants(&refs, &nests, result.lower_predictor_count).map_err(context)?;
  let centered = rows
    .iter()
    .map(|row| {
      row
        .iter()
        .zip(&rows[0])
        .map(|(v, b)| v - b)
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let refs = centered.iter().map(Vec::as_slice).collect::<Vec<_>>();
  let theta = result
    .coefficients
    .iter()
    .enumerate()
    .map(|(r, b)| {
      b.value
        / if r < result.lower_predictor_count {
          lambda
        } else {
          1.0
        }
    })
    .chain([lambda.ln(); 2])
    .collect::<Vec<_>>();
  Ok(
    terms(&refs, &nests, result.lower_predictor_count, &theta)
      .map_err(context)?
      .probabilities,
  )
}
