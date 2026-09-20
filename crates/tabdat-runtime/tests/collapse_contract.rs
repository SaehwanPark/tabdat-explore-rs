use std::fs;
use std::path::PathBuf;
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
      "tabdat-runtime-collapse-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir(&root).expect("fixture directory should be created");
    let parquet = root.join("collapse.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES
          (1, 'F', 30, 10.0, 'a'),
          (2, 'F', 50, 20.0, 'b'),
          (3, 'M', 42, 30.0, 'c'),
          (4, CAST(NULL AS VARCHAR), 10, CAST(NULL AS DOUBLE), CAST(NULL AS VARCHAR))
        ) AS patients(row_id, sex, age, cost, note)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
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

#[test]
fn collapse_requires_an_active_dataset_without_initializing_a_backend() {
  let mut session = Session::new();

  assert_eq!(
    session
      .execute(parse_command("collapse mean age, by(sex)").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "collapse"
    }
  );
  assert!(session.active_dataset().is_none());
}

#[test]
fn collapse_replaces_active_dataset_with_ordered_owned_groups() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  session
    .execute(parse_command("label variable sex \"Sex\"").unwrap())
    .expect("group label should execute");

  let result = session
    .execute(parse_command("collapse mean age cost, by(sex)").unwrap())
    .expect("mean collapse should execute");
  let ExecutionResult::Collapse(result) = result else {
    panic!("collapse should return a collapse result");
  };
  assert_eq!(result.dataset.row_count, 3);
  assert_eq!(
    result
      .dataset
      .columns
      .iter()
      .map(|column| column.name.as_str())
      .collect::<Vec<_>>(),
    ["sex", "mean_age", "mean_cost"]
  );
  assert_eq!(result.dataset.source, fixture.parquet);
  assert_eq!(result.dataset.execution_mode, ExecutionMode::Eager);

  let ExecutionResult::Head(preview) = session
    .execute(parse_command("head 5").unwrap())
    .expect("collapsed preview should execute")
  else {
    panic!("head should return a preview result");
  };
  assert_eq!(
    preview.rows,
    vec![
      vec![
        CellValue::Text("F".to_owned()),
        CellValue::Float(40.0),
        CellValue::Float(15.0),
      ],
      vec![
        CellValue::Text("M".to_owned()),
        CellValue::Float(42.0),
        CellValue::Float(30.0),
      ],
      vec![CellValue::Null, CellValue::Float(10.0), CellValue::Null,],
    ]
  );
  let labels = session
    .active_label_metadata()
    .expect("group label should survive collapse");
  assert_eq!(
    labels.variable_labels,
    vec![(String::from("sex"), String::from("Sex"))]
  );
}

#[test]
fn collapse_supports_all_statistics_and_count_ignores_null_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let cases = [
    ("collapse count note, by(sex)", "count_note"),
    ("collapse mean age, by(sex)", "mean_age"),
    ("collapse sum age, by(sex)", "sum_age"),
    ("collapse min age, by(sex)", "min_age"),
    ("collapse max age, by(sex)", "max_age"),
  ];
  for (command_text, aggregate_name) in cases {
    session
      .execute(fixture.command())
      .expect("fixture should load for each statistic");
    let result = session
      .execute(parse_command(command_text).unwrap())
      .expect("collapse statistic should execute");
    let ExecutionResult::Collapse(result) = result else {
      panic!("collapse should return a collapse result");
    };
    assert_eq!(
      result
        .dataset
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect::<Vec<_>>(),
      ["sex", aggregate_name]
    );
  }

  session
    .execute(fixture.command())
    .expect("fixture should reload");
  let ExecutionResult::Collapse(_) = session
    .execute(parse_command("collapse count note, by(sex)").unwrap())
    .expect("count collapse should execute")
  else {
    panic!("count should return a collapse result");
  };
  let ExecutionResult::Head(preview) = session
    .execute(parse_command("head 5").unwrap())
    .expect("count preview should execute")
  else {
    panic!("head should return a preview result");
  };
  assert_eq!(
    preview.rows,
    vec![
      vec![CellValue::Text("F".to_owned()), CellValue::SignedInteger(2)],
      vec![CellValue::Text("M".to_owned()), CellValue::SignedInteger(1)],
      vec![CellValue::Null, CellValue::SignedInteger(0)],
    ]
  );
}

#[test]
fn collapse_validation_failures_preserve_active_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(fixture.command())
    .expect("fixture should load");
  let before = session
    .active_dataset()
    .expect("load should publish active metadata")
    .clone();

  assert_eq!(
    session
      .execute(parse_command("collapse mean missing, by(sex)").unwrap())
      .unwrap_err(),
    RuntimeError::CollapseUnknownVariable {
      variables: vec!["missing".to_owned()],
    }
  );
  assert_eq!(
    session
      .execute(parse_command("collapse mean note, by(sex)").unwrap())
      .unwrap_err(),
    RuntimeError::CollapseRequiresNumeric {
      variables: vec!["note".to_owned()],
    }
  );
  assert_eq!(session.active_dataset(), Some(&before));

  let ExecutionResult::Head(preview) = session
    .execute(parse_command("head 5").unwrap())
    .expect("the original preview should remain available")
  else {
    panic!("head should return a preview result");
  };
  assert_eq!(preview.rows.len(), 4);
  assert_eq!(preview.rows[0][0], CellValue::SignedInteger(1));
}
