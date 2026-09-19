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
      "tabdat-runtime-replace-{0}-{1}-{2}",
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
fn replace_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("replace age = age + 1").expect("replace should parse"))
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "replace" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn parsed_numeric_replace_preserves_schema_position_rows_and_nulls() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("replace cost = cost / 2 if age >= 42").unwrap())
    .expect("numeric replacement should execute");
  let ExecutionResult::Replace(replace) = result else {
    panic!("replace should return a Replace result");
  };
  assert_eq!(replace.dataset.source, fixture.parquet);
  assert_eq!(replace.dataset.row_count, 3);
  assert_eq!(
    replace
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost"]
  );
  assert_eq!(replace.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(replace.dataset.lazy_engine, None);

  let result = session
    .execute(parse_command("head 3").unwrap())
    .expect("replaced relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(preview.rows[0][3], CellValue::Float(100.0));
  assert_eq!(preview.rows[1][3], CellValue::Float(75.0));
  assert_eq!(preview.rows[2][3], CellValue::Null);
}

#[test]
fn replace_supports_string_values_and_explicit_nulls() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(parse_command("replace sex = 'X' if sex == 'F'").unwrap())
    .expect("string replacement should execute");
  let result = session
    .execute(parse_command("replace cost = null if age == 30").unwrap())
    .expect("null replacement should execute");
  let ExecutionResult::Replace(replace) = result else {
    panic!("replace should return a Replace result");
  };
  assert_eq!(replace.dataset.columns[2].name, "sex");
  assert_eq!(replace.dataset.columns[2].data_type, "VARCHAR");

  let result = session
    .execute(parse_command("head 3").unwrap())
    .expect("replaced relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
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
        CellValue::Text("X".to_owned()),
        CellValue::Null,
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
        CellValue::Text("X".to_owned()),
        CellValue::Null,
      ],
    ]
  );
}

#[test]
fn false_and_null_predicates_preserve_existing_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(parse_command("replace cost = 0 if age < 0").unwrap())
    .expect("false predicate should execute");
  session
    .execute(parse_command("replace cost = 0 if null").unwrap())
    .expect("null predicate should execute");

  let result = session
    .execute(parse_command("head 3").unwrap())
    .expect("relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview.rows[0][3],
    CellValue::Decimal {
      width: 11,
      scale: 1,
      value: 1000,
    }
  );
  assert_eq!(
    preview.rows[1][3],
    CellValue::Decimal {
      width: 11,
      scale: 1,
      value: 1500,
    }
  );
  assert_eq!(preview.rows[2][3], CellValue::Null);
}

#[test]
fn replace_supports_quoted_identifiers_and_empty_relations() {
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
    .execute(parse_command(r#"replace `Age Value` = `Age Value` + 1 if `Age Value` >= 0"#).unwrap())
    .expect("quoted replacement should execute on an empty relation");
  let ExecutionResult::Replace(replace) = result else {
    panic!("replace should return a Replace result");
  };
  assert_eq!(replace.dataset.source, quoted);
  assert_eq!(replace.dataset.row_count, 0);
  assert_eq!(
    replace
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["Age Value", "a\"b"]
  );
}

#[test]
fn invalid_replace_forms_preserve_metadata_and_rows() {
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
      "replace missing = 1",
      RuntimeError::ReplaceTargetUnknownVariable {
        variable: "missing".to_owned(),
      },
    ),
    (
      "replace cost = absent + 1",
      RuntimeError::ReplaceUnknownVariable {
        variables: vec!["absent".to_owned()],
      },
    ),
    (
      "replace cost = 'invalid'",
      RuntimeError::ReplaceTypeMismatch {
        message: "replace target cost is numeric but expression is string".to_owned(),
      },
    ),
    (
      "replace cost = 0 if age",
      RuntimeError::ReplaceRequiresBoolean,
    ),
    (
      "replace cost = abs(age)",
      RuntimeError::ReplaceUnsupportedExpression {
        message: "replace does not support function calls".to_owned(),
      },
    ),
    (
      "replace cost = age > 0",
      RuntimeError::ReplaceTypeMismatch {
        message: "replace target cost is numeric but expression is boolean".to_owned(),
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
    .execute(parse_command("head 3").unwrap())
    .expect("failed replacements should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview.rows[0][3],
    CellValue::Decimal {
      width: 4,
      scale: 1,
      value: 1000,
    }
  );
  assert_eq!(
    preview.rows[1][3],
    CellValue::Decimal {
      width: 4,
      scale: 1,
      value: 1500,
    }
  );
  assert_eq!(preview.rows[2][3], CellValue::Null);
}

#[test]
fn string_target_rejects_numeric_assignment_atomically() {
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
      .execute(parse_command("replace sex = 0").unwrap())
      .unwrap_err(),
    RuntimeError::ReplaceTypeMismatch {
      message: "replace target sex is string but expression is numeric".to_owned(),
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn replace_can_be_applied_repeatedly_after_projection() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(Command::Select {
      variables: vec!["age".to_owned(), "cost".to_owned()],
    })
    .expect("select should leave an active relation");
  session
    .execute(parse_command("replace age = age + 1").unwrap())
    .expect("first replacement should execute");
  session
    .execute(parse_command("replace age = age + 1").unwrap())
    .expect("second replacement should execute");

  let result = session
    .execute(parse_command("head 3").unwrap())
    .expect("replaced relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "cost"]);
  assert_eq!(
    preview.rows[0][0],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 32,
    }
  );
  assert_eq!(
    preview.rows[1][0],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 44,
    }
  );
  assert_eq!(
    preview.rows[2][0],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 56,
    }
  );
}
