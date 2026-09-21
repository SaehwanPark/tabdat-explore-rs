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
    "use runtime slice supports only eager local Parquet or CSV loads"
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
    "use runtime slice supports only eager local Parquet or CSV loads"
  );

  let mut options = fixture.command();
  if let Command::Use { delimiter, .. } = &mut options {
    *delimiter = Some(";".to_owned());
  }
  assert_eq!(
    session.execute(options).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet or CSV loads"
  );

  let mut header = fixture.command();
  if let Command::Use { has_header, .. } = &mut header {
    *has_header = Some(true);
  }
  assert_eq!(
    session.execute(header).unwrap_err().to_string(),
    "use runtime slice supports only eager local Parquet or CSV loads"
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
    "use runtime slice supports only eager local Parquet or CSV loads"
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

  let mut missing_with_options = use_command(&fixture.root.join("missing-with-options.parquet"));
  if let Command::Use { delimiter, .. } = &mut missing_with_options {
    *delimiter = Some(";".to_owned());
  }
  assert_eq!(
    session.execute(missing_with_options).unwrap_err(),
    RuntimeError::UnsupportedUseConfiguration
  );

  let missing_wrong_extension = use_command(&fixture.root.join("missing.csv"));
  assert_eq!(
    session.execute(missing_wrong_extension).unwrap_err(),
    RuntimeError::FileNotFound {
      path: fixture.root.join("missing.csv")
    }
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
      .unwrap_err(),
    RuntimeError::NotAFile {
      path: directory_wrong_extension
    }
  );

  let wrong_extension_path = fixture.root.join("patients.csv");
  fs::write(&wrong_extension_path, b"\xff").expect("invalid CSV fixture should be written");
  let wrong_extension = use_command(&wrong_extension_path);
  assert_eq!(
    session.execute(wrong_extension).unwrap_err(),
    RuntimeError::CsvRead {
      path: wrong_extension_path
    }
  );

  let unsupported_path = fixture.root.join("patients.txt");
  fs::write(&unsupported_path, "not a supported input")
    .expect("unsupported-format fixture should be written");
  assert_eq!(
    session.execute(use_command(&unsupported_path)).unwrap_err(),
    RuntimeError::UnsupportedFormat {
      path: unsupported_path
    }
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
fn tail_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session.execute(parse_command("tail").unwrap()).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "tail" }
  );
  assert_eq!(
    session
      .execute(parse_command("tail").unwrap())
      .unwrap_err()
      .to_string(),
    "tail requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn summarize_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("summarize").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "summarize"
    }
  );
  assert_eq!(
    session
      .execute(parse_command("summarize").unwrap())
      .unwrap_err()
      .to_string(),
    "summarize requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn codebook_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("codebook").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "codebook"
    }
  );
  assert_eq!(
    session
      .execute(parse_command("codebook").unwrap())
      .unwrap_err()
      .to_string(),
    "codebook requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn codebook_returns_counts_types_examples_and_is_read_only() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(
      parse_command(&format!("use {}", fixture.parquet.display()))
        .expect("the fixture use should parse"),
    )
    .expect("parsed eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("codebook age cost").unwrap())
    .expect("codebook should return requested profiles");
  let ExecutionResult::Codebook(codebook) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(codebook.rows.len(), 2);
  assert_eq!(codebook.rows[0].variable, "age");
  assert_eq!(codebook.rows[0].data_type, "INTEGER");
  assert_eq!(codebook.rows[0].nonmissing, 3);
  assert_eq!(codebook.rows[0].missing, 0);
  assert_eq!(codebook.rows[0].distinct, 3);
  assert_eq!(
    codebook.rows[0].examples,
    vec![
      CellValue::SignedInteger(30),
      CellValue::SignedInteger(42),
      CellValue::SignedInteger(54),
    ]
  );
  assert_eq!(codebook.rows[1].variable, "cost");
  assert_eq!(codebook.rows[1].data_type, "DECIMAL(4,1)");
  assert_eq!(codebook.rows[1].nonmissing, 2);
  assert_eq!(codebook.rows[1].missing, 1);
  assert_eq!(codebook.rows[1].distinct, 2);
  assert_eq!(
    codebook.rows[1].examples,
    vec![
      CellValue::Decimal {
        width: 4,
        scale: 1,
        value: 1000,
      },
      CellValue::Decimal {
        width: 4,
        scale: 1,
        value: 1500,
      },
    ]
  );
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("codebook age cost").unwrap())
    .expect("repeated codebook should remain read-only");
  assert_eq!(repeated, ExecutionResult::Codebook(codebook));
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn codebook_default_order_and_examples_preserve_schema_and_duplicates() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("codebook").unwrap())
    .expect("codebook should profile every schema column by default");
  let ExecutionResult::Codebook(defaults) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(
    defaults
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost"]
  );
  assert_eq!(defaults.rows[1].data_type, "DECIMAL(3,1)");
  assert_eq!(defaults.rows[1].distinct, 3);
  assert_eq!(
    defaults.rows[2].examples,
    vec![
      CellValue::Text("F".to_owned()),
      CellValue::Text("M".to_owned()),
      CellValue::Text("F".to_owned()),
    ]
  );

  let result = session
    .execute(parse_command("codebook cost age cost").unwrap())
    .expect("codebook should preserve explicit order and duplicates");
  let ExecutionResult::Codebook(explicit) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(
    explicit
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["cost", "age", "cost"]
  );
  assert_eq!(explicit.rows[0], explicit.rows[2]);
}

