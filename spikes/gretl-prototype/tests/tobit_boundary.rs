#![cfg(feature = "tobit-plugin")]
use gretl_sys::{GretlDataset, GretlModel};
#[test]
fn tobit_bounds_shape_and_failed_buffer_checks() {
  let empty = GretlDataset::new(4, 240);
  for (lower, upper) in [(f64::NAN, None), (0., Some(0.)), (0., Some(f64::INFINITY))] {
    assert!(
      std::panic::catch_unwind(|| GretlModel::tobit(&empty, &[4, 1, 0, 2, 3], lower, upper))
        .is_err()
    );
  }
  let failed = GretlModel::tobit(&empty, &[4, 1, 0, 2, 3], 0., None);
  assert_ne!(failed.errcode(), 0);
  assert!(std::panic::catch_unwind(|| failed.tobit_vcv(0, 0)).is_err());
  let mut dset = GretlDataset::new(4, 240);
  for (t, row) in include_str!("../fixtures/tobit.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let values: Vec<f64> = row.split(',').map(|v| v.parse().unwrap()).collect();
    for (i, value) in [values[0], values[2], values[3]].into_iter().enumerate() {
      dset.set_value(i + 1, t, value);
    }
  }
  let model = GretlModel::tobit(&dset, &[4, 1, 0, 2, 3], 0., None);
  assert_eq!(model.errcode(), 0);
  assert!(std::panic::catch_unwind(|| model.tobit_vcv(4, 0)).is_err());
  assert!(model.tobit_vcv(3, 3).is_finite());
  let ols = GretlModel::ols(&dset, &[4, 1, 0, 2, 3]);
  assert_eq!(ols.errcode(), 0);
  assert!(ols.tobit_vcv(0, 0).is_nan());
}
