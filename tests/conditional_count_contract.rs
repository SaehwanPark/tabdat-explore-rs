#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-count-if-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let reference: Value = serde_json::from_str(include_str!(
      "fixtures/conditional-frequency-reference.json"
    ))
    .unwrap();
    let mut csv = String::from("e,r,c,x,y\n");
    for record in reference["records"].as_array().unwrap() {
      csv.push_str(
        &record
          .as_array()
          .unwrap()
          .iter()
          .map(|value| match value {
            Value::Null => String::new(),
            Value::String(value) => value.clone(),
            value => value.to_string(),
          })
          .collect::<Vec<_>>()
          .join(","),
      );
      csv.push('\n');
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
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
fn count(session: &mut Session, source: &str) -> u64 {
  match execute(session, source) {
    ExecutionResult::Count(result) => result.row_count,
    other => panic!("{other:?}"),
  }
}
#[test]
fn selected_cohort_count_matches_independent_indices_not_category_or_model_samples() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(&mut session, "sql select * from active into cohort");
  execute(&mut session, "label variable r 'Category'");
  execute(&mut session, "regress y x");
  let reference: Value = serde_json::from_str(include_str!(
    "fixtures/conditional-frequency-reference.json"
  ))
  .unwrap();
  let expected = reference["selected_indices"].as_array().unwrap().len() as u64;
  assert_eq!(expected, 6);
  let state = session.state();
  let labels = session.active_label_metadata().unwrap().clone();
  let model = session.last_regression().unwrap().clone();
  let signature = execute(&mut session, "datasignature");
  let inference = execute(&mut session, "lincom x");
  for (source, expected) in [
    ("count", 8),
    ("count if e == 1", expected),
    ("count if e == null", 1),
    ("count if e != null", 7),
    ("count if null", 0),
    ("count if x/0 > 1", 0),
    ("count if r == 'A'", 3),
    ("count if x > 20", 0),
  ] {
    assert_eq!(count(&mut session, source), expected);
    assert_eq!(session.state(), state);
    assert_eq!(session.active_label_metadata().unwrap(), &labels);
    assert_eq!(session.last_regression().unwrap(), &model);
    assert_eq!(execute(&mut session, "datasignature"), signature);
    assert_eq!(execute(&mut session, "lincom x"), inference);
  }
  let table = execute(&mut session, "tabulate r c if e == 1");
  if let ExecutionResult::Tabulate(table) = table {
    assert_eq!(table.rows.len(), 2);
    let total: i128 = table
      .rows
      .iter()
      .flat_map(|row| &row[1..])
      .map(|cell| match cell {
        tabdat_domain::CellValue::SignedInteger(value) => *value,
        other => panic!("{other:?}"),
      })
      .sum();
    assert_eq!(total, 3); // Selected category-complete rows, not the six-row cohort.
  } else {
    panic!("{table:?}");
  }
  execute(&mut session, "use cohort");
  assert_eq!(count(&mut session, "count"), 8);
}
#[test]
fn predicates_validate_without_mutation_and_boolean_empty_native_errors_are_bounded() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  let signature = execute(&mut session, "datasignature");
  for (source, message) in [
    ("count if absent > 1", "count unknown variable"),
    ("count if x", "count predicate requires boolean expression"),
    ("count if r > 1", "type"),
    (
      "count if lower(r) == 'a'",
      "count predicate does not support function calls",
    ),
    ("count if r == '\0'", "count failed"),
  ] {
    let error = session.execute(parse_command(source).unwrap()).unwrap_err();
    assert!(error.to_string().contains(message), "{error}");
    assert_eq!(execute(&mut session, "datasignature"), signature);
  }
  execute(
    &mut session,
    "sql select * from (values (true,cast('NaN' as double)),(false,NULL),(NULL,NULL),(true,cast('Infinity' as double))) as t(b,x) into flags",
  );
  assert_eq!(count(&mut session, "count if b"), 2);
  let summary = execute(&mut session, "summarize x if b");
  if let ExecutionResult::Summarize(result) = summary {
    assert_eq!(result.rows[0].count, 0);
  } else {
    panic!("{summary:?}");
  }
  assert_eq!(count(&mut session, "count"), 4);
  execute(
    &mut session,
    "sql select * from active where false into empty",
  );
  assert_eq!(count(&mut session, "count if b"), 0);
  let mut empty = Session::new();
  assert!(matches!(
    empty
      .execute(parse_command("count if absent > 0").unwrap())
      .unwrap_err(),
    tabdat_domain::RuntimeError::NoActiveDataset { command: "count" }
  ));
}
#[test]
fn quoted_wide_integer_and_native_nonfinite_predicates_keep_exact_read_only_counts() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  execute(
    &mut session,
    "sql select * from (values (9007199254740993::bigint,cast('NaN' as double)),(9007199254740992::bigint,cast('Infinity' as double)),(-9007199254740993::bigint,cast('-Infinity' as double)),(NULL,NULL)) as t(\"wide 雪\",n) into native",
  );
  let signature = execute(&mut session, "datasignature");
  for (source, expected) in [
    ("count if `wide 雪` == 9007199254740993", 1),
    ("count if `wide 雪` == -9007199254740993", 1),
    ("count if `wide 雪` == null", 1),
    ("count if n > 0", 2),
    ("count if n == null", 1),
  ] {
    assert_eq!(count(&mut session, source), expected);
    assert_eq!(execute(&mut session, "datasignature"), signature);
  }
}

