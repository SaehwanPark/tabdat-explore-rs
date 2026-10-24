use std::{
  collections::BTreeMap,
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
fn gold() -> BTreeMap<(String, usize, usize), f64> {
  gold_model("1")
}
fn gold_model(model: &str) -> BTreeMap<(String, usize, usize), f64> {
  include_str!("../../tabdat-stats/tests/fixtures/tobit_reference.csv")
    .lines()
    .skip(1)
    .filter_map(|line| {
      let v: Vec<_> = line.split(',').collect();
      (v[0] == model).then(|| {
        (
          (v[1].into(), v[2].parse().unwrap(), v[3].parse().unwrap()),
          v[4].parse().unwrap(),
        )
      })
    })
    .collect()
}
fn get(r: &BTreeMap<(String, usize, usize), f64>, q: &str, i: usize, j: usize) -> f64 {
  r[&(q.into(), i, j)]
}
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-tobit-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let r = gold();
    let mut csv = String::from("source_row,y,x1,x2,unused\n0,,0,0,\n");
    for i in 1..=36 {
      writeln!(
        csv,
        "{i},{},{},{},",
        get(&r, "input_y", i, 0),
        get(&r, "input_x", i, 1),
        get(&r, "input_x", i, 2)
      )
      .unwrap();
    }
    csv.push_str("37,0,,0,\n38,0,NaN,0,\n39,Inf,0,0,\n");
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
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
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() <= 1e-9 + 1e-8 * b.abs(), "{a} != {b}");
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(h) = run(s, "head 100") else {
    panic!("head");
  };
  h.rows
}
fn number(c: &CellValue) -> f64 {
  let CellValue::Float(v) = c else {
    panic!("DOUBLE: {c:?}");
  };
  *v
}
fn same_rows(a: &[Vec<CellValue>], b: &[Vec<CellValue>]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
      if let (CellValue::Float(a), CellValue::Float(b)) = (a, b)
        && a.is_nan()
        && b.is_nan()
      {
        continue;
      }
      assert_eq!(a, b);
    }
  }
}

#[test]
fn nonzero_bound_no_constant_and_below_limit_recording_match_independent_r() {
  let f = Fixture::new();
  let r = gold_model("2");
  let mut csv = String::from("y,x1,x2\n");
  for i in 1..=36 {
    writeln!(
      csv,
      "{},{},{}",
      get(&r, "input_y", i, 0),
      get(&r, "input_x", i, 1),
      get(&r, "input_x", i, 2)
    )
    .unwrap();
  }
  fs::write(f.0.join("data.csv"), csv).unwrap();
  let mut s = f.load();
  let original = rows(&mut s);
  let ExecutionResult::Tobit(fit) = run(&mut s, "tobit y x1 x2, ll(-0.8) noconstant") else {
    panic!("tobit");
  };
  assert!(!fit.include_intercept);
  assert_eq!(fit.lower_limit, -0.8);
  assert_eq!(fit.observation_count, 36);
  assert_eq!(fit.likelihood_ratio_df, 2);
  close(
    fit.null_log_likelihood,
    get(&r, "null_log_likelihood", 0, 0),
  );
  close(fit.likelihood_ratio, get(&r, "likelihood_ratio", 0, 0));
  close(
    fit.likelihood_ratio_p_value.unwrap(),
    get(&r, "likelihood_ratio_p_value", 0, 0),
  );
  close(fit.scale.value, get(&r, "sigma", 0, 0));
  for (i, c) in fit.coefficients.iter().enumerate() {
    close(c.value, get(&r, "coefficient_value", i + 1, 0));
    close(
      c.standard_error,
      get(&r, "coefficient_standard_error", i + 1, 0),
    );
  }
  for (i, row) in fit.covariance.iter().enumerate() {
    for (j, v) in row.iter().enumerate() {
      close(*v, get(&r, "covariance", i + 1, j + 1));
    }
  }
  same_rows(&rows(&mut s), &original);
  assert!(original.iter().any(|row| number(&row[0]) < -0.8));
  let ExecutionResult::TobitMarginalEffects(effects) = run(&mut s, "estat margins") else {
    panic!("effects");
  };
  for (i, e) in effects.effects.iter().enumerate() {
    close(e.value, get(&r, "ame_value", i + 1, 0));
    close(e.standard_error, get(&r, "ame_standard_error", i + 1, 0));
  }
  for c in [
    "predict latent, xb",
    "predict probability, pr",
    "predict observed, mean",
  ] {
    run(&mut s, c);
  }
  for (i, row) in rows(&mut s).iter().enumerate() {
    for (q, col) in [("latent", 3), ("probability", 4), ("mean", 5)] {
      close(number(&row[col]), get(&r, q, i + 1, 0));
    }
  }
}

