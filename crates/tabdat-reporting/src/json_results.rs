//! JSON result envelopes for the bounded CLI execution surface.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Number, Value, json};
use tabdat_domain::{
  CellValue, CodebookResult, DatasetInfo, ExecutionResult, LoadResult, MissingResult,
  PreviewResult, RegressionResult, SummarizeResult, TtestGroupStats, TtestResult,
};
use tabdat_language::{Command, EstatCommand, EstatSubcommand, ExecutionMode, LazyEngine};

use crate::catalog::ResultEnvelope;

pub const BATCH_UNSUPPORTED_MESSAGE: &str = "--json successful batch results are currently supported only for use, describe, count, codebook, summarize, missing, tabulate, generate, replace, rename, recode, label, encode, decode, sql, keep/drop/select variable lists, sort, gsort, save, export, head, tail, regress, predict, lincom, test, ttest, and estat vif";
pub const SCRIPT_UNSUPPORTED_MESSAGE: &str = "--json successful script results are currently supported only for use, describe, count, codebook, summarize, missing, tabulate, generate, replace, rename, recode, label, encode, decode, sql, keep/drop/select variable lists, sort, gsort, save, export, head, tail, regress, predict, lincom, test, ttest, and estat vif";

/// Whether this command has a successful-result envelope in the batch CLI.
pub fn supports_batch_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Sql { .. }
      | Command::Encode { .. }
      | Command::Decode { .. }
      | Command::Label { .. }
      | Command::Recode { .. }
      | Command::Rename { .. }
      | Command::Sort { .. }
      | Command::Gsort { .. }
      | Command::Use { .. }
      | Command::Generate { .. }
      | Command::Replace { .. }
      | Command::Keep { .. }
      | Command::Drop { .. }
      | Command::Select { .. }
      | Command::Save { .. }
      | Command::Export { .. }
      | Command::Describe
      | Command::Count
      | Command::Codebook { .. }
      | Command::Summarize { .. }
      | Command::Missing { .. }
      | Command::Tabulate { .. }
      | Command::Head { .. }
      | Command::Tail { .. }
      | Command::Regress { .. }
      | Command::Predict { .. }
      | Command::Lincom { .. }
      | Command::Test { .. }
      | Command::Ttest { .. }
      | Command::Estat {
        command: EstatCommand {
          subcommand: EstatSubcommand::Vif,
          ..
        },
      }
      | Command::Exit
  )
}

/// Whether a script command can execute without producing an unsupported JSON
/// success result. `run` is a container for nested commands and has no result
/// envelope of its own.
pub fn supports_script_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Sql { .. }
      | Command::Encode { .. }
      | Command::Decode { .. }
      | Command::Label { .. }
      | Command::Recode { .. }
      | Command::Rename { .. }
      | Command::Sort { .. }
      | Command::Gsort { .. }
      | Command::Use { .. }
      | Command::Generate { .. }
      | Command::Replace { .. }
      | Command::Keep { .. }
      | Command::Drop { .. }
      | Command::Select { .. }
      | Command::Save { .. }
      | Command::Export { .. }
      | Command::Describe
      | Command::Count
      | Command::Codebook { .. }
      | Command::Summarize { .. }
      | Command::Missing { .. }
      | Command::Tabulate { .. }
      | Command::Head { .. }
      | Command::Tail { .. }
      | Command::Regress { .. }
      | Command::Predict { .. }
      | Command::Lincom { .. }
      | Command::Test { .. }
      | Command::Ttest { .. }
      | Command::Estat {
        command: EstatCommand {
          subcommand: EstatSubcommand::Vif,
          ..
        },
      }
      | Command::Exit
      | Command::Run { .. }
  )
}

