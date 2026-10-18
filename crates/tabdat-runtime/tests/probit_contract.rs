// Probit runtime contract. Shares the logit runtime fixture design (the same
// 40-row synthetic data with corrupt rows), so reference values are the
// R 4.6.1 numbers printed by
// crates/tabdat-stats/tests/fixtures/probit_reference.R and pinned in
// crates/tabdat-stats/tests/probit.rs; the extra digits do not change the
// parsed f64 value, so clippy's excessive-precision lint is silenced here too.
#![allow(clippy::excessive_precision)]

use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(missing: bool) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-probit-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = String::from("source_row,y,x1,x2\n");
    if missing {
      csv.push_str("0,,0,0\n");
    }
    for i in 1..=40 {
      let x1 = ((i * 7) % 17) as f64 / 5.0 - 1.6;
      let x2 = ((i * 11) % 19) as f64 / 7.0 - 9.0 / 7.0;
      let y = u8::from((((i * 13) % 37) as f64) < 18.0 + 4.0 * x1 - 3.0 * x2);
      writeln!(csv, "{i},{y},{x1},{x2}").unwrap();
    }
    if missing {
      csv.push_str("41,2,,0\n42,0,NaN,0\n43,Inf,0,0\n");
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }
  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(session: &mut Session, text: &str) -> RuntimeError {
  session.execute(parse_command(text).unwrap()).unwrap_err()
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
fn preview(session: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(result) = execute(session, "head 100") else {
    panic!("expected head")
  };
  result.rows
}
fn float(value: &CellValue) -> f64 {
  let CellValue::Float(v) = value else {
    panic!("expected DOUBLE: {value:?}")
  };
  *v
}

#[test]
fn probit_is_executable_but_requires_data_without_initializing_the_engine() {
  let mut s = Session::new();
  assert_eq!(
    fail(&mut s, "probit y x1"),
    RuntimeError::NoActiveDataset { command: "probit" }
  );
  assert!(s.initialized_capabilities().is_empty());
  assert!(s.state().estimation.is_none());
}

#[test]
fn retained_complete_cases_match_independent_r_and_survive_current_data_edits() {
  let f = Fixture::new(true);
  let mut s = f.load();
  let before = s.state();
  let ExecutionResult::Probit(fit) = execute(&mut s, "probit y x1 x2") else {
    panic!("expected probit")
  };
  close(fit.coefficients[0].value, 0.13364045759262577);
  close(fit.coefficients[1].value, 0.37200554216777859);
  close(fit.coefficients[2].value, -0.35371618791237436);
  close(fit.log_likelihood, -24.51228107498369);
  assert_eq!(fit.event_count, 22);
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (1..=40).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 44);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::Probit
  );
  assert!(s.last_regression().is_none());
  let ame = execute(&mut s, "estat margins");
  let ExecutionResult::ProbitMarginalEffects(ref result) = ame else {
    panic!("expected probit margins")
  };
  close(result.effects[0].value, 0.12987211997972109);
  close(result.effects[0].standard_error, 0.072857744094171742);
  close(result.effects[1].value, -0.12348706131535794);
  close(result.effects[1].standard_error, 0.095838874529783846);
  let owned = s.state().estimation;
  execute(&mut s, "replace x1 = x1 + 100");
  execute(&mut s, "replace y = 0");
  assert_eq!(s.state().estimation, owned);
  assert_eq!(execute(&mut s, "estat margins"), ame);
  execute(&mut s, "drop x1");
  let snapshot = s.state();
  assert!(matches!(
    fail(&mut s, "predict bad, pr"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), snapshot);
  assert_eq!(execute(&mut s, "estat margins"), ame);
}

