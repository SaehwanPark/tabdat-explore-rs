use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::{Command, NlCommand, parse_command};
use tabdat_runtime::{CellValue, EstimationState, ExecutionResult, RuntimeError, Session};
use tabdat_stats::NlsResult;

static NEXT: AtomicU64 = AtomicU64::new(0);
const INPUT: &str = include_str!("../../../tests/fixtures/nls/inputs.csv");
const METADATA: &str = include_str!("../../../tests/fixtures/nls/metadata.csv");
const PARAMETERS: &str = include_str!("../../../tests/fixtures/nls/parameters.csv");
const COVARIANCE: &str = include_str!("../../../tests/fixtures/nls/covariance.csv");
const PROJECTIONS: &str = include_str!("../../../tests/fixtures/nls/projections.csv");
// These frozen numeric fixtures have no quoted commas or escaped fields.
fn records(text: &str) -> impl Iterator<Item = Vec<&str>> {
  text
    .lines()
    .skip(1)
    .map(|line| line.split(',').map(|v| v.trim_matches('"')).collect())
}
struct Fixture(PathBuf);
impl Fixture {
  fn new(case: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-nls-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut text = "ord,x,y\n".to_string();
    for r in records(INPUT).filter(|r| r[0] == case) {
      text.push_str(&format!("{},{},{}\n", r[1], r[2], r[3]));
    }
    fs::write(path.join("input.csv"), text).unwrap();
    Self(path)
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
    CellValue::Float(v) => *v,
    CellValue::SignedInteger(v) => *v as f64,
    CellValue::UnsignedInteger(v) => *v as f64,
    _ => panic!("numeric: {v:?}"),
  }
}
fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-11 + 1e-10 * expected.abs(),
    "{actual:.17e} != {expected:.17e}"
  );
}
fn nl(text: &str) -> NlCommand {
  let Command::Nl { command } = parse_command(text).unwrap() else {
    panic!("nl")
  };
  command
}
fn fit(s: &mut Session, text: &str) -> NlsResult {
  let ExecutionResult::Nl(r) = run(s, text) else {
    panic!("nl fit")
  };
  *r
}
fn owned(s: &Session) -> NlsResult {
  let Some(EstimationState::NonlinearFit(r)) = s.state().estimation else {
    panic!("owned NLS")
  };
  *r
}

