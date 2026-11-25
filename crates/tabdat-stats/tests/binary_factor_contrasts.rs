use tabdat_stats::{
  BinaryFactorContrastDesign, BinaryModel, EstimationProblem, EstimationSample,
  binary_average_factor_contrasts, fit_binary_clustered, fit_binary_logit, fit_binary_probit,
  fit_binary_robust,
};

fn field<'a>(row: &[&'a str], index: usize) -> &'a str {
  row[index].trim_matches('"')
}

fn reference_rows(model: &str, design: &str, covariance: &str) -> Vec<Vec<&'static str>> {
  include_str!("../../../examples/binary-factor-design/binary_factor_contrast_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect::<Vec<_>>())
    .filter(|row| field(row, 0) == model && field(row, 1) == design && field(row, 2) == covariance)
    .collect()
}

fn fixture(design: &str, clustered: bool) -> (EstimationProblem, BinaryFactorContrastDesign) {
  let mut response = Vec::new();
  let mut matrix = Vec::new();
  let mut retained = Vec::new();
  let mut groups = Vec::new();
  for (index, line) in include_str!("../../../examples/binary-factor-design/binary_factor_data.csv")
    .lines()
    .skip(1)
    .enumerate()
  {
    let row = line
      .split(',')
      .map(|value| value.trim_matches('"'))
      .collect::<Vec<_>>();
    if row[1].is_empty() || row[2].is_empty() || row[3].is_empty() {
      continue;
    }
    let x = row[2].parse::<f64>().unwrap();
    let y = row[1].parse::<f64>().unwrap();
    let region = row[3];
    let (predictors, layout) = match design {
      "default" => (
        vec![
          "x".to_owned(),
          "region_North".to_owned(),
          "region_West".to_owned(),
        ],
        vec![
          x,
          f64::from(u8::from(region == "North")),
          f64::from(u8::from(region == "West")),
        ],
      ),
      "west_reference" => (
        vec![
          "x".to_owned(),
          "region_East".to_owned(),
          "region_North".to_owned(),
        ],
        vec![
          x,
          f64::from(u8::from(region == "East")),
          f64::from(u8::from(region == "North")),
        ],
      ),
      "noconstant" => (
        vec![
          "x".to_owned(),
          "region_East".to_owned(),
          "region_North".to_owned(),
          "region_West".to_owned(),
        ],
        vec![
          x,
          f64::from(u8::from(region == "East")),
          f64::from(u8::from(region == "North")),
          f64::from(u8::from(region == "West")),
        ],
      ),
      _ => unreachable!(),
    };
    if response.is_empty() {
      matrix.reserve(240);
    }
    response.push(y);
    retained.push(index);
    matrix.push(layout);
    groups.push(row[6].to_owned());
    assert_eq!(predictors.len(), matrix.last().unwrap().len());
  }

  let (predictor_names, include_intercept) = match design {
    "default" => (
      vec!["x".into(), "region_North".into(), "region_West".into()],
      true,
    ),
    "west_reference" => (
      vec!["x".into(), "region_East".into(), "region_North".into()],
      true,
    ),
    "noconstant" => (
      vec![
        "x".into(),
        "region_East".into(),
        "region_North".into(),
        "region_West".into(),
      ],
      false,
    ),
    _ => unreachable!(),
  };
  let problem = EstimationProblem::new("y", predictor_names, response, matrix)
    .unwrap()
    .with_intercept(include_intercept)
    .with_sample(
      EstimationSample::new(
        retained.clone(),
        240,
        240 - retained.len(),
        None,
        clustered.then_some(groups),
      )
      .unwrap(),
    );
  let predictor_columns = match design {
    "default" => vec![None, Some(1), Some(2)],
    "west_reference" => vec![Some(1), Some(2), None],
    "noconstant" => vec![Some(1), Some(2), Some(3)],
    _ => unreachable!(),
  };
  let baseline_level = if design == "west_reference" {
    "West"
  } else {
    "East"
  };
  (
    problem,
    BinaryFactorContrastDesign {
      variable: "region".into(),
      baseline_level: baseline_level.into(),
      levels: vec!["East".into(), "North".into(), "West".into()],
      predictor_columns,
    },
  )
}

fn fit(
  problem: &EstimationProblem,
  model: BinaryModel,
  covariance: &str,
) -> tabdat_stats::BinaryMleResult {
  match (model, covariance) {
    (BinaryModel::Logit, "model_based") => fit_binary_logit(problem).unwrap(),
    (BinaryModel::Probit, "model_based") => fit_binary_probit(problem).unwrap(),
    (_, "robust") => fit_binary_robust(problem, model).unwrap(),
    (_, "cluster") => fit_binary_clustered(problem, model, "cluster").unwrap(),
    _ => unreachable!(),
  }
}

fn close(actual: f64, expected: f64, context: &str) {
  assert!(
    (actual - expected).abs() <= 2e-8,
    "{context}: {actual:.17} != {expected:.17}"
  );
}

#[test]
fn factor_probability_contrasts_match_base_r_for_links_covariances_and_references() {
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    let model_name = if model == BinaryModel::Logit {
      "logit"
    } else {
      "probit"
    };
    for (design_name, covariance) in [
      ("default", "model_based"),
      ("default", "robust"),
      ("default", "cluster"),
      ("west_reference", "model_based"),
      ("noconstant", "model_based"),
    ] {
      let (problem, design) = fixture(design_name, covariance == "cluster");
      let fitted = fit(&problem, model, covariance);
      let result = binary_average_factor_contrasts(&problem, &fitted, &design).unwrap();
      let reference = reference_rows(model_name, design_name, covariance);
      assert_eq!(result.factor_variable, "region");
      assert_eq!(result.baseline_level, design.baseline_level);
      assert_eq!(result.observation_count, 237);
      assert_eq!(result.robust, covariance != "model_based");
      assert_eq!(result.contrasts.len(), reference.len());
      for (actual, expected) in result.contrasts.iter().zip(reference) {
        assert_eq!(actual.level, field(&expected, 5));
        assert_eq!(
          actual.effect.name,
          format!("region: {} vs {}", field(&expected, 5), field(&expected, 4))
        );
        for (name, value, index) in [
          ("estimate", actual.effect.value, 6),
          ("standard_error", actual.effect.standard_error, 7),
          ("statistic", actual.effect.statistic, 8),
          ("p_value", actual.effect.p_value, 9),
          ("confidence_lower", actual.effect.confidence_lower, 10),
          ("confidence_upper", actual.effect.confidence_upper, 11),
        ] {
          close(
            value,
            field(&expected, index).parse().unwrap(),
            &format!("{model_name}/{design_name}/{covariance}/{name}"),
          );
        }
      }
    }
  }
}

#[test]
fn factor_probability_contrasts_reject_mismatched_or_malformed_designs() {
  let (problem, design) = fixture("default", false);
  let fitted = fit_binary_logit(&problem);
  let fitted = fitted.unwrap();

  let mut changed_sample = problem.clone();
  changed_sample.sample.as_mut().unwrap().retained_indices[0] = 1;
  assert!(binary_average_factor_contrasts(&changed_sample, &fitted, &design).is_err());

  let mut malformed = design;
  malformed.predictor_columns[0] = Some(0);
  assert!(binary_average_factor_contrasts(&problem, &fitted, &malformed).is_err());
}
