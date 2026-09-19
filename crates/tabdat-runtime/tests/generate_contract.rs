use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{Command, DataSource, ExecutionMode, parse_command};
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
      "tabdat-runtime-generate-{0}-{1}-{2}",
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
fn generate_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("generate age2 = age + 1").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "generate"
    }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn parsed_generate_appends_numeric_column_and_preserves_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");

  let result = session
    .execute(parse_command("generate age2 = age + 1").unwrap())
    .expect("numeric generate should execute");
  let ExecutionResult::Generate(generate) = result else {
    panic!("generate should return a Generate result");
  };
  assert_eq!(generate.dataset.source, fixture.parquet);
  assert_eq!(generate.dataset.row_count, 3);
  assert_eq!(
    generate
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    vec!["age", "bmi", "sex", "cost", "age2"]
  );
  assert_eq!(generate.dataset.execution_mode, ExecutionMode::Eager);
  assert_eq!(generate.dataset.lazy_engine, None);

  let preview = session
    .execute(parse_command("head 3").unwrap())
    .expect("generated relation should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview.rows[0][4],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 31
    }
  );
  assert_eq!(
    preview.rows[1][4],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 43
    }
  );
  assert_eq!(
    preview.rows[2][4],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 55
    }
  );
}

#[test]
fn generate_supports_unary_minus_precedence_parentheses_literals_and_quoted_names() {
  let fixture = Fixture::new();
  let quoted = fixture.write_parquet(
    "quoted.parquet",
    "SELECT 2 AS \"age value\", 5 AS base, CAST(NULL AS INTEGER) AS value",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&quoted))
    .expect("quoted fixture should load");

  session
    .execute(parse_command("generate `age squared` = (`age value` + 1) * 2").unwrap())
    .expect("parenthesized arithmetic should execute");
  session
    .execute(parse_command("generate neg = -base").unwrap())
    .expect("unary minus should execute");
  session
    .execute(parse_command("generate direct = 7 / 2").unwrap())
    .expect("numeric literals should execute");

  let preview = session
    .execute(parse_command("head 1").unwrap())
    .expect("generated relation should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(
    preview.columns,
    vec!["age value", "base", "value", "age squared", "neg", "direct"]
  );
  assert_eq!(
    preview.rows[0][3],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: 6
    }
  );
  assert_eq!(
    preview.rows[0][4],
    CellValue::Decimal {
      width: 38,
      scale: 0,
      value: -5
    }
  );
  assert_eq!(preview.rows[0][5], CellValue::Float(3.5));
}

#[test]
fn generate_keeps_empty_relation_schema_and_zero_rows() {
  let fixture = Fixture::new();
  let empty = fixture.write_parquet(
    "empty.parquet",
    "SELECT CAST(NULL AS INTEGER) AS value WHERE FALSE",
  );
  let mut session = Session::new();
  session
    .execute(use_command(&empty))
    .expect("empty fixture should load");

  let result = session
    .execute(parse_command("generate value2 = value + 1").unwrap())
    .expect("generate should add a column to an empty relation");
  let ExecutionResult::Generate(generate) = result else {
    panic!("generate should return a Generate result");
  };
  assert_eq!(generate.dataset.row_count, 0);
  assert_eq!(
    generate
      .dataset
      .columns
      .last()
      .map(|column| column.name.as_str()),
    Some("value2")
  );
}

#[test]
fn generate_rejects_invalid_forms_before_mutating_metadata_or_rows() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");
  let before = session.active_dataset().unwrap().clone();

  let failures = [
    (
      "generate age = age + 1",
      RuntimeError::GenerateTargetExists {
        variable: "age".to_owned(),
      },
    ),
    (
      "generate missing2 = missing + 1",
      RuntimeError::GenerateUnknownVariable {
        variables: vec!["missing".to_owned()],
      },
    ),
    (
      "generate sex2 = sex + 1",
      RuntimeError::GenerateTypeMismatch {
        message: "expression type mismatch: arithmetic requires numeric operands".to_owned(),
      },
    ),
    (
      "generate label = 'x'",
      RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support string expressions".to_owned(),
      },
    ),
    (
      "generate test = age > 0",
      RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support comparison expressions".to_owned(),
      },
    ),
    (
      "generate test = null",
      RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support NULL expressions".to_owned(),
      },
    ),
    (
      "generate test = abs(age)",
      RuntimeError::GenerateUnsupportedExpression {
        message: "generate does not support function calls".to_owned(),
      },
    ),
  ];
  for (command, expected) in failures {
    assert_eq!(
      session
        .execute(parse_command(command).unwrap())
        .unwrap_err(),
      expected
    );
    assert_eq!(session.active_dataset(), Some(&before));
  }

  let preview = session
    .execute(parse_command("head 3").unwrap())
    .expect("failed generate commands should preserve the active relation");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.columns, vec!["age", "bmi", "sex", "cost"]);
  assert_eq!(preview.rows.len(), 3);
}

#[test]
fn generate_propagates_nulls_and_allows_repeated_successes() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(use_command(&fixture.parquet))
    .expect("fixture should load");

  session
    .execute(parse_command("generate age2 = age + 1").unwrap())
    .expect("first generation should execute");
  let result = session
    .execute(parse_command("generate cost2 = cost / 2").unwrap())
    .expect("second generation should execute");
  let ExecutionResult::Generate(generate) = result else {
    panic!("generate should return a Generate result");
  };
  assert_eq!(generate.dataset.columns.len(), 6);

  let preview = session
    .execute(parse_command("head 3").unwrap())
    .expect("generated relation should be previewable");
  let ExecutionResult::Head(preview) = preview else {
    panic!("head should return a Head result");
  };
  assert_eq!(preview.rows[0][5], CellValue::Float(50.0));
  assert_eq!(preview.rows[2][5], CellValue::Null);
}

#[test]
fn generate_rejects_unsafe_unsigned_arithmetic_before_staging() {
  let fixture = Fixture::new();
  let unsigned = fixture.write_parquet("unsigned.parquet", "SELECT CAST(1 AS UBIGINT) AS value");
  let mut session = Session::new();
  session
    .execute(use_command(&unsigned))
    .expect("unsigned fixture should load");
  let before = session.active_dataset().unwrap().clone();

  assert_eq!(
    session
      .execute(parse_command("generate bad = value - 1").unwrap())
      .unwrap_err(),
    RuntimeError::GenerateTypeMismatch {
      message:
        "expression type mismatch: unsigned numeric values do not support subtraction or unary minus"
          .to_owned(),
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));
}
