use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tabdat_domain::{Command, EstimationFamily, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

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
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!("COPY ({sql}) TO ? (FORMAT PARQUET)"),
        [&parquet_string],
      )
      .expect("fixture table should write to parquet");
    Self { root, parquet }
  }

  fn six_instrument_sample() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let csv = root.join("six-instrument.csv");
    fs::write(
      &csv,
      include_str!("../../../examples/robust-ar/robust_ar_six_ci_multiple_data.csv"),
    )
    .expect("six-instrument fixture should be written");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv.to_string_lossy().replace('\'', "''");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{csv_string}', header = true)) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("six-instrument fixture should write to parquet");
    Self { root, parquet }
  }

  fn seven_instrument_sample() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let csv = root.join("seven-instrument.csv");
    fs::write(
      &csv,
      include_str!("../../../examples/robust-ar/robust_ar_seven_ci_multiple_data.csv"),
    )
    .expect("seven-instrument fixture should be written");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv.to_string_lossy().replace('\'', "''");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{csv_string}', header = true)) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("seven-instrument fixture should write to parquet");
    Self { root, parquet }
  }

  fn eight_instrument_sample() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let csv = root.join("eight-instrument.csv");
    fs::write(
      &csv,
      include_str!("../../../examples/robust-ar/robust_ar_eight_ci_multiple_data.csv"),
    )
    .expect("eight-instrument fixture should be written");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv.to_string_lossy().replace('\'', "''");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{csv_string}', header = true)) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("eight-instrument fixture should write to parquet");
    Self { root, parquet }
  }

  fn nine_instrument_sample() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let csv = root.join("nine-instrument.csv");
    fs::write(
      &csv,
      include_str!("../../../examples/robust-ar/robust_ar_nine_ci_multiple_data.csv"),
    )
    .expect("nine-instrument fixture should be written");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv.to_string_lossy().replace('\'', "''");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{csv_string}', header = true)) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("nine-instrument fixture should write to parquet");
    Self { root, parquet }
  }

  fn fourteen_instrument_sample() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-iv-{}-{}-{}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let csv = root.join("fourteen-instrument.csv");
    fs::write(
      &csv,
      include_str!("../../../examples/robust-ar/robust_ar_fourteen_ci_multiple_data.csv"),
    )
    .expect("fourteen-instrument fixture should be written");
    let parquet = root.join("iv.parquet");
    let connection = duckdb::Connection::open_in_memory().expect("fixture connection should open");
    let csv_string = csv.to_string_lossy().replace('\'', "''");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        &format!(
          "COPY (SELECT * FROM read_csv_auto('{csv_string}', header = true)) TO ? (FORMAT PARQUET)"
        ),
        [&parquet_string],
      )
      .expect("fourteen-instrument fixture should write to parquet");
    Self { root, parquet }
  }

  fn school_like_sample() -> Self {
    Self::new_with_sql(
      "WITH base AS (\
        SELECT i::DOUBLE AS i, sin(i / 7.0) + i / 90.0 AS w, \
          cos(i / 5.0) + i / 70.0 AS z1, \
          sin(i / 11.0) - cos(i / 13.0) + i / 200.0 AS z2, \
          cos(i / 8.0) + sin(i / 17.0) + i / 130.0 AS z3, \
          sin(i / 19.0) - cos(i / 21.0) + i / 250.0 AS z4, \
          cos(i / 23.0) + sin(i / 31.0) + i / 180.0 AS z5, \
          sin(i / 29.0) + cos(i / 37.0) + i / 210.0 AS z6, \
          sin(i / 3.0) + cos(i / 9.0) AS v1, \
          cos(i / 4.0) - sin(i / 6.0) AS v2 \
        FROM range(1, 61) AS t(i)\
      ), design AS (\
        SELECT *, 0.3 + 0.4 * w + 0.9 * z1 + 0.2 * z2 + v1 AS x1, \
          -0.2 + 0.2 * w - 0.3 * z1 + 1.1 * z2 + v2 AS x2 \
        FROM base\
      ), outcome AS (\
        SELECT *, 1.2 + 0.5 * w + 1.7 * x1 - 0.6 * x2 + sin(i / 1.7) \
          + cos(i / 2.3) + sin(i / 5.9) AS y \
        FROM design\
      ) SELECT CASE WHEN i = 1 THEN NULL ELSE y END AS y, w, \
          CASE WHEN i = 2 THEN NULL ELSE x1 END AS x1, x2, z1, z2, z3, z4, z5, \
          CASE WHEN i = 3 THEN NULL ELSE CAST(CAST(i AS INTEGER) % 12 AS VARCHAR) END AS group_id, z6 \
        FROM outcome",
    )
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn load(fixture: &Fixture) -> Session {
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use {}", fixture.parquet.display())).unwrap())
    .unwrap();
  session
}

#[test]
fn clustered_gmm_preserves_owned_state_predictions_and_failure_contracts() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let command =
    parse_command("ivregress gmm y, endog(x1) iv(z1 z2) noconstant cluster(group_id)").unwrap();
  let ExecutionResult::ClusterGmm(fit) = session.execute(command).unwrap() else {
    panic!("expected clustered GMM result");
  };
  assert_eq!(fit.result.cluster_count, 12);
  assert_eq!(fit.sample.total_observations, 60);
  assert_eq!(fit.sample.dropped_observations, 3);
  assert_eq!(fit.result.parameter_names, ["x1"]);
  assert!(fit.result.fixed_j.unwrap().is_finite());
  assert!(matches!(
    session.state().estimation,
    Some(tabdat_domain::EstimationState::ClusterGmm(_))
  ));
  for command in ["predict fitted, xb", "predict residual, residuals"] {
    let ExecutionResult::Prediction(result) =
      session.execute(parse_command(command).unwrap()).unwrap()
    else {
      panic!("expected prediction")
    };
    assert_eq!(result.dataset.row_count, 60);
  }
  assert!(session.execute(parse_command("test x1").unwrap()).is_err());
  assert!(
    session
      .execute(parse_command("estat overid").unwrap())
      .is_err()
  );
  assert!(matches!(
    session.state().estimation,
    Some(tabdat_domain::EstimationState::ClusterGmm(_))
  ));
  let error = session
    .execute(
      parse_command("ivregress gmm y w, endog(x1) iv(z1 z2) noconstant cluster(group_id)").unwrap(),
    )
    .unwrap_err();
  assert!(error.to_string().contains("no included exogenous"));
  assert!(session.state().estimation.is_none());
  assert!(
    session
      .execute(parse_command("predict stale, xb").unwrap())
      .is_err()
  );
  session
    .execute(
      parse_command("ivregress gmm y, endog(x1) iv(z1 z2) noconstant cluster(group_id)").unwrap(),
    )
    .unwrap();
  assert!(
    session
      .execute(
        parse_command("ivregress gmm y, endog(x1) iv(z1 z1) noconstant cluster(group_id)").unwrap()
      )
      .is_err()
  );
  assert!(session.state().estimation.is_none());
  assert!(
    session
      .execute(parse_command("predict stale, xb").unwrap())
      .is_err()
  );
}

