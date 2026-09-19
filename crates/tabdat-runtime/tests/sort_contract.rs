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
      "tabdat-runtime-sort-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("sort.parquet");
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
fn sort_requires_an_active_dataset() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("sort group_id").expect("sort should parse"))
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "sort" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn sort_preserves_schema_and_stably_orders_nulls_last() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("sort group_id label").expect("sort should parse"))
    .expect("sort should execute");
  let ExecutionResult::Sort(sort) = result else {
    panic!("sort should return a Sort result");
  };
  assert_eq!(sort.dataset.source, fixture.parquet);
  assert_eq!(sort.dataset.row_count, 4);
  assert_eq!(
    sort
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
  assert_eq!(sort.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(sort.dataset.lazy_engine, None);

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
        CellValue::SignedInteger(2),
        CellValue::Text("b".to_owned()),
        CellValue::Boolean(true),
        CellValue::SignedInteger(1),
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
fn sort_quotes_identifiers_and_avoids_ordinal_collisions_on_empty_relations() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT CAST(NULL AS INTEGER) AS \"a\"\"b\", CAST(NULL AS INTEGER) AS \"__tabdat_sort_ordinal\" WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted.clone()))
    .expect("quoted fixture should load");

  let result = session
    .execute(parse_command(r#"sort `a"b` `a"b`"#).expect("quoted sort should parse"))
    .expect("quoted sort should execute on an empty relation");
  let ExecutionResult::Sort(sort) = result else {
    panic!("sort should return a Sort result");
  };
  assert_eq!(sort.dataset.source, quoted);
  assert_eq!(sort.dataset.row_count, 0);
  assert_eq!(
    sort
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["a\"b", "__tabdat_sort_ordinal"]
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
fn sort_validation_failures_preserve_metadata_and_rows() {
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
      Command::Sort {
        variables: Vec::new(),
      },
      RuntimeError::SortNoVariables,
    ),
    (
      parse_command("sort missing group_id").expect("unknown-variable case should parse"),
      RuntimeError::SortUnknownVariable {
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
    .expect("failed sort commands should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(2));
  assert_eq!(preview.rows[1][0], CellValue::SignedInteger(1));
}
