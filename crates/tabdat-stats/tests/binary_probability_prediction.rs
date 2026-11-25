use tabdat_stats::{
  BinaryMleResult, BinaryModel, EstimationProblem, EstimationSample, binary_probability_prediction,
  fit_binary_clustered, fit_binary_logit, fit_binary_probit, fit_binary_robust,
};

#[derive(Clone)]
struct Row {
  id: usize,
  outcome: Option<f64>,
  x: Option<f64>,
  region: Option<String>,
  cluster: String,
}

fn fixture_rows() -> Vec<Row> {
  include_str!("../../../examples/binary-factor-design/binary_factor_data.csv")
    .lines()
    .skip(1)
    .map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      Row {
        id: fields[0].parse().unwrap(),
        outcome: fields[1].parse().ok(),
        x: fields[2].parse().ok(),
        region: (!fields[3].is_empty()).then(|| fields[3].trim_matches('"').to_owned()),
        cluster: fields[6].to_owned(),
      }
    })
    .collect()
}

fn problem(rows: &[Row], clustered: bool) -> EstimationProblem {
  let retained = rows
    .iter()
    .enumerate()
    .filter_map(|(index, row)| {
      Some((
        index,
        row.outcome?,
        row.x?,
        row.region.as_deref()?,
        row.cluster.clone(),
      ))
    })
    .collect::<Vec<_>>();
  let retained_indices = retained.iter().map(|row| row.0).collect::<Vec<_>>();
  let cluster_groups = retained.iter().map(|row| row.4.clone()).collect();
  EstimationProblem::new(
    "y",
    vec!["x".into(), "region_North".into(), "region_West".into()],
    retained.iter().map(|row| row.1).collect(),
    retained
      .iter()
      .map(|row| {
        vec![
          row.2,
          f64::from(row.3 == "North"),
          f64::from(row.3 == "West"),
        ]
      })
      .collect(),
  )
  .unwrap()
  .with_intercept(true)
  .with_sample(
    EstimationSample::new(
      retained_indices,
      rows.len(),
      rows.len() - retained.len(),
      None,
      clustered.then_some(cluster_groups),
    )
    .unwrap(),
  )
}

fn fit(problem: &EstimationProblem, model: BinaryModel, covariance: &str) -> BinaryMleResult {
  match (model, covariance) {
    (BinaryModel::Logit, "model_based") => fit_binary_logit(problem).unwrap(),
    (BinaryModel::Probit, "model_based") => fit_binary_probit(problem).unwrap(),
    (_, "robust") => fit_binary_robust(problem, model).unwrap(),
    (_, "cluster") => fit_binary_clustered(problem, model, "cluster").unwrap(),
    _ => unreachable!(),
  }
}

fn reference_rows(model: &str, design: &str, covariance: &str) -> Vec<Vec<&'static str>> {
  include_str!(
    "../../../examples/binary-factor-design/binary_factor_probability_inference_reference.csv"
  )
  .lines()
  .skip(1)
  .map(|line| line.split(',').collect::<Vec<_>>())
  .filter(|fields| {
    fields[0].trim_matches('"') == model
      && fields[1].trim_matches('"') == design
      && fields[2].trim_matches('"') == covariance
  })
  .collect()
}

fn close(actual: f64, expected: f64, label: &str) {
  assert!(
    (actual - expected).abs() <= 2e-8,
    "{label}: {actual:.17} != {expected:.17}"
  );
}

#[test]
fn current_row_logit_and_probit_uncertainty_match_independent_r() {
  // Base R 4.6.2 computes X V X' row-wise for both fitted links and all
  // selected covariance forms. It scores 240 current rows, including the row
  // with missing y; x/region exclusions yield NULL outputs for ids 5 and 12.
  let rows = fixture_rows();
  for (model, model_name) in [
    (BinaryModel::Logit, "logit"),
    (BinaryModel::Probit, "probit"),
  ] {
    for covariance in ["model_based", "robust", "cluster"] {
      let problem = problem(&rows, covariance == "cluster");
      let result = fit(&problem, model, covariance);
      for reference in reference_rows(model_name, "default", covariance) {
        let id = reference[3].parse::<usize>().unwrap();
        let level = reference[4].parse::<f64>().unwrap();
        let expected_estimate = reference[5].parse::<f64>().ok();
        let expected_se = reference[6].parse::<f64>().ok();
        let expected_lower = reference[7].parse::<f64>().ok();
        let expected_upper = reference[8].parse::<f64>().ok();
        let row = rows.iter().find(|row| row.id == id).unwrap();
        let Some((x, region)) = row.x.zip(row.region.as_deref()) else {
          assert!(expected_estimate.is_none());
          assert!(expected_se.is_none());
          assert!(expected_lower.is_none());
          assert!(expected_upper.is_none());
          continue;
        };
        let design = [
          1.0,
          x,
          f64::from(region == "North"),
          f64::from(region == "West"),
        ];
        let prediction = binary_probability_prediction(&result, &design, Some(level)).unwrap();
        close(prediction.estimate, expected_estimate.unwrap(), "estimate");
        close(
          prediction.standard_error,
          expected_se.unwrap(),
          "probability standard error",
        );
        close(
          prediction.lower.unwrap(),
          expected_lower.unwrap(),
          "lower bound",
        );
        close(
          prediction.upper.unwrap(),
          expected_upper.unwrap(),
          "upper bound",
        );
        assert!(0.0 <= prediction.lower.unwrap());
        assert!(prediction.lower.unwrap() <= prediction.estimate);
        assert!(prediction.estimate <= prediction.upper.unwrap());
        assert!(prediction.upper.unwrap() <= 1.0);
        let standard_error_only = binary_probability_prediction(&result, &design, None).unwrap();
        close(
          standard_error_only.standard_error,
          prediction.standard_error,
          "standard error-only request",
        );
        assert_eq!(standard_error_only.lower, None);
        assert_eq!(standard_error_only.upper, None);
      }
    }
  }
}

#[test]
fn binary_probability_uncertainty_validates_covariance_and_levels() {
  let rows = fixture_rows();
  let problem = problem(&rows, false);
  let result = fit_binary_logit(&problem).unwrap();
  let design = [1.0, 0.25, 1.0, 0.0];
  let mut zero_covariance = result.clone();
  zero_covariance.covariance = vec![vec![0.0; 4]; 4];
  let prediction = binary_probability_prediction(&zero_covariance, &design, Some(95.0)).unwrap();
  assert_eq!(prediction.standard_error, 0.0);
  assert_eq!(prediction.lower, Some(prediction.estimate));
  assert_eq!(prediction.upper, Some(prediction.estimate));

  assert!(binary_probability_prediction(&result, &design, Some(0.0)).is_err());
  assert!(binary_probability_prediction(&result, &design, Some(100.0)).is_err());
  assert!(binary_probability_prediction(&result, &design[..3], None).is_err());
  let mut negative_covariance = result;
  negative_covariance.covariance = vec![vec![0.0; 4]; 4];
  negative_covariance.covariance[0][0] = -1.0;
  assert!(binary_probability_prediction(&negative_covariance, &design, None).is_err());

  let mut overflowing_interval = fit_binary_logit(&problem).unwrap();
  overflowing_interval.covariance = vec![vec![0.0; 4]; 4];
  overflowing_interval.covariance[0][0] = f64::MAX;
  assert!(
    binary_probability_prediction(&overflowing_interval, &[1e154, 0.0, 0.0, 0.0], Some(95.0),)
      .is_err()
  );
}