#[test]
fn current_row_probability_and_index_predictions_use_the_gaussian_link() {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "label variable x1 \"first predictor\"");
  execute(&mut s, "probit y x1 x2");
  let cache = s.state().estimation;
  let labels = s.state().labels;
  let old = preview(&mut s);
  execute(&mut s, "predict p, pr");
  execute(&mut s, "predict index, xb");
  execute(&mut s, "predict bare");
  assert_eq!(s.state().estimation, cache);
  assert_eq!(s.state().labels, labels);
  let predicted = preview(&mut s);
  assert_eq!(predicted.len(), 44);
  for (index, (before, after)) in old.iter().zip(&predicted).enumerate() {
    assert_eq!(after.len(), old[0].len() + 3);
    // Column layout is preserved and prediction columns append after the data.
    same_cells(&after[..4], before);
    if index == 41 || index == 42 {
      assert_eq!(after[4..], vec![CellValue::Null; 3]);
      continue;
    }
    assert_eq!(after[5], after[6]); // Existing bare-predict xb default is deliberate.
  }
  // Missing/nonfinite outcome is irrelevant to prediction on finite X.
  assert!(matches!(predicted[0][4], CellValue::Float(_)));
  assert!(matches!(predicted[43][4], CellValue::Float(_)));
  // Rows with x1 = x2 = 0 predict the pure-intercept probability in both rows.
  close(float(&predicted[0][4]), 0.55315655470399672);
  close(float(&predicted[43][4]), 0.55315655470399672);
  close(float(&predicted[1][4]), 0.48332013150794201);
  close(float(&predicted[20][4]), 0.39549481488702537);
  close(float(&predicted[40][4]), 0.66888114268343024);
  // Current data, not cached training probabilities, controls new predictions.
  execute(&mut s, "replace x1 = x1 + 1");
  execute(&mut s, "predict shifted, pr");
  let shifted = preview(&mut s);
  assert!((float(shifted[1].last().unwrap()) - float(&predicted[1][4])).abs() > 0.01);
  assert_eq!(s.state().estimation, cache);
  assert_eq!(s.state().labels, labels);
  // Finite-X rows keep finite shifted predictions; missing channels stay NULL.
  for index in (0..=40).chain([43]) {
    assert!(
      matches!(shifted[index].last(), Some(CellValue::Float(v)) if v.is_finite()),
      "row {index}"
    );
  }
  assert!(matches!(shifted[41].last(), Some(CellValue::Null)));
  assert!(matches!(shifted[42].last(), Some(CellValue::Null)));
}

#[test]
fn binary_prediction_options_reject_and_never_mutate_state() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "probit y x1 x2");
  execute(&mut s, "predict p, pr");
  let before = s.state();
  let rows = preview(&mut s);
  assert!(matches!(
    fail(&mut s, "predict p2, residuals"),
    RuntimeError::PredictBinaryUnsupported
  ));
  assert_eq!(s.state(), before);
  assert_eq!(preview(&mut s), rows);
  assert!(parse_command("predict unused, pr interval").is_err());
  for flag in 0..3 {
    let tabdat_domain::Command::Predict { mut command } =
      parse_command("predict unused, pr").unwrap()
    else {
      unreachable!()
    };
    match flag {
      0 => command.interval = true,
      1 => command.std = true,
      _ => command.saving = Some("unused.csv".into()),
    }
    assert!(matches!(
      s.execute(tabdat_domain::Command::Predict { command }),
      Err(RuntimeError::PredictBinaryUnsupported)
    ));
    assert_eq!(s.state(), before);
    assert_eq!(preview(&mut s), rows);
  }
  // Existing variable targets are rejected without mutating data.
  assert!(matches!(
    fail(&mut s, "predict x2, pr"),
    RuntimeError::PredictTargetExists { .. }
  ));
  assert_eq!(s.state(), before);
  assert_eq!(preview(&mut s), rows);
}

#[test]
fn stable_extreme_current_predictions_and_numeric_type_failure_are_atomic() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "probit y x1 x2");
  execute(&mut s, "replace x1 = 10000");
  execute(&mut s, "predict upper, pr");
  assert_eq!(float(&preview(&mut s)[0][4]), 1.0);
  execute(&mut s, "replace x1 = -10000");
  execute(&mut s, "predict lower, pr");
  assert_eq!(float(&preview(&mut s)[0][5]), 0.0);
  execute(&mut s, "drop x1");
  execute(
    &mut s,
    r#"sql """SELECT source_row, y, 'not numeric' AS x1, x2, upper, lower FROM active""" into typed"#,
  );
  let before = s.state();
  let rows = preview(&mut s);
  assert!(
    fail(&mut s, "predict bad, pr")
      .to_string()
      .contains("numeric predictor")
  );
  assert_eq!(s.state(), before);
  assert_eq!(preview(&mut s), rows);
}

#[test]
fn unrepresentable_current_indices_produce_null_without_losing_rows_or_fit() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "probit y x1 x2");
  let cache = s.state().estimation;
  // Huge but finite predictors only saturate the Gaussian link: unlike the
  // logit contract's sum-overflow case, |b1|+|b2| < 1 keeps the index
  // representable at binary64 extremes, so pr is exactly zero and xb stays a
  // finite extreme value rather than NULL.
  execute(
    &mut s,
    r#"sql """SELECT source_row, y, CAST('-1.7976931348623157e308' AS DOUBLE) AS x1, CAST('1.7976931348623157e308' AS DOUBLE) AS x2 FROM active""" into inflated_scale"#,
  );
  execute(&mut s, "predict p, pr");
  execute(&mut s, "predict pxb, xb");
  let rows = preview(&mut s);
  assert_eq!(rows.len(), 40);
  assert!(rows.iter().all(|row| {
    row[4] == CellValue::Float(0.0)
      && matches!(row[5], CellValue::Float(v) if v.is_finite() && v < 0.0)
  }));
  assert_eq!(s.state().estimation, cache);
}

