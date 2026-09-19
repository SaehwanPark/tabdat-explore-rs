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
      "tabdat-runtime-drop-{0}-{1}-{2}",
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
fn drop_requires_an_active_dataset() {
  let mut session = Session::new();
  assert_eq!(
    session
      .execute(Command::Drop {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "drop" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn drop_rejects_an_empty_variable_list() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  assert_eq!(
    session
      .execute(Command::Drop {
        variables: Vec::new()
      })
      .unwrap_err(),
    RuntimeError::DropNoVariables
  );
}

#[test]
fn drop_projects_remaining_columns_and_preserves_rows_and_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("drop `bmi`").expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Drop(drop) = result else {
    panic!("drop should return a Drop result");
  };
  assert_eq!(drop.dataset.source, fixture.parquet);
  assert_eq!(drop.dataset.row_count, 3);
  assert_eq!(
    drop
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "sex", "cost"]
  );

  let preview = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("head should inspect the projected relation");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "sex", "cost"]);
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::SignedInteger(30),
        CellValue::Text("F".to_owned()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1000,
        },
      ],
      vec![
        CellValue::SignedInteger(42),
        CellValue::Text("M".to_owned()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1500,
        },
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Text("F".to_owned()),
        CellValue::Null,
      ],
    ]
  );
}

#[test]
fn drop_treats_repeated_and_quoted_names_as_one_schema_selection() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("drop age `age`").expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Drop(drop) = result else {
    panic!("drop should return a Drop result");
  };
  assert_eq!(
    drop
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["bmi", "sex", "cost"]
  );
}

#[test]
fn drop_quotes_identifiers_with_embedded_double_quotes() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "embedded-quote.parquet",
    "SELECT 1 AS \"odd\"\"name\", 2 AS keep_me",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted))
    .expect("quoted fixture should load");

  let result = session
    .execute(parse_command(r#"drop `odd"name`"#).expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Drop(drop) = result else {
    panic!("drop should return a Drop result");
  };
  assert_eq!(
    drop
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["keep_me"]
  );
}

#[test]
fn drop_unknown_variable_preserves_state_and_relation() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  assert_eq!(
    session
      .execute(Command::Drop {
        variables: vec!["missing".to_owned(), "absent".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::DropUnknownVariable {
      variables: vec!["missing".to_owned(), "absent".to_owned()],
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));

  let result = session
    .execute(parse_command("head 1").expect("head should parse"))
    .expect("the prior relation should remain active");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
}

#[test]
fn drop_all_columns_is_rejected_atomically() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("drop age bmi sex cost").expect("projection should parse"))
      .unwrap_err(),
    RuntimeError::DropWouldRemoveEveryColumn
  );
  assert_eq!(session.active_dataset(), Some(&before));
  let result = session
    .execute(parse_command("head 1").expect("head should parse"))
    .expect("the prior relation should remain active");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
}

#[test]
fn repeated_drops_compose_in_schema_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("drop bmi").expect("first drop should parse"))
    .expect("first drop should execute");
  let result = session
    .execute(parse_command("drop cost").expect("second drop should parse"))
    .expect("second drop should execute");
  let ExecutionResult::Drop(drop) = result else {
    panic!("drop should return a Drop result");
  };
  assert_eq!(
    drop
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "sex"]
  );
  assert_eq!(drop.dataset.row_count, 3);
}

#[test]
fn drop_preserves_empty_relations() {
  let fixture = Fixture::new();
  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS INTEGER) AS age, CAST(NULL AS VARCHAR) AS sex WHERE false",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(empty))
    .expect("empty fixture should load");

  let result = session
    .execute(parse_command("drop sex").expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Drop(drop) = result else {
    panic!("drop should return a Drop result");
  };
  assert_eq!(drop.dataset.row_count, 0);
  assert_eq!(drop.dataset.columns[0].name, "age");
}
