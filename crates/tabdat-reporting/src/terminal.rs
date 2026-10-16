use std::fmt::Write;

use tabdat_domain::{CellValue, DatasetInfo, ExecutionResult, SummaryRow};

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

fn ttest_sample_row(output: &mut String, sample: &tabdat_domain::TtestGroupStats, n: &str) {
  writeln!(
    output,
    "{}\t{n}\t{}\t{}\t{}\t{}\t{}",
    sample.name.escape_debug(),
    inference_number(sample.mean),
    inference_number(sample.standard_error),
    inference_number(sample.standard_deviation),
    inference_number(sample.ci_lower),
    inference_number(sample.ci_upper)
  )
  .unwrap();
}

fn ttest(result: &tabdat_domain::TtestResult) -> String {
  let independent = result.by_variable.is_some();
  let mode = if result.is_paired {
    "paired"
  } else if independent {
    if result.is_welch {
      "Welch unequal-variance"
    } else {
      "pooled equal-variance"
    }
  } else {
    "one-sample"
  };
  let mut output = format!(
    "T test: {mode}\nVariable: {}",
    result.varname1.escape_debug()
  );
  if let Some(variable) = &result.varname2 {
    write!(output, "; paired variable: {}", variable.escape_debug()).unwrap();
  }
  if let Some(variable) = &result.by_variable {
    write!(output, "; by: {}", variable.escape_debug()).unwrap();
  }
  output.push('\n');
  output.push_str("Sample\tN\tMean\tStd. error\tStd. dev.\tCI lower (95%)\tCI upper (95%)\n");
  for sample in std::iter::once(&result.group1_stats)
    .chain(result.group2_stats.iter())
    .chain(result.combined_stats.iter())
  {
    ttest_sample_row(&mut output, sample, &sample.observations.to_string());
  }
  // Independent difference counts are legacy residual-df/zero values, not N.
  let difference_n = if independent {
    "NA".to_owned()
  } else {
    result.difference_stats.observations.to_string()
  };
  ttest_sample_row(&mut output, &result.difference_stats, &difference_n);
  if independent {
    writeln!(
      output,
      "Contrast: {} - {}",
      result.group1_stats.name.escape_debug(),
      result
        .group2_stats
        .as_ref()
        .map(|group| group.name.escape_debug().to_string())
        .unwrap_or_else(|| "NA".to_owned())
    )
    .unwrap();
    output.push_str("Difference N is not an observation count; use group N and reference df.\n");
  } else if result.is_paired {
    writeln!(
      output,
      "Contrast: {} - {}",
      result.varname1.escape_debug(),
      result.varname2.as_deref().unwrap_or("NA").escape_debug()
    )
    .unwrap();
  }
  writeln!(
    output,
    "Null {} = {}",
    if independent || result.is_paired {
      "difference"
    } else {
      "mean"
    },
    inference_number(result.null_value)
  )
  .unwrap();
  writeln!(
    output,
    "t reference; df = {}; t = {}",
    inference_number(result.degrees_of_freedom),
    inference_number(result.t_statistic)
  )
  .unwrap();
  writeln!(
    output,
    "P(left-tail) = {}; P(two-sided) = {}; P(right-tail) = {}",
    inference_number(result.p_left),
    inference_number(result.p_two),
    inference_number(result.p_right)
  )
  .unwrap();
  output.push_str("NA = unavailable, undefined, or not an observation count.\n");
  output
}

fn label_key(key: &tabdat_domain::LabelValue) -> String {
  match key {
    tabdat_domain::LabelValue::Integer(value) => format!("integer {value}"),
    tabdat_domain::LabelValue::Number(value) => format!("number {value:?}"),
    tabdat_domain::LabelValue::Text(value) => format!("text {value:?}"),
  }
}

fn label(result: &tabdat_domain::LabelResult) -> String {
  use tabdat_domain::LabelResultAction;
  let action = match result.action {
    LabelResultAction::Variable => "variable",
    LabelResultAction::Define => "define",
    LabelResultAction::Values => "values",
    LabelResultAction::List => "list",
    LabelResultAction::Drop => "drop",
  };
  let mut output = format!("Label {action}: {}\n", result.message.escape_debug());
  let Some(metadata) = &result.metadata else {
    output.push_str("No label metadata.\n");
    return output;
  };
  output.push_str("Variable labels:\n");
  if metadata.variable_labels.is_empty() {
    output.push_str("  (none)\n");
  }
  for (variable, text) in &metadata.variable_labels {
    writeln!(output, "  {variable:?} -> {text:?}").unwrap();
  }
  output.push_str("Attachments:\n");
  if metadata.attachments.is_empty() {
    output.push_str("  (none)\n");
  }
  for (variable, set) in &metadata.attachments {
    writeln!(output, "  {variable:?} -> {set:?}").unwrap();
  }
  output.push_str("Value-label sets:\n");
  if metadata.value_sets.is_empty() {
    output.push_str("  (none)\n");
  }
  for set in &metadata.value_sets {
    writeln!(output, "  {:?}:", set.name).unwrap();
    if set.mappings.is_empty() {
      output.push_str("    (none)\n");
    }
    for (key, text) in &set.mappings {
      writeln!(output, "    {} -> {text:?}", label_key(key)).unwrap();
    }
  }
  output
}

