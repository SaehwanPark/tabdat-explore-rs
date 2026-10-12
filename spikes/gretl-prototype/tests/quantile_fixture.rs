#![cfg(feature = "quantile-plugin")]
use tabdat_gretl_spike::{EstimationProblem, GretlError, run_quantile};
fn fixture() -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/quantile.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}
#[test]
fn quantiles_match_lp_r_and_canonical_iid_covariance() {
  for q in [0.25, 0.5, 0.75] {
    let result = run_quantile(&fixture(), q).unwrap();
    assert_eq!(result.sample_rows, (0..360).collect::<Vec<_>>());
    assert_eq!(result.quantile, q);
    assert_eq!(
      result
        .coefficients
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>(),
      vec!["intercept", "x1", "x2"]
    );
    assert_eq!(result.covariance.len(), 3);
    assert!(result.covariance.iter().all(|r| r.len() == 3));
    let values: Vec<_> = result
      .coefficients
      .iter()
      .map(|c| c.value)
      .chain(result.coefficients.iter().map(|c| c.standard_error))
      .chain(result.covariance.iter().flatten().copied())
      .chain([
        result.bandwidth,
        result.residual_density,
        result.pseudo_r_squared,
      ])
      .collect();
    let references: Vec<_> = include_str!("../fixtures/quantile-reference.csv")
      .lines()
      .filter(|l| l.split(',').nth(1).unwrap().parse::<f64>().unwrap() == q)
      .collect();
    assert_eq!(references.len(), 2);
    for line in references {
      let reference: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|v| v.parse().unwrap())
        .collect();
      assert_eq!(reference.len(), values.len());
      let maximum = values
        .iter()
        .zip(&reference)
        .map(|(v, e)| (v - e).abs())
        .fold(0.0_f64, f64::max);
      eprintln!(
        "{} q={q} maximum abs {maximum:.3e}",
        line.split(',').next().unwrap()
      );
      for (i, (v, e)) in values.iter().zip(reference).enumerate() {
        assert!(
          v.is_finite() && (v - e).abs() <= 1e-8 + 1e-7 * e.abs(),
          "q={q} {i}: {v} != {e}"
        );
      }
    }
    let original = include_str!("../fixtures/quantile-original-python.csv")
      .lines()
      .find(|l| l.split(',').nth(1).unwrap().parse::<f64>().unwrap() == q)
      .unwrap();
    let original: Vec<f64> = original
      .split(',')
      .skip(2)
      .map(|v| v.parse().unwrap())
      .collect();
    let error = values[..3]
      .iter()
      .zip(&original[..3])
      .map(|(v, e)| (v - e).abs())
      .fold(0.0_f64, f64::max);
    eprintln!("original IRLS q={q} coefficient max discrepancy {error:.3e}");
    // Actual default IRLS is retained as disagreement evidence, not an LP oracle.
    assert!(error > 1e-7);
    let objective = |beta: &[f64]| {
      fixture()
        .data
        .iter()
        .map(|row| {
          let e = row[0] - beta[0] - row[1] * beta[1] - row[2] * beta[2];
          if e >= 0.0 { q * e } else { (q - 1.0) * e }
        })
        .sum::<f64>()
    };
    let gap = objective(&original[..3]) - objective(&values[..3]);
    eprintln!("original IRLS q={q} objective excess {gap:.9e}");
    assert!(
      gap > 1e-8,
      "difference is not only an equally optimal representation"
    );
    let r_iid = include_str!("../fixtures/quantile-r-native-iid.csv")
      .lines()
      .find(|l| l.split(',').nth(1).unwrap().parse::<f64>().unwrap() == q)
      .unwrap();
    let r_iid: Vec<f64> = r_iid
      .split(',')
      .skip(2)
      .map(|v| v.parse().unwrap())
      .collect();
    let difference = result
      .native_iid_covariance
      .iter()
      .flatten()
      .zip(r_iid)
      .map(|(v, e)| (v - e).abs())
      .fold(0.0_f64, f64::max);
    eprintln!("native vs R iid q={q} covariance max difference {difference:.3e}");
    // Native sparsity is diagnostic only: do not require a known discrepancy
    // to recur on every BLAS/platform or reject an improved backend result.
    for (i, row) in result.native_iid_covariance.iter().enumerate() {
      assert!(row[i] >= 0.0);
    }
  }
}
#[test]
fn invalid_quantile_and_rank_failure_recover() {
  let base = fixture();
  for q in [f64::NAN, 0., 1., 0.005] {
    assert!(run_quantile(&base, q).is_err());
  }
  let mut bad = base.clone();
  bad.data[0].pop();
  assert!(run_quantile(&bad, 0.5).is_err());
  bad = base.clone();
  bad.data[0][0] = f64::INFINITY;
  assert!(run_quantile(&bad, 0.5).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[2] = row[1];
  }
  assert!(run_quantile(&bad, 0.5).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[0] = 1.;
  }
  assert!(run_quantile(&bad, 0.5).is_err());
  run_quantile(&base, 0.5).unwrap();
  let mut no_intercept = base;
  no_intercept.include_intercept = false;
  let result = run_quantile(&no_intercept, 0.5).unwrap();
  assert_eq!(result.coefficients.len(), 2);
}
#[test]
#[ignore = "manual fresh process with actual linked quantreg.so temporarily unavailable"]
fn missing_quantile_plugin_returns_owned_error() {
  assert!(matches!(
    run_quantile(&fixture(), 0.5),
    Err(GretlError::EstimationFailed { .. })
  ));
}
#[test]
fn parallel_quantile_calls_serialize() {
  std::thread::scope(|s| {
    for q in [0.25, 0.5, 0.75] {
      s.spawn(move || {
        for _ in 0..5 {
          run_quantile(&fixture(), q).unwrap();
        }
      });
    }
  });
}
