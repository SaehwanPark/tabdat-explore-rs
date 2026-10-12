#![cfg(all(
  feature = "count-plugin",
  feature = "tobit-plugin",
  feature = "quantile-plugin"
))]
use tabdat_gretl_spike::{EstimationProblem, run_ols, run_poisson, run_quantile, run_tobit};
#[cfg(feature = "heckman-plugin")]
use tabdat_gretl_spike::{HeckmanProblem, run_heckman};
#[cfg(feature = "dpanel-plugin")]
use tabdat_gretl_spike::{PanelProblem, run_dynamic_panel};
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
  #[cfg(feature = "heckman-plugin")]
  let selection = HeckmanProblem {
    outcome: "y".into(),
    outcome_predictors: vec!["x1".into(), "x2".into()],
    selection: "selected".into(),
    selection_predictors: vec!["x1".into(), "z".into()],
    include_intercept: true,
    data: include_str!("../fixtures/heckman.csv")
      .lines()
      .skip(1)
      .map(|l| {
        let r: Vec<f64> = l.split(',').map(|v| v.parse().unwrap()).collect();
        vec![r[0], r[1], r[2], r[3], r[1], r[4]]
      })
      .collect(),
  };
  #[cfg(feature = "dpanel-plugin")]
  let dynamic = {
    let rows: Vec<Vec<f64>> = include_str!("../fixtures/dynamic-panel.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
      .collect();
    PanelProblem {
      outcome: "y".into(),
      predictors: vec!["x1".into(), "x2".into()],
      keys: rows
        .iter()
        .map(|r| (r[0] as usize, r[1] as usize))
        .collect(),
      periods: 8,
      data: rows.iter().map(|r| r[2..].to_vec()).collect(),
    }
  };
  std::thread::scope(|scope| {
    for i in 0..4
      + usize::from(cfg!(feature = "survival-plugin"))
      + usize::from(cfg!(feature = "heckman-plugin"))
      + usize::from(cfg!(feature = "dpanel-plugin"))
    {
      let continuous = &continuous;
      let count = &count;
      #[cfg(feature = "survival-plugin")]
      let survival = &survival;
      #[cfg(feature = "heckman-plugin")]
      let selection = &selection;
      #[cfg(feature = "dpanel-plugin")]
      let dynamic = &dynamic;
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
            #[cfg(feature = "heckman-plugin")]
            index if index == 4 + usize::from(cfg!(feature = "survival-plugin")) => {
              run_heckman(selection).unwrap();
            }
            #[cfg(feature = "dpanel-plugin")]
            index
              if index
                == 4
                  + usize::from(cfg!(feature = "survival-plugin"))
                  + usize::from(cfg!(feature = "heckman-plugin")) =>
            {
              run_dynamic_panel(dynamic).unwrap();
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
