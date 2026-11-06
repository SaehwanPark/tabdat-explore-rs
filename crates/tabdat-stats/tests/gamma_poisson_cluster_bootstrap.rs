use std::collections::HashMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, bootstrap_gamma_poisson_clustered, fit_gamma_poisson,
};

const INPUT: &str = include_str!("../../../tests/fixtures/mma24_individual_gamma_re/input.csv");
const NAMES: [&str; 10] = [
  "lnhhexp",
  "insurance",
  "sex",
  "age",
  "married",
  "illdays",
  "actdays",
  "injury",
  "illness",
  "educ",
];
fn problem() -> EstimationProblem {
  let mut lines = INPUT.lines();
  let names: Vec<_> = lines.next().unwrap().split(',').collect();
  let idx = |name: &str| names.iter().position(|v| *v == name).unwrap();
  let mut y = vec![];
  let mut x = vec![];
  let mut keys = vec![];
  let mut kept = vec![];
  let mut total = 0;
  for (i, line) in lines.enumerate() {
    total += 1;
    let fields: Vec<_> = line.split(',').collect();
    if fields[idx("pharvis")].is_empty() {
      continue;
    }
    y.push(fields[idx("pharvis")].parse().unwrap());
    x.push(
      NAMES
        .iter()
        .map(|name| fields[idx(name)].parse().unwrap())
        .collect(),
    );
    keys.push(fields[idx("commune")].to_owned());
    kept.push(i);
  }
  let sample = EstimationSample::new(kept, total, total - y.len(), None, Some(keys)).unwrap();
  EstimationProblem::new("pharvis", NAMES.map(str::to_owned).to_vec(), y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(sample)
}
#[test]
fn copies_are_distinct_gamma_groups_and_all_groups_remain_eligible() {
  let p = problem();
  let report = bootstrap_gamma_poisson_clustered(&p, "commune", 2, 10001).unwrap();
  assert_eq!(report.observation_count, 278);
  assert_eq!(report.cluster_labels.len(), 48);
  assert_eq!(report.zero_total_group_count, 4);
  assert_eq!(report.singleton_group_count, 2);
  assert_eq!(report.sample.dropped_observations, 1);
  assert_eq!(report.covariance_parameter_names.len(), 12);
  assert_eq!(report.covariance_parameter_names[0], "_cons");
  assert_eq!(report.covariance_parameter_names[11], "lnalpha");
  assert_eq!(
    report,
    bootstrap_gamma_poisson_clustered(&p, "commune", 2, 10001).unwrap()
  );
  let mut by_group: HashMap<&str, Vec<usize>> = HashMap::new();
  for (i, key) in p
    .sample
    .as_ref()
    .unwrap()
    .cluster_groups
    .as_ref()
    .unwrap()
    .iter()
    .enumerate()
  {
    by_group.entry(key).or_default().push(i);
  }
  let mut x = vec![];
  let mut y = vec![];
  let mut unique = vec![];
  let mut merged = vec![];
  for (copy, &draw) in report.replicate_cluster_indices[0].iter().enumerate() {
    let original = &report.cluster_labels[draw];
    for &row in &by_group[original.as_str()] {
      x.push(p.design_matrix[row].clone());
      y.push(p.response[row]);
      unique.push(format!("{copy:03}"));
      merged.push(original.clone());
    }
  }
  let n = y.len();
  assert_eq!(report.replicate_observation_counts[0], n);
  let copied = |keys| {
    EstimationProblem::new("pharvis", p.predictor_names.clone(), y.clone(), x.clone())
      .unwrap()
      .with_intercept_name("_cons")
      .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap())
  };
  let distinct = fit_gamma_poisson(&copied(unique), "id", &Default::default()).unwrap();
  let merged_fit = fit_gamma_poisson(&copied(merged), "id", &Default::default()).unwrap();
  for (a, b) in distinct
    .coefficients
    .iter()
    .map(|c| c.value)
    .chain(std::iter::once(distinct.ln_alpha.value))
    .zip(&report.replicate_coefficients[0])
  {
    assert!((a - b).abs() <= 1e-10, "{a} vs {b}");
  }
  assert!(
    (distinct.ln_alpha.value - merged_fit.ln_alpha.value).abs() > 1e-3,
    "merged copies must not silently become one gamma effect"
  );
}
#[test]
fn singular_resample_aborts_instead_of_merging_or_redrawing() {
  let y = vec![0., 0., 10., 12., 1., 2., 8., 10.];
  let x = vec![
    vec![0.],
    vec![0.],
    vec![1.],
    vec![1.],
    vec![0.],
    vec![0.],
    vec![0.],
    vec![0.],
  ];
  let keys = ["a", "a", "b", "b", "c", "c", "d", "d"]
    .map(str::to_owned)
    .to_vec();
  let p = EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(EstimationSample::new((0..8).collect(), 8, 0, None, Some(keys)).unwrap());
  assert!(fit_gamma_poisson(&p, "id", &Default::default()).is_ok());
  let before = p.clone();
  let error = bootstrap_gamma_poisson_clustered(&p, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(error.contains("replicate 1 failed"), "{error}");
  assert_eq!(p, before);
}

#[test]
fn rejects_too_many_retained_groups_before_sampling() {
  let original = problem();
  let keys = original
    .sample
    .as_ref()
    .unwrap()
    .cluster_groups
    .as_ref()
    .unwrap();
  let mut labels: Vec<&str> = Vec::new();
  let mut clusters: Vec<Vec<usize>> = Vec::new();
  for (row, key) in keys.iter().enumerate() {
    let position = labels.iter().position(|&v| v == key).unwrap_or_else(|| {
      labels.push(key);
      clusters.push(vec![]);
      labels.len() - 1
    });
    clusters[position].push(row);
  }
  let mut y = vec![];
  let mut x = vec![];
  let mut out = vec![];
  for group in 0..257 {
    for &row in &clusters[group % clusters.len()] {
      y.push(original.response[row]);
      x.push(original.design_matrix[row].clone());
      out.push(group.to_string());
    }
  }
  let n = y.len();
  let p = EstimationProblem::new("y", original.predictor_names.clone(), y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(out)).unwrap());
  let error = bootstrap_gamma_poisson_clustered(&p, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(error.contains("2 to 256 retained groups"), "{error}");
}

#[test]
fn invalid_original_and_replication_bounds_fail_without_returning_a_report() {
  let p = problem();
  assert!(bootstrap_gamma_poisson_clustered(&p, "id", 1, 0).is_err());
  assert!(bootstrap_gamma_poisson_clustered(&p, "id", 501, 0).is_err());
  let mut no_intercept = p.clone();
  no_intercept.include_intercept = false;
  assert!(bootstrap_gamma_poisson_clustered(&no_intercept, "id", 2, 0).is_err());
  let mut bad = p.clone();
  bad.response[0] = 0.5;
  assert!(bootstrap_gamma_poisson_clustered(&bad, "id", 2, 0).is_err());
  let mut zero = p.clone();
  zero.response.fill(0.0);
  assert!(bootstrap_gamma_poisson_clustered(&zero, "id", 2, 0).is_err());
}
