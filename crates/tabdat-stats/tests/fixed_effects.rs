use serde_json::json;
use tabdat_stats::{
  EstimationProblem, EstimationSample, PostEstimationModel, StatsError,
  f_distribution_survival_probability, fit_fixed_effects,
};

fn problem(case: &str) -> EstimationProblem {
  let mut rows = Vec::new();
  for id in 1..=6 {
    let size = if case == "unbalanced" { id } else { 4 };
    for t in 0..size {
      let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
      let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
      let y = 1.0 + 0.4 * f64::from(id) + 0.7 * x - 0.3 * z + 0.11 * f64::from((id * t + 3) % 7);
      rows.push((id, t, y, x, z));
    }
  }
  if case == "interleaved" {
    rows.sort_by_key(|r| (-r.1, -r.0));
  }
  let n = rows.len();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "z".into()],
    rows.iter().map(|r| r.2).collect(),
    rows.iter().map(|r| vec![r.3, r.4]).collect(),
  )
  .unwrap()
  .with_sample(
    EstimationSample::new(
      (0..n).map(|i| i * 2).collect(),
      n * 2,
      n,
      None,
      Some(rows.iter().map(|r| format!("g{}", r.0)).collect()),
    )
    .unwrap(),
  )
}

#[test]
fn within_fits_match_independent_plm_lsdv_and_group_score_references() {
  let reference: Vec<_> = include_str!("fixtures/fixed_effects_reference.csv")
    .lines()
    .skip(1)
    .map(|r| r.split(',').collect::<Vec<_>>())
    .collect();
  assert!(!reference.is_empty());
  let mut checked = 0;
  for case in ["balanced", "unbalanced", "interleaved"] {
    for clustered in [false, true] {
      let p = problem(case);
      let fit = fit_fixed_effects(&p, "person", clustered).unwrap();
      assert_eq!(fit.within.sample, p.sample);
      assert_eq!(fit.entity_variable, "person");
      assert_eq!(fit.within.parameter_names, ["x", "z"]);
      assert_eq!(fit.within.fit_statistics.log_likelihood, None);
      assert_eq!(fit.within.fit_statistics.adjusted_r_squared, None);
      let post = PostEstimationModel::from_least_squares(&fit.within);
      let inference = post
        .linear_combination_with_inference(&[1.0, -0.5], 2.0, 95.0)
        .unwrap();
      let joint = post
        .test_linear_hypothesis(&[vec![1.0, 0.0], vec![0.0, 1.0]], &[0.0, 0.0])
        .unwrap();
      let mut value = serde_json::to_value(&fit).unwrap();
      if clustered {
        assert!(value["unit_effects_f_test"].is_null());
      } else {
        assert!(value["unit_effects_f_test"].is_object());
      }
      value["lincom"] = serde_json::to_value(inference).unwrap();
      value["joint"] = json!({"f_statistic":joint.f_statistic,"df_num":joint.df_num,"df_denom":joint.df_denom,
        "p_value":f_distribution_survival_probability(joint.f_statistic,joint.df_num as f64,joint.df_denom as f64)});
      let covariance = if clustered { "cluster" } else { "classical" };
      for row in reference
        .iter()
        .filter(|r| r[0] == case && r[1] == covariance)
      {
        let actual = value
          .pointer(row[2])
          .and_then(|v| v.as_f64())
          .unwrap_or_else(|| panic!("missing numerical pointer {row:?}"));
        let expected: f64 = row[3].parse().unwrap();
        let tolerance = if row[2].ends_with("p_value") {
          expected.abs() * 1e-8
        } else {
          1e-9
        };
        assert!(
          actual.is_finite() && (actual - expected).abs() <= tolerance,
          "{row:?}: native={actual:.17e}, R={expected:.17e}, tol={tolerance:.3e}"
        );
        checked += 1;
      }
      assert!(
        (fit
          .effects
          .iter()
          .map(|g| g.effect * g.observation_count as f64)
          .sum::<f64>())
        .abs()
          < 1e-9
      );
      assert_eq!(fit.singleton_groups, usize::from(case == "unbalanced"));
    }
  }
  assert_eq!(checked, reference.len());
}

#[test]
fn centering_shifts_rank_and_df_failure_contracts() {
  for clustered in [false, true] {
    let p = problem("unbalanced");
    let original = fit_fixed_effects(&p, "person", clustered).unwrap();
    let mut shifted = p.clone();
    for y in &mut shifted.response {
      *y += 2000.0;
    }
    for x in &mut shifted.design_matrix {
      x[0] += 1000.0;
    }
    let changed = fit_fixed_effects(&shifted, "person", clustered).unwrap();
    for (a, b) in original
      .within
      .coefficients
      .iter()
      .zip(&changed.within.coefficients)
    {
      assert!((a.value - b.value).abs() < 1e-9);
      assert!((a.standard_error.unwrap() - b.standard_error.unwrap()).abs() < 1e-9);
    }
    for (a, b) in original.effects.iter().zip(&changed.effects) {
      assert!((a.effect - b.effect).abs() < 1e-9);
    }
    for (a, b) in original.fitted_values.iter().zip(&changed.fitted_values) {
      assert!((b - a - 2000.0).abs() < 1e-9);
    }
    assert!(
      (changed.common_intercept - original.common_intercept - 2000.0
        + 1000.0 * original.within.coefficients[0].value)
        .abs()
        < 1e-9
    );
  }
  let mut p = problem("balanced");
  p.sample.as_mut().unwrap().total_observations += 1;
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  p.sample.as_mut().unwrap().retained_indices[0] = usize::MAX;
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  for (i, y) in p.response.iter_mut().enumerate() {
    *y = if i % 2 == 0 { 1e308 } else { -1e308 };
  }
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  p.weights = Some(vec![1.0; p.response.len()]);
  assert!(matches!(
    fit_fixed_effects(&p, "person", false),
    Err(StatsError::InvalidEstimation(_))
  ));
  let mut p = problem("balanced");
  p.include_intercept = false;
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  p.sample = None;
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  for (i, row) in p.design_matrix.iter_mut().enumerate() {
    row[0] = (i / 4) as f64;
  }
  assert!(fit_fixed_effects(&p, "person", false).is_err());
  let mut p = problem("balanced");
  p.sample.as_mut().unwrap().cluster_groups =
    Some((0..p.response.len()).map(|i| i.to_string()).collect());
  assert!(matches!(
    fit_fixed_effects(&p, "person", false),
    Err(StatsError::ZeroDegreesOfFreedom { .. })
  ));
  let mut p = problem("balanced");
  p.sample.as_mut().unwrap().cluster_groups = Some(vec!["single".into(); p.response.len()]);
  assert!(fit_fixed_effects(&p, "person", false).is_ok());
  assert!(fit_fixed_effects(&p, "person", true).is_err());
}