/// Present core workflow guidance or a packaged reference topic without execution.
pub fn render_interactive_help(topic: Option<&str>) -> Result<String, String> {
  match topic {
    None => Ok(concat!(
      "Interactive core workflow\n",
      "  Load local eager CSV/Parquet: use <path>\n",
      "  Inspect: status, describe, codebook [variables], summarize [variables], missing [variables], tabulate, count, head/tail\n",
      "  Transform: generate, replace, rename, sort/gsort, keep/drop/select, recode, encode/decode, label\n",
      "  Cohorts/layouts: sql ... into <name>, use <name>, join, append, reshape, collapse\n",
      "  Grouped EDA: by <keys>: count; by <keys>: summarize uses native means\n",
      "  Linear analysis: regress, predict, lincom, test\n",
      "  Mean comparisons: ttest x == 0; ttest x == y; ttest x, by(group) [welch]\n",
      "  Scripts/files: run <script.td>, save <file.parquet>, export <file.csv>\n",
      "  Help: help [topic] or ?[topic]; example: help summarize\n",
      "  Leave: exit, quit, or EOF\n",
      "Supported forms/options are bounded; reference topics may include parser-only or future syntax.\n",
      "See SPEC.md for current limits. In capable terminals, Tab completes core commands/help topics, local use/run paths, bare use/append/join registered names, and basic bare EDA variable lists.\n",
      "Path/variable prefixes are literal and case-sensitive; open a compatible quote for spaced paths. Piped input is not completed.\n",
      "Ctrl-C cancels pending input; executing-command cancellation is not supported.\n"
    ).to_owned()),
    Some(topic) => crate::catalog::help_topic_result(topic).map(|topic| format!(
      "Reference topic: supported forms/options are bounded; some syntax is parser-only or future. See SPEC.md.\n\n{}",
      topic.text)),
  }
}

fn render_summary_rows(rows: &[SummaryRow]) -> String {
  let mut output = String::from("Variable\tN\tMean\tStd. dev.\tMin\tMax\n");
  for row in rows {
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
        .unwrap_or_else(|| "NA".to_owned()),
    )
    .unwrap();
  }
  output
}

