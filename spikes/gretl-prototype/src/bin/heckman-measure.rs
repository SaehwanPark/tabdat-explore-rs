#![forbid(unsafe_code)]
use std::{hint::black_box, time::Instant};
use tabdat_gretl_spike::{HeckmanProblem, run_heckman};
fn main() {
  let problem = HeckmanProblem {
    outcome: "y".into(),
    outcome_predictors: vec!["x1".into(), "x2".into()],
    selection: "selected".into(),
    selection_predictors: vec!["x1".into(), "z".into()],
    include_intercept: true,
    data: include_str!("../../fixtures/heckman.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l
          .split(',')
          .map(|v| v.parse().expect("fixture float"))
          .collect();
        vec![r[0], r[1], r[2], r[3], r[1], r[4]]
      })
      .collect(),
  };
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init = start.elapsed();
  let start = Instant::now();
  black_box(run_heckman(&problem).expect("first fit"));
  let first = start.elapsed();
  let start = Instant::now();
  for _ in 0..1000 {
    black_box(run_heckman(&problem).expect("repeated fit"));
  }
  let repeated = start.elapsed() / 1000;
  println!(
    "init_ns={} first_owned_heckman_ns={} repeated_owned_heckman_ns={}",
    init.as_nanos(),
    first.as_nanos(),
    repeated.as_nanos()
  );
}