#[test]
fn codebook_rejects_unknown_variables_with_exact_error() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  assert_eq!(
    session
      .execute(parse_command("codebook missing missing age").unwrap())
      .unwrap_err(),
    RuntimeError::CodebookUnknownVariable {
      variables: vec!["missing".to_owned(), "missing".to_owned()]
    }
  );
  assert_eq!(
    session
      .execute(parse_command("codebook missing missing age").unwrap())
      .unwrap_err()
      .to_string(),
    "codebook unknown variable: missing, missing"
  );
}

#[test]
fn codebook_reports_empty_and_all_null_columns() {
  let fixture = Fixture::new();
  let null_path = fixture.write_parquet(
    "all-null-codebook.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value FROM range(2)",
  );
  let empty_path = fixture.write_parquet(
    "empty-codebook.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value FROM range(0)",
  );
  let mut session = Session::new();

  session
    .execute(use_command(&null_path))
    .expect("all-null Parquet should load");
  let result = session
    .execute(parse_command("codebook value").unwrap())
    .expect("all-null codebook should succeed");
  let ExecutionResult::Codebook(codebook) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(codebook.rows[0].nonmissing, 0);
  assert_eq!(codebook.rows[0].missing, 2);
  assert_eq!(codebook.rows[0].distinct, 0);
  assert!(codebook.rows[0].examples.is_empty());

  session
    .execute(use_command(&empty_path))
    .expect("empty Parquet should load");
  let result = session
    .execute(parse_command("codebook").unwrap())
    .expect("empty codebook should succeed");
  let ExecutionResult::Codebook(codebook) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(codebook.rows[0].nonmissing, 0);
  assert_eq!(codebook.rows[0].missing, 0);
  assert_eq!(codebook.rows[0].distinct, 0);
  assert!(codebook.rows[0].examples.is_empty());
}

#[test]
fn codebook_failure_preserves_active_metadata() {
  let fixture = Fixture::new();
  let unsupported_path =
    fixture.write_parquet("unsupported-codebook.parquet", "SELECT [1, 2] AS items");
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  session
    .execute(use_command(&unsupported_path))
    .expect("unsupported-value Parquet should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("codebook items").unwrap())
      .unwrap_err(),
    RuntimeError::CodebookFailed {
      variable: "items".to_owned()
    }
  );
  assert_eq!(
    session
      .execute(parse_command("codebook items").unwrap())
      .unwrap_err()
      .to_string(),
    "codebook failed for variable: items"
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn codebook_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("codebook-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("codebook age").unwrap())
    .expect("codebook should still see the prior active dataset");
  let ExecutionResult::Codebook(codebook) = result else {
    panic!("codebook should return a Codebook result");
  };
  assert_eq!(codebook.rows[0].nonmissing, 3);
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn missing_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("missing").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "missing" }
  );
  assert_eq!(
    session
      .execute(parse_command("missing").unwrap())
      .unwrap_err()
      .to_string(),
    "missing requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn duplicates_requires_an_active_dataset_without_initializing_the_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("duplicates").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "duplicates"
    }
  );
  assert_eq!(
    session
      .execute(parse_command("duplicates").unwrap())
      .unwrap_err()
      .to_string(),
    "duplicates requires an active dataset; run use <path> first"
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn missing_returns_exact_counts_types_percentages_and_is_read_only() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(
      parse_command(&format!("use {}", fixture.parquet.display()))
        .expect("the fixture use should parse"),
    )
    .expect("parsed eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("missing age cost").unwrap())
    .expect("missing should return requested rows");
  let ExecutionResult::Missing(missing) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(missing.rows.len(), 2);
  assert_eq!(missing.rows[0].variable, "age");
  assert_eq!(missing.rows[0].data_type, "INTEGER");
  assert_eq!(missing.rows[0].total, 3);
  assert_eq!(missing.rows[0].missing, 0);
  assert_eq!(missing.rows[0].nonmissing, 3);
  assert_eq!(missing.rows[0].missing_percent, 0.0);
  assert_eq!(missing.rows[1].variable, "cost");
  assert_eq!(missing.rows[1].data_type, "DECIMAL(4,1)");
  assert_eq!(missing.rows[1].total, 3);
  assert_eq!(missing.rows[1].missing, 1);
  assert_eq!(missing.rows[1].nonmissing, 2);
  assert!((missing.rows[1].missing_percent - (100.0 / 3.0)).abs() < 1e-12);
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("missing age cost").unwrap())
    .expect("repeated missing should remain read-only");
  assert_eq!(repeated, ExecutionResult::Missing(missing));
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn missing_default_order_and_explicit_duplicates_are_preserved() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("missing").unwrap())
    .expect("missing should report every schema column by default");
  let ExecutionResult::Missing(defaults) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(
    defaults
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost"]
  );
  assert_eq!(defaults.rows[2].missing, 0);
  assert_eq!(defaults.rows[3].missing, 1);

  let result = session
    .execute(parse_command("missing cost age cost").unwrap())
    .expect("missing should preserve explicit order and duplicates");
  let ExecutionResult::Missing(explicit) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(
    explicit
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["cost", "age", "cost"]
  );
  assert_eq!(explicit.rows[0], explicit.rows[2]);
}

