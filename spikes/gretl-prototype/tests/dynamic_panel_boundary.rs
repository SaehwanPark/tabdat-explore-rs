#![cfg(feature = "dpanel-plugin")]
use gretl_sys::{GretlDataset, GretlModel};
#[test]
fn dynamic_lists_sample_buffers_and_dataset_metadata_are_guarded() {
  let rows: Vec<Vec<f64>> = include_str!("../fixtures/dynamic-panel.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  let mut dset = GretlDataset::new(4, 512);
  for i in 1..4 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (t, r) in rows.iter().enumerate() {
    for i in 0..3 {
      dset.set_value(i + 1, t, r[i + 2]);
    }
  }
  let before = dset.panel_shape();
  let list = [8, 1, -100, 1, 2, 3, -100, 2, 3];
  for malformed in [
    vec![7, 1, -100, 1, 2, 3, -100, 2, 3],
    vec![8, 2, -100, 1, 2, 3, -100, 2, 3],
    vec![8, 1, -100, 1, 0, 3, -100, 0, 3],
    vec![8, 1, -100, 1, 2, 4, -100, 2, 4],
    vec![8, 1, -100, 1, 2, 3, -100, 2, 2],
  ] {
    assert!(std::panic::catch_unwind(|| GretlModel::dynamic_panel(&dset, &malformed, 8)).is_err());
  }
  for periods in [0, 3, 7, 33] {
    assert!(std::panic::catch_unwind(|| GretlModel::dynamic_panel(&dset, &list, periods)).is_err());
  }
  let model = GretlModel::dynamic_panel(&dset, &list, 8);
  assert_eq!(model.errcode(), 0);
  assert_eq!(dset.panel_shape(), before);
  assert_eq!(dset.variable_count(), 4);
  assert!(std::panic::catch_unwind(|| model.sample_mask(511)).is_err());
  assert_eq!(model.sample_mask(512).iter().filter(|&&v| v).count(), 384);
  let ols = GretlModel::ols(&dset, &[4, 1, 0, 2, 3]);
  assert_eq!(ols.errcode(), 0);
  assert!(std::panic::catch_unwind(|| ols.sample_mask(512)).is_err());
  for (t, r) in rows.iter().enumerate() {
    dset.set_value(3, t, r[3]);
  }
  let failed = GretlModel::dynamic_panel(&dset, &list, 8);
  assert_ne!(failed.errcode(), 0);
  assert_eq!(dset.panel_shape(), before);
  assert_eq!(dset.variable_count(), 4);
  assert!(std::panic::catch_unwind(|| failed.dynamic_panel_instruments()).is_err());
  assert!(std::panic::catch_unwind(|| failed.sample_mask(512)).is_err());
  assert!(std::panic::catch_unwind(|| failed.vcv(0, 0)).is_err());
  for (t, r) in rows.iter().enumerate() {
    dset.set_value(3, t, r[4]);
  }
  let recovered = GretlModel::dynamic_panel(&dset, &list, 8);
  assert_eq!(recovered.errcode(), 0);
  assert_eq!(dset.panel_shape(), before);
}
