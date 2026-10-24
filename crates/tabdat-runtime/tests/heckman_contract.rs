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
fn cells(line: &str) -> Vec<&str> {
  line.split(',').map(|v| v.trim_matches('"')).collect()
}
impl Fixture {
  fn new(case: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-heckman-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("row,d,y,x,w,z,unused\n0,,0,0,0,0,NaN\n");
    for line in include_str!("../../tabdat-stats/tests/fixtures/heckman_data.csv")
      .lines()
      .skip(1)
    {
      let c = cells(line);
      if c[0] != case {
        continue;
      }
      writeln!(
        csv,
        "{},{},{},{},{},{},NaN",
        c[1],
        c[2],
        c[3],
        if c[2] == "0" { "" } else { c[4] },
        c[4],
        c[5]
      )
      .unwrap();
    }
    csv.push_str("361,1,,0,0,0,NaN\n362,1,0,,0,0,NaN\n363,0,,,,,NaN\n");
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
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
fn fit(s: &mut Session, constant: bool) -> Box<tabdat_stats::HeckmanTwoStepResult> {
  let text = format!(
    "heckman y x, selectdep(d) select(w z) twostep{}",
    if constant { "" } else { " noconstant" }
  );
  let ExecutionResult::HeckmanTwoStep(f) = run(s, &text) else {
    panic!()
  };
  f
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(h) = run(s, "head 1000") else {
    panic!()
  };
  h.rows
}
fn unchanged(s: &mut Session, before: &[Vec<CellValue>]) {
  assert_eq!(format!("{before:?}"), format!("{:?}", rows(s)));
}
fn number(c: &CellValue) -> f64 {
  let CellValue::Float(v) = c else {
    panic!("not DOUBLE: {c:?}")
  };
  *v
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.16e}!={b:.16e}"
  );
}
fn reference(case: &str) -> BTreeMap<(String, usize, usize), f64> {
  include_str!("../../tabdat-stats/tests/fixtures/heckman_reference.csv")
    .lines()
    .skip(1)
    .filter_map(|line| {
      let c = cells(line);
      (c[0] == case).then(|| {
        (
          (c[1].into(), c[2].parse().unwrap(), c[3].parse().unwrap()),
          c[4].parse().unwrap(),
        )
      })
    })
    .collect()
}

#[test]
fn two_stage_r_reference_covariances_and_asymmetric_missingness_preserve_exact_samples() {
  for case in ["exclusion", "no_constant"] {
    let fixture = Fixture::new(case);
    let mut s = fixture.load();
    run(&mut s, "label variable x \"outcome regressor\"");
    let before = s.state();
    let data = rows(&mut s);
    let f = fit(&mut s, case == "exclusion");
    let r = reference(case);
    assert_eq!(f.selection.observation_count, 360);
    assert_eq!(
      f.selection.sample.as_ref().unwrap().retained_indices,
      (1..=360).collect::<Vec<_>>()
    );
    assert_eq!(f.selection.sample.as_ref().unwrap().total_observations, 364);
    assert_eq!(f.selection.sample.as_ref().unwrap().dropped_observations, 4);
    let selected: Vec<usize> = include_str!("../../tabdat-stats/tests/fixtures/heckman_data.csv")
      .lines()
      .skip(1)
      .filter_map(|line| {
        let c = cells(line);
        (c[0] == case && c[2] == "1").then(|| c[1].parse().unwrap())
      })
      .collect();
    assert_eq!(f.sample.as_ref().unwrap().retained_indices, selected);
    assert_eq!(f.selected_count, selected.len());
    for (stage, coefs, covariance) in [
      (
        "selection",
        &f.selection.coefficients,
        &f.selection.covariance,
      ),
      ("outcome", &f.coefficients, &f.covariance),
    ] {
      for (i, c) in coefs.iter().enumerate() {
        for (name, value) in [
          ("value", c.value),
          ("standard_error", c.standard_error),
          ("statistic", c.statistic),
          ("confidence_lower", c.confidence_lower),
          ("confidence_upper", c.confidence_upper),
        ] {
          close(value, r[&(format!("{stage}_{name}"), i + 1, 0)]);
        }
        let p = r[&(format!("{stage}_p_value"), i + 1, 0)];
        assert!(c.p_value > 0.0 && (c.p_value - p).abs() <= 1e-8 * p);
        for (j, v) in covariance[i].iter().enumerate() {
          close(*v, r[&(format!("{stage}_covariance"), i + 1, j + 1)]);
        }
      }
    }
    close(f.sigma, r[&("sigma".into(), 0, 0)]);
    close(f.rho, r[&("rho".into(), 0, 0)]);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    unchanged(&mut s, &data);
    assert!(matches!(
      s.state().estimation,
      Some(EstimationState::HeckmanTwoStepFit(_))
    ));
    assert_eq!(
      s.state().estimation.unwrap().family(),
      EstimationFamily::Heckman
    );
  }
}

