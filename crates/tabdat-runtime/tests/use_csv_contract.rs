use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  csv: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-use-csv-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let csv = root.join("patients.csv");
    fs::write(
      &csv,
      "age,bmi,sex,cost\n30,22.5,F,100.0\n42,25.0,M,150.0\n54,27.5,F,\n",
    )
    .expect("fixture CSV should be written");
    Self { root, csv }
  }

  fn write(&self, name: &str, contents: &str) -> PathBuf {
    let path = self.root.join(name);
    fs::write(&path, contents).expect("fixture CSV should be written");
    path
  }

  fn command(&self) -> Command {
    use_csv_command(&self.csv, None, None)
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn use_csv_command(path: &Path, delimiter: Option<&str>, has_header: Option<bool>) -> Command {
  Command::Use {
    source: DataSource::LocalPath(path.to_string_lossy().into_owned()),
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
    delimiter: delimiter.map(str::to_owned),
    has_header,
  }
}

#[test]
fn loads_default_csv_with_owned_schema_and_null_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let result = session
    .execute(fixture.command())
    .expect("default local CSV should load");
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };

  assert_eq!(load.dataset.source, fixture.csv);
  assert_eq!(load.dataset.row_count, 3);
  assert_eq!(
    load
      .dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![
      ("age", "BIGINT"),
      ("bmi", "DOUBLE"),
      ("sex", "VARCHAR"),
      ("cost", "DOUBLE"),
    ]
  );

  let preview = session
    .execute(parse_command("head 3").expect("head should parse"))
    .expect("CSV rows should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::SignedInteger(30),
        CellValue::Float(22.5),
        CellValue::Text("F".to_owned()),
        CellValue::Float(100.0),
      ],
      vec![
        CellValue::SignedInteger(42),
        CellValue::Float(25.0),
        CellValue::Text("M".to_owned()),
        CellValue::Float(150.0),
      ],
      vec![
        CellValue::SignedInteger(54),
        CellValue::Float(27.5),
        CellValue::Text("F".to_owned()),
        CellValue::Null,
      ],
    ]
  );
  assert_eq!(session.active_dataset(), Some(&load.dataset));
}

#[test]
fn loads_semicolon_csv_with_bound_delimiter_and_header_options() {
  let fixture = Fixture::new();
  let path = fixture.write(
    "semicolon.csv",
    "id;note;amount\n1;alpha;1.5\n2;\"hello;world\";\n",
  );
  let mut session = Session::new();

  let command = use_csv_command(&path, Some(";"), Some(true));
  let result = session
    .execute(command)
    .expect("explicit CSV options should load");
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };

  assert_eq!(load.dataset.row_count, 2);
  assert_eq!(
    load
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["id", "note", "amount"]
  );
  let preview = session
    .execute(parse_command("head 2").expect("head should parse"))
    .expect("semicolon CSV should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(1));
  assert_eq!(preview.rows[0][1], CellValue::Text("alpha".to_owned()));
  assert_eq!(
    preview.rows[1][1],
    CellValue::Text("hello;world".to_owned())
  );
  assert_eq!(preview.rows[1][2], CellValue::Null);
}

#[test]
fn has_header_false_uses_generated_column_names() {
  let fixture = Fixture::new();
  let path = fixture.write("headerless.csv", "1,alpha\n2,beta\n");
  let mut session = Session::new();

  let result = session
    .execute(use_csv_command(&path, None, Some(false)))
    .expect("headerless CSV should load");
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };

  assert_eq!(load.dataset.row_count, 2);
  assert_eq!(
    load
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["column0", "column1"]
  );
}

#[test]
fn accepts_uppercase_csv_and_quoted_empty_fields() {
  let fixture = Fixture::new();
  let path = fixture.write(
    "quoted.CSV",
    "id,note,value\n1,\"hello, world\",3.5\n2,\"\",\n",
  );
  let mut session = Session::new();

  let result = session
    .execute(use_csv_command(&path, None, None))
    .expect("uppercase CSV suffix should load");
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };
  assert_eq!(load.dataset.row_count, 2);

  let preview = session
    .execute(parse_command("head 2").expect("head should parse"))
    .expect("quoted CSV should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview.rows[0][1],
    CellValue::Text("hello, world".to_owned())
  );
  assert_eq!(preview.rows[1][1], CellValue::Null);
  assert_eq!(preview.rows[1][2], CellValue::Null);
}

#[test]
fn loads_header_only_csv_as_an_empty_relation() {
  let fixture = Fixture::new();
  let path = fixture.write("header-only.csv", "id,note\n");
  let mut session = Session::new();

  let result = session
    .execute(use_csv_command(&path, None, None))
    .expect("header-only CSV should load");
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };

  assert_eq!(load.dataset.row_count, 0);
  assert_eq!(
    load
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["id", "note"]
  );
}

#[test]
fn failed_csv_replacement_preserves_prior_relation_and_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial CSV should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let invalid = fixture.root.join("invalid.csv");
  fs::write(&invalid, b"id,note\n1,\xff\n").expect("invalid CSV fixture should be written");
  assert_eq!(
    session
      .execute(use_csv_command(&invalid, None, None))
      .unwrap_err(),
    RuntimeError::CsvRead { path: invalid }
  );
  assert_eq!(session.active_dataset(), Some(&before));

  let preview = session
    .execute(parse_command("head 1").expect("head should parse"))
    .expect("the prior relation should remain queryable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
}
