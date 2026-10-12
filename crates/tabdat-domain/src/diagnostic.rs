use crate::{Capability, RuntimeError, ScriptError};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Top-level pipeline failures retain their typed source at interface boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabDatError {
  Parse { message: String },
  Script(ScriptError),
  Execution(Box<RuntimeError>),
  CapabilityUnavailable { capability: Capability },
  Cancelled,
}

impl fmt::Display for TabDatError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Parse { message } => f.write_str(message),
      Self::Script(error) => error.fmt(f),
      Self::Execution(error) => error.fmt(f),
      Self::CapabilityUnavailable { capability } => {
        write!(f, "capability unavailable: {capability:?}")
      }
      Self::Cancelled => f.write_str("command cancelled"),
    }
  }
}

impl Error for TabDatError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Script(error) => Some(error),
      Self::Execution(error) => Some(error.as_ref()),
      _ => None,
    }
  }
}

impl From<RuntimeError> for TabDatError {
  fn from(error: RuntimeError) -> Self {
    match error {
      RuntimeError::ScriptError(error) => Self::Script(error),
      error => Self::Execution(Box::new(error)),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
  Parse,
  Script,
  Execution,
  Capability,
  Cancelled,
}

/// Deterministic wire diagnostic without exposing backend debug text or handles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
  pub category: ErrorCategory,
  pub message: String,
  pub path: Option<std::path::PathBuf>,
  pub line: Option<usize>,
}

impl From<&TabDatError> for Diagnostic {
  fn from(error: &TabDatError) -> Self {
    let (category, path, line) = match error {
      TabDatError::Parse { .. } => (ErrorCategory::Parse, None, None),
      TabDatError::Script(error) => (
        ErrorCategory::Script,
        Some(error.path().to_owned()),
        Some(error.line()),
      ),
      TabDatError::Execution(_) => (ErrorCategory::Execution, None, None),
      TabDatError::CapabilityUnavailable { .. } => (ErrorCategory::Capability, None, None),
      TabDatError::Cancelled => (ErrorCategory::Cancelled, None, None),
    };
    Self {
      category,
      message: error.to_string(),
      path,
      line,
    }
  }
}
