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
      "tabdat-logit-runtime-{}-{}",
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
fn same_cells(a: &[CellValue], b: &[CellValue]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    if matches!((a,b), (CellValue::Float(x),CellValue::Float(y)) if x.is_nan() && y.is_nan()) {
      continue;
    }
    assert_eq!(a, b);
  }
}
fn same_rows(a: &[Vec<CellValue>], b: &[Vec<CellValue>]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    same_cells(a, b);
  }
}
fn float(value: &CellValue) -> f64 {
  let CellValue::Float(v) = value else {
    panic!("expected DOUBLE: {value:?}")
  };
  *v
}

#[test]
fn logit_is_executable_but_requires_data_without_initializing_the_engine() {
  let mut s = Session::new();
  assert_eq!(
    fail(&mut s, "logit y x1"),
    RuntimeError::NoActiveDataset { command: "logit" }
  );
  assert!(s.initialized_capabilities().is_empty());
  assert!(s.state().estimation.is_none());
}

#[test]
fn retained_complete_cases_match_independent_r_and_survive_current_data_edits() {
  let f = Fixture::new(true);
  let mut s = f.load();
  let before = s.state();
  let ExecutionResult::Logit(fit) = execute(&mut s, "logit y x1 x2") else {
    panic!("expected logit")
  };
  close(fit.coefficients[1].value, 0.6009973792306854);
  close(fit.log_likelihood, -24.51495924658521);
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (1..=40).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 44);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::Logit
  );
  assert!(s.last_regression().is_none());
  let ame = execute(&mut s, "estat margins");
  let ExecutionResult::LogitMarginalEffects(ref result) = ame else {
    panic!("expected margins")
  };
  close(result.effects[0].value, 0.12738548021377377);
  close(result.effects[0].standard_error, 0.06969557366837566);
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
fn current_row_probability_and_index_predictions_preserve_order_labels_and_fit() {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "label variable x1 \"first predictor\"");
  execute(&mut s, "logit y x1 x2");
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
    same_cells(&after[..4], before);
    if index == 41 || index == 42 {
      assert_eq!(after[4..], vec![CellValue::Null; 3]);
      continue;
    }
    let eta = float(&after[5]);
    close(float(&after[4]), 1.0 / (1.0 + (-eta).exp()));
    assert_eq!(after[5], after[6]); // Existing bare-predict xb default is deliberate.
  }
  // Missing/nonfinite outcome is irrelevant to prediction on finite X.
  assert!(matches!(predicted[0][4], CellValue::Float(_)));
  assert!(matches!(predicted[43][4], CellValue::Float(_)));
  close(float(&predicted[1][4]), 0.4863880794799564);
  close(float(&predicted[20][4]), 0.39769955474325164);
  let before = s.state();
  let rows = preview(&mut s);
  for command in ["predict p, pr", "predict p, residuals"] {
    assert!(s.execute(parse_command(command).unwrap()).is_err());
    assert_eq!(s.state(), before);
    same_rows(&preview(&mut s), &rows);
  }
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
      Err(RuntimeError::PredictLogitUnsupported)
    ));
    assert_eq!(s.state(), before);
    same_rows(&preview(&mut s), &rows);
  }
  // Current data, not cached training probabilities, controls new predictions.
  execute(&mut s, "replace x1 = x1 + 1");
  execute(&mut s, "predict shifted, pr");
  let shifted = preview(&mut s);
  assert!((float(shifted[1].last().unwrap()) - float(&predicted[1][4])).abs() > 0.01);
  assert_eq!(s.state().estimation, cache);
}

