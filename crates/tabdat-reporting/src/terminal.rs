use std::fmt::Write;

use tabdat_domain::{CellValue, ExecutionResult};

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

/// Render the initial interactive EDA subset without changing owned results.
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
