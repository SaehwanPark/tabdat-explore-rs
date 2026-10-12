#![cfg(all(
  feature = "count-plugin",
  feature = "tobit-plugin",
  feature = "quantile-plugin"
))]
use tabdat_gretl_spike::{EstimationProblem, run_ols, run_poisson, run_quantile, run_tobit};
#[cfg(feature = "survival-plugin")]
use tabdat_gretl_spike::{SurvivalDistribution, SurvivalProblem, run_survival};
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
  #[cfg(feature = "survival-plugin")]
  let survival = SurvivalProblem {
    time: "time".into(),
    failure: "failure".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/survival.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l.split(',').map(|v| v.parse().unwrap()).collect();
        vec![r[2], r[3], r[4], r[5]]
      })
      .collect(),
  };
  std::thread::scope(|scope| {
    for i in 0..4 + usize::from(cfg!(feature = "survival-plugin")) {
      let continuous = &continuous;
      let count = &count;
      #[cfg(feature = "survival-plugin")]
      let survival = &survival;
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
            #[cfg(feature = "survival-plugin")]
            4 => {
              run_survival(survival, SurvivalDistribution::Weibull).unwrap();
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
