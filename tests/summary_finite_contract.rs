#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-finite-summary-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y,bad,singleton\n1.5,2,NaN,NaN\n,3,Infinity,Infinity\nNaN,4,-Infinity,-Infinity\nInfinity,5,,\n-Infinity,6,,\n3,6,NaN,5\n4,8,Infinity,\n").unwrap();
    fs::write(root.join("empty.csv"), "x\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn shell(&self, input: &str) -> Output {
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
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn close(actual: &Value, expected: &Value) {
  if expected.is_null() {
    assert!(actual.is_null());
    return;
  }
  let actual = actual.as_f64().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected}, tolerance {tolerance}"
  );
}

#[test]
fn finite_summary_reference_matches_json_terminal_and_nested_samples() {
  let fixture = Fixture::new();
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/summary-finite-reference.json")).unwrap();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "missing x",
    "-c",
    "summarize",
    "-c",
    "summarize singleton x x bad y",
    "-c",
    "count",
  ]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 5);
  assert_eq!(results[1]["data"]["rows"][0]["nonmissing"], 6);
  let rows = results[2]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 4);
  for (actual, expected) in rows.iter().zip(reference["rows"].as_array().unwrap()) {
    assert_eq!(actual["variable"], expected["variable"]);
    assert_eq!(actual["count"], expected["count"]);
    for field in ["mean", "std_dev", "minimum", "maximum"] {
      close(&actual[field], &expected[field]);
    }
  }
  assert_eq!(
    results[3]["data"]["rows"],
    json!([rows[3], rows[0], rows[0], rows[2], rows[1]])
  );
  assert_eq!(results[4]["data"]["row_count"], 7);
  let output = fixture.shell("use data.csv\nsummarize\ncount\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let text = String::from_utf8(output.stdout).unwrap();
  let header = text
    .lines()
    .position(|line| line == "Variable\tN\tMean\tStd. dev.\tMin\tMax")
    .unwrap();
  for (line, expected) in text
    .lines()
    .skip(header + 1)
    .take(4)
    .zip(reference["rows"].as_array().unwrap())
  {
    let fields: Vec<_> = line.split('\t').collect();
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0], expected["variable"].as_str().unwrap());
    assert_eq!(fields[1], expected["count"].as_u64().unwrap().to_string());
    for (actual, key) in fields[2..]
      .iter()
      .zip(["mean", "std_dev", "minimum", "maximum"])
    {
      if expected[key].is_null() {
        assert_eq!(*actual, "NA");
      } else {
        close(&json!(actual.parse::<f64>().unwrap()), &expected[key]);
      }
    }
  }
  assert!(text.ends_with("7 rows\n"));
  fs::write(
    fixture.0.join("child.td"),
    "missing x\nsummarize\nsummarize singleton x x bad y\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  assert_eq!(lines(&nested.stdout), results);
}

#[test]
fn finite_summary_preserves_loaded_rows_model_and_failures_and_matches_sample_indices() {
  let fixture = Fixture::new();
  let mut session = tabdat_execution::Session::new();
  let execute = |session: &mut tabdat_execution::Session, source: &str| {
    session.execute(tabdat_language::parse_command(source).unwrap())
  };
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  )
  .unwrap();
  execute(&mut session, "regress y x").unwrap();
  let data = session.active_dataset().cloned();
  let model = session.last_regression().cloned();
  let preview = execute(&mut session, "head 7").unwrap();
  let before = format!("{preview:?}");
  let tabdat_domain::ExecutionResult::Head(preview) = preview else {
    panic!("expected preview");
  };
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/summary-finite-reference.json")).unwrap();
  for (column, row) in reference["rows"].as_array().unwrap().iter().enumerate() {
    let retained: Vec<_> = preview
      .rows
      .iter()
      .enumerate()
      .filter_map(|(index, values)| {
        let finite = match &values[column] {
          tabdat_domain::CellValue::Float(value) => value.is_finite(),
          tabdat_domain::CellValue::SignedInteger(_) => true,
          tabdat_domain::CellValue::Null => false,
          other => panic!("unexpected fixture scalar {other:?}"),
        };
        finite.then_some(index)
      })
      .collect();
    assert_eq!(json!(retained), row["retained_indices"]);
  }
  execute(&mut session, "summarize singleton x bad y x").unwrap();
  assert!(execute(&mut session, "summarize absent x").is_err());
  assert_eq!(session.active_dataset(), data.as_ref());
  assert_eq!(session.last_regression(), model.as_ref());
  assert_eq!(
    format!("{:?}", execute(&mut session, "head 7").unwrap()),
    before
  );
  execute(&mut session, "predict fitted").unwrap();
}

