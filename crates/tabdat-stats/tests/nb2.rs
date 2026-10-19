use serde_json::Value;
use tabdat_stats::{
  EstimationProblem, Nb2Options, fit_nb2, fit_nb2_with_options, nb2_average_marginal_effects,
};

fn problem(case: &str) -> EstimationProblem {
  let y = vec![
    0., 0., 2., 0., 0., 3., 0., 3., 1., 3., 8., 3., 0., 0., 0., 2., 1., 2., 0., 0., 2., 1., 1., 2.,
    0., 5., 4., 1., 1., 0., 0., 2., 2., 0., 1., 0.,
  ];
  let intercept = case != "noconstant";
  let (names, design) = if case == "intercept" {
    (vec![], vec![vec![]; 36])
  } else {
    (
      vec!["x".into(), "z".into()],
      (0..36)
        .map(|i| vec![-2.0 + 4.0 * i as f64 / 35.0, [0., 1., 0.4][i % 3]])
        .collect(),
    )
  };
  EstimationProblem::new("y", names, y, design)
    .unwrap()
    .with_intercept(intercept)
}
fn close(actual: &Value, expected: &str, tail: bool) {
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
fn both_joint_covariances_dispersion_null_diagnostics_effects_and_predictions_match_r() {
  let references: Vec<Vec<_>> = include_str!("fixtures/nb2_reference.csv")
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect())
    .collect();
  assert_eq!(references.len(), 742);
  for case in ["multiple", "noconstant", "intercept"] {
    let problem = problem(case);
    for robust in [false, true] {
      let fit = fit_nb2(&problem, robust).unwrap();
      let effects = nb2_average_marginal_effects(&problem, &fit).unwrap();
      assert!(fit.max_abs_scaled_score <= 1e-10 && fit.max_abs_scaled_newton_step <= 1e-10);
      assert_eq!(fit.robust, robust);
      let json = serde_json::to_value(&fit).unwrap();
      let effects = serde_json::to_value(effects).unwrap();
      for row in references
        .iter()
        .filter(|r| r[0] == case && r[1] == if robust { "1" } else { "0" })
      {
        let q = row[2];
        let i: usize = row[3].parse().unwrap();
        let j: usize = row[4].parse().unwrap();
        let actual = if let Some(field) = q.strip_prefix("coefficient_") {
          json["coefficients"][i][field].clone()
        } else if let Some(field) = q.strip_prefix("effect_") {
          effects["effects"][i][field].clone()
        } else if json["dispersion"].get(q).is_some() {
          json["dispersion"][q].clone()
        } else {
          match q {
            "covariance" => json["covariance"][i][j].clone(),
            "mean" => json["fitted_means"][i].clone(),
            "xb" => {
              let offset = usize::from(problem.include_intercept);
              let eta = problem.design_matrix[i]
                .iter()
                .zip(&fit.coefficients[offset..])
                .map(|(x, b)| x * b.value)
                .sum::<f64>()
                + if offset == 1 {
                  fit.coefficients[0].value
                } else {
                  0.0
                };
              serde_json::json!(eta)
            }
            "wald_statistic" => json["omnibus_wald"]["statistic"].clone(),
            "wald_p_value" => json["omnibus_wald"]["p_value"].clone(),
            _ => json[q].clone(),
          }
        };
        assert!(
          actual.is_number(),
          "{case}, robust={robust}: {q} = {actual}"
        );
        close(&actual, row[5], q.ends_with("p_value"));
      }
      assert_eq!(fit.covariance.len(), problem.parameter_count() + 1);
      assert_eq!(fit.df_residual, 36 - problem.parameter_count() - 1);
      if robust {
        assert!(fit.likelihood_ratio_p_value.is_none());
      }
      if case == "intercept" {
        assert!(effects["effects"].as_array().unwrap().is_empty());
        assert_eq!(fit.likelihood_ratio, 0.0);
        assert!(fit.likelihood_ratio_p_value.is_none());
        if robust {
          assert_eq!(
            fit
              .omnibus_wald
              .as_ref()
              .unwrap()
              .unavailable_reason
              .as_deref(),
            Some("no_non_intercept_coefficients")
          );
        }
      }
    }
  }
}
#[test]
fn predictor_units_do_not_change_joint_inference_or_convergence() {
  let base = problem("multiple");
  let mut scaled = base.clone();
  for row in &mut scaled.design_matrix {
    row[0] *= 1e6;
    row[1] *= 1e-6;
  }
  for robust in [false, true] {
    let a = fit_nb2(&base, robust).unwrap();
    let b = fit_nb2(&scaled, robust).unwrap();
    let factors = [1., 1e6, 1e-6, 1.];
    for i in 0..4 {
      for j in 0..4 {
        close(
          &serde_json::json!(b.covariance[i][j] * factors[i] * factors[j]),
          &a.covariance[i][j].to_string(),
          false,
        );
      }
    }
    close(
      &serde_json::json!(a.dispersion.alpha),
      &b.dispersion.alpha.to_string(),
      false,
    );
    for (a, b) in a.fitted_means.iter().zip(&b.fitted_means) {
      close(&serde_json::json!(a), &b.to_string(), false);
    }
    let ea = nb2_average_marginal_effects(&base, &a).unwrap();
    let eb = nb2_average_marginal_effects(&scaled, &b).unwrap();
    for i in 0..2 {
      close(
        &serde_json::json!(eb.effects[i].value * factors[i + 1]),
        &ea.effects[i].value.to_string(),
        false,
      );
      close(
        &serde_json::json!(eb.effects[i].standard_error * factors[i + 1]),
        &ea.effects[i].standard_error.to_string(),
        false,
      );
    }
  }
}
#[test]
fn domain_rank_weights_sample_alignment_and_dispersion_boundary_failures_are_explicit() {
  for value in [-1., 0.5, 100001.] {
    let mut p = problem("multiple");
    p.response[0] = value;
    assert!(fit_nb2(&p, false).is_err());
  }
  for value in [0., 1.] {
    let mut p = problem("intercept");
    p.response.fill(value);
    assert!(fit_nb2(&p, false).is_err());
  }
  let mut p = problem("multiple");
  for row in &mut p.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_nb2(&p, false).is_err());
  let mut p = problem("multiple");
  p.weights = Some(vec![1.; 36]);
  assert!(fit_nb2(&p, false).is_err());
  let mut p = problem("multiple");
  p.predictor_names[0] = "lnalpha".into();
  assert!(fit_nb2(&p, false).is_err());
  for max_iterations in [0, 1] {
    assert!(
      fit_nb2_with_options(&problem("multiple"), false, &Nb2Options { max_iterations }).is_err()
    );
  }
  let p = problem("multiple");
  let mut fit = fit_nb2(&p, false).unwrap();
  fit.predictor_names.swap(0, 1);
  assert!(nb2_average_marginal_effects(&p, &fit).is_err());
}
