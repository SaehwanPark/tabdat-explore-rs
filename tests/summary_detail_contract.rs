#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_execution::Session;
use tabdat_language::parse_command;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-summary-detail-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y\n0,1\n1,2\n2,4\n3,6\n").unwrap();
    Self(root)
  }
  fn batch(&self, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(arguments)
      .output()
      .unwrap()
  }
  fn interactive(&self, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(input.as_bytes())
      .unwrap();
    child.wait_with_output().unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn reference() -> Value {
  serde_json::from_str(include_str!("fixtures/summary-detail-reference.json")).unwrap()
}
fn close(actual: &Value, expected: &Value) {
  if expected.is_null() {
    assert!(actual.is_null(), "{actual}");
    return;
  }
  let (actual, expected) = (actual.as_f64().unwrap(), expected.as_f64().unwrap());
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected}; tolerance {tolerance}"
  );
}

#[test]
fn exact_extrema_survive_approximate_quantiles_and_control_safe_rendering() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(
    &mut session,
    "sql SELECT CAST(9007199254740993 + x*2 AS BIGINT) AS wide, CAST(y AS DECIMAL(12,2)) AS amount FROM active into scalars",
  );
  let ordinary = execute(&mut session, "summarize wide amount");
  let result = execute(&mut session, "summarize wide amount, detail");
  let ExecutionResult::SummarizeDetail(detail) = &result else {
    panic!("detail")
  };
  let ExecutionResult::Summarize(ordinary) = ordinary else {
    panic!("ordinary")
  };
  assert_eq!(
    detail
      .rows
      .iter()
      .map(|row| row.summary.clone())
      .collect::<Vec<_>>(),
    ordinary.rows
  );
  assert_eq!(
    detail.rows[0].summary.minimum,
    Some(CellValue::SignedInteger(9007199254740993))
  );
  assert!(matches!(
    detail.rows[1].summary.minimum,
    Some(CellValue::Decimal {
      scale: 2,
      value: 100,
      ..
    })
  ));
  assert_eq!(
    detail.rows[0].percentiles[4].value,
    Some(9007199254740996.0)
  );
  assert_eq!(detail.rows[1].percentiles[4].value, Some(3.0));
  let encoded: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  assert_eq!(
    encoded["data"]["rows"][0]["summary"]["minimum"],
    json!(9007199254740993_i64)
  );
  assert_eq!(encoded["data"]["rows"][1]["summary"]["minimum"], "1.00");
  execute(
    &mut session,
    "sql SELECT wide AS \"value\t雪\" FROM active into escaped",
  );
  let mut result = execute(&mut session, "summarize \"value\t雪\", detail");
  let text = tabdat_reporting::terminal::render_execution_result(&result);
  assert!(text.contains("value\\t雪\t4\t"));
  let ExecutionResult::SummarizeDetail(detail) = &mut result else {
    panic!("detail")
  };
  for (index, value) in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0]
    .into_iter()
    .enumerate()
  {
    detail.rows[0].percentiles[index].value = Some(value);
  }
  let encoded: Value = serde_json::from_str(
    &tabdat_reporting::json_results::serialize_execution_result(&result).unwrap(),
  )
  .unwrap();
  for index in 0..3 {
    assert!(encoded["data"]["rows"][0]["percentiles"][index]["value"].is_null());
  }
  assert_eq!(encoded["data"]["rows"][0]["percentiles"][3]["value"], 0.0);
}

#[test]
fn detail_validation_and_native_failures_stop_files_and_recover_without_partial_results() {
  let fixture = Fixture::new();
  for source in [
    "summarize absent, detail",
    "summarize x if x, detail",
    "summarize x, detail(x)",
    "summarize x, detail detail",
    "summarize x, detail limit",
    "by x: summarize y, detail",
  ] {
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export never.csv",
    ]);
    assert!(!output.status.success(), "{source}");
    let results = lines(&output.stdout);
    assert_eq!(results.len(), 2);
    assert!(results[1].get("error").is_some());
    assert!(!fixture.0.join("never.csv").exists());
  }
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(&mut session, "regress y x");
  execute(
    &mut session,
    "sql SELECT CASE WHEN x < 2 THEN 1e308 ELSE -1e308 END AS huge, y FROM active into overflowing",
  );
  let state = session.state();
  let error = session
    .execute(parse_command("summarize y huge, detail").unwrap())
    .unwrap_err();
  assert!(
    matches!(error, tabdat_domain::RuntimeError::SummaryFailed { variable } if variable == "huge")
  );
  assert_eq!(session.state(), state);
  let result = execute(&mut session, "summarize y, detail");
  assert!(matches!(result, ExecutionResult::SummarizeDetail(_)));
  assert_eq!(session.state(), state);
  let output = fixture.interactive("use data.csv\nregress y x\nsummarize absent, detail\nsummarize x, detail\npredict fitted\nexport recovered.csv\nexit\n");
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8(output.stderr).unwrap(),
    "Error: summarize unknown variable: absent\n"
  );
  assert!(fixture.0.join("recovered.csv").is_file());
  let text = String::from_utf8(output.stdout).unwrap();
  assert_eq!(text.matches("Detailed finite summary:").count(), 1);
  assert!(text.contains("Predicted: 4 rows, 3 columns\n"));
}

