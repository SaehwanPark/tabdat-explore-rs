use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_poisson_with_offset,
  poisson_nb1_overdispersion_score_test, poisson_nb2_overdispersion_score_test,
};

fn data() -> Vec<Vec<String>> {
  include_str!("../../../examples/nb2-overdispersion-score/score_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').map(str::to_owned).collect())
    .collect()
}

fn problem(
  rows: &[Vec<String>],
  outcome_index: usize,
  clustered: bool,
) -> (EstimationProblem, Vec<f64>) {
  let n = rows.len();
  let response = rows
    .iter()
    .map(|row| row[outcome_index].parse().unwrap())
    .collect();
  let design = rows
    .iter()
    .map(|row| vec![row[3].parse().unwrap()])
    .collect();
  let offsets = rows.iter().map(|row| row[4].parse().unwrap()).collect();
  let groups = clustered.then(|| {
    rows
      .iter()
      .map(|row| format!("g{}", row[5].parse::<usize>().unwrap()))
      .collect()
  });
  let sample = EstimationSample::new((0..n).collect(), n, 0, None, groups).unwrap();
  (
    EstimationProblem::new("y", vec!["x".into()], response, design)
      .unwrap()
      .with_sample(sample),
    offsets,
  )
}

fn close(actual: f64, expected: &str, tail: bool) {
  let expected: f64 = expected.parse().unwrap();
  let actual_tolerance = if tail {
    expected.abs() * 2e-8 + f64::MIN_POSITIVE
  } else {
    1e-8
  };
  close_with_tolerance(actual, expected, actual_tolerance);
}

fn close_with_tolerance(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= tolerance,
    "{actual:.17e} != {expected:.17e}; tolerance={tolerance:.3e}"
  );
}

