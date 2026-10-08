use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::{
  Command, GenerateBinaryOperator, GenerateExpression, TestCommand, parse_command,
};
use tabdat_runtime::{ExecutionResult, Session, TestResult};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    Self::with_outcome_scale(1.0)
  }

  fn with_outcome_scale(outcome_scale: f64) -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-test-{}-{nonce}-{fixture_id}",
      std::process::id(),
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("fixture.parquet");
    let parquet_string = parquet.to_string_lossy().replace('\'', "''");
    let outcome_scale = outcome_scale.to_string();
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    connection
      .execute(
        &format!(
          "COPY (SELECT x1, x2, y * {outcome_scale} AS y, w, sigma, grp FROM (VALUES \
          (1.0, 10.0, 5.0, 1.0, 1.0, 'a'), \
          (2.0, 12.0, 6.0, 2.0, 2.0, 'a'), \
          (3.0, 15.0, 8.0, 3.0, 1.5, 'b'), \
          (4.0, 18.0, 9.0, 1.0, 1.0, 'c'), \
          (5.0, 20.0, 11.0, 4.0, 3.0, 'c'), \
          (6.0, 25.0, 12.0, 2.0, 2.0, 'd') \
        ) AS t(x1, x2, y, w, sigma, grp)) TO '{parquet_string}' (FORMAT PARQUET)"
        ),
        [],
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

fn execute_test(session: &mut Session, command: &str) -> TestResult {
  match session
    .execute(parse_command(command).expect("test command should parse"))
    .expect("test command should execute")
  {
    ExecutionResult::Test(result) => result,
    other => panic!("expected ExecutionResult::Test, got {other:?}"),
  }
}

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-12_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.16e} differs from expected {expected:.16e} by more than {tolerance:.3e}"
  );
}

fn assert_result(
  result: &TestResult,
  constraints: &[&str],
  statistic: f64,
  p_value: f64,
  df: usize,
  df_residual: usize,
) {
  assert_eq!(
    result.constraints,
    constraints
      .iter()
      .map(|constraint| (*constraint).to_owned())
      .collect::<Vec<_>>()
  );
  close(result.statistic, statistic);
  close(result.p_value, p_value);
  assert_eq!(result.df, df);
  assert_eq!(result.df_residual, Some(df_residual));
  assert!(!result.is_chi2);
}

#[test]
fn parsed_linear_hypothesis_tests_match_python_and_statsmodels_scipy() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");

  let active_before = session.active_dataset().cloned();
  let model_before = session.last_regression().cloned();

  assert_result(
    &execute_test(&mut session, "test x1 x2"),
    &["x1 = 0", "x2 = 0"],
    182.12068965516917,
    0.000738336718915947,
    2,
    3,
  );
  assert_result(
    &execute_test(&mut session, "test x1 = x2"),
    &["x1 = x2"],
    6.256742705570278,
    0.08760247783654902,
    1,
    3,
  );
  assert_result(
    &execute_test(&mut session, "test (x1 = x2) (x2 = 2)"),
    &["x1 = x2", "x2 = 2"],
    3537.89080459769,
    0.000008724571601717684,
    2,
    3,
  );

  assert_eq!(session.active_dataset().cloned(), active_before);
  assert_eq!(session.last_regression().cloned(), model_before);
}

#[test]
fn tests_use_the_fitted_covariance_for_each_supported_linear_regression_mode() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);

  for (regression, statistic, p_value, df_residual) in [
    ("regress y x1 x2", 6.256742705570278, 0.08760247783654902, 3),
    (
      "regress y x1 x2, wls(w)",
      11.770830014080122,
      0.041511830199647035,
      3,
    ),
    (
      "regress y x1 x2, gls(sigma)",
      3.6233766473472566,
      0.1531000799995986,
      3,
    ),
    (
      "regress y x1 x2, robust",
      11.073028358067353,
      0.0447987530230212,
      3,
    ),
    (
      "regress y x1 x2, cluster(grp)",
      8.789886334525322,
      0.05931620014242662,
      3,
    ),
    (
      "regress y x1 x2, noconstant",
      0.8721101941505085,
      0.40323939879340204,
      4,
    ),
  ] {
    session
      .execute(parse_command(regression).expect("regression command should parse"))
      .expect("regression should fit");
    let active_before = session.active_dataset().cloned();
    let model_before = session.last_regression().cloned();

    assert_result(
      &execute_test(&mut session, "test x1 = x2"),
      &["x1 = x2"],
      statistic,
      p_value,
      1,
      df_residual,
    );

    assert_eq!(session.active_dataset().cloned(), active_before);
    assert_eq!(session.last_regression().cloned(), model_before);
  }
}

#[test]
fn test_remains_scale_invariant_for_small_covariance_values() {
  let fixture = Fixture::with_outcome_scale(1e-8);
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2, cluster(grp)").unwrap())
    .expect("regression should fit");

  assert_result(
    &execute_test(&mut session, "test x1 = x2"),
    &["x1 = x2"],
    8.789886334525322,
    0.05931620014242662,
    1,
    3,
  );
}

#[test]
fn test_errors_match_python_and_leave_session_state_unchanged() {
  let fixture = Fixture::new();
  let mut session = Session::new();

  let missing_model = session
    .execute(parse_command("test x1").unwrap())
    .unwrap_err();
  assert_eq!(
    missing_model.to_string(),
    "no active estimation results found"
  );

  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");
  let active_before = session.active_dataset().cloned();
  let model_before = session.last_regression().cloned();

  for (command, expected) in [
    (
      "test missing",
      "variable 'missing' not found in active model coefficients",
    ),
    ("test x1 x1", "constraints are collinear or singular"),
  ] {
    let error = session
      .execute(parse_command(command).unwrap())
      .unwrap_err();
    assert_eq!(error.to_string(), expected);
    assert_eq!(session.active_dataset().cloned(), active_before);
    assert_eq!(session.last_regression().cloned(), model_before);
  }

  let nonlinear = Command::Test {
    command: TestCommand {
      constraints: vec![GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
        operator: GenerateBinaryOperator::Multiply,
        right: Box::new(GenerateExpression::Identifier("x2".to_owned())),
      }],
    },
  };
  assert_eq!(
    session.execute(nonlinear).unwrap_err().to_string(),
    "nonlinear coefficient multiplication is not supported in linear testing"
  );

  let division_by_zero = Command::Test {
    command: TestCommand {
      constraints: vec![GenerateExpression::Binary {
        left: Box::new(GenerateExpression::Identifier("x1".to_owned())),
        operator: GenerateBinaryOperator::Divide,
        right: Box::new(GenerateExpression::Number("0".to_owned())),
      }],
    },
  };
  assert_eq!(
    session.execute(division_by_zero).unwrap_err().to_string(),
    "division by zero in expression"
  );
  assert_eq!(session.active_dataset().cloned(), active_before);
  assert_eq!(session.last_regression().cloned(), model_before);
}
