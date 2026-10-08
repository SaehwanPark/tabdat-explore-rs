use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{CellValue, ExecutionResult, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-vif-{}-{nonce}-{fixture_id}",
      std::process::id(),
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("fixture.parquet");
    let parquet_string = parquet.to_string_lossy().into_owned();
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES \
          (-2.0, -3.0, 1.0), \
          (-1.0, -4.0, 2.0), \
          (0.0, 2.0, 3.0), \
          (1.0, 0.0, 4.0), \
          (2.0, 5.0, 6.0) \
        ) AS t(x1, x2, y)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture table should write to Parquet");
    Self { root, parquet }
  }

  fn load(&self, session: &mut Session) {
    session
      .execute(parse_command(&format!("use {}", self.parquet.display())).unwrap())
      .expect("fixture should load");
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn estat_vif_uses_the_fitted_sample_and_returns_ordered_values() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");
  session
    .execute(parse_command("select y").unwrap())
    .expect("active data can be transformed after fitting");
  let dataset_before = session.active_dataset().unwrap().clone();
  let model_before = session.last_regression().unwrap().clone();

  let result = session
    .execute(parse_command("estat vif").unwrap())
    .expect("VIF should use the retained regression design");
  let ExecutionResult::Table(table) = result else {
    panic!("estat vif should return an owned table");
  };

  assert_eq!(table.headers, vec!["Variable", "VIF"]);
  assert_eq!(table.rows.len(), 3);
  assert_eq!(table.rows[0][0], CellValue::Text("x1".to_owned()));
  assert_eq!(table.rows[1][0], CellValue::Text("x2".to_owned()));
  assert_eq!(table.rows[2][0], CellValue::Text("mean_vif".to_owned()));
  let expected = 27.0 / 7.0;
  for row in &table.rows {
    let CellValue::Float(actual) = row[1] else {
      panic!("VIF values should be floating-point numbers");
    };
    assert!((actual - expected).abs() <= 1e-12, "VIF was {actual}");
  }
  assert_eq!(session.active_dataset(), Some(&dataset_before));
  assert_eq!(session.last_regression(), Some(&model_before));
}