#[test]
fn five_current_projections_ignore_outcomes_require_only_relevant_equations_and_retain_owned_fit() {
  let fixture = Fixture::new("exclusion");
  let mut s = fixture.load();
  let f = fit(&mut s, true);
  run(&mut s, "label variable x \"outcome regressor\"");
  let owned = s.state().estimation;
  let labels = s.state().labels;
  for (kind, target) in [
    ("xb", "latent"),
    ("pr", "prob"),
    ("mills", "imr"),
    ("ycond", "conditional"),
    ("yexpected", "contribution"),
  ] {
    run(&mut s, &format!("predict {target}, {kind}"));
    assert_eq!(s.state().estimation, owned);
    assert_eq!(s.state().labels, labels);
  }
  let data = rows(&mut s);
  assert_eq!(data.len(), 364);
  for line in include_str!("../../tabdat-stats/tests/fixtures/heckman_predictions.csv")
    .lines()
    .skip(1)
  {
    let c = cells(line);
    if c[0] != "exclusion" {
      continue;
    }
    let i = c[1].parse::<usize>().unwrap();
    for j in 0..5 {
      if matches!(j, 0 | 3 | 4) && data[i][3] == CellValue::Null {
        assert_eq!(data[i][7 + j], CellValue::Null);
      } else {
        close(number(&data[i][7 + j]), c[2 + j].parse().unwrap());
      }
    }
  }
  run(&mut s, "replace w = w + 0.3");
  run(&mut s, "drop y d x");
  run(&mut s, "predict fresh, pr");
  assert_eq!(s.state().estimation, owned);
  let changed = rows(&mut s);
  // Remaining columns: row,w,z,unused, five predictions,fresh.
  for row in changed.iter().skip(1).take(360) {
    let z = [number(&row[1]), number(&row[2])];
    let expected =
      tabdat_stats::heckman_predict_row(&f, &[], &z, tabdat_stats::HeckmanPredictionKind::Pr)
        .unwrap();
    close(number(row.last().unwrap()), expected);
  }
  let before = s.state();
  let data = rows(&mut s);
  assert!(matches!(
    fail(&mut s, "predict absent, ycond"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
  unchanged(&mut s, &data);
}

#[test]
fn parsed_heckman_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("heckman outcome x1, selectdep(selected) select(z1)").unwrap();
  let mut session = Session::new();
  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "heckman" }
  );
  let fixture = Fixture::new("exclusion");
  let mut session = fixture.load();
  fit(&mut session, true);
  let before = session.state();
  let data = rows(&mut session);
  assert!(matches!(
    fail(&mut session, "heckman y x, selectdep(d) select(w z)"),
    RuntimeError::UnsupportedCommand { name: "heckman" }
  ));
  assert_eq!(session.state(), before);
  unchanged(&mut session, &data);
}

