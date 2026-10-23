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
      "tabdat-multinomial-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,choice,x\n0,,0\n");
    let mut id = 1;
    for (g, counts) in [[1, 1, 6], [2, 4, 6], [4, 16, 6]].iter().enumerate() {
      for (category, &count) in counts.iter().enumerate() {
        for _ in 0..count {
          writeln!(csv, "{id},{},{}", category + 1, g as i32 - 1).unwrap();
          id += 1;
        }
      }
    }
    csv.push_str("47,2.5,\n48,1,NaN\n49,Inf,0\n");
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
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = run(s, "head 100") else {
    panic!()
  };
  p.rows
}
fn assert_same_rows(actual: &[Vec<CellValue>], expected: &[Vec<CellValue>]) {
  assert_eq!(actual.len(), expected.len());
  for (a, b) in actual.iter().zip(expected) {
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
fn fit_uses_exact_joint_sample_and_installs_owned_multinomial_state() {
  let f = Fixture::new();
  let mut s = f.load();
  let ExecutionResult::MultinomialLogit(r) = run(&mut s, "mlogit choice x, baseoutcome(1)") else {
    panic!()
  };
  assert_eq!(r.observation_count, 46);
  assert_eq!(r.category_counts, [7, 21, 18]);
  let sample = r.sample.as_ref().unwrap();
  assert_eq!(sample.total_observations, 50);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(sample.retained_indices, (1..47).collect::<Vec<_>>());
  close(r.log_likelihood, -42.0751828674327);
  close(r.covariance[0][2], 0.191374078890336);
  for (i, want) in [2.0_f64.ln(), 2.0_f64.ln(), 3.0_f64.ln(), -2.0_f64.ln()]
    .iter()
    .enumerate()
  {
    close(r.coefficients[i].estimate.value, *want);
  }
  assert_eq!(
    s.state().estimation.unwrap().family(),
    EstimationFamily::MultinomialLogit
  );
}

#[test]
fn prediction_uses_current_predictors_only_while_effects_use_retained_design() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "mlogit choice x");
  let before = run(&mut s, "estat margins");
  let cache = s.state().estimation;
  run(&mut s, "predict p, pr outcome(1)");
  run(&mut s, "predict base, xb outcome(1)");
  let observed = rows(&mut s);
  assert_eq!(observed.len(), 50);
  assert!(matches!(observed[0][3],CellValue::Float(v) if (v-1.0/6.0).abs()<1e-10));
  for index in [47, 48] {
    assert_eq!(observed[index][3], CellValue::Null);
    assert_eq!(observed[index][4], CellValue::Null);
  }
  assert!(matches!(observed[49][3],CellValue::Float(v) if (v-1.0/6.0).abs()<1e-10));
  for (i, row) in observed.iter().enumerate() {
    if ![47, 48].contains(&i) {
      assert_eq!(row[4], CellValue::Float(0.0));
    }
  }
  run(&mut s, "replace x = 2");
  run(&mut s, "replace choice = 99");
  run(&mut s, "predict updated, pr outcome(2)");
  let want = 8.0 / (1.0 + 8.0 + 0.75);
  for row in rows(&mut s) {
    let CellValue::Float(p) = row[5] else {
      panic!()
    };
    close(p, want);
  }
  assert_eq!(run(&mut s, "estat margins"), before);
  assert_eq!(s.state().estimation, cache);
  for command in [
    "predict p, pr outcome(1)",
    "predict missing_level, pr outcome(99)",
    "predict bare, pr",
    "predict residual, residuals",
  ] {
    let state = s.state();
    let data = rows(&mut s);
    let _ = fail(&mut s, command);
    assert_eq!(s.state(), state);
    assert_same_rows(&rows(&mut s), &data);
  }
}

#[test]
fn overflowing_current_indices_fail_without_publishing_or_losing_retained_inference() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "replace x = x/10");
  run(&mut s, "mlogit choice x");
  let effects = run(&mut s, "estat margins");
  run(
    &mut s,
    r#"sql """SELECT id,choice,CAST('1.7976931348623157e308' AS DOUBLE) AS x FROM active""" into inflated"#,
  );
  let state = s.state();
  let data = rows(&mut s);
  let error = fail(&mut s, "predict bad, pr outcome(2)");
  assert!(
    error.to_string().contains("nonfinite linear index"),
    "{error}"
  );
  assert_eq!(s.state(), state);
  assert_same_rows(&rows(&mut s), &data);
  assert_eq!(run(&mut s, "estat margins"), effects);
}

#[test]
fn failed_new_fits_clear_stale_inference_but_do_not_mutate_data() {
  let f = Fixture::new();
  for command in [
    "mlogit missing x",
    "mlogit choice missing",
    "mlogit choice x x",
    "mlogit choice x, baseoutcome(99)",
  ] {
    let mut s = f.load();
    run(&mut s, "mlogit choice x");
    let before = rows(&mut s);
    let metadata = s.state().active_dataset;
    assert!(matches!(
      fail(&mut s, command),
      RuntimeError::MlogitFailed { .. }
    ));
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, metadata);
    assert_same_rows(&rows(&mut s), &before);
  }
}

#[test]
fn exact_decimal_categories_are_validated_before_rounding_and_after_exclusion() {
  let f = Fixture::new();
  for value in [
    "1.00000000000000000001",
    "9007199254740992.00000000000000000000",
  ] {
    let mut s = f.load();
    run(
      &mut s,
      &format!(
        r#"sql """SELECT id, CASE WHEN id=1 THEN CAST('{value}' AS DECIMAL(38,20)) ELSE CAST(choice AS DECIMAL(38,20)) END AS choice, x FROM active WHERE id<49""" into precise"#
      ),
    );
    let before = rows(&mut s);
    let error = fail(&mut s, "mlogit choice x");
    assert!(error.to_string().contains("exact integer"), "{error}");
    assert_same_rows(&rows(&mut s), &before);
  }
  let mut s = f.load();
  run(
    &mut s,
    r#"sql """SELECT id, CASE WHEN choice=2 THEN CAST('9007199254740991' AS DECIMAL(38,20)) ELSE CAST(choice AS DECIMAL(38,20)) END AS choice, x FROM active WHERE id<49""" into precise"#,
  );
  let ExecutionResult::MultinomialLogit(r) =
    run(&mut s, "mlogit choice x, baseoutcome(9007199254740991)")
  else {
    panic!()
  };
  assert_eq!(r.category_levels, [1, 3, 9_007_199_254_740_991]);
  assert_eq!(r.base_outcome, 9_007_199_254_740_991);
  close(r.log_likelihood, -42.0751828674327);
}
