use gretl_sys::{BinaryLink, GretlDataset, GretlModel, parameter_count_supported};
fn packed_panic<T>(call: impl FnOnce() -> T + std::panic::UnwindSafe) {
  let error = std::panic::catch_unwind(call)
    .err()
    .expect("oversized input must fail before native fit");
  let message = error
    .downcast_ref::<String>()
    .map(String::as_str)
    .or_else(|| error.downcast_ref::<&str>().copied())
    .unwrap();
  assert!(
    message.contains("native integer arithmetic") || message.contains("native arithmetic"),
    "{message}"
  );
}
#[test]
fn signed_packed_intermediate_threshold_is_checked() {
  assert!(!parameter_count_supported(0));
  assert!(parameter_count_supported(1));
  assert!(parameter_count_supported(46340));
  assert!(!parameter_count_supported(46341));
  assert!(!parameter_count_supported(usize::MAX));
}
#[test]
fn oversized_nominal_lists_and_auxiliary_parameters_fail_before_native_dispatch() {
  let mut dset = GretlDataset::new(2, 8);
  dset.set_varname(1, "y");
  for t in 0..8 {
    dset.set_value(1, t, 1. + t as f64);
  }
  let mut too_many = vec![1; 46343];
  too_many[0] = 46342;
  too_many[2] = 0;
  packed_panic(|| GretlModel::ols(&dset, &too_many));
  packed_panic(|| GretlModel::binary(&dset, &too_many, BinaryLink::Probit));
  packed_panic(|| GretlModel::panel(&dset, &too_many, 4, false));
  #[cfg(feature = "count-plugin")]
  packed_panic(|| GretlModel::poisson(&dset, &too_many));
  #[cfg(feature = "quantile-plugin")]
  packed_panic(|| GretlModel::quantile(&dset, &too_many, 0.5));
  #[cfg(any(
    feature = "count-plugin",
    feature = "tobit-plugin",
    feature = "survival-plugin"
  ))]
  let plus_aux = {
    let mut list = vec![1; 46342];
    list[0] = 46341;
    list[2] = 0;
    list
  };
  #[cfg(feature = "count-plugin")]
  packed_panic(|| GretlModel::negative_binomial(&dset, &plus_aux));
  #[cfg(feature = "tobit-plugin")]
  packed_panic(|| GretlModel::tobit(&dset, &plus_aux, 0., None));
  #[cfg(feature = "survival-plugin")]
  packed_panic(|| GretlModel::survival(&dset, &plus_aux, false));
  #[cfg(feature = "survival-plugin")]
  packed_panic(|| GretlModel::survival(&dset, &too_many, true));
  let mut iv = vec![1; 46345];
  iv[0] = 46344;
  iv[46343] = -100;
  packed_panic(|| GretlModel::iv(&dset, &iv));
  iv[2] = 0;
  iv[3] = -100;
  iv[46343] = 1;
  packed_panic(|| GretlModel::iv(&dset, &iv));
  iv[3] = 1;
  iv[4] = -100;
  packed_panic(|| GretlModel::iv(&dset, &iv)); // 46340 instruments + inserted constant
  let normal = GretlModel::ols(&dset, &[2, 1, 0]);
  assert_eq!(normal.errcode(), 0);
  assert!((normal.coeff(0) - 4.5).abs() < 1e-12);
}
