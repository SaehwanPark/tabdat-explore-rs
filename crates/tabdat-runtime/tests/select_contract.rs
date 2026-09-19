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
      "tabdat-runtime-select-{0}-{1}-{2}",
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
fn select_requires_an_active_dataset() {
  let mut session = Session::new();
  assert_eq!(
    session
      .execute(Command::Select {
        variables: vec!["age".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "select" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn select_rejects_an_empty_typed_variable_list() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  assert_eq!(
    session
      .execute(Command::Select {
        variables: Vec::new(),
      })
      .unwrap_err(),
    RuntimeError::SelectNoVariables
  );
}

#[test]
fn select_projects_requested_order_and_preserves_rows_nulls_and_source() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("select sex `age`").expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(select.dataset.source, fixture.parquet);
  assert_eq!(select.dataset.row_count, 3);
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["sex", "age"]
  );

  let preview = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("head should inspect the projected relation");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["sex", "age"]);
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::Text("F".to_owned()),
        CellValue::SignedInteger(30)
      ],
      vec![
        CellValue::Text("M".to_owned()),
        CellValue::SignedInteger(42)
      ],
      vec![
        CellValue::Text("F".to_owned()),
        CellValue::SignedInteger(54)
      ],
    ]
  );
}

#[test]
fn select_preserves_duplicate_requests_and_allows_all_columns() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("select age bmi sex cost").expect("all-column projection should parse"))
    .expect("select should allow retaining every column");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost"]
  );

  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load again");

  let result = session
    .execute(parse_command("select age age").expect("projection should parse"))
    .expect("duplicate projection should execute");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "age_1"]
  );

  let result = session
    .execute(parse_command("select age age_1").expect("projected names should parse"))
    .expect("all projected columns should be selectable");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "age_1"]
  );
}

#[test]
fn select_quotes_identifiers_with_embedded_double_quotes() {
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
    .execute(parse_command(r#"select `odd"name`"#).expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(select.dataset.columns[0].name, "odd\"name");
}

#[test]
fn select_unknown_variable_preserves_state_and_relation() {
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
      .execute(Command::Select {
        variables: vec!["missing".to_owned(), "absent".to_owned()],
      })
      .unwrap_err(),
    RuntimeError::SelectUnknownVariable {
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
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
}

#[test]
fn repeated_selects_compose_in_requested_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("select cost age").expect("first projection should parse"))
    .expect("first projection should execute");

  let result = session
    .execute(parse_command("select age cost").expect("second projection should parse"))
    .expect("second projection should execute");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "cost"]
  );
  assert_eq!(select.dataset.row_count, 3);
}

#[test]
fn select_preserves_empty_relations() {
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
    .execute(parse_command("select sex age").expect("projection should parse"))
    .expect("projection should execute");
  let ExecutionResult::Select(select) = result else {
    panic!("select should return a Select result");
  };
  assert_eq!(select.dataset.row_count, 0);
  assert_eq!(
    select
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["sex", "age"]
  );
}
