use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_nb2, fit_nb2_with_offset, nb2_count_goodness_of_fit,
};

const REFERENCE: &str = include_str!("fixtures/nb2_count_gof_reference.csv");

fn problem() -> EstimationProblem {
  let response = vec![
    0., 0., 2., 0., 0., 3., 0., 3., 1., 3., 8., 3., 0., 0., 0., 2., 1., 2., 0., 0., 2., 1., 1., 2.,
    0., 5., 4., 1., 1., 0., 0., 2., 2., 0., 1., 0.,
  ];
  let design = (0..response.len())
    .map(|index| vec![-2.0 + 4.0 * index as f64 / 35.0, [0.0, 1.0, 0.4][index % 3]])
    .collect();
  EstimationProblem::new("y", vec!["x".into(), "z".into()], response, design)
    .unwrap()
    .with_sample(EstimationSample::all_retained(36))
}

fn reference_value(quantity: &str, index: usize) -> f64 {
  REFERENCE
    .lines()
    .skip(1)
    .filter_map(|line| {
      let fields: Vec<&str> = line.split(',').collect();
      if fields[0] == "multiple"
        && fields[1] == quantity
        && fields[2].parse::<usize>().ok() == Some(index)
      {
        fields[4].parse::<f64>().ok()
      } else {
        None
      }
    })
    .next()
    .unwrap_or_else(|| panic!("missing reference for {quantity}[{index}]"))
}

fn scalar_reference(quantity: &str) -> f64 {
  reference_value(quantity, 0)
}

fn close(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.17e}, expected {expected:.17e}, tolerance {tolerance:.3e}"
  );
}

#[test]
fn grouped_cgf_matches_independent_base_r_auxiliary_regression() {
  let problem = problem();
  let fit = fit_nb2(&problem, false).unwrap();
  let result = nb2_count_goodness_of_fit(&problem, &fit, 10).unwrap();

  assert_eq!(result.observation_count, 36);
  assert_eq!(result.tail_from, 10);
  assert_eq!(result.degrees_of_freedom, 10);
  assert_eq!(result.auxiliary_regressor_count, 14);
  assert_eq!(
    result.score_parameter_names,
    ["intercept", "x", "z", "lnalpha"]
  );
  close(result.statistic, scalar_reference("statistic"), 2e-8);
  close(result.p_value, scalar_reference("p_value"), 2e-10);
  let wire = serde_json::to_value(&result).unwrap();
  assert!(wire.get("cluster").is_none());
  assert!(wire.get("moment_covariance").is_none());
  close(
    result.observation_count as f64,
    scalar_reference("observation_count"),
    0.0,
  );
  for (index, bin) in result.table.bins.iter().enumerate() {
    assert_eq!(
      bin.observed_count as f64,
      reference_value("observed_count", index)
    );
    close(
      bin.observed_fraction,
      reference_value("observed_fraction", index),
      2e-12,
    );
    close(
      bin.fitted_fraction,
      reference_value("fitted_fraction", index),
      2e-10,
    );
    close(
      bin.expected_count,
      reference_value("expected_count", index),
      2e-8,
    );
  }
}

#[test]
fn grouped_cgf_rejects_unsupported_covariance_offsets_and_cutoffs() {
  let problem = problem();
  let robust = fit_nb2(&problem, true).unwrap();
  assert!(nb2_count_goodness_of_fit(&problem, &robust, 10).is_err());

  let weighted_sample =
    EstimationSample::new((0..36).collect(), 36, 0, Some(vec![1.0; 36]), None).unwrap();
  let weighted = problem.clone().with_sample(weighted_sample.clone());
  let mut weighted_fit = fit_nb2(&problem, false).unwrap();
  weighted_fit.sample = Some(weighted_sample);
  assert!(nb2_count_goodness_of_fit(&weighted, &weighted_fit, 10).is_err());

  let mut direct_weighted = problem.clone();
  direct_weighted.weights = Some(vec![1.0; 36]);
  assert!(
    nb2_count_goodness_of_fit(&direct_weighted, &fit_nb2(&problem, false).unwrap(), 10).is_err()
  );

  let offset_fit =
    fit_nb2_with_offset(&problem, &vec![0.0; 36], Some("offset"), false, None).unwrap();
  assert!(nb2_count_goodness_of_fit(&problem, &offset_fit, 10).is_err());

  let fit = fit_nb2(&problem, false).unwrap();
  assert!(nb2_count_goodness_of_fit(&problem, &fit, 0).is_err());
  assert!(nb2_count_goodness_of_fit(&problem, &fit, 11).is_err());
}
