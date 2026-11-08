use std::collections::HashMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, bootstrap_normal_poisson_clustered, fit_normal_poisson,
};

fn problem() -> EstimationProblem {
  let tokens: Vec<_> = include_str!("../../../tests/fixtures/mma23_gamma_re_bootstrap/source.asc")
    .split_whitespace()
    .collect();
  let mut x = vec![];
  let mut y = vec![];
  let mut keys = vec![];
  let (records, remainder) = tokens.as_chunks::<25>();
  assert!(remainder.is_empty());
  for (i, record) in records.iter().enumerate() {
    for t in 0..5 {
      x.push(vec![record[10 + t].parse().unwrap()]);
      y.push(record[20 + t].parse().unwrap());
      keys.push((i + 1).to_string());
    }
  }
  let n = y.len();
  EstimationProblem::new("pat", vec!["logr".into()], y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap())
}
fn near(a: f64, b: f64) {
  assert!((a - b).abs() <= 1e-9 + 1e-10 * b.abs(), "{a} vs {b}");
}

#[test]
fn rank_losing_first_copy_aborts_without_redraw_or_input_mutation() {
  let mut p = problem();
  p.predictor_names.push("rare".into());
  let keys = p.sample.as_ref().unwrap().cluster_groups.as_ref().unwrap();
  for (row, key) in p.design_matrix.iter_mut().zip(keys) {
    row.push(if key == "2" { 1. } else { 0. });
  }
  fit_normal_poisson(&p, "id", &Default::default()).unwrap();
  let before = p.clone();
  let error = bootstrap_normal_poisson_clustered(&p, "id", 2, 0)
    .unwrap_err()
    .to_string();
  assert!(error.contains("replicate 1 failed"), "{error}");
  assert_eq!(p, before);
}
fn enlarged_zero_group(additions: usize) -> EstimationProblem {
  let mut p = problem();
  for i in 0..additions {
    let row = i % 5;
    p.response.push(p.response[row]);
    p.design_matrix.push(p.design_matrix[row].clone());
    let s = p.sample.as_mut().unwrap();
    s.retained_indices.push(s.total_observations);
    s.total_observations += 1;
    s.cluster_groups.as_mut().unwrap().push("1".into());
  }
  p
}
#[test]
fn conservative_size_and_actual384_node_work_guards() {
  for (additions, reps, expected) in [
    (1000, 2, "G * largest group size must be at most 40000"),
    (150, 2, "joint K^2 must be at most 20000000"),
    (
      100,
      500,
      "replications * joint K^2 must be at most 3000000000",
    ),
  ] {
    let p = enlarged_zero_group(additions);
    let before = p.clone();
    let error = bootstrap_normal_poisson_clustered(&p, "id", reps, 10001)
      .unwrap_err()
      .to_string();
    assert!(error.contains(expected), "{error}");
    assert_eq!(p, before);
  }
}
fn groups_problem(g: usize) -> EstimationProblem {
  let source = problem();
  let mut x = vec![];
  let mut y = vec![];
  let mut keys = vec![];
  for group in 0..g {
    for t in 0..5 {
      let index = (group % 40) * 5 + t;
      x.push(source.design_matrix[index].clone());
      y.push(source.response[index]);
      keys.push(group.to_string());
    }
  }
  let n = y.len();
  EstimationProblem::new("pat", vec!["logr".into()], y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap())
}
#[test]
fn accepts512_groups_for_bounded_replay_and_rejects513() {
  let r = bootstrap_normal_poisson_clustered(&groups_problem(512), "id", 2, 10001).unwrap();
  assert_eq!(r.cluster_labels.len(), 512);
  assert_eq!(r.replicate_observation_counts, vec![2560; 2]);
  let error = bootstrap_normal_poisson_clustered(&groups_problem(513), "id", 2, 10001)
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("2 to 512 retained groups are required"),
    "{error}"
  );
}

