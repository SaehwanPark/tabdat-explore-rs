use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{
  CellValue, EstimationFamily, EstimationState, ExecutionResult, NonNormalizedNestedLogitState,
};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
const FIT: &str = "nlogit chosen p q, group(id) alternative(alternative) nest(nest) upper(dshore dshorey) nonnormalized";
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-nlogit-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,alternative,nest,chosen,p,q,dshore,dshorey\n");
    let mut id = 1;
    for (p, q, counts) in [
      ([0, 1, 0, 0], [0, 0, 0, 1], [6, 12, 16, 48]),
      ([0, 0, 0, 1], [0, 1, 0, 0], [8, 24, 9, 18]),
      ([1, 0, 0, 1], [0, 0, 1, 0], [12, 6, 75, 50]),
    ] {
      for income in 0..=1 {
        for (chosen, count) in counts.into_iter().enumerate() {
          let count = count * if chosen < 2 { 3_usize.pow(income) } else { 1 };
          for _ in 0..count {
            for alt in 0..4 {
              writeln!(
                csv,
                "{id},{},{},{},{},{},{},{}",
                alt + 1,
                if alt < 2 { 1 } else { 2 },
                u8::from(alt == chosen),
                p[alt],
                q[alt],
                u8::from(alt < 2),
                if alt < 2 { income } else { 0 }
              )
              .unwrap();
            }
            id += 1;
          }
        }
      }
    }
    assert_eq!(id, 705);
    csv.push_str("705,1,1,0.5,0,0,1,0\n705,2,1,1,,0,1,0\n705,3,2,0,0,0,0,0\n705,4,2,0,0,1,0,0\n706,1,1,NaN,0,0,1,0\n706,2,1,0,1,0,1,0\n706,3,2,0,0,0,0,0\n706,4,2,0,0,1,0,0\n");
    fs::write(path.join("data.csv"), csv).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    s
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
fn fit(s: &mut Session) -> Box<NonNormalizedNestedLogitState> {
  let ExecutionResult::NonNormalizedNestedLogit(_) = run(s, FIT) else {
    panic!()
  };
  let Some(EstimationState::NonNormalizedNestedLogit(state)) = s.state().estimation else {
    panic!()
  };
  state
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = run(s, "head 20") else {
    panic!()
  };
  p.rows
}
fn same_rows(a: &[Vec<CellValue>], b: &[Vec<CellValue>]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
      match (a, b) {
        (CellValue::Float(a), CellValue::Float(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        _ => assert_eq!(a, b),
      }
    }
  }
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn owned_fit_preserves_whole_set_sample_and_native_parameter_roles() {
  let f = Fixture::new();
  let mut s = f.load();
  let before = s.state().active_dataset;
  let data = rows(&mut s);
  let state = fit(&mut s);
  let r = &state.result;
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::NonNormalizedNestedLogit
  );
  assert_eq!(r.choice_set_count, 704);
  assert_eq!(r.candidate_choice_set_count, 706);
  assert_eq!(r.excluded_choice_set_count, 2);
  assert_eq!(r.observation_count, 2816);
  assert_eq!(r.chosen_counts, [104, 168, 200, 232]);
  let sample = r.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (0..2816).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 2824);
  assert_eq!(sample.dropped_observations, 8);
  assert_eq!(state.problem.lower_predictor_count, 2);
  assert_eq!(state.problem.choice.case_rows.len(), 704);
  assert_eq!(state.problem.nest_codes.len(), 2816);
  for (coefficient, expected) in
    r.coefficients
      .iter()
      .zip([2_f64.ln(), 3_f64.ln(), 2_f64.ln(), 3_f64.ln()])
  {
    close(coefficient.value, expected);
  }
  for (d, expected) in r.dissimilarities.iter().zip([2., 3.]) {
    close(d.estimate.value, expected);
    assert_eq!(d.test_null, 1.);
  }
  assert!(!r.random_utility_iv_bounds_satisfied);
  assert_eq!(r.independence_df, 2);
  assert_eq!(r.covariance.len(), 6);
  assert_eq!(s.state().active_dataset, before);
  same_rows(&rows(&mut s), &data);
  let before = s.state();
  assert!(
    fail(&mut s, "estat margins")
      .to_string()
      .contains("point predictions only")
  );
  assert_eq!(s.state(), before);
}

