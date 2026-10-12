#![cfg(feature = "quantile-plugin")]
use gretl_sys::{GretlDataset, GretlModel, normal_quantile};
#[test]
fn quantile_checked_tau_lists_and_scalar_probability() {
  let dset = GretlDataset::new(3, 12);
  for q in [f64::NAN, 0., 1., 0.005] {
    assert!(std::panic::catch_unwind(|| GretlModel::quantile(&dset, &[3, 1, 0, 2], q)).is_err());
  }
  assert!(std::panic::catch_unwind(|| GretlModel::quantile(&dset, &[2, 1, 0, 2], 0.5)).is_err());
  for q in [0., 1., f64::NAN] {
    assert!(std::panic::catch_unwind(|| normal_quantile(q)).is_err());
  }
  assert!(normal_quantile(0.5).abs() < 1e-14);
  assert!((normal_quantile(0.975) - 1.959963984540054).abs() < 1e-12);
  let failed = GretlModel::quantile(&dset, &[3, 1, 0, 2], 0.5);
  assert_ne!(failed.errcode(), 0);
  assert!(std::panic::catch_unwind(|| failed.vcv(0, 0)).is_err());
}
