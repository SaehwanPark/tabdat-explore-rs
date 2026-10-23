use serde_json::Value;
use tabdat_stats::{
  CovarianceType, EstimationProblem, EstimationSample, fit_random_effects_ml,
  fit_random_effects_ml_clustered,
};

fn problem(case: &str) -> EstimationProblem {
  let rows: Vec<_> = include_str!("fixtures/random_effects_ml_cluster_inputs.csv")
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect::<Vec<_>>())
    .filter(|r| r[0] == case && !r[4].is_empty())
    .collect();
  let f = |r: &[&str], j: usize| r[j].parse::<f64>().unwrap();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "z".into()],
    rows.iter().map(|r| f(r, 4)).collect(),
    rows.iter().map(|r| vec![f(r, 5), f(r, 6)]).collect(),
  )
  .unwrap()
  .with_sample(
    EstimationSample::new(
      rows.iter().map(|r| f(r, 1) as usize - 1).collect(),
      rows.len() + 1,
      1,
      None,
      Some(rows.iter().map(|r| r[2].to_owned()).collect()),
    )
    .unwrap(),
  )
}

#[test]
fn full_joint_unit_sandwich_matches_independent_dense_r_likelihood_scores() {
  let reference: Vec<_> = include_str!("fixtures/random_effects_ml_cluster_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .collect();
  let mut checked = 0;
  for case in [
    "balanced",
    "unequal",
    "shifted_interleaved",
    "null_boundary",
  ] {
    let p = problem(case);
    let original = p.clone();
    let model = fit_random_effects_ml(&p, "id").unwrap();
    let fit = fit_random_effects_ml_clustered(&p, "id").unwrap();
    assert_eq!(p, original);
    assert_eq!(fit.gls.sample, p.sample);
    assert_eq!(fit.gls.fitted_values, model.gls.fitted_values);
    assert_eq!(fit.gls.residuals, model.gls.residuals);
    assert_eq!(fit.sigma_e_squared, model.sigma_e_squared);
    assert_eq!(fit.sigma_u_squared, model.sigma_u_squared);
    assert_eq!(
      fit.gls.covariance.covariance_type,
      CovarianceType::Cluster("id".into())
    );
    let ml = fit.ml.as_ref().unwrap();
    assert!(ml.unit_variance_test.is_none());
    assert!(
      ml.unit_variance_test_unavailable_reason
        .as_ref()
        .unwrap()
        .contains("unit-cluster")
    );
    assert!(ml.likelihood_ratio_p_value.is_none());
    assert!(
      ml.likelihood_ratio_unavailable_reason
        .as_ref()
        .unwrap()
        .contains("unit-cluster")
    );
    assert_eq!(ml.cluster.as_ref().unwrap().group_count, 8);
    assert_eq!(
      ml.joint_parameter_names,
      model.ml.as_ref().unwrap().joint_parameter_names
    );
    let component = fit.variance_component_inference().unwrap().unwrap();
    assert_eq!(component.method, "unit_cluster_joint_score_log_wald");
    assert_eq!(
      component.reference_distribution,
      "asymptotic_standard_normal"
    );
    let mut value = serde_json::to_value(&fit).unwrap();
    value["ml"]["variance_component_inference"] = serde_json::to_value(component).unwrap();
    for row in reference.iter().filter(|r| r[0] == case) {
      let actual = value
        .pointer(row[1])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{row:?}"));
      let expected = row[2].parse::<f64>().unwrap();
      // Independent dense scores differ from centered coordinates by roundoff;
      // use relative tails and an absolute floor only for near-zero covariances.
      let tolerance = if row[1].ends_with("p_value") {
        1e-7 * expected.abs().max(1e-300)
      } else {
        1e-8 * expected.abs().max(1.0)
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{case} {}: {actual:.17e} vs {expected:.17e}; tol={tolerance:.3e}",
        row[1]
      );
      checked += 1;
    }
    // The independent fixture includes a raw-key-valid, missing-y extra unit.
    assert_eq!(
      fit.gls.sample.as_ref().unwrap().retained_indices,
      (0..p.response.len()).collect::<Vec<_>>()
    );
  }
  assert_eq!(checked, 292);
}

#[test]
fn cluster_ml_rejects_boundary_and_unsupported_input_without_modifying_problem() {
  let mut p = problem("balanced");
  let original = p.clone();
  p.weights = Some(vec![1.; p.response.len()]);
  assert!(fit_random_effects_ml_clustered(&p, "id").is_err());
  p = original;
  for id in 1..=8 {
    let indices: Vec<_> = p
      .sample
      .as_ref()
      .unwrap()
      .cluster_groups
      .as_ref()
      .unwrap()
      .iter()
      .enumerate()
      .filter_map(|(i, key)| (key == &id.to_string()).then_some(i))
      .collect();
    let mean = p.design_matrix[indices[0]][0]
      + indices
        .iter()
        .map(|&i| p.design_matrix[i][0] - p.design_matrix[indices[0]][0])
        .sum::<f64>()
        / 4.;
    for (t, i) in indices.into_iter().enumerate() {
      p.response[i] = 0.7 * (p.design_matrix[i][0] - mean) + [-0.2, 0.2, 0.2, -0.2][t];
    }
  }
  // A fully constrained fit remains installed for nonrobust ML only.
  let classical = fit_random_effects_ml(&p, "id").unwrap();
  assert!(classical.ml.unwrap().unit_variance_boundary());
  let before = p.clone();
  let error = fit_random_effects_ml_clustered(&p, "id")
    .unwrap_err()
    .to_string();
  assert!(error.contains("interior positive unit variance"), "{error}");
  assert_eq!(p, before);
  p.sample.as_mut().unwrap().cluster_groups = None;
  assert!(fit_random_effects_ml_clustered(&p, "id").is_err());
}

#[test]
fn rank_limited_joint_slope_wald_keeps_positive_marginal_uncertainty() {
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut groups = Vec::new();
  for (id, scale) in [-3., -2., -1., -0.5, 0.5, 1., 2., 3.]
    .into_iter()
    .enumerate()
  {
    for time in 0..4 {
      let t = f64::from(time);
      x.push(vec![t, t * t]);
      y.push(1. + 0.7 * t - 0.3 * t * t + scale * (0.8 + 0.1 * t + 0.03 * t * t + 0.2 * t.sin()));
      groups.push(id.to_string());
    }
  }
  let p = EstimationProblem::new("y", vec!["x".into(), "z".into()], y, x)
    .unwrap()
    .with_sample(EstimationSample::new((0..32).collect(), 32, 0, None, Some(groups)).unwrap());
  // Identical within designs and a symmetric unit error multiplier make beta
  // scores proportional to that multiplier: the two-slope covariance has rank 1.
  let fit = fit_random_effects_ml_clustered(&p, "id").unwrap();
  assert!(
    fit
      .gls
      .coefficients
      .iter()
      .all(|c| c.standard_error.unwrap() > 0.)
  );
  assert!(fit.wald_statistic.is_none() && fit.wald_p_value.is_none());
  assert_eq!(
    fit.wald_unavailable_reason.as_deref(),
    Some("singular_slope_covariance")
  );
  assert!(
    fit
      .variance_component_inference()
      .unwrap()
      .unwrap()
      .sigma_u_squared
      .is_some()
  );
}
