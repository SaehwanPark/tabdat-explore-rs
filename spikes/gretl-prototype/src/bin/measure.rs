#![forbid(unsafe_code)]

use std::time::Instant;
use tabdat_gretl_spike::{BinaryLink, EstimationProblem, run_binary, run_ols};

fn problem(csv: &str) -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: csv
      .lines()
      .skip(1)
      .map(|line| {
        line
          .split(',')
          .take(3)
          .map(|v| v.parse().unwrap())
          .collect()
      })
      .collect(),
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  if std::env::args().nth(1).as_deref() == Some("--metadata") {
    println!("tabdat-gretl-spike gretl=2026b measurement-only");
    return Ok(());
  }
  let ols = problem(include_str!("../../fixtures/covariance.csv"));
  let binary = problem(include_str!("../../fixtures/binary.csv"));
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init_us = start.elapsed().as_micros();
  let start = Instant::now();
  std::hint::black_box(run_ols(&ols)?);
  let first_ols_us = start.elapsed().as_micros();
  let start = Instant::now();
  for _ in 0..100 {
    std::hint::black_box(run_ols(&ols)?);
  }
  let repeated_100_ols_us = start.elapsed().as_micros();
  let start = Instant::now();
  std::hint::black_box(run_binary(&binary, BinaryLink::Logit)?);
  let first_logit_us = start.elapsed().as_micros();
  let start = Instant::now();
  for _ in 0..20 {
    std::hint::black_box(run_binary(&binary, BinaryLink::Logit)?);
  }
  let repeated_20_logit_us = start.elapsed().as_micros();
  println!(
    "init_us={init_us},first_ols_us={first_ols_us},repeated_100_ols_us={repeated_100_ols_us},first_logit_us={first_logit_us},repeated_20_logit_us={repeated_20_logit_us}"
  );
  Ok(())
}
