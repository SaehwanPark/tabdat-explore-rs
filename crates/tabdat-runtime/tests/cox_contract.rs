use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
const INPUT: &str = include_str!("../../../tests/fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("../../../tests/fixtures/cox/parameter_names.txt");
const MODEL: &str = include_str!("../../../tests/fixtures/cox/model_based.csv");
const ROBUST: &str = include_str!("../../../tests/fixtures/cox/robust.csv");
const POINTS: &str = include_str!("../../../tests/fixtures/cox/predictions.csv");
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-cox-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut lines = INPUT.lines();
    let header = lines.next().unwrap();
    let names: Vec<_> = header.split(',').collect();
    let base: Vec<String> = lines
      .next()
      .unwrap()
      .split(',')
      .map(str::to_owned)
      .collect();
    let mut extras = Vec::new();
    for changes in [
      vec![("source_row", "0"), ("spell", ""), ("censor1", "2")],
      vec![("source_row", "401"), ("spell", "-1"), ("reprate", "")],
      vec![("source_row", "402"), ("spell", "0"), ("age", "NaN")],
      vec![("source_row", "403"), ("spell", "Inf")],
    ] {
      let mut row = base.clone();
      for (name, value) in changes {
        row[names.iter().position(|n| *n == name).unwrap()] = value.into();
      }
      extras.push(row.join(","));
    }
    fs::write(
      path.join("input.csv"),
      format!(
        "{header}\n{}\n{}{}\n",
        extras[0],
        INPUT
          .lines()
          .skip(1)
          .map(|l| format!("{l}\n"))
          .collect::<String>(),
        extras[1..].join("\n")
      ),
    )
    .unwrap();
    Self(path)
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
fn command(robust: bool) -> String {
  format!(
    "stcox spell {}, failure(censor1) {}",
    NAMES.lines().collect::<Vec<_>>().join(" "),
    if robust { "robust" } else { "" }
  )
}
fn numeric(c: &CellValue) -> f64 {
  let CellValue::Float(v) = c else {
    panic!("DOUBLE {c:?}")
  };
  *v
}
fn close(a: f64, b: f64) {
  assert!(
    (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.17e} != {b:.17e}"
  );
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(h) = run(s, "head 1000") else {
    panic!("head")
  };
  h.rows
}
fn index(s: &Session, name: &str) -> usize {
  s.active_dataset()
    .unwrap()
    .columns
    .iter()
    .position(|c| c.name == name)
    .unwrap()
}

#[test]
fn cox_joint_sample_owned_baseline_and_all_r_inference_cross_runtime_boundary() {
  let f = Fixture::new();
  for (robust, reference) in [(false, MODEL), (true, ROBUST)] {
    let mut s = f.load();
    run(&mut s, "label variable reprate \"replacement rate\"");
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let ExecutionResult::Cox(r) = run(&mut s, &command(robust)) else {
      panic!("Cox")
    };
    assert_eq!(r.observation_count, 400);
    let sample = r.sample.as_ref().unwrap();
    assert_eq!(sample.retained_indices, (1..=400).collect::<Vec<_>>());
    assert_eq!(
      (sample.total_observations, sample.dropped_observations),
      (404, 4)
    );
    let data = serde_json::to_value(&r).unwrap();
    let mut count = 0;
    for line in reference.lines().skip(1) {
      let (path, expected) = line.split_once(',').unwrap();
      let expected: f64 = expected.parse().unwrap();
      let actual = data.pointer(path).unwrap().as_f64().unwrap();
      if path.ends_with("p_value") && expected > 0.0 {
        assert!(
          (actual - expected).abs() <= 1e-300 + 1e-8 * expected,
          "{path}"
        );
      } else {
        close(actual, expected);
      }
      count += 1;
    }
    assert_eq!(
      count,
      7 * 40 + 1600 + 8 * r.baseline.len() + 12 + usize::from(!robust)
    );
    assert_eq!(r.likelihood_ratio_p_value.is_none(), robust);
    assert_eq!(
      s.state().estimation.as_ref().unwrap().family(),
      EstimationFamily::Survival
    );
    assert!(matches!(
      s.state().estimation,
      Some(EstimationState::CoxFit(_))
    ));
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}

#[test]
fn cox_current_points_use_owned_baseline_without_event_and_preserve_nulls_order_labels() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, "label variable reprate \"replacement rate\"");
  let ExecutionResult::Cox(r) = run(&mut s, &command(true)) else {
    panic!("Cox")
  };
  let owned = s.state().estimation;
  let labels = s.state().labels;
  run(&mut s, "drop censor1");
  let mut reference = POINTS.lines();
  let headers: Vec<_> = reference.next().unwrap().split(',').collect();
  let expected: Vec<Vec<f64>> = reference
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  for (prefix, ui) in [("", None), ("ui0_", Some(0)), ("ui1_", Some(1))] {
    if let Some(ui) = ui {
      run(&mut s, &format!("replace ui = {ui}"));
      run(&mut s, "replace rr_ui = reprate * ui");
      run(&mut s, "replace dr_ui = disrate * ui");
    }
    for (target, kind, gold) in [
      ("eta", "xb", "xb"),
      ("h", "cumhaz", "cumulative_hazard"),
      ("s", "survival", "net_survival"),
    ] {
      let target = format!("{prefix}{target}");
      run(&mut s, &format!("predict {target}, {kind}"));
      let col = index(&s, &target);
      let gold = headers
        .iter()
        .position(|h| *h == format!("{prefix}{gold}"))
        .unwrap();
      let actual = rows(&mut s);
      for i in 0..400 {
        close(numeric(&actual[i + 1][col]), expected[i][gold]);
      }
      if kind != "xb" {
        for i in [0, 401, 402, 403] {
          assert_eq!(actual[i][col], CellValue::Null);
        }
      }
      assert_eq!(s.state().estimation, owned);
      assert_eq!(s.state().labels, labels);
    }
  }
  let b = r
    .coefficients
    .iter()
    .find(|c| c.name == "logwage")
    .unwrap()
    .value;
  run(&mut s, "replace logwage = logwage + 0.5 if source_row == 1");
  run(&mut s, "drop spell");
  run(&mut s, "predict changed");
  let changed = index(&s, "changed");
  let eta = index(&s, "ui1_eta");
  let actual = rows(&mut s);
  close(
    numeric(&actual[1][changed]),
    numeric(&actual[1][eta]) + 0.5 * b,
  );
  let before = s.state();
  assert!(matches!(
    fail(&mut s, "predict unavailable, survival"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
}

#[test]
fn cox_prediction_failures_are_atomic_and_time_zero_and_step_tail_are_explicit() {
  let f = Fixture::new();
  let mut s = f.load();
  let ExecutionResult::Cox(r) = run(&mut s, &command(true)) else {
    panic!("Cox")
  };
  for text in [
    "predict reprate, cumhaz",
    "predict bad, hazard",
    "predict bad, n",
    "predict bad, pr",
    "predict bad, residuals",
    "predict bad, pmf(0)",
  ] {
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let _ = fail(&mut s, text);
    assert_eq!(s.state(), before, "{text}");
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
  run(&mut s, "replace spell = -1 if source_row == 400");
  let before = s.state();
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, "predict bad, cumhaz"),
    RuntimeError::StcoxFailed(_)
  ));
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "replace spell = 0 if source_row == 400");
  run(&mut s, "predict zero_h, cumhaz");
  run(&mut s, "predict zero_s, survival");
  let h = index(&s, "zero_h");
  let survival = index(&s, "zero_s");
  let actual = rows(&mut s);
  assert_eq!(actual[400][h], CellValue::Float(0.0));
  assert_eq!(actual[400][survival], CellValue::Float(1.0));
  let last = r.baseline.last().unwrap().time;
  run(
    &mut s,
    &format!("replace spell = {last} if source_row == 400"),
  );
  run(&mut s, "predict last_h, cumhaz");
  run(
    &mut s,
    &format!("replace spell = {} if source_row == 400", last * 100.0),
  );
  run(&mut s, "predict tail_h, cumhaz");
  let l = index(&s, "last_h");
  let t = index(&s, "tail_h");
  let actual = rows(&mut s);
  assert_eq!(actual[400][l], actual[400][t]);
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
    fail(&mut s, "predict overflow, cumhaz"),
    RuntimeError::StcoxFailed(_)
  ));
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "predict tail_s, survival");
  let col = index(&s, "tail_s");
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
fn cox_fit_failures_clear_only_inference_and_parse_failures_do_not_execute() {
  let f = Fixture::new();
  let mut s = f.load();
  run(&mut s, &command(false));
  let before = s.state();
  for text in [
    "predict bad, survival std",
    "predict bad, survival interval",
    "stcox spell, failure(censor1) cluster(source_row)",
    "stcox spell, failure(censor1) noconstant",
  ] {
    assert!(parse_command(text).is_err());
  }
  assert_eq!(s.state(), before);
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, "stcox absent, failure(censor1)"),
    RuntimeError::StcoxFailed(_)
  ));
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "generate constant = 1");
  for text in [
    "stcox spell constant, failure(censor1)",
    "stcox spell reprate reprate, failure(censor1)",
  ] {
    assert!(matches!(fail(&mut s, text), RuntimeError::StcoxFailed(_)));
  }
  run(&mut s, &command(false));
  run(&mut s, "replace censor1 = 2 if source_row == 400");
  let signature = run(&mut s, "datasignature");
  assert!(matches!(
    fail(&mut s, &command(false)),
    RuntimeError::StcoxFailed(_)
  ));
  assert!(s.state().estimation.is_none());
  assert_eq!(run(&mut s, "datasignature"), signature);
  run(&mut s, "replace censor1 = 0");
  assert!(matches!(
    fail(&mut s, &command(false)),
    RuntimeError::StcoxFailed(_)
  ));
}