#[test]
fn failed_new_attempts_clear_stale_fits_without_mutating_data() {
  let f = Fixture::new(false);
  for command in [
    "probit missing x1, robust",
    "probit y x1, cluster(missing)",
    "probit missing x1",
    "probit y x1 x1",
  ] {
    let mut s = f.load();
    execute(&mut s, "regress y x1");
    let dataset = s.state().active_dataset;
    assert!(s.state().estimation.is_some());
    let error = fail(&mut s, command);
    assert!(
      matches!(error, RuntimeError::ProbitFailed { .. }),
      "{error}"
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, dataset);
    assert!(matches!(
      fail(&mut s, "estat margins"),
      RuntimeError::EstatMarginsRequiresPriorLogit
    ));
  }
  let mut s = f.load();
  assert_eq!(
    fail(&mut s, "probit y x1, cluster(missing)").to_string(),
    "probit: unknown cluster variable: missing"
  );
  for value in ["2", "0"] {
    let mut s = f.load();
    execute(&mut s, "probit y x1");
    execute(&mut s, &format!("replace y = {value}"));
    let before = preview(&mut s);
    let error = fail(&mut s, "probit y x1");
    assert!(
      matches!(error, RuntimeError::ProbitFailed { .. }),
      "{error}"
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(preview(&mut s), before);
  }
  for (projection, expected) in [
    (
      "source_row, CAST(y AS VARCHAR) AS y, x1",
      "numeric variable required",
    ),
    (
      "source_row, y, CAST(NULL AS DOUBLE) AS x1",
      "no complete finite observations",
    ),
  ] {
    let mut s = f.load();
    execute(&mut s, "probit y x1");
    execute(
      &mut s,
      &format!("sql \"\"\"SELECT {projection} FROM active\"\"\" into invalid"),
    );
    let before = s.state();
    let rows = preview(&mut s);
    assert!(fail(&mut s, "probit y x1").to_string().contains(expected));
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    assert!(s.state().estimation.is_none());
    assert_eq!(preview(&mut s), rows);
  }
  let mut s = f.load();
  execute(&mut s, "probit y x1");
  execute(
    &mut s,
    r#"sql """SELECT source_row, x1, CASE WHEN x1 > 0 THEN 1 ELSE 0 END AS y FROM active""" into separated"#,
  );
  assert!(
    fail(&mut s, "probit y x1")
      .to_string()
      .contains("separation")
  );
  assert!(s.state().estimation.is_none());
}

#[test]
fn decimal_outcomes_are_validated_exactly_before_float_rounding() {
  let f = Fixture::new(false);
  for (scale, nearby) in [(18, "0.999999999999999999"), (18, "1.000000000000000001")] {
    let mut s = f.load();
    execute(&mut s, "probit y x1 x2");
    execute(
      &mut s,
      &format!(
        r#"sql """SELECT source_row, CASE WHEN y=1 THEN CAST('{nearby}' AS DECIMAL({},{})) ELSE CAST(0 AS DECIMAL({},{})) END AS y, x1, x2 FROM active""" into nearby"#,
        scale + 1,
        scale,
        scale + 1,
        scale
      ),
    );
    let before = s.state();
    let rows = preview(&mut s);
    assert!(
      fail(&mut s, "probit y x1 x2")
        .to_string()
        .contains("exactly 0 or 1")
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(preview(&mut s), rows);
  }
  for scale in [0, 18] {
    let mut s = f.load();
    execute(
      &mut s,
      &format!(
        r#"sql """SELECT source_row, CAST(y AS DECIMAL({},{})) AS y, x1, x2 FROM active""" into exact"#,
        scale + 1,
        scale
      ),
    );
    let ExecutionResult::Probit(fit) = execute(&mut s, "probit y x1 x2") else {
      panic!("expected probit")
    };
    close(fit.coefficients[2].value, -0.35371618791237436);
    close(fit.log_likelihood, -24.51228107498369);
    assert_eq!(fit.event_count, 22);
  }
}

#[test]
fn prior_margins_wording_stays_logit_and_other_families_do_not_supply_margins() {
  let f = Fixture::new(false);
  let mut s = f.load();
  assert!(matches!(
    fail(&mut s, "estat margins"),
    RuntimeError::EstatMarginsRequiresPriorLogit
  ));
  execute(&mut s, "regress y x1");
  assert!(matches!(
    fail(&mut s, "estat margins"),
    RuntimeError::EstatMarginsRequiresPriorLogit
  ));
  execute(&mut s, "probit y x1");
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::Probit(_))
  ));
  assert!(matches!(
    execute(&mut s, "estat margins"),
    ExecutionResult::ProbitMarginalEffects(_)
  ));
  let before = s.state();
  for command in ["lincom x1", "test x1", "estat vif", "estat ar, null(0)"] {
    let error = fail(&mut s, command);
    assert!(
      matches!(
        error,
        RuntimeError::LincomRequiresPriorEstimation
          | RuntimeError::TestRequiresPriorEstimation
          | RuntimeError::EstatRequiresPriorRegression
          | RuntimeError::EstatArRequiresPriorIv
      ),
      "{error}"
    );
    assert_eq!(s.state(), before);
  }
  // predict with no prior estimation keeps the pinned linear-slice wording:
  // pr without a prior binary fit is outside the supported predictor kinds.
  let mut s = Session::new();
  execute(&mut s, &format!("use {}", f.0.join("data.csv").display()));
  assert!(matches!(
    fail(&mut s, "predict unused, pr"),
    RuntimeError::UnsupportedCommand { name: "predict" }
  ));
}

