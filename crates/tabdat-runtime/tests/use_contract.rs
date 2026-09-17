use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine, parse_command};
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};

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
      "tabdat-runtime-use-{0}-{1}-{2}",
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

  fn command(&self) -> Command {
    use_command(&self.parquet)
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn use_command(path: &Path) -> Command {
  Command::Use {
    source: DataSource::LocalPath(path.to_string_lossy().into_owned()),
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
    delimiter: None,
    has_header: None,
  }
}

#[test]
fn loads_existing_local_parquet_and_reports_owned_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let result = session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let ExecutionResult::Load(load) = result;

  assert_eq!(load.dataset.source, fixture.parquet);
  assert_eq!(load.dataset.row_count, 3);
  assert_eq!(load.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(load.dataset.lazy_engine, None);
  assert_eq!(
    load
      .dataset
      .columns
      .iter()
      .map(|column| (column.name.as_str(), column.data_type.as_str()))
      .collect::<Vec<_>>(),
    vec![
      ("age", "INTEGER"),
      ("bmi", "DECIMAL(3,1)"),
      ("sex", "VARCHAR"),
      ("cost", "DECIMAL(4,1)"),
    ]
  );
  assert_eq!(session.active_dataset(), Some(&load.dataset));

  let replacement = session
    .execute(fixture.command())
    .expect("a second eager local Parquet load should replace the active relation");
  let ExecutionResult::Load(replacement) = replacement;
  assert_eq!(replacement.dataset, load.dataset);
  assert_eq!(session.active_dataset(), Some(&replacement.dataset));
}

#[test]
fn parses_use_before_executing_the_command() {
  let fixture = Fixture::new();
  let command_text = format!("use {}", fixture.parquet.display());
  let command = parse_command(&command_text).expect("the fixture path should parse as use");
  let mut session = Session::new();

  let result = session
    .execute(command)
    .expect("a parsed eager local Parquet command should load");
  let ExecutionResult::Load(load) = result;

  assert_eq!(load.dataset.source, fixture.parquet);
  assert_eq!(load.dataset.row_count, 3);
}

#[test]
fn rejects_out_of_scope_use_forms_without_an_active_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let mut lazy = fixture.command();
  if let Command::Use {
    execution_mode,
    lazy_engine,
    ..
  } = &mut lazy
  {
    *execution_mode = ExecutionMode::Lazy;
    *lazy_engine = Some(LazyEngine::DuckDb);
  }
  assert_eq!(
    session.execute(lazy).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet loads"
  );

  let uri = Command::Use {
    source: DataSource::Uri("https://example.com/patients.parquet".to_owned()),
    execution_mode: ExecutionMode::Eager,
    lazy_engine: None,
    delimiter: None,
    has_header: None,
  };
  assert_eq!(
    session.execute(uri).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet loads"
  );

  let mut options = fixture.command();
  if let Command::Use { delimiter, .. } = &mut options {
    *delimiter = Some(";".to_owned());
  }
  assert_eq!(
    session.execute(options).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet loads"
  );

  let mut header = fixture.command();
  if let Command::Use { has_header, .. } = &mut header {
    *has_header = Some(true);
  }
  assert_eq!(
    session.execute(header).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet loads"
  );

  let mut polars_lazy = fixture.command();
  if let Command::Use {
    execution_mode,
    lazy_engine,
    ..
  } = &mut polars_lazy
  {
    *execution_mode = ExecutionMode::Lazy;
    *lazy_engine = Some(LazyEngine::Polars);
  }
  assert_eq!(
    session.execute(polars_lazy).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet loads"
  );

  assert!(session.active_dataset().is_none());
}

#[test]
fn rejects_invalid_paths_and_extensions_before_backend_read() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let missing = use_command(&fixture.root.join("missing.parquet"));
  assert_eq!(
    session.execute(missing).unwrap_err(),
    RuntimeError::FileNotFound {
      path: fixture.root.join("missing.parquet")
    }
  );

  let missing_wrong_extension = use_command(&fixture.root.join("missing.csv"));
  assert_eq!(
    session
      .execute(missing_wrong_extension)
      .unwrap_err()
      .to_string(),
    "use runtime slice supports only local .parquet files"
  );

  let directory_path = fixture.root.join("directory.parquet");
  fs::create_dir(&directory_path).expect("directory fixture should be created");
  let directory = use_command(&directory_path);
  assert_eq!(
    session.execute(directory).unwrap_err(),
    RuntimeError::NotAFile {
      path: directory_path
    }
  );

  let directory_wrong_extension = fixture.root.join("directory.csv");
  fs::create_dir(&directory_wrong_extension)
    .expect("wrong-extension directory fixture should be created");
  let directory_wrong_extension_command = use_command(&directory_wrong_extension);
  assert_eq!(
    session
      .execute(directory_wrong_extension_command)
      .unwrap_err()
      .to_string(),
    "use runtime slice supports only local .parquet files"
  );

  let wrong_extension_path = fixture.root.join("patients.csv");
  fs::write(&wrong_extension_path, "not csv").expect("wrong-extension fixture should be written");
  let wrong_extension = use_command(&wrong_extension_path);
  assert_eq!(
    session.execute(wrong_extension).unwrap_err().to_string(),
    "use runtime slice supports only local .parquet files"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn corrupt_load_preserves_the_prior_active_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  let corrupt = use_command(&corrupt_path);
  assert_eq!(
    session.execute(corrupt).unwrap_err(),
    RuntimeError::ParquetRead { path: corrupt_path }
  );
  assert_eq!(session.active_dataset(), Some(&before));
}
