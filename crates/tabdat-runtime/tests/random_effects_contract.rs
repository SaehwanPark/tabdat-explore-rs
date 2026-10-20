use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationState, ExecutionResult, RandomEffectsState};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-re-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,year,y,x\n");
    for t in (0..4).rev() {
      for id in (1..=7).rev() {
        let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
        let y = 1.0 + 0.4 * f64::from(id) + 0.7 * x + 0.11 * f64::from((id * t + 3) % 7);
        // Exclude one row per usable unit and all rows from unit seven.
        let y = if id == 7 || t == 3 {
          String::new()
        } else {
          y.to_string()
        };
        csv.push_str(&format!("{id},{},{y},{x}\n", 1979 + t));
      }
    }
    fs::write(path.join("data.csv"), csv).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
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
  s.execute(parse_command(text).unwrap())
    .unwrap_or_else(|e| panic!("{text}: {e}"))
}
fn rows(s: &mut Session, query: &str) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Table(t) = run(s, &format!("sql \"\"\"{query}\"\"\"")) else {
    panic!("wrong result")
  };
  t.rows
}
fn number(c: &CellValue) -> Option<f64> {
  match c {
    CellValue::Float(v) => Some(*v),
    CellValue::SignedInteger(v) => Some(*v as f64),
    CellValue::UnsignedInteger(v) => Some(*v as f64),
    CellValue::Null => None,
    _ => panic!("{c:?}"),
  }
}
fn fit(s: &mut Session) -> RandomEffectsState {
  let ExecutionResult::RandomEffects(r) = run(s, "xtreg y x, re id(id) time(year)") else {
    panic!("wrong result")
  };
  *r
}

#[test]
fn joint_sample_balanced_counts_predictions_and_retained_state_are_explicit() {
  let f = Fixture::new();
  let mut s = f.load();
  let data = rows(&mut s, "SELECT * FROM active");
  let r = fit(&mut s);
  assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  let sample = r.result.gls.sample.as_ref().unwrap();
  assert_eq!(sample.total_observations, 28);
  assert_eq!(sample.dropped_observations, 10);
  assert_eq!(
    sample.retained_indices,
    (7..28).filter(|i| i % 7 != 0).collect::<Vec<_>>()
  );
  assert_eq!(r.result.group_count, 6);
  assert_eq!(r.result.group_size, 3);
  assert_eq!(r.result.gls.fit_statistics.degrees_of_freedom, 16);
  run(&mut s, "predict xb");
  run(&mut s, "predict residual, residuals");
  let b = &r.result.gls.coefficients;
  for row in rows(&mut s, "SELECT y,x,xb,residual FROM active") {
    let expected = b[0].value + b[1].value * number(&row[1]).unwrap();
    assert!((number(&row[2]).unwrap() - expected).abs() < 1e-12);
    if let Some(y) = number(&row[0]) {
      assert!((number(&row[3]).unwrap() - (y - expected)).abs() < 1e-12);
    } else {
      assert_eq!(row[3], CellValue::Null);
    }
  }
  run(&mut s, "replace x = x + 1");
  run(&mut s, "drop id year");
  run(&mut s, "predict current, xb");
  for row in rows(&mut s, "SELECT xb,current FROM active") {
    assert!((number(&row[1]).unwrap() - number(&row[0]).unwrap() - b[1].value).abs() < 1e-12);
  }
  let Some(EstimationState::RandomEffects(retained)) = s.state().estimation else {
    panic!("lost fit")
  };
  assert_eq!(*retained, r);
}

