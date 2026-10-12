#![forbid(unsafe_code)]

use gretl_sys::{GretlDataset, GretlModel};
use tabdat_gretl_spike::{EstimationProblem, run_ols};

fn problem(intercept: bool) -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x".into()],
    include_intercept: intercept,
    data: (1..=12)
      .map(|x| {
        vec![
          2.0 * x as f64 + if x % 2 == 0 { 0.1 } else { -0.1 },
          x as f64,
        ]
      })
      .collect(),
  }
}

#[test]
fn no_intercept_keeps_the_reserved_constant_slot() {
  let result = run_ols(&problem(false)).unwrap();
  // Independent scalar least-squares formula: sum(x*y) / sum(x*x).
  let expected = problem(false).data.iter().map(|r| r[0] * r[1]).sum::<f64>()
    / problem(false).data.iter().map(|r| r[1] * r[1]).sum::<f64>();
  assert_eq!(result.ncoeff, 1);
  assert_eq!(result.coefficients[0].name, "x");
  assert!((result.coefficients[0].value - expected).abs() < 1e-12);
}

#[test]
fn rejects_dimensions_before_native_allocation() {
  for (vars, obs) in [(0, 1), (1, 0), (usize::MAX, 1), (1, usize::MAX)] {
    assert!(std::panic::catch_unwind(|| GretlDataset::new(vars, obs)).is_err());
  }
}

#[test]
fn rejects_indices_and_malformed_lists_before_c() {
  let mut dataset = GretlDataset::new(3, 12);
  for t in 0..12 {
    dataset.set_value(1, t, 2.0 * t as f64 + (t % 2) as f64);
    dataset.set_value(2, t, t as f64);
  }
  assert!(
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dataset.set_value(3, 0, 0.0)))
      .is_err()
  );
  assert!(
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(
      || dataset.set_value(1, 12, 0.0)
    ))
    .is_err()
  );
  assert!(
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(
      || dataset.set_varname(usize::MAX, "x")
    ))
    .is_err()
  );
  for list in [
    vec![],
    vec![99, 1, 2],
    vec![-1],
    vec![2, 0, 2],
    vec![2, 1, -1],
    vec![2, 1, 3],
  ] {
    assert!(std::panic::catch_unwind(|| GretlModel::ols(&dataset, &list)).is_err());
  }
  let model = GretlModel::ols(&dataset, &[3, 1, 0, 2]);
  assert_eq!(model.errcode(), 0);
  assert!(std::panic::catch_unwind(|| model.coeff(usize::MAX)).is_err());
  assert!(std::panic::catch_unwind(|| model.sderr(model.ncoeff())).is_err());
  assert!(std::panic::catch_unwind(|| model.vcv(0, model.ncoeff())).is_err());
}

#[test]
fn failed_models_cannot_expose_coefficient_buffers() {
  let mut dataset = GretlDataset::new(3, 8);
  for t in 0..8 {
    dataset.set_value(1, t, f64::NAN);
    dataset.set_value(2, t, t as f64);
  }
  let model = GretlModel::ols(&dataset, &[3, 1, 0, 2]);
  assert_ne!(model.errcode(), 0);
  assert!(std::panic::catch_unwind(|| model.coeff(0)).is_err());
  assert!(std::panic::catch_unwind(|| model.sderr(0)).is_err());
  assert!(std::panic::catch_unwind(|| model.vcv(0, 0)).is_err());
}

#[test]
fn independent_threads_fit_and_drop_repeatedly() {
  let workers: Vec<_> = (0..4)
    .map(|_| {
      std::thread::spawn(|| {
        for _ in 0..20 {
          let result = run_ols(&problem(true)).unwrap();
          assert_eq!(result.nobs, 12);
          assert_eq!(result.ncoeff, 2);
          assert!(result.coefficients.iter().all(|c| c.value.is_finite()));
        }
      })
    })
    .collect();
  for worker in workers {
    worker.join().unwrap();
  }
}

#[test]
fn panel_boundary_rejects_invalid_lists_and_periods_before_ffi() {
  let dataset = GretlDataset::new(3, 12);
  for (list, periods) in [
    (vec![2, 1, 0], 3),
    (vec![2, 1, 2], 3),
    (vec![3, 1, 0, 2], 0),
    (vec![3, 1, 0, 2], 5),
    (vec![3, 1, 0, 2], 12),
  ] {
    assert!(
      std::panic::catch_unwind(|| GretlModel::panel(&dataset, &list, periods, false)).is_err()
    );
  }
}
