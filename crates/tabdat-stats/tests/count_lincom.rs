use tabdat_stats::{
  EstimationProblem, EstimationSample, StatsError, fit_nb2_with_offset, fit_poisson_with_offset,
  nb2_linear_combination, poisson_linear_combination,
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

fn combination(name: &str) -> (Vec<f64>, f64) {
  match name {
    "slope" => (vec![0.0, 1.0, 0.0], 0.0),
    "contrast" => (vec![0.0, 1.0, -1.0], 2.0),
    "sum" => (vec![0.0, 1.0, 1.0], -0.25),
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
fn poisson_and_nb2_linear_combinations_match_independent_r() {
  let reference = include_str!("fixtures/count_lincom_reference.csv");
  for line in reference.lines().skip(1) {
    let fields = line.split(',').collect::<Vec<_>>();
    let family = fields[0];
    let covariance = fields[1];
    let combination_name = fields[2];
    let clustered = covariance == "cluster";
    let robust = covariance == "robust";
    let (problem, offsets) = problem(clustered);
    let (weights, offset) = combination(combination_name);
    let inference = if family == "poisson" {
      let fit = fit_poisson_with_offset(
        &problem,
        &offsets,
        Some("log_time"),
        robust,
        clustered.then_some("person"),
      )
      .unwrap();
      poisson_linear_combination(&fit, &weights, offset, 95.0).unwrap()
    } else {
      let fit = fit_nb2_with_offset(
        &problem,
        &offsets,
        Some("log_time"),
        robust,
        clustered.then_some("person"),
      )
      .unwrap();
      nb2_linear_combination(&fit, &weights, offset, 95.0).unwrap()
    };
    for (actual, field) in [
      (inference.estimate, 3),
      (inference.standard_error, 4),
      (inference.statistic.unwrap(), 5),
      (inference.p_value, 6),
      (inference.ci_lower, 7),
      (inference.ci_upper, 8),
      (inference.confidence_level, 9),
    ] {
      close(actual, fields[field].parse().unwrap());
    }
  }
  assert_eq!(reference.lines().count(), 19);
}

#[test]
fn nb2_linear_combination_validates_joint_covariance_and_zero_se_is_undefined() {
  let (problem, offsets) = problem(false);
  let mut fit = fit_nb2_with_offset(&problem, &offsets, Some("log_time"), false, None).unwrap();
  let mut weights = vec![0.0; fit.coefficients.len()];
  weights[1] = 1.0;
  assert!(nb2_linear_combination(&fit, &weights, 0.0, 95.0).is_ok());

  fit.covariance.pop();
  assert!(matches!(
    nb2_linear_combination(&fit, &weights, 0.0, 95.0),
    Err(StatsError::DimensionMismatch(_))
  ));

  let fit = fit_nb2_with_offset(&problem, &offsets, Some("log_time"), false, None).unwrap();
  let zero = nb2_linear_combination(&fit, &vec![0.0; fit.coefficients.len()], 2.0, 95.0).unwrap();
  assert_eq!(zero.estimate, 2.0);
  assert_eq!(zero.standard_error, 0.0);
  assert_eq!(zero.statistic, None);
  assert!(zero.p_value.is_nan());
  assert_eq!(zero.ci_lower, 2.0);
  assert_eq!(zero.ci_upper, 2.0);
}
