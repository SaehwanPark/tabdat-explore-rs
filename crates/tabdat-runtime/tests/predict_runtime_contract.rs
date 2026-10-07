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
  fn new_with_sql(sql: &str) -> Self {
    let root = fixture_root("runtime-predict");
    let parquet = root.join("fixture.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!("COPY ({sql}) TO ? (FORMAT PARQUET)"),
        [&parquet_string],
      )
      .expect("fixture table should write to Parquet");
    Self { root, parquet }
  }

  fn new_with_csv(name: &str, csv: &str) -> Self {
    let root = fixture_root("runtime-predict");
    let csv_path = root.join(name);
    let parquet = root.join("fixture.parquet");
    fs::write(&csv_path, csv).expect("fixture CSV should be written");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv_path.to_string_lossy().into_owned();
    let parquet_string = parquet.to_string_lossy().into_owned();
    let escaped_csv_path = csv_string.replace('\'', "''");
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{escaped_csv_path}')) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("fixture CSV should write to Parquet");
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

fn fixture_root(prefix: &str) -> PathBuf {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock should be after Unix epoch")
    .as_nanos();
  let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
  let root = std::env::temp_dir().join(format!(
    "tabdat-{prefix}-{}-{nonce}-{fixture_id}",
    std::process::id(),
  ));
  fs::create_dir_all(&root).expect("fixture directory should be created");
  root
}

fn number(value: &CellValue) -> Option<f64> {
  match value {
    CellValue::Float(value) => Some(*value),
    CellValue::SignedInteger(value) => Some(*value as f64),
    CellValue::UnsignedInteger(value) => Some(*value as f64),
    CellValue::Decimal { value, scale, .. } => {
      Some(*value as f64 / 10_f64.powi(*scale as i32))
    }
    CellValue::Null | CellValue::Boolean(_) | CellValue::Text(_) | CellValue::Bytes(_) => None,
  }
}

fn close_with_tolerance(actual: f64, expected: f64, absolute_tolerance: f64, relative_tolerance: f64) {
  let allowed = absolute_tolerance.max(relative_tolerance * expected.abs());
  assert!(
    (actual - expected).abs() <= allowed,
    "actual {actual} differs from expected {expected} by {}, allowed {allowed}",
    (actual - expected).abs(),
  );
}

#[test]
fn regression_xb_and_residual_predictions_preserve_rows_and_missingness() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (0.0, 1.0), \
      (1.0, 4.0), \
      (2.0, 4.0), \
      (3.0, 8.0), \
      (4.0, 10.0), \
      (CAST(NULL AS DOUBLE), 8.0), \
      (5.0, CAST(NULL AS DOUBLE)) \
    ) AS t(x, y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x").unwrap())
    .expect("regression should fit");
  let model = session.last_regression().unwrap().clone();

  let xb_result = session.execute(parse_command("predict fitted").unwrap());
  assert!(xb_result.is_ok(), "default xb prediction should execute: {xb_result:?}");
  let residual_result = session.execute(parse_command("predict residual, residuals").unwrap());
  assert!(
    residual_result.is_ok(),
    "residual prediction should execute: {residual_result:?}"
  );

  let metadata = session.active_dataset().unwrap();
  assert_eq!(metadata.row_count, 7);
  assert_eq!(
    metadata.columns.iter().map(|column| column.name.as_str()).collect::<Vec<_>>(),
    vec!["x", "y", "fitted", "residual"]
  );
  assert_eq!(metadata.columns[2].data_type, "DOUBLE");
  assert_eq!(session.last_regression(), Some(&model));

  let result = session
    .execute(parse_command("head 7").unwrap())
    .expect("predicted relation should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a preview");
  };
  let expected_xb = [1.0, 3.2, 5.4, 7.6, 9.8, f64::NAN, 12.0];
  let expected_residual = [0.0, 0.8, -1.4, 0.4, 0.2, f64::NAN, f64::NAN];
  for (index, row) in preview.rows.iter().enumerate() {
    match number(&row[2]) {
      Some(actual) => close_with_tolerance(actual, expected_xb[index], 1e-12, 1e-12),
      None => assert!(expected_xb[index].is_nan(), "unexpected missing xb at row {index}"),
    }
    match number(&row[3]) {
      Some(actual) => close_with_tolerance(actual, expected_residual[index], 1e-12, 1e-12),
      None => assert!(
        expected_residual[index].is_nan(),
        "unexpected missing residual at row {index}"
      ),
    }
  }
}