#[test]
fn missing_reports_zero_percent_for_all_null_and_empty_relations() {
  let fixture = Fixture::new();
  let null_path = fixture.write_parquet(
    "all-null-missing.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value FROM range(2)",
  );
  let empty_path = fixture.write_parquet(
    "empty-missing.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value FROM range(0)",
  );
  let mut session = Session::new();

  session
    .execute(use_command(&null_path))
    .expect("all-null Parquet should load");
  let result = session
    .execute(parse_command("missing value").unwrap())
    .expect("all-null missing should succeed");
  let ExecutionResult::Missing(nulls) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(nulls.rows[0].total, 2);
  assert_eq!(nulls.rows[0].missing, 2);
  assert_eq!(nulls.rows[0].nonmissing, 0);
  assert_eq!(nulls.rows[0].missing_percent, 100.0);

  session
    .execute(use_command(&empty_path))
    .expect("empty Parquet should load");
  let result = session
    .execute(parse_command("missing").unwrap())
    .expect("empty missing should succeed");
  let ExecutionResult::Missing(empty) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(empty.rows[0].total, 0);
  assert_eq!(empty.rows[0].missing, 0);
  assert_eq!(empty.rows[0].nonmissing, 0);
  assert_eq!(empty.rows[0].missing_percent, 0.0);
}

