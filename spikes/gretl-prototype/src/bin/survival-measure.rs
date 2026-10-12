#![forbid(unsafe_code)]
use std::{hint::black_box, time::Instant};
use tabdat_gretl_spike::{SurvivalDistribution, SurvivalProblem, run_survival};
fn main() {
  let problem = SurvivalProblem {
    time: "time".into(),
    failure: "failure".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../../fixtures/survival.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l
          .split(',')
          .map(|v| v.parse().expect("fixture float"))
          .collect();
        vec![r[2], r[3], r[4], r[5]]
      })
      .collect(),
  };
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init = start.elapsed();
  let start = Instant::now();
  black_box(run_survival(&problem, SurvivalDistribution::Weibull).expect("first fit"));
  let first = start.elapsed();
  let start = Instant::now();
  for _ in 0..1000 {
    black_box(run_survival(&problem, SurvivalDistribution::Weibull).expect("repeated fit"));
  }
  let repeated = start.elapsed() / 1000;
  println!(
    "init_ns={} first_owned_survival_ns={} repeated_owned_survival_ns={}",
    init.as_nanos(),
    first.as_nanos(),
    repeated.as_nanos()
  );
}
