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
      "tabdat-missing-report-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("raw.csv"),
      "x,y,label,allnull\n1.5,2,A,\n,3,,\nNaN,4,B,\nInfinity,5,C,\n-Infinity,6,,\n3,6,D,\n4,8,E,\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "x,label\n").unwrap();
    let fixture = Self(root);
    // Materialize actual stored empty strings and a sentinel, rather than
    // confuse CSV empty-field parsing with the report's NULL-only definition.
    let output = fixture.batch(&[
      "-c",
      "use raw.csv",
      "-c",
      "recode y (nonmissing = ''), generate(empty)",
      "-c",
      "generate sentinel = -999",
      "-c",
      "save data.parquet",
    ]);
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    fixture
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

#[test]
fn missing_reports_null_only_counts_types_order_duplicates_and_finite_sample_difference() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.parquet",
    "-c",
    "missing",
    "-c",
    "missing label x label",
    "-c",
    "summarize y",
    "-c",
    "ttest x == 0",
  ]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 5);
  assert_eq!(results[1]["result_type"], "MissingResult");
  assert_eq!(results[1]["schema_version"], 1);
  assert_eq!(results[1]["data"]["missing_definition"], "sql_null");
  let rows = results[1]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 6);
  for (index, (name, missing)) in [
    ("x", 1),
    ("y", 0),
    ("label", 2),
    ("allnull", 7),
    ("empty", 0),
    ("sentinel", 0),
  ]
  .iter()
  .enumerate()
  {
    let row = &rows[index];
    assert_eq!(row["variable"], *name);
    assert_eq!(row["total"], 7);
    assert_eq!(row["missing"], *missing);
    assert_eq!(row["nonmissing"], 7 - missing);
    assert_eq!(
      row["data_type"],
      results[0]["data"]["dataset"]["columns"][index]["data_type"]
    );
    assert!(
      (row["missing_percent"].as_f64().unwrap() - *missing as f64 * 100.0 / 7.0).abs() < 1e-12
    );
  }
  assert_eq!(rows[0]["data_type"], "DOUBLE");
  assert_eq!(rows[2]["data_type"], "VARCHAR");
  assert_eq!(
    results[2]["data"]["rows"],
    json!([rows[2], rows[0], rows[2]])
  );
  // SQL NULL excludes only row 1; ttest separately excludes NaN and +/-infinity.
  assert_eq!(rows[0]["nonmissing"], 6);
  assert_eq!(results[3]["data"]["rows"][0]["count"], 7);
  assert_eq!(results[4]["data"]["group1_stats"]["count"], 3);
  let output =
    fixture.shell("use data.parquet\nmissing label x label\nsummarize y\nttest x == 0\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Missingness: SQL NULL only"));
  let header = text
    .lines()
    .position(|line| line == "Variable\tType\tTotal\tMissing\tNonmissing\tMissing (%)")
    .unwrap();
  let table: Vec<_> = text.lines().skip(header + 1).take(3).collect();
  for (actual, row) in table.iter().zip([&rows[2], &rows[0], &rows[2]]) {
    let fields: Vec<_> = actual.split('\t').collect();
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0], row["variable"].as_str().unwrap());
    assert_eq!(fields[1], row["data_type"].as_str().unwrap());
    for (field, key) in fields[2..5].iter().zip(["total", "missing", "nonmissing"]) {
      assert_eq!(*field, row[key].as_u64().unwrap().to_string());
    }
    assert_eq!(
      fields[5].parse::<f64>().unwrap(),
      row["missing_percent"].as_f64().unwrap()
    );
  }
}

#[test]
fn empty_and_all_null_missingness_preserve_existing_zero_percent_convention() {
  let fixture = Fixture::new();
  let output = fixture.batch(&["--json", "-c", "use empty.csv", "-c", "missing"]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 2);
  for row in results[1]["data"]["rows"].as_array().unwrap() {
    for key in ["total", "missing", "nonmissing"] {
      assert_eq!(row[key], 0);
    }
    assert_eq!(row["missing_percent"], 0.0);
  }
  let output = fixture.shell("use empty.csv\nmissing\ncount\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("\t0\t0\t0\t0\n"));
  assert!(text.ends_with("0 rows\n"));
}

#[test]
fn nested_missingness_streams_and_errors_are_located_without_later_effects() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.parquet\nrun child.td\ncount\n",
  )
  .unwrap();
  fs::write(fixture.0.join("child.td"), "missing label x label\n").unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  let batch = fixture.batch(&[
    "--json",
    "-c",
    "use data.parquet",
    "-c",
    "missing label x label",
    "-c",
    "count",
  ]);
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  assert_eq!(lines(&nested.stdout), lines(&batch.stdout));
  let plain = fixture.shell("use data.parquet\nmissing label x label\ncount\nexit\n");
  let streamed = fixture.shell("run parent.td\nexit\n");
  assert!(streamed.status.success());
  assert!(streamed.stderr.is_empty());
  assert!(
    String::from_utf8(streamed.stdout)
      .unwrap()
      .starts_with(&String::from_utf8(plain.stdout).unwrap())
  );
  fs::write(
    fixture.0.join("child.td"),
    "missing x\nmissing absent\nexport never.csv\n",
  )
  .unwrap();
  let output = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_eq!(output.status.code(), Some(1));
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 3);
  assert_eq!(results[1]["result_type"], "MissingResult");
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
  let output = fixture.batch(&["--json", "-c", "missing"]);
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(lines(&output.stdout)[0]["error"]["type"], "ExecutionError");
}

#[test]
fn missing_success_failure_and_serialization_preserve_dataset_and_model() {
  let fixture = Fixture::new();
  let output = fixture.shell(
    "use data.parquet\nregress y x\nmissing x\nmissing absent\npredict fitted\ncount\nexit\n",
  );
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8_lossy(&output.stderr)
      .matches("Error:")
      .count(),
    1
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Predicted: 7 rows, 7 columns\n"));
  assert!(text.ends_with("7 rows\n"));
  let mut session = tabdat_execution::Session::new();
  let execute = |session: &mut tabdat_execution::Session, source: &str| {
    session.execute(tabdat_language::parse_command(source).unwrap())
  };
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.parquet").display()),
  )
  .unwrap();
  execute(&mut session, "regress y x").unwrap();
  let data = session.active_dataset().cloned();
  let model = session.last_regression().cloned();
  // NaN is unequal to itself; a typed debug snapshot also distinguishes NaN
  // from +/-infinity, which the JSON preview encoder intentionally maps to null.
  let before = format!("{:?}", execute(&mut session, "head 7").unwrap());
  let result = execute(&mut session, "missing label x label").unwrap();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert!(tabdat_reporting::terminal::render_execution_result(&result).contains("SQL NULL only"));
  assert!(execute(&mut session, "missing absent").is_err());
  assert_eq!(session.active_dataset(), data.as_ref());
  assert_eq!(session.last_regression(), model.as_ref());
  let after = format!("{:?}", execute(&mut session, "head 7").unwrap());
  assert_eq!(before, after);
}
