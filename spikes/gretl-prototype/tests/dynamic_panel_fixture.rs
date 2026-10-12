#![cfg(feature = "dpanel-plugin")]
use tabdat_gretl_spike::{GretlError, PanelProblem, run_dynamic_panel};
fn fixture() -> PanelProblem {
  let rows: Vec<Vec<f64>> = include_str!("../fixtures/dynamic-panel.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  PanelProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    keys: rows
      .iter()
      .map(|r| (r[0] as usize, r[1] as usize))
      .collect(),
    periods: 8,
    data: rows.iter().map(|r| r[2..].to_vec()).collect(),
  }
}
#[test]
fn difference_points_entity_covariance_and_native_keys_match_references() {
  let result = run_dynamic_panel(&fixture()).unwrap();
  assert_eq!(result.groups, 64);
  assert_eq!(result.instruments, 3);
  assert_eq!(
    result.sample_rows,
    (0..512).filter(|i| i % 8 >= 2).collect::<Vec<_>>()
  );
  assert_eq!(
    result.keys,
    result
      .sample_rows
      .iter()
      .map(|i| (i / 8, i % 8))
      .collect::<Vec<_>>()
  );
  assert_eq!(
    result
      .coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    ["L1.y", "x1", "x2"]
  );
  let values: Vec<_> = result
    .coefficients
    .iter()
    .map(|c| c.value)
    .chain(result.coefficients.iter().map(|c| c.standard_error))
    .chain(result.covariance.iter().flatten().copied())
    .collect();
  eprintln!("native {values:?}");
  for line in include_str!("../fixtures/dynamic-panel-reference.csv").lines() {
    let expected: Vec<f64> = line
      .split(',')
      .skip(1)
      .map(|v| v.parse().unwrap())
      .collect();
    assert_eq!(expected.len(), values.len());
    let max = values
      .iter()
      .zip(&expected)
      .map(|(v, e)| (v - e).abs())
      .fold(0., f64::max);
    eprintln!("{} max {max:.3e}", line.split(',').next().unwrap());
    for (v, e) in values.iter().zip(expected) {
      assert!(
        v.is_finite() && (v - e).abs() <= 1e-9 + 1e-8 * e.abs(),
        "{v} != {e}"
      );
    }
  }
  let original: Vec<f64> = include_str!("../fixtures/dynamic-panel-original-python.csv")
    .trim()
    .split(',')
    .skip(1)
    .map(|v| v.parse().unwrap())
    .collect();
  for (v, e) in values[..3].iter().zip(&original[..3]) {
    assert!((v - e).abs() <= 1e-9 + 1e-8 * e.abs());
  }
  eprintln!(
    "original independent-row SE discrepancy {:.3e}",
    values[3..6]
      .iter()
      .zip(&original[3..6])
      .map(|(v, e)| (v - e).abs())
      .fold(0., f64::max)
  );
}
#[test]
fn invalid_dynamic_shapes_names_rank_and_recovery() {
  let base = fixture();
  let mut bad = base.clone();
  for periods in [0, 1, 3, 33, 7] {
    bad = base.clone();
    bad.periods = periods;
    assert!(run_dynamic_panel(&bad).is_err());
  }
  bad.keys[2] = bad.keys[1];
  assert!(run_dynamic_panel(&bad).is_err());
  bad = base.clone();
  bad.data[0].pop();
  assert!(run_dynamic_panel(&bad).is_err());
  bad = base.clone();
  bad.data[0][0] = f64::NAN;
  assert!(run_dynamic_panel(&bad).is_err());
  bad = base.clone();
  bad.predictors[0] = "L1.y".into();
  assert!(matches!(
    run_dynamic_panel(&bad),
    Err(GretlError::InvalidProblem(_))
  ));
  bad = base.clone();
  for r in &mut bad.data {
    r[2] = r[1];
  }
  assert!(run_dynamic_panel(&bad).is_err());
  run_dynamic_panel(&base).unwrap();
}
#[test]
fn independent_dynamic_threads_serialize() {
  std::thread::scope(|scope| {
    for _ in 0..4 {
      scope.spawn(|| {
        for _ in 0..5 {
          run_dynamic_panel(&fixture()).unwrap();
        }
      });
    }
  });
}
#[test]
#[ignore = "manual fresh process with actual linked dpanel.so unavailable"]
fn missing_dpanel_plugin_returns_owned_error() {
  assert!(matches!(
    run_dynamic_panel(&fixture()),
    Err(GretlError::EstimationFailed { .. })
  ));
}
