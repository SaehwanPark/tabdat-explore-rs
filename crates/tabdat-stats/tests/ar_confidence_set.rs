use tabdat_stats::{
  ArConfidenceSetShape, InstrumentalVariablesProblem, StatsError, anderson_rubin_confidence_set,
  anderson_rubin_test,
};

fn close(actual: f64, expected: f64) {
  assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn problem(strength: f64, direct_z1: f64, direct_z2: f64) -> InstrumentalVariablesProblem {
  let mut problem = InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: vec!["w".into()],
    endogenous_names: vec!["x".into()],
    instrument_names: vec!["z1".into(), "z2".into()],
    response: vec![],
    exogenous: vec![],
    endogenous: vec![],
    instruments: vec![],
    include_intercept: true,
    sample: None,
  };
  for i in 1..=40 {
    let basis = |k: f64| (2.0 * std::f64::consts::PI * k * i as f64 / 40.0).cos();
    let x = strength * basis(1.0) + basis(3.0);
    let w = basis(5.0);
    problem
      .response
      .push(1.5 * x + 0.5 * w + direct_z1 * basis(1.0) + direct_z2 * basis(2.0) + basis(4.0));
    problem.exogenous.push(vec![w]);
    problem.endogenous.push(vec![x]);
    problem.instruments.push(vec![basis(1.0), basis(2.0)]);
  }
  problem
}

#[test]
fn confidence_set_shapes_and_endpoints_match_independent_r() {
  // R 4.6.1 lm.fit/qf/polyroot: fixtures/ar_confidence_set_reference.R.
  use ArConfidenceSetShape::*;
  for (strength, z1, z2, expected, roots) in [
    (
      2.0,
      0.0,
      0.0,
      Bounded,
      Some((1.282246370466847, 1.717753629533153)),
    ),
    (0.01, 0.0, 0.0, AllReal, None),
    (
      0.1,
      2.0,
      0.0,
      Disjoint,
      Some((-4.536179309204275, 5.198096652710294)),
    ),
    (2.0, 0.0, 10.0, Empty, None),
  ] {
    let problem = problem(strength, z1, z2);
    let result = anderson_rubin_confidence_set(&problem, 95.0).unwrap();
    assert_eq!(result.shape, expected);
    assert_eq!(result.endogenous, "x");
    assert_eq!(
      (
        result.observation_count,
        result.numerator_df,
        result.denominator_df
      ),
      (40, 2, 36)
    );
    close(result.critical_f, 3.259446306144109);
    match expected {
      Bounded => {
        let (lower, upper) = roots.unwrap();
        close(result.intervals[0].lower.unwrap(), lower);
        close(result.intervals[0].upper.unwrap(), upper);
      }
      Disjoint => {
        let (lower, upper) = roots.unwrap();
        assert_eq!(result.intervals.len(), 2);
        assert_eq!(result.intervals[0].lower, None);
        assert_eq!(result.intervals[1].upper, None);
        close(result.intervals[0].upper.unwrap(), lower);
        close(result.intervals[1].lower.unwrap(), upper);
      }
      AllReal => {
        assert_eq!(result.intervals.len(), 1);
        assert_eq!(
          (result.intervals[0].lower, result.intervals[0].upper),
          (None, None)
        );
      }
      Empty => assert!(result.intervals.is_empty()),
      _ => unreachable!(),
    }
    for beta in [-100.0, -5.0, 0.0, 1.3, 1.5, 1.8, 6.0, 100.0] {
      let contains = result.intervals.iter().any(|interval| {
        interval.lower.is_none_or(|bound| beta >= bound)
          && interval.upper.is_none_or(|bound| beta <= bound)
      });
      let test = anderson_rubin_test(&problem, &[beta]).unwrap();
      assert_eq!(contains, test.p_value >= 0.05, "{expected:?}, beta={beta}");
    }
    if let Some((lower, upper)) = roots {
      for beta in [lower, upper] {
        close(
          anderson_rubin_test(&problem, &[beta]).unwrap().statistic,
          result.critical_f,
        );
      }
    }
  }
}

#[test]
fn noconstant_and_nondefault_level_match_r_and_zero_column_restriction_executes() {
  let mut problem = problem(2.0, 0.0, 0.0);
  problem.include_intercept = false;
  let result = anderson_rubin_confidence_set(&problem, 90.0).unwrap();
  assert_eq!(result.denominator_df, 37);
  close(result.critical_f, 2.45201433032702);
  close(result.intervals[0].lower.unwrap(), 1.314875969835299);
  close(result.intervals[0].upper.unwrap(), 1.685124030164702);
  problem.exogenous_names.clear();
  for row in &mut problem.exogenous {
    row.clear();
  }
  let result = anderson_rubin_confidence_set(&problem, 95.0).unwrap();
  assert_eq!(result.denominator_df, 38);
  for beta in [0.0, 1.5, 10.0] {
    let contains = result.intervals.iter().any(|interval| {
      interval.lower.is_none_or(|bound| beta >= bound)
        && interval.upper.is_none_or(|bound| beta <= bound)
    });
    assert_eq!(
      contains,
      anderson_rubin_test(&problem, &[beta]).unwrap().p_value >= 0.05
    );
  }
}

#[test]
fn invalid_scope_levels_rank_df_and_variance_are_errors() {
  let mut problem = problem(2.0, 0.0, 0.0);
  for level in [0.0, 100.0, -1.0, f64::NAN, f64::INFINITY] {
    assert!(anderson_rubin_confidence_set(&problem, level).is_err());
  }
  let mut multiple = problem.clone();
  multiple.endogenous_names.push("x2".into());
  for row in &mut multiple.endogenous {
    row.push(row[0] * 2.0);
  }
  assert!(anderson_rubin_confidence_set(&multiple, 95.0).is_err());
  let mut singular = problem.clone();
  for row in &mut singular.instruments {
    row[1] = row[0];
  }
  assert!(matches!(
    anderson_rubin_confidence_set(&singular, 95.0),
    Err(StatsError::SingularMatrix(_))
  ));
  let mut short = problem.clone();
  short.response.truncate(4);
  short.exogenous.truncate(4);
  short.endogenous.truncate(4);
  short.instruments.truncate(4);
  assert!(matches!(
    anderson_rubin_confidence_set(&short, 95.0),
    Err(StatsError::ZeroDegreesOfFreedom { .. })
  ));
  for (y, x) in problem.response.iter_mut().zip(&problem.endogenous) {
    *y = x[0];
  }
  assert!(anderson_rubin_confidence_set(&problem, 95.0).is_err());
  problem.response.fill(0.0);
  assert!(anderson_rubin_confidence_set(&problem, 95.0).is_err());
}
