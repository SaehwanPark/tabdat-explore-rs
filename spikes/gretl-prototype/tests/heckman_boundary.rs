#![cfg(feature = "heckman-plugin")]
use gretl_sys::{BinaryLink, GretlDataset, GretlModel};
#[test]
fn selection_lists_and_early_second_stage_failure_restore_dataset() {
  let rows: Vec<Vec<f64>> = include_str!("../fixtures/heckman.csv")
    .lines()
    .skip(1)
    .map(|l| {
      let r: Vec<f64> = l.split(',').map(|v| v.parse().unwrap()).collect();
      vec![r[0], r[1], r[2], r[3], r[1], r[4]]
    })
    .collect();
  let mut dset = GretlDataset::new(7, 480);
  for i in 1..7 {
    dset.set_varname(i, &format!("td_v{i}"));
  }
  for (t, row) in rows.iter().enumerate() {
    for (i, v) in row.iter().enumerate() {
      dset.set_value(i + 1, t, *v);
    }
  }
  for list in [
    vec![8, 1, 0, 2, 3, -100, 4, 0, 5, 6],
    vec![8, 1, 0, 2, -100, 4, 0, -100, 6],
    vec![7, 1, 0, 2, -100, 4, 5, 6],
    vec![8, 1, 0, 2, -100, 4, 0, 5, 7],
  ] {
    assert!(std::panic::catch_unwind(|| GretlModel::heckman(&dset, &list)).is_err());
  }
  let mut oversized = vec![2; 46344];
  oversized[0] = 46343;
  oversized[1] = 1;
  oversized[2] = 0;
  oversized[4] = -100;
  oversized[5] = 4;
  oversized[6] = 0;
  assert!(std::panic::catch_unwind(|| GretlModel::heckman(&dset, &oversized)).is_err());
  for t in 0..480 {
    dset.set_value(4, t, if t == 3 || t == 173 { 1. } else { 0. });
  }
  let first = GretlModel::binary(&dset, &[4, 4, 0, 5, 6], BinaryLink::Probit);
  assert_eq!(
    first.errcode(),
    0,
    "first-stage precondition for early second-stage failure"
  );
  let failed = GretlModel::heckman(&dset, &[9, 1, 0, 2, 3, -100, 4, 0, 5, 6]);
  assert_ne!(failed.errcode(), 0);
  assert_eq!(
    dset.variable_count(),
    7,
    "temporary Mills column must be removed on early return"
  );
  assert!(std::panic::catch_unwind(|| failed.vcv(0, 0)).is_err());
  assert!(std::panic::catch_unwind(|| failed.selection_nobs()).is_err());
  for (t, row) in rows.iter().enumerate() {
    dset.set_value(4, t, row[3]);
  }
  let result = GretlModel::heckman(&dset, &[9, 1, 0, 2, 3, -100, 4, 0, 5, 6]);
  assert_eq!(result.errcode(), 0);
  assert_eq!(result.selection_nobs(), 480);
  assert_eq!(dset.variable_count(), 7);
}
