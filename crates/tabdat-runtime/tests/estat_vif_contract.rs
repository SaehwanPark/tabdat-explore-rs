use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session, TableResult};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    Self::from_query(
      "SELECT * FROM (VALUES \
        (-2.0, -3.0, 1.0, 100.0, 100.0), \
        (-1.0, -4.0, 2.0, 1.0, 1.0), \
        (0.0, 2.0, 3.0, 1.0, 1.0), \
        (1.0, 0.0, 4.0, 1.0, 1.0), \
        (2.0, 5.0, 6.0, 1.0, 1.0) \
      ) AS t(x1, x2, y, w, sigma)",
    )
  }

  fn from_query(query: &str) -> Self {
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
        &format!("COPY ({query}) TO ? (FORMAT PARQUET)"),
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

fn table_from(result: ExecutionResult) -> TableResult {
  let ExecutionResult::Table(table) = result else {
    panic!("estat vif should return an owned table");
  };
  table
}

fn assert_two_predictor_vifs(table: &TableResult, expected: f64) {
  assert_eq!(table.headers, vec!["Variable".to_owned(), "VIF".to_owned()]);
  assert_eq!(table.rows.len(), 3);
  assert_eq!(table.rows[0][0], CellValue::Text("x1".to_owned()));
  assert_eq!(table.rows[1][0], CellValue::Text("x2".to_owned()));
  assert_eq!(table.rows[2][0], CellValue::Text("mean_vif".to_owned()));
  for row in &table.rows {
    let CellValue::Float(actual) = &row[1] else {
      panic!("VIF values should be floating-point numbers");
    };
    assert!((*actual - expected).abs() <= 1e-12, "VIF was {actual}");
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
  let table = table_from(result);

  assert_two_predictor_vifs(&table, 27.0 / 7.0);
  assert_eq!(session.active_dataset(), Some(&dataset_before));
  assert_eq!(session.last_regression(), Some(&model_before));
}

#[test]
fn estat_vif_uses_unweighted_auxiliary_models_for_ols_wls_and_gls() {
  for regression in [
    "regress y x1 x2",
    "regress y x1 x2, wls(w)",
    "regress y x1 x2, gls(sigma)",
  ] {
    let fixture = Fixture::new();
    let mut session = Session::new();
    fixture.load(&mut session);
    session
      .execute(parse_command(regression).unwrap())
      .expect("regression should fit");
    let table = table_from(
      session
        .execute(parse_command("estat vif").unwrap())
        .expect("VIF should be available for each linear estimator"),
    );
    assert_two_predictor_vifs(&table, 27.0 / 7.0);
  }
}

#[test]
fn estat_vif_uses_the_weighted_regression_complete_case_sample() {
  let fixture = Fixture::from_query(
    "SELECT * FROM (VALUES \
      (-2.0, -3.0, 1.0, NULL, NULL), \
      (-1.0, -4.0, 2.0, 1.0, 1.0), \
      (0.0, 2.0, 3.0, 1.0, 1.0), \
      (1.0, 0.0, 4.0, 1.0, 1.0), \
      (2.0, 5.0, 6.0, 1.0, 1.0) \
    ) AS t(x1, x2, y, w, sigma)",
  );
  for regression in ["regress y x1 x2, wls(w)", "regress y x1 x2, gls(sigma)"] {
    let mut session = Session::new();
    fixture.load(&mut session);
    session
      .execute(parse_command(regression).unwrap())
      .expect("weighted regression should exclude the row with a missing weight");
    let table = table_from(
      session
        .execute(parse_command("estat vif").unwrap())
        .expect("VIF should use the fitted complete-case design"),
    );
    assert_two_predictor_vifs(&table, 171.0 / 46.0);
  }
}

#[test]
fn estat_vif_uses_the_models_no_intercept_convention() {
  let fixture = Fixture::from_query(
    "SELECT * FROM (VALUES \
      (1.0, 5.0, 2.0), \
      (2.0, 7.0, 3.0), \
      (3.0, 1.0, 1.0), \
      (4.0, 6.0, 7.0), \
      (5.0, 3.0, 4.0) \
    ) AS t(x1, x2, y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2, noconstant").unwrap())
    .expect("no-intercept regression should fit");
  let table = table_from(
    session
      .execute(parse_command("estat vif").unwrap())
      .expect("VIF should follow the model's intercept convention"),
  );
  let expected = 6600.0 / 2879.0;
  assert_eq!(table.rows[0][0], CellValue::Text("x1".to_owned()));
  assert_eq!(table.rows[1][0], CellValue::Text("x2".to_owned()));
  for row in &table.rows {
    let CellValue::Float(actual) = &row[1] else {
      panic!("VIF values should be floating-point numbers");
    };
    assert!((*actual - expected).abs() <= 1e-12, "VIF was {actual}");
  }
}

#[test]
fn estat_vif_returns_one_for_a_single_predictor() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1").unwrap())
    .expect("regression should fit");
  let table = table_from(
    session
      .execute(parse_command("estat vif").unwrap())
      .expect("single-predictor VIF should be available"),
  );
  assert_eq!(table.rows.len(), 2);
  assert_eq!(table.rows[0][0], CellValue::Text("x1".to_owned()));
  assert_eq!(table.rows[0][1], CellValue::Float(1.0));
  assert_eq!(table.rows[1][0], CellValue::Text("mean_vif".to_owned()));
  assert_eq!(table.rows[1][1], CellValue::Float(1.0));
}

#[test]
fn estat_vif_normalizes_auxiliary_fit_failures() {
  let fixture = Fixture::from_query(
    "SELECT * FROM (VALUES \
      (1.0, 2.0), \
      (2.0, 3.0), \
      (3.0, 5.0), \
      (4.0, 4.0) \
    ) AS t(x, y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x, noconstant").unwrap())
    .expect("single-predictor no-intercept regression should fit");
  assert_eq!(
    session
      .execute(parse_command("estat vif").unwrap())
      .unwrap_err(),
    RuntimeError::EstatVifFailed
  );
}

#[test]
fn estat_vif_checks_dataset_then_prior_model_requirements() {
  let mut session = Session::new();
  assert_eq!(
    session
      .execute(parse_command("estat vif").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "estat" }
  );

  let fixture = Fixture::new();
  fixture.load(&mut session);
  assert_eq!(
    session
      .execute(parse_command("estat vif").unwrap())
      .unwrap_err(),
    RuntimeError::EstatRequiresPriorRegression
  );
}