/// Serialize supported execution results using the stable versioned CLI envelope.
///
/// `None` means the corresponding command must be rejected before execution by
/// [`supports_batch_command`].
pub fn serialize_execution_result(result: &ExecutionResult) -> Option<String> {
  let (data, result_type) = match result {
    ExecutionResult::Load(load) => (load_result_data(load), "LoadResult"),
    ExecutionResult::SqlCreate(result) => (
      json!({"table_name": result.table_name, "dataset": dataset_info_data(&result.dataset)}),
      "SqlCreateResult",
    ),
    ExecutionResult::Activate(result) => (
      json!({"table_name": result.table_name, "dataset": dataset_info_data(&result.dataset)}),
      "ActivateResult",
    ),
    ExecutionResult::Save(result) => (
      persistence_result_data(&result.path, &result.dataset),
      "SaveResult",
    ),
    ExecutionResult::Export(result) => (
      persistence_result_data(&result.path, &result.dataset),
      "ExportResult",
    ),
    ExecutionResult::Generate(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "GenerateResult",
    ),
    ExecutionResult::Replace(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "ReplaceResult",
    ),
    ExecutionResult::Keep(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "KeepResult",
    ),
    ExecutionResult::Drop(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DropResult",
    ),
    ExecutionResult::Select(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "SelectResult",
    ),
    ExecutionResult::Encode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "EncodeResult",
    ),
    ExecutionResult::Decode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DecodeResult",
    ),
    ExecutionResult::Label(result) => (
      json!({"action": result.action, "message": result.message, "metadata": result.metadata}),
      "LabelResult",
    ),
    ExecutionResult::Recode(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "RecodeResult",
    ),
    ExecutionResult::Rename(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "RenameResult",
    ),
    ExecutionResult::Sort(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "SortResult",
    ),
    ExecutionResult::Gsort(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "GsortResult",
    ),
    ExecutionResult::Describe(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "DescribeResult",
    ),
    ExecutionResult::Count(count) => (json!({ "row_count": count.row_count }), "CountResult"),
    ExecutionResult::Codebook(result) => (codebook_result_data(result), "CodebookResult"),
    ExecutionResult::Summarize(summary) => (summarize_result_data(summary), "SummarizeResult"),
    ExecutionResult::Missing(result) => (missing_result_data(result), "MissingResult"),
    ExecutionResult::Tabulate(table) => (
      table_result_data(&table.headers, &table.rows),
      "TabulateResult",
    ),
    ExecutionResult::Table(table) => (
      table_result_data(&table.headers, &table.rows),
      "TableResult",
    ),
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      (preview_result_data(preview), "PreviewResult")
    }
    ExecutionResult::Regression(result) => (regression_result_data(result), "RegressionResult"),
    ExecutionResult::Ttest(result) => (ttest_result_data(result), "TtestResult"),
    ExecutionResult::Prediction(result) => (
      json!({"dataset": dataset_info_data(&result.dataset)}),
      "PredictionResult",
    ),
    ExecutionResult::Lincom(result) => (
      json!({
        "label": result.label, "estimate": Number::from_f64(result.estimate),
        "standard_error": Number::from_f64(result.standard_error), "statistic": Number::from_f64(result.statistic),
        "p_value": Number::from_f64(result.p_value), "ci_lower": Number::from_f64(result.ci_lower),
        "ci_upper": Number::from_f64(result.ci_upper), "ci_level": Number::from_f64(result.ci_level), "df_residual": result.df_residual,
      }),
      "LincomResult",
    ),
    ExecutionResult::Test(result) => (
      json!({
        "constraints": result.constraints, "statistic": Number::from_f64(result.statistic),
        "p_value": Number::from_f64(result.p_value), "df": result.df,
        "df_residual": result.df_residual, "is_chi2": result.is_chi2,
      }),
      "TestResult",
    ),
    _ => return None,
  };

  let envelope = ResultEnvelope::new(data, result_type);
  Some(serde_json::to_string(&envelope).expect("JSON value envelopes serialize infallibly"))
}

fn persistence_result_data(path: &std::path::Path, dataset: &DatasetInfo) -> Value {
  json!({"path": path.to_string_lossy(), "dataset": dataset_info_data(dataset)})
}

fn table_result_data(headers: &[String], rows: &[Vec<CellValue>]) -> Value {
  json!({
    "headers": headers,
    "rows": rows.iter().map(|row| row.iter().map(cell_value_data).collect::<Vec<_>>()).collect::<Vec<_>>(),
  })
}

fn codebook_result_data(result: &CodebookResult) -> Value {
  json!({
    "missing_definition": "sql_null", "example_limit": 3, "examples_nonfinite_encoding": "null",
    "rows": result.rows.iter().map(|row| json!({
      "variable": row.variable, "data_type": row.data_type,
      "nonmissing": row.nonmissing, "missing": row.missing, "distinct": row.distinct,
      "examples": row.examples.iter().map(cell_value_data).collect::<Vec<_>>(),
    })).collect::<Vec<_>>()
  })
}

fn missing_result_data(result: &MissingResult) -> Value {
  json!({
    "missing_definition": "sql_null",
    "rows": result.rows.iter().map(|row| json!({
      "variable": row.variable, "data_type": row.data_type, "total": row.total,
      "missing": row.missing, "nonmissing": row.nonmissing,
      "missing_percent": Number::from_f64(row.missing_percent),
    })).collect::<Vec<_>>()
  })
}

fn ttest_group_data(group: &TtestGroupStats, count: Option<usize>) -> Value {
  json!({
    "name": group.name, "count": count,
    "mean": Number::from_f64(group.mean), "standard_error": Number::from_f64(group.standard_error),
    "standard_deviation": Number::from_f64(group.standard_deviation),
    "ci_lower": Number::from_f64(group.ci_lower), "ci_upper": Number::from_f64(group.ci_upper),
  })
}

