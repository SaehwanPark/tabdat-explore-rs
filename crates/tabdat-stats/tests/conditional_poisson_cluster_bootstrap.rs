use tabdat_stats::{EstimationProblem, EstimationSample, bootstrap_conditional_poisson_clustered};

fn sample(groups: usize, singleton: bool) -> EstimationProblem {
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut keys = Vec::new();
  for g in 0..groups {
    y.extend([1.0, (2 + g % 3) as f64]);
    x.extend([vec![0.0], vec![1.0]]);
    keys.extend([format!("g{g}"), format!("g{g}")]);
  }
  if singleton {
    y.push(3.0);
    x.push(vec![7.0]);
    keys.push("singleton".to_string());
  }
  y.extend([0.0, 0.0]);
  x.extend([vec![0.0], vec![1.0]]);
  keys.extend(["zero".to_string(), "zero".to_string()]);
  let n = y.len();
  EstimationProblem::new("y", vec!["x".to_string()], y, x)
    .unwrap()
    .with_intercept(false)
    .with_sample(EstimationSample::new((0..n).collect(), n + 1, 1, None, Some(keys)).unwrap())
}

#[test]
fn every_refit_matches_closed_form_and_zero_groups_are_excluded() {
  let p = sample(20, true);
  let report = bootstrap_conditional_poisson_clustered(&p, "id", 100, 10001).unwrap();
  assert_eq!(report.observation_count, 41);
  assert_eq!(report.joint_finite_observation_count, 43);
  assert_eq!(
    (
      report.candidate_group_count,
      report.zero_total_group_count,
      report.zero_total_observation_count
    ),
    (22, 1, 2)
  );
  assert_eq!(report.sample.dropped_observations, 3);
  assert_eq!(report.cluster_labels.len(), 21);
  assert_eq!(report.cluster_labels.last().unwrap(), "singleton");
  assert!(!report.cluster_labels.iter().any(|v| v == "zero"));
  for (r, draws) in report.replicate_cluster_indices.iter().enumerate() {
    let positive: Vec<_> = draws.iter().copied().filter(|&i| i < 20).collect();
    let sum: usize = positive.iter().map(|i| 2 + i % 3).sum();
    let expected = (sum as f64 / positive.len() as f64).ln();
    assert!((report.replicate_coefficients[r][0] - expected).abs() < 1e-9);
    assert_eq!(
      report.replicate_observation_counts[r],
      positive.len() * 2 + draws.len() - positive.len()
    );
  }
  let again = bootstrap_conditional_poisson_clustered(&p, "id", 100, 10001).unwrap();
  assert_eq!(report, again);
}

#[test]
fn rejects_group_and_conservative_work_caps() {
  let too_many = sample(257, false);
  let err = bootstrap_conditional_poisson_clustered(&too_many, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(err.contains("2 to 256 retained"), "{err}");

  let mut too_large = sample(32, false);
  for i in 0..1250 {
    too_large.response.push(1.0);
    too_large.design_matrix.push(vec![(i % 2) as f64]);
    let sample = too_large.sample.as_mut().unwrap();
    sample.retained_indices.push(sample.total_observations);
    sample.total_observations += 1;
    sample.cluster_groups.as_mut().unwrap().push("g0".into());
  }
  let err = bootstrap_conditional_poisson_clustered(&too_large, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(
    err.contains("G * largest group size must be at most 40000"),
    "{err}"
  );

  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut keys = Vec::new();
  for group in 0..50 {
    for row in 0..if group == 0 { 500 } else { 2 } {
      y.push(1.0);
      x.push(
        (0..16)
          .map(|j| (((row + 1) * (j + 1) * (group + 1)) as f64 * 0.011).sin())
          .collect::<Vec<_>>(),
      );
      keys.push(group.to_string());
    }
  }
  let n = y.len();
  let p = EstimationProblem::new("y", (0..16).map(|j| format!("x{j}")).collect(), y, x)
    .unwrap()
    .with_intercept(false)
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap());
  let err = bootstrap_conditional_poisson_clustered(&p, "id", 500, 0)
    .unwrap_err()
    .to_string();
  assert!(err.contains("2500000000"), "{err}");
}

#[test]
fn a_singular_copy_aborts_and_invalid_contracts_fail() {
  let p = sample(1, true);
  let error = bootstrap_conditional_poisson_clustered(&p, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(error.contains("replicate 1 failed"), "{error}");
  assert!(bootstrap_conditional_poisson_clustered(&p, "id", 1, 0).is_err());
  assert!(bootstrap_conditional_poisson_clustered(&p, "id", 501, 0).is_err());
  let mut intercept = p.clone();
  intercept.include_intercept = true;
  assert!(bootstrap_conditional_poisson_clustered(&intercept, "id", 2, 0).is_err());
  let mut fractional = p.clone();
  fractional.response[0] = 0.5;
  assert!(bootstrap_conditional_poisson_clustered(&fractional, "id", 2, 0).is_err());
}
