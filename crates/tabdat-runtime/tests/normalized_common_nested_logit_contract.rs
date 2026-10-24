use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{
  CellValue, EstimationFamily, EstimationState, ExecutionResult, NormalizedCommonNestedLogitState,
};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
const FIT: &str = "nlogit chosen p q, group(id) alternative(alternative) nest(nest) upper(dshore dshorey) normalized common";
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-normalized-common-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,alternative,nest,chosen,p,q,dshore,dshorey\n");
    let mut id = 1;
    for (p, q, k) in [
      ([0, 2, 1, 3], [1, 0, 1, 0], 2),
      ([0, 2, 0, 2], [1, 0, 2, 1], 3),
      ([0, 2, 0, 2], [1, 0, 1, 0], 1),
    ] {
      for income in 0..=1 {
        let e = 3_usize.pow(income);
        for (chosen, count) in [18 * e, 32 * e, 9 * k, 16 * k].into_iter().enumerate() {
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
    assert_eq!(id, 901);
    csv.push_str("901,1,1,0.5,0,1,1,0\n901,2,1,1,,0,1,0\n901,3,2,0,1,1,0,0\n901,4,2,0,3,0,0,0\n902,1,1,NaN,0,1,1,0\n902,2,1,0,2,0,1,0\n902,3,2,0,1,1,0,0\n902,4,2,0,3,0,0,0\n");
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
fn fit(s: &mut Session) -> Box<NormalizedCommonNestedLogitState> {
  assert!(matches!(
    run(s, FIT),
    ExecutionResult::NormalizedCommonNestedLogit(_)
  ));
  let Some(EstimationState::NormalizedCommonNestedLogit(state)) = s.state().estimation else {
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
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn native_fit_retains_raw_design_whole_set_sample_natural_parameters_and_owned_state() {
  let f = Fixture::new();
  let mut s = f.load();
  let before = s.state().active_dataset;
  let data = rows(&mut s);
  let state = fit(&mut s);
  let r = &state.result;
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::NormalizedCommonNestedLogit
  );
  assert_eq!(r.choice_set_count, 900);
  assert_eq!(r.candidate_choice_set_count, 902);
  assert_eq!(r.excluded_choice_set_count, 2);
  assert_eq!(r.observation_count, 3600);
  assert_eq!(r.chosen_counts, [216, 384, 108, 192]);
  let sample = r.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (0..3600).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 3608);
  assert_eq!(sample.dropped_observations, 8);
  assert_eq!(state.problem.choice.case_rows.len(), 900);
  assert_eq!(state.problem.nest_codes.len(), 3600);
  assert_eq!(
    state.problem.choice.design.design_matrix[0],
    [0.0, 1.0, 1.0, 0.0]
  );
  for (c, b) in r
    .coefficients
    .iter()
    .zip([2_f64.ln(), 3_f64.ln(), 2_f64.ln(), 3_f64.ln()])
  {
    close(c.value, b);
  }
  close(r.dissimilarity.value, 0.5);
  assert_eq!(r.scale_test_null, 1.0);
  assert_eq!(r.independence_df, 1);
  assert!(r.random_utility_iv_bounds_satisfied);
  assert_eq!(r.covariance.len(), 5);
  assert_eq!(s.state().active_dataset, before);
  assert_eq!(rows(&mut s), data);
  let before = s.state();
  assert!(
    fail(&mut s, "estat margins")
      .to_string()
      .contains("point predictions only")
  );
  assert_eq!(s.state(), before);
  // Named model storage remains explicitly limited to its existing families.
  let before = s.state();
  assert!(
    fail(&mut s, "estimates store nested")
      .to_string()
      .contains("store supports")
  );
  assert_eq!(s.state(), before);
}

#[test]
fn current_set_prediction_ignores_outcomes_and_group_ids_and_nulls_whole_missing_sets() {
  let f = Fixture::new();
  let mut s = f.load();
  let state = fit(&mut s);
  run(&mut s, "predict baseline, pr");
  for (row, p) in rows(&mut s).iter().take(4).zip([0.18, 0.32, 0.18, 0.32]) {
    let CellValue::Float(actual) = row[8] else {
      panic!()
    };
    close(actual, p);
  }
  run(
    &mut s,
    r#"sql """SELECT * FROM active WHERE id>=901 ORDER BY id,alternative""" into missing_sets"#,
  );
  run(&mut s, "predict current, pr");
  let current = rows(&mut s);
  assert_eq!(current.len(), 8);
  for row in &current[..4] {
    assert_eq!(row[9], CellValue::Null);
  }
  for (row, p) in current[4..].iter().zip([0.18, 0.32, 0.18, 0.32]) {
    let CellValue::Float(actual) = row[9] else {
      panic!()
    };
    close(actual, p);
  }
  run(
    &mut s,
    r#"sql """SELECT 'new-person' AS id,alternative,nest,p,q,dshore,dshorey FROM active WHERE id=902 ORDER BY alternative""" into new_id"#,
  );
  run(&mut s, "predict new_probability, pr");
  for (row, p) in rows(&mut s).iter().zip([0.18, 0.32, 0.18, 0.32]) {
    let CellValue::Float(actual) = row[7] else {
      panic!()
    };
    close(actual, p);
  }
  assert_eq!(
    s.state().estimation,
    Some(EstimationState::NormalizedCommonNestedLogit(state))
  );
}

#[test]
fn structural_failures_precede_exclusion_and_failed_fits_clear_only_inference() {
  let f = Fixture::new();
  for projection in [
    "SELECT * FROM active WHERE NOT(id=901 AND alternative=4)",
    "SELECT * REPLACE(CASE WHEN id=901 AND alternative=3 THEN 2 ELSE alternative END AS alternative) FROM active",
    "SELECT * REPLACE(CASE WHEN id=901 THEN NULL ELSE id END AS id) FROM active",
    "SELECT * REPLACE(CASE WHEN id=901 AND alternative=1 THEN CAST('1.00000000000000000001' AS DECIMAL(38,20)) ELSE CAST(nest AS DECIMAL(38,20)) END AS nest) FROM active",
    "SELECT * REPLACE(CASE WHEN id=901 AND alternative=1 THEN 2 ELSE nest END AS nest) FROM active",
    "SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN 0 ELSE dshore END AS dshore) FROM active",
  ] {
    let mut s = f.load();
    fit(&mut s);
    run(&mut s, &format!("sql \"\"\"{projection}\"\"\" into broken"));
    let before = s.state().active_dataset;
    let data = rows(&mut s);
    assert!(
      matches!(fail(&mut s, FIT), RuntimeError::NlogitCommonFailed(..)),
      "{projection}"
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before);
    assert_eq!(rows(&mut s), data);
  }
}

#[test]
fn prediction_errors_preserve_model_data_metadata_and_publish_no_target() {
  let f = Fixture::new();
  for projection in [
    "SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN 0 ELSE dshore END AS dshore) FROM active",
    "SELECT * REPLACE(CASE WHEN id=901 AND alternative=1 THEN 2 ELSE nest END AS nest) FROM active",
    "SELECT * FROM active WHERE NOT(id=901 AND alternative=4)",
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
      RuntimeError::NlogitCommonFailed(..)
    ));
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s), data);
  }
  let mut s = f.load();
  fit(&mut s);
  run(&mut s, "predict baseline, pr");
  for text in [
    "predict baseline, pr",
    "predict bad, xb",
    "predict bad, pr outcome(1)",
    "predict bad, posterior_predictive std",
  ] {
    let before = s.state();
    let data = rows(&mut s);
    fail(&mut s, text);
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s), data);
  }
}
