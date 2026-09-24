#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::error::StatsError;

/// Explicit, inspectable sample identity tracking for statistical estimation.
///
/// Ensures sample composition and retained row mapping are transparent rather than
/// concealing different row sets behind identical row counts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimationSample {
  /// 0-based indices of original candidate rows retained in the estimation sample.
  pub retained_indices: Vec<usize>,
  /// Total number of candidate observations before missingness/filter exclusion.
  pub total_observations: usize,
  /// Number of observations dropped due to missing values or explicit subset filters.
  pub dropped_observations: usize,
  /// Optional observation weights aligned with retained observations.
  pub weights: Option<Vec<f64>>,
  /// Optional cluster group identifiers aligned with retained observations.
  pub cluster_groups: Option<Vec<String>>,
}

impl EstimationSample {
  /// Construct an estimation sample with full validation of invariants.
  pub fn new(
    retained_indices: Vec<usize>,
    total_observations: usize,
    dropped_observations: usize,
    weights: Option<Vec<f64>>,
    cluster_groups: Option<Vec<String>>,
  ) -> Result<Self, StatsError> {
    if retained_indices.len() + dropped_observations != total_observations {
      return Err(StatsError::DimensionMismatch(format!(
        "retained indices count ({}) + dropped observations ({}) must equal total observations ({})",
        retained_indices.len(),
        dropped_observations,
        total_observations
      )));
    }

    if let Some(w) = &weights {
      if w.len() != retained_indices.len() {
        return Err(StatsError::DimensionMismatch(format!(
          "weights count ({}) must match retained sample count ({})",
          w.len(),
          retained_indices.len()
        )));
      }
      for &val in w {
        if !val.is_finite() {
          return Err(StatsError::NonFiniteValue("weight is not finite".into()));
        }
        if val <= 0.0 {
          return Err(StatsError::NonPositiveWeights);
        }
      }
    }

    if let Some(c) = &cluster_groups
      && c.len() != retained_indices.len()
    {
      return Err(StatsError::DimensionMismatch(format!(
        "cluster groups count ({}) must match retained sample count ({})",
        c.len(),
        retained_indices.len()
      )));
    }

    Ok(Self {
      retained_indices,
      total_observations,
      dropped_observations,
      weights,
      cluster_groups,
    })
  }

  /// Convenience constructor when all observations from 0..n are retained.
  pub fn all_retained(n: usize) -> Self {
    Self {
      retained_indices: (0..n).collect(),
      total_observations: n,
      dropped_observations: 0,
      weights: None,
      cluster_groups: None,
    }
  }

  /// Construct an estimation sample from a boolean retention mask.
  pub fn from_mask(
    mask: &[bool],
    weights: Option<Vec<f64>>,
    cluster_groups: Option<Vec<String>>,
  ) -> Result<Self, StatsError> {
    let mut retained_indices = Vec::new();
    let mut dropped_observations = 0;
    for (idx, &keep) in mask.iter().enumerate() {
      if keep {
        retained_indices.push(idx);
      } else {
        dropped_observations += 1;
      }
    }
    Self::new(
      retained_indices,
      mask.len(),
      dropped_observations,
      weights,
      cluster_groups,
    )
  }

  /// Number of retained observations used in estimation.
  #[inline]
  pub fn retained_count(&self) -> usize {
    self.retained_indices.len()
  }

  /// Whether observation weights are applied.
  #[inline]
  pub fn is_weighted(&self) -> bool {
    self.weights.is_some()
  }

  /// Whether cluster identifiers are attached.
  #[inline]
  pub fn is_clustered(&self) -> bool {
    self.cluster_groups.is_some()
  }
}
