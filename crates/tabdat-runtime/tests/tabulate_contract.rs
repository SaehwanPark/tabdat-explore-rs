use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, TabulateCommand, parse_command};
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
      "tabdat-runtime-tabulate-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("tabulate.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (1, 'F', 30), (2, 'F', 30), (3, 'F', 54), (4, 'M', 42), (5, CAST(NULL AS VARCHAR), 30)) AS patients(row_id, sex, age)) TO ? (FORMAT PARQUET)",
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
fn tabulate_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("tabulate sex").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "tabulate"
    }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn tabulate_one_way_and_two_way_frequency_results_are_ordered_and_owned() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let one_way = session
    .execute(parse_command("tabulate sex").unwrap())
    .expect("one-way table should execute");
  let ExecutionResult::Tabulate(one_way) = one_way else {
    panic!("one-way tabulate should return a table result");
  };
  assert_eq!(one_way.headers, vec!["sex", "Count", "Percent"]);
  assert_eq!(
    one_way.rows[0][..2],
    [CellValue::Text("F".to_owned()), CellValue::SignedInteger(3)]
  );
  assert_float(&one_way.rows[0][2], 75.0);
  assert_eq!(
    one_way.rows[1][..2],
    [CellValue::Text("M".to_owned()), CellValue::SignedInteger(1)]
  );
  assert_float(&one_way.rows[1][2], 25.0);

  let two_way = session
    .execute(parse_command("tabulate sex age, row col").unwrap())
    .expect("two-way table should execute");
  let ExecutionResult::Tabulate(two_way) = two_way else {
    panic!("two-way tabulate should return a table result");
  };
  assert_eq!(
    two_way.headers,
    vec![
      "sex", "30 Count", "30 Row %", "30 Col %", "42 Count", "42 Row %", "42 Col %", "54 Count",
      "54 Row %", "54 Col %",
    ]
  );
  assert_eq!(two_way.rows[0][0], CellValue::Text("F".to_owned()));
  assert_eq!(two_way.rows[0][1], CellValue::SignedInteger(2));
  assert_float(&two_way.rows[0][2], 66.666_666_666_666_67);
  assert_float(&two_way.rows[0][3], 100.0);
  assert_eq!(two_way.rows[0][4], CellValue::SignedInteger(0));
  assert_float(&two_way.rows[0][5], 0.0);
  assert_float(&two_way.rows[0][6], 0.0);
  assert_eq!(two_way.rows[0][7], CellValue::SignedInteger(1));
  assert_float(&two_way.rows[0][8], 33.333_333_333_333_33);
  assert_float(&two_way.rows[0][9], 100.0);
  assert_eq!(two_way.rows[1][0], CellValue::Text("M".to_owned()));
  assert_eq!(two_way.rows[1][1], CellValue::SignedInteger(0));
  assert_eq!(two_way.rows[1][4], CellValue::SignedInteger(1));
  assert_eq!(two_way.rows[1][7], CellValue::SignedInteger(0));
  assert_float(&two_way.rows[1][5], 100.0);
  assert_float(&two_way.rows[1][6], 100.0);
}

#[test]
fn tabulate_missing_option_includes_null_categories_and_cells() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let without_missing = session
    .execute(parse_command("tabulate sex").unwrap())
    .expect("default table should execute");
  let with_missing = session
    .execute(parse_command("tabulate sex, missing").unwrap())
    .expect("missing table should execute");
  let ExecutionResult::Tabulate(without_missing) = without_missing else {
    panic!("default table should return a table result");
  };
  let ExecutionResult::Tabulate(with_missing) = with_missing else {
    panic!("missing table should return a table result");
  };
  assert!(
    without_missing
      .rows
      .iter()
      .all(|row| row[0] != CellValue::Null)
  );
  assert_eq!(with_missing.rows.len(), 3);
  assert_eq!(with_missing.rows[2][0], CellValue::Null);
  assert_eq!(with_missing.rows[2][1], CellValue::SignedInteger(1));
  assert_float(&with_missing.rows[2][2], 20.0);
}

#[test]
fn tabulate_uses_attached_value_labels_unless_nolabel_is_requested() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label define sexlbl \"F\" \"Female\" \"M\" \"Male\"").unwrap())
    .expect("sex label set should execute");
  session
    .execute(parse_command("label values sex sexlbl").unwrap())
    .expect("sex labels should attach");
  session
    .execute(
      parse_command("label define agelbl 30 \"Thirty\" 42 \"Forty-two\" 54 \"Fifty-four\"")
        .unwrap(),
    )
    .expect("age label set should execute");
  session
    .execute(parse_command("label values age agelbl").unwrap())
    .expect("age labels should attach");

  let labeled = session
    .execute(parse_command("tabulate sex age").unwrap())
    .expect("labeled table should execute");
  let raw = session
    .execute(parse_command("tabulate sex age, nolabel").unwrap())
    .expect("raw table should execute");
  let ExecutionResult::Tabulate(labeled) = labeled else {
    panic!("labeled table should return a table result");
  };
  let ExecutionResult::Tabulate(raw) = raw else {
    panic!("raw table should return a table result");
  };
  assert_eq!(labeled.rows[0][0], CellValue::Text("Female".to_owned()));
  assert_eq!(labeled.headers[1], "Thirty Count");
  assert_eq!(labeled.headers[2], "Forty-two Count");
  assert_eq!(labeled.headers[3], "Fifty-four Count");
  assert_eq!(raw.rows[0][0], CellValue::Text("F".to_owned()));
  assert_eq!(raw.headers[1], "30 Count");
}

#[test]
fn tabulate_validation_failures_preserve_active_state() {
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
      .execute(parse_command("tabulate missing").unwrap())
      .unwrap_err(),
    RuntimeError::TabulateUnknownVariable {
      variables: vec!["missing".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(Command::Tabulate {
        command: TabulateCommand {
          row_variables: vec!["sex".to_owned()],
          column_variables: vec![],
          row_percent: true,
          column_percent: false,
          include_missing: false,
          nolabel: false,
        },
      })
      .unwrap_err(),
    RuntimeError::TabulatePercentageRequiresTwoWay { option: "row" }
  );
  assert_eq!(session.active_dataset(), Some(&before));
}
