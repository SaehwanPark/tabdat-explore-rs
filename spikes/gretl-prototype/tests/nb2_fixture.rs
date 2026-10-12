#![forbid(unsafe_code)]
#![cfg(feature = "count-plugin")]
use tabdat_gretl_spike::{CountFamily, EstimationProblem, run_negative_binomial, run_poisson};
fn fixture() -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/nb2.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}
#[test]
fn nb2_matches_python_and_r_joint_observed_covariance() {
  let r = run_negative_binomial(&fixture()).unwrap();
  assert_eq!(r.family, CountFamily::NegativeBinomial2);
  assert_eq!(r.sample_rows, (0..320).collect::<Vec<_>>());
  assert_eq!(
    r.coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    ["intercept", "x1", "x2", "alpha"]
  );
  let mut a: Vec<_> = r.coefficients.iter().map(|c| c.value).collect();
  a.extend(r.coefficients.iter().map(|c| c.standard_error));
  a.extend(r.covariance.iter().flatten());
  a.push(r.log_likelihood);
  println!("native NB2: {a:?}");
  for l in include_str!("../fixtures/nb2-reference.csv").lines() {
    let f: Vec<_> = l.split(',').collect();
    let e: Vec<f64> = f[2..].iter().map(|v| v.parse().unwrap()).collect();
    assert_eq!(a.len(), e.len());
    for (a, e) in a.iter().zip(e) {
      assert!(a.is_finite() && e.is_finite());
      assert!(
        (a - e).abs() <= 1e-7 + 1e-6 * e.abs(),
        "{}: {a} != {e}",
        f[0]
      );
    }
  }
}
#[test]
fn nb2_invalid_data_and_rank_failure_recover_without_poisson_state_leakage() {
  for v in [-1.0, 0.5, f64::NAN] {
    let mut p = fixture();
    p.data[0][0] = v;
    assert!(run_negative_binomial(&p).is_err());
  }
  let mut p = fixture();
  for row in &mut p.data {
    row[2] = row[1];
  }
  assert!(run_negative_binomial(&p).is_err());
  for _ in 0..5 {
    assert!(run_negative_binomial(&fixture()).is_ok());
    assert_eq!(
      run_poisson(&fixture()).unwrap().family,
      CountFamily::Poisson
    );
  }
}
