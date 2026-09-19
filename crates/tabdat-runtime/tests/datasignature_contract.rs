use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine, parse_command};
use tabdat_runtime::{DatasignatureResult, ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-datasignature-{}-{nonce}-{fixture_id}",
      std::process::id()
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    Self { root }
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
    lazy_engine: None::<LazyEngine>,
    delimiter: None,
    has_header: None,
  }
}

fn sample_query() -> &'static str {
  "SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)"
}

#[test]
fn datasignature_requires_active_dataset_without_initializing_a_session() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(Command::Datasignature).unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "datasignature"
    }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn datasignature_matches_the_pinned_sample_digest_and_preserves_state() {
  let fixture = Fixture::new();
  let path = fixture.write_parquet("patients.parquet", sample_query());
  let mut session = Session::new();
  let load = session
    .execute(use_command(&path))
    .expect("sample Parquet should load");
  let before = session.active_dataset().cloned();

  let result = session
    .execute(Command::Datasignature)
    .expect("datasignature should succeed");
  let ExecutionResult::Datasignature(result) = result else {
    panic!("datasignature should return its owned result");
  };
  assert_eq!(
    result,
    DatasignatureResult {
      algorithm: "sha256".to_owned(),
      signature: "0b61cef05ab04301df893652f666b6ed974668f0cd214ebae17cfff02a6b3aad".to_owned(),
      row_count: 3,
      column_count: 4,
    }
  );
  assert_eq!(session.active_dataset(), before.as_ref());

  let repeated = session
    .execute(Command::Datasignature)
    .expect("a repeated datasignature should succeed");
  assert_eq!(repeated, ExecutionResult::Datasignature(result));
  let _ = load;
}

#[test]
fn datasignature_matches_temporal_nonfinite_decimal_list_and_null_values() {
  let fixture = Fixture::new();
  let path = fixture.write_parquet(
    "complex.parquet",
    "SELECT * FROM (VALUES (date '2024-01-01', timestamptz '2024-01-01 12:34:56.123456+00', cast('NaN' AS DOUBLE), cast(1.20 AS DECIMAL(10, 2)), [1, 2]), (NULL, NULL, cast('Infinity' AS DOUBLE), NULL, NULL)) AS signature_data(day, observed_at, score, amount, tags)",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&path))
    .expect("complex Parquet should load");

  let result = session
    .execute(Command::Datasignature)
    .expect("complex datasignature should succeed");
  let ExecutionResult::Datasignature(result) = result else {
    panic!("datasignature should return its owned result");
  };
  assert_eq!(result.row_count, 2);
  assert_eq!(result.column_count, 5);
  assert_eq!(
    result.signature,
    "c2d343670c50720e4e4360322c980346b3eea1c0984aba4acb437aced8c816ea"
  );
}

#[test]
fn datasignature_accepts_an_empty_schema_and_changes_with_schema_or_row_order() {
  let fixture = Fixture::new();
  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS INTEGER) AS id WHERE FALSE",
  );
  let original = fixture.write_parquet(
    "original.parquet",
    "SELECT * FROM (VALUES (1, 'a'), (2, 'b')) AS values_data(id, label)",
  );
  let reordered = fixture.write_parquet(
    "reordered.parquet",
    "SELECT * FROM (VALUES (2, 'b'), (1, 'a')) AS values_data(id, label)",
  );
  let narrowed = fixture.write_parquet(
    "narrowed.parquet",
    "SELECT id FROM (VALUES (1, 'a'), (2, 'b')) AS values_data(id, label)",
  );

  let mut empty_session = Session::new();
  empty_session
    .execute(use_command(&empty))
    .expect("empty Parquet should load");
  let ExecutionResult::Datasignature(empty_result) = empty_session
    .execute(Command::Datasignature)
    .expect("empty datasignature should succeed")
  else {
    panic!("empty datasignature should return its owned result");
  };
  assert_eq!(empty_result.row_count, 0);
  assert_eq!(empty_result.column_count, 1);
  assert_eq!(
    empty_result.signature,
    "9ee1723d85281a5b7423fe90999069a22ac2a4be650ba1a2b39d94fdff23d295"
  );

  let mut original_session = Session::new();
  original_session
    .execute(use_command(&original))
    .expect("original Parquet should load");
  let original_result = original_session
    .execute(Command::Datasignature)
    .expect("original datasignature should succeed");

  let mut reordered_session = Session::new();
  reordered_session
    .execute(use_command(&reordered))
    .expect("reordered Parquet should load");
  let reordered_result = reordered_session
    .execute(Command::Datasignature)
    .expect("reordered datasignature should succeed");

  let mut narrowed_session = Session::new();
  narrowed_session
    .execute(use_command(&narrowed))
    .expect("narrowed Parquet should load");
  let narrowed_result = narrowed_session
    .execute(Command::Datasignature)
    .expect("narrowed datasignature should succeed");

  assert_ne!(original_result, reordered_result);
  assert_ne!(reordered_result, narrowed_result);
}

#[test]
fn datasignature_is_parsed_as_a_direct_zero_argument_command() {
  assert_eq!(
    parse_command(" DATASIGNATURE ").unwrap(),
    Command::Datasignature
  );
}
