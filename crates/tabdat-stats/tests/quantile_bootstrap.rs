use tabdat_stats::{EstimationProblem, bootstrap_quantiles};

fn problem(case: &str) -> EstimationProblem {
  let rows: Vec<Vec<f64>> = include_str!("fixtures/quantile-bootstrap/inputs.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "z".into()],
    rows
      .iter()
      .map(|r| r[0] + if case == "shifted" { 100.0 } else { 0.0 })
      .collect(),
    rows
      .iter()
      .map(|r| vec![r[1] * if case == "rescaled" { 100.0 } else { 1.0 }, r[2]])
      .collect(),
  )
  .unwrap()
  .with_intercept(case != "noconstant")
}

#[test]
fn joint_covariance_bias_and_percentiles_match_independent_common_draw_quantreg() {
  let mut comparisons = 0;
  for (case, seed) in [
    ("joint", u64::MAX),
    ("noconstant", 7),
    ("shifted", 7),
    ("rescaled", 7),
  ] {
    let p = problem(case);
    let fit = bootstrap_quantiles(&p, &[0.2, 0.5, 0.8], 40, seed).unwrap();
    assert_eq!(
      fit,
      bootstrap_quantiles(&p, &[0.2, 0.5, 0.8], 40, seed).unwrap()
    );
    // Adding quantiles consumes no extra RNG draws or changes marginal fits.
    let median = bootstrap_quantiles(&p, &[0.5], 40, seed).unwrap();
    let k = p.parameter_count();
    assert_eq!(fit.coefficients[k..2 * k], median.coefficients);
    for i in 0..k {
      for j in 0..k {
        assert_eq!(
          fit.covariance_matrix[k + i][k + j],
          median.covariance_matrix[i][j]
        );
      }
    }
    let value = serde_json::to_value(fit).unwrap();
    for line in include_str!("fixtures/quantile-bootstrap/reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = line.split(',').collect();
      if r[0] != case {
        continue;
      }
      let actual = value.pointer(r[1]).unwrap().as_f64().unwrap();
      let expected: f64 = r[2].parse().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-8,
        "{r:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 477);
}

#[test]
fn bounds_and_failed_original_or_replicate_are_explicit() {
  let p = problem("joint");
  for q in [
    vec![],
    vec![0.0],
    vec![f64::NAN],
    vec![0.5, 0.5],
    vec![0.8, 0.2],
    vec![0.5; 20],
  ] {
    assert!(bootstrap_quantiles(&p, &q, 40, 7).is_err());
  }
  for b in [0, 1, 5001, usize::MAX] {
    assert!(bootstrap_quantiles(&p, &[0.5], b, 7).is_err());
  }
  assert!(bootstrap_quantiles(&p.clone().with_weights(vec![1.; 120]), &[0.5], 40, 7).is_err());
  let small = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1., 2., 4., 8., 16.],
    vec![vec![0.], vec![1.], vec![2.], vec![3.], vec![4.]],
  )
  .unwrap();
  let error = bootstrap_quantiles(&small, &[0.5], 100, 1)
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  let mut rank = p.clone();
  rank.design_matrix = vec![vec![1., 1.]; 120];
  assert!(bootstrap_quantiles(&rank, &[0.5], 40, 7).is_err());
  let quantiles: Vec<_> = (1..=19).map(|i| i as f64 / 20.0).collect();
  assert!(
    bootstrap_quantiles(&p, &quantiles, 5000, 7)
      .unwrap_err()
      .to_string()
      .contains("N*K²*quantiles*(replications+1)")
  );
}

#[test]
fn identified_exact_point_fits_do_not_require_replicate_iid_sparsity() {
  let p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    (0..120).map(|i| 1.0 + 2.0 * i as f64).collect(),
    (0..120).map(|i| vec![i as f64]).collect(),
  )
  .unwrap();
  assert!(tabdat_stats::fit_quantile_iid(&p, 0.5).is_err());
  let fit = bootstrap_quantiles(&p, &[0.2, 0.5, 0.8], 40, 7).unwrap();
  for row in fit.coefficients.chunks(2) {
    assert!((row[0].estimate - 1.0).abs() < 1e-8);
    assert!((row[1].estimate - 2.0).abs() < 1e-8);
    assert!(row.iter().all(|b| b.standard_error < 1e-8));
  }
}