#[test]
fn prediction_and_restriction_failures_are_atomic_and_new_fit_failures_clear_only_fit() {
  let f = Fixture::new();
  let mut s = f.load();
  fit(&mut s);
  for text in [
    "predict y, xb",
    "predict bad, xbu",
    "predict bad, u",
    "predict bad, pr",
    "predict bad, posterior_predictive interval",
    "lincom x",
    "test x = 0",
  ] {
    let before = s.state();
    let data = rows(&mut s, "SELECT * FROM active");
    assert!(s.execute(parse_command(text).unwrap()).is_err(), "{text}");
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  }
  let before = s.state();
  assert!(parse_command("xtreg y x, re id(id)").is_err());
  assert_eq!(s.state(), before);
  for options in ["cluster(year)", "cluster(unknown)"] {
    fit(&mut s);
    let mut expected = s.state();
    expected.estimation = None;
    let data = rows(&mut s, "SELECT * FROM active");
    assert!(
      s.execute(parse_command(&format!("xtreg y x, re id(id) time(year) {options}")).unwrap())
        .is_err()
    );
    assert_eq!(s.state(), expected);
    assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  }
  fit(&mut s);
  run(&mut s, "replace x = NULL");
  run(&mut s, "predict nulls, xb");
  assert!(
    rows(&mut s, "SELECT nulls FROM active")
      .iter()
      .all(|r| r[0] == CellValue::Null)
  );
}

#[test]
fn raw_key_errors_and_unbalanced_retained_samples_stop_without_mutating_data() {
  for change in [
    "replace year = year + 0.5",
    "replace year = 1979",
    "replace id = NULL",
    "replace y = NULL if id == 1",
  ] {
    let f = Fixture::new();
    let mut s = f.load();
    fit(&mut s);
    run(&mut s, change);
    // Removing every case of one unit is allowed if the remaining units balance.
    if change == "replace y = NULL if id == 1" {
      let r = fit(&mut s);
      assert_eq!(r.result.group_count, 5);
      continue;
    }
    let data = rows(&mut s, "SELECT * FROM active");
    assert!(
      s.execute(parse_command("xtreg y x, re id(id) time(year)").unwrap())
        .is_err()
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
  }
  let f = Fixture::new();
  let mut s = f.load();
  fit(&mut s);
  run(
    &mut s,
    "sql \"\"\"SELECT * FROM active WHERE NOT (id=1 AND year=1979)\"\"\" into unbalanced",
  );
  let data = rows(&mut s, "SELECT * FROM active");
  let error = s
    .execute(parse_command("xtreg y x, re id(id) time(year)").unwrap())
    .unwrap_err();
  assert!(
    error.to_string().contains("equal retained unit counts"),
    "{error}"
  );
  assert!(s.state().estimation.is_none());
  assert_eq!(rows(&mut s, "SELECT * FROM active"), data);
}

#[test]
fn robust_alias_equals_unit_clustering_with_unchanged_sample_and_predictions() {
  let f = Fixture::new();
  let mut s = f.load();
  let classical = fit(&mut s);
  let ExecutionResult::RandomEffects(cluster) =
    run(&mut s, "xtreg y x, re id(id) time(year) cluster(id)")
  else {
    panic!("wrong result")
  };
  run(&mut s, "predict cluster_xb, xb");
  let ExecutionResult::RandomEffects(robust) =
    run(&mut s, "xtreg y x, re id(id) time(year) robust")
  else {
    panic!("wrong result")
  };
  assert_eq!(cluster, robust);
  assert_eq!(cluster.result.theta, classical.result.theta);
  assert_eq!(cluster.result.gls.sample, classical.result.gls.sample);
  assert_eq!(
    cluster.result.gls.coefficients[1].value,
    classical.result.gls.coefficients[1].value
  );
  assert_ne!(
    cluster.result.gls.covariance.matrix,
    classical.result.gls.covariance.matrix
  );
  run(&mut s, "predict robust_xb, xb");
  for row in rows(&mut s, "SELECT cluster_xb,robust_xb FROM active") {
    assert_eq!(row[0], row[1]);
  }
  let before = s.state();
  assert!(
    s.execute(parse_command("predict bad, xbu").unwrap())
      .is_err()
  );
  assert_eq!(s.state(), before);
}