#[test]
fn clustered_gmm_instrument_limit_failure_clears_previous_fit() {
  let columns = (1..=73)
    .map(|i| format!("sin(n / {}.0) + n / {}.0 AS z{i}", i + 2, i + 100))
    .collect::<Vec<_>>()
    .join(", ");
  let sql = format!(
    "SELECT n AS y, n + sin(n) AS x, (n % 20)::VARCHAR AS grp, {columns} FROM range(1,101) t(n)"
  );
  let fixture = Fixture::new_with_sql(&sql);
  let mut session = load(&fixture);
  session
    .execute(parse_command("ivregress gmm y, endog(x) iv(z1 z2) noconstant cluster(grp)").unwrap())
    .unwrap();
  let instruments = (1..=73)
    .map(|i| format!("z{i}"))
    .collect::<Vec<_>>()
    .join(" ");
  let command = format!("ivregress gmm y, endog(x) iv({instruments}) noconstant cluster(grp)");
  let error = session
    .execute(parse_command(&command).unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("72 instruments"), "{error}");
  assert!(session.state().estimation.is_none());
  assert!(
    session
      .execute(parse_command("predict stale, xb").unwrap())
      .is_err()
  );
}

#[test]
fn clustered_gmm_jointly_excludes_nonfinite_numeric_cluster_keys() {
  let fixture = Fixture::new_with_sql(
    "WITH base AS (SELECT i, sin(i / 7.0) + i / 90.0 AS z1, \
       cos(i / 5.0) + i / 70.0 AS z2, sin(i / 3.0) AS noise FROM range(1,61) t(i)) \
     SELECT z1 + z2 + noise AS x, 2 * (z1 + z2 + noise) + cos(i / 4.0) AS y, z1, z2, \
       CASE WHEN i=1 THEN 'NaN'::DOUBLE WHEN i=2 THEN 'Infinity'::DOUBLE \
         WHEN i=3 THEN '-0.0'::DOUBLE WHEN i=4 THEN 0.0 ELSE (i % 12)::DOUBLE END AS grp FROM base",
  );
  let mut session = load(&fixture);
  let ExecutionResult::ClusterGmm(fit) = session
    .execute(parse_command("ivregress gmm y, endog(x) iv(z1 z2) noconstant cluster(grp)").unwrap())
    .unwrap()
  else {
    panic!("expected clustered GMM result")
  };
  assert_eq!(fit.sample.dropped_observations, 2);
  assert_eq!(fit.sample.retained_indices.first(), Some(&2));
  assert_eq!(fit.sample.cluster_groups.as_ref().unwrap()[..2], ["0", "0"]);
  assert_eq!(fit.result.cluster_count, 12);
}