fn ttest_result_data(result: &TtestResult) -> Value {
  // Independent differences store legacy residual-df/zero, not an observation
  // count. Never expose that field as a sample size in the CLI contract.
  let difference_count = result
    .by_variable
    .is_none()
    .then_some(result.difference_stats.observations);
  json!({
    "varname1": result.varname1, "varname2": result.varname2, "by_variable": result.by_variable,
    "is_paired": result.is_paired, "is_welch": result.is_welch,
    "null_value": Number::from_f64(result.null_value), "reference_distribution": "student_t", "ci_level": 95.0,
    "group1_stats": ttest_group_data(&result.group1_stats, Some(result.group1_stats.observations)),
    "group2_stats": result.group2_stats.as_ref().map(|group| ttest_group_data(group, Some(group.observations))),
    "combined_stats": result.combined_stats.as_ref().map(|group| ttest_group_data(group, Some(group.observations))),
    "difference_stats": ttest_group_data(&result.difference_stats, difference_count),
    "t_statistic": Number::from_f64(result.t_statistic), "degrees_of_freedom": Number::from_f64(result.degrees_of_freedom),
    "p_left": Number::from_f64(result.p_left), "p_two": Number::from_f64(result.p_two), "p_right": Number::from_f64(result.p_right),
  })
}

fn regression_result_data(result: &RegressionResult) -> Value {
  json!({
    "outcome": result.outcome, "predictors": result.predictors,
    "estimator": result.estimator, "covariance": result.covariance,
    "observation_count": result.observation_count, "include_intercept": result.include_intercept,
    "df_residual": result.least_squares.fit_statistics.degrees_of_freedom,
    "r_squared": result.r_squared.and_then(Number::from_f64),
    "adjusted_r_squared": result.adjusted_r_squared.and_then(Number::from_f64),
    "root_mse": result.root_mse.and_then(Number::from_f64),
    "sample": result.least_squares.sample.as_ref().map(|sample| json!({
      "total_observations": sample.total_observations, "dropped_observations": sample.dropped_observations,
    })),
    "coefficients": result.coefficients.iter().map(|coefficient| json!({
      "name": coefficient.name, "value": Number::from_f64(coefficient.value),
      "standard_error": coefficient.standard_error.and_then(Number::from_f64),
      "statistic": coefficient.statistic.and_then(Number::from_f64),
      "p_value": coefficient.p_value.and_then(Number::from_f64),
    })).collect::<Vec<_>>()
  })
}

fn summarize_result_data(result: &SummarizeResult) -> Value {
  json!({
    "rows": result.rows.iter().map(|row| json!({
      "variable": row.variable,
      "count": row.count,
      "mean": row.mean.and_then(Number::from_f64),
      "std_dev": row.std_dev.and_then(Number::from_f64),
      "minimum": row.minimum.as_ref().map(cell_value_data),
      "maximum": row.maximum.as_ref().map(cell_value_data),
    })).collect::<Vec<_>>()
  })
}

fn preview_result_data(result: &PreviewResult) -> Value {
  json!({
    "columns": result.columns,
    "rows": result.rows.iter().map(|row| row.iter().map(cell_value_data).collect::<Vec<_>>()).collect::<Vec<_>>(),
  })
}

fn cell_value_data(value: &CellValue) -> Value {
  match value {
    CellValue::Null => Value::Null,
    CellValue::Boolean(value) => json!(value),
    CellValue::SignedInteger(value) => Value::Number(
      Number::from_i128(*value).expect("arbitrary precision is enabled for JSON results"),
    ),
    CellValue::UnsignedInteger(value) => Value::Number(
      Number::from_u128(*value).expect("arbitrary precision is enabled for JSON results"),
    ),
    CellValue::Float(value) => Number::from_f64(*value).map_or(Value::Null, Value::Number),
    CellValue::Decimal { scale, value, .. } => Value::String(decimal_string(*value, *scale)),
    CellValue::Text(value) => Value::String(value.clone()),
    CellValue::Bytes(value) => Value::String(format!("base64:{}", BASE64.encode(value))),
  }
}

