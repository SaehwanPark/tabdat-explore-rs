#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::Command,
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
