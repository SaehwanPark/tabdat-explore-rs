use tabdat_stats::{EstimationProblem, EstimationSample, fit_random_effects_ml_interior};

fn problem(case: &str) -> EstimationProblem {
  let mut rows = Vec::new();
  for id in 1..=8 {
    for t in 0..4 {
      let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
      let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
      let mut y = 1.0 + 0.7 * x - 0.3 * z
        + 0.3 * f64::from((id * id + 1) % 5)
        + 0.11 * f64::from((id * t + 3) % 7);
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
      (0..n).collect(),
      n,
      0,
      None,
      Some(rows.iter().map(|r| r.0.to_string()).collect()),
    )
    .unwrap(),
  )
}

#[test]
fn interior_ml_matches_nlme_point_fit_and_independent_joint_information() {
  let reference: Vec<_> = include_str!("fixtures/random_effects_ml_reference.csv")
    .lines()
    .skip(1)
    .map(|s| s.split(',').collect::<Vec<_>>())
    .collect();
  let mut checked = 0;
  for case in ["balanced", "interleaved", "shifted"] {
    let p = problem(case);
    let fit = fit_random_effects_ml_interior(&p, "id").unwrap();
    assert_eq!(fit.regression.sample, p.sample);
    assert!(fit.profile_score.abs() < 1e-9);
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
        "{row:?}: actual={actual:.17e}, tol={tolerance:.3e}"
      );
      checked += 1;
    }
  }
  assert_eq!(checked, 336);
}

#[test]
fn unsupported_or_degenerate_interior_requests_fail_without_a_result() {
  let mut p = problem("balanced");
  p.weights = Some(vec![1.0; p.response.len()]);
  assert!(fit_random_effects_ml_interior(&p, "id").is_err());
  p = problem("balanced");
  p.response.fill(0.0);
  assert!(fit_random_effects_ml_interior(&p, "id").is_err());
  p = problem("balanced");
  p.sample.as_mut().unwrap().cluster_groups = None;
  assert!(fit_random_effects_ml_interior(&p, "id").is_err());
}
