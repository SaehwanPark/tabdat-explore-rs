#![cfg(all(
  feature = "count-plugin",
  feature = "tobit-plugin",
  feature = "quantile-plugin"
))]
use tabdat_gretl_spike::{EstimationProblem, run_ols, run_poisson, run_quantile, run_tobit};
fn problem(source: &str) -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: source
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}
#[test]
fn mixed_plugins_and_core_share_serialized_lifetime() {
  let continuous = problem(include_str!("../fixtures/quantile.csv"));
  let count = problem(include_str!("../fixtures/poisson.csv"));
  std::thread::scope(|scope| {
    for i in 0..4 {
      let continuous = &continuous;
      let count = &count;
      scope.spawn(move || {
        for _ in 0..5 {
          match i {
            0 => {
              run_quantile(continuous, 0.5).unwrap();
            }
            1 => {
              run_tobit(continuous, 0., None).unwrap();
            }
            2 => {
              run_poisson(count).unwrap();
            }
            _ => {
              run_ols(continuous).unwrap();
            }
          }
        }
      });
    }
  });
}