#[test]
fn missing_rejects_unknown_variables_without_changing_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("missing absent absent age").unwrap())
      .unwrap_err(),
    RuntimeError::MissingUnknownVariable {
      variables: vec!["absent".to_owned(), "absent".to_owned()]
    }
  );
  assert_eq!(
    session
      .execute(parse_command("missing absent absent age").unwrap())
      .unwrap_err()
      .to_string(),
    "missing unknown variable: absent, absent"
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn missing_counts_quoted_and_container_columns_without_value_conversion() {
  let fixture = Fixture::new();
  let quoted_path = fixture.write_parquet(
    "quoted-missing.parquet",
    "SELECT 1 AS \"weird name\", NULL::INTEGER AS \"other col\"",
  );
  let list_path = fixture.write_parquet(
    "list-missing.parquet",
    "SELECT [1, 2] AS items UNION ALL SELECT NULL AS items",
  );
  let mut session = Session::new();

  session
    .execute(use_command(&quoted_path))
    .expect("quoted-column Parquet should load");
  let result = session
    .execute(parse_command("missing `weird name` `other col`").unwrap())
    .expect("missing should quote identifiers deliberately");
  let ExecutionResult::Missing(quoted) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(quoted.rows[0].variable, "weird name");
  assert_eq!(quoted.rows[0].total, 1);
  assert_eq!(quoted.rows[0].missing, 0);
  assert_eq!(quoted.rows[1].variable, "other col");
  assert_eq!(quoted.rows[1].missing, 1);

  session
    .execute(use_command(&list_path))
    .expect("list Parquet should load");
  let result = session
    .execute(parse_command("missing items").unwrap())
    .expect("missing should count container columns without owning values");
  let ExecutionResult::Missing(list) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(list.rows[0].total, 2);
  assert_eq!(list.rows[0].missing, 1);
  assert_eq!(list.rows[0].nonmissing, 1);
}

#[test]
fn missing_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("missing-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("missing cost").unwrap())
    .expect("missing should still see the prior active dataset");
  let ExecutionResult::Missing(missing) = result else {
    panic!("missing should return a Missing result");
  };
  assert_eq!(missing.rows[0].missing, 1);
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn duplicates_reports_null_equal_groups_and_preserves_key_requests() {
  let fixture = Fixture::new();
  let duplicate_path = fixture.write_parquet(
    "duplicates.parquet",
    "SELECT * FROM (VALUES (1, 'a'), (1, 'a'), (2, 'b'), (CAST(NULL AS INTEGER), 'c'), (CAST(NULL AS INTEGER), 'c'), (CAST(NULL AS INTEGER), 'd')) AS duplicates(id, label)",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&duplicate_path))
    .expect("duplicate fixture should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("duplicates report id").unwrap())
    .expect("the report alias should parse and execute");
  let ExecutionResult::Duplicates(id) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(id.variables, vec!["id"]);
  assert_eq!(id.total_rows, 6);
  assert_eq!(id.unique_groups, 3);
  assert_eq!(id.duplicate_groups, 2);
  assert_eq!(id.duplicate_rows, 5);
  assert_eq!(id.extra_rows, 3);
  assert_eq!(id.max_copies, 3);

  let result = session
    .execute(parse_command("duplicates").unwrap())
    .expect("the default key list should use schema order");
  let ExecutionResult::Duplicates(all) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(all.variables, vec!["id", "label"]);
  assert_eq!(all.total_rows, 6);
  assert_eq!(all.unique_groups, 4);
  assert_eq!(all.duplicate_groups, 2);
  assert_eq!(all.duplicate_rows, 4);
  assert_eq!(all.extra_rows, 2);
  assert_eq!(all.max_copies, 2);

  let result = session
    .execute(parse_command("duplicates id id").unwrap())
    .expect("duplicate key requests should remain valid");
  let ExecutionResult::Duplicates(repeated) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(repeated.variables, vec!["id", "id"]);
  assert_eq!(repeated.total_rows, id.total_rows);
  assert_eq!(repeated.unique_groups, id.unique_groups);
  assert_eq!(repeated.duplicate_groups, id.duplicate_groups);
  assert_eq!(repeated.duplicate_rows, id.duplicate_rows);
  assert_eq!(repeated.extra_rows, id.extra_rows);
  assert_eq!(repeated.max_copies, id.max_copies);
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn duplicates_reports_empty_relations_and_quoted_columns() {
  let fixture = Fixture::new();
  let empty_path = fixture.write_parquet(
    "empty-duplicates.parquet",
    "SELECT CAST(NULL AS INTEGER) AS \"weird name\", CAST(NULL AS VARCHAR) AS label FROM range(0)",
  );
  let quoted_path = fixture.write_parquet(
    "quoted-duplicates.parquet",
    "SELECT * FROM (VALUES (1, 'a'), (1, 'a'), (2, 'b')) AS duplicates(\"weird name\", label)",
  );
  let mut session = Session::new();

  session
    .execute(use_command(&empty_path))
    .expect("empty duplicate fixture should load");
  let result = session
    .execute(parse_command("duplicates").unwrap())
    .expect("empty duplicates should succeed");
  let ExecutionResult::Duplicates(empty) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(empty.variables, vec!["weird name", "label"]);
  assert_eq!(empty.total_rows, 0);
  assert_eq!(empty.unique_groups, 0);
  assert_eq!(empty.duplicate_groups, 0);
  assert_eq!(empty.duplicate_rows, 0);
  assert_eq!(empty.extra_rows, 0);
  assert_eq!(empty.max_copies, 0);

  session
    .execute(use_command(&quoted_path))
    .expect("quoted duplicate fixture should load");
  let result = session
    .execute(parse_command("duplicates `weird name`").unwrap())
    .expect("quoted duplicate key should execute");
  let ExecutionResult::Duplicates(quoted) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(quoted.variables, vec!["weird name"]);
  assert_eq!(quoted.total_rows, 3);
  assert_eq!(quoted.unique_groups, 2);
  assert_eq!(quoted.duplicate_groups, 1);
  assert_eq!(quoted.duplicate_rows, 2);
  assert_eq!(quoted.extra_rows, 1);
  assert_eq!(quoted.max_copies, 2);
}

#[test]
fn duplicates_avoids_internal_count_alias_collisions() {
  let fixture = Fixture::new();
  let path = fixture.write_parquet(
    "duplicate-alias-collision.parquet",
    "SELECT * FROM (VALUES (1, 'x'), (1, 'x'), (2, 'y')) AS duplicates(\"__tabdat_duplicate_count\", label)",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&path))
    .expect("alias-collision fixture should load");

  let result = session
    .execute(parse_command("duplicates `__tabdat_duplicate_count`").unwrap())
    .expect("duplicate count alias should not shadow a user column");
  let ExecutionResult::Duplicates(report) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(report.variables, vec!["__tabdat_duplicate_count"]);
  assert_eq!(report.total_rows, 3);
  assert_eq!(report.unique_groups, 2);
  assert_eq!(report.duplicate_groups, 1);
  assert_eq!(report.duplicate_rows, 2);
  assert_eq!(report.extra_rows, 1);
  assert_eq!(report.max_copies, 2);
}

#[test]
fn duplicates_rejects_unknown_variables_without_changing_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("duplicates absent absent age").unwrap())
      .unwrap_err(),
    RuntimeError::DuplicatesUnknownVariable {
      variables: vec!["absent".to_owned(), "absent".to_owned()]
    }
  );
  assert_eq!(
    session
      .execute(parse_command("duplicates absent absent age").unwrap())
      .unwrap_err()
      .to_string(),
    "duplicates unknown variable: absent, absent"
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn duplicates_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("duplicates-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("duplicates age").unwrap())
    .expect("duplicates should still see the prior active dataset");
  let ExecutionResult::Duplicates(report) = result else {
    panic!("duplicates should return a Duplicates result");
  };
  assert_eq!(report.total_rows, 3);
  assert_eq!(report.unique_groups, 3);
  assert_eq!(report.duplicate_groups, 0);
  assert_eq!(report.duplicate_rows, 0);
  assert_eq!(report.extra_rows, 0);
  assert_eq!(report.max_copies, 1);
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn isid_reports_unique_keys_and_allows_missing_with_missok() {
  let fixture = Fixture::new();
  let path = fixture.write_parquet(
    "isid-unique.parquet",
    "SELECT * FROM (VALUES (1, 1, 'a'), (1, 2, 'b'), (2, 1, 'c'), (CAST(NULL AS INTEGER), 1, 'd'), (CAST(NULL AS INTEGER), 2, 'e')) AS key_data(patient_id, visit, status)",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&path))
    .expect("isid fixture should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("isid patient_id visit, missok").unwrap())
    .expect("unique keys should pass with missok");
  let ExecutionResult::Isid(report) = result else {
    panic!("isid should return an Isid result");
  };
  assert_eq!(report.variables, vec!["patient_id", "visit"]);
  assert_eq!(report.total_rows, 5);
  assert_eq!(report.unique_groups, 5);
  assert_eq!(report.missing_key_rows, 2);
  assert!(report.missok);

  let repeated = session
    .execute(parse_command("isid patient_id visit patient_id, missok").unwrap())
    .expect("repeated key variables should remain valid");
  let ExecutionResult::Isid(repeated) = repeated else {
    panic!("isid should return an Isid result");
  };
  assert_eq!(
    repeated.variables,
    vec!["patient_id", "visit", "patient_id"]
  );
  assert_eq!(repeated.total_rows, report.total_rows);
  assert_eq!(repeated.unique_groups, report.unique_groups);
  assert_eq!(repeated.missing_key_rows, report.missing_key_rows);
  assert_eq!(repeated.missok, report.missok);
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn isid_rejects_missing_keys_without_missok_and_duplicate_groups_always() {
  let fixture = Fixture::new();
  let unique_path = fixture.write_parquet(
    "isid-missing.parquet",
    "SELECT * FROM (VALUES (1, 1), (1, 2), (2, 1), (CAST(NULL AS INTEGER), 1), (CAST(NULL AS INTEGER), 2)) AS key_data(patient_id, visit)",
  );
  let duplicate_path = fixture.write_parquet(
    "isid-duplicate.parquet",
    "SELECT * FROM (VALUES (1, 1), (1, 1), (CAST(NULL AS INTEGER), 1), (CAST(NULL AS INTEGER), 1)) AS key_data(patient_id, visit)",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&unique_path))
    .expect("missing-key fixture should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("isid patient_id visit").unwrap())
      .unwrap_err(),
    RuntimeError::IsidSemanticFailure {
      missing_key_rows: 2,
      duplicate_rows: 0,
      duplicate_groups: 0,
      missok: false,
    }
  );
  assert_eq!(
    session
      .execute(parse_command("isid patient_id visit").unwrap())
      .unwrap_err()
      .to_string(),
    "isid failed: 2 rows have missing key values (use , missok to permit them)"
  );
  assert_eq!(session.active_dataset(), Some(&before));

  session
    .execute(use_command(&duplicate_path))
    .expect("duplicate-key fixture should replace the active dataset");
  let both = session
    .execute(parse_command("isid patient_id visit").unwrap())
    .unwrap_err();
  assert_eq!(
    both.to_string(),
    "isid failed: 2 rows have missing key values (use , missok to permit them); 4 rows are in 2 duplicate key groups"
  );
  assert_eq!(
    session
      .execute(parse_command("isid patient_id visit, missok").unwrap())
      .unwrap_err(),
    RuntimeError::IsidSemanticFailure {
      missing_key_rows: 2,
      duplicate_rows: 4,
      duplicate_groups: 2,
      missok: true,
    }
  );
  assert_eq!(
    session
      .execute(parse_command("isid patient_id visit, missok").unwrap())
      .unwrap_err()
      .to_string(),
    "isid failed: 4 rows are in 2 duplicate key groups"
  );
}

#[test]
fn isid_handles_empty_relations_and_internal_alias_collisions() {
  let fixture = Fixture::new();
  let empty_path = fixture.write_parquet(
    "isid-empty.parquet",
    "SELECT CAST(NULL AS INTEGER) AS patient_id, CAST(NULL AS INTEGER) AS visit FROM range(0)",
  );
  let alias_path = fixture.write_parquet(
    "isid-alias-collision.parquet",
    "SELECT * FROM (VALUES (1, 10), (2, 20)) AS key_data(\"__tabdat_isid_count\", visit)",
  );
  let mut session = Session::new();

  session
    .execute(use_command(&empty_path))
    .expect("empty isid fixture should load");
  let result = session
    .execute(parse_command("isid patient_id visit").unwrap())
    .expect("empty key relation should pass");
  let ExecutionResult::Isid(empty) = result else {
    panic!("isid should return an Isid result");
  };
  assert_eq!(empty.variables, vec!["patient_id", "visit"]);
  assert_eq!(empty.total_rows, 0);
  assert_eq!(empty.unique_groups, 0);
  assert_eq!(empty.missing_key_rows, 0);
  assert!(!empty.missok);

  session
    .execute(use_command(&alias_path))
    .expect("alias-collision fixture should load");
  let result = session
    .execute(parse_command("isid `__tabdat_isid_count` visit").unwrap())
    .expect("the internal alias should not shadow a user column");
  let ExecutionResult::Isid(alias) = result else {
    panic!("isid should return an Isid result");
  };
  assert_eq!(alias.variables, vec!["__tabdat_isid_count", "visit"]);
  assert_eq!(alias.total_rows, 2);
  assert_eq!(alias.unique_groups, 2);
  assert_eq!(alias.missing_key_rows, 0);
}

#[test]
fn isid_rejects_unknown_variables_without_changing_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("isid absent absent age").unwrap())
      .unwrap_err(),
    RuntimeError::IsidUnknownVariable {
      variables: vec!["absent".to_owned(), "absent".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("isid absent absent age").unwrap())
      .unwrap_err()
      .to_string(),
    "isid unknown variable: absent, absent"
  );
  assert_eq!(session.active_dataset(), Some(&before));
}

