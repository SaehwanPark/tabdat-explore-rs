use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_nb2, fit_nb2_clustered, nb2_count_goodness_of_fit,
};

const DATA: &str = include_str!("fixtures/nb2_cluster_count_gof_data.csv");
const REFERENCE: &str = include_str!("fixtures/nb2_cluster_count_gof_reference.csv");

fn problem() -> EstimationProblem {
  let mut clusters = Vec::new();
  let mut response = Vec::new();
  let mut design = Vec::new();
  for line in DATA.lines().skip(1) {
    let row: Vec<&str> = line.split(',').collect();
    clusters.push(row[0].to_owned());
    response.push(row[1].parse().unwrap());
    design.push(vec![row[2].parse().unwrap()]);
  }
  EstimationProblem::new("y", vec!["x".into()], response, design)
    .unwrap()
    .with_sample(
      EstimationSample::new(
        (0..clusters.len()).collect(),
        clusters.len(),
        0,
        None,
        Some(clusters),
      )
      .unwrap(),
    )
}

fn reference_value(quantity: &str, i: usize, j: usize) -> f64 {
  REFERENCE
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields: Vec<&str> = line.split(',').collect();
      (fields[0] == "clustered"
        && fields[1] == quantity
        && fields[2].parse::<usize>().ok() == Some(i)
        && fields[3].parse::<usize>().ok() == Some(j))
      .then(|| fields[4].parse::<f64>().unwrap())
    })
    .unwrap_or_else(|| panic!("missing reference for {quantity}[{i},{j}]"))
}

fn close(actual: f64, expected: f64, tolerance: f64) {
  assert!(
    (actual - expected).abs() <= tolerance,
    "actual {actual:.17e}, expected {expected:.17e}, tolerance {tolerance:.3e}"
  );
}

#[test]
fn clustered_grouped_gof_matches_independent_base_r_reference() {
  let problem = problem();
  let fit = fit_nb2_clustered(&problem, "person").unwrap();
  let result = nb2_count_goodness_of_fit(&problem, &fit, 4).unwrap();
  let cluster = result.cluster.as_ref().unwrap();
  let covariance = result.moment_covariance.as_ref().unwrap();

  assert_eq!(result.observation_count, 480);
  assert_eq!(result.tail_from, 4);
  assert_eq!(result.degrees_of_freedom, 4);
  assert_eq!(result.auxiliary_regressor_count, 7);
  assert_eq!(result.score_parameter_names, ["intercept", "x", "lnalpha"]);
  assert_eq!(cluster.variable, "person");
  assert_eq!(cluster.group_count, 96);
  close(
    cluster.covariance_correction,
    reference_value("covariance_correction", 0, 0),
    1e-14,
  );
  close(result.statistic, reference_value("statistic", 0, 0), 2e-6);
  close(result.p_value, reference_value("p_value", 0, 0), 2e-7);
  close(
    fit.log_likelihood,
    reference_value("log_likelihood", 0, 0),
    2e-7,
  );
  close(fit.dispersion.alpha, reference_value("alpha", 0, 0), 2e-8);
  for (index, coefficient) in fit.coefficients.iter().enumerate() {
    close(
      coefficient.value,
      reference_value("coefficient", index, 0),
      2e-7,
    );
  }
  assert_eq!(covariance.len(), 4);
  for (row, values) in covariance.iter().enumerate() {
    assert_eq!(values.len(), 4);
    for (column, value) in values.iter().enumerate() {
      close(
        *value,
        reference_value("moment_covariance", row, column),
        2e-7,
      );
    }
  }
  for (index, bin) in result.table.bins.iter().enumerate() {
    assert_eq!(
      bin.observed_count as f64,
      reference_value("observed_count", index, 0)
    );
    close(
      bin.fitted_fraction,
      reference_value("fitted_fraction", index, 0),
      2e-8,
    );
  }
  assert!(serde_json::to_value(&result).unwrap()["cluster"].is_object());
  assert!(serde_json::to_value(&result).unwrap()["moment_covariance"].is_array());
}

#[test]
fn clustered_grouped_gof_rejects_unsupported_or_rank_deficient_inputs() {
  let mut clustered = problem();
  let fit = fit_nb2_clustered(&clustered, "person").unwrap();

  let mut mismatched_groups = clustered.clone();
  mismatched_groups.sample.as_mut().unwrap().cluster_groups =
    Some(vec!["other".into(); mismatched_groups.observation_count()]);
  assert!(nb2_count_goodness_of_fit(&mismatched_groups, &fit, 4).is_err());

  let mut changed_fit = fit.clone();
  changed_fit.sample.as_mut().unwrap().cluster_groups = None;
  assert!(nb2_count_goodness_of_fit(&clustered, &changed_fit, 4).is_err());

  let mut robust = clustered.clone();
  robust.sample.as_mut().unwrap().cluster_groups = None;
  let row_robust_fit = fit_nb2(&robust, true).unwrap();
  assert!(nb2_count_goodness_of_fit(&robust, &row_robust_fit, 4).is_err());

  clustered.sample.as_mut().unwrap().cluster_groups = Some(
    (0..clustered.observation_count())
      .map(|i| (i % 3).to_string())
      .collect(),
  );
  let few_group_fit = fit_nb2_clustered(&clustered, "few").unwrap();
  assert!(nb2_count_goodness_of_fit(&clustered, &few_group_fit, 4).is_err());

  let mut one_group = clustered;
  one_group.sample.as_mut().unwrap().cluster_groups =
    Some(vec!["only".into(); one_group.observation_count()]);
  assert!(fit_nb2_clustered(&one_group, "person").is_err());
}