#[test]
fn explicit_nls_requires_active_data_and_initializes_no_unneeded_engine() {
  let mut s = Session::new();
  assert_eq!(
    s.execute(parse_command("nl y = a+b*x, params(a b) start(1 0)").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset { command: "nl" }
  );
  assert!(s.initialized_capabilities().is_empty());
}
#[test]
fn all_six_formulas_cross_ast_runtime_prediction_and_full_r_inference() {
  for metadata in records(METADATA).filter(|r| r[5] == "fit") {
    let f = Fixture::new(metadata[0]);
    let mut s = f.load();
    let original = table(&mut s, "sql SELECT ord,x,y FROM active");
    for mode in ["classical", "hc1"] {
      let command = format!(
        "nl y = {}, params({}) start({}){}",
        metadata[1],
        metadata[3].replace(';', " "),
        metadata[4].replace(';', " "),
        if mode == "hc1" { " robust" } else { "" }
      );
      let result = fit(&mut s, &command);
      assert_eq!(
        result.model.parameter_names,
        metadata[3].split(';').collect::<Vec<_>>()
      );
      assert_eq!(
        result.model.predictor_names,
        if metadata[2].is_empty() {
          vec![]
        } else {
          vec![metadata[2]]
        }
      );
      assert_eq!(owned(&s), result);
      assert_eq!(table(&mut s, "sql SELECT ord,x,y FROM active"), original);
      for row in records(PARAMETERS).filter(|r| r[0] == metadata[0] && r[1] == mode) {
        let c = result
          .coefficients
          .iter()
          .find(|c| c.name == row[2])
          .unwrap();
        for (v, expected) in [
          c.value,
          c.standard_error,
          c.statistic.unwrap(),
          c.p_value.unwrap(),
          c.confidence_lower.unwrap(),
          c.confidence_upper.unwrap(),
        ]
        .iter()
        .zip(&row[3..])
        {
          close(*v, expected.parse().unwrap());
        }
      }
      for row in records(COVARIANCE).filter(|r| r[0] == metadata[0] && r[1] == mode) {
        let i = result
          .model
          .parameter_names
          .iter()
          .position(|v| v == row[2])
          .unwrap();
        let j = result
          .model
          .parameter_names
          .iter()
          .position(|v| v == row[3])
          .unwrap();
        close(result.covariance_matrix[i][j], row[4].parse().unwrap());
      }
      let mean = format!("m_{mode}");
      let residual = format!("r_{mode}");
      run(&mut s, &format!("predict {mean}"));
      run(&mut s, &format!("predict {residual}, residuals"));
      let rows = table(
        &mut s,
        &format!("sql SELECT ord,{mean},{residual} FROM active"),
      );
      let expected: Vec<_> = records(PROJECTIONS)
        .filter(|r| r[0] == metadata[0])
        .collect();
      assert_eq!(rows.len(), expected.len());
      for (row, expected) in rows.iter().zip(expected) {
        assert_eq!(
          numeric(&row[0]) as usize,
          expected[1].parse::<usize>().unwrap()
        );
        close(numeric(&row[1]), expected[2].parse().unwrap());
        close(numeric(&row[2]), expected[3].parse().unwrap());
      }
      assert_eq!(owned(&s), result);
    }
  }
}
#[test]
fn all_four_frozen_nonregular_controls_reject_through_the_runtime() {
  let rejected: Vec<_> = records(METADATA)
    .filter(|r| r[5].starts_with("reject_"))
    .collect();
  assert_eq!(rejected.len(), 4);
  for metadata in rejected {
    let f = Fixture::new(metadata[0]);
    let mut s = f.load();
    let old = s.state();
    let data = table(&mut s, "sql SELECT * FROM active");
    let text = format!(
      "nl y = {}, params({}) start({})",
      metadata[1],
      metadata[3].replace(';', " "),
      metadata[4].replace(';', " ")
    );
    assert!(
      matches!(
        s.execute(parse_command(&text).unwrap()).unwrap_err(),
        RuntimeError::NlFailed(_)
      ),
      "{}",
      metadata[0]
    );
    assert_eq!(s.state(), old);
    assert_eq!(table(&mut s, "sql SELECT * FROM active"), data);
  }
}

#[test]
fn explicit_parameter_order_and_noconstant_never_insert_or_remove_parameters() {
  let f = Fixture::new("linear");
  let mut s = f.load();
  let ordinary = fit(&mut s, "nl y = a+b*x, params(a b) start(1 0)");
  let no_constant = fit(&mut s, "nl y = a+b*x, params(a b) start(1 0) noconstant");
  assert_eq!(ordinary, no_constant);
  let reordered = fit(&mut s, "nl y = a+b*x, params(b a) start(0 1)");
  assert_eq!(reordered.model.parameter_names, ["b", "a"]);
  for (a, b) in ordinary
    .coefficients
    .iter()
    .zip(reordered.coefficients.iter().rev())
  {
    close(a.value, b.value);
    close(a.standard_error, b.standard_error);
  }
}
#[test]
fn joint_finite_sample_ignores_unrelated_missingness_and_keeps_original_indices() {
  let f = Fixture::new("linear");
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT ord,x,y,CAST(NULL AS DOUBLE) AS unused FROM (SELECT * FROM active UNION ALL SELECT 0,CAST('NaN' AS DOUBLE),1 UNION ALL SELECT 8,1,NULL UNION ALL SELECT 9,NULL,2) rows ORDER BY ord into analysis",
  );
  run(&mut s, "use analysis");
  let r = fit(&mut s, "nl y = a+b*x, params(a b) start(1 0)");
  assert_eq!(r.sample.total_observations, 10);
  assert_eq!(r.sample.dropped_observations, 3);
  assert_eq!(r.sample.retained_indices, (1..8).collect::<Vec<_>>());
  assert_eq!(r.observation_count, 7);
  close(r.coefficients[0].value, -0.45);
}
#[test]
fn failures_clear_stale_fit_but_leave_typed_data_metadata_labels_and_tables_unchanged() {
  let f = Fixture::new("linear");
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT ord,CAST(9223372036854775808 AS UBIGINT)+CAST(ord AS UBIGINT) AS id,x,y,CAST(ord+0.25 AS DECIMAL(12,2)) AS amount,'row-' || ord AS tag FROM active into analysis",
  );
  run(&mut s, "use analysis");
  run(&mut s, "label variable x \"original x label\"");
  run(&mut s, "sql SELECT ord FROM active into unrelated");
  run(&mut s, "use analysis");
  let base = nl("nl y = a+b*x, params(a b) start(1 0)");
  let mut invalid = Vec::new();
  let mut c = base.clone();
  c.start_values[0] = "NaN".into();
  invalid.push(c);
  let mut c = base.clone();
  c.parameter_names[1] = "a".into();
  invalid.push(c);
  let mut c = base.clone();
  c.parameter_names.push("unused".into());
  c.start_values.push("0".into());
  invalid.push(c);
  let mut c = base.clone();
  c.outcome = "absent".into();
  invalid.push(c);
  let mut c = base.clone();
  c.outcome = "tag".into();
  invalid.push(c);
  let mut c = base.clone();
  c.expression = tabdat_language::GenerateExpression::Number("inf".into());
  invalid.push(c);
  let mut c = base.clone();
  c.expression = tabdat_language::GenerateExpression::FunctionCall {
    name: "exp".into(),
    arguments: vec![],
  };
  invalid.push(c);
  let mut c = base.clone();
  for _ in 0..33 {
    c.expression = tabdat_language::GenerateExpression::UnaryMinus(Box::new(c.expression));
  }
  invalid.push(c);
  invalid.push(nl("nl y = log(a+x), params(a) start(1)"));
  invalid.push(nl("nl y = a/(b+x), params(a b) start(1 2)"));
  for command in invalid {
    run(&mut s, "regress y x");
    assert!(s.state().estimation.is_some());
    let old = s.state();
    let data = table(&mut s, "sql SELECT * FROM active");
    assert!(matches!(
      s.execute(Command::Nl { command }).unwrap_err(),
      RuntimeError::NlFailed(_)
    ));
    let mut expected = old;
    expected.estimation = None;
    assert_eq!(s.state(), expected);
    assert_eq!(table(&mut s, "sql SELECT * FROM active"), data);
  }
  fit(&mut s, "nl y = a+b*x, params(a b) start(1 0)");
  let state = s.state();
  let data = table(&mut s, "sql SELECT * FROM active");
  run(&mut s, "predict nls_mean, mean");
  let after = table(&mut s, "sql SELECT ord,id,x,y,amount,tag FROM active");
  assert_eq!(after, data);
  assert_eq!(s.state().estimation, state.estimation);
  assert_eq!(s.state().labels, state.labels);
  assert_eq!(
    s.state().named_tables["unrelated"],
    state.named_tables["unrelated"]
  );
  let mut expected_tables = state.named_tables;
  expected_tables
    .get_mut("analysis")
    .unwrap()
    .columns
    .push(tabdat_runtime::ColumnInfo {
      name: "nls_mean".into(),
      data_type: "DOUBLE".into(),
    });
  assert_eq!(s.state().named_tables, expected_tables);
  assert_eq!(
    s.active_dataset()
      .unwrap()
      .columns
      .last()
      .unwrap()
      .data_type,
    "DOUBLE"
  );
}
#[test]
fn current_mean_and_residual_samples_and_all_mean_aliases_are_distinct_from_linear_xb() {
  let f = Fixture::new("exponential_slope");
  let mut s = f.load();
  let r = fit(&mut s, "nl y = exp(a*x), params(a) start(0.5)");
  let a = r.coefficients[0].value;
  run(&mut s, "replace x = x+0.25");
  run(&mut s, "replace y = NULL if ord == 1");
  run(&mut s, "replace x = NULL if ord == 2");
  for command in [
    "predict mu",
    "predict explicit_xb, xb",
    "predict explicit_mean, mean",
    "predict resid, residuals",
  ] {
    run(&mut s, command);
  }
  let rows = table(
    &mut s,
    "sql SELECT ord,x,y,mu,explicit_xb,explicit_mean,resid FROM active",
  );
  for row in rows {
    if numeric(&row[0]) == 2.0 {
      assert!(row[3..].iter().all(|v| *v == CellValue::Null));
      continue;
    }
    let mean = (a * numeric(&row[1])).exp();
    for v in &row[3..6] {
      close(numeric(v), mean);
    }
    if row[2] == CellValue::Null {
      assert_eq!(row[6], CellValue::Null);
    } else {
      close(numeric(&row[6]), numeric(&row[2]) - mean);
    }
  }
  assert_eq!(owned(&s), r);
}
#[test]
fn failed_predictions_are_atomic_and_preserve_owned_fit() {
  let f = Fixture::new("exponential_slope");
  let mut s = f.load();
  fit(&mut s, "nl y = exp(a*x), params(a) start(0.5)");
  let mut commands = vec![
    parse_command("predict X").unwrap(),
    parse_command("predict bad, pr").unwrap(),
  ];
  let Command::Predict { command: base } = parse_command("predict bad").unwrap() else {
    panic!("predict")
  };
  let mut c = base.clone();
  c.std = true;
  commands.push(Command::Predict { command: c });
  let mut c = base.clone();
  c.level = "90".into();
  commands.push(Command::Predict { command: c });
  let mut c = base;
  c.interval = true;
  commands.push(Command::Predict { command: c });
  for command in commands {
    let old = s.state();
    let data = table(&mut s, "sql SELECT * FROM active");
    assert!(s.execute(command).is_err());
    assert_eq!(s.state(), old);
    assert_eq!(table(&mut s, "sql SELECT * FROM active"), data);
  }
  run(&mut s, "replace x = 10000 if ord == 1");
  let old = s.state();
  let data = table(&mut s, "sql SELECT * FROM active");
  assert!(matches!(
    s.execute(parse_command("predict overflow").unwrap())
      .unwrap_err(),
    RuntimeError::NlFailed(_)
  ));
  assert_eq!(s.state(), old);
  assert_eq!(table(&mut s, "sql SELECT * FROM active"), data);
}
#[test]
fn parameter_only_mean_needs_no_x_or_current_y_but_residual_does() {
  let f = Fixture::new("constant_exponential");
  let mut s = f.load();
  let r = fit(&mut s, "nl y = exp(a), params(a) start(0)");
  run(&mut s, "drop y x");
  run(&mut s, "predict mu, mean");
  let rows = table(&mut s, "sql SELECT mu FROM active");
  assert_eq!(rows.len(), r.observation_count);
  for row in rows {
    close(numeric(&row[0]), r.coefficients[0].value.exp());
  }
  let old = s.state();
  assert!(matches!(
    s.execute(parse_command("predict residual, residuals").unwrap())
      .unwrap_err(),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), old);
}
