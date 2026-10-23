use std::{
  fmt::Write,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationFamily, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-clogit-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,alternative,chosen,p,q\n");
    let mut id = 1;
    for (chosen, &count) in [2, 4, 6].iter().enumerate() {
      for _ in 0..count {
        for alt in 0..3 {
          writeln!(
            csv,
            "{id},{},{},{},{}",
            alt + 1,
            u8::from(alt == chosen),
            u8::from(alt == 1),
            u8::from(alt == 2)
          )
          .unwrap();
        }
        id += 1;
      }
    }
    // Domain errors on one row do not override whole-set finite-case exclusion.
    csv.push_str("13,1,0.5,0,0\n13,2,1,,0\n13,3,0,0,1\n14,1,NaN,0,0\n14,2,0,1,0\n14,3,1,0,1\n");
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
fn fit(s: &mut Session) -> Box<tabdat_domain::ConditionalLogitState> {
  run(s, "clogit chosen p q, group(id) alternative(alternative)");
  let Some(tabdat_domain::EstimationState::ConditionalLogit(state)) = s.state().estimation else {
    panic!()
  };
  state
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = run(s, "head 100") else {
    panic!()
  };
  p.rows
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
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

#[test]
fn whole_choice_set_missingness_preserves_exact_sample_and_independent_decision_units() {
  let f = Fixture::new();
  let mut s = f.load();
  let state = fit(&mut s);
  let r = &state.result;
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::ConditionalLogit
  );
  assert_eq!(r.observation_count, 36);
  assert_eq!(r.choice_set_count, 12);
  assert_eq!(r.candidate_choice_set_count, 14);
  assert_eq!(r.excluded_choice_set_count, 2);
  assert_eq!(r.alternative_levels, [1, 2, 3]);
  assert_eq!(r.chosen_counts, [2, 4, 6]);
  let sample = r.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (0..36).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 42);
  assert_eq!(sample.dropped_observations, 6);
  close(r.coefficients[0].value, 2.0_f64.ln());
  close(r.coefficients[1].value, 3.0_f64.ln());
  close(r.covariance[0][0], 0.75);
  close(r.covariance[0][1], 0.5);
  close(r.covariance[1][1], 2.0 / 3.0);
  assert_eq!(state.problem.case_rows.len(), 12);
  assert!(state.problem.case_rows.iter().all(|c| c.len() == 3));
  let before = s.state();
  assert!(
    fail(&mut s, "estat margins")
      .to_string()
      .contains("point predictions only")
  );
  assert_eq!(s.state(), before);
}

#[test]
fn current_prediction_nulls_entire_missing_set_and_ignores_current_outcomes_and_old_ids() {
  let f = Fixture::new();
  let mut s = f.load();
  let state = fit(&mut s);
  run(&mut s, "predict probability, pr");
  let observed = rows(&mut s);
  assert_eq!(observed.len(), 42);
  for (i, row) in observed.iter().enumerate() {
    if (36..39).contains(&i) {
      assert_eq!(row[5], CellValue::Null);
    } else {
      let CellValue::Float(p) = row[5] else {
        panic!()
      };
      close(p, [1.0 / 6.0, 1.0 / 3.0, 0.5][i % 3]);
    }
  }
  // Missing/invalid current outcomes do not enter a choice-probability query.
  run(
    &mut s,
    r#"sql """SELECT id+100 AS id, alternative, coalesce(p,1) AS p, q FROM active ORDER BY id,alternative""" into new_cases"#,
  );
  run(&mut s, "predict updated, pr");
  for (i, row) in rows(&mut s).iter().enumerate() {
    let CellValue::Float(p) = row[4] else {
      panic!()
    };
    close(p, [1.0 / 6.0, 1.0 / 3.0, 0.5][i % 3]);
  }
  assert_eq!(
    s.state().estimation,
    Some(tabdat_domain::EstimationState::ConditionalLogit(state))
  );
  for command in [
    "predict updated, pr",
    "predict bad, xb",
    "predict bad, pr outcome(1)",
  ] {
    let before = s.state();
    let original = rows(&mut s);
    let _ = fail(&mut s, command);
    assert_eq!(s.state(), before);
    same_rows(&rows(&mut s), &original);
  }
}

