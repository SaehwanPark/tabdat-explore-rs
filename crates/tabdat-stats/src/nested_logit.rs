//! Earlier, non-normalized two-level nested choice probabilities. These are not
//! the preferred common-utility RUM normalization when nest scales differ.

use crate::conditional_logit::{Layout, alternative_order, validate as validate_choices};
use crate::distributions::chi_square_tail;
use crate::matrix::choice_information_inverse;
use crate::{
  ConditionalLogitProblem, EstimationSample, LogitEstimate, StatsError, fit_conditional_logit,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const TOLERANCE: f64 = 1e-10;
const NORMAL_95: f64 = 1.959963984540054;

/// Design columns are lower-utility predictors followed by upper predictors.
/// Upper predictors must be constant within each case/nest. Unlike conditional
/// logit, case-common lower utility shifts must NOT be subtracted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonNormalizedNestedLogitProblem {
  pub choice: ConditionalLogitProblem,
  pub lower_predictor_count: usize,
  pub nest_variable: String,
  pub nest_codes: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedLogitOptions {
  pub max_iterations: usize,
}
impl Default for NestedLogitOptions {
  fn default() -> Self {
    Self {
      max_iterations: 200,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NestedScaleEstimate {
  pub nest_code: i64,
  pub estimate: LogitEstimate,
  /// One is an interior independence restriction; zero is outside the positive
  /// scale domain and is not the null used by the reported normal diagnostic.
  pub test_null: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonNormalizedNestedLogitResult {
  pub outcome: String,
  pub predictor_names: Vec<String>,
  pub lower_predictor_count: usize,
  pub group_variable: String,
  pub alternative_variable: String,
  pub nest_variable: String,
  pub alternative_levels: Vec<i64>,
  pub nest_levels: Vec<i64>,
  /// Nest codes in the same order as alternative_levels.
  pub alternative_nests: Vec<i64>,
  pub chosen_counts: Vec<usize>,
  pub observation_count: usize,
  pub choice_set_count: usize,
  pub choice_set_size: usize,
  pub candidate_choice_set_count: usize,
  pub excluded_choice_set_count: usize,
  pub coefficients: Vec<LogitEstimate>,
  pub dissimilarities: Vec<NestedScaleEstimate>,
  /// Slopes/upper coefficients, then the two natural (not logged) scales.
  pub covariance_parameter_names: Vec<String>,
  pub covariance: Vec<Vec<f64>>,
  pub log_likelihood: f64,
  pub restricted_log_likelihood: f64,
  pub independence_likelihood_ratio: f64,
  pub independence_df: usize,
  pub independence_p_value: f64,
  /// Descriptive fitted-IV bound check, not a test of the composite RUM null.
  pub random_utility_iv_bounds_satisfied: bool,
  pub iterations: usize,
  pub converged_starts: usize,
  pub max_abs_scaled_score: f64,
  pub max_abs_scaled_newton_step: f64,
  pub fitted_probabilities: Vec<f64>,
  pub sample: Option<EstimationSample>,
}

fn invalid(message: &str) -> StatsError {
  StatsError::InvalidEstimation(format!("nlogit nonnormalized: {message}"))
}
fn failed(message: &str) -> StatsError {
  StatsError::EstimationFailed(format!("nlogit nonnormalized: {message}"))
}
fn maximum_absolute(values: &[f64]) -> f64 {
  values.iter().map(|v| v.abs()).fold(0.0, f64::max)
}
fn matrix_vector(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
  a.iter()
    .map(|row| row.iter().zip(x).map(|(a, b)| a * b).sum())
    .collect()
}
fn inference(
  name: String,
  value: f64,
  variance: f64,
  null: f64,
) -> Result<LogitEstimate, StatsError> {
  let standard_error = variance.sqrt();
  let statistic = (value - null) / standard_error;
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
    return Err(failed("nonfinite normal inference or nonpositive variance"));
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

struct NestedLayout {
  choice: Layout,
  nest_levels: Vec<i64>,
  alternative_nests: Vec<i64>,
  row_nest: Vec<usize>,
}
fn upper_constants(rows: &[&[f64]], nests: &[usize], lower: usize) -> Result<(), StatsError> {
  for nest in 0..2 {
    let first = nests
      .iter()
      .position(|&n| n == nest)
      .ok_or_else(|| invalid("every set requires both nests"))?;
    if nests.iter().filter(|&&n| n == nest).count() < 2 {
      return Err(invalid("each nest requires at least two alternatives"));
    }
    if rows
      .iter()
      .zip(nests)
      .any(|(row, &n)| n == nest && row[lower..] != rows[first][lower..])
    {
      return Err(invalid(
        "upper predictors must be constant within each case/nest",
      ));
    }
  }
  Ok(())
}
fn validate(problem: &NonNormalizedNestedLogitProblem) -> Result<NestedLayout, StatsError> {
  let choice = validate_choices(&problem.choice)?;
  let d = &problem.choice.design;
  let n = d.response.len();
  let k = d.predictor_names.len();
  let p = k + 2;
  if problem.lower_predictor_count == 0
    || problem.lower_predictor_count > k
    || problem.lower_predictor_count > 16
    || k - problem.lower_predictor_count > 16
    || p > 32
    || choice.cases.len() <= p
    || n.saturating_mul(p * p) > 20_000_000
  {
    return Err(invalid(
      "requires 1..16 lower, 0..16 upper predictors, P<=32, G>P and rows*P^2<=20000000 (P includes both scales)",
    ));
  }
  if problem.nest_variable.trim().is_empty()
    || problem.nest_codes.len() != n
    || problem
      .nest_codes
      .iter()
      .any(|c| c.unsigned_abs() > 9_007_199_254_740_991)
  {
    return Err(invalid(
      "requires a named exact nest code per retained row within +/- (2^53-1)",
    ));
  }
  let nest_levels = problem
    .nest_codes
    .iter()
    .copied()
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
  if nest_levels.len() != 2 {
    return Err(invalid("exactly two nests are supported"));
  }
  let row_nest = problem
    .nest_codes
    .iter()
    .map(|c| nest_levels.binary_search(c).unwrap())
    .collect::<Vec<_>>();
  let alternative_nests = choice.cases[0]
    .iter()
    .map(|&i| problem.nest_codes[i])
    .collect::<Vec<_>>();
  for case in &choice.cases {
    if case
      .iter()
      .zip(&alternative_nests)
      .any(|(&i, &code)| problem.nest_codes[i] != code)
    {
      return Err(invalid(
        "alternative-to-nest mapping must be identical in every set",
      ));
    }
    let rows = case
      .iter()
      .map(|&i| d.design_matrix[i].as_slice())
      .collect::<Vec<_>>();
    let nests = case.iter().map(|&i| row_nest[i]).collect::<Vec<_>>();
    upper_constants(&rows, &nests, problem.lower_predictor_count)?;
  }
  let mut names = d.predictor_names.clone();
  names.extend(nest_levels.iter().map(|code| format!("iv[{code}]")));
  if names.iter().collect::<BTreeSet<_>>().len() != p {
    return Err(invalid("all covariance parameter names must be unique"));
  }
  Ok(NestedLayout {
    choice,
    nest_levels,
    alternative_nests,
    row_nest,
  })
}

struct Softmax {
  p: Vec<f64>,
  log_p: Vec<f64>,
  log_sum: f64,
}
fn softmax(eta: &[f64]) -> Result<Softmax, StatsError> {
  if eta.is_empty() || eta.iter().any(|v| !v.is_finite()) {
    return Err(failed("nonfinite utility"));
  }
  let center = eta.iter().copied().fold(f64::NEG_INFINITY, f64::max);
  let log_sum_centered = eta.iter().map(|v| (v - center).exp()).sum::<f64>().ln();
  let log_p = eta
    .iter()
    .map(|v| (v - center) - log_sum_centered)
    .collect::<Vec<_>>();
  let log_sum = center + log_sum_centered;
  if !log_sum.is_finite() {
    return Err(failed("nonfinite inclusive value"));
  }
  Ok(Softmax {
    p: log_p.iter().map(|v| v.exp()).collect(),
    log_p,
    log_sum,
  })
}
struct Terms {
  within: Vec<f64>,
  nest: Vec<f64>,
  inclusive: [f64; 2],
  lambda: [f64; 2],
  probabilities: Vec<f64>,
  log_probabilities: Vec<f64>,
}
fn terms(
  rows: &[&[f64]],
  nests: &[usize],
  lower: usize,
  theta: &[f64],
) -> Result<Terms, StatsError> {
  let k = theta.len() - 2;
  let lambda = [theta[k].exp(), theta[k + 1].exp()];
  if lambda.iter().any(|v| !v.is_finite() || *v <= 0.0) {
    return Err(failed("undefined positive nest scale"));
  }
  let eta = rows
    .iter()
    .map(|row| {
      row[..lower]
        .iter()
        .zip(&theta[..lower])
        .map(|(x, b)| x * b)
        .sum::<f64>()
    })
    .collect::<Vec<_>>();
  let mut inclusive = [0.0; 2];
  let mut upper = [0.0; 2];
  let mut within = vec![0.0; rows.len()];
  let mut log_within = vec![0.0; rows.len()];
  for m in 0..2 {
    let positions = nests
      .iter()
      .enumerate()
      .filter_map(|(i, &nest)| (nest == m).then_some(i))
      .collect::<Vec<_>>();
    let conditional = softmax(&positions.iter().map(|&i| eta[i]).collect::<Vec<_>>())?;
    inclusive[m] = conditional.log_sum;
    upper[m] = rows[positions[0]][lower..]
      .iter()
      .zip(&theta[lower..k])
      .map(|(x, b)| x * b)
      .sum::<f64>()
      + lambda[m] * inclusive[m];
    for (j, &i) in positions.iter().enumerate() {
      within[i] = conditional.p[j];
      log_within[i] = conditional.log_p[j];
    }
  }
  let top = softmax(&upper)?;
  let probabilities = within
    .iter()
    .zip(nests)
    .map(|(p, &m)| p * top.p[m])
    .collect();
  let log_probabilities = log_within
    .iter()
    .zip(nests)
    .map(|(lp, &m)| lp + top.log_p[m])
    .collect();
  Ok(Terms {
    within,
    nest: top.p,
    inclusive,
    lambda,
    probabilities,
    log_probabilities,
  })
}

struct Evaluation {
  log_likelihood: f64,
  probabilities: Vec<f64>,
  score: Vec<f64>,
  observed: Vec<Vec<f64>>,
  fisher: Vec<Vec<f64>>,
}
fn evaluate(
  x: &[Vec<f64>],
  layout: &NestedLayout,
  lower: usize,
  theta: &[f64],
) -> Result<Evaluation, StatsError> {
  let p = theta.len();
  let k = p - 2;
  let mut result = Evaluation {
    log_likelihood: 0.0,
    probabilities: vec![0.0; x.len()],
    score: vec![0.0; p],
    observed: vec![vec![0.0; p]; p],
    fisher: vec![vec![0.0; p]; p],
  };
  for (case, &selected) in layout.choice.cases.iter().zip(&layout.choice.chosen) {
    let rows = case.iter().map(|&i| x[i].as_slice()).collect::<Vec<_>>();
    let nests = case.iter().map(|&i| layout.row_nest[i]).collect::<Vec<_>>();
    let t = terms(&rows, &nests, lower, theta)?;
    let chosen = case.iter().position(|&i| i == selected).unwrap();
    let chosen_nest = nests[chosen];
    result.log_likelihood += t.log_probabilities[chosen];
    for (i, &row) in case.iter().enumerate() {
      result.probabilities[row] = t.probabilities[i];
    }
    let mut means = vec![vec![0.0; lower]; 2];
    let mut covariances = vec![vec![vec![0.0; lower]; lower]; 2];
    for (i, row) in rows.iter().enumerate() {
      for (r, mean) in means[nests[i]].iter_mut().enumerate() {
        *mean += t.within[i] * row[r];
      }
    }
    for (i, row) in rows.iter().enumerate() {
      let m = nests[i];
      for r in 0..lower {
        for s in 0..lower {
          covariances[m][r][s] += t.within[i] * (row[r] - means[m][r]) * (row[s] - means[m][s]);
        }
      }
    }
    let mut gradients = vec![vec![0.0; p]; 2];
    let mut hessians = vec![vec![vec![0.0; p]; p]; 2];
    for m in 0..2 {
      for r in 0..lower {
        gradients[m][r] = t.lambda[m] * means[m][r];
        hessians[m][r][k + m] = gradients[m][r];
        hessians[m][k + m][r] = gradients[m][r];
        for s in 0..lower {
          hessians[m][r][s] = t.lambda[m] * covariances[m][r][s];
        }
      }
      let first = nests.iter().position(|&n| n == m).unwrap();
      gradients[m][lower..k].copy_from_slice(&rows[first][lower..k]);
      gradients[m][k + m] = t.lambda[m] * t.inclusive[m];
      hessians[m][k + m][k + m] = gradients[m][k + m];
    }
    let mean_gradient = (0..p)
      .map(|r| (0..2).map(|m| t.nest[m] * gradients[m][r]).sum::<f64>())
      .collect::<Vec<_>>();
    // Difference sums preserve small tail scores near a certain chosen outcome.
    for r in 0..p {
      result.score[r] += (0..2)
        .map(|m| t.nest[m] * (gradients[chosen_nest][r] - gradients[m][r]))
        .sum::<f64>();
      if r < lower {
        result.score[r] += rows
          .iter()
          .enumerate()
          .filter(|(i, _)| nests[*i] == chosen_nest)
          .map(|(i, row)| t.within[i] * (rows[chosen][r] - row[r]))
          .sum::<f64>();
      }
      for s in 0..p {
        let between = (0..2)
          .map(|m| {
            t.nest[m] * (gradients[m][r] - mean_gradient[r]) * (gradients[m][s] - mean_gradient[s])
          })
          .sum::<f64>();
        result.observed[r][s] += between - hessians[chosen_nest][r][s]
          + (0..2).map(|m| t.nest[m] * hessians[m][r][s]).sum::<f64>();
        result.fisher[r][s] += between;
        if r < lower && s < lower {
          result.observed[r][s] += covariances[chosen_nest][r][s];
          result.fisher[r][s] += (0..2)
            .map(|m| t.nest[m] * covariances[m][r][s])
            .sum::<f64>();
        }
      }
    }
  }
  let g = layout.choice.cases.len() as f64;
  for v in result
    .score
    .iter_mut()
    .chain(result.observed.iter_mut().flatten())
    .chain(result.fisher.iter_mut().flatten())
  {
    *v /= g;
  }
  if !result.log_likelihood.is_finite()
    || result
      .score
      .iter()
      .chain(result.observed.iter().flatten())
      .chain(result.fisher.iter().flatten())
      .any(|v| !v.is_finite())
  {
    return Err(failed("nonfinite likelihood derivatives"));
  }
  Ok(result)
}

struct Solution {
  theta: Vec<f64>,
  evaluation: Evaluation,
  inverse: Vec<Vec<f64>>,
  iterations: usize,
  score_bound: f64,
  step_bound: f64,
}
fn optimize(
  x: &[Vec<f64>],
  layout: &NestedLayout,
  lower: usize,
  mut theta: Vec<f64>,
  options: &NestedLogitOptions,
) -> Result<Solution, StatsError> {
  for iteration in 0..options.max_iterations {
    let current = evaluate(x, layout, lower, &theta)?;
    let observed_inverse = choice_information_inverse(&current.observed).ok();
    // Expected information gives an ascent direction when the nonconcave
    // observed Hessian is indefinite. It is never the reported covariance.
    let inverse = match &observed_inverse {
      Some(inverse) => inverse.clone(),
      None => choice_information_inverse(&current.fisher)
        .map_err(|e| failed(&format!("unidentified nested parameters: {e}")))?,
    };
    let step = matrix_vector(&inverse, &current.score);
    if step.iter().any(|v| !v.is_finite()) {
      return Err(failed("nonfinite scoring step"));
    }
    let score_bound = maximum_absolute(&current.score);
    let step_bound = maximum_absolute(&step);
    if observed_inverse.is_some() && score_bound <= TOLERANCE && step_bound <= TOLERANCE {
      return Ok(Solution {
        theta,
        evaluation: current,
        inverse,
        iterations: iteration + 1,
        score_bound,
        step_bound,
      });
    }
    let mut damping = 1.0;
    let mut accepted = None;
    for _ in 0..40 {
      let candidate = theta
        .iter()
        .zip(&step)
        .map(|(b, s)| b + damping * s)
        .collect::<Vec<_>>();
      if let Ok(trial) = evaluate(x, layout, lower, &candidate)
        && trial.log_likelihood
          >= current.log_likelihood - 1e-12 * (1.0 + current.log_likelihood.abs())
      {
        accepted = Some(candidate);
        break;
      }
      damping *= 0.5;
    }
    theta = accepted.ok_or_else(|| failed("nested likelihood line search did not converge"))?;
  }
  Err(failed(
    "iteration limit reached; possible separation or nonidentification",
  ))
}

/// Positive-scale, non-normalized two-level ML. Requires an identified lambda=1
/// conditional restriction and a positive full observed information at the fit.
/// Two deterministic starts improve local-search coverage, not a global proof.
pub fn fit_non_normalized_nested_logit(
  problem: &NonNormalizedNestedLogitProblem,
  options: &NestedLogitOptions,
) -> Result<NonNormalizedNestedLogitResult, StatsError> {
  let layout = validate(problem)?;
  let d = &problem.choice.design;
  let n = d.response.len();
  let k = d.predictor_names.len();
  let g = layout.choice.cases.len();
  let restricted = fit_conditional_logit(&problem.choice, &Default::default())?;
  let mut x = d.design_matrix.clone();
  let mut scales = Vec::with_capacity(k);
  for r in 0..k {
    let max = x.iter().map(|row| row[r].abs()).fold(0.0, f64::max);
    if max == 0.0 {
      return Err(invalid("a predictor is identically zero"));
    }
    let scale = max * (x.iter().map(|row| (row[r] / max).powi(2)).sum::<f64>() / n as f64).sqrt();
    if !scale.is_finite() || scale <= 0.0 {
      return Err(failed("undefined raw predictor scale"));
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
    .chain([0.0; 2])
    .collect::<Vec<_>>();
  let restriction = evaluate(
    &x,
    &layout,
    problem.lower_predictor_count,
    &restriction_theta,
  )?;
  choice_information_inverse(&restriction.fisher).map_err(|_| {
    invalid("nest scales are not locally identified at the independence restriction; a regular LR reference is unavailable")
  })?;
  let mut best: Option<Solution> = None;
  let mut converged_starts = 0;
  let mut last_error = String::new();
  for lambda in [1.0_f64, 2.0] {
    let mut theta = restricted
      .coefficients
      .iter()
      .zip(&scales)
      .enumerate()
      .map(|(r, (b, s))| {
        b.value * s
          / if r < problem.lower_predictor_count {
            lambda
          } else {
            1.0
          }
      })
      .collect::<Vec<_>>();
    theta.extend([lambda.ln(); 2]);
    match optimize(&x, &layout, problem.lower_predictor_count, theta, options) {
      Ok(solution) => {
        converged_starts += 1;
        if best
          .as_ref()
          .is_none_or(|b| solution.evaluation.log_likelihood > b.evaluation.log_likelihood)
        {
          best = Some(solution);
        }
      }
      Err(e) => last_error = e.to_string(),
    }
  }
  let solution = best.ok_or_else(|| failed(&last_error))?;
  let ll = solution.evaluation.log_likelihood;
  if ll < restricted.log_likelihood - 1e-12 * (1.0 + restricted.log_likelihood.abs()) {
    return Err(failed(
      "fitted likelihood is below its conditional restriction",
    ));
  }
  let lambdas = [solution.theta[k].exp(), solution.theta[k + 1].exp()];
  let jacobian = scales
    .iter()
    .map(|s| 1.0 / s)
    .chain(lambdas)
    .collect::<Vec<_>>();
  let covariance = solution
    .inverse
    .iter()
    .enumerate()
    .map(|(r, row)| {
      row
        .iter()
        .enumerate()
        .map(|(s, v)| v / g as f64 * jacobian[r] * jacobian[s])
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
        solution.theta[r] / scales[r],
        covariance[r][r],
        0.0,
      )
    })
    .collect::<Result<Vec<_>, _>>()?;
  let dissimilarities = (0..2)
    .map(|m| {
      Ok(NestedScaleEstimate {
        nest_code: layout.nest_levels[m],
        estimate: inference(
          format!("iv[{}]", layout.nest_levels[m]),
          lambdas[m],
          covariance[k + m][k + m],
          1.0,
        )?,
        test_null: 1.0,
      })
    })
    .collect::<Result<Vec<_>, StatsError>>()?;
  let covariance_parameter_names = d
    .predictor_names
    .iter()
    .cloned()
    .chain(dissimilarities.iter().map(|d| d.estimate.name.clone()))
    .collect();
  let mut chosen_counts = vec![0; layout.choice.levels.len()];
  for &i in &layout.choice.chosen {
    chosen_counts[layout
      .choice
      .levels
      .binary_search(&problem.choice.alternative_codes[i])
      .unwrap()] += 1;
  }
  let lr = (2.0 * (ll - restricted.log_likelihood)).max(0.0);
  Ok(NonNormalizedNestedLogitResult {
    outcome: d.outcome.clone(),
    predictor_names: d.predictor_names.clone(),
    lower_predictor_count: problem.lower_predictor_count,
    group_variable: problem.choice.group_variable.clone(),
    alternative_variable: problem.choice.alternative_variable.clone(),
    nest_variable: problem.nest_variable.clone(),
    choice_set_size: layout.choice.levels.len(),
    alternative_levels: layout.choice.levels,
    nest_levels: layout.nest_levels,
    alternative_nests: layout.alternative_nests,
    chosen_counts,
    observation_count: n,
    choice_set_count: g,
    candidate_choice_set_count: layout.choice.candidate_cases,
    excluded_choice_set_count: layout.choice.candidate_cases - g,
    coefficients,
    dissimilarities,
    covariance_parameter_names,
    covariance,
    log_likelihood: ll,
    restricted_log_likelihood: restricted.log_likelihood,
    independence_likelihood_ratio: lr,
    independence_df: 2,
    independence_p_value: chi_square_tail(lr, 2),
    random_utility_iv_bounds_satisfied: lambdas.iter().all(|v| *v <= 1.0),
    iterations: solution.iterations,
    converged_starts,
    max_abs_scaled_score: solution.score_bound,
    max_abs_scaled_newton_step: solution.step_bound,
    fitted_probabilities: solution.evaluation.probabilities,
    sample: d.sample.clone(),
  })
}

/// Complete current-set probabilities in input-row order, without outcomes.
/// The retained alternative/nest map and within-nest upper constancy are required.
pub fn non_normalized_nested_predict_set(
  result: &NonNormalizedNestedLogitResult,
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
  if k > 30
    || result.lower_predictor_count == 0
    || result.lower_predictor_count > k
    || result.coefficients.len() != k
    || result.nest_levels.len() != 2
    || result.dissimilarities.len() != 2
    || result.nest_levels[0] >= result.nest_levels[1]
    || rows
      .iter()
      .any(|row| row.len() != k || row.iter().any(|v| !v.is_finite()))
    || result.coefficients.iter().any(|b| !b.value.is_finite())
    || result
      .dissimilarities
      .iter()
      .zip(&result.nest_levels)
      .any(|(d, n)| d.nest_code != *n || !d.estimate.value.is_finite() || d.estimate.value <= 0.0)
  {
    return Err(invalid(
      "prediction requires finite retained parameters and current predictors",
    ));
  }
  let nests = nest_codes
    .iter()
    .map(|c| {
      result
        .nest_levels
        .binary_search(c)
        .map_err(|_| invalid("unknown nest in prediction"))
    })
    .collect::<Result<Vec<_>, _>>()?;
  let refs = rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
  upper_constants(&refs, &nests, result.lower_predictor_count)?;
  let theta = result
    .coefficients
    .iter()
    .map(|b| b.value)
    .chain(result.dissimilarities.iter().map(|d| d.estimate.value.ln()))
    .collect::<Vec<_>>();
  Ok(terms(&refs, &nests, result.lower_predictor_count, &theta)?.probabilities)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::EstimationProblem;

  #[test]
  fn score_and_observed_hessian_match_likelihood_derivatives_away_from_the_mle() {
    let mut y = Vec::new();
    let mut x = Vec::new();
    for g in 0..12 {
      for j in 0..4 {
        y.push(f64::from(j == g % 4));
        x.push(vec![
          ((g + 3 * j) % 7) as f64 / 3.0,
          ((2 * g + j) % 5) as f64 / 4.0,
          f64::from(j < 2),
          if j < 2 { 0.3 * g as f64 } else { 0.0 },
        ]);
      }
    }
    let problem = NonNormalizedNestedLogitProblem {
      choice: ConditionalLogitProblem {
        design: EstimationProblem::new(
          "chosen",
          ["p", "q", "dshore", "dshorey"].map(String::from).to_vec(),
          y,
          x,
        )
        .unwrap()
        .with_intercept(false),
        group_variable: "id".into(),
        alternative_variable: "alternative".into(),
        case_rows: (0..12).map(|g| (4 * g..4 * g + 4).collect()).collect(),
        alternative_codes: (0..48).map(|r| (r % 4 + 1) as i64).collect(),
      },
      lower_predictor_count: 2,
      nest_variable: "nest".into(),
      nest_codes: (0..48).map(|r| if r % 4 < 2 { 1 } else { 2 }).collect(),
    };
    let layout = validate(&problem).unwrap();
    let x = &problem.choice.design.design_matrix;
    let theta = [0.3, -0.2, 0.4, 0.1, 1.2_f64.ln(), 0.8_f64.ln()];
    let current = evaluate(x, &layout, 2, &theta).unwrap();
    assert!(
      current
        .observed
        .iter()
        .flatten()
        .zip(current.fisher.iter().flatten())
        .any(|(a, b)| (a - b).abs() > 1e-3)
    );
    for r in 0..theta.len() {
      let mut approximations = Vec::new();
      for h in [0.0002, 0.0001] {
        let mut plus = theta;
        plus[r] += h;
        let mut minus = theta;
        minus[r] -= h;
        let plus = evaluate(x, &layout, 2, &plus).unwrap();
        let minus = evaluate(x, &layout, 2, &minus).unwrap();
        let gradient = (plus.log_likelihood - minus.log_likelihood) / (2.0 * h * 12.0);
        let hessian = plus
          .score
          .iter()
          .zip(&minus.score)
          .map(|(a, b)| (a - b) / (2.0 * h))
          .collect::<Vec<_>>();
        approximations.push((gradient, hessian));
      }
      let gradient = (4.0 * approximations[1].0 - approximations[0].0) / 3.0;
      assert!((gradient - current.score[r]).abs() < 1e-8);
      for s in 0..theta.len() {
        let hessian = (4.0 * approximations[1].1[s] - approximations[0].1[s]) / 3.0;
        assert!(
          (hessian + current.observed[r][s]).abs() < 1e-8,
          "{r},{s}: {hessian} vs {}",
          current.observed[r][s]
        );
      }
    }
  }
}