#[test]
fn detailed_samples_and_statistics_match_independent_numpy_and_ordinary_summaries() {
  let fixture = Fixture::new();
  let reference = reference();
  for case in reference["cases"].as_array().unwrap() {
    fs::write(fixture.0.join("case.csv"), case["csv"].as_str().unwrap()).unwrap();
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", fixture.0.join("case.csv").display()),
    );
    // Explicit types also cover empty/all-NULL CSV inference, not just numeric fixtures.
    execute(
      &mut session,
      "sql SELECT CAST(x AS DOUBLE) AS x, CAST(y AS DOUBLE) AS y, CAST(selected AS BOOLEAN) AS selected FROM active into sample",
    );
    let state = session.state();
    let condition = case["condition"]
      .as_str()
      .map(|condition| format!(" if {condition}"))
      .unwrap_or_default();
    let detailed = execute(&mut session, &format!("summarize x y{condition}, detail"));
    let ordinary = execute(&mut session, &format!("summarize x y{condition}"));
    let ExecutionResult::SummarizeDetail(detail) = &detailed else {
      panic!("wrong detail variant")
    };
    let ExecutionResult::Summarize(ordinary) = ordinary else {
      panic!("wrong ordinary variant")
    };
    assert_eq!(
      detail
        .rows
        .iter()
        .map(|row| row.summary.clone())
        .collect::<Vec<_>>(),
      ordinary.rows
    );
    assert_eq!(session.state(), state);
    let encoded: Value = serde_json::from_str(
      &tabdat_reporting::json_results::serialize_execution_result(&detailed).unwrap(),
    )
    .unwrap();
    assert_eq!(encoded["result_type"], "SummarizeDetailResult");
    assert_eq!(encoded["schema_version"], 1);
    assert_eq!(encoded["data"]["quantile_method"], "linear_type_7");
    let ExecutionResult::Head(raw) = execute(&mut session, "head 100") else {
      panic!("preview")
    };
    for (column, expected) in case["rows"].as_array().unwrap().iter().enumerate() {
      let selected = |row: &[CellValue]| match case["condition"].as_str() {
        None => true,
        Some("(x + 0) > 1000") => {
          matches!(row[0], CellValue::Float(value) if value.is_finite() && value > 1000.0)
        }
        Some("selected") => row[2] == CellValue::Boolean(true),
        _ => panic!("unexpected predicate"),
      };
      let finite_indices: Vec<_> = raw
        .rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
          (selected(row) && matches!(row[column], CellValue::Float(value) if value.is_finite()))
            .then_some(index)
        })
        .collect();
      assert_eq!(
        json!(finite_indices),
        expected["finite_indices"],
        "{}",
        case["name"]
      );
      let observed = &encoded["data"]["rows"][column];
      assert_eq!(observed["summary"]["variable"], expected["variable"]);
      assert_eq!(observed["summary"]["count"], expected["count"]);
      for key in ["mean", "std_dev", "minimum", "maximum"] {
        close(&observed["summary"][key], &expected[key]);
      }
      assert_eq!(detail.rows[column].percentiles.len(), 9);
      for (index, percentile) in observed["percentiles"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
      {
        assert_eq!(percentile["percentile"], reference["percentiles"][index]);
        close(&percentile["value"], &expected["percentiles"][index]);
        if expected["count"] == 0 {
          assert_eq!(detail.rows[column].percentiles[index].value, None);
        }
      }
    }
    let ExecutionResult::SummarizeDetail(default) =
      execute(&mut session, &format!("summarize{condition}, detail"))
    else {
      panic!("default")
    };
    assert_eq!(default, *detail);
    let ExecutionResult::SummarizeDetail(repeated) =
      execute(&mut session, &format!("summarize y x y{condition}, detail"))
    else {
      panic!("repeated")
    };
    assert_eq!(
      repeated.rows,
      vec![
        detail.rows[1].clone(),
        detail.rows[0].clone(),
        detail.rows[1].clone()
      ]
    );
    assert_eq!(session.state(), state);
  }
}