#[test]
fn ivregress_2sls_fits_multiple_endogenous_regressors_and_reports_sample() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let command =
    parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2) cluster(group_id)").unwrap();
  let ExecutionResult::Regression(result) = session.execute(command).unwrap() else {
    panic!("expected a regression result");
  };
  assert_eq!(result.estimator, "2sls");
  assert_eq!(result.covariance, "cluster(group_id)");
  assert_eq!(result.predictors, ["w", "x1", "x2"]);
  assert_eq!(result.observation_count, 57);
  assert_eq!(
    result
      .least_squares
      .sample
      .as_ref()
      .unwrap()
      .total_observations,
    60
  );
  assert_eq!(
    result
      .least_squares
      .sample
      .as_ref()
      .unwrap()
      .dropped_observations,
    3
  );
  assert_eq!(result.first_stage.as_ref().unwrap().len(), 2);
  assert_eq!(
    result
      .first_stage
      .as_ref()
      .unwrap()
      .iter()
      .map(|diagnostic| diagnostic.endogenous.as_str())
      .collect::<Vec<_>>(),
    ["x1", "x2"]
  );
  assert_eq!(
    session.state().estimation.unwrap().family(),
    EstimationFamily::InstrumentalVariables
  );

  let prediction = session
    .execute(parse_command("predict fitted, xb").unwrap())
    .unwrap();
  let ExecutionResult::Prediction(prediction) = prediction else {
    panic!("expected a prediction result");
  };
  assert_eq!(prediction.dataset.row_count, 60);
  assert!(
    prediction
      .dataset
      .columns
      .iter()
      .any(|column| column.name == "fitted")
  );
  let linear_combination = session
    .execute(parse_command("lincom x1").unwrap())
    .unwrap();
  let ExecutionResult::Lincom(linear_combination) = linear_combination else {
    panic!("expected a linear-combination result");
  };
  let coefficient = result
    .coefficients
    .iter()
    .find(|coefficient| coefficient.name == "x1")
    .unwrap();
  assert!((linear_combination.estimate - coefficient.value).abs() < 1e-12);
}

