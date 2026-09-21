use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
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
      "tabdat-runtime-export-{0}-{1}-{2}",
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

  fn with_quoted_text() -> Self {
    let mut fixture = Self::new();
    let parquet = fixture.root.join("quoted.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (1, 'plain', 1.25), (2, 'hello, \"world\"', NULL)) AS quoted(id, note, cost)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("quoted-text Parquet should be written");
    fixture.parquet = parquet;
    fixture
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

fn read_csv(path: &Path) -> String {
  fs::read_to_string(path).expect("CSV output should be readable")
}

#[test]
fn export_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("export output.csv").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "export" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn parsed_export_writes_exact_csv_and_preserves_active_state() {
  let fixture = Fixture::new();
  let output = fixture.root.join("nested output.CSV");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session
    .active_dataset()
    .expect("active metadata should be published")
    .clone();

  let command = parse_command(&format!("export \"{}\"", output.display()))
    .expect("quoted output path should parse");
  let result = session.execute(command).expect("export should succeed");
  let ExecutionResult::Export(export) = result else {
    panic!("export should return an Export result");
  };

  assert_eq!(export.path, output);
  assert_eq!(export.dataset.source, output);
  assert_eq!(export.dataset.row_count, active_before.row_count);
  assert_eq!(export.dataset.columns, active_before.columns);
  assert_eq!(export.dataset.execution_mode, active_before.execution_mode);
  assert_eq!(export.dataset.lazy_engine, active_before.lazy_engine);
  assert_eq!(session.active_dataset(), Some(&active_before));
  assert_eq!(
    read_csv(&output),
    "age,bmi,sex,cost\n42,25.0,M,150.0\n30,22.5,F,100.0\n54,27.5,F,\n"
  );
}

#[test]
fn export_quotes_text_and_preserves_null_values() {
  let fixture = Fixture::with_quoted_text();
  let output = fixture.root.join("quoted.csv");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  session
    .execute(parse_command(&format!("export {}", output.display())).unwrap())
    .expect("export should succeed");

  assert_eq!(
    read_csv(&output),
    "id,note,cost\n1,plain,1.25\n2,\"hello, \"\"world\"\"\",\n"
  );
}

#[test]
fn export_writes_transformed_active_data() {
  let fixture = Fixture::new();
  let output = fixture.root.join("transformed.csv");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("generate age2 = age + 1").unwrap())
    .expect("generate should succeed");

  session
    .execute(parse_command(&format!("export {}", output.display())).unwrap())
    .expect("export should write the transformed relation");

  assert_eq!(
    read_csv(&output),
    "age,bmi,sex,cost,age2\n42,25.0,M,150.0,43\n30,22.5,F,100.0,31\n54,27.5,F,,55\n"
  );
}

#[test]
fn export_rejects_unsupported_extensions_and_preserves_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session.active_dataset().cloned();
  let output = fixture.root.join("out.feather");

  let error = session
    .execute(parse_command(&format!("export {}", output.display())).unwrap())
    .unwrap_err();
  assert_eq!(
    error,
    RuntimeError::ExportUnsupportedFormat { path: output }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
}

#[test]
fn export_requires_replace_for_existing_files_and_replaces_when_requested() {
  let fixture = Fixture::new();
  let output = fixture.root.join("existing.csv");
  fs::write(&output, b"old output\n").expect("existing output should be written");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");

  assert_eq!(
    session
      .execute(parse_command(&format!("export {}", output.display())).unwrap())
      .unwrap_err(),
    RuntimeError::ExportTargetExists {
      path: output.clone()
    }
  );
  assert_eq!(read_csv(&output), "old output\n");

  session
    .execute(parse_command(&format!("export {}, replace", output.display())).unwrap())
    .expect("replace should overwrite the existing file");
  assert_eq!(
    read_csv(&output),
    "age,bmi,sex,cost\n42,25.0,M,150.0\n30,22.5,F,100.0\n54,27.5,F,\n"
  );
}

#[test]
fn export_rejects_directory_targets_and_reports_parent_failures() {
  let fixture = Fixture::new();
  let directory_target = fixture.root.join("directory.csv");
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
      .execute(parse_command(&format!("export {}, replace", directory_target.display())).unwrap())
      .unwrap_err(),
    RuntimeError::ExportTargetNotAFile {
      path: directory_target
    }
  );
  let parent_output = parent_file.join("out.csv");
  assert_eq!(
    session
      .execute(parse_command(&format!("export {}", parent_output.display())).unwrap())
      .unwrap_err(),
    RuntimeError::ExportParentFailed {
      path: parent_output
    }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
}

#[test]
fn export_reports_backend_copy_failures_without_mutating_active_state() {
  let fixture = Fixture::new();
  let output = fixture.root.join(format!("{}.csv", "x".repeat(300)));
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let active_before = session.active_dataset().cloned();

  let error = session
    .execute(parse_command(&format!("export {}", output.display())).unwrap())
    .unwrap_err();

  assert_eq!(
    error,
    RuntimeError::ExportFailed {
      path: output.clone()
    }
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
  assert!(!output.exists());

  let recovery = fixture.root.join("recovery.csv");
  session
    .execute(parse_command(&format!("export {}", recovery.display())).unwrap())
    .expect("a later export should still observe the active relation");
  assert_eq!(
    read_csv(&recovery),
    "age,bmi,sex,cost\n42,25.0,M,150.0\n30,22.5,F,100.0\n54,27.5,F,\n"
  );
}

#[test]
fn export_handles_empty_relations() {
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
  let output = fixture.root.join("empty.csv");
  let mut session = Session::new();
  session
    .execute(Command::Use {
      source: DataSource::LocalPath(empty.to_string_lossy().into_owned()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    })
    .expect("empty fixture should load");

  let result = session
    .execute(parse_command(&format!("export {}", output.display())).unwrap())
    .expect("empty relation should export");
  let ExecutionResult::Export(export) = result else {
    panic!("export should return an Export result");
  };
  assert_eq!(export.dataset.row_count, 0);
  assert_eq!(export.dataset.columns.len(), 1);
  assert_eq!(read_csv(&output), "id\n");
}
