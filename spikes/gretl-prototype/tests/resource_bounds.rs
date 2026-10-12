use gretl_sys::{GretlDataset, GretlModel, dataset_dimensions_supported};

#[test]
fn dataset_budget_is_checked_without_allocating_extreme_inputs() {
  for (variables, observations) in [
    (0, 8),
    (2, 0),
    (4097, 1),
    (2, 1_000_001),
    (4096, 2442),
    (usize::MAX, usize::MAX),
  ] {
    assert!(!dataset_dimensions_supported(variables, observations));
    assert!(std::panic::catch_unwind(|| GretlDataset::new(variables, observations)).is_err());
  }
  assert!(dataset_dimensions_supported(4096, 2441));
  assert!(dataset_dimensions_supported(10, 1_000_000));
}

#[test]
fn repeated_index_work_budget_rejects_before_native_fit_then_recovers() {
  let mut data = GretlDataset::new(2, 8);
  for t in 0..8 {
    data.set_value(1, t, t as f64);
  }
  let mut list = vec![1; 1027];
  list[0] = 1026;
  list[2] = 0;
  let error = std::panic::catch_unwind(|| GretlModel::ols(&data, &list))
    .err()
    .unwrap();
  let text = error
    .downcast_ref::<&str>()
    .copied()
    .or_else(|| error.downcast_ref::<String>().map(String::as_str))
    .unwrap();
  assert!(text.contains("work budget"));
  let model = GretlModel::ols(&data, &[2, 1, 0]);
  assert_eq!(model.errcode(), 0);
  drop(data);
  // The copied model must outlive the input dataset, not borrow its storage.
  assert!((model.coeff(0) - 3.5).abs() < 1e-12);
  assert!(model.vcv(0, 0).is_finite());
}
