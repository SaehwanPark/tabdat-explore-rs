#![forbid(unsafe_code)]
use gretl_sys::{GretlDataset, GretlModel};
use std::time::Instant;
fn main() {
  let start = Instant::now();
  let variant = if std::env::args().any(|a| a == "--parallel") {
    std::thread::scope(|scope| {
      let handles: Vec<_> = (0..4)
        .map(|_| scope.spawn(gretl_sys::blas_variant_for_measurement))
        .collect();
      let variants: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("initialization worker"))
        .collect();
      assert!(variants.iter().all(|v| v == &variants[0]));
      variants[0].clone()
    })
  } else {
    gretl_sys::blas_variant_for_measurement()
  };
  let init = start.elapsed();
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let repeated = start.elapsed();
  let mut dset = GretlDataset::new(3, 30);
  dset.set_varname(1, "y");
  dset.set_varname(2, "x");
  for i in 0..30 {
    let x = i as f64;
    dset.set_value(1, i, 1. + 2. * x + [-0.05, 0.1, -0.05][i % 3]);
    dset.set_value(2, i, x);
  }
  let model = GretlModel::ols(&dset, &[3, 1, 0, 2]);
  assert_eq!(model.errcode(), 0);
  assert!((model.coeff(0) - 1.).abs() < 1e-10 && (model.coeff(1) - 2.).abs() < 1e-10);
  println!(
    "blas={variant} init_ns={} repeated_init_ns={} ols=ok",
    init.as_nanos(),
    repeated.as_nanos()
  );
}
