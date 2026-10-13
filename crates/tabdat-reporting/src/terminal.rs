use std::fmt::Write;

use tabdat_domain::{CellValue, DatasetInfo, ExecutionResult};

fn cell(value: &CellValue) -> String {
  match value {
    CellValue::Null => "NA".to_owned(),
    CellValue::Boolean(value) => value.to_string(),
    CellValue::SignedInteger(value) => value.to_string(),
    CellValue::UnsignedInteger(value) => value.to_string(),
    CellValue::Float(value) => value.to_string(),
    CellValue::Text(value) => format!("{value:?}"),
    CellValue::Bytes(value) => {
      let mut text = String::from("0x");
      for byte in value {
        write!(text, "{byte:02x}").unwrap();
      }
      text
    }
    CellValue::Decimal { scale, value, .. } => {
      let mut digits = value.unsigned_abs().to_string();
      let scale = usize::from(*scale);
      if scale > 0 {
        if digits.len() <= scale {
          digits = format!("{}{}", "0".repeat(scale + 1 - digits.len()), digits);
        }
        digits.insert(digits.len() - scale, '.');
      }
      if *value < 0 {
        format!("-{digits}")
      } else {
        digits
      }
    }
  }
}

fn optional_number(value: Option<f64>) -> String {
  value
    .map(|value| value.to_string())
    .unwrap_or_else(|| "NA".to_owned())
}

fn table(columns: &[String], rows: &[Vec<CellValue>]) -> String {
  let mut output = columns
    .iter()
    .map(|name| name.escape_debug().to_string())
    .collect::<Vec<_>>()
    .join("\t");
  output.push('\n');
  for row in rows {
    writeln!(
      output,
      "{}",
      row.iter().map(cell).collect::<Vec<_>>().join("\t")
    )
    .unwrap();
  }
  writeln!(output, "({} rows shown)", rows.len()).unwrap();
  output
}

fn dataset_outcome(action: &str, dataset: &DatasetInfo) -> String {
  format!(
    "{action}: {} rows, {} columns\n",
    dataset.row_count,
    dataset.columns.len()
  )
}

fn inference_number(value: f64) -> String {
  if value.is_nan() {
    "NA".to_owned()
  } else {
    value.to_string()
  }
}

fn optional_inference(value: Option<f64>) -> String {
  value
    .map(inference_number)
    .unwrap_or_else(|| "NA".to_owned())
}

fn regression(result: &tabdat_domain::RegressionResult) -> String {
  let mut output = format!(
    "Regression: {} on {}\nEstimator: {}; covariance: {}; intercept: {}\nN = {}; residual df = {}\n",
    result.outcome.escape_debug(),
    result
      .predictors
      .iter()
      .map(|name| name.escape_debug().to_string())
      .collect::<Vec<_>>()
      .join(" "),
    result.estimator.escape_debug(),
    result.covariance.escape_debug(),
    result.include_intercept,
    result.observation_count,
    result.least_squares.fit_statistics.degrees_of_freedom
  );
  if let Some(sample) = &result.least_squares.sample {
    writeln!(
      output,
      "Candidate rows = {}; excluded = {}",
      sample.total_observations, sample.dropped_observations
    )
    .unwrap();
  }
  writeln!(
    output,
    "R-squared = {}; adjusted R-squared = {}; root MSE = {}",
    optional_inference(result.r_squared),
    optional_inference(result.adjusted_r_squared),
    optional_inference(result.root_mse)
  )
  .unwrap();
  output.push_str("Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)\n");
  for coefficient in &result.coefficients {
    writeln!(
      output,
      "{}\t{}\t{}\t{}\t{}",
      coefficient.name.escape_debug(),
      inference_number(coefficient.value),
      optional_inference(coefficient.standard_error),
      optional_inference(coefficient.statistic),
      optional_inference(coefficient.p_value)
    )
    .unwrap();
  }
  output.push_str("NA = unavailable or undefined.\n");
  output
}