fn decimal_string(value: i128, scale: u8) -> String {
  let scale = usize::from(scale);
  let digits = value.unsigned_abs().to_string();
  let formatted = if scale == 0 {
    digits
  } else if digits.len() <= scale {
    format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
  } else {
    let split = digits.len() - scale;
    format!("{}.{}", &digits[..split], &digits[split..])
  };

  if value.is_negative() {
    format!("-{formatted}")
  } else {
    formatted
  }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expression_results_encode_owned_metadata_without_affected_or_overflow_counts() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "new\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Generate(tabdat_domain::GenerateResult {
          dataset: dataset.clone(),
        }),
        "GenerateResult",
      ),
      (
        ExecutionResult::Replace(tabdat_domain::ReplaceResult {
          dataset: dataset.clone(),
        }),
        "ReplaceResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
  }

  #[test]
  fn projection_results_encode_owned_metadata_and_admit_only_variable_lists() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.parquet"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "wide\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Keep(tabdat_domain::KeepResult {
          dataset: dataset.clone(),
        }),
        "KeepResult",
      ),
      (
        ExecutionResult::Drop(tabdat_domain::DropResult {
          dataset: dataset.clone(),
        }),
        "DropResult",
      ),
      (
        ExecutionResult::Select(tabdat_domain::SelectResult {
          dataset: dataset.clone(),
        }),
        "SelectResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
    for source in ["keep x", "drop x", "select x x"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    for source in ["keep if x > 0", "drop if x == null"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
  }

  #[test]
  fn ordering_results_encode_owned_metadata_without_inventing_keys_or_counts() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "sort\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, tag) in [
      (
        ExecutionResult::Sort(tabdat_domain::SortResult {
          dataset: dataset.clone(),
        }),
        "SortResult",
      ),
      (
        ExecutionResult::Gsort(tabdat_domain::GsortResult {
          dataset: dataset.clone(),
        }),
        "GsortResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], tag);
      assert_eq!(
        value["data"],
        json!({"dataset": dataset_info_data(&dataset)})
      );
    }
    for source in ["sort x x", "gsort -x +y", "gsort `-x`"] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
    let command = tabdat_language::parse_command("by x: sort y").unwrap();
    assert!(!supports_batch_command(&command));
    assert!(!supports_script_command(&command));
  }

  #[test]
  fn rename_result_encodes_only_owned_post_operation_metadata() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "renamed\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Rename(tabdat_domain::RenameResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "RenameResult");
    assert_eq!(
      value["data"],
      json!({"dataset": dataset_info_data(&dataset)})
    );
    let command = tabdat_language::parse_command("rename old new").unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }

  #[test]
  fn sql_creation_and_activation_report_owned_names_and_metadata() {
    let dataset = DatasetInfo {
      source: "cohort\t目录".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "x\n".into(),
        data_type: "DECIMAL(38,4)".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, name) in [
      (
        ExecutionResult::SqlCreate(tabdat_domain::SqlCreateResult {
          table_name: "cohort\t目录".into(),
          dataset: dataset.clone(),
        }),
        "SqlCreateResult",
      ),
      (
        ExecutionResult::Activate(tabdat_domain::ActivateResult {
          table_name: "cohort\t目录".into(),
          dataset: dataset.clone(),
        }),
        "ActivateResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":name,"data":{"table_name":"cohort\t目录","dataset":dataset_info_data(&dataset)}})
      );
    }
    for source in [
      "sql select 1",
      "sql with q as (select 1 as x) select x from q into cohort",
      "use cohort",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn category_coding_reports_owned_schema_without_dictionary_or_guessed_counts() {
    let dataset = DatasetInfo {
      source: "source\t目录.csv".into(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "code\n".into(),
        data_type: "BIGINT".into(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, name) in [
      (
        ExecutionResult::Encode(tabdat_domain::EncodeResult {
          dataset: dataset.clone(),
        }),
        "EncodeResult",
      ),
      (
        ExecutionResult::Decode(tabdat_domain::DecodeResult {
          dataset: dataset.clone(),
        }),
        "DecodeResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":name,"data":{"dataset":dataset_info_data(&dataset)}})
      );
    }
    for source in [
      "encode x, generate(code)",
      "encode x, generate(code) label(codes)",
      "decode code, generate(text)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn label_results_preserve_owned_tagged_keys_order_and_absent_metadata() {
    use tabdat_domain::{LabelMetadata, LabelResult, LabelResultAction, LabelValue, ValueLabelSet};
    let metadata = LabelMetadata {
      variable_labels: vec![
        ("z\t列".into(), "Title\n\u{1b}".into()),
        ("a".into(), "A".into()),
      ],
      value_sets: vec![ValueLabelSet {
        name: "codes".into(),
        mappings: vec![
          (LabelValue::Integer(i64::MAX), "wide".into()),
          (
            LabelValue::Number("9007199254740993.001".into()),
            "exact".into(),
          ),
          (LabelValue::Text("1".into()), "text".into()),
          (LabelValue::Integer(i64::MIN), "negative".into()),
        ],
      }],
      attachments: vec![("z\t列".into(), "codes".into())],
    };
    for (action, spelling) in [
      (LabelResultAction::Variable, "variable"),
      (LabelResultAction::Define, "define"),
      (LabelResultAction::Values, "values"),
      (LabelResultAction::List, "list"),
      (LabelResultAction::Drop, "drop"),
    ] {
      let result = ExecutionResult::Label(LabelResult {
        action,
        message: "Done\n".into(),
        metadata: Some(metadata.clone()),
      });
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(
        value,
        json!({"schema_version":1,"result_type":"LabelResult","data":{
          "action":spelling,"message":"Done\n","metadata":{
            "variable_labels":[["z\t列","Title\n\u{1b}"],["a","A"]],
            "value_sets":[{"name":"codes","mappings":[
              [{"kind":"integer","payload":i64::MAX},"wide"],
              [{"kind":"number","payload":"9007199254740993.001"},"exact"],
              [{"kind":"text","payload":"1"},"text"],
              [{"kind":"integer","payload":i64::MIN},"negative"]
            ]}],"attachments":[["z\t列","codes"]]
          }
        }})
      );
    }
    let result = ExecutionResult::Label(LabelResult {
      action: LabelResultAction::List,
      message: "Label dictionary".into(),
      metadata: None,
    });
    let value: Value = serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(
      value["data"],
      json!({"action":"list","message":"Label dictionary","metadata":null})
    );
    for source in [
      "label variable x 'X'",
      "label define codes 1 'one'",
      "label values x codes",
      "label list codes",
      "label drop codes",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn recode_result_encodes_owned_metadata_without_rule_or_changed_cell_fields() {
    let dataset = DatasetInfo {
      source: std::path::PathBuf::from("source\t目录.csv"),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "recoded\n".to_owned(),
        data_type: "DECIMAL(38,0)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    let result = ExecutionResult::Recode(tabdat_domain::RecodeResult {
      dataset: dataset.clone(),
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "RecodeResult");
    assert_eq!(
      value["data"],
      json!({"dataset": dataset_info_data(&dataset)})
    );
    for source in [
      "recode x (1=2), replace",
      "recode x (missing=0), generate(flag)",
    ] {
      let command = tabdat_language::parse_command(source).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn persistence_json_encodes_requested_paths_and_owned_output_metadata() {
    let path = std::path::PathBuf::from("目录/out\t\".parquet");
    let dataset = DatasetInfo {
      source: path.clone(),
      row_count: u64::MAX,
      columns: vec![tabdat_domain::ColumnInfo {
        name: "wide\n".to_owned(),
        data_type: "DECIMAL(38,2)".to_owned(),
      }],
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
    };
    for (result, result_type) in [
      (
        ExecutionResult::Save(tabdat_domain::SaveResult {
          path: path.clone(),
          dataset: dataset.clone(),
        }),
        "SaveResult",
      ),
      (
        ExecutionResult::Export(tabdat_domain::ExportResult {
          path: path.clone(),
          dataset: dataset.clone(),
        }),
        "ExportResult",
      ),
    ] {
      let serialized = serialize_execution_result(&result).unwrap();
      assert_eq!(serialized.lines().count(), 1);
      let value: Value = serde_json::from_str(&serialized).unwrap();
      assert_eq!(value["schema_version"], 1);
      assert_eq!(value["result_type"], result_type);
      assert_eq!(value["data"]["path"], "目录/out\t\".parquet");
      assert_eq!(value["data"]["dataset"], dataset_info_data(&dataset));
      assert_eq!(value["data"]["dataset"]["row_count"], u64::MAX);
    }
    for input in [
      "save out.parquet",
      "save out.parquet, replace",
      "export out.csv",
      "export out.csv, replace",
    ] {
      let command = tabdat_language::parse_command(input).unwrap();
      assert!(supports_batch_command(&command));
      assert!(supports_script_command(&command));
    }
  }

  #[test]
  fn describe_json_preserves_owned_metadata_with_existing_placeholder_policy() {
    let result = ExecutionResult::Describe(tabdat_domain::DescribeResult {
      dataset: DatasetInfo {
        source: std::path::PathBuf::from("目录/source\t\".parquet"),
        row_count: u64::MAX,
        columns: vec![
          tabdat_domain::ColumnInfo {
            name: "é\t\"\n".to_owned(),
            data_type: "DECIMAL(38,2)".to_owned(),
          },
          tabdat_domain::ColumnInfo {
            name: "wide".to_owned(),
            data_type: "UHUGEINT".to_owned(),
          },
        ],
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
      },
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "DescribeResult");
    assert_eq!(
      value["data"]["dataset"],
      json!({
        "path": "目录/source\t\".parquet", "row_count": u64::MAX,
        "columns": [{"name": "é\t\"\n", "data_type": "DECIMAL(38,2)"}, {"name": "wide", "data_type": "UHUGEINT"}],
        "execution_mode": "eager", "lazy_engine": null,
        "label_metadata": null, "panel_metadata": null,
      })
    );
    assert!(supports_batch_command(&Command::Describe));
    assert!(supports_script_command(&Command::Describe));
  }

  #[test]
  fn owned_tables_encode_cells_without_reinterpreting_vif_labels() {
    let result = ExecutionResult::Table(tabdat_domain::TableResult {
      headers: vec!["Variable\n".to_owned(), "VIF".to_owned()],
      rows: vec![
        vec![
          CellValue::Text("mean_vif".to_owned()),
          CellValue::Float(0.0),
        ],
        vec![
          CellValue::Text("mean_vif".to_owned()),
          CellValue::Float(f64::INFINITY),
        ],
        vec![CellValue::Text("label\t\"\n".to_owned()), CellValue::Null],
        vec![
          CellValue::Float(f64::NAN),
          CellValue::Float(f64::NEG_INFINITY),
        ],
        vec![
          CellValue::SignedInteger(i128::MAX),
          CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          },
        ],
      ],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["result_type"], "TableResult");
    assert_eq!(value["data"]["headers"], json!(["Variable\n", "VIF"]));
    assert_eq!(
      value["data"]["rows"],
      json!([
        ["mean_vif", 0.0],
        ["mean_vif", null],
        ["label\t\"\n", null],
        [null, null],
        [i128::MAX, "-0.01"],
      ])
    );
    for input in ["estat residuals", "by x: summarize x"] {
      let command = tabdat_language::parse_command(input).unwrap();
      assert!(!supports_batch_command(&command));
      assert!(!supports_script_command(&command));
    }
    let command = tabdat_language::parse_command("estat vif").unwrap();
    assert!(supports_batch_command(&command));
    assert!(supports_script_command(&command));
  }

  #[test]
  fn codebook_json_uses_existing_scalar_policy_without_loss_of_owned_labels_or_counts() {
    let row = |variable: &str, data_type: &str, examples| tabdat_domain::CodebookRow {
      variable: variable.to_owned(),
      data_type: data_type.to_owned(),
      nonmissing: u64::MAX,
      missing: 0,
      distinct: u64::MAX,
      examples,
    };
    let result = ExecutionResult::Codebook(CodebookResult {
      rows: vec![
        row(
          "signed",
          "HUGEINT",
          vec![
            CellValue::SignedInteger(i128::MIN),
            CellValue::SignedInteger(i128::MAX),
          ],
        ),
        row(
          "unsigned",
          "UHUGEINT",
          vec![CellValue::UnsignedInteger(u128::MAX)],
        ),
        row(
          "decimal",
          "DECIMAL(38,2)",
          vec![CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          }],
        ),
        row("blob", "BLOB", vec![CellValue::Bytes(vec![0, 255])]),
        row(
          "label\t\"\n",
          "VARCHAR",
          vec![CellValue::Text("text\t\"\n".to_owned())],
        ),
        row(
          "bool",
          "BOOLEAN",
          vec![CellValue::Boolean(true), CellValue::Boolean(false)],
        ),
        row(
          "nonfinite",
          "DOUBLE",
          vec![
            CellValue::Float(f64::NAN),
            CellValue::Float(f64::INFINITY),
            CellValue::Float(f64::NEG_INFINITY),
          ],
        ),
      ],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let value: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(value["result_type"], "CodebookResult");
    let rows = value["data"]["rows"].as_array().unwrap();
    assert!(
      rows
        .iter()
        .all(|row| row["nonmissing"] == u64::MAX && row["distinct"] == u64::MAX)
    );
    for (index, expected) in [
      json!([i128::MIN, i128::MAX]),
      json!([u128::MAX]),
      json!(["-0.01"]),
      json!(["base64:AP8="]),
      json!(["text\t\"\n"]),
      json!([true, false]),
      json!([null, null, null]),
    ]
    .iter()
    .enumerate()
    {
      assert_eq!(rows[index]["examples"], *expected);
    }
    assert_eq!(rows[4]["variable"], "label\t\"\n");
    let empty = ExecutionResult::Codebook(CodebookResult { rows: vec![] });
    let value: Value = serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(value["data"]["rows"], json!([]));
  }

  #[test]
  fn missing_json_preserves_large_counts_labels_null_percent_and_empty_rows() {
    let result = ExecutionResult::Missing(MissingResult {
      rows: vec![tabdat_domain::MissingRow {
        variable: "label\t\"\n".to_owned(),
        data_type: "DECIMAL(38,2)".to_owned(),
        total: u64::MAX,
        missing: 1,
        nonmissing: u64::MAX - 1,
        missing_percent: f64::NAN,
      }],
    });
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let envelope: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
      envelope,
      json!({"schema_version": 1, "result_type": "MissingResult", "data": {
        "missing_definition": "sql_null", "rows": [{"variable": "label\t\"\n", "data_type": "DECIMAL(38,2)",
        "total": u64::MAX, "missing": 1, "nonmissing": u64::MAX - 1, "missing_percent": null}]
      }})
    );
    let empty = ExecutionResult::Missing(MissingResult { rows: vec![] });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(
      envelope["data"],
      json!({"missing_definition": "sql_null", "rows": []})
    );
  }

  #[test]
  fn ttest_json_boundary_preserves_labels_zero_and_null_policies() {
    let group = TtestGroupStats {
      name: "label\tline\n\"quoted\"".to_owned(),
      observations: 3,
      mean: f64::NAN,
      standard_error: 0.0,
      standard_deviation: f64::INFINITY,
      ci_lower: f64::NEG_INFINITY,
      ci_upper: 2.0,
    };
    // Synthetic owned values exercise the encoder, not a fitted inference case.
    let result = ExecutionResult::Ttest(Box::new(TtestResult {
      varname1: "x".to_owned(),
      varname2: None,
      by_variable: Some("grp".to_owned()),
      is_paired: false,
      is_welch: false,
      null_value: 0.0,
      group1_stats: group.clone(),
      group2_stats: Some(TtestGroupStats {
        name: "B".to_owned(),
        ..group.clone()
      }),
      combined_stats: Some(TtestGroupStats {
        name: "combined".to_owned(),
        observations: 6,
        ..group.clone()
      }),
      difference_stats: TtestGroupStats {
        name: "diff".to_owned(),
        observations: 4,
        ..group
      },
      t_statistic: f64::INFINITY,
      degrees_of_freedom: 4.0,
      p_left: 0.0,
      p_two: f64::INFINITY,
      p_right: f64::NAN,
    }));
    let serialized = serialize_execution_result(&result).unwrap();
    assert_eq!(serialized.lines().count(), 1);
    let envelope: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(envelope["result_type"], "TtestResult");
    let data = &envelope["data"];
    assert_eq!(
      data["group1_stats"],
      json!({
        "name": "label\tline\n\"quoted\"", "count": 3, "mean": null,
        "standard_error": 0.0, "standard_deviation": null, "ci_lower": null, "ci_upper": 2.0,
      })
    );
    assert_eq!(data["combined_stats"]["count"], 6);
    assert!(data["difference_stats"]["count"].is_null());
    assert_eq!(data["null_value"], 0.0);
    assert_eq!(data["p_left"], 0.0);
    assert!(data["p_two"].is_null());
    assert!(data["p_right"].is_null());
    assert!(data["t_statistic"].is_null());
    assert_eq!(data["degrees_of_freedom"], 4.0);
  }

  #[test]
  fn tabulate_serialization_preserves_owned_labels_scalar_policies_and_empty_rows() {
    let result = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "label\tcategory".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![
        vec![
          CellValue::Text("label\n\"quoted\"".to_owned()),
          CellValue::SignedInteger(i128::MAX),
          CellValue::Float(50.0),
        ],
        vec![
          CellValue::Null,
          CellValue::SignedInteger(1),
          CellValue::Float(f64::NAN),
        ],
      ],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(
      envelope,
      json!({"schema_version": 1, "result_type": "TabulateResult", "data": {
        "headers": ["label\tcategory", "Count", "Percent"], "rows": [["label\n\"quoted\"", i128::MAX, 50.0], [null, 1, null]]
      }})
    );
    let empty = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec!["cat".to_owned(), "Count".to_owned(), "Percent".to_owned()],
      rows: vec![],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&empty).unwrap()).unwrap();
    assert_eq!(
      envelope["data"],
      json!({"headers": ["cat", "Count", "Percent"], "rows": []})
    );
  }

  #[test]
  fn summary_serialization_preserves_order_precision_and_null_policies() {
    use tabdat_domain::SummaryRow;
    let large = SummaryRow {
      variable: "large".to_owned(),
      count: u64::MAX,
      mean: None,
      std_dev: None,
      minimum: Some(CellValue::SignedInteger(i128::MIN)),
      maximum: Some(CellValue::UnsignedInteger(u128::MAX)),
    };
    let result = ExecutionResult::Summarize(SummarizeResult {
      rows: vec![
        large.clone(),
        SummaryRow {
          variable: "empty".to_owned(),
          count: 0,
          mean: None,
          std_dev: None,
          minimum: None,
          maximum: None,
        },
        SummaryRow {
          variable: "decimal".to_owned(),
          count: 1,
          mean: Some(0.01),
          std_dev: None,
          minimum: Some(CellValue::Decimal {
            width: 38,
            scale: 2,
            value: 1,
          }),
          maximum: Some(CellValue::Decimal {
            width: 38,
            scale: 2,
            value: 1,
          }),
        },
        SummaryRow {
          variable: "nonfinite".to_owned(),
          count: 2,
          mean: Some(f64::INFINITY),
          std_dev: Some(f64::NAN),
          minimum: Some(CellValue::Float(f64::NEG_INFINITY)),
          maximum: Some(CellValue::Float(f64::INFINITY)),
        },
        large,
      ],
    });
    let envelope: Value =
      serde_json::from_str(&serialize_execution_result(&result).unwrap()).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["result_type"], "SummarizeResult");
    let rows = envelope["data"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], rows[4]);
    assert_eq!(rows[0]["count"], json!(u64::MAX));
    assert_eq!(rows[0]["minimum"], json!(i128::MIN));
    assert_eq!(rows[0]["maximum"], json!(u128::MAX));
    assert_eq!(
      rows[1],
      json!({"variable": "empty", "count": 0, "mean": null, "std_dev": null, "minimum": null, "maximum": null})
    );
    assert_eq!(
      rows[2],
      json!({"variable": "decimal", "count": 1, "mean": 0.01, "std_dev": null, "minimum": "0.01", "maximum": "0.01"})
    );
    assert_eq!(
      rows[3],
      json!({"variable": "nonfinite", "count": 2, "mean": null, "std_dev": null, "minimum": null, "maximum": null})
    );
  }

  #[test]
  fn preview_serialization_preserves_cell_conversion_policies() {
    let result = ExecutionResult::Head(PreviewResult {
      columns: vec![
        "null".to_owned(),
        "boolean".to_owned(),
        "signed".to_owned(),
        "unsigned".to_owned(),
        "finite".to_owned(),
        "nan".to_owned(),
        "positive_infinity".to_owned(),
        "negative_infinity".to_owned(),
        "decimal".to_owned(),
        "small_decimal".to_owned(),
        "zero_decimal".to_owned(),
        "text".to_owned(),
        "bytes".to_owned(),
        "empty_bytes".to_owned(),
      ],
      rows: vec![vec![
        CellValue::Null,
        CellValue::Boolean(true),
        CellValue::SignedInteger(i128::MIN),
        CellValue::UnsignedInteger(u128::MAX),
        CellValue::Float(1.25),
        CellValue::Float(f64::NAN),
        CellValue::Float(f64::INFINITY),
        CellValue::Float(f64::NEG_INFINITY),
        CellValue::Decimal {
          width: 8,
          scale: 2,
          value: -120,
        },
        CellValue::Decimal {
          width: 5,
          scale: 3,
          value: 7,
        },
        CellValue::Decimal {
          width: 3,
          scale: 0,
          value: 0,
        },
        CellValue::Text("λ".to_owned()),
        CellValue::Bytes(vec![0xff, 0x00]),
        CellValue::Bytes(Vec::new()),
      ]],
    });

    let serialized = serialize_execution_result(&result).expect("preview is supported");
    let envelope: Value = serde_json::from_str(&serialized).expect("valid JSON");

    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["result_type"], "PreviewResult");
    assert_eq!(envelope["data"]["columns"][0], "null");
    let row = &envelope["data"]["rows"][0];
    assert_eq!(row[0], Value::Null);
    assert_eq!(row[1], true);
    assert_eq!(
      row[2].as_number().unwrap().to_string(),
      i128::MIN.to_string()
    );
    assert_eq!(
      row[3].as_number().unwrap().to_string(),
      u128::MAX.to_string()
    );
    assert_eq!(row[4], 1.25);
    assert_eq!(row[5], Value::Null);
    assert_eq!(row[6], Value::Null);
    assert_eq!(row[7], Value::Null);
    assert_eq!(row[8], "-1.20");
    assert_eq!(row[9], "0.007");
    assert_eq!(row[10], "0");
    assert_eq!(row[11], "λ");
    assert_eq!(row[12], "base64:/wA=");
    assert_eq!(row[13], "base64:");
    assert!(!serialized.contains("NaN"));
    assert!(!serialized.contains("Infinity"));
  }

  #[test]
  fn tail_and_empty_previews_use_the_same_result_envelope() {
    let result = ExecutionResult::Tail(PreviewResult {
      columns: vec!["x".to_owned()],
      rows: Vec::new(),
    });

    let serialized = serialize_execution_result(&result).expect("tail preview is supported");
    let envelope: Value = serde_json::from_str(&serialized).expect("valid JSON");

    assert_eq!(envelope["result_type"], "PreviewResult");
    assert_eq!(envelope["data"], json!({"columns": ["x"], "rows": []}));
  }
}
