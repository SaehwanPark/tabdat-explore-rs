use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type SchemaColumns = Vec<(String, String)>;
type PatientRows = Vec<(i32, f64, String, Option<f64>)>;

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
      "tabdat-runtime-save-{0}-{1}-{2}",
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
        "COPY (SELECT * FROM (VALUES (42, 25.0, 'M', 150.0), (30, 22.5, 'F', 100.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
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

fn read_back(path: &Path) -> (SchemaColumns, PatientRows) {
  let connection = Connection::open_in_memory().expect("read-back connection should open");
  let path_string = path.to_string_lossy().into_owned();
  let mut describe = connection
    .prepare("DESCRIBE SELECT * FROM read_parquet(?)")
    .expect("Parquet schema should be inspectable");
  let columns = describe
    .query_map([&path_string], |row| Ok((row.get(0)?, row.get(1)?)))
    .expect("Parquet schema should be readable")
    .map(|row| row.expect("schema row should be readable"))
    .collect::<Vec<_>>();

  let mut rows = connection
    .prepare("SELECT age, bmi, sex, cost FROM read_parquet(?)")
    .expect("Parquet rows should be queryable");
  let rows = rows
    .query_map([&path_string], |row| {
      Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })
    .expect("Parquet rows should be readable")
    .map(|row| row.expect("row should be readable"))
    .collect::<Vec<_>>();
  (columns, rows)
}

#[test]
fn save_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("save output.parquet").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "save" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn parsed_save_writes_current_rows_and_preserves_active_state() {
  let fixture = Fixture::new();
  let output = fixture.root.join("nested output.PARQUET");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session
    .active_dataset()
    .expect("active metadata should be published")
    .clone();

  let command = parse_command(&format!("save \"{}\"", output.display()))
    .expect("quoted output path should parse");
  let result = session.execute(command).expect("save should succeed");
  let ExecutionResult::Save(save) = result else {
    panic!("save should return a Save result");
  };

  assert_eq!(save.path, output);
  assert_eq!(save.dataset.source, output);
  assert_eq!(save.dataset.row_count, active_before.row_count);
  assert_eq!(save.dataset.columns, active_before.columns);
  assert_eq!(save.dataset.execution_mode, active_before.execution_mode);
  assert_eq!(save.dataset.lazy_engine, active_before.lazy_engine);
  assert_eq!(session.active_dataset(), Some(&active_before));

  let (columns, rows) = read_back(&output);
  assert_eq!(
    columns,
    vec![
      ("age".to_owned(), "INTEGER".to_owned()),
      ("bmi".to_owned(), "DECIMAL(3,1)".to_owned()),
      ("sex".to_owned(), "VARCHAR".to_owned()),
      ("cost".to_owned(), "DECIMAL(4,1)".to_owned()),
    ]
  );
  assert_eq!(
    rows,
    vec![
      (42, 25.0, "M".to_owned(), Some(150.0)),
      (30, 22.5, "F".to_owned(), Some(100.0)),
      (54, 27.5, "F".to_owned(), None),
    ]
  );
}

#[test]
fn save_writes_transformed_active_data() {
  let fixture = Fixture::new();
  let output = fixture.root.join("transformed.parquet");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("generate age2 = age + 1").unwrap())
    .expect("generate should succeed");

  session
    .execute(parse_command(&format!("save {}", output.display())).unwrap())
    .expect("save should write the transformed relation");

  let connection = Connection::open_in_memory().expect("read-back connection should open");
  let path_string = output.to_string_lossy().into_owned();
  let mut statement = connection
    .prepare("SELECT age, age2 FROM read_parquet(?)")
    .expect("transformed Parquet should be queryable");
  let rows = statement
    .query_map([&path_string], |row| Ok((row.get(0)?, row.get(1)?)))
    .expect("transformed rows should be readable")
    .map(|row| row.expect("transformed row should be readable"))
    .collect::<Vec<(i32, i32)>>();
  assert_eq!(rows, vec![(42, 43), (30, 31), (54, 55)]);
}

