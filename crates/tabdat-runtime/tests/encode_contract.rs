use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, LabelValue, parse_command};
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session, ValueLabelSet};

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
      "tabdat-runtime-encode-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("encode.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 'b'), (42, 'a'), (54, CAST(NULL AS VARCHAR)), (60, 'b')) AS patients(age, sex)) TO ? (FORMAT PARQUET)",
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
fn encode_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("encode sex, generate(sex_n)").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "encode" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn encode_assigns_sorted_integer_codes_and_preserves_nulls() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  let result = session
    .execute(parse_command("encode sex, generate(sex_n)").unwrap())
    .expect("encode should execute");
  let ExecutionResult::Encode(encoded) = result else {
    panic!("encode should return an Encode result");
  };
  assert_eq!(encoded.dataset.source, fixture.parquet);
  assert_eq!(encoded.dataset.row_count, 4);
  assert_eq!(
    encoded
      .dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![("age", "INTEGER"), ("sex", "VARCHAR"), ("sex_n", "INTEGER"),]
  );

  let result = session
    .execute(parse_command("head 4").unwrap())
    .expect("encoded relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview
      .rows
      .iter()
      .map(|row| row[2].clone())
      .collect::<Vec<_>>(),
    vec![
      CellValue::SignedInteger(2),
      CellValue::SignedInteger(1),
      CellValue::Null,
      CellValue::SignedInteger(2),
    ]
  );
}

#[test]
fn encode_label_metadata_supports_named_decode_sets_and_variable_labels() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label variable sex \"Sex\"").unwrap())
    .expect("variable label should execute");

  session
    .execute(parse_command("encode sex, generate(sex_n) label(sex_label)").unwrap())
    .expect("named encode label should execute");
  let metadata = session
    .active_label_metadata()
    .expect("encode should publish label metadata");
  assert_eq!(
    metadata.variable_labels,
    vec![
      ("sex".to_owned(), "Sex".to_owned()),
      ("sex_n".to_owned(), "Sex".to_owned()),
    ]
  );
  assert_eq!(
    metadata.value_sets,
    vec![ValueLabelSet {
      name: "sex_label".to_owned(),
      mappings: vec![
        (LabelValue::Integer(1), "a".to_owned()),
        (LabelValue::Integer(2), "b".to_owned()),
      ],
    }]
  );
  assert_eq!(
    metadata.attachments,
    vec![("sex_n".to_owned(), "sex_label".to_owned())]
  );

  let result = session
    .execute(parse_command("decode sex_n, generate(sex_text)").unwrap())
    .expect("decode should consume the attached label set");
  assert!(matches!(result, ExecutionResult::Decode(_)));
}

#[test]
fn encode_supports_quoted_names_empty_relations_and_repeated_successes() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT * FROM (VALUES ('b'), ('a')) AS values_data(\"a\"\"b\")",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted.clone()))
    .expect("quoted fixture should load");

  session
    .execute(parse_command(r#"encode `a"b`, generate(`new"code`)"#).unwrap())
    .expect("quoted encode should execute");
  let result = session
    .execute(parse_command(r#"encode `a"b`, generate(second)"#).unwrap())
    .expect("repeated encode should execute");
  let ExecutionResult::Encode(encoded) = result else {
    panic!("encode should return an Encode result");
  };
  assert_eq!(
    encoded
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["a\"b", "new\"code", "second"]
  );

  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS VARCHAR) AS sex WHERE FALSE",
  );
  let mut empty_session = Session::new();
  empty_session
    .execute(fixture.command_for(empty))
    .expect("empty fixture should load");
  let result = empty_session
    .execute(parse_command("encode sex, generate(sex_n)").unwrap())
    .expect("empty encode should execute");
  let ExecutionResult::Encode(encoded) = result else {
    panic!("encode should return an Encode result");
  };
  assert_eq!(encoded.dataset.row_count, 0);
  assert_eq!(encoded.dataset.columns[1].data_type, "BIGINT");
}

#[test]
fn encode_validation_and_backend_failures_are_atomic() {
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
      parse_command("encode missing, generate(missing_n)").unwrap(),
      RuntimeError::EncodeUnknownVariable {
        variable: "missing".to_owned(),
      },
    ),
    (
      parse_command("encode age, generate(age_n)").unwrap(),
      RuntimeError::EncodeRequiresString {
        variable: "age".to_owned(),
      },
    ),
    (
      parse_command("encode sex, generate(age)").unwrap(),
      RuntimeError::EncodeTargetExists {
        variable: "age".to_owned(),
      },
    ),
    (
      Command::Encode {
        source: "sex".to_owned(),
        generate: "bad\0name".to_owned(),
        label: None,
      },
      RuntimeError::EncodeFailed,
    ),
  ];
  for (command, expected) in failures {
    assert_eq!(session.execute(command).unwrap_err(), expected);
    assert_eq!(session.active_dataset(), Some(&before));
  }

  let result = session
    .execute(parse_command("head 4").unwrap())
    .expect("failed encodes should preserve the active relation");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "sex"]);
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
  assert_eq!(preview.rows[2][1], CellValue::Null);
}
