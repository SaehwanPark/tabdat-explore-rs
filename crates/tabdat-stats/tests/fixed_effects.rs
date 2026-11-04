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
    match (&original.unit_effects_f_test, &changed.unit_effects_f_test) {
      (Some(original), Some(changed)) => {
        assert_eq!(original.df_num, changed.df_num);
        assert_eq!(original.df_denom, changed.df_denom);
        assert!((original.f_statistic - changed.f_statistic).abs() < 1e-9);
        assert!((original.p_value - changed.p_value).abs() < 1e-9);
      }
      (None, None) => {}
      _ => panic!("a common shift changed unit-effects test availability"),
    }
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

#[test]
fn exact_within_fit_keeps_the_model_when_the_unit_effect_test_is_undefined() {
  let rows: Vec<_> = (1..=3)
    .flat_map(|id| (0..4).map(move |time| (id, time, 5.0 * f64::from(id) + 2.0 * f64::from(time))))
    .collect();
  let problem = EstimationProblem::new(
    "y",
    vec!["x".into()],
    rows.iter().map(|row| row.2).collect(),
    rows.iter().map(|row| vec![f64::from(row.1)]).collect(),
  )
  .unwrap()
  .with_sample(
    EstimationSample::new(
      (0..rows.len()).collect(),
      rows.len(),
      0,
      None,
      Some(rows.iter().map(|row| format!("g{}", row.0)).collect()),
    )
    .unwrap(),
  );

  let result = fit_fixed_effects(&problem, "person", false).unwrap();
  assert_eq!(result.within.coefficients[0].value, 2.0);
  assert_eq!(
    result.within.fit_statistics.residual_sum_of_squares,
    Some(0.0)
  );
  assert!(result.unit_effects_f_test.is_none());
  assert!(
    result
      .unit_effects_f_test_unavailable_reason
      .as_deref()
      .unwrap()
      .contains("residual variance is degenerate")
  );
  let normalized = result.normalized_level.unwrap();
  let common = normalized
    .linear_combination_with_inference(&[1.0, 0.0], 0.0, 95.0)
    .unwrap();
  assert_eq!(common.standard_error, 0.0);
  assert_eq!(common.statistic, None);
  assert!(common.p_value.is_nan());
  assert_eq!(common.ci_lower, common.estimate);
  assert_eq!(common.ci_upper, common.estimate);
}

#[test]
fn normalized_common_inference_matches_independent_weighted_dummy_qr() {
  let reference: Vec<_> = include_str!("fixtures/fixed_effects_common_reference.csv")
    .lines()
    .skip(1)
    .map(|row| row.split(',').collect::<Vec<_>>())
    .collect();
  assert_eq!(reference.len(), 96);
  let mut checked = 0;
  for case in ["balanced", "unbalanced", "interleaved"] {
    let fit = fit_fixed_effects(&problem(case), "person", false).unwrap();
    let normalized = fit.normalized_level.as_ref().unwrap();
    assert_eq!(normalized.parameter_names, ["_cons", "x", "z"]);
    assert_eq!(fit.within.parameter_names, ["x", "z"]);
    assert_eq!(
      normalized.degrees_of_freedom,
      fit.within.fit_statistics.degrees_of_freedom
    );
    for i in 0..2 {
      for j in 0..2 {
        assert_eq!(
          normalized.covariance.matrix[i + 1][j + 1],
          fit.within.covariance.matrix[i][j]
        );
      }
    }
    let common = normalized
      .linear_combination_with_inference(&[1.0, 0.0, 0.0], 0.0, 95.0)
      .unwrap();
    let contrast = normalized
      .linear_combination_with_inference(&[1.0, 2.0, -0.5], 1.3, 95.0)
      .unwrap();
    let joint = normalized
      .test_linear_hypothesis(&[vec![1.0, 0.0, 0.0], vec![0.0, 1.0, -0.5]], &[0.0, 0.2])
      .unwrap();
    let mut value = serde_json::to_value(&fit).unwrap();
    value["common"] = serde_json::to_value(common).unwrap();
    value["contrast"] = serde_json::to_value(contrast).unwrap();
    value["joint"] = json!({"f_statistic":joint.f_statistic,"df_num":joint.df_num,"df_denom":joint.df_denom,
      "p_value":f_distribution_survival_probability(joint.f_statistic,joint.df_num as f64,joint.df_denom as f64)});
    for row in reference.iter().filter(|r| r[0] == case) {
      let actual = value.pointer(row[1]).and_then(|v| v.as_f64()).unwrap();
      let expected: f64 = row[2].parse().unwrap();
      let tolerance = if row[1].ends_with("p_value") {
        expected.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{row:?}: native={actual:e}, R={expected:e}"
      );
      checked += 1;
    }
  }
  assert_eq!(checked, reference.len());
}

