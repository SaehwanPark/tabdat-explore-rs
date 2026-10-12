#![forbid(unsafe_code)]
use std::{hint::black_box, time::Instant};
use tabdat_gretl_spike::{PanelProblem, run_dynamic_panel};
fn main() {
  let rows: Vec<Vec<f64>> = include_str!("../../fixtures/dynamic-panel.csv")
    .lines()
    .skip(1)
    .map(|l| {
      l.split(',')
        .map(|v| v.parse().expect("fixture float"))
        .collect()
    })
    .collect();
  let problem = PanelProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    keys: rows
      .iter()
      .map(|r| (r[0] as usize, r[1] as usize))
      .collect(),
    periods: 8,
    data: rows.iter().map(|r| r[2..].to_vec()).collect(),
  };
  let start = Instant::now();
  gretl_sys::initialize_for_measurement();
  let init = start.elapsed();
  let start = Instant::now();
  black_box(run_dynamic_panel(&problem).expect("first fit"));
  let first = start.elapsed();
  let start = Instant::now();
  for _ in 0..1000 {
    black_box(run_dynamic_panel(&problem).expect("repeat fit"));
  }
  let repeated = start.elapsed() / 1000;
  println!(
    "init_ns={} first_owned_dynamic_ns={} repeated_owned_dynamic_ns={}",
    init.as_nanos(),
    first.as_nanos(),
    repeated.as_nanos()
  );
}
