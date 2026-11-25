use std::{
  collections::HashMap,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-count-residuals-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let y = [
      0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 5, 4, 1, 1, 0, 0,
      2, 2, 0, 1, 0,
    ];
    let mut csv = "id,y,x,z,person\n".to_owned();
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{},{}\n",
        i + 1,
        y,
        -2. + 4. * i as f64 / 35.,
        [0., 1., 0.4][i % 3],
        i % 6
      ));
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }

  fn session(&self) -> Session {
    let mut session = Session::new();
    run(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn number(cell: &CellValue) -> Option<f64> {
  match cell {
    CellValue::Float(value) => Some(*value),
    CellValue::SignedInteger(value) => Some(*value as f64),
    CellValue::UnsignedInteger(value) => Some(*value as f64),
    CellValue::Decimal { value, scale, .. } => Some(*value as f64 / 10_f64.powi(i32::from(*scale))),
    _ => None,
  }
}

fn reference() -> HashMap<u64, [Option<f64>; 4]> {
  include_str!("fixtures/count_response_residual_reference.csv")
    .lines()
    .skip(1)
    .map(|line| {
      let fields: Vec<_> = line.split(',').collect();
      let parse = |field: &str| {
        if field == "NA" {
          None
        } else {
          Some(field.parse().unwrap())
        }
      };
      (
        fields[0].parse().unwrap(),
        [
          parse(fields[2]),
          parse(fields[3]),
          parse(fields[4]),
          parse(fields[5]),
        ],
      )
    })
    .collect()
}

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 2e-9 + 2e-8 * expected.abs(),
    "actual={actual:.17}, expected={expected:.17}"
  );
}

#[test]
fn raw_response_residuals_match_base_r_on_current_rows_and_ignore_covariance() {
  let fixture = Fixture::new();
  let expected = reference();
  assert_eq!(expected.len(), 36);

  for family in ["poisson", "nbreg"] {
    let mut covariance_results = Vec::new();
    for covariance in ["", " robust", " cluster(person)"] {
      let mut session = fixture.session();
      run(&mut session, "generate log_time = .2 * x");
      run(
        &mut session,
        &format!("{family} y x z, offset(log_time){covariance}"),
      );
      let retained = session.state().estimation;

      run(&mut session, "replace x = x + .2");
      run(&mut session, "replace log_time = log_time - .3");
      run(&mut session, "replace y = y + 1");
      run(&mut session, "replace y = NULL if id == 1");
      run(&mut session, "replace x = NULL if id == 2");
      run(&mut session, "replace log_time = NULL if id == 3");
      run(&mut session, "predict current_mean, n");
      run(&mut session, "predict response_residual, residuals");
      let ExecutionResult::Head(table) = run(&mut session, "head 100") else {
        panic!("head")
      };
      let index = |name: &str| {
        table
          .columns
          .iter()
          .position(|column| column == name)
          .unwrap()
      };
      let (id_index, y_index, mean_index, residual_index) = (
        index("id"),
        index("y"),
        index("current_mean"),
        index("response_residual"),
      );
      assert_eq!(table.rows.len(), 36);
      let mut rows = Vec::new();
      for (position, row) in table.rows.iter().enumerate() {
        let id = number(&row[id_index]).unwrap() as u64;
        assert_eq!(id as usize, position + 1);
        let ref_row = expected[&id];
        let actual_mean = number(&row[mean_index]);
        let actual_residual = number(&row[residual_index]);
        match (
          actual_mean,
          ref_row[if family == "poisson" { 0 } else { 2 }],
        ) {
          (Some(actual), Some(expected)) => close(actual, expected),
          (None, None) => {}
          other => panic!("current mean differs from base R for id={id}: {other:?}"),
        }
        match (
          actual_residual,
          ref_row[if family == "poisson" { 1 } else { 3 }],
        ) {
          (Some(actual), Some(expected)) => {
            close(actual, expected);
            let current_y = number(&row[y_index]).unwrap_or_else(|| {
              panic!("current y is not numeric for id={id}: {:?}", row[y_index])
            });
            let current_mean =
              actual_mean.unwrap_or_else(|| panic!("current mean is missing for id={id}"));
            close(actual, current_y - current_mean);
          }
          (None, None) => {}
          other => panic!("response residual differs from base R for id={id}: {other:?}"),
        }
        rows.push((actual_mean, actual_residual));
      }
      assert_eq!(session.state().estimation, retained);
      covariance_results.push(rows);
    }
    assert_eq!(covariance_results[0], covariance_results[1]);
    assert_eq!(covariance_results[0], covariance_results[2]);
  }
}