#[test]
fn summarize_returns_requested_statistics_in_order_and_is_read_only() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(
      parse_command(&format!("use {}", fixture.parquet.display()))
        .expect("the fixture use should parse"),
    )
    .expect("parsed eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("summarize age cost").unwrap())
    .expect("summarize should return requested numeric columns");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(summary.rows.len(), 2);
  assert_eq!(summary.rows[0].variable, "age");
  assert_eq!(summary.rows[0].count, 3);
  assert_eq!(summary.rows[0].mean, Some(42.0));
  assert_eq!(summary.rows[0].std_dev, Some(12.0));
  assert_eq!(summary.rows[0].minimum, Some(CellValue::SignedInteger(30)));
  assert_eq!(summary.rows[0].maximum, Some(CellValue::SignedInteger(54)));
  assert_eq!(summary.rows[1].variable, "cost");
  assert_eq!(summary.rows[1].count, 2);
  assert_eq!(summary.rows[1].mean, Some(125.0));
  assert!((summary.rows[1].std_dev.unwrap() - 35.35533905932738).abs() < 1e-12);
  assert_eq!(
    summary.rows[1].minimum,
    Some(CellValue::Decimal {
      width: 4,
      scale: 1,
      value: 1000,
    })
  );
  assert_eq!(
    summary.rows[1].maximum,
    Some(CellValue::Decimal {
      width: 4,
      scale: 1,
      value: 1500,
    })
  );
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("summarize age cost").unwrap())
    .expect("repeated summarize should remain read-only");
  assert_eq!(repeated, ExecutionResult::Summarize(summary));
  assert_eq!(session.active_dataset(), Some(&expected));
}

