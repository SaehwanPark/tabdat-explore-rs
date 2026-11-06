use tabdat_stats::{EstimationProblem, EstimationSample, bootstrap_ols, bootstrap_ols_clustered};
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
fn boundary_problem(largest: usize) -> EstimationProblem {
  let mut x = Vec::new();
  let mut y = Vec::new();
  let mut keys = Vec::new();
  for group in 0..60 {
    let size = if group == 0 { largest } else { 1 };
    for row in 0..size {
      let g = group as f64 + 1.0;
      x.push((1..=6).map(|j| (g * j as f64 * 0.139).sin()).collect());
      y.push((g * 0.71).cos() + row as f64 * 0.0001);
      keys.push(group.to_string());
    }
  }
  let n = y.len();
  EstimationProblem::new("y", (1..=6).map(|i| format!("x{i}")).collect(), y, x)
    .unwrap()
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap())
}

#[test]
fn separate_cluster_and_iid_work_caps_are_enforced_before_resampling() {
  // 60*136*500*7² = 199,920,000: immediately below the cluster cap,
  // and above the unchanged 100-million IID limit.
  let below = boundary_problem(136);
  let result = bootstrap_ols_clustered(&below, 500, 10001).unwrap();
  assert_eq!(result.bootstrap.replications, 500);
  assert_eq!(result.cluster_count, 60);
  assert_eq!(result.replicate_observation_counts.len(), 500);
  // 60*137*500*7² = 201,390,000: reject before the original fit.
  let above = boundary_problem(137);
  let err = bootstrap_ols_clustered(&above, 500, 10001)
    .unwrap_err()
    .to_string();
  assert!(err.contains("must be at most 200000000"), "{err}");
  let n = 4082;
  let iid = EstimationProblem::new(
    "y",
    (1..=6).map(|i| format!("x{i}")).collect(),
    vec![1.0; n],
    vec![vec![1.0; 6]; n],
  )
  .unwrap();
  let err = bootstrap_ols(&iid, 500, 10001).unwrap_err().to_string();
  assert!(err.contains("must be at most 100000000"), "{err}");
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