#[test]
fn all500_control_joint_refits_and_empirical_inference_match_independent_r() {
  let p = problem();
  let before = p.clone();
  let report = bootstrap_normal_poisson_clustered(&p, "id", 500, 10001).unwrap();
  assert_eq!(p, before);
  assert_eq!(
    report.cluster_labels,
    (1..=40).map(|v| v.to_string()).collect::<Vec<_>>()
  );
  assert_eq!(report.cluster_sizes, vec![5; 40]);
  assert_eq!(report.zero_total_group_count, 8);
  assert_eq!(report.singleton_group_count, 0);
  assert_eq!(report.original_quadrature_nodes, [80, 96]);
  assert_eq!(report.refit_quadrature_nodes, [176, 208]);
  assert_eq!(
    report.refit_initializer,
    "original_joint_estimate_raw_to_copy_rms"
  );
  assert_eq!(
    report.covariance_parameter_names,
    ["_cons", "logr", "lnsigma2"]
  );
  assert_eq!(report.replicate_observation_counts, vec![200; 500]);
  assert_eq!(report.replicate_cluster_indices.len(), 500);
  for (rep, line) in
    include_str!("../../../tests/fixtures/mma23_native_normal_re_bootstrap/fits.csv")
      .lines()
      .skip(1)
      .enumerate()
  {
    let fields: Vec<_> = line.split(',').collect();
    assert_eq!(fields[0].parse::<usize>().unwrap(), rep + 1);
    for j in 0..3 {
      near(
        report.replicate_coefficients[rep][j],
        fields[j + 1].parse().unwrap(),
      );
    }
  }
  let reference: HashMap<_, f64> =
    include_str!("../../../tests/fixtures/mma23_native_normal_re_bootstrap/reference.csv")
      .lines()
      .skip(1)
      .map(|line| {
        let (k, v) = line.split_once(',').unwrap();
        (k, v.parse().unwrap())
      })
      .collect();
  assert_eq!(reference.len(), 31);
  near(report.observation_count as f64, reference["candidate_rows"]);
  near(
    report.cluster_labels.len() as f64,
    reference["firms_per_draw"],
  );
  near(report.replications as f64, reference["replications"]);
  for (j, name) in ["intercept", "slope", "lnvariance"].iter().enumerate() {
    let c = &report.coefficients[j];
    for (prefix, a) in [
      ("estimate", c.estimate),
      ("mean", c.bootstrap_mean),
      ("bias", c.bias),
      ("se", c.standard_error),
      ("lower", c.ci_lower),
      ("upper", c.ci_upper),
    ] {
      near(a, reference[format!("{prefix}_{name}").as_str()]);
    }
  }
  for (i, j, key) in [
    (0, 0, "aa"),
    (0, 1, "ab"),
    (0, 2, "at"),
    (1, 1, "bb"),
    (1, 2, "bt"),
    (2, 2, "tt"),
  ] {
    near(
      report.covariance_matrix[i][j],
      reference[format!("covariance_{key}").as_str()],
    );
    assert_eq!(
      report.covariance_matrix[i][j],
      report.covariance_matrix[j][i]
    );
  }
  let original = fit_normal_poisson(&p, "id", &Default::default()).unwrap();
  let means = &original.fitted_means;
  near(
    means.iter().sum::<f64>() / 200.,
    reference["original_mu_mean"],
  );
  near(
    means.iter().copied().fold(f64::INFINITY, f64::min),
    reference["original_mu_min"],
  );
  near(
    means.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    reference["original_mu_max"],
  );
  near(
    p.response
      .iter()
      .zip(means)
      .map(|(y, m)| y - m)
      .sum::<f64>()
      / 200.,
    reference["original_residual_mean"],
  );
}

#[test]
fn deterministic_small_replay_and_option_failures_preserve_problem() {
  let p = problem();
  let before = p.clone();
  let a = bootstrap_normal_poisson_clustered(&p, "id", 2, 10001).unwrap();
  assert_eq!(
    a,
    bootstrap_normal_poisson_clustered(&p, "id", 2, 10001).unwrap()
  );
  assert_eq!(a.cluster_labels.len(), 40);
  for reps in [0, 1, 501] {
    assert!(
      bootstrap_normal_poisson_clustered(&p, "id", reps, 10001)
        .unwrap_err()
        .to_string()
        .contains("replications must be from 2 to 500")
    );
  }
  assert_eq!(p, before);
}
