use tabdat_stats::{EstimationProblem, EstimationSample, bootstrap_poisson_clustered};

const INPUT: &str =
  include_str!("../../../tests/fixtures/mma24_individual_pooled_bootstrap/input.csv");
const FITS: &str =
  include_str!("../../../tests/fixtures/mma24_individual_pooled_bootstrap/fits.csv");
const SUMMARY: &str =
  include_str!("../../../tests/fixtures/mma24_individual_pooled_bootstrap/summary.csv");
const DRAWS: &str =
  include_str!("../../../tests/fixtures/mma24_individual_pooled_bootstrap/draw_indices.csv");
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

fn synthetic() -> EstimationProblem {
  let mut x = Vec::new();
  let mut y = Vec::new();
  let mut groups = Vec::new();
  let mut retained = Vec::new();
  for (i, line) in INPUT.lines().skip(1).enumerate() {
    let f = line.trim().split(',').collect::<Vec<_>>();
    if f[1].is_empty() {
      continue;
    }
    retained.push(i);
    y.push(f[1].parse::<f64>().unwrap());
    x.push(
      [2, 11, 4, 3, 5, 9, 10, 8, 7, 6]
        .map(|j| f[j].parse::<f64>().unwrap())
        .to_vec(),
    );
    groups.push(f[12].to_owned());
  }
  let sample = EstimationSample::new(retained, 241, 1, None, Some(groups)).unwrap();
  EstimationProblem::new("pharvis", NAMES.map(str::to_owned).to_vec(), y, x)
    .unwrap()
    .with_sample(sample)
}
fn near(a: f64, b: f64, label: &str) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9,
    "{label}: {a} != {b}"
  );
}

#[test]
fn every_copied_fit_draw_and_full_covariance_match_independent_references() {
  let problem = synthetic();
  let snapshot = problem.clone();
  let report = bootstrap_poisson_clustered(&problem, 500, 10001).unwrap();
  assert_eq!(problem, snapshot);
  assert_eq!(report.observation_count, 240);
  assert_eq!(report.cluster_sizes.len(), 21);
  assert_eq!(report.cluster_sizes.last(), Some(&1));
  assert_eq!(report.replicate_coefficients.len(), 500);
  assert_eq!(report.replicate_cluster_indices.len(), 500);
  assert_eq!(report.covariance_parameter_names.len(), 11);
  assert_eq!(report.rng, "xoshiro256++/splitmix64/u64-rejection-v1");
  let fits = FITS.lines().skip(1).collect::<Vec<_>>();
  let draws = DRAWS.lines().skip(1).collect::<Vec<_>>();
  assert_eq!(fits.len(), 500);
  assert_eq!(draws.len(), 500);
  for i in 0..500 {
    let values = fits[i]
      .split(',')
      .map(|s| s.trim().parse::<f64>().unwrap())
      .collect::<Vec<_>>();
    let expected = draws[i]
      .split(',')
      .map(|s| s.parse::<usize>().unwrap())
      .collect::<Vec<_>>();
    assert_eq!(values[0] as usize, i + 1);
    assert_eq!(expected[0], i + 1);
    assert_eq!(report.replicate_cluster_indices[i], expected[1..]);
    assert_eq!(report.replicate_observation_counts[i], values[1] as usize);
    assert_eq!(
      report.replicate_observation_counts[i],
      expected[1..]
        .iter()
        .map(|&g| report.cluster_sizes[g])
        .sum::<usize>()
    );
    for j in 0..11 {
      near(
        report.replicate_coefficients[i][j],
        values[j + 2],
        "replicate coefficient",
      );
    }
  }
  assert_ne!(
    report.replicate_observation_counts.iter().min(),
    report.replicate_observation_counts.iter().max()
  );
  let summary = SUMMARY.lines().skip(1).collect::<Vec<_>>();
  assert_eq!(summary.len(), 121);
  for (row, line) in summary.iter().enumerate() {
    let v = line
      .split(',')
      .map(|s| s.parse::<f64>().unwrap())
      .collect::<Vec<_>>();
    let (i, j) = (row / 11, row % 11);
    assert_eq!((v[0] as usize, v[1] as usize), (i + 1, j + 1));
    let c = &report.coefficients[i];
    for (a, b) in [
      (c.estimate, v[2]),
      (c.bootstrap_mean, v[3]),
      (c.bias, v[4]),
      (c.standard_error, v[5]),
      (c.ci_lower, v[6]),
      (c.ci_upper, v[7]),
      (report.covariance_matrix[i][j], v[8]),
    ] {
      near(a, b, "summary");
    }
  }
}

