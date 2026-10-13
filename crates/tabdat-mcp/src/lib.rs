#![forbid(unsafe_code)]
//! Transport-independent typed tool boundary. No MCP/JSON-RPC server is provided.

use serde::{Deserialize, Serialize};
use tabdat_domain::*;
use tabdat_execution::Session;
use tabdat_reporting::{catalog, json_results};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case", deny_unknown_fields)]
pub enum ToolRequest {
  Catalog,
  Effects,
  Describe { name: String },
  Help { topic: String },
  Explain { command: String },
  Execute { command: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolReply {
  Catalog(CommandCatalogResult),
  Effects(CommandEffectCatalogResult),
  Described(CommandSchemaResult),
  Help(HelpTopicResult),
  Explained(CommandExplainResult),
  Executed(Box<ExecutionResult>),
}

impl ToolReply {
  /// Reuse the exact existing successful CLI execution envelope when supported.
  /// Unsupported reporting variants remain typed results, not fabricated JSON.
  pub fn execution_json(&self) -> Option<String> {
    match self {
      Self::Executed(result) => json_results::serialize_execution_result(result),
      _ => None,
    }
  }
}

/// Route tool inputs through the same parser/coordinator/reporting contracts.
/// A transport must enforce its own authentication, resource limits and output
/// framing before exposing Execute; this function is not an execution sandbox.
pub fn dispatch(session: &mut Session, request: ToolRequest) -> Result<ToolReply, TabDatError> {
  let input_error = |message| TabDatError::Parse { message };
  match request {
    ToolRequest::Catalog => Ok(ToolReply::Catalog(catalog::command_catalog_result())),
    ToolRequest::Effects => Ok(ToolReply::Effects(catalog::command_effect_catalog_result())),
    ToolRequest::Describe { name } => catalog::describe_command_result(&name)
      .map(ToolReply::Described)
      .map_err(input_error),
    ToolRequest::Help { topic } => catalog::help_topic_result(&topic)
      .map(ToolReply::Help)
      .map_err(input_error),
    ToolRequest::Explain { command } => catalog::explain_result(&command)
      .map(ToolReply::Explained)
      .map_err(|(message, _)| input_error(message)),
    ToolRequest::Execute { command } => {
      let command =
        tabdat_language::parse_command(&command).map_err(|error| input_error(error.to_string()))?;
      session
        .execute(command)
        .map(|result| ToolReply::Executed(Box::new(result)))
        .map_err(TabDatError::from)
    }
  }
}
