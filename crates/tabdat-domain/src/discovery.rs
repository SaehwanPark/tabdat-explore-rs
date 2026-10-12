use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCatalogEntry {
  pub help_topic: Option<String>,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCatalogResult {
  pub commands: Vec<CommandCatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEffectEntry {
  pub effects: Vec<crate::CommandEffect>,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEffectCatalogResult {
  pub commands: Vec<CommandEffectEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentDescriptor {
  pub name: String,
  pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionDescriptor {
  pub name: String,
  pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSchemaResult {
  pub arguments: Vec<ArgumentDescriptor>,
  pub help_topic: Option<String>,
  pub name: String,
  pub options: Vec<OptionDescriptor>,
  pub syntax: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpTopicResult {
  pub help_topic: String,
  pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandExplainResult {
  pub command_name: String,
  pub execution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultEnvelope<T> {
  pub data: T,
  pub result_type: String,
  pub schema_version: u32,
}

impl<T> ResultEnvelope<T> {
  pub fn new(data: T, result_type: &str) -> Self {
    Self {
      data,
      result_type: result_type.to_string(),
      schema_version: 1,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub line: Option<usize>,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub path: Option<String>,
  #[serde(rename = "type")]
  pub error_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
  pub error: ErrorDetail,
  pub schema_version: u32,
}

impl ErrorEnvelope {
  pub fn new(message: impl Into<String>, error_type: &str) -> Self {
    Self {
      error: ErrorDetail {
        line: None,
        message: message.into(),
        path: None,
        error_type: error_type.to_string(),
      },
      schema_version: 1,
    }
  }

  pub fn with_location(
    message: impl Into<String>,
    error_type: &str,
    path: impl Into<String>,
    line: usize,
  ) -> Self {
    Self {
      error: ErrorDetail {
        line: Some(line),
        message: message.into(),
        path: Some(path.into()),
        error_type: error_type.to_string(),
      },
      schema_version: 1,
    }
  }
}