fn small_problem(zero_group: bool) -> EstimationProblem {
  let mut x = Vec::new();
  let mut y = Vec::new();
  let mut groups = Vec::new();
  for g in 0..2 {
    for i in 0..4 {
      x.push(vec![if zero_group { i as f64 } else { g as f64 }]);
      y.push(if zero_group && g == 1 {
        0.0
      } else {
        1.0 + (i % 2) as f64 + g as f64
      });
      groups.push(g.to_string());
    }
  }
  let sample = EstimationSample::new((0..8).collect(), 8, 0, None, Some(groups)).unwrap();
  EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_sample(sample)
}

#[test]
fn failed_singular_and_zero_count_copies_abort_without_modifying_problem() {
  for zero_group in [false, true] {
    let p = small_problem(zero_group);
    let before = p.clone();
    let error = bootstrap_poisson_clustered(&p, 2, 0)
      .unwrap_err()
      .to_string();
    assert!(error.contains("replicate 1 failed"), "{error}");
    assert_eq!(p, before);
  }
}

fn bounded_problem(sizes: &[usize], predictors: usize) -> EstimationProblem {
  let groups = sizes
    .iter()
    .enumerate()
    .flat_map(|(g, &n)| std::iter::repeat_n(g.to_string(), n))
    .collect::<Vec<_>>();
  let n = groups.len();
  let sample = EstimationSample::new((0..n).collect(), n, 0, None, Some(groups)).unwrap();
  EstimationProblem::new(
    "y",
    (0..predictors).map(|i| format!("x{i}")).collect(),
    vec![1.0; n],
    vec![vec![1.0; predictors]; n],
  )
  .unwrap()
  .with_sample(sample)
}

#[test]
fn admission_checks_reject_missing_groups_weights_and_bounds_before_fitting() {
  let p = small_problem(false);
  assert!(
    bootstrap_poisson_clustered(&p, 1, 0)
      .unwrap_err()
      .to_string()
      .contains("replications")
  );
  assert!(
    bootstrap_poisson_clustered(&p, 501, 0)
      .unwrap_err()
      .to_string()
      .contains("replications")
  );
  assert!(
    bootstrap_poisson_clustered(&p.clone().with_intercept(false), 2, 0)
      .unwrap_err()
      .to_string()
      .contains("intercept")
  );
  assert!(
    bootstrap_poisson_clustered(&p.clone().with_weights(vec![1.0; 8]), 2, 0)
      .unwrap_err()
      .to_string()
      .contains("unweighted")
  );
  let mut no_groups = p.clone();
  no_groups.sample.as_mut().unwrap().cluster_groups = None;
  assert!(
    bootstrap_poisson_clustered(&no_groups, 2, 0)
      .unwrap_err()
      .to_string()
      .contains("cluster identifiers")
  );
  let mut one = p.clone();
  one.sample.as_mut().unwrap().cluster_groups = Some(vec!["one".into(); 8]);
  assert!(
    bootstrap_poisson_clustered(&one, 2, 0)
      .unwrap_err()
      .to_string()
      .contains("2 to 256")
  );
  let many_groups = bounded_problem(&vec![1; 257], 1);
  assert!(
    bootstrap_poisson_clustered(&many_groups, 2, 0)
      .unwrap_err()
      .to_string()
      .contains("2 to 256")
  );
  let oversized = bounded_problem(&[20_001, 1], 1);
  assert!(
    bootstrap_poisson_clustered(&oversized, 2, 0)
      .unwrap_err()
      .to_string()
      .contains("at most 40000")
  );
  let work = bounded_problem(&[20_000, 20_000], 15);
  assert!(
    bootstrap_poisson_clustered(&work, 500, 0)
      .unwrap_err()
      .to_string()
      .contains("at most 2500000000")
  );
  let too_many_parameters = bounded_problem(&[10, 10], 16);
  assert!(
    bootstrap_poisson_clustered(&too_many_parameters, 2, 0)
      .unwrap_err()
      .to_string()
      .contains("at most 16 coefficients")
  );
}