#[test]
fn weighted_regression_predictions_do_not_require_weight_columns_after_fit() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (0.0, 1.0, 1.0, 1.0), \
      (1.0, 4.0, 2.0, 2.0), \
      (2.0, 4.0, 1.0, 1.0), \
      (3.0, 8.0, 2.0, 2.0), \
      (4.0, 10.0, 1.0, 1.0) \
    ) AS t(x, y, w, sigma)",
  );

  for (model_command, target) in [
    ("regress y x, wls(w)", "wls_fit"),
    ("regress y x, gls(sigma)", "gls_fit"),
  ] {
    let mut session = Session::new();
    fixture.load(&mut session);
    session
      .execute(parse_command(model_command).unwrap())
      .expect("weighted regression should fit");
    session
      .execute(parse_command("select x y").unwrap())
      .expect("weight columns should be removable after fitting");

    let result = session.execute(parse_command(&format!("predict {target}")).unwrap());
    assert!(result.is_ok(), "weighted prediction should execute: {result:?}");
    assert_eq!(session.active_dataset().unwrap().row_count, 5);
    assert_eq!(session.active_dataset().unwrap().columns.last().unwrap().name, target);
  }
}

#[test]
fn prediction_uses_nist_certified_longley_coefficients_as_reference() {
  // Fixture SHA-256: 4ed9507415d5453fc13e8309e425fcb63b02f322cba132b2cba068d0c9558c3b
  // Certified coefficient source: NIST/ITL Statistical Reference Datasets, Longley.
  const NIST_COEFFICIENTS: [f64; 7] = [
    -3482258.63459582,
    15.0618722713733,
    -0.0358191792925910,
    -2.02022980381683,
    -1.03322686717359,
    -0.0511041056535807,
    1829.15146461355,
  ];
  const XB_REL_TOLERANCE: f64 = 1e-9;
  const RESIDUAL_ABS_TOLERANCE: f64 = 1e-6;

  let fixture = Fixture::new_with_csv(
    "longley.csv",
    include_str!("../../tabdat-stats/fixtures/longley.csv"),
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2 x3 x4 x5 x6").unwrap())
    .expect("Longley regression should fit");
  session
    .execute(parse_command("predict yhat, xb").unwrap())
    .expect("Longley fitted values should be created");
  session
    .execute(parse_command("predict residual, residuals").unwrap())
    .expect("Longley residuals should be created");

  let result = session
    .execute(parse_command("head 16").unwrap())
    .expect("Longley predictions should be previewable");
  let ExecutionResult::Head(preview) = result else {
    panic!("head should return a preview");
  };
  assert_eq!(preview.rows.len(), 16);

  for row in &preview.rows {
    let outcome = number(&row[0]).expect("Longley outcome should be numeric");
    let predictors = (1..=6)
      .map(|index| number(&row[index]).expect("Longley predictor should be numeric"))
      .collect::<Vec<_>>();
    let expected_xb = NIST_COEFFICIENTS[0]
      + predictors
        .iter()
        .zip(&NIST_COEFFICIENTS[1..])
        .map(|(predictor, coefficient)| predictor * coefficient)
        .sum::<f64>();
    let actual_xb = number(&row[7]).expect("fitted value should be numeric");
    let actual_residual = number(&row[8]).expect("residual should be numeric");

    close_with_tolerance(actual_xb, expected_xb, 1e-6, XB_REL_TOLERANCE);
    close_with_tolerance(
      actual_residual,
      outcome - expected_xb,
      RESIDUAL_ABS_TOLERANCE,
      0.0,
    );
  }
}

#[test]
fn failed_prediction_does_not_change_dataset_or_regression_state() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (1.0, 3.0), (2.0, 5.0), (3.0, 7.0)) AS t(x, y)",
  );
  let mut session = Session::new();
  fixture.load(&mut session);
  let no_model = session.execute(parse_command("predict fitted").unwrap());
  assert!(no_model.is_err());

  session
    .execute(parse_command("regress y x").unwrap())
    .expect("regression should fit");
  let dataset_before = session.active_dataset().unwrap().clone();
  let model_before = session.last_regression().unwrap().clone();
  let collision = session.execute(parse_command("predict x").unwrap());
  assert!(collision.is_err());
  assert_eq!(session.active_dataset(), Some(&dataset_before));
  assert_eq!(session.last_regression(), Some(&model_before));

  session
    .execute(parse_command("select y").unwrap())
    .expect("predictor can be removed after fitting");
  let dataset_before = session.active_dataset().unwrap().clone();
  let model_before = session.last_regression().unwrap().clone();
  let missing_predictor = session.execute(parse_command("predict fitted").unwrap());
  assert!(missing_predictor.is_err());
  assert_eq!(session.active_dataset(), Some(&dataset_before));
  assert_eq!(session.last_regression(), Some(&model_before));
}

#[test]
fn prediction_requires_active_data_before_model_lookup() {
  let mut session = Session::new();
  let result = session.execute(parse_command("predict fitted").unwrap());
  assert!(result.is_err());
}