/// Render the supported interactive EDA and linear-analysis results without recomputing statistics.
/// Other executed results receive an explicit display limitation, not silence.
pub fn render_execution_result(result: &ExecutionResult) -> String {
  match result {
    ExecutionResult::Load(load) => format!(
      "Loaded {} rows, {} columns from {}\n",
      load.dataset.row_count,
      load.dataset.columns.len(),
      load.dataset.source.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Describe(describe) => {
      let mut output = format!(
        "{} rows, {} columns\nVariable\tType\n",
        describe.dataset.row_count,
        describe.dataset.columns.len()
      );
      for column in &describe.dataset.columns {
        writeln!(
          output,
          "{}\t{}",
          column.name.escape_debug(),
          column.data_type.escape_debug()
        )
        .unwrap();
      }
      output
    }
    ExecutionResult::Count(count) => format!("{} rows\n", count.row_count),
    ExecutionResult::Summarize(summary) => {
      let mut output = String::from("Variable\tN\tMean\tStd. dev.\tMin\tMax\n");
      for row in &summary.rows {
        writeln!(
          output,
          "{}\t{}\t{}\t{}\t{}\t{}",
          row.variable.escape_debug(),
          row.count,
          optional_number(row.mean),
          optional_number(row.std_dev),
          row
            .minimum
            .as_ref()
            .map(cell)
            .unwrap_or_else(|| "NA".to_owned()),
          row
            .maximum
            .as_ref()
            .map(cell)
            .unwrap_or_else(|| "NA".to_owned())
        )
        .unwrap();
      }
      output
    }
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      table(&preview.columns, &preview.rows)
    }
    ExecutionResult::Table(result) => table(&result.headers, &result.rows),
    ExecutionResult::Tabulate(result) => table(&result.headers, &result.rows),
    ExecutionResult::Generate(result) => dataset_outcome("Generated", &result.dataset),
    ExecutionResult::Replace(result) => dataset_outcome("Replaced", &result.dataset),
    ExecutionResult::Rename(result) => dataset_outcome("Renamed", &result.dataset),
    ExecutionResult::Sort(result) => dataset_outcome("Sorted", &result.dataset),
    ExecutionResult::Gsort(result) => dataset_outcome("Sorted", &result.dataset),
    ExecutionResult::Keep(result) => dataset_outcome("Kept", &result.dataset),
    ExecutionResult::Drop(result) => dataset_outcome("Dropped", &result.dataset),
    ExecutionResult::Select(result) => dataset_outcome("Selected", &result.dataset),
    ExecutionResult::Save(result) => format!(
      "Saved {} rows, {} columns to {}\n",
      result.dataset.row_count,
      result.dataset.columns.len(),
      result.path.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Export(result) => format!(
      "Exported {} rows, {} columns to {}\n",
      result.dataset.row_count,
      result.dataset.columns.len(),
      result.path.to_string_lossy().escape_debug()
    ),
    ExecutionResult::Regression(result) => regression(result),
    ExecutionResult::Prediction(result) => dataset_outcome("Predicted", &result.dataset),
    ExecutionResult::Lincom(result) => format!(
      "Linear combination: {}\nt reference; residual df = {}\nEstimate\tStd. error\tt\tP(two-sided)\tCI lower ({}%)\tCI upper ({}%)\n{}\t{}\t{}\t{}\t{}\t{}\nNA = unavailable or undefined.\n",
      result.label.escape_debug(),
      result
        .df_residual
        .map(|df| df.to_string())
        .unwrap_or_else(|| "NA".to_owned()),
      result.ci_level,
      result.ci_level,
      inference_number(result.estimate),
      inference_number(result.standard_error),
      inference_number(result.statistic),
      inference_number(result.p_value),
      inference_number(result.ci_lower),
      inference_number(result.ci_upper)
    ),
    ExecutionResult::Test(result) => {
      let mut output = String::from("Linear restrictions:\n");
      for constraint in &result.constraints {
        writeln!(output, "  {}", constraint.escape_debug()).unwrap();
      }
      let distribution = if result.is_chi2 {
        format!("chi-square({})", result.df)
      } else {
        format!(
          "F({}, {})",
          result.df,
          result
            .df_residual
            .map(|df| df.to_string())
            .unwrap_or_else(|| "NA".to_owned())
        )
      };
      writeln!(
        output,
        "{distribution} = {}; P(upper-tail) = {}",
        inference_number(result.statistic),
        inference_number(result.p_value)
      )
      .unwrap();
      output.push_str("NA = unavailable or undefined.\n");
      output
    }
    ExecutionResult::Run(run) => format!(
      "Executed {} commands from {}\n",
      run.executed_commands,
      run.path.to_string_lossy().escape_debug()
    ),
    _ => "Command completed; terminal details for this result are not yet supported.\n".to_owned(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tabdat_domain::{SummarizeResult, SummaryRow};

  #[test]
  fn scalar_display_preserves_integer_decimal_and_missing_values() {
    assert_eq!(
      cell(&CellValue::SignedInteger(i128::MIN)),
      i128::MIN.to_string()
    );
    assert_eq!(
      cell(&CellValue::UnsignedInteger(u128::MAX)),
      u128::MAX.to_string()
    );
    for (scale, value, expected) in [
      (2, -1, "-0.01"),
      (2, 0, "0.00"),
      (0, 12, "12"),
      (2, 1234, "12.34"),
    ] {
      assert_eq!(
        cell(&CellValue::Decimal {
          width: 38,
          scale,
          value
        }),
        expected
      );
    }
    assert_eq!(cell(&CellValue::Null), "NA");
    assert_eq!(
      cell(&CellValue::Text("NA\t\n\u{1b}".to_owned())),
      "\"NA\\t\\n\\u{1b}\""
    );
    assert_eq!(cell(&CellValue::Bytes(vec![0, 255])), "0x00ff");
  }

  #[test]
  fn frequency_display_preserves_owned_order_labels_and_empty_tables() {
    let result = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "category".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![
        vec![
          CellValue::Text("label\t10".to_owned()),
          CellValue::SignedInteger(2),
          CellValue::Float(50.0),
        ],
        vec![
          CellValue::SignedInteger(2),
          CellValue::SignedInteger(1),
          CellValue::Float(25.0),
        ],
        vec![
          CellValue::Null,
          CellValue::SignedInteger(1),
          CellValue::Float(25.0),
        ],
      ],
    });
    assert_eq!(
      render_execution_result(&result),
      "category\tCount\tPercent\n\"label\\t10\"\t2\t50\n2\t1\t25\nNA\t1\t25\n(3 rows shown)\n"
    );
    let empty = ExecutionResult::Tabulate(tabdat_domain::TabulateResult {
      headers: vec![
        "category".to_owned(),
        "Count".to_owned(),
        "Percent".to_owned(),
      ],
      rows: vec![],
    });
    assert_eq!(
      render_execution_result(&empty),
      "category\tCount\tPercent\n(0 rows shown)\n"
    );
  }

  #[test]
  fn post_estimation_names_the_reference_and_preserves_undefined_inference() {
    let result = ExecutionResult::Lincom(tabdat_domain::LincomResult {
      label: "constant".to_owned(),
      estimate: 5.0,
      standard_error: 0.0,
      statistic: f64::NAN,
      p_value: f64::NAN,
      ci_lower: 5.0,
      ci_upper: 5.0,
      ci_level: 95.0,
      df_residual: Some(4),
    });
    let text = render_execution_result(&result);
    assert!(text.contains("t reference; residual df = 4"));
    assert!(text.contains("CI lower (95%)\tCI upper (95%)\n5\t0\tNA\tNA\t5\t5\n"));
    assert!(!text.contains("NaN"));
    for (is_chi2, expected) in [(false, "F(2, 4)"), (true, "chi-square(2)")] {
      let test = ExecutionResult::Test(tabdat_domain::TestResult {
        constraints: vec!["x = 0".to_owned()],
        statistic: 3.0,
        p_value: 0.0,
        df: 2,
        df_residual: Some(4),
        is_chi2,
      });
      assert!(
        render_execution_result(&test).contains(&format!("{expected} = 3; P(upper-tail) = 0"))
      );
    }
  }

  #[test]
  fn summary_displays_absent_statistics_without_inventing_zeroes() {
    let result = ExecutionResult::Summarize(SummarizeResult {
      rows: vec![SummaryRow {
        variable: "empty".to_owned(),
        count: 0,
        mean: None,
        std_dev: None,
        minimum: None,
        maximum: None,
      }],
    });
    assert_eq!(
      render_execution_result(&result),
      "Variable\tN\tMean\tStd. dev.\tMin\tMax\nempty\t0\tNA\tNA\tNA\tNA\n"
    );
  }
}
