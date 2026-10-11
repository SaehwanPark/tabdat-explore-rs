//! JSON result envelopes for the bounded CLI execution surface.

use serde_json::{Value, json};
use tabdat_language::{Command, ExecutionMode, LazyEngine};
use tabdat_runtime::{DatasetInfo, ExecutionResult, LoadResult};

use crate::catalog::ResultEnvelope;

pub const SCRIPT_UNSUPPORTED_MESSAGE: &str =
  "--json successful script results are currently supported only for use and count";

/// Whether this command has a successful-result envelope in the batch CLI.
pub fn supports_batch_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Use { .. } | Command::Count | Command::Exit
  )
}

/// Whether a script command can execute without producing an unsupported JSON
/// success result. `run` is a container for nested commands and has no result
/// envelope of its own.
pub fn supports_script_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Use { .. } | Command::Count | Command::Exit | Command::Run { .. }
  )
}

/// Serialize supported execution results using the stable versioned CLI envelope.
///
/// `None` means the corresponding command must be rejected before execution by
/// [`supports_batch_command`].
pub fn serialize_execution_result(result: &ExecutionResult) -> Option<String> {
  let (data, result_type) = match result {
    ExecutionResult::Load(load) => (load_result_data(load), "LoadResult"),
    ExecutionResult::Count(count) => (json!({ "row_count": count.row_count }), "CountResult"),
    _ => return None,
  };

  let envelope = ResultEnvelope::new(data, result_type);
  Some(serde_json::to_string(&envelope).expect("JSON value envelopes serialize infallibly"))
}

fn load_result_data(result: &LoadResult) -> Value {
  json!({ "dataset": dataset_info_data(&result.dataset) })
}

fn dataset_info_data(dataset: &DatasetInfo) -> Value {
  // Keep the Python DatasetInfo field set; panel and label metadata are not
  // represented by this bounded Rust result type.
  json!({
    "columns": dataset.columns.iter().map(|column| json!({
      "data_type": column.data_type,
      "name": column.name,
    })).collect::<Vec<_>>(),
    "execution_mode": match dataset.execution_mode {
      ExecutionMode::Eager => "eager",
      ExecutionMode::Lazy => "lazy",
    },
    "label_metadata": null,
    "lazy_engine": match dataset.lazy_engine {
      Some(LazyEngine::DuckDb) => Some("duckdb"),
      Some(LazyEngine::Polars) => Some("polars"),
      None => None,
    },
    "panel_metadata": null,
    "path": dataset.source.to_string_lossy(),
    "row_count": dataset.row_count,
  })
}
