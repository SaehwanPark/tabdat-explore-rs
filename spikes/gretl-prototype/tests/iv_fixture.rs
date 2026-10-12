#![forbid(unsafe_code)]
use tabdat_gretl_spike::{IvProblem, run_iv};
fn fixture() -> IvProblem {
  IvProblem {
    outcome: "y".into(),
    exogenous: vec!["x".into()],
    endogenous: "d".into(),
    instruments: vec!["z1".into(), "z2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/iv.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}
#[test]
fn iv_matches_python_and_independent_r_unadjusted_covariance() {
  let r = run_iv(&fixture()).unwrap();
  assert_eq!(r.sample_rows, (0..160).collect::<Vec<_>>());
  assert_eq!(
    r.coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    ["intercept", "x", "d"]
  );
  let mut a: Vec<_> = r.coefficients.iter().map(|c| c.value).collect();
  a.extend(r.coefficients.iter().map(|c| c.standard_error));
  a.extend(r.covariance.iter().flatten());
  a.push(r.r_squared);
  println!("native IV: {a:?}");
  for l in include_str!("../fixtures/iv-reference.csv").lines() {
    let f: Vec<_> = l.split(',').collect();
    let e: Vec<f64> = f[2..].iter().map(|v| v.parse().unwrap()).collect();
    assert_eq!(a.len(), e.len());
    for (a, e) in a.iter().zip(e) {
      assert!(a.is_finite() && e.is_finite());
      assert!(
        (a - e).abs() <= 1e-10 + 1e-9 * e.abs(),
        "{}: {a} != {e}",
        f[0]
      );
    }
  }
}
#[test]
fn iv_rejects_invalid_and_underidentified_inputs_and_recovers() {
  let mut p = fixture();
  p.data[0][0] = f64::NAN;
  assert!(run_iv(&p).is_err());
  p = fixture();
  p.data[0].pop();
  assert!(run_iv(&p).is_err());
  p = fixture();
  p.instruments.clear();
  assert!(run_iv(&p).is_err());
  p = fixture();
  for r in &mut p.data {
    r[3] = r[1];
    r[4] = r[1];
  }
  assert!(run_iv(&p).is_err());
  assert!(run_iv(&fixture()).is_ok());
}
#[test]
fn low_iv_lists_are_checked_before_c() {
  use gretl_sys::{GretlDataset, GretlModel};
  let d = GretlDataset::new(5, 10);
  for list in [
    vec![4, 1, 2, -100, 9],
    vec![4, 0, 2, -100, 3],
    vec![4, 1, -100, -100, 3],
    vec![4, 1, 2, 3, 4],
    vec![4, 1, 2, 3, -100],
    vec![4, 1, -1, -100, 3],
  ] {
    assert!(
      std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| GretlModel::iv(&d, &list))).is_err()
    );
  }
}
