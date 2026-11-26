use serde::{Deserialize, Serialize};

use crate::StatsError;

/// Exact one-predictor enumeration stays bounded and keeps tied regions reportable.
pub const MAX_MAXIMUM_SCORE_OBSERVATIONS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaximumScoreDirection {
  /// The fitted class is one above the selected predictor threshold.
  Increasing,
  /// The fitted class is one below the selected predictor threshold.
  Decreasing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaximumScoreRule {
  Constant {
    class: bool,
  },
  Threshold {
    direction: MaximumScoreDirection,
    /// Largest observed predictor value in the lower training region.
    lower: f64,
    /// Smallest observed predictor value in the upper training region.
    upper: f64,
    /// Deterministic representative of the open interval `(lower, upper)`.
    threshold: f64,
  },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaximumScoreRegion {
  pub rule: MaximumScoreRule,
  pub correct_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaximumScoreClassification {
  pub true_positive: usize,
  pub false_positive: usize,
  pub false_negative: usize,
  pub true_negative: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaximumScoreResult {
  pub observation_count: usize,
  pub event_count: usize,
  pub correct_count: usize,
  pub empirical_score: f64,
  /// Euclidean unit-norm representative, ordered as intercept then slope.
  pub intercept: f64,
  pub slope: f64,
  pub classification: MaximumScoreClassification,
  /// All finite-sample maximizing classifier regions in deterministic order.
  pub optimal_regions: Vec<MaximumScoreRegion>,
  /// The first optimal region under the documented deterministic ordering.
  pub selected_rule: MaximumScoreRule,
  /// Sample classifications in input row order.
  pub predicted_classes: Vec<bool>,
}

#[derive(Clone, Copy)]
struct PredictorGroup {
  value: f64,
  count: usize,
  events: usize,
}

struct Candidate {
  rule: MaximumScoreRule,
  correct_count: usize,
}

/// Fits a binary maximum-score classifier with an intercept and one predictor.
///
/// The finite-sample objective is the number of correctly classified rows. The
/// routine enumerates every threshold split between distinct predictor values
/// for both slope directions and includes both constant classifiers. It reports
/// every tied maximizer and makes no uncertainty or significance claim.
pub fn fit_binary_maximum_score(
  outcome: &[f64],
  predictor: &[f64],
) -> Result<MaximumScoreResult, StatsError> {
  if outcome.is_empty() {
    return Err(StatsError::EmptySample);
  }
  if outcome.len() != predictor.len() {
    return Err(StatsError::DimensionMismatch(
      "maximum score outcome and predictor lengths differ".to_owned(),
    ));
  }
  if outcome.len() > MAX_MAXIMUM_SCORE_OBSERVATIONS {
    return Err(StatsError::InsufficientObservations(format!(
      "maximum score supports at most {MAX_MAXIMUM_SCORE_OBSERVATIONS} observations"
    )));
  }

  for value in outcome {
    if !value.is_finite() {
      return Err(StatsError::NonFiniteValue(
        "maximum score outcome must be finite".to_owned(),
      ));
    }
    if *value != 0.0 && *value != 1.0 {
      return Err(StatsError::InvalidEstimation(
        "maximum score outcome must contain only exact 0/1 values".to_owned(),
      ));
    }
  }
  for value in predictor {
    if !value.is_finite() {
      return Err(StatsError::NonFiniteValue(
        "maximum score predictor must be finite".to_owned(),
      ));
    }
  }

  let mut ordered: Vec<(f64, bool)> = predictor
    .iter()
    .zip(outcome)
    .map(|(x, y)| (if *x == 0.0 { 0.0 } else { *x }, *y == 1.0))
    .collect();
  ordered.sort_by(|left, right| {
    if left.0 == right.0 {
      std::cmp::Ordering::Equal
    } else {
      left.0.total_cmp(&right.0)
    }
  });

  let mut groups = Vec::<PredictorGroup>::new();
  for (value, event) in ordered {
    if let Some(group) = groups.last_mut().filter(|group| group.value == value) {
      group.count += 1;
      group.events += usize::from(event);
    } else {
      groups.push(PredictorGroup {
        value,
        count: 1,
        events: usize::from(event),
      });
    }
  }
  if groups.len() < 2 {
    return Err(StatsError::InvalidEstimation(
      "maximum score requires predictor variation".to_owned(),
    ));
  }

  let observation_count = outcome.len();
  let event_count = groups.iter().map(|group| group.events).sum::<usize>();
  let mut candidates = Vec::with_capacity(2 + 2 * groups.len().saturating_sub(1));
  candidates.push(Candidate {
    rule: MaximumScoreRule::Constant { class: false },
    correct_count: observation_count - event_count,
  });
  candidates.push(Candidate {
    rule: MaximumScoreRule::Constant { class: true },
    correct_count: event_count,
  });

  let mut left_count = 0;
  let mut left_events = 0;
  for split in 1..groups.len() {
    let group = groups[split - 1];
    left_count += group.count;
    left_events += group.events;
    let right_count = observation_count - left_count;
    let right_events = event_count - left_events;
    let lower = group.value;
    let upper = groups[split].value;

    let increasing_threshold =
      interval_representative(lower, upper, MaximumScoreDirection::Increasing);
    candidates.push(Candidate {
      rule: MaximumScoreRule::Threshold {
        direction: MaximumScoreDirection::Increasing,
        lower,
        upper,
        threshold: increasing_threshold,
      },
      correct_count: left_count - left_events + right_events,
    });

    let decreasing_threshold =
      interval_representative(lower, upper, MaximumScoreDirection::Decreasing);
    candidates.push(Candidate {
      rule: MaximumScoreRule::Threshold {
        direction: MaximumScoreDirection::Decreasing,
        lower,
        upper,
        threshold: decreasing_threshold,
      },
      correct_count: left_events + right_count - right_events,
    });
  }

  let best_count = candidates
    .iter()
    .map(|candidate| candidate.correct_count)
    .max()
    .expect("both constant classifiers are candidates");
  let optimal_regions: Vec<MaximumScoreRegion> = candidates
    .into_iter()
    .filter(|candidate| candidate.correct_count == best_count)
    .map(|candidate| MaximumScoreRegion {
      rule: candidate.rule,
      correct_count: candidate.correct_count,
    })
    .collect();
  let selected_rule = optimal_regions
    .first()
    .expect("a finite candidate set has a maximizer")
    .rule
    .clone();
  let (intercept, slope) = unit_norm_coefficients(&selected_rule);
  let predicted_classes = predictor
    .iter()
    .map(|x| maximum_score_predict(&selected_rule, *x))
    .collect::<Result<Vec<_>, _>>()?;
  let mut classification = MaximumScoreClassification {
    true_positive: 0,
    false_positive: 0,
    false_negative: 0,
    true_negative: 0,
  };
  for (actual, predicted) in outcome.iter().zip(&predicted_classes) {
    match (*actual == 1.0, *predicted) {
      (true, true) => classification.true_positive += 1,
      (false, true) => classification.false_positive += 1,
      (true, false) => classification.false_negative += 1,
      (false, false) => classification.true_negative += 1,
    }
  }
  let correct_count = classification.true_positive + classification.true_negative;

  Ok(MaximumScoreResult {
    observation_count,
    event_count,
    correct_count,
    empirical_score: correct_count as f64 / observation_count as f64,
    intercept,
    slope,
    classification,
    optimal_regions,
    selected_rule,
    predicted_classes,
  })
}

/// Predicts the class for one finite predictor value under a retained rule.
pub fn maximum_score_predict(rule: &MaximumScoreRule, predictor: f64) -> Result<bool, StatsError> {
  if !predictor.is_finite() {
    return Err(StatsError::NonFiniteValue(
      "maximum score prediction requires a finite predictor".to_owned(),
    ));
  }
  Ok(match rule {
    MaximumScoreRule::Constant { class } => *class,
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Increasing,
      threshold,
      ..
    } => predictor > *threshold,
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Decreasing,
      threshold,
      ..
    } => predictor < *threshold,
  })
}

fn interval_representative(lower: f64, upper: f64, direction: MaximumScoreDirection) -> f64 {
  let midpoint = lower * 0.5 + upper * 0.5;
  if lower < midpoint && midpoint < upper {
    midpoint
  } else {
    match direction {
      MaximumScoreDirection::Increasing => lower,
      MaximumScoreDirection::Decreasing => upper,
    }
  }
}

fn unit_norm_coefficients(rule: &MaximumScoreRule) -> (f64, f64) {
  match rule {
    MaximumScoreRule::Constant { class } => (if *class { 1.0 } else { -1.0 }, 0.0),
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Increasing,
      threshold,
      ..
    } => {
      let norm = threshold.hypot(1.0);
      (-threshold / norm, 1.0 / norm)
    }
    MaximumScoreRule::Threshold {
      direction: MaximumScoreDirection::Decreasing,
      threshold,
      ..
    } => {
      let norm = threshold.hypot(1.0);
      (threshold / norm, -1.0 / norm)
    }
  }
}
