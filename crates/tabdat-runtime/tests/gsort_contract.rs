use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session};

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
      "tabdat-runtime-gsort-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("gsort.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (2, 'b', true, 1), (1, 'a', false, 2), (1, 'a', true, 3), (CAST(NULL AS INTEGER), 'a', false, 4)) AS rows(group_id, label, flag, row_id)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn command_for(&self, path: PathBuf) -> Command {
    Command::Use {
      source: DataSource::LocalPath(path.to_string_lossy().into_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    }
  }

  fn command(&self) -> Command {
    self.command_for(self.parquet.clone())
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

#[test]
fn gsort_requires_an_active_dataset() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("gsort -group_id").expect("gsort should parse"))
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "gsort" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn gsort_preserves_schema_and_stably_orders_mixed_directions() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("gsort -group_id +label").expect("gsort should parse"))
    .expect("gsort should execute");
  let ExecutionResult::Gsort(gsort) = result else {
    panic!("gsort should return a Gsort result");
  };
  assert_eq!(gsort.dataset.source, fixture.parquet);
  assert_eq!(gsort.dataset.row_count, 4);
  assert_eq!(
    gsort
      .dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![
      ("group_id", "INTEGER"),
      ("label", "VARCHAR"),
      ("flag", "BOOLEAN"),
      ("row_id", "INTEGER"),
    ]
  );
  assert_eq!(gsort.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(gsort.dataset.lazy_engine, None);

  let result = session
    .execute(parse_command("head 10").expect("head should parse"))
    .expect("sorted relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["group_id", "label", "flag", "row_id"]);
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::SignedInteger(2),
        CellValue::Text("b".to_owned()),
        CellValue::Boolean(true),
        CellValue::SignedInteger(1),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("a".to_owned()),
        CellValue::Boolean(false),
        CellValue::SignedInteger(2),
      ],
      vec![
        CellValue::SignedInteger(1),
        CellValue::Text("a".to_owned()),
        CellValue::Boolean(true),
        CellValue::SignedInteger(3),
      ],
      vec![
        CellValue::Null,
        CellValue::Text("a".to_owned()),
        CellValue::Boolean(false),
        CellValue::SignedInteger(4),
      ],
    ]
  );
}

#[test]
fn gsort_quotes_identifiers_and_avoids_ordinal_collisions_on_empty_relations() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT CAST(NULL AS INTEGER) AS \"a\"\"b\", CAST(NULL AS INTEGER) AS \"__TABDAT_SORT_ORDINAL\" WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted.clone()))
    .expect("quoted fixture should load");

  let result = session
    .execute(parse_command(r#"gsort `a"b` `a"b`"#).expect("quoted gsort should parse"))
    .expect("quoted gsort should execute on an empty relation");
  let ExecutionResult::Gsort(gsort) = result else {
    panic!("gsort should return a Gsort result");
  };
  assert_eq!(gsort.dataset.source, quoted);
  assert_eq!(gsort.dataset.row_count, 0);
  assert_eq!(
    gsort
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["a\"b", "__TABDAT_SORT_ORDINAL"]
  );

  let result = session
    .execute(parse_command("head 1").expect("head should parse"))
    .expect("empty sorted relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert!(preview.rows.is_empty());
}

#[test]
fn gsort_validation_failures_preserve_metadata_and_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  let failures = [
    (
      Command::Gsort { keys: Vec::new() },
      RuntimeError::GsortNoVariables,
    ),
    (
      parse_command("gsort -missing +group_id").expect("unknown-variable case should parse"),
      RuntimeError::GsortUnknownVariable {
        variables: vec!["missing".to_owned()],
      },
    ),
  ];
  for (command, expected) in failures {
    assert_eq!(session.execute(command).unwrap_err(), expected);
    assert_eq!(session.active_dataset(), Some(&before));
  }

  let result = session
    .execute(parse_command("head 4").expect("head should parse"))
    .expect("failed gsort commands should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(2));
  assert_eq!(preview.rows[1][0], CellValue::SignedInteger(1));
}