#[test]
fn response_residuals_validate_current_outcomes_and_fail_atomically() {
  let fixture = Fixture::new();
  for (family, mutation) in [
    ("poisson", "replace y = .5 if id == 1"),
    ("nbreg", "replace y = 100001 if id == 1"),
  ] {
    let mut session = fixture.session();
    run(&mut session, "generate log_time = .2 * x");
    run(&mut session, &format!("{family} y x z, offset(log_time)"));
    run(&mut session, mutation);
    let before = session.state();
    let error = session
      .execute(parse_command("predict invalid_residual, residuals").unwrap())
      .unwrap_err();
    assert!(
      error
        .to_string()
        .contains("current outcome must be an exact nonnegative integer")
    );
    assert_eq!(session.state(), before);
  }

  let mut collision = fixture.session();
  run(&mut collision, "generate log_time = .2 * x");
  run(&mut collision, "poisson y x z, offset(log_time)");
  run(&mut collision, "generate response_residual = 1");
  let before = collision.state();
  assert!(
    collision
      .execute(parse_command("predict response_residual, residuals").unwrap())
      .is_err()
  );
  assert_eq!(collision.state(), before);

  let mut non_numeric_outcome = fixture.session();
  run(&mut non_numeric_outcome, "generate log_time = .2 * x");
  run(&mut non_numeric_outcome, "poisson y x z, offset(log_time)");
  run(
    &mut non_numeric_outcome,
    r#"sql """SELECT * REPLACE (CAST(y AS VARCHAR) AS y) FROM active""" into string_outcome"#,
  );
  let before = non_numeric_outcome.state();
  let error = non_numeric_outcome
    .execute(parse_command("predict invalid_residual, residuals").unwrap())
    .unwrap_err();
  assert!(
    error
      .to_string()
      .contains("prediction requires numeric current outcome: y")
  );
  assert_eq!(non_numeric_outcome.state(), before);

  let mut missing_outcome = fixture.session();
  run(&mut missing_outcome, "generate log_time = .2 * x");
  run(&mut missing_outcome, "poisson y x z, offset(log_time)");
  run(&mut missing_outcome, "drop y");
  let before = missing_outcome.state();
  let error = missing_outcome
    .execute(parse_command("predict no_outcome, residuals").unwrap())
    .unwrap_err();
  assert!(matches!(error, RuntimeError::PredictUnknownVariable { .. }));
  assert_eq!(missing_outcome.state(), before);
}

#[test]
fn response_residuals_use_the_current_exposure() {
  let fixture = Fixture::new();
  for family in ["poisson", "nbreg"] {
    let mut session = fixture.session();
    run(&mut session, "generate exposure = 1 + .2 * (x + 3)");
    run(&mut session, &format!("{family} y x z, exposure(exposure)"));
    run(&mut session, "predict original_mean, n");
    run(&mut session, "replace exposure = exposure * 2");
    run(&mut session, "replace y = y + 1");
    run(&mut session, "predict current_mean, n");
    run(&mut session, "predict response_residual, residuals");

    let ExecutionResult::Head(table) = run(&mut session, "head 100") else {
      panic!("head")
    };
    assert_eq!(table.rows.len(), 36);
    let current_mean = table
      .columns
      .iter()
      .position(|name| name == "current_mean")
      .unwrap();
    let response_residual = table
      .columns
      .iter()
      .position(|name| name == "response_residual")
      .unwrap();
    for row in table.rows {
      close(
        number(&row[current_mean]).unwrap(),
        2.0 * number(&row[6]).unwrap(),
      );
      close(
        number(&row[response_residual]).unwrap(),
        number(&row[1]).unwrap() - number(&row[current_mean]).unwrap(),
      );
    }
  }
}

#[test]
fn response_residuals_support_empty_current_data() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  run(&mut session, "generate log_time = .2 * x");
  run(&mut session, "poisson y x z, offset(log_time)");
  run(&mut session, "drop if id >= 1");
  run(&mut session, "predict empty_residual, residuals");
  let ExecutionResult::Head(table) = run(&mut session, "head 10") else {
    panic!("head")
  };
  assert!(table.rows.is_empty());
  assert!(
    table
      .columns
      .iter()
      .any(|column| column == "empty_residual")
  );
}