#[test]
fn summarize_without_variables_selects_numeric_columns_in_schema_order() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("summarize").unwrap())
    .expect("summarize should select numeric columns by default");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(
    summary
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "cost"]
  );
  assert_eq!(summary.rows[1].count, 3);
  assert_eq!(summary.rows[1].mean, Some(25.0));
  assert_eq!(
    summary.rows[1].minimum,
    Some(CellValue::Decimal {
      width: 3,
      scale: 1,
      value: 225,
    })
  );
  assert_eq!(
    summary.rows[1].maximum,
    Some(CellValue::Decimal {
      width: 3,
      scale: 1,
      value: 275,
    })
  );
}

#[test]
fn summarize_preserves_explicit_variable_order_and_duplicates() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("summarize cost age cost").unwrap())
    .expect("summarize should preserve explicit order");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(
    summary
      .rows
      .iter()
      .map(|row| row.variable.as_str())
      .collect::<Vec<_>>(),
    vec!["cost", "age", "cost"]
  );
  assert_eq!(summary.rows[0], summary.rows[2]);
}

#[test]
fn summarize_rejects_unknown_and_non_numeric_variables_with_exact_errors() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  assert_eq!(
    session
      .execute(parse_command("summarize missing age").unwrap())
      .unwrap_err(),
    RuntimeError::SummaryUnknownVariable {
      variables: vec!["missing".to_owned()]
    }
  );
  assert_eq!(
    session
      .execute(parse_command("summarize missing age").unwrap())
      .unwrap_err()
      .to_string(),
    "summarize unknown variable: missing"
  );
  assert_eq!(
    session
      .execute(parse_command("summarize sex").unwrap())
      .unwrap_err(),
    RuntimeError::SummaryRequiresNumeric {
      variables: vec!["sex".to_owned()]
    }
  );
  assert_eq!(
    session
      .execute(parse_command("summarize sex").unwrap())
      .unwrap_err()
      .to_string(),
    "summarize requires numeric variables: sex"
  );
}

