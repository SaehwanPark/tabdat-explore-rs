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
const NAMES: &str = include_str!("../../../tests/fixtures/gompertz/parameter_names.txt");
const MODEL: &str = include_str!("../../../tests/fixtures/gompertz/model_based.csv");
const ROBUST: &str = include_str!("../../../tests/fixtures/gompertz/robust.csv");
const NO_MODEL: &str =
  include_str!("../../../tests/fixtures/gompertz/no_intercept/model_based.csv");
const NO_ROBUST: &str = include_str!("../../../tests/fixtures/gompertz/no_intercept/robust.csv");
const POINTS: &str = include_str!("../../../tests/fixtures/gompertz/predictions.csv");
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let dir = std::env::temp_dir().join(format!(
      "tabdat-gompertz-runtime-{}-{}",
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
    .filter(|s| *s != "gamma")
    .collect::<Vec<_>>()
    .join(" ");
  format!(
    "streg spell {names}, failure(censor1) dist(gompertz) {} {}",
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
fn gompertz_joint_missingness_original_positions_and_full_inference_cross_runtime_boundary() {
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
    let ExecutionResult::Gompertz(r) = run(&mut s, &fit_command(robust, no_intercept)) else {
      panic!("Gompertz")
    };
    let g = gold(reference);
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
    let data = serde_json::to_value(&r).unwrap();
    for (path, expected) in &g {
      let value = data
        .pointer(path)
        .unwrap_or_else(|| panic!("missing {path}"))
        .as_f64()
        .unwrap();
      if path.ends_with("p_value") && *expected > 0.0 {
        assert!(
          (value - expected).abs() <= 1e-300 + 1e-8 * expected,
          "{path}: {value} != {expected}"
        );
      } else {
        close(value, *expected);
      }
    }
    assert_eq!(
      g.len(),
      6 * r.coefficients.len()
        + (r.coefficients.len() + 1).pow(2)
        + 6
        + if robust { 13 } else { 14 }
    );
    assert_eq!(r.likelihood_ratio_p_value.is_none(), robust);
    assert_eq!(
      s.state().estimation.as_ref().unwrap().family(),
      EstimationFamily::Survival
    );
    assert!(matches!(
      s.state().estimation,
      Some(EstimationState::GompertzFit(_))
    ));
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}

#[test]
fn gompertz_current_predictions_keep_labels_fit_order_and_nulls_without_event_column() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "label variable reprate \"replacement rate\"");
  let ExecutionResult::Gompertz(r) = run(&mut s, &fit_command(true, false)) else {
    panic!("Gompertz")
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
fn gompertz_prediction_failures_are_atomic_and_probability_overflow_is_not_count_overflow() {
  let f = Fixture::new();
  let mut s = f.load();
  let ExecutionResult::Gompertz(r) = run(&mut s, &fit_command(true, false)) else {
    panic!("Gompertz")
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
  run(&mut s, "replace spell = -1 if source_row == 400");
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
fn gompertz_original_decimal_failure_codes_are_checked_after_joint_eligibility() {
  let f = Fixture::new();
  for (value, eligible, valid) in [
    ("1", true, true),
    ("1.000000000000000001", true, false),
    ("0.999999999999999999", true, false),
    ("1.000000000000000001", false, true),
    ("0.999999999999999999", false, true),
  ] {
    let mut s = f.load();
    let condition = if eligible {
      "censor1=1"
    } else {
      "source_row=0"
    };
    run(
      &mut s,
      &format!(
        "sql \"\"\"SELECT * EXCLUDE(censor1),CASE WHEN {condition} THEN CAST('{value}' AS DECIMAL(38,18)) ELSE CAST(censor1 AS DECIMAL(38,18)) END AS censor1 FROM active\"\"\" into decimal_events"
      ),
    );
    run(&mut s, "label variable reprate \"replacement rate\"");
    // An invalid new fit must not leave earlier linear inference available.
    run(&mut s, "regress logwage reprate");
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let result = s.execute(parse_command(&fit_command(true, false)).unwrap());
    assert_eq!(
      result.is_ok(),
      valid,
      "value={value}, eligible={eligible}, error={:?}",
      result.as_ref().err()
    );
    if valid {
      let ExecutionResult::Gompertz(r) = result.unwrap() else {
        panic!("Gompertz")
      };
      assert_eq!(r.observation_count, 400);
      assert_eq!(r.failure_count, 120);
      assert_eq!(
        r.sample.as_ref().unwrap().retained_indices,
        (1..=400).collect::<Vec<_>>()
      );
      let expected = gold(ROBUST);
      close(r.log_likelihood, expected["/log_likelihood"]);
      close(r.shape.value, expected["/shape/value"]);
      for (i, c) in r.coefficients.iter().enumerate() {
        close(
          c.value,
          expected[format!("/coefficients/{i}/value").as_str()],
        );
        close(
          c.standard_error,
          expected[format!("/coefficients/{i}/standard_error").as_str()],
        );
      }
    } else {
      assert!(result.unwrap_err().to_string().contains("exactly0 or1"));
      assert!(s.state().estimation.is_none());
    }
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}

#[test]
fn gompertz_failed_fit_clears_only_inference_and_new_modes_never_score_another_family() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "regress logwage reprate");
  let linear = s.state();
  assert!(matches!(
    fail(&mut s, "predict wrong, survival"),
    RuntimeError::StregFailed(_)
  ));
  assert_eq!(s.state(), linear);
  for suffix in ["dist(exponential)", "dist(gompertz) cluster(source_row)"] {
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

#[test]
fn gompertz_current_time_zero_is_defined_and_parse_failures_preserve_the_fit() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, &fit_command(true, false));
  let before = s.state();
  assert!(parse_command("streg spell reprate, failure(censor1) dist(gompertzz)").is_err());
  assert_eq!(s.state(), before);
  run(&mut s, "drop censor1");
  run(&mut s, "replace spell = 0 if source_row == 400");
  let owned = s.state().estimation;
  for (target, kind) in [
    ("eta_zero", "xb"),
    ("h_zero", "hazard"),
    ("cum_zero", "cumhaz"),
    ("S_zero", "survival"),
  ] {
    run(&mut s, &format!("predict {target}, {kind}"));
    assert_eq!(s.state().estimation, owned);
  }
  let meta = s.state().active_dataset.unwrap();
  let data = rows(&mut s);
  let at =
    |name: &str| numeric(&data[400][meta.columns.iter().position(|c| c.name == name).unwrap()]);
  close(at("h_zero"), at("eta_zero").exp());
  assert_eq!(at("cum_zero"), 0.0);
  assert_eq!(at("S_zero"), 1.0);
}

#[test]
fn gompertz_negative_gamma_is_not_clipped_by_runtime_and_retains_positive_tail() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "replace spell = 0.1 + spell/100 if censor1 == 1");
  let ExecutionResult::Gompertz(r) = run(&mut s, &fit_command(true, false)) else {
    panic!("gompertz")
  };
  assert!(r.shape.value < 0.0);
  let (j, eta) = r
    .fitted_xb
    .iter()
    .enumerate()
    .min_by(|a, b| a.1.total_cmp(b.1))
    .unwrap();
  let limit = (-eta.exp() / (-r.shape.value)).exp();
  assert!(limit > 0.0 && limit < 1.0);
  run(&mut s, "drop censor1");
  run(&mut s, "replace spell = 1000000");
  let owned = s.state().estimation;
  run(&mut s, "predict negative_tail, survival");
  assert_eq!(s.state().estimation, owned);
  let meta = s.state().active_dataset.unwrap();
  let col = meta
    .columns
    .iter()
    .position(|c| c.name == "negative_tail")
    .unwrap();
  close(numeric(&rows(&mut s)[j + 1][col]), limit);
}