#[test]
fn numerical_prediction_failure_is_atomic_and_numerical_refit_clears_stale_state() {
  let f = Fixture::new();
  let mut s = f.load();
  let ExecutionResult::Tobit(fit) = run(&mut s, "tobit y x1 x2, ll(0)") else {
    panic!("tobit");
  };
  assert!(
    fit
      .coefficients
      .iter()
      .skip(1)
      .map(|c| c.value.abs())
      .sum::<f64>()
      > 1.1
  );
  for c in fit.coefficients.iter().skip(1) {
    run(&mut s, &format!("replace {} = {:.0}", c.name, 1.7e308_f64));
    if c.value < 0.0 {
      run(&mut s, &format!("replace {} = 0 - {}", c.name, c.name));
    }
  }
  let state = s.state();
  let before = rows(&mut s);
  assert_eq!(number(&before[1][2]).abs(), 1.7e308_f64);
  assert_eq!(number(&before[1][3]).abs(), 1.7e308_f64);
  for kind in ["xb", "pr", "mean"] {
    assert!(matches!(
      fail(&mut s, &format!("predict bad, {kind}")),
      RuntimeError::TobitFailed(_)
    ));
    assert_eq!(s.state(), state);
    same_rows(&rows(&mut s), &before);
  }
  let mut s = f.load();
  run(&mut s, "tobit y x1 x2, ll(0)");
  run(&mut s, "replace y = 0");
  let before = rows(&mut s);
  let state = s.state();
  assert!(matches!(
    fail(&mut s, "tobit y x1 x2, ll(0)"),
    RuntimeError::TobitFailed(_)
  ));
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, state.active_dataset);
  assert_eq!(s.state().labels, state.labels);
  same_rows(&rows(&mut s), &before);
}

#[test]
fn gaussian_tobit_and_mean_require_a_successful_owned_fit() {
  let mut s = Session::new();
  assert_eq!(
    fail(&mut s, "tobit y x, ll(0)"),
    RuntimeError::NoActiveDataset { command: "tobit" }
  );
  assert_eq!(
    fail(&mut s, "predict expected, mean"),
    RuntimeError::TobitFailed(
      "predict mean requires a prior successful lower-censored Gaussian Tobit fit".into()
    )
  );
  assert!(s.initialized_capabilities().is_empty());
  assert!(s.state().estimation.is_none());
}

#[test]
fn joint_sample_observed_covariance_and_retained_effects_match_independent_r() {
  let f = Fixture::new();
  let mut s = f.load();
  let r = gold();
  run(&mut s, "label variable x1 \"first predictor\"");
  let before = s.state();
  let ExecutionResult::Tobit(fit) = run(&mut s, "tobit y x1 x2, ll(0)") else {
    panic!("tobit");
  };
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::TobitFit(_))
  ));
  assert_eq!(
    s.state().estimation.unwrap().family(),
    EstimationFamily::Tobit
  );
  assert!(s.last_regression().is_none());
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (1..=36).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 40);
  assert_eq!(sample.dropped_observations, 4);
  close(fit.log_likelihood, get(&r, "log_likelihood", 0, 0));
  close(fit.scale.value, get(&r, "sigma", 0, 0));
  for (i, row) in fit.covariance.iter().enumerate() {
    for (j, v) in row.iter().enumerate() {
      close(*v, get(&r, "covariance", i + 1, j + 1));
    }
  }
  let margins = run(&mut s, "estat margins");
  let ExecutionResult::TobitMarginalEffects(ref effects) = margins else {
    panic!("margins");
  };
  for (i, e) in effects.effects.iter().enumerate() {
    close(e.value, get(&r, "ame_value", i + 1, 0));
    close(e.standard_error, get(&r, "ame_standard_error", i + 1, 0));
  }
  let owned = s.state().estimation;
  run(&mut s, "replace x1 = x1 + 10");
  run(&mut s, "replace y = -100");
  assert_eq!(run(&mut s, "estat margins"), margins);
  assert_eq!(s.state().estimation, owned);
  run(&mut s, "drop x1");
  let state = s.state();
  assert!(matches!(
    fail(&mut s, "predict bad, mean"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), state);
  assert_eq!(run(&mut s, "estat margins"), margins);
}

