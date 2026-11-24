use tabdat_stats::{
  BinaryModel, EstimationProblem, EstimationSample, binary_average_discrete_change,
  fit_binary_clustered, fit_binary_logit, fit_binary_probit, fit_binary_robust,
};

fn problem(clustered: bool) -> EstimationProblem {
  let rows = include_str!("../../../examples/binary-discrete-change/binary_discrete_change.csv")
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
    vec!["x".into(), "treated".into()],
    rows.iter().map(|row| row[0]).collect(),
    rows.iter().map(|row| vec![row[1], row[2]]).collect(),
  )
  .unwrap()
  .with_intercept(true)
  .with_sample(EstimationSample::new((0..120).collect(), 120, 0, None, groups).unwrap())
}

fn fit(
  problem: &EstimationProblem,
  model: BinaryModel,
  covariance: &str,
) -> tabdat_stats::BinaryMleResult {
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

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 2e-10,
    "{actual:.16e} != {expected:.16e}"
  );
}

fn reference(model: &str, covariance: &str) -> Vec<f64> {
  let row = include_str!("fixtures/binary_discrete_change_reference.csv")
    .lines()
    .skip(1)
    .find(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      fields[0] == model && fields[1] == covariance
    })
    .unwrap();
  row
    .split(',')
    .skip(4)
    .map(|value| value.parse::<f64>().unwrap())
    .collect()
}

#[test]
fn discrete_probability_changes_match_independent_r_for_each_link_and_covariance() {
  // Base R 4.6.2 reference.R uses glm, analytic link information and explicit
  // row/group score sandwiches. The delta method includes every coefficient
  // and covariance cross term; moderate outputs use 2e-10 absolute tolerance.
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    let model_name = match model {
      BinaryModel::Logit => "logit",
      BinaryModel::Probit => "probit",
    };
    for covariance in ["model", "robust", "cluster"] {
      let problem = problem(covariance == "cluster");
      let fitted = fit(&problem, model, covariance);
      let before = fitted.clone();
      let result = binary_average_discrete_change(&problem, &fitted, "treated").unwrap();
      let expected = reference(model_name, covariance);
      assert_eq!(result.model, model);
      assert_eq!(result.predictor, "treated");
      assert_eq!((result.from_value, result.to_value), (0, 1));
      assert_eq!(result.observation_count, 120);
      assert_eq!(result.robust, covariance != "model");
      assert_eq!(
        result.cluster.as_ref().map(|cluster| cluster.group_count),
        (covariance == "cluster").then_some(20)
      );
      close(result.effect.value, expected[0]);
      close(result.effect.standard_error, expected[1]);
      close(result.effect.statistic, expected[2]);
      close(result.effect.p_value, expected[3]);
      close(result.effect.confidence_lower, expected[4]);
      close(result.effect.confidence_upper, expected[5]);
      assert_eq!(fitted, before);
    }
  }
}

#[test]
fn discrete_changes_reject_unknown_nonbinary_and_single_level_predictors() {
  let problem = problem(false);
  let fitted = fit_binary_logit(&problem).unwrap();
  assert!(binary_average_discrete_change(&problem, &fitted, "missing").is_err());

  let mut nonbinary = problem.clone();
  for row in &mut nonbinary.design_matrix {
    row[1] *= 2.0;
  }
  let nonbinary_fit = fit_binary_logit(&nonbinary).unwrap();
  let error = binary_average_discrete_change(&nonbinary, &nonbinary_fit, "treated").unwrap_err();
  assert!(error.to_string().contains("only 0 and 1"));

  let mut single_level = problem;
  single_level.include_intercept = false;
  for row in &mut single_level.design_matrix {
    row[1] = 1.0;
  }
  let single_level_fit = fit_binary_logit(&single_level).unwrap();
  let error =
    binary_average_discrete_change(&single_level, &single_level_fit, "treated").unwrap_err();
  assert!(error.to_string().contains("requires both 0 and 1"));
}

#[test]
fn discrete_change_requires_the_same_retained_sample_and_finite_covariance() {
  let problem = problem(false);
  let fitted = fit_binary_probit(&problem).unwrap();
  let mut different_sample = problem.clone();
  different_sample.sample.as_mut().unwrap().retained_indices[0] = 1;
  assert!(binary_average_discrete_change(&different_sample, &fitted, "treated").is_err());

  let mut nonfinite = fitted;
  nonfinite.covariance[0][0] = f64::NAN;
  assert!(binary_average_discrete_change(&problem, &nonfinite, "treated").is_err());
}
