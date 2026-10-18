use serde_json::Value;
use tabdat_stats::{
  EstimationProblem, PoissonOptions, fit_poisson, fit_poisson_with_options,
  poisson_average_marginal_effects,
};

fn problem(case: &str) -> EstimationProblem {
  let x = [-2., -1., 0., 1., 2., 3., 0.5, 1.5, -0.5, 2.5, -1.5, 3.5];
  let z = [0., 1., 0., 1., 0., 1., 0., 1., 1., 0., 0.2, 0.7];
  match case {
    "multiple" | "noconstant" => EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      vec![0., 1., 0., 2., 1., 5., 2., 0., 3., 1., 4., 2.],
      x.iter().zip(z).map(|(&x, z)| vec![x, z]).collect(),
    )
    .unwrap()
    .with_intercept(case == "multiple"),
    "intercept" => {
      EstimationProblem::new("y", vec![], vec![0., 1., 2., 4., 0., 3.], vec![vec![]; 6]).unwrap()
    }
    "zero_noconstant" => EstimationProblem::new(
      "y",
      vec!["x".into()],
      vec![0.; 4],
      [-2., -1., 1., 2.].iter().map(|&x| vec![x]).collect(),
    )
    .unwrap()
    .with_intercept(false),
    "constant" => EstimationProblem::new("y", vec![], vec![1.; 4], vec![vec![]; 4]).unwrap(),
    _ => panic!("unknown fixture: {case}"),
  }
}
fn close(actual: &Value, expected: &str, tail: bool) {
  if expected == "NA" {
    assert!(actual.is_null(), "{actual}");
    return;
  }
  let expected: f64 = expected.parse().unwrap();
  let actual = actual.as_f64().unwrap();
  let tolerance = if tail {
    expected.abs() * 1e-10 + f64::MIN_POSITIVE
  } else {
    1e-10
  };
  assert!(
    actual.is_finite() && (actual - expected).abs() <= tolerance,
    "{actual} != {expected}, tolerance {tolerance}"
  );
}