#[test]
fn anderson_rubin_uses_retained_sample_and_preserves_state_on_success_and_failure() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let ar = parse_command("estat ar, null(1.7 -0.6)").unwrap();
  assert_eq!(
    session.execute(ar.clone()).unwrap_err(),
    RuntimeError::EstatArRequiresPriorIv
  );
  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  assert_eq!(
    session.execute(ar.clone()).unwrap_err(),
    RuntimeError::EstatArRequiresPriorIv
  );
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap();
  let before = session.state();
  let ExecutionResult::AndersonRubin(result) = session.execute(ar.clone()).unwrap() else {
    panic!("expected AR test");
  };
  assert_eq!(result.observation_count, 58);
  assert_eq!(session.state(), before);
  let tabdat_domain::EstimationState::InstrumentalVariables(state) = before.estimation.unwrap()
  else {
    panic!("expected IV state");
  };
  assert_eq!(
    state.problem.sample.as_ref().unwrap().retained_indices,
    (2..60).collect::<Vec<_>>()
  );
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(
    session.execute(ar.clone()).unwrap(),
    ExecutionResult::AndersonRubin(result)
  );
  assert_eq!(session.state(), changed);
  assert!(
    session
      .execute(parse_command("estat ar, null(0)").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), changed);
  for option in ["robust", "cluster(group_id)"] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1 x2) iv(z1 z2) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let test = session.execute(ar.clone()).unwrap();
    let ExecutionResult::RobustAndersonRubin(result) = &test else {
      panic!("robust AR");
    };
    assert_eq!(
      result.observation_count,
      if option == "robust" { 58 } else { 57 }
    );
    let tabdat_domain::EstimationState::InstrumentalVariables(state) =
      before.estimation.as_ref().unwrap()
    else {
      panic!("IV state");
    };
    let expected = (2..60)
      .filter(|i| option == "robust" || *i != 2)
      .collect::<Vec<_>>();
    assert_eq!(
      state.problem.sample.as_ref().unwrap().retained_indices,
      expected
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(ar.clone()).unwrap(), test);
    assert!(
      session
        .execute(parse_command("estat ar, null(0)").unwrap())
        .is_err()
    );
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn wu_hausman_reuses_retained_sample_and_supports_joint_just_identified_models() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let command = parse_command("estat endogenous").unwrap();
  assert_eq!(
    session.execute(command.clone()).unwrap_err(),
    RuntimeError::EstatEndogenousRequiresPriorIv
  );
  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  let before = session.state();
  assert_eq!(
    session.execute(command.clone()).unwrap_err(),
    RuntimeError::EstatEndogenousRequiresPriorIv
  );
  assert_eq!(session.state(), before);
  let ExecutionResult::Regression(fit) = session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap()
  else {
    panic!("fit");
  };
  assert_eq!(
    fit.least_squares.sample.as_ref().unwrap().retained_indices,
    (2..60).collect::<Vec<_>>()
  );
  let before = session.state();
  let result = session.execute(command.clone()).unwrap();
  let ExecutionResult::WuHausman(test) = &result else {
    panic!("Wu-Hausman result");
  };
  assert_eq!(test.endogenous, ["x1", "x2"]);
  assert_eq!(test.observation_count, 58);
  assert_eq!(test.numerator_df, 2);
  assert_eq!(test.denominator_df, 52);
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(session.execute(command.clone()).unwrap(), result);
  assert_eq!(session.state(), changed);
  for mode in ["robust", "cluster(group_id)"] {
    let mut session = load(&fixture);
    let ExecutionResult::Regression(fit) = session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1 x2) iv(z1 z2) {mode}"
        ))
        .unwrap(),
      )
      .unwrap()
    else {
      panic!("fit");
    };
    let first_retained = if mode.starts_with("cluster") { 3 } else { 2 };
    assert_eq!(
      fit.least_squares.sample.as_ref().unwrap().retained_indices,
      (first_retained..60).collect::<Vec<_>>()
    );
    let before = session.state();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustEndogeneity(test) = &result else {
      panic!("robust endogeneity result");
    };
    assert_eq!(test.endogenous, ["x1", "x2"]);
    // The third candidate has a missing group ID and is excluded only by CR1.
    let expected_n = if mode.starts_with("cluster") { 57 } else { 58 };
    assert_eq!(test.observation_count, expected_n);
    assert_eq!(test.auxiliary_parameter_count, 6);
    assert_eq!(test.auxiliary_residual_df, expected_n - 6);
    assert_eq!(test.degrees_of_freedom, 2);
    assert_eq!(
      test.cluster_group_count.is_some(),
      mode.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command.clone()).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn robust_endogeneity_failure_keeps_the_retained_iv_fit_and_dataset() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  session
    .execute(
      parse_command(
        r#"sql """SELECT *, CASE WHEN w > 0 THEN 1 ELSE 0 END AS two FROM active""" into two_groups"#,
      )
      .unwrap(),
    )
    .unwrap();
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2) cluster(two)").unwrap())
    .unwrap();
  let before = session.state();
  let error = session
    .execute(parse_command("estat endogenous").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("Wald"), "{error}");
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("predict xb_after_failure, xb").unwrap())
    .unwrap();
}

