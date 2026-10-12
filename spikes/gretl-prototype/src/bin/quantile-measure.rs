#![forbid(unsafe_code)]
use std::{hint::black_box, time::Instant};
use tabdat_gretl_spike::{EstimationProblem, run_quantile};
fn main() {
  let problem = EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../../fixtures/quantile.csv")
      .lines()
      .skip(1)
      .map(|row| {
        row
          .split(',')
          .map(|v| v.parse().expect("fixture float"))
          .collect()
      })
      .collect(),
  };
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init = start.elapsed();
  let start = Instant::now();
  black_box(run_quantile(&problem, 0.5).expect("first fit"));
  let first = start.elapsed();
  let start = Instant::now();
  for _ in 0..1000 {
    black_box(run_quantile(&problem, 0.5).expect("repeated fit"));
  }
  let repeated = start.elapsed() / 1000;
  println!(
    "init_ns={} first_owned_quantile_ns={} repeated_owned_quantile_ns={}",
    init.as_nanos(),
    first.as_nanos(),
    repeated.as_nanos()
  );
}
