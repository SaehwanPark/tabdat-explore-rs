//! Immutable reporting snapshots, not restorable estimator/design caches.

use serde::{Deserialize, Serialize};

pub const MAX_STORED_ESTIMATES: usize = 32;
pub const MAX_COMPARISON_MODELS: usize = 8;
pub const MAX_COMPARISON_TERMS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "names", rename_all = "snake_case")]
pub enum EstimatesCommand {
  Store(String),
  Table(Vec<String>),
  Drop(Vec<String>),
}

impl EstimatesCommand {
  /// Validate typed callers as well as parsed commands, before registry effects.
  pub fn validate(&self) -> Result<(), String> {
    let names: Vec<&str> = match self {
      Self::Store(name) => vec![name],
      Self::Table(names) | Self::Drop(names) => names.iter().map(String::as_str).collect(),
    };
    let limit = if matches!(self, Self::Table(_)) {
      MAX_COMPARISON_MODELS
    } else {
      MAX_STORED_ESTIMATES
    };
    if names.is_empty() || names.len() > limit {
      return Err(format!("requires 1 to {limit} explicit estimate names"));
    }
    for name in &names {
      let bytes = name.as_bytes();
      if bytes.is_empty()
        || bytes.len() > 32
        || !(bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        || !bytes
          .iter()
          .all(|c| c.is_ascii_alphanumeric() || *c == b'_')
      {
        return Err("estimate names must be ASCII identifiers of 1 to 32 bytes".into());
      }
    }
    if names
      .iter()
      .collect::<std::collections::BTreeSet<_>>()
      .len()
      != names.len()
    {
      return Err("estimate names must be unique".into());
    }
    Ok(())
  }
}

/// Selected coefficient inference is copied from a fit, never recomputed here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimateSnapshot {
  pub name: String,
  pub estimator: String,
  pub outcome: String,
  /// The explicit NLS formula and ordered bindings; absent for other estimators.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub nls_model: Option<tabdat_stats::NlsModel>,
  /// Centered response-scale R-squared for NLS, when defined.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub nls_centered_r_squared: Option<f64>,
  pub predictors: Vec<String>,
  pub include_intercept: bool,
  pub covariance: String,
  pub cluster_variable: Option<String>,
  pub cluster_count: Option<usize>,
  pub covariance_parameter_names: Vec<String>,
  pub covariance_matrix: Vec<Vec<f64>>,
  pub coefficients: Vec<tabdat_stats::CoefficientEstimate>,
  pub observation_count: usize,
  pub event_count: Option<usize>,
  /// Candidate positions at fit time; not current-data identities/fingerprints.
  pub sample: tabdat_stats::EstimationSample,
  pub reference_distribution: String,
  pub inference_degrees_of_freedom: Option<usize>,
  pub log_likelihood: Option<f64>,
  pub log_likelihood_convention: Option<String>,
  pub r_squared: Option<f64>,
  pub mcfadden_r_squared: Option<f64>,
  pub root_mse: Option<f64>,
  /// Linear residual df can differ from clustered coefficient-inference df.
  pub df_residual: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstimateTableResult {
  /// Column order is exactly the explicit request, not registry sort order.
  pub models: Vec<EstimateSnapshot>,
  /// First-seen union; absent model terms are unavailable, never zero.
  pub parameter_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstimateStoreResult {
  pub name: String,
  pub estimator: String,
  pub stored_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstimateDropResult {
  pub names: Vec<String>,
  pub stored_count: usize,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn requests_have_shared_name_duplicate_and_size_bounds() {
    assert!(EstimatesCommand::Store("Model_1".into()).validate().is_ok());
    for name in ["", "1fit", "a b", "a,b", "*", "Δ", "a\n", &"a".repeat(33)] {
      assert!(
        EstimatesCommand::Store(name.into()).validate().is_err(),
        "{name:?}"
      );
    }
    assert!(EstimatesCommand::Table(vec![]).validate().is_err());
    assert!(
      EstimatesCommand::Drop(vec!["a".into(), "a".into()])
        .validate()
        .is_err()
    );
    assert!(
      EstimatesCommand::Table((0..9).map(|i| format!("m{i}")).collect())
        .validate()
        .is_err()
    );
    assert!(
      EstimatesCommand::Drop((0..33).map(|i| format!("m{i}")).collect())
        .validate()
        .is_err()
    );
    assert!(
      EstimatesCommand::Table(vec!["A".into(), "a".into()])
        .validate()
        .is_ok()
    );
  }

  #[test]
  fn requests_roundtrip_without_implicit_actions() {
    for command in [
      EstimatesCommand::Store("one".into()),
      EstimatesCommand::Table(vec!["two".into(), "one".into()]),
      EstimatesCommand::Drop(vec!["one".into()]),
    ] {
      let text = serde_json::to_string(&command).unwrap();
      assert_eq!(
        serde_json::from_str::<EstimatesCommand>(&text).unwrap(),
        command
      );
    }
  }
}