#[test]
fn summarize_reports_no_numeric_columns_and_all_null_statistics() {
  let fixture = Fixture::new();
  let text_path = fixture.write_parquet("text.parquet", "SELECT 'F' AS sex");
  let null_path = fixture.write_parquet(
    "all-null.parquet",
    "SELECT CAST(NULL AS DOUBLE) AS value FROM range(2)",
  );
  let one_value_path =
    fixture.write_parquet("one-value.parquet", "SELECT CAST(7.5 AS DOUBLE) AS value");
  let mut session = Session::new();

  session
    .execute(use_command(&text_path))
    .expect("text-only eager local Parquet should load");
  assert_eq!(
    session
      .execute(parse_command("summarize").unwrap())
      .unwrap_err(),
    RuntimeError::SummaryNoNumericColumns
  );
  assert_eq!(
    session
      .execute(parse_command("summarize").unwrap())
      .unwrap_err()
      .to_string(),
    "summarize found no numeric columns"
  );

  session
    .execute(use_command(&null_path))
    .expect("all-null numeric Parquet should load");
  let result = session
    .execute(parse_command("summarize value").unwrap())
    .expect("all-null numeric summary should succeed");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(summary.rows[0].count, 0);
  assert_eq!(summary.rows[0].mean, None);
  assert_eq!(summary.rows[0].std_dev, None);
  assert_eq!(summary.rows[0].minimum, None);
  assert_eq!(summary.rows[0].maximum, None);

  session
    .execute(use_command(&one_value_path))
    .expect("single-value numeric Parquet should load");
  let result = session
    .execute(parse_command("summarize value").unwrap())
    .expect("single-value numeric summary should succeed");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(summary.rows[0].count, 1);
  assert_eq!(summary.rows[0].mean, Some(7.5));
  assert_eq!(summary.rows[0].std_dev, None);
  assert_eq!(summary.rows[0].minimum, Some(CellValue::Float(7.5)));
  assert_eq!(summary.rows[0].maximum, Some(CellValue::Float(7.5)));
}

