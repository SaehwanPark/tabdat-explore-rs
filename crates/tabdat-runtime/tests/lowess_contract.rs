use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::{Command, LowessCommand, parse_command};
use tabdat_runtime::{CellValue, ExecutionResult, RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
const INPUT: &str = include_str!("../../../tests/fixtures/lowess/input.csv");
const EXPECTED: &str = include_str!("../../../tests/fixtures/lowess/expected.csv");
struct Fixture(PathBuf);
impl Fixture {
  fn new(input: &str) -> Self {
    let dir = std::env::temp_dir().join(format!(
      "tabdat-lowess-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("input.csv"), input).unwrap();
    Self(dir)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("input.csv").display()),
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
fn table(s: &mut Session, text: &str) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Table(t) = run(s, text) else {
    panic!("table")
  };
  t.rows
}
fn numeric(v: &CellValue) -> f64 {
  match v {
    CellValue::Float(x) => *x,
    CellValue::SignedInteger(x) => *x as f64,
    CellValue::UnsignedInteger(x) => *x as f64,
    _ => panic!("numeric: {v:?}"),
  }
}
fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-11 + 1e-10 * expected.abs(),
    "{actual:.17e} != {expected:.17e}"
  );
}
fn direct(target: &str, bandwidth: &str) -> Command {
  Command::Lowess {
    command: LowessCommand {
      outcome: "y".into(),
      predictor: "x".into(),
      target_variable: target.into(),
      bandwidth: bandwidth.into(),
    },
  }
}
fn mixed(s: &mut Session) {
  run(
    s,
    "sql SELECT ord, CAST(9223372036854775808 AS UBIGINT)+CAST(ord AS UBIGINT) AS id, x, y, CAST(ord+0.25 AS DECIMAL(12,2)) AS amount, 'row-' || ord AS tag FROM (SELECT source_row AS ord, CAST(x AS DOUBLE) AS x, CAST(y AS DOUBLE) AS y FROM active UNION ALL SELECT 0,NULL,2 UNION ALL SELECT 32,1,CAST('NaN' AS DOUBLE) UNION ALL SELECT 33,CAST('Infinity' AS DOUBLE),4 UNION ALL SELECT 34,2,NULL) AS rows ORDER BY ord into analysis",
  );
  run(s, "use analysis");
  run(s, "label variable x \"original predictor label\"");
}

#[test]
fn parsed_default_is_still_two_thirds_and_requires_active_data() {
  let command = parse_command("lowess y x, gen(y_hat)").unwrap();
  assert_eq!(command, direct("y_hat", &(2.0_f64 / 3.0).to_string()));
  let mut s = Session::new();
  assert_eq!(
    s.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "lowess" }
  );
  assert!(s.initialized_capabilities().is_empty());
}
#[test]
fn parsed_explicit_bandwidth_keeps_its_ast_and_requires_active_data() {
  let command = parse_command("lowess y x, gen(y_hat) bandwidth=0.5").unwrap();
  assert_eq!(command, direct("y_hat", "0.5"));
  assert_eq!(
    Session::new().execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "lowess" }
  );
}
#[test]
fn all_three_spans_cross_runtime_in_original_order_against_qr() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  let old = s.active_dataset().unwrap().clone();
  for (i, (span, k, constant)) in [(0.8, 12, 0), (0.4, 5, 5), (0.1, 1, 13)]
    .into_iter()
    .enumerate()
  {
    let ExecutionResult::Lowess(r) =
      run(&mut s, &format!("lowess y x, gen(fit{i}) bandwidth={span}"))
    else {
      panic!("LOWESS")
    };
    assert_eq!(r.bandwidth, span);
    assert_eq!(r.half_window, k);
    assert_eq!(r.local_constant_windows, constant);
    assert_eq!(r.sample.retained_indices, (0..31).collect::<Vec<_>>());
    assert_eq!(r.sample.total_observations, 31);
    assert_eq!(r.sample.dropped_observations, 0);
    assert!(r.sample.weights.is_none() && r.sample.cluster_groups.is_none());
    assert_eq!(&r.dataset.columns[..3], old.columns.as_slice());
    assert_eq!(r.dataset.columns.last().unwrap().data_type, "DOUBLE");
  }
  let values = table(&mut s, "sql SELECT * FROM active");
  for (actual, reference) in values.iter().zip(EXPECTED.lines().skip(1)) {
    let expected: Vec<f64> = reference.split(',').map(|x| x.parse().unwrap()).collect();
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
      close(numeric(a), b);
    }
  }
  assert!(s.state().estimation.is_none());
  let ExecutionResult::Lowess(default) = run(&mut s, "lowess y x, gen(default_fit)") else {
    panic!("LOWESS")
  };
  assert_eq!(default.bandwidth, 2.0 / 3.0);
  assert_eq!(default.half_window, 10);
}
#[test]
fn coincident_rank_windows_and_k0_are_local_means_and_identity_not_a_new_model() {
  let f = Fixture::new(include_str!("../../../tests/fixtures/lowess/all_tied.csv"));
  let mut s = f.load();
  let ExecutionResult::Lowess(r) = run(&mut s, "lowess y x, gen(local_mean) bandwidth=0.4") else {
    panic!("LOWESS")
  };
  assert_eq!(r.local_constant_windows, 7);
  assert_eq!(r.half_window, 1);
  let ExecutionResult::Lowess(r) = run(&mut s, "lowess y x, gen(identity_fit) bandwidth=0.1")
  else {
    panic!("LOWESS")
  };
  assert_eq!(r.local_constant_windows, 7);
  assert_eq!(r.half_window, 0);
  assert_eq!(r.dataset.columns.last().unwrap().data_type, "DOUBLE");
  let values = table(&mut s, "sql SELECT y, local_mean, identity_fit FROM active");
  let expected = include_str!("../../../tests/fixtures/lowess/all_tied_expected.csv");
  for (actual, reference) in values.iter().zip(expected.lines().skip(1)) {
    let expected: Vec<f64> = reference.split(',').map(|x| x.parse().unwrap()).collect();
    close(numeric(&actual[1]), expected[4]);
    assert_eq!(numeric(&actual[0]), numeric(&actual[2]));
  }
  assert!(s.state().estimation.is_none());
  assert!(
    s.execute(parse_command("predict impossible").unwrap())
      .is_err()
  );
}
#[test]
fn joint_finite_sample_nulls_wide_types_labels_named_table_and_prior_fit_are_preserved() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  mixed(&mut s);
  run(&mut s, "regress y x");
  run(&mut s, "predict old_mean, xb");
  let fit = s.state().estimation.clone();
  let labels = s.active_label_metadata().cloned();
  let immutable = table(
    &mut s,
    "sql SELECT ord,id,amount,tag FROM active ORDER BY ord",
  );
  assert!(matches!(
    immutable[1][1],
    CellValue::UnsignedInteger(9223372036854775809)
  ));
  assert!(matches!(
    immutable[1][2],
    CellValue::Decimal {
      value: 125,
      scale: 2,
      ..
    }
  ));
  let ExecutionResult::Lowess(r) = run(&mut s, "lowess y x, gen(smoothed) bandwidth=0.4") else {
    panic!("LOWESS")
  };
  assert_eq!(r.sample.retained_indices, (1..=31).collect::<Vec<_>>());
  assert_eq!(r.sample.total_observations, 35);
  assert_eq!(r.sample.dropped_observations, 4);
  assert_eq!(s.active_table_name(), Some("analysis"));
  assert_eq!(s.named_tables()["analysis"], r.dataset);
  assert_eq!(s.state().estimation, fit);
  assert_eq!(s.active_label_metadata(), labels.as_ref());
  assert_eq!(
    table(
      &mut s,
      "sql SELECT ord,id,amount,tag FROM active ORDER BY ord"
    ),
    immutable
  );
  run(&mut s, "predict new_mean, xb");
  let values = table(
    &mut s,
    "sql SELECT ord,smoothed,old_mean,new_mean FROM active ORDER BY ord",
  );
  for (i, row) in values.iter().enumerate() {
    assert_eq!(row[2], row[3]);
    if [0, 32, 33, 34].contains(&i) {
      assert_eq!(row[1], CellValue::Null);
    }
  }
  assert!(
    !matches!(values[34][2], CellValue::Null),
    "old-model current-X scoring does not need current Y"
  );
  for (row, reference) in values[1..=31].iter().zip(EXPECTED.lines().skip(1)) {
    let expected: Vec<f64> = reference.split(',').map(|v| v.parse().unwrap()).collect();
    close(numeric(&row[1]), expected[4]);
  }
}
#[test]
fn schema_target_and_direct_api_option_refusals_leave_full_state_and_data_unchanged() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  mixed(&mut s);
  run(&mut s, "regress y x");
  let before = s.state();
  let signature = run(&mut s, "datasignature");
  let mut commands = vec![direct("x", "0.8"), direct("X", "0.8"), direct("", "0.8")];
  for bw in ["garbage", "NaN", "inf", "0", "-1", "1", "0.001"] {
    commands.push(direct("not_published", bw));
  }
  for (y, x) in [
    ("unknown", "x"),
    ("y", "unknown"),
    ("tag", "x"),
    ("y", "tag"),
  ] {
    commands.push(Command::Lowess {
      command: LowessCommand {
        outcome: y.into(),
        predictor: x.into(),
        target_variable: "not_published".into(),
        bandwidth: "0.8".into(),
      },
    });
  }
  for command in commands {
    assert!(matches!(
      s.execute(command),
      Err(RuntimeError::LowessFailed(_))
    ));
    assert_eq!(s.state(), before);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}
