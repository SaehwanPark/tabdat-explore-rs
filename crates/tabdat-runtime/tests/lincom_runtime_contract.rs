use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::parse_command;
use tabdat_runtime::{ExecutionResult, LincomResult, Session};

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
          (1.0, 10.0, 5.0, 1.0, 1.0, 'a'), \
          (2.0, 12.0, 6.0, 2.0, 2.0, 'a'), \
          (3.0, 15.0, 8.0, 3.0, 1.5, 'b'), \
          (4.0, 18.0, 9.0, 1.0, 1.0, 'c'), \
          (5.0, 20.0, 11.0, 4.0, 3.0, 'c'), \
          (6.0, 25.0, 12.0, 2.0, 2.0, 'd') \
        ) AS t(x1, x2, y, w, sigma, grp)) TO ? (FORMAT PARQUET)",
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

fn execute_lincom(session: &mut Session, command: &str) -> LincomResult {
  match session
    .execute(parse_command(command).expect("lincom command should parse"))
    .expect("lincom should execute")
  {
    ExecutionResult::Lincom(result) => result,
    other => panic!("expected ExecutionResult::Lincom, got {other:?}"),
  }
}

fn close(actual: f64, expected: f64) {
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.16e} differs from expected {expected:.16e} by more than {tolerance:.3e}"
  );
}

fn assert_result(result: &LincomResult, label: &str, values: [f64; 6], degrees_of_freedom: usize) {
  assert_eq!(result.label, label);
  for (actual, expected) in [
    result.estimate,
    result.standard_error,
    result.statistic,
    result.p_value,
    result.ci_lower,
    result.ci_upper,
  ]
  .into_iter()
  .zip(values)
  {
    close(actual, expected);
  }
  assert_eq!(result.ci_level, 95.0);
  assert_eq!(result.df_residual, Some(degrees_of_freedom));
}

#[test]
fn lincom_matches_python_and_statsmodels_scipy_for_affine_expressions() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");

  let active_before = session.active_dataset().cloned();
  let model_before = session.last_regression().cloned();
  assert_result(
    &execute_lincom(&mut session, "lincom x1 - x2"),
    "(x1 - x2)",
    [
      1.9119718309859137,
      0.7643765262708596,
      2.501348177597489,
      0.08760247783654902,
      -0.5206154208903784,
      4.344559082862206,
    ],
    3,
  );
  assert_eq!(session.active_dataset(), active_before.as_ref());
  assert_eq!(session.last_regression(), model_before.as_ref());

  assert_result(
    &execute_lincom(&mut session, "lincom x1 + 2 * x2"),
    "(x1 + (2 * x2))",
    [
      1.5633802816901397,
      0.19337366503867606,
      8.084763152094432,
      0.003954139640165558,
      0.9479789758486358,
      2.1787815875316436,
    ],
    3,
  );
  assert_result(
    &execute_lincom(&mut session, "lincom intercept + x1"),
    "(intercept + x1)",
    [
      5.947183098591548,
      1.8373433779115673,
      3.2368381273137232,
      0.04796594223283117,
      0.09993645401939322,
      11.794429743163704,
    ],
    3,
  );
  assert_result(
    &execute_lincom(&mut session, "lincom 2 + x1"),
    "(2 + x1)",
    [
      3.795774647887322,
      0.57137809026885,
      6.6431925069113165,
      0.006950259688841776,
      1.9773945555911596,
      5.614154740183485,
    ],
    3,
  );
  assert_result(
    &execute_lincom(&mut session, "lincom x1 / 2"),
    "(x1 / 2)",
    [
      0.8978873239436612,
      0.285689045134425,
      3.1428832824905104,
      0.05154832697519063,
      -0.01130272220442008,
      1.8070773700917426,
    ],
    3,
  );
  assert_result(
    &execute_lincom(&mut session, "lincom -x1"),
    "-(x1)",
    [
      -1.7957746478873224,
      0.57137809026885,
      -3.1428832824905104,
      0.05154832697519063,
      -3.6141547401834853,
      0.02260544440884016,
    ],
    3,
  );

  let constant = execute_lincom(&mut session, "lincom 2.0");
  assert_eq!(constant.label, "2.0");
  assert_eq!(constant.estimate, 2.0);
  assert_eq!(constant.standard_error, 0.0);
  assert!(constant.statistic.is_nan());
  assert!(constant.p_value.is_nan());
  assert_eq!(constant.ci_lower, 2.0);
  assert_eq!(constant.ci_upper, 2.0);
  assert_eq!(constant.ci_level, 95.0);
  assert_eq!(constant.df_residual, Some(3));
  assert_eq!(
    execute_lincom(&mut session, "lincom 0.00001").label,
    "1e-05"
  );
  assert_eq!(
    execute_lincom(&mut session, "lincom 10000000000000000.0").label,
    "1e+16"
  );
}

