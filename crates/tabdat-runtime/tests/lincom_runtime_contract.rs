use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::Session;

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
      "tabdat-runtime-lincom-{}-{nonce}-{fixture_id}",
      std::process::id(),
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("fixture.parquet");
    let parquet_string = parquet.to_string_lossy().into_owned();
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES \
          (1.0, 10.0, 5.0), \
          (2.0, 12.0, 6.0), \
          (3.0, 15.0, 8.0), \
          (4.0, 18.0, 9.0), \
          (5.0, 20.0, 11.0), \
          (6.0, 25.0, 12.0) \
        ) AS t(x1, x2, y)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture should write to Parquet");
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
fn lincom_executes_after_linear_regression() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");

  let result = session.execute(parse_command("lincom x1 - x2").unwrap());

  assert!(result.is_ok(), "lincom should execute: {result:?}");
}