#[test]
fn clustered_binary_inference_owns_groups_and_retained_delta_design() {
  let f = Fixture::new(true);
  for (family, covariance12, ame_se) in [
    ("logit", 0.0305324276218534, 0.0535030958247444),
    ("probit", 0.0155928852262543, 0.0570919529285024),
  ] {
    let mut s = f.load();
    execute(
      &mut s,
      r#"sql """SELECT *, (source_row*7)%9 AS cluster_id FROM active""" into grouped"#,
    );
    execute(&mut s, "label variable x1 \"first predictor\"");
    let before = s.state();
    let result = execute(&mut s, &format!("{family} y x1 x2, cluster(cluster_id)"));
    let (ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit)) = result else {
      panic!("binary fit")
    };
    assert_eq!(fit.cluster.as_ref().unwrap().variable, "cluster_id");
    assert_eq!(fit.cluster.as_ref().unwrap().group_count, 9);
    assert_eq!(
      fit.sample.as_ref().unwrap().retained_indices,
      (1..=40).collect::<Vec<_>>()
    );
    assert_eq!(
      fit
        .sample
        .as_ref()
        .unwrap()
        .cluster_groups
        .as_ref()
        .unwrap(),
      &(1..=40)
        .map(|i| ((i * 7) % 9).to_string())
        .collect::<Vec<_>>()
    );
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    close(fit.covariance[1][2], covariance12);
    assert!(fit.likelihood_ratio_p_value.is_none());
    let effects = execute(&mut s, "estat margins");
    let ame = match &effects {
      ExecutionResult::LogitMarginalEffects(ame) => ame,
      ExecutionResult::ProbitMarginalEffects(ame) => ame.as_ref(),
      _ => panic!("effects"),
    };
    close(ame.effects[0].standard_error, ame_se);
    assert_eq!(ame.cluster, fit.cluster);
    let retained = s.state().estimation;
    execute(&mut s, "drop cluster_id");
    execute(&mut s, "predict without_group, pr");
    assert_eq!(s.state().estimation, retained);
    assert_eq!(execute(&mut s, "estat margins"), effects);
    execute(&mut s, "generate cluster_id = 99");
    execute(&mut s, "replace x1 = x1 + 10");
    assert_eq!(execute(&mut s, "estat margins"), effects);
    let cache = s.state().estimation;
    execute(&mut s, "predict p, pr");
    assert_eq!(s.state().estimation, cache);
    let rows = preview(&mut s);
    assert!(
      fail(&mut s, &format!("{family} y x1 x2, cluster(cluster_id)"))
        .to_string()
        .contains("at least two")
    );
    assert!(s.state().estimation.is_none());
    for (a, b) in preview(&mut s).iter().zip(&rows) {
      same_cells(a, b);
    }
  }
}

