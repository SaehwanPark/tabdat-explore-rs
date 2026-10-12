#![cfg(feature = "survival-plugin")]
use gretl_sys::{GretlDataset, GretlModel};
#[test]
fn duration_lists_binary_censor_and_unidentified_sample_are_checked() {
  let mut dset = GretlDataset::new(5, 320);
  for list in [
    vec![5, 1, 0, 2, 3, -100, 4],
    vec![5, 1, 0, -100, 2, 3],
    vec![4, 1, 2, -100, 4],
    vec![4, 1, 0, -100, 0],
    vec![6, 1, 0, 2, 3, -100, 5],
  ] {
    assert!(std::panic::catch_unwind(|| GretlModel::survival(&dset, &list, false)).is_err());
  }
  let failed = GretlModel::survival(&dset, &[6, 1, 0, 2, 3, -100, 4], false);
  assert_ne!(failed.errcode(), 0);
  assert!(std::panic::catch_unwind(|| failed.vcv(0, 0)).is_err());
  for i in 1..5 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (t, row) in include_str!("../fixtures/survival.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let r: Vec<f64> = row.split(',').map(|v| v.parse().unwrap()).collect();
    dset.set_value(1, t, r[2]);
    dset.set_value(2, t, r[4]);
    dset.set_value(3, t, r[5]);
    dset.set_value(4, t, 1.);
  }
  let all_censored = GretlModel::survival(&dset, &[6, 1, 0, 2, 3, -100, 4], false);
  assert_ne!(all_censored.errcode(), 0);
  for t in 0..320 {
    dset.set_value(4, t, 0.);
  }
  // The low-level shim must normalize a redundant zero dummy too, not only the
  // facade: otherwise native precheck silently removes a real predictor.
  let complete = GretlModel::survival(&dset, &[6, 1, 0, 2, 3, -100, 4], false);
  assert_eq!(complete.errcode(), 0);
  assert_eq!(complete.ncoeff(), 4);
  assert_eq!(complete.nobs(), 320);
  dset.set_value(4, 0, 0.5);
  let invalid = GretlModel::survival(&dset, &[6, 1, 0, 2, 3, -100, 4], true);
  assert_ne!(invalid.errcode(), 0);
}