#[test]
fn detailed_inspection_and_failures_preserve_named_labels_fit_and_current_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  );
  execute(
    &mut session,
    "sql SELECT *, 'a' AS category FROM active into sample",
  );
  execute(&mut session, "label variable x 'original x'");
  execute(&mut session, "regress y x if x >= 1");
  let state = session.state();
  let rows = execute(&mut session, "head 10");
  for source in [
    "summarize, detail",
    "summarize x if x >= 2, detail",
    "summarize x if (x + 0) > 1000, detail",
  ] {
    let result = execute(&mut session, source);
    assert!(matches!(result, ExecutionResult::SummarizeDetail(_)));
    assert_eq!(session.state(), state);
    assert_eq!(execute(&mut session, "head 10"), rows);
  }
  for source in [
    "summarize absent, detail",
    "summarize category, detail",
    "summarize x if absent > 0, detail",
    "summarize x if x, detail",
    "summarize x if inrange(x, 0, 2), detail",
    "summarize x, detail(x)",
    "summarize x, detail detail",
    "summarize x, detail limit",
    "by category: summarize x, detail",
  ] {
    let command = parse_command(source).unwrap();
    assert!(session.execute(command).is_err(), "{source}");
    assert_eq!(session.state(), state, "{source}");
    assert_eq!(execute(&mut session, "head 10"), rows);
  }
  execute(&mut session, "predict fitted");
  assert_eq!(
    session.last_regression(),
    state
      .estimation
      .as_ref()
      .unwrap()
      .linear()
      .map(|state| &state.result)
  );
}

#[test]
fn detail_streams_equally_through_batch_nested_run_text_and_prediction_persistence() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "summarize x y, detail",
    "regress y x",
    "summarize y x y if x >= 1, detail",
    "predict fitted",
    "export output.csv",
    "head 4",
  ];
  let arguments: Vec<_> = std::iter::once("--json")
    .chain(commands.iter().flat_map(|source| ["-c", *source]))
    .collect();
  let output = fixture.batch(&arguments);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let batch = lines(&output.stdout);
  assert_eq!(
    batch
      .iter()
      .map(|result| result["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "SummarizeDetailResult",
      "RegressionResult",
      "SummarizeDetailResult",
      "PredictionResult",
      "ExportResult",
      "PreviewResult"
    ]
  );
  let exported = fs::read(fixture.0.join("output.csv")).unwrap();
  fs::write(fixture.0.join("child.td"), commands[1..].join("\n") + "\n").unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  for args in [
    vec!["--json", "-f", "parent.td"],
    vec!["--json", "-c", "run parent.td"],
  ] {
    fs::remove_file(fixture.0.join("output.csv")).unwrap();
    let nested = fixture.batch(&args);
    assert!(
      nested.status.success(),
      "{}",
      String::from_utf8_lossy(&nested.stderr)
    );
    assert!(nested.stderr.is_empty());
    let mut nested = lines(&nested.stdout);
    // Includes resolve paths against their source, not the batch working directory.
    for index in [0, 4, 5] {
      nested[index]["data"]["dataset"]["path"] = batch[index]["data"]["dataset"]["path"].clone();
    }
    nested[5]["data"]["path"] = batch[5]["data"]["path"].clone();
    assert_eq!(nested, batch);
    assert_eq!(fs::read(fixture.0.join("output.csv")).unwrap(), exported);
  }
  fs::remove_file(fixture.0.join("output.csv")).unwrap();
  let output = fixture.interactive(&format!("help summarize\n{}\nexit\n", commands.join("\n")));
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert_eq!(text.matches("Detailed finite summary:").count(), 2);
  assert_eq!(text.matches("Percentile\tValue\n").count(), 5);
  assert!(text.contains("50\t1.5\n"));
  assert_eq!(fs::read(fixture.0.join("output.csv")).unwrap(), exported);
  fs::remove_file(fixture.0.join("output.csv")).unwrap();
  let text_arguments: Vec<_> = std::iter::once("--text")
    .chain(commands.iter().flat_map(|source| ["-c", *source]))
    .collect();
  let text_output = fixture.batch(&text_arguments);
  assert!(text_output.status.success());
  assert!(text_output.stderr.is_empty());
  let rendered = String::from_utf8(text_output.stdout).unwrap();
  assert_eq!(rendered.matches("Detailed finite summary:").count(), 2);
  assert_eq!(fs::read(fixture.0.join("output.csv")).unwrap(), exported);
  let preview = &batch[6]["data"]["rows"];
  for (index, row) in preview.as_array().unwrap().iter().enumerate() {
    assert!((row[2].as_f64().unwrap() - (0.7 + 1.7 * index as f64)).abs() < 1e-10);
  }
}