#[test]
fn lincom_uses_the_fitted_covariance_and_supports_no_intercept_models() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);

  let cases = [
    (
      "regress y x1 x2, wls(w)",
      "lincom x1 - x2",
      "(x1 - x2)",
      [
        2.2141020932794717,
        0.6453481932756083,
        3.430864324638927,
        0.041511830199647104,
        0.16031611996799588,
        4.267888066590947,
      ],
      3,
    ),
    (
      "regress y x1 x2, gls(sigma)",
      "lincom x1 - x2",
      "(x1 - x2)",
      [
        1.5992544268406363,
        0.8401577174716891,
        1.9035169154350196,
        0.15310007999959874,
        -1.0745023969827336,
        4.273011250664006,
      ],
      3,
    ),
    (
      "regress y x1 x2, robust",
      "lincom x1 - x2",
      "(x1 - x2)",
      [
        1.9119718309859137,
        0.5745770609021545,
        3.327616017221241,
        0.044798753023021165,
        0.08341118641708012,
        3.7405324755547475,
      ],
      3,
    ),
    (
      "regress y x1 x2, cluster(grp)",
      "lincom x1 - x2",
      "(x1 - x2)",
      [
        1.9119718309859137,
        0.6448962625217005,
        2.9647742468062086,
        0.059316200142426556,
        -0.14037589696754393,
        3.9643195589393714,
      ],
      3,
    ),
    (
      "regress y x1 x2, noconstant",
      "lincom x1 - x2",
      "(x1 - x2)",
      [
        -0.41664869527711823,
        0.4461535414752152,
        -0.9338684030153865,
        0.403239398793402,
        -1.6553695116726401,
        0.8220721211184037,
      ],
      4,
    ),
  ];

  for (regression, command, label, expected, df) in cases {
    session
      .execute(parse_command(regression).unwrap())
      .unwrap_or_else(|error| panic!("{regression} should fit: {error}"));
    assert_result(&execute_lincom(&mut session, command), label, expected, df);
  }
}

#[test]
fn lincom_preserves_the_fitted_state_and_rejects_invalid_forms_with_oracle_errors() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  fixture.load(&mut session);
  session
    .execute(parse_command("regress y x1 x2").unwrap())
    .expect("regression should fit");
  session
    .execute(parse_command("keep x1").unwrap())
    .expect("active relation projection should succeed");

  let active_before = session.active_dataset().cloned();
  let model_before = session.last_regression().cloned();
  assert_result(
    &execute_lincom(&mut session, "lincom x1 - x2"),
    "(x1 - x2)",
    [
      1.9119718309859137,
      0.7643765262708596,
      2.501348177597489,
      0.08760247783654902,
      -0.5206154208903784,
      4.344559082862206,
    ],
    3,
  );

  for (command, expected) in [
    (
      "lincom x3",
      "variable 'x3' not found in active model coefficients",
    ),
    (
      "lincom x1 * x2",
      "nonlinear coefficient multiplication is not supported in linear testing",
    ),
    (
      "lincom x1 / x2",
      "nonlinear coefficient division is not supported in linear testing",
    ),
    ("lincom x1 / 0", "division by zero in expression"),
    (
      "lincom abs(x1)",
      "functions are not supported in linear testing",
    ),
    (
      "lincom null",
      "unsupported expression type in linear testing",
    ),
    (
      "lincom \"text\"",
      "unsupported expression type in linear testing",
    ),
    ("lincom x1 == 0", "unsupported binary operator: =="),
  ] {
    let error = session
      .execute(parse_command(command).unwrap())
      .unwrap_err();
    assert_eq!(error.to_string(), expected, "{command}");
    assert_eq!(session.active_dataset(), active_before.as_ref());
    assert_eq!(session.last_regression(), model_before.as_ref());
  }
}