#[test]
fn save_rejects_unsupported_extensions_and_preserves_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session.active_dataset().cloned();

  let error = session
    .execute(parse_command(&format!("save {}", fixture.root.join("out.csv").display())).unwrap())
    .unwrap_err();
  assert_eq!(
    error,
    RuntimeError::SaveUnsupportedFormat {
      path: fixture.root.join("out.csv")
    }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
  assert!(!fixture.root.join("out.csv").exists());
}

#[test]
fn save_requires_replace_for_existing_files_and_replaces_when_requested() {
  let fixture = Fixture::new();
  let output = fixture.root.join("existing.parquet");
  fs::write(&output, b"old output").expect("existing output should be written");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  assert_eq!(
    session
      .execute(parse_command(&format!("save {}", output.display())).unwrap())
      .unwrap_err(),
    RuntimeError::SaveTargetExists {
      path: output.clone()
    }
  );
  assert_eq!(
    fs::read(&output).expect("old output should remain"),
    b"old output"
  );

  session
    .execute(parse_command(&format!("save {}, replace", output.display())).unwrap())
    .expect("replace should overwrite the existing file");
  let (_, rows) = read_back(&output);
  assert_eq!(rows.len(), 3);
}

#[test]
fn save_rejects_directory_targets_and_reports_parent_failures_atomically() {
  let fixture = Fixture::new();
  let directory_target = fixture.root.join("directory.parquet");
  fs::create_dir(&directory_target).expect("directory target should be created");
  let parent_file = fixture.root.join("parent-file");
  fs::write(&parent_file, b"not a directory").expect("parent file should be written");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session.active_dataset().cloned();

  assert_eq!(
    session
      .execute(parse_command(&format!("save {}, replace", directory_target.display())).unwrap())
      .unwrap_err(),
    RuntimeError::SaveTargetNotAFile {
      path: directory_target
    }
  );
  assert_eq!(
    session
      .execute(parse_command(&format!("save {}/out.parquet", parent_file.display())).unwrap())
      .unwrap_err(),
    RuntimeError::SaveParentFailed {
      path: parent_file.join("out.parquet")
    }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
}

#[test]
fn save_reports_backend_copy_failures_without_mutating_active_state() {
  let fixture = Fixture::new();
  let output = fixture.root.join(format!("{}.parquet", "x".repeat(300)));
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session.active_dataset().cloned();

  let error = session
    .execute(parse_command(&format!("save {}", output.display())).unwrap())
    .unwrap_err();

  assert_eq!(
    error,
    RuntimeError::SaveFailed {
      path: output.clone()
    }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
  assert!(!output.exists());
}

#[test]
fn save_handles_empty_relations() {
  let fixture = Fixture::new();
  let empty = fixture.root.join("empty.parquet");
  let connection = Connection::open_in_memory().expect("fixture connection should open");
  let empty_string = empty.to_string_lossy().into_owned();
  connection
    .execute(
      "COPY (SELECT CAST(NULL AS INTEGER) AS id WHERE false) TO ? (FORMAT PARQUET)",
      [&empty_string],
    )
    .expect("empty Parquet should be written");
  let output = fixture.root.join("empty-output.parquet");
  let mut session = Session::new();
  session
    .execute(use_command(&empty))
    .expect("empty fixture should load");

  let result = session
    .execute(parse_command(&format!("save {}", output.display())).unwrap())
    .expect("empty relation should save");
  let ExecutionResult::Save(save) = result else {
    panic!("save should return a Save result");
  };
  assert_eq!(save.dataset.row_count, 0);
  assert_eq!(save.dataset.columns.len(), 1);
  let output_string = output.to_string_lossy().into_owned();
  let row_count: i64 = Connection::open_in_memory()
    .expect("read-back connection should open")
    .query_row(
      "SELECT COUNT(*) FROM read_parquet(?)",
      [&output_string],
      |row| row.get(0),
    )
    .expect("empty Parquet row count should be readable");
  assert_eq!(row_count, 0);
}
