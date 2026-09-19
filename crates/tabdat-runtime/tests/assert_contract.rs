use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::{AssertResult, ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-assert-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("patients.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn write_parquet(&self, name: &str, query: &str) -> PathBuf {
    let path = self.root.join(name);
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = path.to_string_lossy().into_owned();
    connection
      .execute(
        &format!("COPY ({query}) TO ? (FORMAT PARQUET)"),
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    path
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn use_command(path: &Path) -> Command {
  Command::Use {
    source: DataSource::LocalPath(path.to_string_lossy().into_owned()),
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
    delimiter: None,
    has_header: None,
  }
}

#[test]
fn assert_requires_an_active_dataset() {
  let mut session = Session::new();
  assert_eq!(
    session
      .execute(parse_command("assert age > 0").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "assert" }
  );
}

#[test]
fn assert_passes_and_preserves_active_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  let result = session
    .execute(parse_command("assert age > 0").unwrap())
    .expect("positive ages should pass");
  assert_eq!(
    result,
    ExecutionResult::Assert(AssertResult {
      checked: 3,
      failed: 0,
    })
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn assert_counts_false_and_null_rows_and_preserves_state_on_failure() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("assert cost > 0").unwrap())
      .unwrap_err(),
    RuntimeError::AssertSemanticFailure {
      checked: 3,
      failed: 1,
    }
  );
  assert_eq!(
    session
      .execute(parse_command("assert cost > 0").unwrap())
      .unwrap_err()
      .to_string(),
    "assertion failed: 1 of 3 rows failed"
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn assert_validates_expression_domain_and_unknown_variables_before_querying() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("assert age").unwrap())
      .unwrap_err(),
    RuntimeError::AssertRequiresBoolean
  );
  assert_eq!(
    session
      .execute(parse_command("assert missing > 0").unwrap())
      .unwrap_err(),
    RuntimeError::AssertUnknownVariable {
      variables: vec!["missing".to_owned()]
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn assert_supports_null_comparisons_arithmetic_and_quoted_identifiers() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT 1 AS \"age value\", CAST(NULL AS INTEGER) AS value WHERE TRUE",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&quoted))
    .expect("quoted fixture should load");

  let result = session
    .execute(parse_command("assert `age value` + 1 > 1").unwrap())
    .expect("arithmetic predicate should pass");
  assert_eq!(
    result,
    ExecutionResult::Assert(AssertResult {
      checked: 1,
      failed: 0,
    })
  );

  assert_eq!(
    session
      .execute(parse_command("assert value == null").unwrap())
      .expect("NULL equality should pass"),
    ExecutionResult::Assert(AssertResult {
      checked: 1,
      failed: 0,
    })
  );
}

#[test]
fn assert_empty_dataset_passes() {
  let fixture = Fixture::new();
  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&empty))
    .expect("empty fixture should load");

  assert_eq!(
    session
      .execute(parse_command("assert value == null").unwrap())
      .expect("empty predicate should pass"),
    ExecutionResult::Assert(AssertResult {
      checked: 0,
      failed: 0,
    })
  );
}