/// Render owned execution results without recomputing data or statistics.
/// Exhaustive matching makes a new result variant require an explicit display decision.
pub fn render_execution_result(result: &ExecutionResult) -> String {
  match result {
    ExecutionResult::Status(status) => {
      let mut output = format!(
        "Session status\nData engine initialized: {}\n",
        status.data_initialized
      );
      if let Some(dataset) = &status.dataset {
        writeln!(
          output,
          "Active source: {:?}\nActive dimensions: {} rows, {} columns",
          dataset.source.to_string_lossy(),
          dataset.row_count,
          dataset.columns.len()
        )
        .unwrap();
        output.push_str("Current columns:\n");
        for column in &dataset.columns {
          writeln!(output, "  {:?}: {:?}", column.name, column.data_type).unwrap();
        }
      } else {
        output.push_str("Active dataset: none\n");
      }
      writeln!(
        output,
        "Active named table: {}\nRegistered named tables: {:?}",
        status
          .active_table_name
          .as_ref()
          .map_or_else(|| "none".to_owned(), |name| format!("{name:?}")),
        status.named_tables
      )
      .unwrap();
      if let Some(model) = &status.linear_model {
        writeln!(output, "Retained linear model: outcome={:?}; predictors={:?}; intercept={}\nRetained N = {}; residual df = {}\nOriginal candidate rows = {}; excluded = {}",
          model.outcome, model.predictors, model.include_intercept, model.observation_count,
          model.degrees_of_freedom,
          model.sample_total_observations.map_or_else(|| "NA".to_owned(), |value| value.to_string()),
          model.sample_dropped_observations.map_or_else(|| "NA".to_owned(), |value| value.to_string())).unwrap();
        output.push_str(
          "Retained fit describes its original sample, not current-data prediction eligibility.\n",
        );
      } else {
        output.push_str("Retained linear model: none\n");
      }
      output
    }
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
    ExecutionResult::Summarize(summary) => render_summary_rows(&summary.rows),
    ExecutionResult::SummarizeDetail(result) => {
      let mut output = String::from("Detailed finite summary: type-7 linear DOUBLE percentiles\n");
      for row in &result.rows {
        output.push_str(&render_summary_rows(std::slice::from_ref(&row.summary)));
        output.push_str("Percentile\tValue\n");
        for percentile in &row.percentiles {
          writeln!(
            output,
            "{}\t{}",
            percentile.percentile,
            optional_number(percentile.value)
          )
          .unwrap();
        }
      }
      output
    }
    ExecutionResult::Codebook(result) => {
      let mut output = String::from(
        "Codebook: SQL NULL missingness; native non-NULL distinct counts. Examples are up to three non-NULL scan values, not necessarily unique.\nVariable\tType\tNonmissing\tMissing\tDistinct\tExamples\n",
      );
      for row in &result.rows {
        writeln!(
          output,
          "{}\t{}\t{}\t{}\t{}\t[{}]",
          row.variable.escape_debug(),
          row.data_type.escape_debug(),
          row.nonmissing,
          row.missing,
          row.distinct,
          row.examples.iter().map(cell).collect::<Vec<_>>().join(", ")
        )
        .unwrap();
      }
      output
    }
    ExecutionResult::Missing(result) => {
      let mut output = String::from(
        "Missingness: SQL NULL only (stored NaN, infinity, empty strings and sentinels are not missing).\nVariable\tType\tTotal\tMissing\tNonmissing\tMissing (%)\n",
      );
      for row in &result.rows {
        writeln!(
          output,
          "{}\t{}\t{}\t{}\t{}\t{}",
          row.variable.escape_debug(),
          row.data_type.escape_debug(),
          row.total,
          row.missing,
          row.nonmissing,
          inference_number(row.missing_percent)
        )
        .unwrap();
      }
      output.push_str("Empty relations report 0% by convention; NA = unavailable or undefined.\n");
      output
    }
    ExecutionResult::Duplicates(result) => format!(
      "Duplicate key report\nKey variables: {}\nRows checked: {}\nDistinct key groups: {}\nDuplicate groups: {}\nRows in duplicate groups: {}\nSurplus rows: {}\nMaximum copies: {}\nSQL NULL key components are grouped; this report does not remove rows.\n",
      result
        .variables
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", "),
      result.total_rows,
      result.unique_groups,
      result.duplicate_groups,
      result.duplicate_rows,
      result.extra_rows,
      result.max_copies
    ),
    ExecutionResult::Isid(result) => format!(
      "isid passed\nKey variables: {}\nRows checked: {}\nDistinct key groups: {}\nRows with SQL NULL keys: {}\nMissing keys allowed: {}\n",
      result
        .variables
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", "),
      result.total_rows,
      result.unique_groups,
      result.missing_key_rows,
      if result.missok { "yes" } else { "no" }
    ),
    ExecutionResult::Assert(result) => format!(
      "assertion passed: {} rows\nFailed rows: {}\n",
      result.checked, result.failed
    ),
    ExecutionResult::Datasignature(result) => format!(
      "Data signature\nAlgorithm: {}\nRows: {}\nColumns: {}\nSignature: {}\n",
      result.algorithm.escape_debug(),
      result.row_count,
      result.column_count,
      result.signature.escape_debug()
    ),
    ExecutionResult::Head(preview) | ExecutionResult::Tail(preview) => {
      table(&preview.columns, &preview.rows)
    }
    ExecutionResult::SqlCreate(result) => dataset_outcome(
      &format!("Created and activated table {:?}", result.table_name),
      &result.dataset,
    ),
    ExecutionResult::Activate(result) => dataset_outcome(
      &format!("Activated table {:?}", result.table_name),
      &result.dataset,
    ),
    ExecutionResult::Join(result) => dataset_outcome("Joined", &result.dataset),
    ExecutionResult::Append(result) => dataset_outcome("Appended", &result.dataset),
    ExecutionResult::Reshape(result) => dataset_outcome("Reshaped", &result.dataset),
    ExecutionResult::Recode(result) => dataset_outcome("Recoded", &result.dataset),
    ExecutionResult::By(result) => format!(
      "Grouped results (native; no finite-sample reinterpretation)\n{}",
      table(&result.headers, &result.rows)
    ),
    ExecutionResult::Table(result) => table(&result.headers, &result.rows),
    ExecutionResult::Tabulate(result) => table(&result.headers, &result.rows),
    ExecutionResult::Collapse(result) => dataset_outcome("Collapsed", &result.dataset),
    ExecutionResult::Encode(result) => dataset_outcome("Encoded", &result.dataset),
    ExecutionResult::Decode(result) => dataset_outcome("Decoded", &result.dataset),
    ExecutionResult::Label(result) => label(result),
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
    ExecutionResult::Ttest(result) => ttest(result),
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
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tabdat_domain::{SummarizeResult, SummaryRow};

  #[test]
  fn key_guard_display_escapes_names_and_preserves_owned_counters() {
    let duplicates = ExecutionResult::Duplicates(tabdat_domain::DuplicatesResult {
      variables: vec!["key\n列".into()],
      total_rows: 5,
      unique_groups: 3,
      duplicate_groups: 1,
      duplicate_rows: 3,
      extra_rows: 2,
      max_copies: 3,
    });
    let text = render_execution_result(&duplicates);
    assert!(text.contains("Key variables: \"key\\n列\"\n"));
    assert!(text.contains("Distinct key groups: 3\nDuplicate groups: 1\nRows in duplicate groups: 3\nSurplus rows: 2\nMaximum copies: 3\n"));
    for missok in [false, true] {
      let result = ExecutionResult::Isid(tabdat_domain::IsidResult {
        variables: vec!["x\t".into()],
        total_rows: 0,
        unique_groups: 0,
        missing_key_rows: 0,
        missok,
      });
      let text = render_execution_result(&result);
      assert!(text.starts_with("isid passed\nKey variables: \"x\\t\"\nRows checked: 0\n"));
      assert!(text.contains(if missok {
        "Missing keys allowed: yes"
      } else {
        "Missing keys allowed: no"
      }));
    }
  }

  #[test]
  fn codebook_display_preserves_examples_and_escapes_controls() {
    let result = ExecutionResult::Codebook(tabdat_domain::CodebookResult {
      rows: vec![tabdat_domain::CodebookRow {
        variable: "x\tline\n\u{1b}[2J".to_owned(),
        data_type: "TYPE\nFAKE".to_owned(),
        nonmissing: 3,
        missing: 0,
        distinct: 2,
        examples: vec![
          CellValue::Text("a\t\n\u{1b}[2J".to_owned()),
          CellValue::Decimal {
            width: 38,
            scale: 2,
            value: -1,
          },
          CellValue::Bytes(vec![0, 255]),
        ],
      }],
    });
    let text = render_execution_result(&result);
    assert!(text.contains("x\\tline\\n\\u{1b}[2J\tTYPE\\nFAKE\t3\t0\t2\t["));
    assert!(text.contains("-0.01, 0x00ff]"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("not necessarily unique"));
    let empty =
      render_execution_result(&ExecutionResult::Codebook(tabdat_domain::CodebookResult {
        rows: vec![],
      }));
    assert!(empty.ends_with("Variable\tType\tNonmissing\tMissing\tDistinct\tExamples\n"));
  }

  #[test]
  fn missing_display_escapes_owned_names_and_types_without_fabricating_percent() {
    let result = ExecutionResult::Missing(tabdat_domain::MissingResult {
      rows: vec![tabdat_domain::MissingRow {
        variable: "x\tline\n\u{1b}[2J".to_owned(),
        data_type: "TYPE\nFAKE".to_owned(),
        total: 1,
        missing: 0,
        nonmissing: 1,
        missing_percent: f64::NAN,
      }],
    });
    let text = render_execution_result(&result);
    assert!(text.contains("x\\tline\\n\\u{1b}[2J\tTYPE\\nFAKE\t1\t0\t1\tNA\n"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.starts_with("Missingness: SQL NULL only"));
  }

  #[test]
  fn ttest_display_escapes_labels_and_keeps_singleton_inference_undefined() {
    let sample = tabdat_domain::TtestGroupStats {
      name: "label\tline\n\u{1b}[2J".to_owned(),
      observations: 1,
      mean: 2.0,
      standard_error: f64::NAN,
      standard_deviation: f64::NAN,
      ci_lower: f64::NAN,
      ci_upper: f64::NAN,
    };
    let result = ExecutionResult::Ttest(Box::new(tabdat_domain::TtestResult {
      varname1: "x\nbad".to_owned(),
      varname2: None,
      by_variable: None,
      is_paired: false,
      is_welch: false,
      null_value: 2.0,
      group1_stats: sample.clone(),
      group2_stats: None,
      combined_stats: None,
      difference_stats: tabdat_domain::TtestGroupStats {
        name: "mean".to_owned(),
        ..sample
      },
      t_statistic: f64::NAN,
      degrees_of_freedom: 0.0,
      p_left: f64::NAN,
      p_two: f64::NAN,
      p_right: f64::NAN,
    }));
    let text = render_execution_result(&result);
    assert!(text.contains("Variable: x\\nbad\n"));
    assert!(text.contains("label\\tline\\n\\u{1b}[2J\t1\t2\tNA\tNA\tNA\tNA\n"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("t reference; df = 0; t = NA\n"));
    assert!(text.contains("P(left-tail) = NA; P(two-sided) = NA; P(right-tail) = NA\n"));
  }

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