#[test]
fn both_covariances_all_inference_diagnostics_means_and_delta_effects_match_independent_r() {
  let references: Vec<Vec<_>> = include_str!("fixtures/poisson_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect();
  assert_eq!(references.len(), 469);
  for case in [
    "multiple",
    "noconstant",
    "intercept",
    "zero_noconstant",
    "constant",
  ] {
    let problem = problem(case);
    for robust in [false, true] {
      let fit = fit_poisson(&problem, robust).unwrap();
      assert!(fit.max_abs_scaled_score <= 1e-10 && fit.max_abs_scaled_newton_step <= 1e-10);
      let effects = poisson_average_marginal_effects(&problem, &fit).unwrap();
      let json = serde_json::to_value(&fit).unwrap();
      let effects = serde_json::to_value(effects).unwrap();
      for row in references
        .iter()
        .filter(|r| r[0] == case && r[1] == if robust { "1" } else { "0" })
      {
        let quantity = row[2];
        let i: usize = row[3].parse().unwrap();
        let j: usize = row[4].parse().unwrap();
        let actual = if let Some(field) = quantity.strip_prefix("coefficient_") {
          json["coefficients"][i][field].clone()
        } else if let Some(field) = quantity.strip_prefix("effect_") {
          effects["effects"][i][field].clone()
        } else {
          match quantity {
            "covariance" => json["covariance"][i][j].clone(),
            "mean" => json["fitted_means"][i].clone(),
            "xb" => {
              let offset = usize::from(problem.include_intercept);
              let xb = problem.design_matrix[i]
                .iter()
                .zip(&fit.coefficients[offset..])
                .map(|(x, c)| x * c.value)
                .sum::<f64>()
                + if offset == 1 {
                  fit.coefficients[0].value
                } else {
                  0.
                };
              serde_json::json!(xb)
            }
            "wald_statistic" => json["omnibus_wald"]["statistic"].clone(),
            "wald_p_value" => json["omnibus_wald"]["p_value"].clone(),
            _ => json[quantity].clone(),
          }
        };
        close(&actual, row[5], quantity.ends_with("p_value"));
      }
      assert_eq!(fit.robust, robust);
      if robust {
        assert!(fit.likelihood_ratio_p_value.is_none());
      }
      if case == "constant" && robust {
        assert_eq!(fit.coefficients[0].standard_error, 0.);
        assert!(
          fit.coefficients[0].statistic.is_none() && fit.coefficients[0].confidence_lower.is_none()
        );
      }
    }
  }
}

#[test]
fn predictor_units_do_not_relax_convergence_or_change_fit_covariance_and_effects() {
  let base = problem("multiple");
  let mut scaled = base.clone();
  for row in &mut scaled.design_matrix {
    row[0] *= 1e6;
    row[1] *= 1e-6;
  }
  for robust in [false, true] {
    let a = fit_poisson(&base, robust).unwrap();
    let b = fit_poisson(&scaled, robust).unwrap();
    let factors = [1., 1e6, 1e-6];
    for i in 0..3 {
      close(
        &serde_json::json!(b.coefficients[i].value * factors[i]),
        &a.coefficients[i].value.to_string(),
        false,
      );
      for j in 0..3 {
        close(
          &serde_json::json!(b.covariance[i][j] * factors[i] * factors[j]),
          &a.covariance[i][j].to_string(),
          false,
        );
      }
    }
    for (x, y) in a.fitted_means.iter().zip(&b.fitted_means) {
      close(&serde_json::json!(x), &y.to_string(), false);
    }
    let ea = poisson_average_marginal_effects(&base, &a).unwrap();
    let eb = poisson_average_marginal_effects(&scaled, &b).unwrap();
    for i in 0..2 {
      close(
        &serde_json::json!(eb.effects[i].value * factors[i + 1]),
        &ea.effects[i].value.to_string(),
        false,
      );
    }
  }
}

#[test]
fn domain_rank_weights_iteration_and_boundary_failures_never_return_a_fit() {
  for y in [-1., 0.5, 9_007_199_254_740_992.] {
    let mut p = problem("multiple");
    p.response[0] = y;
    assert!(fit_poisson(&p, false).is_err());
  }
  let mut p = problem("multiple");
  p.response.fill(0.);
  assert!(fit_poisson(&p, false).is_err());
  let mut p = problem("multiple");
  for row in &mut p.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_poisson(&p, false).is_err());
  let mut p = problem("multiple");
  p.weights = Some(vec![1.; 12]);
  assert!(fit_poisson(&p, false).is_err());
  assert!(
    fit_poisson_with_options(
      &problem("multiple"),
      false,
      &PoissonOptions { max_iterations: 0 }
    )
    .is_err()
  );
  assert!(
    fit_poisson_with_options(
      &problem("multiple"),
      false,
      &PoissonOptions { max_iterations: 1 }
    )
    .is_err()
  );
  let boundary = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![2., 1., 0., 0.],
    vec![vec![0.], vec![0.], vec![1.], vec![1.]],
  )
  .unwrap();
  assert!(fit_poisson(&boundary, false).is_err());
  // Without an intercept, all-zero y can have an interior optimum: exp(bx)
  // cannot approach zero jointly when x has opposite signs.
  let interior = fit_poisson(&problem("zero_noconstant"), false).unwrap();
  assert_eq!(interior.coefficients[0].value, 0.);
  assert_eq!(interior.fitted_means, vec![1.; 4]);
}

#[test]
fn retained_alignment_is_checked_and_intercept_only_has_no_fake_omnibus_test() {
  let p = problem("multiple");
  let mut fit = fit_poisson(&p, false).unwrap();
  fit.predictor_names.swap(0, 1);
  assert!(poisson_average_marginal_effects(&p, &fit).is_err());
  let fit = fit_poisson(&problem("intercept"), true).unwrap();
  let wald = fit.omnibus_wald.unwrap();
  assert_eq!(wald.degrees_of_freedom, 0);
  assert_eq!(
    wald.unavailable_reason.as_deref(),
    Some("no_non_intercept_coefficients")
  );
  assert!(wald.statistic.is_none() && wald.p_value.is_none());
}
