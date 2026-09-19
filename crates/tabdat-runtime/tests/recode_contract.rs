use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{
  Command, DataSource, ExecutionMode, RecodeInput, RecodeRule, RecodeTarget, RecodeValue,
  parse_command,
};
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
      "tabdat-runtime-recode-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("recode.parquet");
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
fn recode_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("recode age (1 = 0), replace").expect("recode should parse"))
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "recode" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn generated_recode_preserves_schema_rows_and_applies_ordered_ranges() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(
      parse_command(
        "recode age (min/35 = 1) (36/50 = 2) (51/max = 3) (else = -1), generate(age_group)",
      )
      .expect("range recode should parse"),
    )
    .expect("range recode should execute");
  let ExecutionResult::Recode(recode) = result else {
    panic!("recode should return a Recode result");
  };
  assert_eq!(recode.dataset.source, fixture.parquet);
  assert_eq!(recode.dataset.row_count, 3);
  assert_eq!(
    recode
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost", "age_group"]
  );
  assert_eq!(recode.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(recode.dataset.lazy_engine, None);

  let result = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("recoded relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview
      .rows
      .iter()
      .map(|row| row[4].clone())
      .collect::<Vec<_>>(),
    vec![
      CellValue::SignedInteger(1),
      CellValue::SignedInteger(2),
      CellValue::SignedInteger(3),
    ]
  );
}

#[test]
fn replacement_recode_supports_missingness_and_string_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(
      parse_command("recode cost (missing = 999) (nonmissing = 100), replace")
        .expect("missingness recode should parse"),
    )
    .expect("missingness recode should execute");
  session
    .execute(
      parse_command("recode sex ('F' = 'female') ('M' = 'male'), replace")
        .expect("string recode should parse"),
    )
    .expect("string recode should execute");

  let result = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("recoded relation should be previewable");
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
        CellValue::Text("female".to_owned()),
        CellValue::Decimal {
          width: 11,
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
        CellValue::Text("male".to_owned()),
        CellValue::Decimal {
          width: 11,
          scale: 1,
          value: 1000,
        },
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 275,
        },
        CellValue::Text("female".to_owned()),
        CellValue::Decimal {
          width: 11,
          scale: 1,
          value: 9990,
        },
      ],
    ]
  );
}

#[test]
fn generated_string_recode_uses_text_output_and_quoted_names() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT CAST(1 AS INTEGER) AS \"a\"\"b\", CAST(NULL AS VARCHAR) AS \"empty value\" WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted.clone()))
    .expect("quoted fixture should load");

  let quoted_tick = char::from(96);
  let command = format!(
    "recode {quoted_tick}a\"b{quoted_tick} (1 = 'one'), generate({quoted_tick}new value{quoted_tick})"
  );
  let result = session
    .execute(parse_command(&command).expect("quoted recode should parse"))
    .expect("quoted recode should execute on an empty relation");
  let ExecutionResult::Recode(recode) = result else {
    panic!("recode should return a Recode result");
  };
  assert_eq!(recode.dataset.source, quoted);
  assert_eq!(recode.dataset.row_count, 0);
  assert_eq!(
    recode
      .dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![
      ("a\"b", "INTEGER"),
      ("empty value", "VARCHAR"),
      ("new value", "VARCHAR"),
    ]
  );
}

#[test]
fn recode_validation_failures_preserve_metadata_and_rows() {
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
      Command::Recode {
        variables: Vec::new(),
        rules: Vec::new(),
        target: RecodeTarget::Replace,
      },
      RuntimeError::RecodeNoVariables,
    ),
    (
      Command::Recode {
        variables: vec!["age".to_owned()],
        rules: Vec::new(),
        target: RecodeTarget::Replace,
      },
      RuntimeError::RecodeNoRules,
    ),
    (
      parse_command("recode missing (1 = 0), replace").expect("unknown-variable case should parse"),
      RuntimeError::RecodeUnknownVariable {
        variables: vec!["missing".to_owned()],
      },
    ),
    (
      parse_command("recode age (1 = 0), generate(age_group other)")
        .expect("cardinality case should parse"),
      RuntimeError::RecodeGenerateCountMismatch,
    ),
    (
      parse_command("recode age bmi (1 = 0), generate(age_group age_group)")
        .expect("duplicate-target case should parse"),
      RuntimeError::RecodeGenerateDuplicateVariable,
    ),
    (
      parse_command("recode age (1 = 0), generate(bmi)")
        .expect("existing-target case should parse"),
      RuntimeError::RecodeGenerateTargetExists {
        variable: "bmi".to_owned(),
      },
    ),
    (
      parse_command("recode sex (1/10 = 'other'), replace")
        .expect("nonnumeric-range case should parse"),
      RuntimeError::RecodeRangeRequiresNumeric {
        variable: "sex".to_owned(),
      },
    ),
    (
      Command::Recode {
        variables: vec!["age".to_owned()],
        rules: vec![RecodeRule {
          inputs: vec![RecodeInput::Value(RecodeValue::Number(
            "not-a-number".to_owned(),
          ))],
          output: RecodeValue::Number("0".to_owned()),
        }],
        target: RecodeTarget::Replace,
      },
      RuntimeError::RecodeFailed,
    ),
  ];
  for (command, expected) in failures {
    assert_eq!(session.execute(command).unwrap_err(), expected);
    assert_eq!(session.active_dataset(), Some(&before));
  }

  let result = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("failed recodes should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
  assert_eq!(preview.rows[2][3], CellValue::Null);
}