#[test]
fn cox_checks_original_decimal_failure_codes_after_joint_finite_eligibility() {
  let f = Fixture::new();
  for (value, eligible, valid) in [
    ("1", true, true),
    ("1.000000000000000001", true, false),
    ("0.999999999999999999", true, false),
    ("1.000000000000000001", false, true),
    ("0.999999999999999999", false, true),
  ] {
    let mut s = f.load();
    // Invalid events on source_row0 are jointly excluded for missing time;
    // source_row1..400 contain both valid DECIMAL zeros and ones.
    let condition = if eligible {
      "censor1=1"
    } else {
      "source_row=0"
    };
    run(
      &mut s,
      &format!(
        "sql \"\"\"SELECT * EXCLUDE(censor1), CASE WHEN {condition} THEN CAST('{value}' AS DECIMAL(38,18)) ELSE CAST(censor1 AS DECIMAL(38,18)) END AS censor1 FROM active\"\"\" into cox_codes"
      ),
    );
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    let result = s.execute(parse_command(&command(true)).unwrap());
    assert_eq!(
      result.is_ok(),
      valid,
      "value={value}, eligible={eligible}: {result:?}"
    );
    if valid {
      let ExecutionResult::Cox(r) = result.unwrap() else {
        panic!("Cox")
      };
      assert_eq!(r.observation_count, 400);
      assert_eq!(
        r.sample.as_ref().unwrap().retained_indices,
        (1..=400).collect::<Vec<_>>()
      );
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
fn cox_baseline_only_ties_are_nelson_aalen_not_km_and_xb_needs_no_columns() {
  let f = Fixture::new();
  fs::write(f.0.join("baseline.csv"), "t,d\n1,1\n1,0\n2,1\n3,0\n").unwrap();
  let mut s = Session::new();
  run(
    &mut s,
    &format!("use \"{}\"", f.0.join("baseline.csv").display()),
  );
  let ExecutionResult::Cox(r) = run(&mut s, "stcox t, failure(d) robust") else {
    panic!("Cox")
  };
  assert!(r.coefficients.is_empty());
  assert_eq!(r.baseline.len(), 2);
  assert_eq!((r.baseline[0].risk_count, r.baseline[1].risk_count), (4, 2));
  close(r.baseline[1].cumulative_hazard.unwrap(), 0.75);
  close(r.baseline[1].survival, (-0.75_f64).exp());
  assert!((r.baseline[1].survival - 0.375).abs() > 0.01);
  run(&mut s, "drop d");
  run(&mut s, "predict h, cumhaz");
  run(&mut s, "drop t");
  run(&mut s, "predict eta");
  let col = index(&s, "eta");
  for row in rows(&mut s) {
    assert_eq!(row[col], CellValue::Float(0.0));
  }
}
