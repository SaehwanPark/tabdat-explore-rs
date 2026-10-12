#![cfg(feature = "survival-plugin")]
use tabdat_gretl_spike::{GretlError, SurvivalDistribution, SurvivalProblem, run_survival};
fn fixture(weibull: bool) -> SurvivalProblem {
  SurvivalProblem {
    time: "time".into(),
    failure: "failure".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/survival.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l.split(',').map(|v| v.parse().unwrap()).collect();
        let start = 2 * usize::from(weibull);
        vec![r[start], r[start + 1], r[4], r[5]]
      })
      .collect(),
  }
}
#[test]
fn survival_ph_covariance_matches_python_observed_and_r() {
  for (family, dist) in [
    ("exponential", SurvivalDistribution::Exponential),
    ("weibull", SurvivalDistribution::Weibull),
  ] {
    let problem = fixture(family == "weibull");
    let result = run_survival(&problem, dist).unwrap();
    let k = if family == "weibull" { 4 } else { 3 };
    assert_eq!(result.sample_rows, (0..320).collect::<Vec<_>>());
    assert_eq!(result.distribution, dist);
    assert_eq!(
      result.failure_rows,
      problem
        .data
        .iter()
        .enumerate()
        .filter(|(_, r)| r[1] == 1.)
        .map(|(i, _)| i)
        .collect::<Vec<_>>()
    );
    assert_eq!(
      result.censored_rows,
      problem
        .data
        .iter()
        .enumerate()
        .filter(|(_, r)| r[1] == 0.)
        .map(|(i, _)| i)
        .collect::<Vec<_>>()
    );
    assert_eq!(result.coefficients.len(), k);
    assert_eq!(result.covariance.len(), k);
    assert_eq!(
      result
        .coefficients
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>(),
      if k == 4 {
        vec!["intercept", "x1", "x2", "log_shape"]
      } else {
        vec!["intercept", "x1", "x2"]
      }
    );
    let values: Vec<_> = result
      .coefficients
      .iter()
      .map(|c| c.value)
      .chain(result.coefficients.iter().map(|c| c.standard_error))
      .chain(result.covariance.iter().flatten().copied())
      .chain([result.log_likelihood])
      .collect();
    let refs: Vec<_> = include_str!("../fixtures/survival-reference.csv")
      .lines()
      .filter(|l| l.split(',').nth(1) == Some(family))
      .collect();
    assert_eq!(refs.len(), 2);
    for line in refs {
      let expected: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|v| v.parse().unwrap())
        .collect();
      assert_eq!(expected.len(), values.len());
      let maximum = values
        .iter()
        .zip(&expected)
        .map(|(v, e)| (v - e).abs())
        .fold(0.0_f64, f64::max);
      eprintln!(
        "{} {family} max abs {maximum:.3e}",
        line.split(',').next().unwrap()
      );
      for (i, (v, e)) in values.iter().zip(expected).enumerate() {
        assert!(
          v.is_finite() && (v - e).abs() <= 1e-7 + 1e-6 * e.abs(),
          "{family} {i}: {v} != {e}"
        );
      }
    }
    let original: Vec<f64> = include_str!("../fixtures/survival-original-python.csv")
      .lines()
      .find(|l| l.split(',').nth(1) == Some(family))
      .unwrap()
      .split(',')
      .skip(2)
      .map(|v| v.parse().unwrap())
      .collect();
    for (v, e) in values[..3].iter().zip(&original[..3]) {
      assert!((v - e).abs() <= 1e-7 + 1e-6 * e.abs());
    }
    let error = values[k..k + 3]
      .iter()
      .zip(&original[3..6])
      .map(|(v, e)| (v - e).abs())
      .fold(0.0_f64, f64::max);
    eprintln!("original {family} approximate SE discrepancy {error:.3e}");
  }
}
#[test]
fn invalid_censoring_rank_and_all_event_recovery() {
  let base = fixture(true);
  let dist = SurvivalDistribution::Weibull;
  let mut bad = base.clone();
  bad.include_intercept = false;
  assert!(run_survival(&bad, dist).is_err());
  for value in [0., -1., f64::INFINITY] {
    bad = base.clone();
    bad.data[0][0] = value;
    assert!(run_survival(&bad, dist).is_err());
  }
  bad = base.clone();
  bad.data[0][1] = 0.5;
  assert!(run_survival(&bad, dist).is_err());
  bad = base.clone();
  bad.data[0].pop();
  assert!(run_survival(&bad, dist).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[1] = 0.;
  }
  assert!(run_survival(&bad, dist).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[3] = row[2];
  }
  assert!(run_survival(&bad, dist).is_err());
  bad = base.clone();
  for row in &mut bad.data {
    row[3] = 1. - row[1];
  }
  assert!(run_survival(&bad, dist).is_err());
  run_survival(&base, dist).unwrap();
  for family in [
    SurvivalDistribution::Exponential,
    SurvivalDistribution::Weibull,
  ] {
    let mut complete = base.clone();
    for row in &mut complete.data {
      row[1] = 1.;
    }
    let result = run_survival(&complete, family).unwrap();
    assert_eq!(result.failure_rows.len(), 320);
    assert!(result.censored_rows.is_empty());
    assert_eq!(
      result.coefficients.len(),
      if family == SurvivalDistribution::Weibull {
        4
      } else {
        3
      }
    );
  }
}
#[test]
fn time_unit_change_preserves_hazard_and_event_density_jacobian() {
  for distribution in [
    SurvivalDistribution::Exponential,
    SurvivalDistribution::Weibull,
  ] {
    let mut problem = fixture(distribution == SurvivalDistribution::Weibull);
    let original = run_survival(&problem, distribution).unwrap();
    let shape = if distribution == SurvivalDistribution::Weibull {
      original.coefficients[3].value.exp()
    } else {
      1.0
    };
    for row in &mut problem.data {
      row[0] *= 2.0;
    }
    let scaled = run_survival(&problem, distribution).unwrap();
    let expected = original.coefficients[0].value - shape * 2.0_f64.ln();
    assert!((scaled.coefficients[0].value - expected).abs() <= 1e-7 + 1e-6 * expected.abs());
    for (a, b) in scaled.coefficients[1..]
      .iter()
      .zip(&original.coefficients[1..])
    {
      assert!((a.value - b.value).abs() <= 1e-7 + 1e-6 * b.value.abs());
    }
    let expected = original.log_likelihood - original.failure_rows.len() as f64 * 2.0_f64.ln();
    assert!((scaled.log_likelihood - expected).abs() <= 1e-7 + 1e-6 * expected.abs());
  }
}
#[test]
fn survival_parallel_families_serialize() {
  std::thread::scope(|s| {
    for weibull in [false, true, false, true] {
      s.spawn(move || {
        let problem = fixture(weibull);
        let dist = if weibull {
          SurvivalDistribution::Weibull
        } else {
          SurvivalDistribution::Exponential
        };
        for _ in 0..5 {
          run_survival(&problem, dist).unwrap();
        }
      });
    }
  });
}
#[test]
#[ignore = "manual fresh process with actual linked duration.so temporarily unavailable"]
fn missing_survival_plugin_returns_owned_error() {
  assert!(matches!(
    run_survival(&fixture(true), SurvivalDistribution::Weibull),
    Err(GretlError::EstimationFailed { .. })
  ));
}
