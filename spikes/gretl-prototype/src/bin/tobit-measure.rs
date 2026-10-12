#![forbid(unsafe_code)]
use std::{hint::black_box, time::Instant};
use tabdat_gretl_spike::{EstimationProblem, run_tobit};
fn main() {
  // Fixture parsing is setup, outside native init/fit timers, not a CSV benchmark.
  let data = include_str!("../../fixtures/tobit.csv")
    .lines()
    .skip(1)
    .map(|row| {
      let r: Vec<f64> = row
        .split(',')
        .map(|v| v.parse().expect("fixture float"))
        .collect();
      vec![r[0], r[2], r[3]]
    })
    .collect();
  let problem = EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data,
  };
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init = start.elapsed();
  let start = Instant::now();
  black_box(run_tobit(&problem, 0., None).expect("first fit"));
  let first = start.elapsed();
  let start = Instant::now();
  for _ in 0..1000 {
    black_box(run_tobit(&problem, 0., None).expect("repeated fit"));
  }
  let repeated = start.elapsed() / 1000;
  println!(
    "init_ns={} first_owned_tobit_ns={} repeated_owned_tobit_ns={}",
    init.as_nanos(),
    first.as_nanos(),
    repeated.as_nanos()
  );
}
