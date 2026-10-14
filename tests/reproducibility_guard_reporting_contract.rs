#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-repro-guards-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x,y\n1,2\n2,5\n3,5\n4,9\n5,\n").unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn batch(&self, commands: &[&str]) -> Vec<Value> {
    let mut args = vec!["--json"];
    for source in commands {
      args.extend(["-c", source]);
    }
    success(self.run(&args))
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
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
fn value(result: &ExecutionResult) -> Value {
  serde_json::from_str(&tabdat_reporting::json_results::serialize_execution_result(result).unwrap())
    .unwrap()
}
fn lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn success(output: Output) -> Vec<Value> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  lines(&output.stdout)
}
#[test]
fn owned_envelopes_and_control_safe_terminal_preserve_exact_fields_and_admission() {
  let results = [
    (
      ExecutionResult::Assert(tabdat_runtime::AssertResult {
        checked: u64::MAX,
        failed: 0,
      }),
      "AssertResult",
      json!({"checked":u64::MAX,"failed":0}),
    ),
    (
      ExecutionResult::Datasignature(tabdat_runtime::DatasignatureResult {
        algorithm: "sha256\n\u{1b}".into(),
        signature: "雪\r".into(),
        row_count: u64::MAX,
        column_count: 3,
      }),
      "DatasignatureResult",
      json!({"algorithm":"sha256\n\u{1b}","signature":"雪\r","row_count":u64::MAX,"column_count":3}),
    ),
  ];
  for (result, kind, data) in results {
    assert_eq!(
      value(&result),
      json!({"schema_version":1,"result_type":kind,"data":data})
    );
    let terminal = tabdat_reporting::terminal::render_execution_result(&result);
    assert!(!terminal.contains('\u{1b}'));
    assert!(!terminal.contains('\r'));
    assert!(!terminal.contains("unsupported"));
  }
  for source in ["assert x >= 1", "datasignature"] {
    let command = parse_command(source).unwrap();
    assert!(tabdat_reporting::json_results::supports_batch_command(
      &command
    ));
    assert!(tabdat_reporting::json_results::supports_script_command(
      &command
    ));
  }
}
#[test]
fn current_data_guards_preserve_named_labels_cached_fit_and_failed_predicate_state() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable x 'Predictor'");
  execute(&mut session, "regress y x");
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let preview = execute(&mut session, "head 10");
  let inference = execute(&mut session, "lincom x");
  let signature = execute(&mut session, "datasignature");
  assert_eq!(value(&signature)["data"]["row_count"], 5);
  assert_eq!(
    session
      .last_regression()
      .unwrap()
      .sample
      .as_ref()
      .unwrap()
      .retained_indices,
    [0, 1, 2, 3]
  );
  for source in ["assert x >= 1", "assert x == x", "datasignature"] {
    let result = execute(&mut session, source);
    if source == "datasignature" {
      assert_eq!(result, signature);
    } else {
      assert_eq!(value(&result)["data"], json!({"checked":5,"failed":0}));
    }
  }
  for (source, message) in [
    ("assert y >= 0", "assertion failed: 1 of 5 rows failed"),
    ("assert x > 3", "assertion failed: 3 of 5 rows failed"),
    ("assert null", "assertion failed: 5 of 5 rows failed"),
    ("assert absent > 0", "unknown"),
    ("assert x", "predicate requires boolean expression"),
  ] {
    let error = session.execute(parse_command(source).unwrap()).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(execute(&mut session, "head 10"), preview);
    assert_eq!(execute(&mut session, "lincom x"), inference);
    assert_eq!(execute(&mut session, "datasignature"), signature);
  }
  execute(&mut session, "use cohort");
  assert_eq!(execute(&mut session, "head 10"), preview);
}
#[test]
fn pinned_native_digest_empty_null_nonfinite_and_schema_order_changes() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age,bmi,sex,cost) into sample",
  );
  let pinned = value(&execute(&mut session, "datasignature"));
  assert_eq!(
    pinned["data"],
    json!({"algorithm":"sha256","signature":"0b61cef05ab04301df893652f666b6ed974668f0cd214ebae17cfff02a6b3aad","row_count":3,"column_count":4})
  );
  execute(&mut session, "label variable age 'Age'");
  assert_eq!(value(&execute(&mut session, "datasignature")), pinned);
  execute(&mut session, "gsort -age");
  assert_ne!(value(&execute(&mut session, "datasignature")), pinned);
  execute(&mut session, "sort age");
  assert_eq!(value(&execute(&mut session, "datasignature")), pinned);
  execute(&mut session, "rename age years");
  let renamed = value(&execute(&mut session, "datasignature"));
  assert_ne!(renamed, pinned);
  execute(&mut session, "replace years = years + 1");
  assert_ne!(value(&execute(&mut session, "datasignature")), renamed);
  execute(
    &mut session,
    "sql select cast('NaN' as double) as x, '' as s union all select cast('Infinity' as double), NULL union all select NULL, '雪' into edge",
  );
  let edge = execute(&mut session, "datasignature");
  assert_eq!(execute(&mut session, "datasignature"), edge);
  assert!(
    session
      .execute(parse_command("assert s != null").unwrap())
      .unwrap_err()
      .to_string()
      .contains("1 of 3 rows")
  );
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  assert_eq!(
    value(&execute(&mut session, "assert x > 0"))["data"],
    json!({"checked":0,"failed":0})
  );
  let empty = value(&execute(&mut session, "datasignature"));
  assert_eq!(empty["data"]["row_count"], 0);
  assert_eq!(empty["data"]["column_count"], 2);
  assert_eq!(empty["data"]["signature"].as_str().unwrap().len(), 64);
}
#[test]
fn guarded_analysis_prediction_persistence_nested_and_terminal_recovery() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "assert x >= 1",
    "datasignature",
    "regress y x",
    "predict fitted",
    "assert fitted > 0",
    "datasignature",
    "save result.parquet",
    "export result.csv",
  ];
  let batch = fixture.batch(&commands);
  assert_eq!(batch[1]["data"], json!({"checked":5,"failed":0}));
  assert_ne!(batch[2]["data"]["signature"], batch[6]["data"]["signature"]);
  let roundtrip = fixture.batch(&["use result.parquet", "datasignature", "assert fitted > 0"]);
  assert_eq!(roundtrip[1]["data"], batch[6]["data"]);
  fs::remove_file(fixture.0.join("result.parquet")).unwrap();
  fs::remove_file(fixture.0.join("result.csv")).unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    format!("{}\nrun child.td\n", commands[..3].join("\n")),
  )
  .unwrap();
  fs::write(fixture.0.join("child.td"), commands[3..].join("\n")).unwrap();
  assert_eq!(success(fixture.run(&["--json", "-f", "parent.td"])), batch);
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(b"use data.csv\nassert y >= 0\nassert x >= 1\ndatasignature\nexit\n")
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("assertion passed: 5 rows\nFailed rows: 0"));
  assert!(text.contains(&format!(
    "Signature: {}",
    batch[2]["data"]["signature"].as_str().unwrap()
  )));
  assert!(
    String::from_utf8(output.stderr)
      .unwrap()
      .contains("assertion failed: 1 of 5 rows failed")
  );
}
#[test]
fn guard_errors_have_no_success_envelope_and_stop_later_files_with_child_location() {
  let fixture = Fixture::new();
  for source in [
    "assert y >= 0",
    "assert absent == 1",
    "assert x",
    "by x: assert x > 0",
    "by x: datasignature",
  ] {
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    let output = fixture.run(&["--json", "-f", "parent.td"]);
    assert_eq!(
      output.status.code(),
      Some(1),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let emitted = lines(&output.stdout);
    assert_eq!(emitted[0]["result_type"], "LoadResult");
    assert!(
      emitted
        .iter()
        .skip(1)
        .all(|line| line.get("error").is_some()),
      "{source}: {emitted:?}"
    );
    assert!(
      String::from_utf8(output.stderr)
        .unwrap()
        .contains("child.td:1:")
    );
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  for source in ["assert x > 0", "datasignature"] {
    let output = fixture.run(&["--json", "-c", source]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
      lines(&output.stdout)
        .iter()
        .all(|line| line.get("error").is_some())
    );
  }
  for source in [
    "datasignature x",
    "datasignature, clear",
    "assert x > 0, detail",
    "assert x > 0 if x > 1",
    "assert x > 0 or x < 0",
    "assert lower(x) == 1",
  ] {
    let output = fixture.run(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      source,
      "-c",
      "export forbidden.csv",
    ]);
    assert_eq!(output.status.code(), Some(2), "{source}");
    let emitted = lines(&output.stdout);
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[0]["result_type"], "LoadResult");
    assert_eq!(emitted[1]["error"]["type"], "ParseError");
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
}
