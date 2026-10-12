#![cfg(feature = "tobit-plugin")]
use tabdat_gretl_spike::{EstimationProblem, GretlError, run_tobit};
fn fixture(both: bool) -> EstimationProblem {
  let rows: Vec<Vec<f64>> = include_str!("../fixtures/tobit.csv")
    .lines()
    .skip(1)
    .map(|s| s.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: rows
      .iter()
      .map(|r| vec![r[usize::from(both)], r[2], r[3]])
      .collect(),
  }
}
#[test]
fn tobit_matches_observed_information_references() {
  for family in ["left", "both"] {
    let result = run_tobit(
      &fixture(family == "both"),
      0.,
      (family == "both").then_some(2.),
    )
    .unwrap();
    assert_eq!(result.sample_rows, (0..240).collect::<Vec<_>>());
    assert_eq!(
      result
        .coefficients
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>(),
      vec!["intercept", "x1", "x2"]
    );
    assert_eq!(result.covariance.len(), 4);
    assert!(result.covariance.iter().all(|r| r.len() == 4));
    let values: Vec<_> = result
      .coefficients
      .iter()
      .map(|c| c.value)
      .chain(result.coefficients.iter().map(|c| c.standard_error))
      .chain(result.covariance.iter().flatten().copied())
      .chain([result.log_sigma, result.log_likelihood])
      .collect();
    let references: Vec<_> = include_str!("../fixtures/tobit-reference.csv")
      .lines()
      .filter(|l| l.split(',').nth(1) == Some(family))
      .collect();
    assert_eq!(references.len(), 2);
    for line in references {
      let expected: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|v| v.parse().unwrap())
        .collect();
      assert_eq!(values.len(), expected.len());
      let maximum = values
        .iter()
        .zip(&expected)
        .map(|(v, e)| (v - e).abs())
        .fold(0.0_f64, f64::max);
      eprintln!(
        "{} {family} max absolute difference {maximum:.3e}",
        line.split(',').next().unwrap()
      );
      for (i, (v, e)) in values.iter().zip(expected).enumerate() {
        assert!(
          v.is_finite() && (v - e).abs() <= 1e-7 + 1e-6 * e.abs(),
          "{family} {i}: {v} != {e}"
        );
      }
    }
    let original = include_str!("../fixtures/tobit-original-python.csv")
      .lines()
      .find(|l| l.split(',').nth(1) == Some(family))
      .unwrap();
    let original: Vec<f64> = original
      .split(',')
      .skip(2)
      .map(|v| v.parse().unwrap())
      .collect();
    for (value, expected) in values[..3].iter().zip(&original[..3]) {
      assert!((value - expected).abs() <= 1e-7 + 1e-6 * expected.abs());
    }
    // Preserve the pinned Python fallback's quasi-Newton covariance discrepancy.
    // Its coefficients pass, but these SEs are explicitly NOT accepted references.
    assert!(
      values[3..6]
        .iter()
        .zip(&original[3..6])
        .any(|(v, e)| (v - e).abs() > 1e-4)
    );
  }
}
#[test]
fn tobit_invalid_native_failure_and_recovery() {
  let base = fixture(false);
  for (lower, upper) in [(f64::NAN, None), (0., Some(0.)), (0., Some(f64::INFINITY))] {
    assert!(run_tobit(&base, lower, upper).is_err());
  }
  let mut bad = base.clone();
  bad.data[0].pop();
  assert!(run_tobit(&bad, 0., None).is_err());
  bad = base.clone();
  bad.data[0][1] = f64::INFINITY;
  assert!(run_tobit(&bad, 0., None).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[0] = 0.;
  }
  assert!(run_tobit(&bad, 0., None).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = row[1];
  }
  assert!(run_tobit(&bad, 0., None).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = 1.;
  }
  assert!(run_tobit(&bad, 0., None).is_err());
  run_tobit(&base, 0., None).unwrap();
  let mut no_intercept = base;
  no_intercept.include_intercept = false;
  let result = run_tobit(&no_intercept, 0., None).unwrap();
  assert_eq!(result.coefficients.len(), 2);
  assert_eq!(result.covariance.len(), 3);
}
#[test]
fn independent_tobit_threads_serialize_plugin_calls() {
  std::thread::scope(|s| {
    for both in [false, true, false, true] {
      s.spawn(move || {
        let p = fixture(both);
        for _ in 0..5 {
          let r = run_tobit(&p, 0., both.then_some(2.)).unwrap();
          assert_eq!(r.sample_rows.len(), 240);
        }
      });
    }
  });
}
#[test]
#[ignore = "manual fresh process with actual linked interval.so temporarily unavailable"]
fn missing_tobit_plugin_returns_owned_error() {
  assert!(matches!(
    run_tobit(&fixture(false), 0., None),
    Err(GretlError::EstimationFailed { .. })
  ));
}
