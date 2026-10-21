use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_random_effects, fit_random_effects_unbalanced,
};
#[test]
fn unbalanced_trace_components_coefficients_and_covariance_match_plm_and_direct_matrices() {
  let mut compared = 0;
  for case in ["unequal", "shifted_interleaved", "zero_component"] {
    let rows: Vec<Vec<f64>> = include_str!("fixtures/random_effects_gls_unbalanced_inputs.csv")
      .lines()
      .skip(1)
      .filter_map(|l| {
        let r: Vec<_> = l.split(',').collect();
        (r[0] == case).then(|| r[1..].iter().map(|v| v.parse().unwrap()).collect())
      })
      .collect();
    let n = rows.len();
    let p = EstimationProblem::new(
      "y",
      vec!["x".into(), "z".into()],
      rows.iter().map(|r| r[2]).collect(),
      rows.iter().map(|r| vec![r[3], r[4]]).collect(),
    )
    .unwrap()
    .with_sample(
      EstimationSample::new(
        (0..n).collect(),
        n,
        0,
        None,
        Some(rows.iter().map(|r| r[0].to_string()).collect()),
      )
      .unwrap(),
    );
    assert!(fit_random_effects(&p, "id").is_err());
    let fit = fit_random_effects_unbalanced(&p, "id").unwrap();
    let value = serde_json::to_value(&fit).unwrap();
    for l in include_str!("fixtures/random_effects_gls_unbalanced_reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = l.split(',').collect();
      if r[0] != case {
        continue;
      }
      let a = value
        .pointer(r[1])
        .and_then(serde_json::Value::as_f64)
        .unwrap();
      let e: f64 = r[2].parse().unwrap();
      let tolerance = if r[1].ends_with("p_value") {
        e.abs() * 1e-7 + 1e-300
      } else {
        1e-8
      };
      assert!((a - e).abs() <= tolerance, "{r:?}: {a} != {e}");
      compared += 1;
    }
  }
  assert_eq!(compared, 243);
}