#[test]
fn normalized_common_covariance_transforms_with_predictor_and_response_shifts() {
  let mut p = problem("balanced");
  for y in &mut p.response {
    *y = (*y * 16.0).round() / 16.0;
  }
  for row in &mut p.design_matrix {
    for x in row {
      *x = (*x * 16.0).round() / 16.0;
    }
  }
  let before = fit_fixed_effects(&p, "person", false)
    .unwrap()
    .normalized_level
    .unwrap();
  for y in &mut p.response {
    *y += 32.0;
  }
  for row in &mut p.design_matrix {
    row[0] += 16.0;
  }
  let after = fit_fixed_effects(&p, "person", false)
    .unwrap()
    .normalized_level
    .unwrap();
  let a = &before.covariance.matrix;
  let b = &after.covariance.matrix;
  assert!(
    (after.parameters[0] - (before.parameters[0] + 32.0 - 16.0 * before.parameters[1])).abs()
      < 1e-9
  );
  assert!((b[0][0] - (a[0][0] - 32.0 * a[0][1] + 256.0 * a[1][1])).abs() < 1e-9);
  for j in 1..3 {
    assert!((b[0][j] - (a[0][j] - 16.0 * a[1][j])).abs() < 1e-9);
  }
}

#[test]
fn huge_predictor_offsets_withhold_unrecoverable_common_covariance_not_slopes() {
  let mut p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    (1..=3)
      .flat_map(|g| (0..4).map(move |t| 3.0 + g as f64 + 0.5 * t as f64 + 0.0625 * (t * t) as f64))
      .collect(),
    (1..=3)
      .flat_map(|g| (0..4).map(move |t| vec![g as f64 * 0.5 + t as f64 * 0.25]))
      .collect(),
  )
  .unwrap()
  .with_sample(
    EstimationSample::new(
      (0..12).collect(),
      12,
      0,
      None,
      Some(
        (1..=3)
          .flat_map(|g| (0..4).map(move |_| g.to_string()))
          .collect(),
      ),
    )
    .unwrap(),
  );
  let before = fit_fixed_effects(&p, "person", false).unwrap();
  let expected = before
    .normalized_level
    .as_ref()
    .unwrap()
    .linear_combination_with_inference(&[1.0, 0.0], 0.0, 95.0)
    .unwrap();
  assert!(expected.standard_error > 0.0);
  for row in &mut p.design_matrix {
    row[0] += 1e12;
  }
  let after = fit_fixed_effects(&p, "person", false).unwrap();
  assert!(after.normalized_level.is_none());
  assert!(
    after
      .normalized_level_unavailable_reason
      .as_deref()
      .unwrap()
      .contains("ill-conditioned")
  );
  assert_eq!(before.within.coefficients, after.within.coefficients);
  assert_eq!(before.within.covariance, after.within.covariance);
  assert_eq!(before.fitted_values, after.fitted_values);
}

#[test]
fn normalized_common_withholding_and_legacy_state_are_explicit() {
  let fit = fit_fixed_effects(&problem("balanced"), "person", true).unwrap();
  assert!(fit.normalized_level.is_none());
  assert!(
    fit
      .normalized_level_unavailable_reason
      .as_deref()
      .unwrap()
      .contains("unit-cluster")
  );
  let mut p = problem("balanced");
  p.predictor_names[0] = "_cons".into();
  let fit = fit_fixed_effects(&p, "person", false).unwrap();
  assert!(fit.normalized_level.is_none());
  assert!(
    fit
      .normalized_level_unavailable_reason
      .as_deref()
      .unwrap()
      .contains("conflicts")
  );
  let fit = fit_fixed_effects(&problem("balanced"), "person", false).unwrap();
  let mut legacy = serde_json::to_value(&fit).unwrap();
  legacy.as_object_mut().unwrap().remove("normalized_level");
  legacy
    .as_object_mut()
    .unwrap()
    .remove("normalized_level_unavailable_reason");
  let restored: tabdat_stats::FixedEffectsResult = serde_json::from_value(legacy).unwrap();
  assert!(restored.normalized_level.is_none());
  assert_eq!(restored.common_intercept, fit.common_intercept);
  assert_eq!(restored.within, fit.within);
}