#[test]
fn model_row_robust_and_cluster_score_variances_match_independent_base_r() {
  let rows = data();
  assert_eq!(rows.len(), 180);
  let reference: Vec<Vec<_>> = include_str!("fixtures/poisson_nb2_score_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect();
  assert_eq!(reference.len(), 3);

  // R and Rust use independently converged Poisson coefficients. Compare the
  // resulting score/Z within 1e-8 absolute, score variance within 2e-7 absolute,
  // and the one-sided tail within 2e-8 relative tolerance.
  for row in reference {
    let (outcome_index, clustered) = match row[1] {
      "model" | "robust" => (1, false),
      "cluster" => (2, true),
      other => panic!("unexpected covariance {other}"),
    };
    let (problem, offsets) = problem(&rows, outcome_index, clustered);
    let fit = fit_poisson_with_offset(
      &problem,
      &offsets,
      Some("log_exposure"),
      row[1] == "robust",
      clustered.then_some("group"),
    )
    .unwrap();
    let result = poisson_nb2_overdispersion_score_test(&problem, &fit).unwrap();

    assert_eq!(result.observation_count.to_string(), row[2]);
    close(result.covariance_correction, row[4], false);
    assert_eq!(result.offset_variable.as_deref(), Some("log_exposure"));
    assert_eq!(
      result
        .cluster
        .as_ref()
        .map(|cluster| cluster.group_count.to_string()),
      (row[3] != "NA").then(|| row[3].to_owned())
    );
    close(result.score, row[5], false);
    close_with_tolerance(result.score_variance, row[6].parse().unwrap(), 2e-7);
    close(result.statistic, row[7], false);
    close(result.p_value, row[8], true);
  }
}

#[test]
fn nb1_direction_score_and_covariance_match_independent_base_r() {
  let rows = data();
  let reference: Vec<Vec<_>> = include_str!("fixtures/poisson_nb1_score_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect();
  assert_eq!(reference.len(), 3);

  for row in reference {
    let (outcome_index, clustered) = match row[1] {
      "model" | "robust" => (1, false),
      "cluster" => (2, true),
      other => panic!("unexpected covariance {other}"),
    };
    let (problem, offsets) = problem(&rows, outcome_index, clustered);
    let fit = fit_poisson_with_offset(
      &problem,
      &offsets,
      Some("log_exposure"),
      row[1] == "robust",
      clustered.then_some("group"),
    )
    .unwrap();
    let result = poisson_nb1_overdispersion_score_test(&problem, &fit).unwrap();

    assert_eq!(result.method, "one_sided_nb1_variance_score");
    assert_eq!(result.observation_count.to_string(), row[2]);
    assert_eq!(result.reference_distribution, "asymptotic_standard_normal");
    assert_eq!(
      result.covariance,
      match row[1] {
        "model" => "poisson_information_nb1_score",
        "robust" => "robust_row_nb1_score_n_over_n_minus_1",
        "cluster" => "cluster_nb1_score_g_over_g_minus_1",
        _ => unreachable!(),
      }
    );
    close(result.covariance_correction, row[4], false);
    assert_eq!(result.offset_variable.as_deref(), Some("log_exposure"));
    assert_eq!(
      result
        .cluster
        .as_ref()
        .map(|cluster| cluster.group_count.to_string()),
      (row[3] != "NA").then(|| row[3].to_owned())
    );
    close(result.score, row[5], false);
    close_with_tolerance(result.score_variance, row[6].parse().unwrap(), 2e-7);
    close(result.statistic, row[7], false);
    close(result.p_value, row[8], true);
  }
}

#[test]
fn nb1_direction_requires_positive_means() {
  let problem = EstimationProblem::new("y", vec![], vec![0.0, 1.0], vec![vec![], vec![]]).unwrap();
  let mut fit = fit_poisson_with_offset(&problem, &[0.0, 0.0], None, false, None).unwrap();
  fit.fitted_means[0] = 0.0;
  assert!(poisson_nb1_overdispersion_score_test(&problem, &fit).is_err());
}

#[test]
fn rejects_misaligned_fit_weighted_input_and_zero_robust_score_variance() {
  let rows = data();
  let (fit_problem, offsets) = problem(&rows, 1, false);
  let fit =
    fit_poisson_with_offset(&fit_problem, &offsets, Some("log_exposure"), false, None).unwrap();

  let mut misaligned = fit.clone();
  misaligned.fitted_means.pop();
  assert!(poisson_nb2_overdispersion_score_test(&fit_problem, &misaligned).is_err());

  let mut changed_sample = fit_problem.clone();
  changed_sample.sample.as_mut().unwrap().retained_indices[0] = 1;
  assert!(poisson_nb2_overdispersion_score_test(&changed_sample, &fit).is_err());

  let mut weighted = fit_problem.clone();
  weighted.weights = Some(vec![1.0; weighted.response.len()]);
  assert!(poisson_nb2_overdispersion_score_test(&weighted, &fit).is_err());
  let mut sample_weighted = fit_problem.clone();
  sample_weighted.sample.as_mut().unwrap().weights =
    Some(vec![1.0; sample_weighted.response.len()]);
  assert!(poisson_nb2_overdispersion_score_test(&sample_weighted, &fit).is_err());

  let n = 8;
  let constant = EstimationProblem::new("y", vec![], vec![1.0; n], vec![vec![]; n]).unwrap();
  let mut robust_fit = fit_poisson_with_offset(&constant, &vec![0.0; n], None, true, None).unwrap();
  // A finite but degenerate retained score vector must not divide by zero.
  robust_fit.fitted_means.fill(2.0);
  assert!(poisson_nb2_overdispersion_score_test(&constant, &robust_fit).is_err());
}

#[test]
fn score_metadata_keeps_one_sided_direction_and_cluster_unit_explicit() {
  let rows = data();
  let (problem, offsets) = problem(&rows, 2, true);
  let fit = fit_poisson_with_offset(
    &problem,
    &offsets,
    Some("log_exposure"),
    false,
    Some("group"),
  )
  .unwrap();
  let result = poisson_nb2_overdispersion_score_test(&problem, &fit).unwrap();
  let wire = serde_json::to_value(&result).unwrap();
  assert_eq!(result.method, "one_sided_nb2_variance_score");
  assert_eq!(result.reference_distribution, "asymptotic_standard_normal");
  assert!(result.statistic < 0.0);
  assert_eq!(
    wire["null_hypothesis"],
    "alpha = 0 (Poisson conditional variance)"
  );
  assert_eq!(
    wire["alternative_hypothesis"],
    "alpha > 0 in Var(Y|X) = mu + alpha*mu^2"
  );
  assert_eq!(wire["cluster"]["variable"], "group");
  assert_eq!(wire["cluster"]["group_count"], 30);
  assert_eq!(result.covariance, "cluster_nb2_score_g_over_g_minus_1");
}
