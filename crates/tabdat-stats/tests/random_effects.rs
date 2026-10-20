use tabdat_stats::{EstimationProblem, EstimationSample, StatsError, fit_random_effects};

fn problem(case: &str) -> EstimationProblem {
  let mut rows = Vec::new();
  for id in 1..=6 {
    let size = if case == "unbalanced" { id } else { 4 };
    for t in 0..size {
      let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
      let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
      let mut y =
        1.0 + 0.4 * f64::from(id) + 0.7 * x - 0.3 * z + 0.11 * f64::from((id * t + 3) % 7);
      if case == "zero_component" {
        y = 1.0 + 0.7 * x - 0.3 * z + 0.5 * (-1.0_f64).powi(t) * (1.0 + 0.02 * f64::from(id));
      }
      if case == "shifted" {
        y += 100.0;
      }
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
fn balanced_re_matches_independent_plm_and_explicit_gls_reference() {
  let reference: Vec<_> = include_str!("fixtures/random_effects_reference.csv")
    .lines()
    .skip(1)
    .map(|r| r.split(',').collect::<Vec<_>>())
    .collect();
  let mut checked = 0;
  for case in ["balanced", "zero_component", "interleaved", "shifted"] {
    let p = problem(case);
    let fit = fit_random_effects(&p, "person").unwrap();
    assert_eq!(fit.gls.sample, p.sample);
    assert_eq!(fit.gls.parameter_names, ["intercept", "x", "z"]);
    assert_eq!(fit.gls.fit_statistics.r_squared, None);
    assert_eq!(fit.gls.fit_statistics.log_likelihood, None);
    if case == "zero_component" {
      assert!(fit.raw_sigma_u_squared < 0.0);
      assert_eq!(fit.theta, 0.0);
      assert_eq!(fit.sigma_u_squared, 0.0);
    }
    let value = serde_json::to_value(fit).unwrap();
    for row in reference.iter().filter(|r| r[0] == case) {
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
        actual.is_finite() && (actual - expected).abs() <= tolerance,
        "{row:?}: actual={actual:.17e}; tolerance={tolerance:.3e}"
      );
      checked += 1;
    }
  }
  assert_eq!(checked, 352);
}

#[test]
fn unsupported_samples_weights_rank_and_degenerate_variances_fail_explicitly() {
  assert!(matches!(
    fit_random_effects(&problem("unbalanced"), "person"),
    Err(StatsError::InvalidEstimation(_))
  ));
  let mut p = problem("balanced");
  p.weights = Some(vec![1.0; p.response.len()]);
  assert!(fit_random_effects(&p, "person").is_err());
  p = problem("balanced");
  for x in &mut p.design_matrix {
    x[1] = x[0];
  }
  assert!(fit_random_effects(&p, "person").is_err());
  p = problem("balanced");
  p.response.fill(1.0);
  assert!(fit_random_effects(&p, "person").is_err());
  p = problem("balanced");
  p.sample.as_mut().unwrap().cluster_groups = None;
  assert!(fit_random_effects(&p, "person").is_err());
}
