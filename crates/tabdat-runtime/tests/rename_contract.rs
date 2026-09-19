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
      "tabdat-runtime-rename-{0}-{1}-{2}",
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
fn rename_requires_an_active_dataset() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("rename age years").expect("rename should parse"))
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "rename" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn rename_preserves_schema_position_type_rows_nulls_and_source() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("rename sex gender").expect("rename should parse"))
    .expect("rename should execute");
  let ExecutionResult::Rename(rename) = result else {
    panic!("rename should return a Rename result");
  };
  assert_eq!(rename.dataset.source, fixture.parquet);
  assert_eq!(rename.dataset.row_count, 3);
  assert_eq!(
    rename
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "gender", "cost"]
  );
  assert_eq!(rename.dataset.columns[2].data_type, "VARCHAR");
  assert_eq!(rename.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(rename.dataset.lazy_engine, None);

  let result = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("renamed relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "gender", "cost"]);
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::SignedInteger(30),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 225,
        },
        CellValue::Text("F".to_owned()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1000,
        },
      ],
      vec![
        CellValue::SignedInteger(42),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 250,
        },
        CellValue::Text("M".to_owned()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1500,
        },
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 275,
        },
        CellValue::Text("F".to_owned()),
        CellValue::Null,
      ],
    ]
  );
}

#[test]
fn rename_quotes_identifiers_with_embedded_double_quotes_and_empty_relations() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT CAST(NULL AS INTEGER) AS \"Age Value\", CAST(NULL AS VARCHAR) AS \"a\"\"b\" WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted.clone()))
    .expect("quoted fixture should load");

  let result = session
    .execute(parse_command(r#"rename `a"b` `odd"name`"#).expect("quoted rename should parse"))
    .expect("quoted rename should execute on an empty relation");
  let ExecutionResult::Rename(rename) = result else {
    panic!("rename should return a Rename result");
  };
  assert_eq!(rename.dataset.source, quoted);
  assert_eq!(rename.dataset.row_count, 0);
  assert_eq!(
    rename
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["Age Value", "odd\"name"]
  );
  assert_eq!(rename.dataset.columns[1].data_type, "VARCHAR");

  let result = session
    .execute(parse_command("head 1").expect("head should parse"))
    .expect("empty renamed relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert!(preview.rows.is_empty());
}

#[test]
fn rename_validation_failures_preserve_metadata_and_rows() {
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
      "rename age bmi",
      RuntimeError::RenameTargetExists {
        variable: "bmi".to_owned(),
      },
    ),
    (
      "rename age age",
      RuntimeError::RenameTargetExists {
        variable: "age".to_owned(),
      },
    ),
    (
      "rename missing age",
      RuntimeError::RenameUnknownVariable {
        variable: "missing".to_owned(),
      },
    ),
  ];
  for (command, expected) in failures {
    assert_eq!(
      session
        .execute(parse_command(command).expect("failure case should parse"))
        .unwrap_err(),
      expected
    );
    assert_eq!(session.active_dataset(), Some(&before));
  }

  let result = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("failed rename commands should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(preview.rows.len(), 3);
  assert_eq!(preview.rows[0][2], CellValue::Text("F".to_owned()));
}