#[test]
fn arithmetic_failure_after_a_completed_window_does_not_publish_or_clear_prior_fit() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT source_row AS ord,y,CASE WHEN source_row<=2 THEN -1e308 ELSE 1e308 END AS x FROM active WHERE source_row<=5 ORDER BY ord into difficult",
  );
  run(&mut s, "use difficult");
  run(&mut s, "regress y ord");
  run(&mut s, "label variable x \"extreme predictor\"");
  let state = s.state();
  let signature = run(&mut s, "datasignature");
  let error = s.execute(direct("not_published", "0.8")).unwrap_err();
  assert!(matches!(error, RuntimeError::LowessFailed(_)));
  assert!(error.to_string().contains("difference"));
  assert_eq!(s.state(), state);
  assert_eq!(run(&mut s, "datasignature"), signature);
}
#[test]
fn row_and_window_work_limits_fail_atomically_without_destroying_a_retained_fit() {
  let f = Fixture::new(INPUT);
  for (n, bandwidth) in [(5_000, "0.9202"), (10_000, "0.8"), (10_001, "0.8")] {
    let mut s = f.load();
    run(
      &mut s,
      &format!(
        "sql SELECT range AS x,CAST(range%7 AS DOUBLE) AS y FROM range({n}) ORDER BY x into large"
      ),
    );
    run(&mut s, "use large");
    run(&mut s, "regress y x");
    let before = s.state();
    let signature = run(&mut s, "datasignature");
    assert!(matches!(
      s.execute(direct("not_published", bandwidth)),
      Err(RuntimeError::LowessFailed(_))
    ));
    assert_eq!(s.state(), before);
    assert_eq!(run(&mut s, "datasignature"), signature);
  }
}
#[test]
fn limits_count_retained_pairs_and_null_outputs_keep_all_candidate_rows() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT range AS ord,CASE WHEN range IN (0,10001) THEN 1.0 ELSE NULL END AS x,CASE WHEN range IN (0,10001) THEN 2.0 ELSE NULL END AS y FROM range(10002) ORDER BY ord into sparse",
  );
  run(&mut s, "use sparse");
  let ExecutionResult::Lowess(r) = run(&mut s, "lowess y x, gen(smoothed)") else {
    panic!("LOWESS")
  };
  assert_eq!(r.sample.retained_indices, vec![0, 10001]);
  assert_eq!(r.sample.total_observations, 10002);
  assert_eq!(r.sample.dropped_observations, 10000);
  assert_eq!(r.dataset.row_count, 10002);
  let counts = table(&mut s, "sql SELECT count(smoothed),count(*) FROM active");
  assert_eq!(numeric(&counts[0][0]), 2.);
  assert_eq!(numeric(&counts[0][1]), 10002.);
}
