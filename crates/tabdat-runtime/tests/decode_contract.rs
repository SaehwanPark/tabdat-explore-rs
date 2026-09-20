use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::{CellValue, DecodeResult, ExecutionResult, RuntimeError, Session};

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
      "tabdat-runtime-decode-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("decode.parquet");
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
fn decode_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("decode sex_n, generate(sex_str)").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "decode" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn decode_roundtrips_encode_codes_and_preserves_nulls() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("encode sex, generate(sex_n)").unwrap())
    .expect("encode should execute");
  session
    .execute(parse_command("rename sex_n sex_code").unwrap())
    .expect("rename should preserve decode provenance");
  session
    .execute(parse_command("keep age sex_code").unwrap())
    .expect("keep should preserve decode provenance");

  let result = session
    .execute(parse_command("decode sex_code, generate(sex_str)").unwrap())
    .expect("decode should execute");
  let ExecutionResult::Decode(DecodeResult { dataset }) = result else {
    panic!("decode should return a Decode result");
  };
  assert_eq!(dataset.row_count, 4);
  assert_eq!(
    dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![
      ("age", "INTEGER"),
      ("sex_code", "INTEGER"),
      ("sex_str", "VARCHAR"),
    ]
  );

  let result = session
    .execute(parse_command("head 4").unwrap())
    .expect("decoded relation should be previewable");
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
      CellValue::Text("b".to_owned()),
      CellValue::Text("a".to_owned()),
      CellValue::Null,
      CellValue::Text("b".to_owned()),
    ]
  );
}

#[test]
fn decode_supports_quoted_names_and_empty_mappings() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT * FROM (VALUES ('b'), ('a')) AS values_data(\"a\"\"b\")",
  );
  let mut session = Session::new();
  session
    .execute(fixture.command_for(quoted))
    .expect("quoted fixture should load");
  session
    .execute(parse_command(r#"encode `a"b`, generate(`new"code`)"#).unwrap())
    .expect("quoted encode should execute");
  session
    .execute(parse_command(r#"decode `new"code`, generate(`text"value`)"#).unwrap())
    .expect("quoted decode should execute");

  let result = session
    .execute(parse_command("head 2").unwrap())
    .expect("quoted decoded relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["a\"b", "new\"code", "text\"value"]);
  assert_eq!(preview.rows[0][2], CellValue::Text("b".to_owned()));
  assert_eq!(preview.rows[1][2], CellValue::Text("a".to_owned()));

  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS VARCHAR) AS sex WHERE FALSE",
  );
  let mut empty_session = Session::new();
  empty_session
    .execute(fixture.command_for(empty))
    .expect("empty fixture should load");
  empty_session
    .execute(parse_command("encode sex, generate(sex_n)").unwrap())
    .expect("empty encode should execute");
  let result = empty_session
    .execute(parse_command("decode sex_n, generate(sex_str)").unwrap())
    .expect("empty decode should execute");
  let ExecutionResult::Decode(decoded) = result else {
    panic!("decode should return a Decode result");
  };
  assert_eq!(decoded.dataset.row_count, 0);
  assert_eq!(decoded.dataset.columns[1].data_type, "BIGINT");
  assert_eq!(decoded.dataset.columns[2].data_type, "VARCHAR");
}

#[test]
fn decode_validation_and_backend_failures_are_atomic() {
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
      .execute(parse_command("decode age, generate(age_str)").unwrap())
      .unwrap_err(),
    RuntimeError::DecodeRequiresAttachedLabels {
      variable: "age".to_owned(),
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));

  session
    .execute(parse_command("encode sex, generate(sex_n)").unwrap())
    .expect("encode should execute");
  let after_encode = session
    .active_dataset()
    .expect("encode should publish metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("decode sex_n, generate(sex)").unwrap())
      .unwrap_err(),
    RuntimeError::DecodeTargetExists {
      variable: "sex".to_owned(),
    }
  );
  assert_eq!(session.active_dataset(), Some(&after_encode));

  assert_eq!(
    session
      .execute(Command::Decode {
        source: "sex_n".to_owned(),
        generate: "bad\0name".to_owned(),
      })
      .unwrap_err(),
    RuntimeError::DecodeFailed
  );
  assert_eq!(session.active_dataset(), Some(&after_encode));

  session
    .execute(parse_command("decode sex_n, generate(sex_str)").unwrap())
    .expect("a failed decode should leave the map available for retry");
}