#[test]
fn unsupported_new_fit_clears_stale_state_without_mutating_data_or_labels() {
  let fixture = Fixture::new("exclusion");
  for command in [
    "heckman y x, selectdep(d) select(w z) twostep robust",
    "heckman y x, selectdep(d) select(w z) twostep cluster(row)",
  ] {
    let mut s = fixture.load();
    fit(&mut s, true);
    run(&mut s, "label variable x \"preserve me\"");
    let before = s.state();
    let data = rows(&mut s);
    assert!(matches!(
      fail(&mut s, command),
      RuntimeError::HeckmanFailed(_)
    ));
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    unchanged(&mut s, &data);
  }
  let mut s = fixture.load();
  fit(&mut s, true);
  run(&mut s, "replace d = 2");
  let before = s.state();
  let data = rows(&mut s);
  assert!(
    fail(&mut s, "heckman y x, selectdep(d) select(w z) twostep")
      .to_string()
      .contains("exactly0 or1")
  );
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, before.active_dataset);
  unchanged(&mut s, &data);
}

#[test]
fn exact_decimal_selection_codes_are_checked_before_float_rounding() {
  let fixture = Fixture::new("exclusion");
  for (value, valid) in [
    ("1", true),
    ("1.000000000000000001", false),
    ("0.999999999999999999", false),
  ] {
    let mut s = fixture.load();
    run(
      &mut s,
      &format!(
        "sql \"\"\"SELECT row,CASE WHEN d=1 THEN CAST('{value}' AS DECIMAL(38,18)) ELSE CAST(d AS DECIMAL(38,18)) END AS d,y,x,w,z,unused FROM active\"\"\" into coded"
      ),
    );
    let before = s.state();
    let data = rows(&mut s);
    let result = s.execute(parse_command("heckman y x, selectdep(d) select(w z) twostep").unwrap());
    assert_eq!(result.is_ok(), valid, "{value}: {result:?}");
    if !valid {
      assert!(s.state().estimation.is_none());
    }
    assert_eq!(s.state().active_dataset, before.active_dataset);
    unchanged(&mut s, &data);
  }
}

#[test]
fn prediction_failures_are_atomic_and_preserve_owned_fit_and_labels() {
  let fixture = Fixture::new("exclusion");
  let mut s = fixture.load();
  fit(&mut s, true);
  run(&mut s, "label variable z \"selection regressor\"");
  for command in [
    "predict x, pr",
    "predict invalid, residuals",
    "predict invalid, posterior_predictive std",
    "predict invalid, posterior_predictive interval",
    "predict invalid, n",
    "predict invalid, mean",
    "estat margins",
  ] {
    let before = s.state();
    let data = rows(&mut s);
    fail(&mut s, command);
    assert_eq!(s.state(), before);
    unchanged(&mut s, &data);
  }
  // Raw API requests must also reject auxiliary point-inference options;
  // the CLI parser rejects several combinations before execution.
  for mode in 0..4 {
    let tabdat_domain::Command::Predict { mut command } =
      parse_command("predict invalid, pr").unwrap()
    else {
      panic!()
    };
    match mode {
      0 => command.std = true,
      1 => command.interval = true,
      2 => command.level = "90".into(),
      _ => command.saving = Some(fixture.0.join("not-written.csv").display().to_string()),
    }
    let before = s.state();
    let data = rows(&mut s);
    assert!(matches!(
      s.execute(tabdat_domain::Command::Predict { command }),
      Err(RuntimeError::HeckmanFailed(_))
    ));
    assert_eq!(s.state(), before);
    unchanged(&mut s, &data);
    assert!(!fixture.0.join("not-written.csv").exists());
  }
  run(&mut s, "replace z = -1000000");
  let before = s.state();
  let data = rows(&mut s);
  assert!(matches!(
    fail(&mut s, "predict invalid, mills"),
    RuntimeError::HeckmanFailed(_)
  ));
  assert_eq!(s.state(), before);
  unchanged(&mut s, &data);
  run(&mut s, "predict zero, pr");
  assert_eq!(s.state().estimation, before.estimation);
}