#[test]
fn current_prediction_is_whole_set_and_ignores_outcomes_and_retained_group_ids() {
  let f = Fixture::new();
  let mut s = f.load();
  let state = fit(&mut s);
  run(&mut s, "predict baseline, pr");
  for (row, expected) in rows(&mut s).iter().take(4).zip([6., 12., 16., 48.]) {
    let CellValue::Float(p) = row[8] else {
      panic!()
    };
    close(p, expected / 82.);
  }
  run(
    &mut s,
    r#"sql """SELECT * FROM active WHERE id>=705 ORDER BY id,alternative""" into missing_sets"#,
  );
  run(&mut s, "predict current, pr");
  let observed = rows(&mut s);
  assert_eq!(observed.len(), 8);
  for row in &observed[..4] {
    assert_eq!(row[9], CellValue::Null);
  }
  let mut total = 0.;
  for row in &observed[4..] {
    let CellValue::Float(p) = row[9] else {
      panic!()
    };
    total += p;
  }
  close(total, 1.);
  run(
    &mut s,
    r#"sql """SELECT 'new-person' AS id,alternative,nest,p,q,dshore,dshorey FROM active WHERE id=706 ORDER BY alternative""" into new_id_without_outcome"#,
  );
  run(&mut s, "predict new_probability, pr");
  for (row, expected) in rows(&mut s).iter().zip([6., 12., 16., 48.]) {
    let CellValue::Float(p) = row[7] else {
      panic!()
    };
    close(p, expected / 82.);
  }
  assert_eq!(
    s.state().estimation,
    Some(EstimationState::NonNormalizedNestedLogit(state))
  );
}

#[test]
fn raw_nest_mapping_and_keys_fail_before_exclusion_and_failed_fits_clear_only_inference() {
  let f = Fixture::new();
  for projection in [
    "SELECT * FROM active WHERE NOT(id=705 AND alternative=4)",
    "SELECT * REPLACE(CASE WHEN id=705 AND alternative=3 THEN 2 ELSE alternative END AS alternative) FROM active",
    "SELECT * REPLACE(CASE WHEN id=705 THEN NULL ELSE id END AS id) FROM active",
    "SELECT * REPLACE(CASE WHEN id=705 AND alternative=1 THEN CAST('1.00000000000000000001' AS DECIMAL(38,20)) ELSE CAST(nest AS DECIMAL(38,20)) END AS nest) FROM active",
    "SELECT * REPLACE(CASE WHEN id=705 AND alternative=1 THEN 2 ELSE nest END AS nest) FROM active",
    "SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN 0 ELSE dshore END AS dshore) FROM active",
  ] {
    let mut s = f.load();
    fit(&mut s);
    run(&mut s, &format!("sql \"\"\"{projection}\"\"\" into broken"));
    let before = s.state().active_dataset;
    let data = rows(&mut s);
    assert!(
      matches!(fail(&mut s, FIT), RuntimeError::NlogitFailed { .. }),
      "{projection}"
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before);
    same_rows(&rows(&mut s), &data);
  }
}

#[test]
fn prediction_failures_publish_no_column_and_preserve_fit_rows_and_metadata() {
  let f = Fixture::new();
  for projection in [
    "SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN 0 ELSE dshore END AS dshore) FROM active",
    "SELECT * REPLACE(CASE WHEN id=705 AND alternative=1 THEN 2 ELSE nest END AS nest) FROM active",
    "SELECT * FROM active WHERE NOT(id=705 AND alternative=4)",
  ] {
    let mut s = f.load();
    fit(&mut s);
    run(
      &mut s,
      &format!("sql \"\"\"{projection}\"\"\" into bad_current"),
    );
    let before = s.state();
    let data = rows(&mut s);
    assert!(matches!(
      fail(&mut s, "predict bad, pr"),
      RuntimeError::NlogitFailed { .. }
    ));
    assert_eq!(s.state(), before);
    same_rows(&rows(&mut s), &data);
  }
  let mut s = f.load();
  fit(&mut s);
  run(&mut s, "predict baseline, pr");
  for command in [
    "predict baseline, pr",
    "predict bad, xb",
    "predict bad, pr outcome(1)",
    "predict bad, posterior_predictive std",
  ] {
    let before = s.state();
    let data = rows(&mut s);
    let _ = fail(&mut s, command);
    assert_eq!(s.state(), before);
    same_rows(&rows(&mut s), &data);
  }
}

#[test]
fn decimal_group_identity_and_retained_chosen_domain_are_not_rounded_through_f64() {
  let f = Fixture::new();
  let mut s = f.load();
  run(
    &mut s,
    r#"sql """SELECT * REPLACE(CAST(id AS DECIMAL(38,0))+CAST('9007199254740992' AS DECIMAL(38,0)) AS id) FROM active""" into exact_groups"#,
  );
  assert_eq!(fit(&mut s).result.choice_set_count, 704);
  let mut s = f.load();
  fit(&mut s);
  run(
    &mut s,
    r#"sql """SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN CAST('1.00000000000000000001' AS DECIMAL(38,20)) ELSE CAST(chosen AS DECIMAL(38,20)) END AS chosen) FROM active WHERE id<=704""" into bad_domain"#,
  );
  let before = s.state().active_dataset;
  let data = rows(&mut s);
  assert!(fail(&mut s, FIT).to_string().contains("exactly0 or1"));
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, before);
  same_rows(&rows(&mut s), &data);
}