#[test]
fn sargan_uses_retained_structural_residuals_and_preserves_success_and_failure_state() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let overid = parse_command("estat overid").unwrap();
  assert_eq!(
    session.execute(overid.clone()).unwrap_err(),
    RuntimeError::EstatOveridRequiresPriorIv
  );
  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  let before = session.state();
  assert_eq!(
    session.execute(overid.clone()).unwrap_err(),
    RuntimeError::EstatOveridRequiresPriorIv
  );
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap();
  let before = session.state();
  assert!(
    session
      .execute(overid.clone())
      .unwrap_err()
      .to_string()
      .contains("requires overidentification")
  );
  assert_eq!(session.state(), before);
  let ExecutionResult::Regression(fit) = session
    .execute(parse_command("ivregress 2sls y w, endog(x1) iv(z1 z2)").unwrap())
    .unwrap()
  else {
    panic!("fit");
  };
  assert_eq!(
    fit.least_squares.sample.as_ref().unwrap().retained_indices,
    (2..60).collect::<Vec<_>>()
  );
  let before = session.state();
  let result = session.execute(overid.clone()).unwrap();
  let ExecutionResult::Sargan(test) = &result else {
    panic!("Sargan result");
  };
  assert_eq!(test.observation_count, 58);
  assert_eq!(test.degrees_of_freedom, 1);
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(session.execute(overid.clone()).unwrap(), result);
  assert_eq!(session.state(), changed);
  let mut session = load(&fixture);
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1) iv(z1 z2) cluster(group_id)").unwrap())
    .unwrap();
  let before = session.state();
  let result = session.execute(overid.clone()).unwrap();
  let ExecutionResult::RobustOveridScore(test) = &result else {
    panic!("clustered overidentification score result");
  };
  assert_eq!(test.observation_count, 57);
  assert_eq!(test.degrees_of_freedom, 1);
  assert_eq!(test.cluster_variable.as_deref(), Some("group_id"));
  assert_eq!(test.cluster_group_count, Some(12));
  assert_eq!(test.covariance_correction, 1.0);
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  session
    .execute(parse_command("drop group_id").unwrap())
    .unwrap();
  let changed = session.state();
  assert_eq!(session.execute(overid).unwrap(), result);
  assert_eq!(session.state(), changed);
}

#[test]
fn robust_overid_score_reuses_exact_fit_sample_and_failure_keeps_iv_state() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let overid = parse_command("estat overid").unwrap();
  let ExecutionResult::Regression(fit) = session
    .execute(parse_command("ivregress 2sls y w, endog(x1) iv(z1 z2) robust").unwrap())
    .unwrap()
  else {
    panic!("fit");
  };
  assert_eq!(
    fit.least_squares.sample.as_ref().unwrap().retained_indices,
    (2..60).collect::<Vec<_>>()
  );
  let before = session.state();
  let result = session.execute(overid.clone()).unwrap();
  let ExecutionResult::RobustOveridScore(test) = &result else {
    panic!("score");
  };
  assert_eq!(test.observation_count, 58);
  assert_eq!(test.degrees_of_freedom, 1);
  assert_eq!(test.instrument_parameter_count, 4);
  assert_eq!(test.structural_parameter_count, 3);
  assert_eq!(test.covariance_correction, 1.0);
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(session.execute(overid.clone()).unwrap(), result);
  assert_eq!(session.state(), changed);

  let mut session = load(&fixture);
  session
    .execute(parse_command("replace y = 0").unwrap())
    .unwrap();
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1) iv(z1 z2) robust").unwrap())
    .unwrap();
  let before = session.state();
  assert!(
    session
      .execute(overid)
      .unwrap_err()
      .to_string()
      .contains("structural residual variance")
  );
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("predict xb_after_overid_failure, xb").unwrap())
    .unwrap();
}

