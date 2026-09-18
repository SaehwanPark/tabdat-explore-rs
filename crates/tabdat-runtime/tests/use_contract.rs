use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, LazyEngine, parse_command};
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
  let ExecutionResult::Load(load) = result else {
    panic!("use should return a Load result");
  };

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
  let ExecutionResult::Load(replacement) = replacement else {
    panic!("a replacement use should return a Load result");
  };
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
  let ExecutionResult::Load(load) = result else {
    panic!("a parsed use should return a Load result");
  };

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

#[test]
fn describe_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("describe").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "describe"
    }
  );
  assert_eq!(
    session
      .execute(parse_command("describe").unwrap())
      .unwrap_err()
      .to_string(),
    "describe requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn describe_returns_unchanged_owned_active_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("describe").unwrap())
    .expect("describe should return active metadata");
  let ExecutionResult::Describe(described) = result else {
    panic!("describe should return a Describe result");
  };
  assert_eq!(described.dataset, expected);
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("describe").unwrap())
    .expect("repeated describe should remain read-only");
  let ExecutionResult::Describe(repeated) = repeated else {
    panic!("repeated describe should return a Describe result");
  };
  assert_eq!(repeated.dataset, expected);
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn describe_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("describe").unwrap())
    .expect("describe should still see the prior active dataset");
  let ExecutionResult::Describe(described) = result else {
    panic!("describe should return a Describe result");
  };
  assert_eq!(described.dataset, before);
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn count_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("count").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "count" }
  );
  assert_eq!(
    session
      .execute(parse_command("count").unwrap())
      .unwrap_err()
      .to_string(),
    "count requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn count_returns_the_cached_active_row_count_and_preserves_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("count").unwrap())
    .expect("count should return the active row count");
  let ExecutionResult::Count(count) = result else {
    panic!("count should return a Count result");
  };
  assert_eq!(count.row_count, 3);
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("count").unwrap())
    .expect("repeated count should remain read-only");
  let ExecutionResult::Count(repeated) = repeated else {
    panic!("repeated count should return a Count result");
  };
  assert_eq!(repeated.row_count, 3);
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn count_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("count-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("count").unwrap())
    .expect("count should still see the prior active dataset");
  let ExecutionResult::Count(count) = result else {
    panic!("count should return a Count result");
  };
  assert_eq!(count.row_count, 3);
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn head_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(parse_command("head").unwrap()).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "head" }
  );
  assert_eq!(
    session
      .execute(parse_command("head").unwrap())
      .unwrap_err()
      .to_string(),
    "head requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn head_returns_ordered_owned_rows_with_decimal_and_null_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("head 2").unwrap())
    .expect("head should return the first two rows");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"],);
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
        CellValue::Text("F".to_owned()),
        CellValue::Decimal {
          width: 4,
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
        CellValue::Text("M".to_owned()),
        CellValue::Decimal {
          width: 4,
          scale: 1,
          value: 1500,
        },
      ],
    ]
  );
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("head 2").unwrap())
    .expect("repeated head should remain read-only");
  assert_eq!(repeated, ExecutionResult::Head(preview));
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn head_default_and_oversized_limits_return_all_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  for command in ["head", "head 99", "head 9223372036854775807"] {
    let result = session
      .execute(parse_command(command).unwrap())
      .expect("head should return all rows when the limit is large enough");
    let ExecutionResult::Head(preview) = result else {
      panic!("head should return a Head result");
    };
    assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
    assert_eq!(preview.rows.len(), 3);
    assert_eq!(preview.rows[2][0], CellValue::SignedInteger(54));
    assert_eq!(preview.rows[2][3], CellValue::Null);
  }
}

#[test]
fn head_preserves_nontrivial_source_insertion_order() {
  let fixture = Fixture::new();
  let connection = Connection::open_in_memory().expect("fixture connection should open");
  let parquet_string = fixture.parquet.to_string_lossy().into_owned();
  connection
    .execute(
      "COPY (SELECT * FROM (VALUES (42, 25.0, 'M', 150.0), (30, 22.5, 'F', 100.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
      [&parquet_string],
    )
    .expect("reordered fixture Parquet should be written");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("head 2").unwrap())
    .expect("head should return the first two source rows");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(42));
  assert_eq!(preview.rows[1][0], CellValue::SignedInteger(30));
}

#[test]
fn head_zero_returns_columns_without_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("head 0").unwrap())
    .expect("head zero should return an empty preview");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert!(preview.rows.is_empty());
}

#[test]
fn head_above_i64_limit_fails_deterministically_and_preserves_metadata() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();
  let too_large = (i64::MAX as u128 + 1).to_string();

  assert_eq!(
    session
      .execute(parse_command(&format!("head {too_large}")).unwrap())
      .unwrap_err(),
    RuntimeError::PreviewFailed { command: "head" }
  );
  assert_eq!(
    session
      .execute(parse_command(&format!("head {too_large}")).unwrap())
      .unwrap_err()
      .to_string(),
    "head failed"
  );
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn head_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("head-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("head 1").unwrap())
    .expect("head should still see the prior active dataset");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows.len(), 1);
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn leaves_isid_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Isid {
    variables: vec!["patient_id".to_owned()],
    missok: false,
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "isid" }
  );
}

#[test]
fn leaves_run_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Run {
    path: "analysis.td".to_owned(),
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "run" }
  );
}

#[test]
fn leaves_rename_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Rename {
    old_name: "old_name".to_owned(),
    new_name: "new_name".to_owned(),
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "rename" }
  );
}

#[test]
fn leaves_select_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Select {
    variables: vec!["age".to_owned(), "sex".to_owned()],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "select" }
  );
}

#[test]
fn leaves_sort_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Sort {
    variables: vec!["age".to_owned(), "sex".to_owned()],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "sort" }
  );
}

#[test]
fn leaves_gsort_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Gsort {
    keys: vec![tabdat_language::SortKey {
      variable: "age".to_owned(),
      descending: true,
    }],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "gsort" }
  );
}

#[test]
fn leaves_save_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Save {
    path: "output.parquet".to_owned(),
    replace: false,
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "save" }
  );
}

#[test]
fn leaves_export_execution_deferred() {
  let mut session = Session::new();
  let command = Command::Export {
    path: "output.csv".to_owned(),
    replace: true,
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "export" }
  );
}
