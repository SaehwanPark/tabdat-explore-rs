use serde_json::json;
use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_nb2, fit_nb2_clustered, nb2_average_marginal_effects,
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
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= 1e-10,
    "{actual} != {expected}"
  );
}
#[test]
fn full_joint_group_covariance_dispersion_wald_effects_and_predictions_match_independent_r() {
  let references: Vec<Vec<_>> = include_str!("fixtures/nb2_cluster_reference.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect())
    .collect();
  assert_eq!(references.len(), 535);
  for case in ["multiple", "noconstant", "intercept", "singleton"] {
    let p = problem(case);
    let fit = fit_nb2_clustered(&p, "person").unwrap();
    let effects = nb2_average_marginal_effects(&p, &fit).unwrap();
    let j = serde_json::to_value(&fit).unwrap();
    let e = serde_json::to_value(&effects).unwrap();
    for row in references.iter().filter(|r| r[0] == case) {
      let q = row[1];
      let i: usize = row[2].parse().unwrap();
      let k: usize = row[3].parse().unwrap();
      let actual = if let Some(field) = q.strip_prefix("coefficient_") {
        j["coefficients"][i][field].clone()
      } else if let Some(field) = q.strip_prefix("effect_") {
        e["effects"][i][field].clone()
      } else if let Some(field) = q.strip_prefix("dispersion_test_") {
        j["dispersion_test"][field].clone()
      } else if j["dispersion"].get(q).is_some() {
        j["dispersion"][q].clone()
      } else {
        match q {
          "covariance" => j["covariance"][i][k].clone(),
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
          "wald_statistic" => j["omnibus_wald"]["statistic"].clone(),
          "wald_p_value" => j["omnibus_wald"]["p_value"].clone(),
          "cluster_group_count" => j["cluster"]["group_count"].clone(),
          "cluster_correction" => {
            let g = fit.cluster.as_ref().unwrap().group_count as f64;
            json!(g / (g - 1.))
          }
          _ => j[q].clone(),
        }
      };
      if row[4] == "NA" {
        assert!(actual.is_null(), "{case}/{q}: {actual}");
        continue;
      }
      let expected: f64 = row[4].parse().unwrap();
      let actual = actual.as_f64().unwrap();
      let tolerance = if q.ends_with("p_value") {
        expected.abs() * 1e-10 + f64::MIN_POSITIVE
      } else if matches!(
        q,
        "log_likelihood"
          | "null_log_likelihood"
          | "likelihood_ratio"
          | "residual_deviance"
          | "pearson_chi_square"
      ) {
        1e-7
      } else {
        1e-10
      };
      assert!(
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "{case}/{q}: {actual} != {expected}, tolerance {tolerance}"
      );
    }
    assert!(j["dispersion"].get("p_value").is_none());
    assert!(j["dispersion_test"]["p_value"].is_null());
    assert!(j["dispersion_test"]["reference_distribution"].is_null());
    assert_eq!(
      j["dispersion_test"]["unavailable_reason"],
      "clustered_likelihood_not_independent"
    );
    assert_eq!(effects.cluster, fit.cluster);
  }
}
#[test]
fn clustered_nb2_preserves_point_null_and_convergence_and_singletons_equal_full_joint_row_robust() {
  for case in ["multiple", "noconstant", "intercept", "singleton"] {
    let p = problem(case);
    let a = fit_nb2_clustered(&p, "person").unwrap();
    let mut ungrouped = p.clone();
    ungrouped.sample.as_mut().unwrap().cluster_groups = None;
    let b = fit_nb2(&ungrouped, false).unwrap();
    let r = fit_nb2(&ungrouped, true).unwrap();
    assert!(a.robust && a.likelihood_ratio_p_value.is_none());
    assert_eq!(
      a.cluster.as_ref().unwrap().group_count,
      if case == "singleton" { 36 } else { 6 }
    );
    assert_eq!(a.fitted_means, b.fitted_means);
    assert_eq!(a.dispersion.alpha, b.dispersion.alpha);
    assert_eq!(a.null_alpha, b.null_alpha);
    assert_eq!(a.log_likelihood, b.log_likelihood);
    assert_eq!(a.null_log_likelihood, b.null_log_likelihood);
    assert_eq!(a.iterations, b.iterations);
    assert_eq!(a.max_abs_scaled_score, b.max_abs_scaled_score);
    assert_eq!(
      a.coefficients.iter().map(|b| b.value).collect::<Vec<_>>(),
      b.coefficients.iter().map(|b| b.value).collect::<Vec<_>>()
    );
    assert_eq!(a.df_residual, 36 - p.parameter_count() - 1);
    assert_eq!(a.covariance.len(), p.parameter_count() + 1);
    let e = nb2_average_marginal_effects(&p, &a).unwrap();
    assert_eq!(e.cluster, a.cluster);
    if case == "singleton" {
      for (i, row) in a.covariance.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
          close(*v, r.covariance[i][j]);
        }
      }
      close(
        a.dispersion.log_alpha_standard_error,
        r.dispersion.log_alpha_standard_error,
      );
    }
    if case == "intercept" {
      assert!(e.effects.is_empty());
      assert_eq!(
        a.omnibus_wald.unwrap().unavailable_reason.as_deref(),
        Some("no_non_intercept_coefficients")
      );
    }
  }
}
#[test]
fn cluster_labels_and_predictor_units_preserve_joint_dispersion_covariance_effects_and_wald() {
  let p = problem("multiple");
  let a = fit_nb2_clustered(&p, "person").unwrap();
  let mut changed = p.clone();
  changed.sample.as_mut().unwrap().cluster_groups =
    Some((0..36).map(|i| format!("renamed{}", 6 - i % 6)).collect());
  for row in &mut changed.design_matrix {
    row[0] *= 1e6;
    row[1] *= 1e-6;
  }
  let b = fit_nb2_clustered(&changed, "other").unwrap();
  let factors = [1., 1e6, 1e-6, 1.];
  for i in 0..4 {
    for j in 0..4 {
      close(
        b.covariance[i][j] * factors[i] * factors[j],
        a.covariance[i][j],
      );
    }
  }
  close(a.dispersion.alpha, b.dispersion.alpha);
  close(
    a.dispersion.log_alpha_standard_error,
    b.dispersion.log_alpha_standard_error,
  );
  close(
    a.omnibus_wald.as_ref().unwrap().statistic.unwrap(),
    b.omnibus_wald.as_ref().unwrap().statistic.unwrap(),
  );
  let ea = nb2_average_marginal_effects(&p, &a).unwrap();
  let eb = nb2_average_marginal_effects(&changed, &b).unwrap();
  for i in 0..2 {
    close(ea.effects[i].value, eb.effects[i].value * factors[i + 1]);
    close(
      ea.effects[i].standard_error,
      eb.effects[i].standard_error * factors[i + 1],
    );
  }
}
#[test]
fn exact_group_admission_rank_guards_and_old_owned_reads_are_preserved() {
  let p = problem("multiple");
  assert!(fit_nb2(&p, false).is_err());
  assert!(fit_nb2(&p, true).is_err());
  assert!(fit_nb2_clustered(&p, "").is_err());
  let mut absent = p.clone();
  absent.sample = None;
  assert!(fit_nb2_clustered(&absent, "id").is_err());
  let mut short = p.clone();
  short
    .sample
    .as_mut()
    .unwrap()
    .cluster_groups
    .as_mut()
    .unwrap()
    .pop();
  assert!(fit_nb2_clustered(&short, "id").is_err());
  let mut one = p.clone();
  one.sample.as_mut().unwrap().cluster_groups = Some(vec!["only".into(); 36]);
  assert!(fit_nb2_clustered(&one, "id").is_err());
  let mut weighted = p.clone();
  weighted.weights = Some(vec![1.; 36]);
  assert!(fit_nb2_clustered(&weighted, "id").is_err());
  let mut two = p.clone();
  two.sample.as_mut().unwrap().cluster_groups =
    Some((0..36).map(|i| (i % 2).to_string()).collect());
  let a = fit_nb2_clustered(&two, "id").unwrap();
  let w = a.omnibus_wald.unwrap();
  assert_eq!(w.degrees_of_freedom, 2);
  assert_eq!(
    w.unavailable_reason.as_deref(),
    Some("insufficient_cluster_rank")
  );
  assert!(w.statistic.is_none() && w.p_value.is_none());
  let mut p = p;
  p.sample.as_mut().unwrap().cluster_groups = None;
  let fit = fit_nb2(&p, false).unwrap();
  let wire = serde_json::to_value(&fit).unwrap();
  assert!(wire.get("cluster").is_none());
  assert_eq!(
    serde_json::from_value::<tabdat_stats::Nb2Result>(wire).unwrap(),
    fit
  );
  let e = nb2_average_marginal_effects(&p, &fit).unwrap();
  let wire = serde_json::to_value(&e).unwrap();
  assert!(wire.get("cluster").is_none());
  assert_eq!(
    serde_json::from_value::<tabdat_stats::Nb2MarginalEffects>(wire).unwrap(),
    e
  );
  let grouped = fit_nb2_clustered(&problem("multiple"), "person").unwrap();
  let wire = serde_json::to_value(&grouped).unwrap();
  assert_eq!(
    serde_json::from_value::<tabdat_stats::Nb2Result>(wire).unwrap(),
    grouped
  );
}