#[test]
fn scalar_ar_confidence_sets_reuse_retained_sample_and_reject_endogenous_nuisance_parameters() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  let ci = parse_command("estat ar, ci").unwrap();
  assert_eq!(
    session.execute(ci.clone()).unwrap_err(),
    RuntimeError::EstatArRequiresPriorIv
  );
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap();
  let before = session.state();
  assert!(
    session
      .execute(ci.clone())
      .unwrap_err()
      .to_string()
      .contains("exactly one endogenous")
  );
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1) iv(z1 z2)").unwrap())
    .unwrap();
  let before = session.state();
  let result = session.execute(ci.clone()).unwrap();
  let ExecutionResult::AndersonRubinConfidenceSet(set) = &result else {
    panic!("expected confidence set");
  };
  assert_eq!(set.endogenous, "x1");
  assert_eq!(set.observation_count, 58);
  assert_eq!(session.state(), before);
  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(session.execute(ci.clone()).unwrap(), result);
  assert_eq!(session.state(), changed);
  for (option, expected_n) in [("robust", 58), ("cluster(group_id)", 57)] {
    let mut session = load(&fixture);
    session
      .execute(parse_command(&format!("ivregress 2sls y w, endog(x1) iv(z1) {option}")).unwrap())
      .unwrap();
    let before = session.state();
    let result = session.execute(ci.clone()).unwrap();
    let ExecutionResult::RobustAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust AR confidence set");
    };
    assert_eq!(set.endogenous, "x1");
    assert_eq!(set.excluded_instrument, "z1");
    assert_eq!(set.observation_count, expected_n);
    assert_eq!(set.auxiliary_parameter_count, 3);
    assert_eq!(set.auxiliary_residual_df, expected_n - 3);
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(ci.clone()).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
  for (option, expected_n) in [("robust", 58), ("cluster(group_id)", 57)] {
    let mut session = load(&fixture);
    session
      .execute(parse_command(&format!("ivregress 2sls y w, endog(x1) iv(z1 z2) {option}")).unwrap())
      .unwrap();
    let before = session.state();
    let result = session.execute(ci.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust two-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x1");
    assert_eq!(set.excluded_instruments, ["z1", "z2"]);
    assert_eq!(set.observation_count, expected_n);
    assert_eq!(set.auxiliary_parameter_count, 4);
    assert_eq!(set.auxiliary_residual_df, expected_n - 4);
    assert_eq!(set.degrees_of_freedom, 2);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(ci.clone()).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn robust_ar_sets_reuse_retained_fit_and_cap_fourteen_dimensions() {
  let fixture = Fixture::school_like_sample();
  for (option, expected_n) in [("robust", 58), ("cluster(group_id)", 57)] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1) iv(z1 z2 z3) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust three-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x1");
    assert_eq!(set.excluded_instruments, ["z1", "z2", "z3"]);
    assert_eq!(set.degrees_of_freedom, 3);
    assert_eq!(set.observation_count, expected_n);
    assert_eq!(set.auxiliary_parameter_count, 5);
    assert_eq!(set.auxiliary_residual_df, expected_n - 5);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }

  for (option, expected_n) in [("robust", 58), ("cluster(group_id)", 57)] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1) iv(z1 z2 z3 z4) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust four-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x1");
    assert_eq!(set.excluded_instruments, ["z1", "z2", "z3", "z4"]);
    assert_eq!(set.degrees_of_freedom, 4);
    assert_eq!(set.observation_count, expected_n);
    assert_eq!(set.auxiliary_parameter_count, 6);
    assert_eq!(set.auxiliary_residual_df, expected_n - 6);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }

  for (option, expected_n) in [("robust", 58), ("cluster(group_id)", 57)] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1) iv(z1 z2 z3 z4 z5) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust five-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x1");
    assert_eq!(set.excluded_instruments, ["z1", "z2", "z3", "z4", "z5"]);
    assert_eq!(set.degrees_of_freedom, 5);
    assert_eq!(set.observation_count, expected_n);
    assert_eq!(set.auxiliary_parameter_count, 7);
    assert_eq!(set.auxiliary_residual_df, expected_n - 7);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }

  let six_fixture = Fixture::six_instrument_sample();
  for option in ["robust", "cluster(group)"] {
    let mut session = load(&six_fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust six-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      ["z1", "z2", "z3", "z4", "z5", "z6"]
    );
    assert_eq!(set.degrees_of_freedom, 6);
    assert_eq!(set.observation_count, 150);
    assert_eq!(set.auxiliary_parameter_count, 8);
    assert_eq!(set.auxiliary_residual_df, 142);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }

  let fourteen_fixture = Fixture::fourteen_instrument_sample();
  for option in ["robust", "cluster(group)"] {
    let mut session = load(&fourteen_fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 z14) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust fourteen-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      [
        "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9", "z10", "z11", "z12", "z13", "z14"
      ]
    );
    assert_eq!(set.degrees_of_freedom, 14);
    assert_eq!(set.observation_count, 336);
    assert_eq!(set.auxiliary_parameter_count, 16);
    assert_eq!(set.auxiliary_residual_df, 320);
    assert_eq!(
      set.cluster_group_count.is_some(),
      option.starts_with("cluster")
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }

  let mut session = load(&fourteen_fixture);
  session
    .execute(
      parse_command(
        "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9 z10 z11 z12 z13 z14 row) robust",
      )
      .unwrap(),
    )
    .unwrap();
  let before = session.state();
  assert!(
    session
      .execute(parse_command("estat ar, ci").unwrap())
      .unwrap_err()
      .to_string()
      .contains("at most fourteen excluded instruments")
  );
  assert_eq!(session.state(), before);
}

