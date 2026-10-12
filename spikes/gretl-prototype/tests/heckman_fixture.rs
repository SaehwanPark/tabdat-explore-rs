#![cfg(feature = "heckman-plugin")]
use tabdat_gretl_spike::{GretlError, HeckmanProblem, run_heckman};
fn fixture() -> HeckmanProblem {
  HeckmanProblem {
    outcome: "y".into(),
    outcome_predictors: vec!["x1".into(), "x2".into()],
    selection: "selected".into(),
    selection_predictors: vec!["x1".into(), "z".into()],
    include_intercept: true,
    data: include_str!("../fixtures/heckman.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l.split(',').map(|v| v.parse().unwrap()).collect();
        vec![r[0], r[1], r[2], r[3], r[1], r[4]]
      })
      .collect(),
  }
}
#[test]
fn two_step_coefficients_and_corrected_equation_covariances_match_references() {
  let problem = fixture();
  let result = run_heckman(&problem).unwrap();
  assert_eq!(result.selection_rows, (0..480).collect::<Vec<_>>());
  assert_eq!(
    result.outcome_rows,
    problem
      .data
      .iter()
      .enumerate()
      .filter(|(_, r)| r[3] == 1.)
      .map(|(i, _)| i)
      .collect::<Vec<_>>()
  );
  assert_eq!(
    result
      .outcome_coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    vec!["intercept", "x1", "x2", "mills_lambda"]
  );
  assert_eq!(
    result
      .selection_coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    vec!["intercept", "x1", "z"]
  );
  assert_eq!(result.outcome_covariance.len(), 4);
  assert_eq!(result.selection_covariance.len(), 3);
  let values: Vec<_> = result
    .outcome_coefficients
    .iter()
    .chain(&result.selection_coefficients)
    .map(|c| c.value)
    .chain(
      result
        .outcome_coefficients
        .iter()
        .chain(&result.selection_coefficients)
        .map(|c| c.standard_error),
    )
    .chain(result.outcome_covariance.iter().flatten().copied())
    .chain(result.selection_covariance.iter().flatten().copied())
    .chain([result.sigma, result.rho])
    .collect();
  let references: Vec<_> = include_str!("../fixtures/heckman-reference.csv")
    .lines()
    .collect();
  assert_eq!(references.len(), 2);
  for line in references {
    let expected: Vec<f64> = line
      .split(',')
      .skip(1)
      .map(|v| v.parse().unwrap())
      .collect();
    assert_eq!(values.len(), expected.len());
    let max = values
      .iter()
      .zip(&expected)
      .map(|(v, e)| (v - e).abs())
      .fold(0.0_f64, f64::max);
    eprintln!(
      "{} corrected max difference {max:.3e}",
      line.split(',').next().unwrap()
    );
    for (i, (v, e)) in values.iter().zip(expected).enumerate() {
      assert!(
        v.is_finite() && (v - e).abs() <= 1e-7 + 1e-6 * e.abs(),
        "{i}: {v} != {e}"
      );
    }
  }
  let original: Vec<f64> = include_str!("../fixtures/heckman-original-python.csv")
    .trim()
    .split(',')
    .skip(1)
    .map(|v| v.parse().unwrap())
    .collect();
  for (v, e) in values[..7].iter().zip(&original[..7]) {
    assert!((v - e).abs() <= 1e-7 + 1e-6 * e.abs());
  }
  let error = values[7..11]
    .iter()
    .zip(&original[7..11])
    .map(|(v, e)| (v - e).abs())
    .fold(0.0_f64, f64::max);
  eprintln!("original ordinary-OLS outcome SE max discrepancy {error:.3e}");
}
#[test]
fn invalid_selection_equation_and_native_failure_recover() {
  let base = fixture();
  let mut bad = base.clone();
  bad.include_intercept = false;
  assert!(run_heckman(&bad).is_err());
  bad = base.clone();
  bad.data[0].pop();
  assert!(run_heckman(&bad).is_err());
  bad = base.clone();
  bad.data[0][0] = f64::NAN;
  assert!(run_heckman(&bad).is_err());
  bad = base.clone();
  bad.data[0][3] = 0.5;
  assert!(run_heckman(&bad).is_err());
  bad = base.clone();
  bad.data[0][4] += 1.;
  assert!(run_heckman(&bad).is_err());
  for (outcome, selection, predictor) in [
    ("", "selected", "x2"),
    ("y", "mills_lambda", "x2"),
    ("y", "selected", "x1"),
    ("y\0", "selected", "x2"),
  ] {
    bad = base.clone();
    bad.outcome = outcome.into();
    bad.selection = selection.into();
    bad.outcome_predictors[1] = predictor.into();
    assert!(matches!(
      run_heckman(&bad),
      Err(GretlError::InvalidProblem(_))
    ));
  }
  bad = base.clone();
  bad.selection_predictors[1] = "y".into();
  assert!(matches!(
    run_heckman(&bad),
    Err(GretlError::InvalidProblem(_))
  ));
  bad = base.clone();
  bad.outcome_predictors[1] = "selected".into();
  assert!(matches!(
    run_heckman(&bad),
    Err(GretlError::InvalidProblem(_))
  ));
  for value in [0., 1.] {
    bad = base.clone();
    for row in &mut bad.data {
      row[3] = value;
    }
    assert!(run_heckman(&bad).is_err());
  }
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = row[1];
  }
  assert!(run_heckman(&bad).is_err());
  run_heckman(&base).unwrap();
}
#[test]
fn unselected_outcomes_do_not_enter_second_stage() {
  let mut problem = fixture();
  let expected = run_heckman(&problem).unwrap();
  for row in &mut problem.data {
    if row[3] == 0. {
      row[0] = 12345.;
    }
  }
  let result = run_heckman(&problem).unwrap();
  assert_eq!(result.outcome_coefficients, expected.outcome_coefficients);
  assert_eq!(result.outcome_rows, expected.outcome_rows);
}
#[test]
fn parallel_selection_fits_serialize() {
  std::thread::scope(|s| {
    for _ in 0..4 {
      s.spawn(|| {
        let problem = fixture();
        for _ in 0..5 {
          run_heckman(&problem).unwrap();
        }
      });
    }
  });
}
#[test]
#[ignore = "manual fresh process with actual linked heckit.so temporarily unavailable"]
fn missing_heckman_plugin_returns_owned_error() {
  assert!(matches!(
    run_heckman(&fixture()),
    Err(GretlError::EstimationFailed { .. })
  ));
}