#[test]
fn bad_or_incomplete_keys_fail_even_in_sets_with_missing_predictors_and_do_not_publish() {
  let f = Fixture::new();
  for projection in [
    "SELECT * FROM active WHERE NOT(id=13 AND alternative=3)",
    "SELECT * REPLACE(CASE WHEN id=13 AND alternative=3 THEN 2 ELSE alternative END AS alternative) FROM active",
    "SELECT * REPLACE(CASE WHEN id=13 THEN NULL ELSE id END AS id) FROM active",
    "SELECT * REPLACE(CASE WHEN id=13 AND alternative=1 THEN CAST('1.00000000000000000001' AS DECIMAL(38,20)) ELSE CAST(alternative AS DECIMAL(38,20)) END AS alternative) FROM active",
    "SELECT * FROM active WHERE alternative<>3",
  ] {
    let mut s = f.load();
    fit(&mut s);
    run(&mut s, &format!("sql \"\"\"{projection}\"\"\" into broken"));
    let before = s.state();
    let data = rows(&mut s);
    assert!(matches!(
      fail(&mut s, "predict bad, pr"),
      RuntimeError::ClogitFailed { .. }
    ));
    assert_eq!(s.state(), before);
    same_rows(&rows(&mut s), &data);
  }
}

#[test]
fn exact_decimal_group_identity_and_chosen_domain_are_not_rounded_through_f64() {
  let f = Fixture::new();
  let mut s = f.load();
  run(
    &mut s,
    r#"sql """SELECT * REPLACE(CAST(id AS DECIMAL(38,0))+CAST('9007199254740992' AS DECIMAL(38,0)) AS id) FROM active""" into wide_ids"#,
  );
  let state = fit(&mut s);
  assert_eq!(state.result.choice_set_count, 12);
  assert_eq!(state.result.candidate_choice_set_count, 14);
  let mut s = f.load();
  run(
    &mut s,
    r#"sql """SELECT * REPLACE(CASE WHEN id=1 AND alternative=1 THEN CAST('1.00000000000000000001' AS DECIMAL(38,20)) ELSE CAST(chosen AS DECIMAL(38,20)) END AS chosen) FROM active WHERE id<=12""" into exact_choice"#,
  );
  let data = rows(&mut s);
  assert!(
    fail(
      &mut s,
      "clogit chosen p q, group(id) alternative(alternative)"
    )
    .to_string()
    .contains("exactly0 or1")
  );
  assert!(s.state().estimation.is_none());
  same_rows(&rows(&mut s), &data);
}

#[test]
fn failed_new_fits_clear_prior_inference_without_mutating_rows_or_metadata() {
  let f = Fixture::new();
  for command in [
    "clogit chosen missing, group(id) alternative(alternative)",
    "clogit missing p q, group(id) alternative(alternative)",
    "clogit chosen p p, group(id) alternative(alternative)",
    "clogit chosen p q, group(missing) alternative(alternative)",
  ] {
    let mut s = f.load();
    fit(&mut s);
    let data = rows(&mut s);
    let metadata = s.state().active_dataset;
    assert!(matches!(
      fail(&mut s, command),
      RuntimeError::ClogitFailed { .. }
    ));
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, metadata);
    same_rows(&rows(&mut s), &data);
  }
  for expression in ["0", "1", "2"] {
    let mut s = f.load();
    fit(&mut s);
    run(&mut s, &format!("replace chosen = {expression}"));
    let data = rows(&mut s);
    let _ = fail(
      &mut s,
      "clogit chosen p q, group(id) alternative(alternative)",
    );
    assert!(s.state().estimation.is_none());
    same_rows(&rows(&mut s), &data);
  }
}

#[test]
fn nonfinite_utility_arithmetic_is_an_atomic_error_not_a_dropped_alternative() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "replace p = p/10");
  fit(&mut s);
  run(
    &mut s,
    r#"sql """SELECT * REPLACE(CASE WHEN alternative=2 THEN CAST('1.7976931348623157e308' AS DOUBLE) ELSE 0 END AS p) FROM active""" into extreme"#,
  );
  let before = s.state();
  let data = rows(&mut s);
  assert!(
    fail(&mut s, "predict bad, pr")
      .to_string()
      .contains("nonfinite within-choice-set linear index")
  );
  assert_eq!(s.state(), before);
  same_rows(&rows(&mut s), &data);
}
