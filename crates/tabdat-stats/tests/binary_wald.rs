use tabdat_stats::{
  BinaryMleResult, BinaryModel, EstimationProblem, EstimationSample, StatsError, binary_wald_test,
  fit_binary_clustered, fit_binary_logit, fit_binary_probit, fit_binary_robust,
};

fn problem(clustered: bool) -> EstimationProblem {
  let rows = include_str!("fixtures/binary_wald_data.csv")
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .map(|value| value.trim_matches('"').parse::<f64>().unwrap())
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();
  let groups = clustered.then(|| rows.iter().map(|row| row[3].to_string()).collect());
  EstimationProblem::new(
    "y",
    vec!["x1".into(), "x2".into()],
    rows.iter().map(|row| row[0]).collect(),
    rows.iter().map(|row| vec![row[1], row[2]]).collect(),
  )
  .unwrap()
  .with_intercept(true)
  .with_sample(EstimationSample::new((0..40).collect(), 40, 0, None, groups).unwrap())
}

fn fit(problem: &EstimationProblem, family: &str, covariance: &str) -> BinaryMleResult {
  let model = if family == "logit" {
    BinaryModel::Logit
  } else {
    BinaryModel::Probit
  };
  match covariance {
    "model" => match model {
      BinaryModel::Logit => fit_binary_logit(problem).unwrap(),
      BinaryModel::Probit => fit_binary_probit(problem).unwrap(),
    },
    "robust" => fit_binary_robust(problem, model).unwrap(),
    "cluster" => fit_binary_clustered(problem, model, "group").unwrap(),
    _ => unreachable!(),
  }
}

fn restriction(name: &str) -> (Vec<Vec<f64>>, Vec<f64>) {
  match name {
    "one" => (vec![vec![0.0, 1.0, 1.0]], vec![0.2]),
    "joint" => (
      vec![vec![0.0, 1.0, 1.0], vec![0.0, 1.0, -1.0]],
      vec![0.2, 0.4],
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
fn model_based_robust_and_cluster_wald_tests_match_independent_r() {
  // Base R 4.6.2 glm, converged observed information and explicit score
  // sandwiches: fixtures/binary_wald_reference.R.
  let reference = include_str!("fixtures/binary_wald_reference.csv");
  for line in reference.lines().skip(1) {
    let fields = line.split(',').collect::<Vec<_>>();
    let problem = problem(fields[1] == "cluster");
    let result = fit(&problem, fields[0], fields[1]);
    let (matrix, vector) = restriction(fields[2]);
    let before = result.clone();
    let test = binary_wald_test(&result, &matrix, &vector).unwrap();
    assert_eq!(result, before);
    assert_eq!(test.degrees_of_freedom, fields[3].parse::<usize>().unwrap());
    close(test.statistic, fields[4].parse().unwrap());
    close(test.p_value, fields[5].parse().unwrap());
    assert!((0.0..=1.0).contains(&test.p_value));
  }
}

#[test]
fn binary_wald_test_reuses_the_existing_slope_omnibus_and_rejects_singular_inputs() {
  for (family, covariance) in [
    ("logit", "robust"),
    ("logit", "cluster"),
    ("probit", "robust"),
    ("probit", "cluster"),
  ] {
    let problem = problem(covariance == "cluster");
    let result = fit(&problem, family, covariance);
    let omnibus = result.omnibus_wald.as_ref().unwrap();
    let test = binary_wald_test(
      &result,
      &[vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]],
      &[0.0, 0.0],
    )
    .unwrap();
    close(test.statistic, omnibus.statistic.unwrap());
    close(test.p_value, omnibus.p_value.unwrap());
  }

  let problem = problem(false);
  let result = fit(&problem, "logit", "model");
  assert!(binary_wald_test(&result, &[], &[]).is_err());
  assert!(binary_wald_test(&result, &[vec![0.0, 1.0]], &[0.0]).is_err());
  assert!(binary_wald_test(&result, &[vec![0.0, 1.0, 1.0]], &[f64::NAN]).is_err());
  assert!(matches!(
    binary_wald_test(
      &result,
      &[vec![0.0, 1.0, 1.0], vec![0.0, 2.0, 2.0]],
      &[0.0, 0.0]
    ),
    Err(StatsError::SingularMatrix(_))
  ));
}