#[test]
fn all_three_current_predictions_preserve_order_labels_and_owned_fit() {
  let f = Fixture::new();
  let mut s = f.load();
  let r = gold();
  run(&mut s, "label variable x1 \"first predictor\"");
  run(&mut s, "tobit y x1 x2, ll(0)");
  let owned = s.state().estimation;
  let labels = s.state().labels;
  for command in [
    "predict latent",
    "predict probability, pr",
    "predict expected, mean",
  ] {
    run(&mut s, command);
    assert_eq!(s.state().estimation, owned);
    assert_eq!(s.state().labels, labels);
  }
  let data = rows(&mut s);
  assert_eq!(data.len(), 40);
  for (i, row) in data.iter().enumerate().take(37).skip(1) {
    for (q, col) in [("latent", 5), ("probability", 6), ("mean", 7)] {
      close(number(&row[col]), get(&r, q, i, 0));
    }
  }
  for row in [0, 39] {
    assert!(matches!(data[row][5], CellValue::Float(_)));
    assert!(matches!(data[row][6], CellValue::Float(_)));
    assert!(matches!(data[row][7], CellValue::Float(_)));
  }
  for row in [37, 38] {
    assert_eq!(
      data[row][5..8],
      [CellValue::Null, CellValue::Null, CellValue::Null]
    );
  }
  // Responses/unused columns are unnecessary at prediction; the original finite
  // sample is retained for effects even when current outcomes disappear.
  run(&mut s, "drop y unused");
  run(&mut s, "predict after_drop, mean");
  assert_eq!(s.state().estimation, owned);
}

#[test]
fn prediction_failures_are_atomic_and_failed_new_fits_clear_stale_inference() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "tobit y x1 x2, ll(0)");
  for command in [
    "predict x1, mean",
    "predict bad, residuals",
    "predict bad, n",
  ] {
    let state = s.state();
    let data = rows(&mut s);
    let _ = fail(&mut s, command);
    assert_eq!(s.state(), state);
    same_rows(&rows(&mut s), &data);
  }
  run(&mut s, "drop x1");
  run(&mut s, "rename unused x1");
  let state = s.state();
  let data = rows(&mut s);
  assert!(matches!(
    fail(&mut s, "predict bad, mean"),
    RuntimeError::TobitFailed(_)
  ));
  assert_eq!(s.state(), state);
  same_rows(&rows(&mut s), &data);
  // Reject oversized designs before querying/allocating their row matrix.
  let oversized = format!("tobit y {}, ll(0)", vec!["x2"; 33].join(" "));
  assert!(
    fail(&mut s, &oversized)
      .to_string()
      .contains("1..32 location parameters")
  );
  for options in ["robust", "cluster(source_row)", "ul(10)"] {
    let before = s.state();
    let data = rows(&mut s);
    assert!(matches!(
      fail(&mut s, &format!("tobit y x2, ll(0) {options}")),
      RuntimeError::TobitFailed(_)
    ));
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    same_rows(&rows(&mut s), &data);
  }
  assert_eq!(
    fail(&mut s, "predict bad, mean"),
    RuntimeError::TobitFailed(
      "predict mean requires a prior successful lower-censored Gaussian Tobit fit".into()
    )
  );
}
