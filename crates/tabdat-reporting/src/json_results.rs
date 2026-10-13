//! JSON result envelopes for the bounded CLI execution surface.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Number, Value, json};
use tabdat_domain::{
  CellValue, DatasetInfo, ExecutionResult, LoadResult, PreviewResult, RegressionResult,
  SummarizeResult,
};
use tabdat_language::{Command, ExecutionMode, LazyEngine};

use crate::catalog::ResultEnvelope;

pub const BATCH_UNSUPPORTED_MESSAGE: &str = "--json successful batch results are currently supported only for use, count, summarize, tabulate, head, tail, regress, predict, lincom, and test";
pub const SCRIPT_UNSUPPORTED_MESSAGE: &str = "--json successful script results are currently supported only for use, count, summarize, tabulate, head, tail, regress, predict, lincom, and test";

/// Whether this command has a successful-result envelope in the batch CLI.
pub fn supports_batch_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Use { .. }
      | Command::Count
      | Command::Summarize { .. }
      | Command::Tabulate { .. }
      | Command::Head { .. }
      | Command::Tail { .. }
      | Command::Regress { .. }
      | Command::Predict { .. }
      | Command::Lincom { .. }
      | Command::Test { .. }
      | Command::Exit
  )
}

/// Whether a script command can execute without producing an unsupported JSON
/// success result. `run` is a container for nested commands and has no result
/// envelope of its own.
pub fn supports_script_command(command: &Command) -> bool {
  matches!(
    command,
    Command::Use { .. }
      | Command::Count
      | Command::Summarize { .. }
      | Command::Tabulate { .. }
      | Command::Head { .. }
      | Command::Tail { .. }
      | Command::Regress { .. }
      | Command::Predict { .. }
      | Command::Lincom { .. }
      | Command::Test { .. }
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
    ExecutionResult::Count(count) => (json!({ "row_count": count.row_count }), "CountResult"),
    ExecutionResult::Summarize(summary) => (summarize_result_data(summary), "SummarizeResult"),
    ExecutionResult::Tabulate(table) => (
      json!({
        "headers": table.headers,
        "rows": table.rows.iter().map(|row| row.iter().map(cell_value_data).collect::<Vec<_>>()).collect::<Vec<_>>(),
      }),
      "TabulateResult",
    ),
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      (preview_result_data(preview), "PreviewResult")
    }
    ExecutionResult::Regression(result) => (regression_result_data(result), "RegressionResult"),
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
