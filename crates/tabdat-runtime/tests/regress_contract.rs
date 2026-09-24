use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new_with_sql(sql: &str) -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-regress-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("regression.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!("COPY ({sql}) TO ? (FORMAT PARQUET)"),
        [&parquet_string],
      )
      .expect("fixture table should write to parquet");
    Self { root, parquet }
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

#[test]
fn regress_requires_active_dataset() {
  let mut session = Session::new();
  let command = parse_command("regress cost age bmi").expect("valid syntax");
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "regress" }
  );
}

#[test]
fn regress_reports_unknown_variables() {
  let fixture = Fixture::new_with_sql("SELECT * FROM (VALUES (10.0, 1.0), (20.0, 2.0)) AS t(y, x)");
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let err = session
    .execute(parse_command("regress y missing_x missing_z").unwrap())
    .unwrap_err();

  assert_eq!(
    err,
    RuntimeError::RegressUnknownVariable {
      variables: vec!["missing_x".into(), "missing_z".into()]
    }
  );
}

#[test]
fn regress_requires_numeric_variables() {
  let fixture =
    Fixture::new_with_sql("SELECT * FROM (VALUES (10.0, 'text_var', 1.0)) AS t(y, text_col, x)");
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let err = session
    .execute(parse_command("regress y text_col x").unwrap())
    .unwrap_err();

  assert_eq!(
    err,
    RuntimeError::RegressRequiresNumeric {
      variables: vec!["text_col".into()]
    }
  );
}

#[test]
fn regress_rejects_empty_or_all_missing_sample() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (CAST(NULL AS DOUBLE), 1.0), (20.0, CAST(NULL AS DOUBLE))) AS t(y, x)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let err = session
    .execute(parse_command("regress y x").unwrap())
    .unwrap_err();

  assert_eq!(err, RuntimeError::RegressNoObservations);
}

#[test]
fn regress_rejects_non_positive_weights_for_wls() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (10.0, 1.0, 0.0), (20.0, 2.0, 1.0)) AS t(y, x, w)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let err = session
    .execute(parse_command("regress y x, wls(w)").unwrap())
    .unwrap_err();

  assert_eq!(err, RuntimeError::RegressRequiresPositiveWeights);
}

#[test]
fn regress_rejects_non_positive_sigma_for_gls() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (10.0, 1.0, -0.5), (20.0, 2.0, 1.0)) AS t(y, x, s)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let err = session
    .execute(parse_command("regress y x, gls(s)").unwrap())
    .unwrap_err();

  assert_eq!(err, RuntimeError::RegressRequiresPositiveSigma);
}

#[test]
fn regress_classical_ols_fits_and_stores_session_state() {
  // 5 rows: y = 2 + 3*x1 - 1*x2
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (12.0, 4.0, 2.0), \
      (8.0, 3.0, 3.0), \
      (5.0, 2.0, 3.0), \
      (15.0, 5.0, 2.0), \
      (10.0, 4.0, 4.0) \
    ) AS t(y, x1, x2)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("regress y x1 x2").unwrap())
    .unwrap();

  let ExecutionResult::Regression(reg) = result else {
    panic!("expected ExecutionResult::Regression");
  };

  assert_eq!(reg.outcome, "y");
  assert_eq!(reg.predictors, vec!["x1", "x2"]);
  assert_eq!(reg.estimator, "ols");
  assert_eq!(reg.covariance, "nonrobust");
  assert_eq!(reg.observation_count, 5);
  assert!(reg.include_intercept);

  // Check coefficients
  assert_eq!(reg.coefficients.len(), 3);
  assert_eq!(reg.coefficients[0].name, "intercept");
  assert_eq!(reg.coefficients[1].name, "x1");
  assert_eq!(reg.coefficients[2].name, "x2");

  assert!((reg.coefficients[0].value - 2.0).abs() < 1e-9);
  assert!((reg.coefficients[1].value - 3.0).abs() < 1e-9);
  assert!((reg.coefficients[2].value - (-1.0)).abs() < 1e-9);

  // Standard errors should be near zero because it is a perfect linear fit
  assert!(reg.coefficients[0].standard_error.unwrap() < 1e-6);
  assert!(reg.coefficients[1].standard_error.unwrap() < 1e-6);
  assert!(reg.coefficients[2].standard_error.unwrap() < 1e-6);

  assert!((reg.r_squared.unwrap() - 1.0).abs() < 1e-9);

  // Verify session state was populated
  assert!(session.last_regression().is_some());
  let last = session.last_regression().unwrap();
  assert_eq!(last.fit_statistics.observation_count, 5);
}

