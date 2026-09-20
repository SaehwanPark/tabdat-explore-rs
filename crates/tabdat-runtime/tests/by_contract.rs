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
      "tabdat-runtime-by-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("by.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES
          ('F', 30, 10.0, 'a'),
          ('F', 50, 20.0, 'b'),
          ('M', 42, 30.0, 'c'),
          (CAST(NULL AS VARCHAR), 10, CAST(NULL AS DOUBLE), CAST(NULL AS VARCHAR))
        ) AS patients(sex, age, cost, note)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn command(&self) -> Command {
    Command::Use {
      source: DataSource::LocalPath(self.parquet.to_string_lossy().into_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    }
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn assert_float(value: &CellValue, expected: f64) {
  let CellValue::Float(actual) = value else {
    panic!("expected a floating-point cell, got {value:?}");
  };
  assert!(
    (*actual - expected).abs() < 0.000_001,
    "{actual} != {expected}"
  );
}

#[test]
fn by_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("by sex: count").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "by" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn by_summarize_returns_owned_grouped_rows_and_preserves_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label variable sex \"Sex\"").unwrap())
    .expect("group label should execute");
  let before = session
    .active_dataset()
    .expect("load should publish active metadata")
    .clone();
  let labels_before = session
    .active_label_metadata()
    .expect("label should be active")
    .clone();

  let result = session
    .execute(parse_command("by sex: summarize age").unwrap())
    .expect("grouped summarize should execute");
  let ExecutionResult::By(result) = result else {
    panic!("by should return a grouped table result");
  };
  assert_eq!(result.headers, vec!["sex", "mean_age"]);
  assert_eq!(result.rows.len(), 3);
  assert_eq!(result.rows[0][0], CellValue::Text("F".to_owned()));
  assert_float(&result.rows[0][1], 40.0);
  assert_eq!(result.rows[1][0], CellValue::Text("M".to_owned()));
  assert_float(&result.rows[1][1], 42.0);
  assert_eq!(result.rows[2][0], CellValue::Null);
  assert_float(&result.rows[2][1], 10.0);
  assert_eq!(session.active_dataset(), Some(&before));
  assert_eq!(session.active_label_metadata(), Some(&labels_before));
}

#[test]
fn by_summarize_defaults_to_numeric_non_group_columns() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("by sex: summarize").unwrap())
    .expect("default grouped summarize should execute");
  let ExecutionResult::By(result) = result else {
    panic!("by should return a grouped table result");
  };
  assert_eq!(result.headers, vec!["sex", "mean_age", "mean_cost"]);
  assert_eq!(result.rows[0][0], CellValue::Text("F".to_owned()));
  assert_float(&result.rows[0][1], 40.0);
  assert_float(&result.rows[0][2], 15.0);
  assert_eq!(result.rows[1][0], CellValue::Text("M".to_owned()));
  assert_float(&result.rows[1][1], 42.0);
  assert_float(&result.rows[1][2], 30.0);
  assert_eq!(result.rows[2][0], CellValue::Null);
  assert_float(&result.rows[2][1], 10.0);
  assert_eq!(result.rows[2][2], CellValue::Null);
}

#[test]
fn by_count_includes_null_groups_and_counts_every_row() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("by sex: count").unwrap())
    .expect("grouped count should execute");
  let ExecutionResult::By(result) = result else {
    panic!("by should return a grouped table result");
  };
  assert_eq!(result.headers, vec!["sex", "Count"]);
  assert_eq!(
    result.rows,
    vec![
      vec![CellValue::Text("F".to_owned()), CellValue::SignedInteger(2),],
      vec![CellValue::Text("M".to_owned()), CellValue::SignedInteger(1),],
      vec![CellValue::Null, CellValue::SignedInteger(1)],
    ]
  );
}

#[test]
fn by_validation_and_deferred_children_preserve_active_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("by missing: count").unwrap())
      .unwrap_err(),
    RuntimeError::ByUnknownVariable {
      variables: vec!["missing".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("by sex: summarize missing").unwrap())
      .unwrap_err(),
    RuntimeError::ByUnknownVariable {
      variables: vec!["missing".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("by sex: summarize note").unwrap())
      .unwrap_err(),
    RuntimeError::BySummarizeRequiresNumeric {
      variables: vec!["note".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("by sex: describe").unwrap())
      .unwrap_err(),
    RuntimeError::ByUnsupportedCommand
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn by_default_summarize_reports_when_no_numeric_non_group_column_exists() {
  let fixture = Fixture::new();
  let text_fixture = fixture.root.join("text-only.parquet");
  let connection = Connection::open_in_memory().expect("text fixture connection should open");
  let text_fixture_string = text_fixture.to_string_lossy().into_owned();
  connection
    .execute(
      "COPY (SELECT * FROM (VALUES ('F', 'a'), ('M', 'b')) AS observations(sex, note)) TO ? (FORMAT PARQUET)",
      [&text_fixture_string],
    )
    .expect("text-only Parquet should be written");

  let mut session = Session::new();
  session
    .execute(Command::Use {
      source: DataSource::LocalPath(text_fixture.to_string_lossy().into_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    })
    .expect("text-only fixture should load");

  assert_eq!(
    session
      .execute(parse_command("by sex: summarize").unwrap())
      .unwrap_err(),
    RuntimeError::BySummarizeNoNumericColumns
  );
}
