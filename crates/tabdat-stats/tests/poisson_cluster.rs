use serde_json::{Value, json};
use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_poisson, fit_poisson_clustered,
  poisson_average_marginal_effects,
};

fn problem(case: &str) -> EstimationProblem {
  let y = vec![
    0., 0., 2., 0., 0., 3., 0., 3., 1., 3., 8., 3., 0., 0., 0., 2., 1., 2., 0., 0., 2., 1., 1., 2.,
    0., 5., 4., 1., 1., 0., 0., 2., 2., 0., 1., 0.,
  ];
  let (names, x) = if case == "intercept" {
    (vec![], vec![vec![]; 36])
  } else {
    (
      vec!["x".into(), "z".into()],
      (0..36)
        .map(|i| vec![-2. + 4. * i as f64 / 35., [0., 1., 0.4][i % 3]])
        .collect(),
    )
  };
  let groups = (0..36)
    .map(|i| format!("g{}", if case == "singleton" { i } else { i % 6 }))
    .collect();
  EstimationProblem::new("y", names, y, x)
    .unwrap()
    .with_intercept(case != "noconstant")
    .with_sample(EstimationSample::new((0..36).collect(), 36, 0, None, Some(groups)).unwrap())
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
    "{actual} != {expected}; tol={tolerance}"
  );
}
#[test]
fn group_scores_normal_inference_wald_effects_and_predictions_match_sandwich() {
  let rows: Vec<Vec<_>> = include_str!("fixtures/poisson_cluster_reference.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect())
    .collect();
  assert_eq!(rows.len(), 467);
  for case in ["multiple", "noconstant", "intercept", "singleton"] {
    let p = problem(case);
    let fit = fit_poisson_clustered(&p, "person").unwrap();
    assert!(fit.robust && fit.likelihood_ratio_p_value.is_none());
    assert!(fit.max_abs_scaled_score <= 1e-10 && fit.max_abs_scaled_newton_step <= 1e-10);
    let effects = poisson_average_marginal_effects(&p, &fit).unwrap();
    assert_eq!(effects.cluster, fit.cluster);
    let j = serde_json::to_value(&fit).unwrap();
    let e = serde_json::to_value(&effects).unwrap();
    for row in rows.iter().filter(|r| r[0] == case) {
      let q = row[1];
      let i: usize = row[2].parse().unwrap();
      let k: usize = row[3].parse().unwrap();
      let actual = if let Some(field) = q.strip_prefix("coefficient_") {
        j["coefficients"][i][field].clone()
      } else if let Some(field) = q.strip_prefix("effect_") {
        e["effects"][i][field].clone()
      } else {
        match q {
          "covariance" => j["covariance"][i][k].clone(),
          "cluster_group_count" => j["cluster"]["group_count"].clone(),
          "cluster_correction" => {
            let g = fit.cluster.as_ref().unwrap().group_count as f64;
            json!(g / (g - 1.))
          }
          "wald_statistic" => j["omnibus_wald"]["statistic"].clone(),
          "wald_p_value" => j["omnibus_wald"]["p_value"].clone(),
          "mean" => j["fitted_means"][i].clone(),
          "xb" => {
            let offset = usize::from(p.include_intercept);
            json!(
              p.design_matrix[i]
                .iter()
                .zip(&fit.coefficients[offset..])
                .map(|(x, b)| x * b.value)
                .sum::<f64>()
                + if offset == 1 {
                  fit.coefficients[0].value
                } else {
                  0.
                }
            )
          }
          _ => j[q].clone(),
        }
      };
      // The predeclared accumulated-total contract is 1e-7, not the
      // coefficient/covariance/Wald contract of 1e-10.
      if matches!(
        q,
        "log_likelihood"
          | "null_log_likelihood"
          | "likelihood_ratio"
          | "residual_deviance"
          | "pearson_chi_square"
      ) {
        let expected: f64 = row[4].parse().unwrap();
        let actual = actual.as_f64().unwrap();
        assert!(
          (actual - expected).abs() <= 1e-7,
          "{case}/{q}: {actual} != {expected}"
        );
      } else {
        close(&actual, row[4], q.ends_with("p_value"));
      }
    }
    if case == "intercept" {
      assert!(effects.effects.is_empty());
      assert_eq!(
        fit.omnibus_wald.unwrap().unavailable_reason.as_deref(),
        Some("no_non_intercept_coefficients")
      );
    }
  }
}
#[test]
fn singleton_groups_match_existing_robust_and_relabeling_and_units_preserve_inference() {
  let p = problem("singleton");
  let fit = fit_poisson_clustered(&p, "person").unwrap();
  let mut ungrouped = p.clone();
  ungrouped.sample.as_mut().unwrap().cluster_groups = None;
  let robust = fit_poisson(&ungrouped, true).unwrap();
  for i in 0..3 {
    for j in 0..3 {
      close(
        &json!(fit.covariance[i][j]),
        &robust.covariance[i][j].to_string(),
        false,
      );
    }
  }
  assert_eq!(
    fit.coefficients.iter().map(|b| b.value).collect::<Vec<_>>(),
    robust
      .coefficients
      .iter()
      .map(|b| b.value)
      .collect::<Vec<_>>()
  );
  let p = problem("multiple");
  let a = fit_poisson_clustered(&p, "person").unwrap();
  let mut b = p.clone();
  b.sample.as_mut().unwrap().cluster_groups =
    Some((0..36).map(|i| format!("renamed{}", 6 - i % 6)).collect());
  for row in &mut b.design_matrix {
    row[0] *= 1e6;
    row[1] *= 1e-6;
  }
  let b = fit_poisson_clustered(&b, "other").unwrap();
  let factors = [1., 1e6, 1e-6];
  for i in 0..3 {
    for j in 0..3 {
      close(
        &json!(b.covariance[i][j] * factors[i] * factors[j]),
        &a.covariance[i][j].to_string(),
        false,
      );
    }
  }
  for (a, b) in a.fitted_means.iter().zip(&b.fitted_means) {
    close(&json!(b), &a.to_string(), false);
  }
  close(
    &json!(b.omnibus_wald.unwrap().statistic.unwrap()),
    &a.omnibus_wald.unwrap().statistic.unwrap().to_string(),
    false,
  );
}
#[test]
fn missing_misaligned_single_groups_weights_and_noncluster_admission_fail() {
  let p = problem("multiple");
  assert!(fit_poisson(&p, false).is_err());
  assert!(fit_poisson(&p, true).is_err());
  assert!(fit_poisson_clustered(&p, "").is_err());
  let mut absent = p.clone();
  absent.sample = None;
  assert!(fit_poisson_clustered(&absent, "id").is_err());
  let mut short = p.clone();
  short
    .sample
    .as_mut()
    .unwrap()
    .cluster_groups
    .as_mut()
    .unwrap()
    .pop();
  assert!(fit_poisson_clustered(&short, "id").is_err());
  let mut one = p.clone();
  one.sample.as_mut().unwrap().cluster_groups = Some(vec!["only".into(); 36]);
  assert!(fit_poisson_clustered(&one, "id").is_err());
  let mut weighted = p.clone();
  weighted.weights = Some(vec![1.; 36]);
  assert!(fit_poisson_clustered(&weighted, "id").is_err());
  let mut two = p.clone();
  two.sample.as_mut().unwrap().cluster_groups =
    Some((0..36).map(|i| (i % 2).to_string()).collect());
  let fit = fit_poisson_clustered(&two, "id").unwrap();
  let w = fit.omnibus_wald.unwrap();
  assert_eq!(w.degrees_of_freedom, 2);
  assert_eq!(
    w.unavailable_reason.as_deref(),
    Some("insufficient_cluster_rank")
  );
  assert!(w.statistic.is_none());
}
#[test]
fn zero_cluster_variance_and_old_owned_fit_reads_do_not_invent_inference() {
  let p = EstimationProblem::new("y", vec![], vec![1.; 6], vec![vec![]; 6])
    .unwrap()
    .with_sample(
      EstimationSample::new(
        (0..6).collect(),
        6,
        0,
        None,
        Some(vec![
          "a".into(),
          "a".into(),
          "a".into(),
          "b".into(),
          "b".into(),
          "b".into(),
        ]),
      )
      .unwrap(),
    );
  let fit = fit_poisson_clustered(&p, "id").unwrap();
  let b = &fit.coefficients[0];
  assert_eq!(b.standard_error, 0.);
  assert!(
    b.p_value.is_none()
      && b.statistic.is_none()
      && b.confidence_lower.is_none()
      && b.confidence_upper.is_none()
  );
  let mut p = problem("multiple");
  p.sample.as_mut().unwrap().cluster_groups = None;
  let fit = fit_poisson(&p, false).unwrap();
  let wire = serde_json::to_value(&fit).unwrap();
  assert!(wire.get("cluster").is_none());
  let read: tabdat_stats::PoissonResult = serde_json::from_value(wire).unwrap();
  assert_eq!(fit, read);
  let e = poisson_average_marginal_effects(&p, &fit).unwrap();
  let wire = serde_json::to_value(&e).unwrap();
  assert!(wire.get("cluster").is_none());
  let read: tabdat_stats::PoissonMarginalEffects = serde_json::from_value(wire).unwrap();
  assert_eq!(e, read);
}