#[test]
fn near_active_refinement_matches_independent_quantreg_without_relaxing_certificates() {
  let mut p = problem("joint");
  let expected = [1.0737644346688988, 0.7485521398303408, -0.19414337088513894];
  p.response[110] = expected[0]
    + expected[1] * p.design_matrix[110][0]
    + expected[2] * p.design_matrix[110][1]
    + 1e-8;
  let fit = tabdat_stats::fit_quantile_iid(&p, 0.35).unwrap();
  assert!(fit.relative_duality_gap <= 1e-8 && fit.max_score_violation <= 1e-7);
  let value = serde_json::to_value(fit).unwrap();
  let mut compared = 0;
  for line in include_str!("fixtures/quantile-bootstrap/reference.csv")
    .lines()
    .skip(1)
  {
    let r: Vec<_> = line.split(',').collect();
    if r[0] != "near_active" {
      continue;
    }
    let a = value.pointer(r[1]).unwrap().as_f64().unwrap();
    let e: f64 = r[2].parse().unwrap();
    let tolerance = if r[1].ends_with("p_value") {
      e.abs() * 1e-7 + 1e-300
    } else {
      1e-8
    };
    assert!((a - e).abs() <= tolerance, "{r:?}: {a} != {e}");
    compared += 1;
  }
  assert_eq!(compared, 27);
}

#[test]
fn shared_pair_contrasts_match_independent_quantreg_and_preserve_old_reports() {
  let mut compared = 0;
  for (case, seed) in [
    ("joint", u64::MAX),
    ("noconstant", 7),
    ("shifted", 7),
    ("rescaled", 7),
  ] {
    let p = problem(case);
    let baseline = bootstrap_quantiles(&p, &[0.25, 0.75], 40, seed).unwrap();
    let old_json = serde_json::to_value(&baseline).unwrap();
    assert!(old_json.get("contrast").is_none());
    let roundtrip: tabdat_stats::QuantileBootstrapResult =
      serde_json::from_value(old_json).unwrap();
    assert_eq!(roundtrip, baseline);
    for parameter in p.parameter_names() {
      let mut report =
        tabdat_stats::bootstrap_quantiles_with_contrast(&p, &[0.25, 0.75], 40, seed, &parameter)
          .unwrap();
      let contrast = report.contrast.as_ref().unwrap();
      assert!(contrast.inference_unavailable_reason.is_none());
      assert_eq!(
        contrast.reference_distribution.as_deref(),
        Some("asymptotic_standard_normal")
      );
      let value = serde_json::to_value(&report).unwrap();
      for line in include_str!("fixtures/quantile-bootstrap/contrast_reference.csv")
        .lines()
        .skip(1)
      {
        let row: Vec<_> = line.split(',').collect();
        if row[0] != format!("{case}:{parameter}") {
          continue;
        }
        let actual = value.pointer(row[1]).unwrap().as_f64().unwrap();
        let expected: f64 = row[2].parse().unwrap();
        let tolerance = if row[1].ends_with("p_value") {
          expected.abs() * 1e-7 + 1e-300
        } else {
          1e-8
        };
        assert!(
          (actual - expected).abs() <= tolerance,
          "{row:?}: {actual} != {expected}"
        );
        compared += 1;
      }
      report.contrast = None;
      assert_eq!(report, baseline);
    }
  }
  assert_eq!(compared, 66);
}

#[test]
fn malformed_contrasts_fail_and_zero_variance_keeps_marginal_results_without_normal_inference() {
  let p = problem("joint");
  let mut ambiguous = p.clone();
  ambiguous.predictor_names[0] = ambiguous.intercept_name.clone();
  assert!(
    tabdat_stats::bootstrap_quantiles_with_contrast(&ambiguous, &[0.25, 0.75], 40, 7, "intercept")
      .unwrap_err()
      .to_string()
      .contains("ambiguous contrast coefficient")
  );
  for (qs, name) in [
    (vec![0.5], "x"),
    (vec![0.2, 0.5, 0.8], "x"),
    (vec![0.25, 0.75], "absent"),
  ] {
    assert!(tabdat_stats::bootstrap_quantiles_with_contrast(&p, &qs, 40, 7, name).is_err());
  }
  assert!(
    tabdat_stats::bootstrap_quantiles_with_contrast(
      &problem("noconstant"),
      &[0.25, 0.75],
      40,
      7,
      "intercept"
    )
    .is_err()
  );
  let exact = EstimationProblem::new(
    "y",
    vec!["x".into()],
    (0..120).map(|i| 1. + 2. * i as f64).collect(),
    (0..120).map(|i| vec![i as f64]).collect(),
  )
  .unwrap();
  let report =
    tabdat_stats::bootstrap_quantiles_with_contrast(&exact, &[0.25, 0.75], 40, 7, "x").unwrap();
  let c = report.contrast.as_ref().unwrap();
  assert_eq!(c.estimate, 0.);
  assert_eq!(c.standard_error, Some(0.));
  assert_eq!(
    c.inference_unavailable_reason.as_deref(),
    Some("zero_contrast_variance")
  );
  assert!(
    c.statistic.is_none()
      && c.p_value.is_none()
      && c.ci_lower.is_none()
      && c.ci_upper.is_none()
      && c.reference_distribution.is_none()
  );
  assert_eq!(report.coefficients.len(), 4);
}