#[test]
fn parser_admission_error_streams_file_guards_and_interactive_recovery_remain_bounded() {
  let fixture = Fixture::new();
  let query = parse_command("count if e == 1").unwrap();
  assert_eq!(query.name(), "count");
  assert_eq!(query.effects(), &[tabdat_domain::CommandEffect::Read]);
  assert_eq!(
    query.requirements(),
    parse_command("count").unwrap().requirements()
  );
  assert!(tabdat_reporting::json_results::supports_batch_command(
    &query
  ));
  assert!(tabdat_reporting::json_results::supports_script_command(
    &query
  ));
  let grouped = tabdat_domain::Command::By {
    command: tabdat_domain::ByCommand {
      groups: vec!["r".into()],
      command: Box::new(query),
    },
  };
  assert!(!tabdat_reporting::json_results::supports_batch_command(
    &grouped
  ));
  assert!(!tabdat_reporting::json_results::supports_script_command(
    &grouped
  ));
  for (source, exit, message) in [
    ("count if", 2, "missing expression after if"),
    ("count x if e == 1", 2, "count does not accept arguments"),
    (
      "count if e == 1, missing",
      2,
      "count does not accept arguments",
    ),
    ("count if absent > 0", 1, "count unknown variable"),
    (
      "count if x",
      1,
      "count predicate requires boolean expression",
    ),
    ("count if r > 1", 1, "type"),
    (
      "count if lower(r) == 'a'",
      1,
      "count predicate does not support function calls",
    ),
    ("count if r == '\0'", 1, "count failed"),
  ] {
    fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    for mode in ["--json", "--text"] {
      // NUL is a native preparation failure in files, not a legal OS argument.
      if !source.contains('\0') {
        let batch = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
          .current_dir(&fixture.0)
          .args([
            mode,
            "-c",
            "use data.csv",
            "-c",
            source,
            "-c",
            "export forbidden.csv",
          ])
          .output()
          .unwrap();
        assert_eq!(batch.status.code(), Some(exit), "{source}");
        assert!(String::from_utf8_lossy(&batch.stderr).contains(message));
        assert!(!fixture.0.join("forbidden.csv").exists());
      }
      let script = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
        .current_dir(&fixture.0)
        .args([mode, "-f", "parent.td"])
        .output()
        .unwrap();
      assert_eq!(script.status.code(), Some(exit), "{source}");
      assert!(String::from_utf8_lossy(&script.stderr).contains(message));
      assert!(String::from_utf8_lossy(&script.stderr).contains("child.td:1:"));
      assert!(!fixture.0.join("forbidden.csv").exists());
      if mode == "--json" {
        let rows: Vec<Value> = String::from_utf8(script.stdout)
          .unwrap()
          .lines()
          .map(|line| serde_json::from_str(line).unwrap())
          .collect();
        assert_eq!(rows[0]["result_type"], "LoadResult");
        assert!(rows.iter().all(|row| row["result_type"] != "CountResult"));
      }
    }
  }
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .env("TERM", "dumb")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(b"use data.csv\ncount if e == 1\ncount if absent > 0\ncount if e == null\ncount\nexport recovered.csv\nexit\n").unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success());
  assert!(String::from_utf8_lossy(&output.stderr).contains("count unknown variable: absent"));
  assert_eq!(
    fs::read(fixture.0.join("recovered.csv")).unwrap(),
    fs::read(fixture.0.join("data.csv")).unwrap()
  );
}

#[test]
fn actual_json_text_nested_analysis_and_persistence_streams_preserve_all_rows() {
  let fixture = Fixture::new();
  let sources = [
    "use data.csv",
    "count if e == 1",
    "tabulate r c if e == 1",
    "summarize x if e == 1",
    "regress y x",
    "predict fitted",
    "count if fitted > 5",
    "save original.parquet, replace",
    "use original.parquet",
    "count",
  ];
  fs::write(fixture.0.join("child.td"), sources[1..].join("\n")).unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  for mode in ["--json", "--text"] {
    let mut args = vec![mode];
    for source in sources {
      args.extend(["-c", source]);
    }
    let batch = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&fixture.0)
      .args(&args)
      .output()
      .unwrap();
    assert!(
      batch.status.success(),
      "{}",
      String::from_utf8_lossy(&batch.stderr)
    );
    let script = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&fixture.0)
      .args([mode, "-f", "parent.td"])
      .output()
      .unwrap();
    assert!(
      script.status.success(),
      "{}",
      String::from_utf8_lossy(&script.stderr)
    );
    assert_eq!(script.stdout, batch.stdout);
    if mode == "--json" {
      let rows: Vec<Value> = String::from_utf8(batch.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
      assert_eq!(rows[1]["data"]["row_count"], 6);
      assert_eq!(rows.last().unwrap()["data"]["row_count"], 8);
    }
  }
}