#[test]
fn regress_robust_hc1_covariance() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (12.0, 4.0), \
      (8.0, 3.0), \
      (5.5, 2.0), \
      (15.2, 5.0), \
      (9.8, 4.0), \
      (4.2, 1.0) \
    ) AS t(y, x1)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("regress y x1, robust").unwrap())
    .unwrap();

  let ExecutionResult::Regression(reg) = result else {
    panic!("expected Regression");
  };

  assert_eq!(reg.covariance, "robust");
  assert_eq!(reg.observation_count, 6);
  assert!(reg.coefficients[0].standard_error.unwrap() > 0.0);
  assert!(reg.coefficients[1].standard_error.unwrap() > 0.0);
}

#[test]
fn regress_clustered_covariance() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (12.0, 4.0, 'grpA'), \
      (8.0, 3.0, 'grpA'), \
      (5.5, 2.0, 'grpA'), \
      (15.2, 5.0, 'grpB'), \
      (9.8, 4.0, 'grpB'), \
      (4.2, 1.0, 'grpB') \
    ) AS t(y, x1, cluster_id)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("regress y x1, cluster(cluster_id)").unwrap())
    .unwrap();

  let ExecutionResult::Regression(reg) = result else {
    panic!("expected Regression");
  };

  assert_eq!(reg.covariance, "cluster(cluster_id)");
  assert_eq!(reg.observation_count, 6);
  assert!(reg.coefficients[0].standard_error.unwrap() > 0.0);
  assert!(reg.coefficients[1].standard_error.unwrap() > 0.0);
}

#[test]
fn regress_wls_and_gls_modes() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (12.0, 4.0, 1.2, 0.8), \
      (8.0, 3.0, 0.9, 1.1), \
      (5.5, 2.0, 1.0, 1.0), \
      (15.2, 5.0, 1.5, 0.6) \
    ) AS t(y, x1, w, s)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  // WLS
  let res_wls = session
    .execute(parse_command("regress y x1, wls(w)").unwrap())
    .unwrap();
  let ExecutionResult::Regression(reg_wls) = res_wls else {
    panic!("expected Regression");
  };
  assert_eq!(reg_wls.estimator, "wls");

  // GLS
  let res_gls = session
    .execute(parse_command("regress y x1, gls(s)").unwrap())
    .unwrap();
  let ExecutionResult::Regression(reg_gls) = res_gls else {
    panic!("expected Regression");
  };
  assert_eq!(reg_gls.estimator, "gls");
}

#[test]
fn regress_noconstant_suppresses_intercept() {
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES (10.0, 2.0), (20.0, 4.0), (30.0, 6.0)) AS t(y, x)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("regress y x, noconstant").unwrap())
    .unwrap();

  let ExecutionResult::Regression(reg) = result else {
    panic!("expected Regression");
  };

  assert!(!reg.include_intercept);
  assert_eq!(reg.coefficients.len(), 1);
  assert_eq!(reg.coefficients[0].name, "x");
  assert!((reg.coefficients[0].value - 5.0).abs() < 1e-9);
}

#[test]
fn regress_sample_tracking_with_missing_rows() {
  // 6 rows total, rows index 1 and 4 have nulls -> 4 retained, 2 dropped
  let fixture = Fixture::new_with_sql(
    "SELECT * FROM (VALUES \
      (10.0, 2.0), \
      (CAST(NULL AS DOUBLE), 3.0), \
      (30.0, 6.0), \
      (40.0, 8.0), \
      (50.0, CAST(NULL AS DOUBLE)), \
      (60.0, 12.0) \
    ) AS t(y, x)",
  );
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();

  let result = session
    .execute(parse_command("regress y x").unwrap())
    .unwrap();

  let ExecutionResult::Regression(reg) = result else {
    panic!("expected Regression");
  };

  assert_eq!(reg.observation_count, 4);

  let sample = reg.least_squares.sample.expect("sample should be tracked");
  assert_eq!(sample.total_observations, 6);
  assert_eq!(sample.dropped_observations, 2);
  assert_eq!(sample.retained_indices, vec![0, 2, 3, 5]);
}
