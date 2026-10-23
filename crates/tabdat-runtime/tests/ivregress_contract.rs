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

  fn school_like_sample() -> Self {
    Self::new_with_sql(
      "WITH base AS (\
        SELECT i::DOUBLE AS i, sin(i / 7.0) + i / 90.0 AS w, \
          cos(i / 5.0) + i / 70.0 AS z1, \
          sin(i / 11.0) - cos(i / 13.0) + i / 200.0 AS z2, \
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
          CASE WHEN i = 2 THEN NULL ELSE x1 END AS x1, x2, z1, z2, \
          CASE WHEN i = 3 THEN NULL ELSE CAST(CAST(i AS INTEGER) % 12 AS VARCHAR) END AS group_id \
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
    session
      .execute(
        parse_command(&format!(
          "ivregress 2sls y w, endog(x1 x2) iv(z1 z2) {mode}"
        ))
        .unwrap(),
      )
      .unwrap();
    let before = session.state();
    assert!(
      session
        .execute(command.clone())
        .unwrap_err()
        .to_string()
        .contains("requires a nonrobust")
    );
    assert_eq!(session.state(), before);
  }
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
  for option in ["robust", "cluster(group_id)"] {
    let mut session = load(&fixture);
    session
      .execute(parse_command(&format!("ivregress 2sls y w, endog(x1) iv(z1 z2) {option}")).unwrap())
      .unwrap();
    let before = session.state();
    assert!(
      session
        .execute(overid.clone())
        .unwrap_err()
        .to_string()
        .contains("requires a nonrobust")
    );
    assert_eq!(session.state(), before);
  }
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
  for option in ["robust", "cluster(group_id)"] {
    let mut session = load(&fixture);
    session
      .execute(parse_command(&format!("ivregress 2sls y w, endog(x1) iv(z1 z2) {option}")).unwrap())
      .unwrap();
    let before = session.state();
    assert!(
      session
        .execute(ci.clone())
        .unwrap_err()
        .to_string()
        .contains("requires a nonrobust")
    );
    assert_eq!(session.state(), before);
  }
}

#[test]
fn projected_ar_confidence_sets_reuse_the_retained_fit_and_keep_robust_limits_explicit() {
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
    assert!(
      session
        .execute(parse_command("estat ar, ci coefficient(x1)").unwrap())
        .unwrap_err()
        .to_string()
        .contains("robust/cluster confidence sets are not implemented")
    );
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
