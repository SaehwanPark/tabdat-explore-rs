use tabdat_stats::{
  EstimationProblem, EstimationSample, StatsError, fit_nb2_with_offset, fit_poisson_with_offset,
  nb2_wald_test, poisson_wald_test,
};

fn problem(clustered: bool) -> (EstimationProblem, Vec<f64>) {
  let rows = include_str!("../../../examples/poisson-offset/poisson_offset.csv")
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let response = rows.iter().map(|row| row[1]).collect::<Vec<_>>();
  let design = rows
    .iter()
    .map(|row| vec![row[2], row[3]])
    .collect::<Vec<_>>();
  let offsets = rows.iter().map(|row| row[4]).collect::<Vec<_>>();
  let cluster_groups = clustered.then(|| rows.iter().map(|row| row[5].to_string()).collect());
  let sample = EstimationSample::new(
    (0..rows.len()).collect(),
    rows.len(),
    0,
    None,
    cluster_groups,
  )
  .unwrap();
  let problem = EstimationProblem::new("y", vec!["x".into(), "z".into()], response, design)
    .unwrap()
    .with_intercept(true)
    .with_sample(sample);
  (problem, offsets)
}

fn restrictions(name: &str) -> (Vec<Vec<f64>>, Vec<f64>) {
  match name {
    "one" => (vec![vec![0.0, 1.0, 0.0]], vec![0.0]),
    "contrast" => (vec![vec![0.0, 1.0, -1.0]], vec![0.25]),
    "joint" => (
      vec![vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]],
      vec![0.0, 0.0],
    ),
    _ => unreachable!(),
  }
}

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-8,
    "{actual:.16e} != {expected:.16e}"
  );
}

#[test]
fn poisson_and_nb2_general_wald_tests_match_independent_r() {
  let reference = include_str!("fixtures/count_wald_reference.csv");
  for line in reference.lines().skip(1) {
    let fields = line.split(',').collect::<Vec<_>>();
    let family = fields[0];
    let covariance = fields[1];
    let restriction = fields[2];
    let clustered = covariance == "cluster";
    let robust = covariance == "robust";
    let (matrix, vector) = restrictions(restriction);
    let (problem, offsets) = problem(clustered);
    let test = if family == "poisson" {
      let fit = fit_poisson_with_offset(
        &problem,
        &offsets,
        Some("log_time"),
        robust,
        clustered.then_some("person"),
      )
      .unwrap();
      poisson_wald_test(&fit, &matrix, &vector).unwrap()
    } else {
      let fit = fit_nb2_with_offset(
        &problem,
        &offsets,
        Some("log_time"),
        robust,
        clustered.then_some("person"),
      )
      .unwrap();
      nb2_wald_test(&fit, &matrix, &vector).unwrap()
    };
    assert_eq!(test.degrees_of_freedom, fields[3].parse::<usize>().unwrap());
    close(test.statistic, fields[4].parse().unwrap());
    close(test.p_value, fields[5].parse().unwrap());
  }
  assert_eq!(reference.lines().count(), 19);
}

#[test]
fn nb2_wald_uses_mean_block_and_rejects_invalid_shapes_or_names() {
  let (problem, offsets) = problem(false);
  let mut fit = fit_nb2_with_offset(&problem, &offsets, Some("log_time"), false, None).unwrap();
  let (matrix, vector) = restrictions("one");
  let expected = nb2_wald_test(&fit, &matrix, &vector).unwrap();
  assert_eq!(fit.covariance.len(), fit.coefficients.len() + 1);

  fit.covariance.pop();
  assert!(matches!(
    nb2_wald_test(&fit, &matrix, &vector),
    Err(StatsError::DimensionMismatch(_))
  ));

  let mut fit = fit_nb2_with_offset(&problem, &offsets, Some("log_time"), false, None).unwrap();
  fit.coefficients.swap(0, 1);
  assert!(matches!(
    nb2_wald_test(&fit, &matrix, &vector),
    Err(StatsError::IncompatibleHypothesis(_))
  ));
  assert!(expected.statistic.is_finite());
}