#[test]
fn seven_instrument_robust_ar_sets_reuse_retained_fit() {
  let fixture = Fixture::seven_instrument_sample();
  for option in ["robust", "cluster(group)"] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust seven-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      ["z1", "z2", "z3", "z4", "z5", "z6", "z7"]
    );
    assert_eq!(set.degrees_of_freedom, 7);
    assert_eq!(set.observation_count, 168);
    assert_eq!(set.auxiliary_parameter_count, 9);
    assert_eq!(set.auxiliary_residual_df, 159);
    assert_eq!(
      set.cluster_group_count,
      option.starts_with("cluster").then_some(24)
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn eight_instrument_robust_ar_sets_reuse_retained_fit() {
  let fixture = Fixture::eight_instrument_sample();
  for option in ["robust", "cluster(group)"] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust eight-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      ["z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8"]
    );
    assert_eq!(set.degrees_of_freedom, 8);
    assert_eq!(set.observation_count, 192);
    assert_eq!(set.auxiliary_parameter_count, 10);
    assert_eq!(set.auxiliary_residual_df, 182);
    assert_eq!(
      set.cluster_group_count,
      option.starts_with("cluster").then_some(24)
    );
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn nine_instrument_robust_ar_sets_reuse_retained_fit() {
  let fixture = Fixture::nine_instrument_sample();
  for (option, expected_interval) in [
    ("robust", [0.911362825704433, 1.28291406681336]),
    ("cluster(group)", [0.945792747726182, 1.20035149024702]),
  ] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x) iv(z1 z2 z3 z4 z5 z6 z7 z8 z9) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let command = parse_command("estat ar, ci").unwrap();
    let result = session.execute(command.clone()).unwrap();
    let ExecutionResult::RobustMultipleInstrumentAndersonRubinConfidenceSet(set) = &result else {
      panic!("expected robust nine-instrument AR confidence set");
    };
    assert_eq!(set.endogenous, "x");
    assert_eq!(
      set.excluded_instruments,
      ["z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8", "z9"]
    );
    assert_eq!(set.degrees_of_freedom, 9);
    assert_eq!(set.observation_count, 216);
    assert_eq!(set.auxiliary_parameter_count, 11);
    assert_eq!(set.auxiliary_residual_df, 205);
    assert_eq!(
      set.cluster_group_count,
      option.starts_with("cluster").then_some(24)
    );
    assert_eq!(set.intervals.len(), 1);
    let interval = &set.intervals[0];
    assert!((interval.lower.unwrap() - expected_interval[0]).abs() <= 3e-8);
    assert!((interval.upper.unwrap() - expected_interval[1]).abs() <= 3e-8);
    assert_eq!(session.state(), before);
    session
      .execute(parse_command("replace y = 999").unwrap())
      .unwrap();
    session.execute(parse_command("drop z1").unwrap()).unwrap();
    let changed = session.state();
    assert_eq!(session.execute(command).unwrap(), result);
    assert_eq!(session.state(), changed);
  }
}

