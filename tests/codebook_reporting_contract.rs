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
      "tabdat-codebook-report-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y,label,allnull\n1.5,2,A,\n,3,,\nNaN,4,B,\nInfinity,5,A,\n-Infinity,6,,\n3,6,C,\n4,8,A,\n",
    )
    .unwrap();
    fs::write(root.join("empty.csv"), "x,label\n").unwrap();
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

#[test]
fn codebook_reports_owned_profiles_order_repeats_and_nonfinite_example_policy() {
  let fixture = Fixture::new();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "codebook",
    "-c",
    "codebook label x label",
    "-c",
    "missing x",
    "-c",
    "summarize x",
  ]);
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 5);
  assert_eq!(results[1]["schema_version"], 1);
  assert_eq!(results[1]["result_type"], "CodebookResult");
  let data = &results[1]["data"];
  assert_eq!(data["missing_definition"], "sql_null");
  assert_eq!(data["example_limit"], 3);
  assert_eq!(data["examples_nonfinite_encoding"], "null");
  let rows = data["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 4);
  for (index, (variable, nonmissing, missing, distinct, examples)) in [
    ("x", 6, 1, 6, json!([1.5, null, null])),
    ("y", 7, 0, 6, json!([2, 3, 4])),
    ("label", 5, 2, 3, json!(["A", "B", "A"])),
    ("allnull", 0, 7, 0, json!([])),
  ]
  .iter()
  .enumerate()
  {
    assert_eq!(
      rows[index],
      json!({"variable": variable, "data_type": results[0]["data"]["dataset"]["columns"][index]["data_type"], "nonmissing": nonmissing, "missing": missing, "distinct": distinct, "examples": examples})
    );
  }
  assert_eq!(
    results[2]["data"]["rows"],
    json!([rows[2], rows[0], rows[2]])
  );
  assert_eq!(results[3]["data"]["rows"][0]["nonmissing"], 6);
  assert_eq!(results[4]["data"]["rows"][0]["count"], 3);
  let output = fixture.shell("use data.csv\ncodebook label x label\ncount\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("not necessarily unique"));
  let header = text
    .lines()
    .position(|line| line == "Variable\tType\tNonmissing\tMissing\tDistinct\tExamples")
    .unwrap();
  let table: Vec<_> = text.lines().skip(header + 1).take(3).collect();
  assert_eq!(
    table,
    [
      "label\tVARCHAR\t5\t2\t3\t[\"A\", \"B\", \"A\"]",
      "x\tDOUBLE\t6\t1\t6\t[1.5, NaN, inf]",
      "label\tVARCHAR\t5\t2\t3\t[\"A\", \"B\", \"A\"]"
    ]
  );
  assert!(text.ends_with("7 rows\n"));
  fs::write(
    fixture.0.join("child.td"),
    "codebook\ncodebook label x label\nmissing x\nsummarize x\n",
  )
  .unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  let nested = fixture.batch(&["--json", "-f", "parent.td"]);
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  assert_eq!(lines(&nested.stdout), results);
}

#[test]
fn codebook_empty_profiles_and_errors_keep_prior_results_and_locations() {
  let fixture = Fixture::new();
  let output = fixture.batch(&["--json", "-c", "use empty.csv", "-c", "codebook"]);
  assert!(output.status.success());
  for row in lines(&output.stdout)[1]["data"]["rows"].as_array().unwrap() {
    for key in ["nonmissing", "missing", "distinct"] {
      assert_eq!(row[key], 0);
    }
    assert_eq!(row["examples"], json!([]));
  }
  let prepared = fixture.batch(&[
    "-c",
    "use data.csv",
    "-c",
    "recode y (nonmissing = ''), generate(stored_empty)",
    "-c",
    "save strings.parquet",
  ]);
  assert!(prepared.status.success());
  let profile = fixture.batch(&[
    "--json",
    "-c",
    "use strings.parquet",
    "-c",
    "codebook stored_empty",
  ]);
  assert!(profile.status.success());
  assert_eq!(
    lines(&profile.stdout)[1]["data"]["rows"][0]["examples"],
    json!(["", "", ""])
  );
  assert_eq!(
    lines(&profile.stdout)[1]["data"]["rows"][0]["nonmissing"],
    7
  );
  assert_eq!(lines(&profile.stdout)[1]["data"]["rows"][0]["distinct"], 1);
  let output = fixture.batch(&["--json", "-c", "codebook"]);
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(lines(&output.stdout)[0]["error"]["type"], "ExecutionError");
  fs::write(
    fixture.0.join("child.td"),
    "codebook label\ncodebook absent\nexport never.csv\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let output = fixture.batch(&["--json", "-f", "parent.td"]);
  assert_eq!(output.status.code(), Some(1));
  let results = lines(&output.stdout);
  assert_eq!(results.len(), 3);
  assert_eq!(results[1]["result_type"], "CodebookResult");
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
fn codebook_rendering_success_and_failure_preserve_data_and_saved_model() {
  let fixture = Fixture::new();
  let output = fixture.shell(
    "use data.csv\nregress y x\ncodebook label x\ncodebook absent\npredict fitted\ncount\nexit\n",
  );
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8_lossy(&output.stderr)
      .matches("Error:")
      .count(),
    1
  );
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Predicted: 7 rows, 5 columns\n"));
  assert!(text.ends_with("7 rows\n"));
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
  let before = format!("{:?}", execute(&mut session, "head 7").unwrap());
  let result = execute(&mut session, "codebook label x label").unwrap();
  assert!(tabdat_reporting::json_results::serialize_execution_result(&result).is_some());
  assert!(tabdat_reporting::terminal::render_execution_result(&result).contains("Codebook:"));
  assert!(execute(&mut session, "codebook absent").is_err());
  assert_eq!(session.active_dataset(), data.as_ref());
  assert_eq!(session.last_regression(), model.as_ref());
  assert_eq!(
    format!("{:?}", execute(&mut session, "head 7").unwrap()),
    before
  );
}