#[test]
fn cluster_missingness_precedes_outcome_domain_and_wide_ids_never_round_together() {
  let f = Fixture::new(true);
  for family in ["logit", "probit"] {
    let mut s = f.load();
    execute(
      &mut s,
      r#"sql """SELECT source_row, CASE WHEN source_row=1 THEN 2 ELSE y END AS y, x1, x2, CASE WHEN source_row IN (1,2) THEN NULL WHEN source_row=3 THEN CAST('NaN' AS DOUBLE) ELSE CAST(source_row%9 AS DOUBLE) END AS cluster_id FROM active""" into grouped"#,
    );
    let result = execute(&mut s, &format!("{family} y x1 x2, cluster(cluster_id)"));
    let (ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit)) = result else {
      panic!("fit")
    };
    assert_eq!(
      fit.sample.as_ref().unwrap().retained_indices,
      (4..=40).collect::<Vec<_>>()
    );
    assert_eq!(fit.sample.as_ref().unwrap().dropped_observations, 7);
    let mut s = f.load();
    execute(
      &mut s,
      r#"sql """SELECT *, CAST('9007199254740992' AS DECIMAL(38,0)) + CAST(source_row%2 AS DECIMAL(38,0)) AS cluster_id FROM active""" into wide_groups"#,
    );
    let result = execute(&mut s, &format!("{family} y x1 x2, cluster(cluster_id)"));
    let (ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit)) = result else {
      panic!("fit")
    };
    assert_eq!(fit.cluster.as_ref().unwrap().group_count, 2);
    let mut s = f.load();
    execute(
      &mut s,
      r#"sql """SELECT *, CASE WHEN source_row%2=0 THEN 'a,b' ELSE 'a' END AS cluster_id FROM active""" into text_groups"#,
    );
    let result = execute(&mut s, &format!("{family} y x1 x2, cluster(cluster_id)"));
    let (ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit)) = result else {
      panic!("fit")
    };
    assert_eq!(fit.cluster.as_ref().unwrap().group_count, 2);
  }
}

#[test]
fn robust_binary_fits_preserve_sample_data_predictions_and_retained_delta_estimand() {
  let f = Fixture::new(true);
  for (family, covariance12, ame_se) in [
    ("logit", 0.0254652771050161, 0.0644521764962715),
    ("probit", 0.0143048319483742, 0.0684286804542189),
  ] {
    let mut s = f.load();
    execute(&mut s, "label variable x1 \"first predictor\"");
    let dataset = s.state().active_dataset;
    let labels = s.state().labels;
    let before = preview(&mut s);
    execute(&mut s, &format!("{family} y x1 x2"));
    let classical = s.state().estimation;
    let result = execute(&mut s, &format!("{family} y x1 x2, robust"));
    let (ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit)) = result else {
      panic!("expected binary fit")
    };
    assert!(fit.robust);
    let sample = fit.sample.as_ref().unwrap();
    assert_eq!(sample.retained_indices, (1..=40).collect::<Vec<_>>());
    assert_eq!(sample.total_observations, 44);
    assert_eq!(sample.dropped_observations, 4);
    let (EstimationState::Logit(old) | EstimationState::Probit(old)) = classical.unwrap() else {
      panic!("expected prior binary fit")
    };
    assert_eq!(fit.fitted_probabilities, old.result.fitted_probabilities);
    for (a, b) in fit.coefficients.iter().zip(&old.result.coefficients) {
      assert_eq!(a.value, b.value);
    }
    close(fit.covariance[1][2], covariance12);
    assert!(fit.likelihood_ratio_p_value.is_none());
    assert_eq!(s.state().active_dataset, dataset);
    assert_eq!(s.state().labels, labels);
    for (a, b) in preview(&mut s).iter().zip(&before) {
      same_cells(a, b);
    }
    let effects = execute(&mut s, "estat margins");
    let ame = match &effects {
      ExecutionResult::LogitMarginalEffects(ame) => ame,
      ExecutionResult::ProbitMarginalEffects(ame) => ame.as_ref(),
      _ => panic!("expected binary effects"),
    };
    assert!(ame.robust);
    close(ame.effects[0].standard_error, ame_se);
    execute(&mut s, "predict p, pr");
    let state = s.state().estimation;
    assert_eq!(s.state().labels, labels);
    execute(&mut s, "replace x1 = x1 + 10");
    execute(&mut s, "replace y = 0");
    assert_eq!(execute(&mut s, "estat margins"), effects);
    assert_eq!(s.state().estimation, state);
    let changed = preview(&mut s);
    assert!(
      fail(&mut s, &format!("{family} y x1 x2, robust"))
        .to_string()
        .contains("both outcome classes")
    );
    assert!(s.state().estimation.is_none());
    for (a, b) in preview(&mut s).iter().zip(&changed) {
      same_cells(a, b);
    }
  }
}

fn same_cells(a: &[CellValue], b: &[CellValue]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    if matches!((a, b), (CellValue::Float(x), CellValue::Float(y)) if x.is_nan() && y.is_nan()) {
      continue;
    }
    assert_eq!(a, b);
  }
}
