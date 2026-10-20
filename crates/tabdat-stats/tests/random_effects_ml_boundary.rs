use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_random_effects_ml, fit_random_effects_ml_interior,
};

#[test]
fn active_zero_variance_face_matches_gaussian_lm_ml_and_constrained_information() {
  let mut comparisons = 0;
  for case in ["pooled", "shifted_interleaved", "both_boundary"] {
    let mut rows = Vec::new();
    for id in 1..=8 {
      for t in 0..4 {
        let x = 0.3 * f64::from(id) + 0.2 * f64::from(t) + 0.07 * f64::from((id * t + 2) % 5);
        let z = 0.13 * f64::from(t * t) + 0.03 * f64::from((id + 2 * t) % 4);
        let y = (if case == "both_boundary" {
          0.0
        } else {
          0.7 * x
        }) + [-0.2, 0.2, 0.2, -0.2][t as usize]
          + if case == "shifted_interleaved" {
            100.0
          } else {
            0.0
          };
        rows.push((id, t, y, x, z));
      }
    }
    if case == "shifted_interleaved" {
      rows.sort_by_key(|r| (-r.1, -r.0));
    }
    let p = EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      rows.iter().map(|r| r.2).collect(),
      rows.iter().map(|r| vec![r.3, r.4]).collect(),
    )
    .unwrap()
    .with_sample(
      EstimationSample::new(
        (0..32).collect(),
        32,
        0,
        None,
        Some(rows.iter().map(|r| r.0.to_string()).collect()),
      )
      .unwrap(),
    );
    let fit = fit_random_effects_ml(&p, "id").unwrap();
    assert_eq!(fit.gls.sample, p.sample);
    let ml = fit.ml.as_ref().unwrap();
    assert!(ml.unit_variance_boundary());
    assert_eq!(ml.null_sigma_u_squared == 0.0, case == "both_boundary");
    assert_eq!(
      ml.likelihood_ratio_unavailable_reason
        .as_ref()
        .unwrap()
        .contains("restricted-null"),
      case == "both_boundary"
    );
    assert!(ml.profile_score < 0.0);
    assert_eq!(
      ml.joint_parameter_names,
      ["intercept", "x", "z", "log_sigma_e_squared"]
    );
    assert!(ml.likelihood_ratio_p_value.is_none());
    assert!(
      ml.likelihood_ratio_unavailable_reason
        .as_ref()
        .unwrap()
        .contains("full-model")
    );
    assert!(
      fit_random_effects_ml_interior(&p, "id")
        .unwrap_err()
        .to_string()
        .contains("zero-unit-variance optimum")
    );
    let value = serde_json::to_value(fit).unwrap();
    for line in include_str!("fixtures/random_effects_ml_boundary_reference.csv")
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
        "{case} {}: {actual} vs {expected}",
        row[1]
      );
      comparisons += 1;
    }
  }
  assert_eq!(comparisons, 372);
}