#[test]
fn finite_arithmetic_failure_keeps_prior_data_model_and_script_results() {
  let fixture = Fixture::new();
  let path = fixture.0.join("overflow.csv");
  fs::write(&path, "x,y,overflow\n0,1,1e308\n1,2,-1e308\n2,1.5,\n3,4,\n").unwrap();
  let mut session = tabdat_execution::Session::new();
  let execute = |session: &mut tabdat_execution::Session, source: &str| {
    session.execute(tabdat_language::parse_command(source).unwrap())
  };
  execute(&mut session, &format!("use {}", path.display())).unwrap();
  execute(&mut session, "regress y x").unwrap();
  let data = session.active_dataset().cloned();
  let model = session.last_regression().cloned();
  let before = format!("{:?}", execute(&mut session, "head 4").unwrap());
  assert_eq!(
    execute(&mut session, "summarize y overflow").unwrap_err(),
    tabdat_domain::RuntimeError::SummaryFailed {
      variable: "overflow".to_owned()
    }
  );
  assert_eq!(session.active_dataset(), data.as_ref());
  assert_eq!(session.last_regression(), model.as_ref());
  assert_eq!(
    format!("{:?}", execute(&mut session, "head 4").unwrap()),
    before
  );
  execute(&mut session, "summarize y").unwrap();
  execute(&mut session, "predict fitted").unwrap();
  fs::write(
    fixture.0.join("child.td"),
    "summarize y\nsummarize overflow\nexport never.csv\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use overflow.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let output = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_eq!(output.status.code(), Some(1));
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 3);
  assert_eq!(results[1]["result_type"], "SummarizeResult");
  assert_eq!(results[2]["error"]["type"], "ScriptError");
  assert_eq!(results[2]["error"]["line"], 2);
  assert_eq!(
    results[2]["error"]["path"],
    fixture
      .0
      .join("child.td")
      .canonicalize()
      .unwrap()
      .to_string_lossy()
      .as_ref()
  );
  assert!(!fixture.0.join("never.csv").exists());
}

#[test]
fn summary_without_finite_values_and_singleton_have_explicit_undefined_statistics() {
  let fixture = Fixture::new();
  for (file, source, count, mean) in [
    (
      "invalid.csv",
      "x\nNaN\nInfinity\n-Infinity\n",
      0,
      Value::Null,
    ),
    ("nan.csv", "x\nNaN\nNaN\n", 0, Value::Null),
    ("one.csv", "x\nNaN\n7.5\nInfinity\n", 1, json!(7.5)),
  ] {
    fs::write(fixture.0.join(file), source).unwrap();
    let command = format!("use {file}");
    let output = fixture.batch(&["--json", "-c", &command, "-c", "summarize x"]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    let results = lines(&output.stdout);
    let row = &results[1]["data"]["rows"][0];
    assert_eq!(row["count"], count);
    assert_eq!(row["mean"], mean);
    assert!(row["std_dev"].is_null());
    assert_eq!(row["minimum"], mean);
    assert_eq!(row["maximum"], mean);
  }
  // Give a header-only relation a numeric column using supported generation.
  let output = fixture.batch(&[
    "-c",
    "use empty.csv",
    "-c",
    "generate numeric = 0",
    "-c",
    "save empty.parquet",
  ]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let output = fixture.batch(&["--json", "-c", "use empty.parquet", "-c", "summarize"]);
  assert!(output.status.success());
  for row in lines(&output.stdout)[1]["data"]["rows"].as_array().unwrap() {
    assert_eq!(row["count"], 0);
    for field in ["mean", "std_dev", "minimum", "maximum"] {
      assert!(row[field].is_null());
    }
  }
}