#[test]
fn stable_extreme_current_predictions_and_numeric_type_failure_are_atomic() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "logit y x1 x2");
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
fn failed_new_attempts_clear_stale_fits_without_mutating_data() {
  let f = Fixture::new(false);
  for command in [
    "logit missing x1, robust",
    "logit y x1, cluster(missing)",
    "logit missing x1",
    "logit y x1 x1",
  ] {
    let mut s = f.load();
    execute(&mut s, "regress y x1");
    let dataset = s.state().active_dataset;
    assert!(s.state().estimation.is_some());
    fail(&mut s, command);
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, dataset);
    assert!(matches!(
      fail(&mut s, "estat margins"),
      RuntimeError::EstatMarginsRequiresPriorLogit
    ));
  }
  for value in ["2", "0"] {
    let mut s = f.load();
    execute(&mut s, "logit y x1");
    execute(&mut s, &format!("replace y = {value}"));
    let before = preview(&mut s);
    fail(&mut s, "logit y x1");
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
    execute(&mut s, "logit y x1");
    execute(
      &mut s,
      &format!("sql \"\"\"SELECT {projection} FROM active\"\"\" into invalid"),
    );
    let before = s.state();
    let rows = preview(&mut s);
    assert!(fail(&mut s, "logit y x1").to_string().contains(expected));
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    assert!(s.state().estimation.is_none());
    same_rows(&preview(&mut s), &rows);
  }
  let mut s = f.load();
  execute(&mut s, "logit y x1");
  execute(
    &mut s,
    r#"sql """SELECT source_row, x1, CASE WHEN x1 > 0 THEN 1 ELSE 0 END AS y FROM active""" into separated"#,
  );
  assert!(
    fail(&mut s, "logit y x1")
      .to_string()
      .contains("separation")
  );
  assert!(s.state().estimation.is_none());
}

#[test]
fn decimal_outcomes_are_validated_exactly_before_float_rounding_and_after_exclusions() {
  let f = Fixture::new(false);
  for (scale, nearby) in [
    (18, "0.999999999999999999"),
    (18, "1.000000000000000001"),
    (37, "0.9999999999999999999999999999999999999"),
  ] {
    let mut s = f.load();
    execute(&mut s, "logit y x1 x2");
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
      fail(&mut s, "logit y x1 x2")
        .to_string()
        .contains("exactly 0 or 1")
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before.active_dataset);
    same_rows(&preview(&mut s), &rows);
  }
  for scale in [0, 18, 37] {
    let mut s = f.load();
    execute(
      &mut s,
      &format!(
        r#"sql """SELECT source_row, CAST(y AS DECIMAL({},{})) AS y, x1, x2 FROM active""" into exact"#,
        scale + 1,
        scale
      ),
    );
    let ExecutionResult::Logit(fit) = execute(&mut s, "logit y x1 x2") else {
      panic!("expected logit")
    };
    close(fit.coefficients[1].value, 0.6009973792306854);
    assert_eq!(fit.event_count, 22);
  }
  let mut s = f.load();
  execute(
    &mut s,
    r#"sql """SELECT source_row, CAST(y AS DECIMAL(19,18)) AS y, x1, x2 FROM active UNION ALL SELECT 999, CAST('0.999999999999999999' AS DECIMAL(19,18)), CAST(NULL AS DOUBLE), 0""" into excluded"#,
  );
  let ExecutionResult::Logit(fit) = execute(&mut s, "logit y x1 x2") else {
    panic!("expected logit")
  };
  assert_eq!(fit.observation_count, 40);
  assert_eq!(
    fit.sample.as_ref().unwrap().retained_indices,
    (0..40).collect::<Vec<_>>()
  );
  assert_eq!(fit.sample.as_ref().unwrap().dropped_observations, 1);
}

#[test]
fn unrepresentable_current_indices_produce_null_without_losing_rows_or_fit() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "logit y x1 x2");
  let cache = s.state().estimation;
  // Each product is finite, but the two negative terms overflow when summed.
  execute(
    &mut s,
    r#"sql """SELECT source_row, y, CAST('-1.7976931348623157e308' AS DOUBLE) AS x1, CAST('1.7976931348623157e308' AS DOUBLE) AS x2 FROM active""" into overflowing_sum"#,
  );
  execute(&mut s, "predict p, pr");
  execute(&mut s, "predict xb, xb");
  let rows = preview(&mut s);
  assert_eq!(rows.len(), 40);
  assert!(
    rows
      .iter()
      .all(|row| row[4] == CellValue::Null && row[5] == CellValue::Null)
  );
  assert_eq!(s.state().estimation, cache);
}

#[test]
fn linear_only_postestimation_never_uses_logit_and_other_families_do_not_supply_margins() {
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
  execute(&mut s, "logit y x1, noconstant");
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::Logit(_))
  ));
  let before = s.state();
  for command in ["lincom x1", "test x1", "estat vif", "estat ar, null(0)"] {
    fail(&mut s, command);
    assert_eq!(s.state(), before);
  }
}
