use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_random_effects_ml, fit_random_effects_ml_interior,
};

#[test]
fn unequal_counts_match_nlme_ml_and_independent_joint_information() {
  let mut comparisons = 0;
  for case in ["unequal", "shifted_interleaved", "boundary"] {
    let rows: Vec<_> = include_str!("fixtures/random_effects_ml_unbalanced_inputs.csv")
      .lines()
      .skip(1)
      .map(|s| s.split(',').collect::<Vec<_>>())
      .filter(|r| r[0] == case)
      .collect();
    let p = EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      rows.iter().map(|r| r[3].parse().unwrap()).collect(),
      rows
        .iter()
        .map(|r| vec![r[4].parse().unwrap(), r[5].parse().unwrap()])
        .collect(),
    )
    .unwrap()
    .with_sample(
      EstimationSample::new(
        (0..rows.len()).collect(),
        rows.len(),
        0,
        None,
        Some(rows.iter().map(|r| r[1].to_owned()).collect()),
      )
      .unwrap(),
    );
    assert!(
      fit_random_effects_ml_interior(&p, "id")
        .unwrap_err()
        .to_string()
        .contains("balanced")
    );
    let fit = fit_random_effects_ml(&p, "id").unwrap();
    assert_eq!(fit.gls.sample, p.sample);
    assert_eq!(
      fit.gls.diagnostics.method,
      "unbalanced_gaussian_random_effects_ml"
    );
    assert_eq!(fit.group_size, 0);
    assert_eq!(fit.group_size_range(), Some((1, 5)));
    let (theta_min, theta_max) = fit.theta_range().unwrap();
    let ml = fit.ml.as_ref().unwrap();
    let unit_variance_test = ml.unit_variance_test.as_ref().unwrap();
    assert_eq!(ml.unit_variance_boundary(), case == "boundary");
    assert_eq!(ml.likelihood_ratio_p_value.is_none(), case == "boundary");
    assert_eq!(
      unit_variance_test.reference_distribution,
      "asymptotic_50_50_chi_square_0_1_mixture"
    );
    if case == "boundary" {
      assert_eq!(unit_variance_test.statistic, 0.0);
      assert_eq!(unit_variance_test.p_value, 1.0);
    } else {
      assert!(unit_variance_test.statistic > 0.0);
      assert!((0.0..0.5).contains(&unit_variance_test.p_value));
    }
    assert_eq!(
      ml.joint_parameter_names.last().unwrap(),
      if case == "boundary" {
        "log_sigma_e_squared"
      } else {
        "log_sigma_u_squared"
      }
    );
    let population: Vec<_> = p
      .design_matrix
      .iter()
      .map(|row| {
        fit.gls.coefficients[0].value
          + row
            .iter()
            .zip(&fit.gls.coefficients[1..])
            .map(|(x, b)| x * b.value)
            .sum::<f64>()
      })
      .collect();
    let residuals: Vec<_> = p
      .response
      .iter()
      .zip(&population)
      .map(|(y, xb)| y - xb)
      .collect();
    let variance_inference = fit.variance_component_inference().unwrap().unwrap();
    if case == "boundary" {
      assert!(variance_inference.sigma_u_squared.is_none());
      assert!(
        variance_inference
          .sigma_u_squared_unavailable_reason
          .as_deref()
          .is_some_and(|reason| reason.contains("boundary"))
      );
    } else {
      assert!(variance_inference.sigma_u_squared.is_some());
      assert!(
        variance_inference
          .sigma_u_squared_unavailable_reason
          .is_none()
      );
    }
    let mut value = serde_json::to_value(fit).unwrap();
    value["ml"]["variance_component_inference"] = serde_json::to_value(variance_inference).unwrap();
    value["population_fitted_values"] = serde_json::json!(population);
    value["population_residuals"] = serde_json::json!(residuals);
    value["minimum_group_size"] = 1.into();
    value["maximum_group_size"] = 5.into();
    value["theta_min"] = theta_min.into();
    value["theta_max"] = theta_max.into();
    for line in include_str!("fixtures/random_effects_ml_unbalanced_reference.csv")
      .lines()
      .skip(1)
    {
      let row: Vec<_> = line.split(',').collect();
      if row[0] != case {
        continue;
      }
      let actual = value
        .pointer(row[1])
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| panic!("{row:?}"));
      let expected: f64 = row[2].parse().unwrap();
      let tolerance = if row[1].ends_with("p_value") {
        expected.abs() * 1e-8
      } else {
        1e-9
      };
      assert!(
        (actual - expected).abs() <= tolerance,
        "{row:?}: actual {actual:.17e}, expected {expected:.17e}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 531);
}