#[test]
fn summarize_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("summarize-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("summarize age").unwrap())
    .expect("summarize should still see the prior active dataset");
  let ExecutionResult::Summarize(summary) = result else {
    panic!("summarize should return a Summarize result");
  };
  assert_eq!(summary.rows[0].count, 3);
  assert_eq!(session.active_dataset(), Some(&before));
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
fn tail_returns_ordered_owned_rows_with_decimal_and_null_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let use_text = format!("use {}", fixture.parquet.display());
  session
    .execute(parse_command(&use_text).expect("the fixture use should parse"))
    .expect("parsed eager local Parquet should load");
  let expected = session
    .active_dataset()
    .expect("the load should publish active metadata")
    .clone();

  let result = session
    .execute(parse_command("tail 2").unwrap())
    .expect("tail should return the final two rows");
  let ExecutionResult::Tail(preview) = result else {
    panic!("tail should return a Tail result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(
    preview.rows,
    vec![
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
      vec![
        CellValue::SignedInteger(54),
        CellValue::Decimal {
          width: 3,
          scale: 1,
          value: 275,
        },
        CellValue::Text("F".to_owned()),
        CellValue::Null,
      ],
    ]
  );
  assert_eq!(session.active_dataset(), Some(&expected));

  let repeated = session
    .execute(parse_command("tail 2").unwrap())
    .expect("repeated tail should remain read-only");
  assert_eq!(repeated, ExecutionResult::Tail(preview));
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
fn tail_default_and_oversized_limits_return_all_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  for command in ["tail", "tail 99", "tail 9223372036854775807"] {
    let result = session
      .execute(parse_command(command).unwrap())
      .expect("tail should return all rows when the limit is large enough");
    let ExecutionResult::Tail(preview) = result else {
      panic!("tail should return a Tail result");
    };
    assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
    assert_eq!(preview.rows.len(), 3);
    assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
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
fn tail_preserves_nontrivial_source_insertion_order() {
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
    .execute(parse_command("tail 2").unwrap())
    .expect("tail should return the final two source rows");
  let ExecutionResult::Tail(preview) = result else {
    panic!("tail should return a Tail result");
  };
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(30));
  assert_eq!(preview.rows[1][0], CellValue::SignedInteger(54));
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
fn tail_zero_returns_columns_without_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("eager local Parquet should load");

  let result = session
    .execute(parse_command("tail 0").unwrap())
    .expect("tail zero should return an empty preview");
  let ExecutionResult::Tail(preview) = result else {
    panic!("tail should return a Tail result");
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
fn tail_above_i64_limit_fails_deterministically_and_preserves_metadata() {
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
      .execute(parse_command(&format!("tail {too_large}")).unwrap())
      .unwrap_err(),
    RuntimeError::PreviewFailed { command: "tail" }
  );
  assert_eq!(
    session
      .execute(parse_command(&format!("tail {too_large}")).unwrap())
      .unwrap_err()
      .to_string(),
    "tail failed"
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
fn tail_after_failed_replacement_keeps_the_prior_dataset() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("initial eager local Parquet should load");
  let before = session
    .active_dataset()
    .expect("initial load should publish active metadata")
    .clone();

  let corrupt_path = fixture.root.join("tail-replacement-corrupt.parquet");
  fs::write(&corrupt_path, "not parquet").expect("corrupt fixture should be written");
  assert_eq!(
    session.execute(use_command(&corrupt_path)).unwrap_err(),
    RuntimeError::ParquetRead {
      path: corrupt_path.clone()
    }
  );

  let result = session
    .execute(parse_command("tail 1").unwrap())
    .expect("tail should still see the prior active dataset");
  let ExecutionResult::Tail(preview) = result else {
    panic!("tail should return a Tail result");
  };
  assert_eq!(preview.rows.len(), 1);
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(54));
  assert_eq!(session.active_dataset(), Some(&before));
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
fn rename_requires_an_active_dataset_before_execution() {
  let mut session = Session::new();
  let command = Command::Rename {
    old_name: "old_name".to_owned(),
    new_name: "new_name".to_owned(),
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "rename" }
  );
}

#[test]
fn generate_requires_an_active_dataset_before_execution() {
  let mut session = Session::new();
  let command = parse_command("generate age2 = age + 1").expect("generate should parse");

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "generate"
    }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn replace_requires_an_active_dataset_before_execution() {
  let mut session = Session::new();
  let command = parse_command("replace age = age + 1 if age > 0")
    .expect("replace should parse before execution");

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "replace" }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn select_requires_an_active_dataset() {
  let mut session = Session::new();
  let command = Command::Select {
    variables: vec!["age".to_owned(), "sex".to_owned()],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "select" }
  );
}

#[test]
fn sort_requires_an_active_dataset_before_execution() {
  let mut session = Session::new();
  let command = Command::Sort {
    variables: vec!["age".to_owned(), "sex".to_owned()],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "sort" }
  );
}

#[test]
fn gsort_requires_an_active_dataset_before_execution() {
  let mut session = Session::new();
  let command = Command::Gsort {
    keys: vec![tabdat_language::SortKey {
      variable: "age".to_owned(),
      descending: true,
    }],
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "gsort" }
  );
}

#[test]
fn save_requires_an_active_dataset() {
  let mut session = Session::new();
  let command = Command::Save {
    path: "output.parquet".to_owned(),
    replace: false,
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "save" }
  );
}

#[test]
fn export_requires_an_active_dataset() {
  let mut session = Session::new();
  let command = Command::Export {
    path: "output.csv".to_owned(),
    replace: true,
  };

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "export" }
  );
}
