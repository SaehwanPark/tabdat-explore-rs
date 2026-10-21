use tabdat_stats::{EstimationProblem, EstimationSample, bootstrap_ols_clustered};
fn problem(case: &str) -> EstimationProblem {
  let rows: Vec<Vec<&str>> = include_str!("fixtures/cluster-bootstrap/inputs.csv")
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect())
    .collect();
  let groups = rows
    .iter()
    .map(|r| r[if case == "balanced" { 3 } else { 4 }].to_string())
    .collect();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "z".into()],
    rows
      .iter()
      .map(|r| r[0].parse::<f64>().unwrap() + if case == "shifted" { 100.0 } else { 0.0 })
      .collect(),
    rows
      .iter()
      .map(|r| vec![r[1].parse().unwrap(), r[2].parse().unwrap()])
      .collect(),
  )
  .unwrap()
  .with_intercept(case != "noconstant")
  .with_sample(EstimationSample::new((0..20).collect(), 20, 0, None, Some(groups)).unwrap())
}
#[test]
fn whole_cluster_draws_match_r_and_preserve_variable_unbalanced_sizes() {
  let mut comparisons = 0;
  for (case, seed) in [
    ("balanced", u64::MAX),
    ("unbalanced", 7),
    ("noconstant", 7),
    ("shifted", 7),
  ] {
    let p = problem(case);
    let fit = bootstrap_ols_clustered(&p, 40, seed).unwrap();
    assert_eq!(fit, bootstrap_ols_clustered(&p, 40, seed).unwrap());
    assert_ne!(
      fit,
      bootstrap_ols_clustered(&p, 40, seed.wrapping_add(1)).unwrap()
    );
    let json = serde_json::to_value(&fit).unwrap();
    for line in include_str!("fixtures/cluster-bootstrap/reference.csv")
      .lines()
      .skip(1)
    {
      let r: Vec<_> = line.split(',').collect();
      if r[0] != case {
        continue;
      }
      let actual = json.pointer(r[1]).unwrap().as_f64().unwrap();
      let expected: f64 = r[2].parse().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-9,
        "{r:?}: {actual} != {expected}"
      );
      comparisons += 1;
    }
    if case == "balanced" {
      assert_eq!(fit.replicate_observation_counts, vec![20; 40]);
    } else {
      assert_ne!(
        fit.replicate_observation_counts.iter().min(),
        fit.replicate_observation_counts.iter().max()
      );
    }
  }
  assert_eq!(comparisons, 285);
}
#[test]
fn group_failures_work_bound_and_rank_failures_are_explicit() {
  let mut p = problem("unbalanced");
  p.sample = None;
  assert!(bootstrap_ols_clustered(&p, 40, 1).is_err());
  p = problem("unbalanced");
  p.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 20]);
  assert!(bootstrap_ols_clustered(&p, 40, 1).is_err());
  p.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 21]);
  assert!(
    bootstrap_ols_clustered(&p, 40, 1)
      .unwrap_err()
      .to_string()
      .contains("count")
  );
  // Original N fits the IID cap, but G*max(T_i)*B*K² must fail before fitting.
  p = problem("unbalanced");
  p.sample.as_mut().unwrap().cluster_groups = Some(
    (0..20)
      .map(|i| {
        if i < 10 {
          "large".into()
        } else {
          format!("u{i}")
        }
      })
      .collect(),
  );
  p.predictor_names = (0..20).map(|i| format!("x{i}")).collect();
  p.design_matrix = vec![vec![1.; 20]; 20];
  assert!(
    bootstrap_ols_clustered(&p, 5000, 1)
      .unwrap_err()
      .to_string()
      .contains("largest cluster")
  );
  let mut p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1., 2., 3., 8.],
    vec![vec![0.], vec![0.], vec![0.], vec![1.]],
  )
  .unwrap();
  p.sample = Some(
    EstimationSample::new(
      (0..4).collect(),
      4,
      0,
      None,
      Some(vec!["a".into(), "a".into(), "b".into(), "b".into()]),
    )
    .unwrap(),
  );
  let error = bootstrap_ols_clustered(&p, 100, 1).unwrap_err().to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
}
