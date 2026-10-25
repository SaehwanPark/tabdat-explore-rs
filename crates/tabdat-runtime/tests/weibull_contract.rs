use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
const INPUT: &str = include_str!("../../../tests/fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("../../../tests/fixtures/weibull/parameter_names.txt");
const MODEL: &str = include_str!("../../../tests/fixtures/weibull/model_based.csv");
const ROBUST: &str = include_str!("../../../tests/fixtures/weibull/robust.csv");
const NO_MODEL: &str = include_str!("../../../tests/fixtures/weibull/no_intercept/model_based.csv");
const NO_ROBUST: &str = include_str!("../../../tests/fixtures/weibull/no_intercept/robust.csv");
const POINTS: &str = include_str!("../../../tests/fixtures/weibull/predictions.csv");
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let dir = std::env::temp_dir().join(format!(
      "tabdat-weibull-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let mut lines = INPUT.lines();
    let header = lines.next().unwrap();
    let names: Vec<_> = header.split(',').collect();
    let base: Vec<String> = lines
      .next()
      .unwrap()
      .split(',')
      .map(str::to_owned)
      .collect();
    let mut rows = Vec::new();
    for changes in [
      vec![("source_row", "0"), ("spell", ""), ("censor1", "2")],
      vec![("source_row", "401"), ("spell", "0"), ("reprate", "")],
      vec![("source_row", "402"), ("spell", "0"), ("age", "NaN")],
      vec![("source_row", "403"), ("spell", "Inf")],
    ] {
      let mut row = base.clone();
      for (name, value) in changes {
        row[names.iter().position(|n| *n == name).unwrap()] = value.into();
      }
      rows.push(row.join(","));
    }
    let csv = format!(
      "{header}\n{}\n{}{}\n",
      rows[0],
      INPUT
        .lines()
        .skip(1)
        .map(|l| format!("{l}\n"))
        .collect::<String>(),
      rows[1..].join("\n")
    );
    fs::write(dir.join("input.csv"), csv).unwrap();
    Self(dir)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("input.csv").display()),
    );
    run(&mut s, "generate rr_ui = reprate * ui");
    run(&mut s, "generate dr_ui = disrate * ui");
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
fn fit_command(robust: bool, no_intercept: bool) -> String {
  let names = NAMES
    .lines()
    .skip(1)
    .filter(|s| *s != "lnshape")
    .collect::<Vec<_>>()
    .join(" ");
  format!(
    "streg spell {names}, failure(censor1) dist(weibull) {} {}",
    if robust { "robust" } else { "" },
    if no_intercept { "noconstant" } else { "" }
  )
}
fn gold(text: &str) -> BTreeMap<&str, f64> {
  text
    .lines()
    .skip(1)
    .map(|l| {
      let (p, v) = l.split_once(',').unwrap();
      (p, v.parse().unwrap())
    })
    .collect()
}
fn close(a: f64, b: f64) {
  assert!(
    (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.17e} != {b:.17e}"
  );
}
fn numeric(cell: &CellValue) -> f64 {
  let CellValue::Float(v) = cell else {
    panic!("DOUBLE expected {cell:?}")
  };
  *v
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(h) = run(s, "head 1000") else {
    panic!("head")
  };
  h.rows
}

#[test]
fn weibull_joint_missingness_original_positions_and_full_inference_cross_runtime_boundary() {
  let f = Fixture::new();
  for (robust, no_intercept, reference) in [
    (false, false, MODEL),
    (true, false, ROBUST),
    (false, true, NO_MODEL),
    (true, true, NO_ROBUST),
  ] {
    let mut s = f.load();
    run(&mut s, "label variable reprate \"replacement rate\"");
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let ExecutionResult::Weibull(r) = run(&mut s, &fit_command(robust, no_intercept)) else {
      panic!("Weibull")
    };
    let g = gold(reference);
    let k = r.coefficients.len();
    assert_eq!(r.observation_count, 400);
    assert_eq!(r.failure_count as f64, g["/failure_count"]);
    assert_eq!(
      r.sample.as_ref().unwrap().retained_indices,
      (1..=400).collect::<Vec<_>>()
    );
    assert_eq!(r.sample.as_ref().unwrap().total_observations, 404);
    assert_eq!(r.sample.as_ref().unwrap().dropped_observations, 4);
    assert_eq!(r.include_intercept, !no_intercept);
    assert_eq!(r.robust, robust);
    for (i, c) in r.coefficients.iter().enumerate() {
      for (name, value) in [
        ("value", c.value),
        ("standard_error", c.standard_error),
        ("statistic", c.statistic.unwrap()),
        ("p_value", c.p_value.unwrap()),
        ("confidence_lower", c.confidence_lower.unwrap()),
        ("confidence_upper", c.confidence_upper.unwrap()),
      ] {
        let key = format!("/coefficients/{i}/{name}");
        let expected = g[key.as_str()];
        if name == "p_value" && expected > 0.0 {
          assert!((value - expected).abs() <= 1e-300 + 1e-8 * expected);
        } else {
          close(value, expected);
        }
      }
    }
    for i in 0..=k {
      for j in 0..=k {
        close(
          r.covariance_matrix[i][j],
          g[format!("/covariance_matrix/{i}/{j}").as_str()],
        );
      }
    }
    close(r.shape.value, g["/shape/value"]);
    close(r.shape.log_standard_error, g["/shape/log_standard_error"]);
    close(r.log_likelihood, g["/log_likelihood"]);
    close(r.normalized_log_likelihood, g["/normalized_log_likelihood"]);
    close(r.null_log_likelihood, g["/null_log_likelihood"]);
    close(r.likelihood_ratio, g["/likelihood_ratio"]);
    assert_eq!(r.likelihood_ratio_p_value.is_none(), robust);
    assert_eq!(
      s.state().estimation.as_ref().unwrap().family(),
      EstimationFamily::Survival
    );
    assert!(matches!(
      s.state().estimation,
      Some(EstimationState::WeibullFit(_))
    ));
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}

#[test]
fn weibull_current_predictions_keep_labels_fit_order_and_nulls_without_event_column() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "label variable reprate \"replacement rate\"");
  let ExecutionResult::Weibull(r) = run(&mut s, &fit_command(true, false)) else {
    panic!("Weibull")
  };
  let owned = s.state().estimation;
  let labels = s.state().labels;
  run(&mut s, "drop censor1");
  for (target, kind) in [
    ("eta", "xb"),
    ("hazard", "hazard"),
    ("cumhaz", "cumhaz"),
    ("survival", "survival"),
  ] {
    run(&mut s, &format!("predict {target}, {kind}"));
    assert_eq!(s.state().estimation, owned);
    assert_eq!(s.state().labels, labels);
  }
  let data = rows(&mut s);
  assert_eq!(data.len(), 404);
  let mut reference = POINTS.lines();
  let headers: Vec<_> = reference.next().unwrap().split(',').collect();
  let meta = s.state().active_dataset.unwrap();
  for (i, line) in reference.enumerate() {
    let expected: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
    for (target, name) in [
      ("eta", "xb"),
      ("hazard", "hazard"),
      ("cumhaz", "cumulative_hazard"),
      ("survival", "net_survival"),
    ] {
      let column = meta.columns.iter().position(|c| c.name == target).unwrap();
      close(
        numeric(&data[i + 1][column]),
        expected[headers.iter().position(|h| *h == name).unwrap()],
      );
    }
  }
  for i in [0, 401, 402, 403] {
    for name in ["hazard", "cumhaz", "survival"] {
      let col = meta.columns.iter().position(|c| c.name == name).unwrap();
      assert_eq!(data[i][col], CellValue::Null);
    }
  }
  // Time/event are not required for xb; scoring reads current X, not training predictions.
  let b = r
    .coefficients
    .iter()
    .find(|c| c.name == "logwage")
    .unwrap()
    .value;
  run(&mut s, "replace logwage = logwage + 0.5 if source_row == 1");
  run(&mut s, "drop spell");
  run(&mut s, "predict changed, xb");
  let meta = s.state().active_dataset.unwrap();
  let changed = meta
    .columns
    .iter()
    .position(|c| c.name == "changed")
    .unwrap();
  let eta = meta.columns.iter().position(|c| c.name == "eta").unwrap();
  let after = rows(&mut s);
  close(
    numeric(&after[1][changed]),
    numeric(&after[1][eta]) + 0.5 * b,
  );
  assert_eq!(s.state().estimation, owned);
  let before = s.state();
  assert!(matches!(
    fail(&mut s, "predict unavailable, survival"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
}

#[test]
fn weibull_prediction_failures_are_atomic_and_probability_overflow_is_not_count_overflow() {
  let f = Fixture::new();
  let mut s = f.load();
  let ExecutionResult::Weibull(r) = run(&mut s, &fit_command(true, false)) else {
    panic!("Weibull")
  };
  for command in [
    "predict reprate, hazard",
    "predict wrong, pr",
    "predict wrong, mean",
    "predict wrong, n",
    "predict wrong, residuals",
    "predict wrong, pmf(0)",
  ] {
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let _ = fail(&mut s, command);
    assert_eq!(s.state(), before);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
  run(&mut s, "replace spell = 0 if source_row == 400");
  let before = s.state();
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, "predict bad, hazard"),
    RuntimeError::StregFailed(_)
  ));
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "replace spell = 1 if source_row == 400");
  let b = r
    .coefficients
    .iter()
    .find(|c| c.name == "logwage")
    .unwrap()
    .value;
  run(
    &mut s,
    &format!("replace logwage = {} if source_row == 400", 10000.0 / b),
  );
  let before = s.state();
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, "predict bad, cumhaz"),
    RuntimeError::StregFailed(_)
  ));
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "predict tail, survival");
  let meta = s.state().active_dataset.unwrap();
  let col = meta.columns.iter().position(|c| c.name == "tail").unwrap();
  assert_eq!(rows(&mut s)[400][col], CellValue::Float(0.0));
  run(&mut s, "drop reprate");
  let before = s.state();
  assert!(matches!(
    fail(&mut s, "predict absent, xb"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
}

#[test]
fn weibull_failed_fit_clears_only_inference_and_new_modes_never_score_another_family() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "regress logwage reprate");
  let linear = s.state();
  assert!(matches!(
    fail(&mut s, "predict wrong, survival"),
    RuntimeError::StregFailed(_)
  ));
  assert_eq!(s.state(), linear);
  for suffix in ["dist(exponential)", "dist(weibull) cluster(source_row)"] {
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    assert!(matches!(
      fail(
        &mut s,
        &format!("streg spell reprate, failure(censor1) {suffix}")
      ),
      RuntimeError::StregFailed(_)
    ));
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
  run(&mut s, &fit_command(true, false));
  run(&mut s, "replace censor1 = 2 if source_row == 400");
  let before = s.state();
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, &fit_command(true, false)),
    RuntimeError::StregFailed(_)
  ));
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert_eq!(run(&mut s, "datasignature"), signature);
}