#[test]
fn projected_ar_confidence_sets_reuse_the_retained_fit_for_homoskedastic_and_robust_fits() {
  let fixture = Fixture::school_like_sample();
  let mut session = load(&fixture);
  session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap();
  let before = session.state();
  assert!(
    session
      .execute(parse_command("estat ar, ci coefficient(w)").unwrap())
      .unwrap_err()
      .to_string()
      .contains("must name an endogenous regressor")
  );
  assert_eq!(session.state(), before);
  let command = parse_command("estat ar, ci coefficient(x1) level(90)").unwrap();
  let result = session.execute(command.clone()).unwrap();
  let ExecutionResult::ProjectedAndersonRubinConfidenceSet(set) = &result else {
    panic!("expected projected AR confidence set");
  };
  assert_eq!(set.coefficient, "x1");
  assert_eq!(set.nuisance_endogenous, ["x2"]);
  assert_eq!(set.observation_count, 58);
  assert_eq!(set.confidence_level, 90.0);
  assert_eq!(session.state(), before);

  session
    .execute(parse_command("replace y = 999").unwrap())
    .unwrap();
  session.execute(parse_command("drop z1").unwrap()).unwrap();
  let changed = session.state();
  assert_eq!(session.execute(command).unwrap(), result);
  assert_eq!(session.state(), changed);

  for option in ["robust", "cluster(group_id)"] {
    let mut session = load(&fixture);
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1 x2) iv(z1 z2) {option}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    let result = session
      .execute(parse_command("estat ar, ci coefficient(x1)").unwrap())
      .unwrap();
    let ExecutionResult::RobustProjectedAndersonRubinConfidenceSet(set) = result else {
      panic!("expected robust projected AR confidence set after {option}");
    };
    assert_eq!(set.coefficient, "x1");
    assert_eq!(set.nuisance_endogenous, ["x2"]);
    assert_eq!(
      set.observation_count,
      if option == "robust" { 58 } else { 57 }
    );
    assert_eq!(set.degrees_of_freedom, 2);
    assert!(set.closure_conservative);
    assert_eq!(session.state(), before);
  }
}

#[test]
fn ivregress_failure_clears_stale_model_state_but_gmm_is_explicitly_unsupported() {
  let fixture = Fixture::new_with_sql(
    "SELECT i::DOUBLE AS i, sin(i::DOUBLE) AS y, sin(i::DOUBLE * 0.4) AS w, \
      i::DOUBLE * 2 AS x1, i::DOUBLE * 3 AS x2, i::DOUBLE AS z1, i::DOUBLE AS z2 \
     FROM range(1, 31) AS t(i)",
  );
  let mut session = load(&fixture);
  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  assert!(session.state().estimation.is_some());

  assert_eq!(
    session
      .execute(parse_command("ivregress gmm y, endog(x) iv(z)").unwrap())
      .unwrap_err(),
    RuntimeError::IvRegressUnsupportedEstimator {
      estimator: "gmm".into()
    }
  );
  assert!(session.state().estimation.is_none());

  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  let Command::IvRegress { mut command } =
    parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2) cluster(group_id)").unwrap()
  else {
    panic!("expected an IV command");
  };
  command.robust = true;
  assert_eq!(
    session.execute(Command::IvRegress { command }).unwrap_err(),
    RuntimeError::IvRegressConflictingCovariance
  );
  assert!(session.state().estimation.is_none());

  session
    .execute(parse_command("regress y w").unwrap())
    .unwrap();
  let error = session
    .execute(parse_command("ivregress 2sls y w, endog(x1 x2) iv(z1 z2)").unwrap())
    .unwrap_err();
  assert!(matches!(error, RuntimeError::IvRegressFailed { .. }));
  assert!(session.state().estimation.is_none());
}
