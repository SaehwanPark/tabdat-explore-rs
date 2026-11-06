use std::{collections::BTreeMap, fs, path::Path};
use tabdat_stats::{
  EstimationProblem, EstimationSample, GammaPoissonOptions, fit_gamma_poisson, fit_nb2,
};
const PREDICTORS: [&str; 10] = [
  "lnhhexp",
  "insurance",
  "sex",
  "age",
  "married",
  "illdays",
  "actdays",
  "injury",
  "illness",
  "educ",
];
fn fixture() -> std::path::PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/mma24_individual_gamma_re")
}
fn rows(path: &Path) -> Vec<BTreeMap<String, String>> {
  let text = fs::read_to_string(path).unwrap();
  let mut lines = text.lines();
  let names = lines
    .next()
    .unwrap()
    .split(',')
    .map(|v| v.trim_matches('"').to_owned())
    .collect::<Vec<_>>();
  lines
    .map(|line| {
      let values = line
        .split(',')
        .map(|s| s.trim_matches('"').to_owned())
        .collect::<Vec<_>>();
      assert_eq!(names.len(), values.len());
      names.iter().cloned().zip(values).collect()
    })
    .collect()
}
fn problem(root: &Path) -> (EstimationProblem, Vec<String>) {
  let source = rows(&root.join("input.csv"));
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut keys = Vec::new();
  let mut positions = Vec::new();
  let mut ids = Vec::new();
  for (i, row) in source.iter().enumerate() {
    if ["pharvis", "commune"]
      .iter()
      .chain(PREDICTORS.iter())
      .any(|key| row[*key].is_empty())
    {
      continue;
    }
    y.push(row["pharvis"].parse().unwrap());
    x.push(
      PREDICTORS
        .iter()
        .map(|key| row[*key].parse().unwrap())
        .collect(),
    );
    keys.push(row["commune"].clone());
    positions.push(i);
    ids.push(row["source_row"].clone());
  }
  let n = y.len();
  let sample =
    EstimationSample::new(positions, source.len(), source.len() - n, None, Some(keys)).unwrap();
  (
    EstimationProblem::new("pharvis", PREDICTORS.map(str::to_owned).to_vec(), y, x)
      .unwrap()
      .with_intercept_name("_cons")
      .with_sample(sample),
    ids,
  )
}
fn close(actual: f64, expected: f64, path: &str) {
  let tol = if path.ends_with("p_value") {
    expected.abs() * 1e-8
  } else if path.contains("log_likelihood") {
    1e-8
  } else {
    1e-9
  };
  assert!(
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= tol,
    "{path}: {actual} vs {expected}, tolerance {tol}"
  );
}
#[test]
fn independent_r_full_joint_reference_and_every_marginal_prediction() {
  // Explicit opt-in original-data replay; without it this always runs the
  // checked-in synthetic oracle, never silently passes an absent reference.
  let root = std::env::var_os("TABDAT_GAMMA_RE_REFERENCE_ROOT")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(fixture);
  let (p, ids) = problem(&root);
  let fit = fit_gamma_poisson(&p, "commune", &Default::default()).unwrap();
  let json = serde_json::to_value(&fit).unwrap();
  let reference = rows(&root.join("reference.csv"));
  assert_eq!(reference.len(), 236);
  let mut maximum = 0.0_f64;
  for r in &reference {
    let mut value = &json;
    for part in r["path"].split('/') {
      value = if value.is_array() {
        &value[part.parse::<usize>().unwrap()]
      } else {
        &value[part]
      };
    }
    let expected = r["value"].parse().unwrap();
    let actual = value
      .as_f64()
      .unwrap_or_else(|| panic!("missing numeric oracle path: {}", r["path"]));
    close(actual, expected, &r["path"]);
    maximum = maximum.max((actual - expected).abs());
  }
  let predictions = rows(&root.join("predictions.csv"))
    .into_iter()
    .map(|r| (r["source_row"].clone(), r))
    .collect::<BTreeMap<_, _>>();
  assert_eq!(predictions.len(), p.response.len());
  for (i, id) in ids.iter().enumerate() {
    let row = &predictions[id];
    let xb = fit.coefficients[0].value
      + p.design_matrix[i]
        .iter()
        .zip(&fit.coefficients[1..])
        .map(|(x, b)| x * b.value)
        .sum::<f64>();
    close(
      fit.fitted_means[i],
      row["re_mean"].parse().unwrap(),
      "re_mean",
    );
    close(xb, row["re_xb"].parse().unwrap(), "re_xb");
    close(
      p.response[i] - fit.fitted_means[i],
      row["re_residual"].parse().unwrap(),
      "re_residual",
    );
  }
  assert!(fit.max_abs_scaled_score <= 1e-10 && fit.max_abs_scaled_newton_step <= 1e-10);
  assert_eq!(&fit.sample, p.sample.as_ref().unwrap());
  println!(
    "236 R quantities and {} prediction rows pass; max quantity discrepancy {maximum:e}; iterations {}, score {:e}, step {:e}",
    ids.len(),
    fit.iterations,
    fit.max_abs_scaled_score,
    fit.max_abs_scaled_newton_step
  );
}
#[test]
fn shared_groups_differ_from_row_nb_and_singleton_gamma_is_exactly_row_nb() {
  let (p, _) = problem(&fixture());
  let shared = fit_gamma_poisson(&p, "commune", &Default::default()).unwrap();
  assert!(shared.zero_total_group_count >= 2 && shared.singleton_group_count == 2);
  let mut singletons = p.clone();
  singletons.sample.as_mut().unwrap().cluster_groups =
    Some((0..p.response.len()).map(|i| i.to_string()).collect());
  let individual = fit_gamma_poisson(&singletons, "row", &Default::default()).unwrap();
  let mut row_problem = p.clone();
  row_problem.sample.as_mut().unwrap().cluster_groups = None;
  let nb = fit_nb2(&row_problem, false).unwrap();
  assert!((shared.alpha - individual.alpha).abs() > 1e-3);
  close(
    individual.log_likelihood,
    nb.log_likelihood,
    "log_likelihood",
  );
  close(individual.alpha, nb.dispersion.alpha, "alpha");
  for (a, b) in individual.coefficients.iter().zip(&nb.coefficients) {
    close(a.value, b.value, &a.name);
    close(a.standard_error, b.standard_error, "se");
  }
  for (a, b) in individual
    .covariance
    .iter()
    .flatten()
    .zip(nb.covariance.iter().flatten())
  {
    close(*a, *b, "covariance");
  }
}
#[test]
fn predictor_units_and_row_order_do_not_change_gamma_inference_or_means() {
  let (p, _) = problem(&fixture());
  let a = fit_gamma_poisson(&p, "commune", &Default::default()).unwrap();
  let mut other = p.clone();
  let factors = [1e5, 1e-4, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
  for row in &mut other.design_matrix {
    for (x, f) in row.iter_mut().zip(factors) {
      *x *= f;
    }
  }
  let b = fit_gamma_poisson(&other, "commune", &Default::default()).unwrap();
  close(a.alpha, b.alpha, "alpha");
  close(a.log_likelihood, b.log_likelihood, "log_likelihood");
  for (i, f) in std::iter::once(1.0).chain(factors).enumerate() {
    close(
      a.coefficients[i].value,
      b.coefficients[i].value * f,
      "coefficient",
    );
    for (j, g) in std::iter::once(1.0).chain(factors).enumerate() {
      close(a.covariance[i][j], b.covariance[i][j] * f * g, "covariance");
    }
  }
  for (a, b) in a.fitted_means.iter().zip(&b.fitted_means) {
    close(*a, *b, "mean");
  }
  other = p.clone();
  other.response.reverse();
  other.design_matrix.reverse();
  let sample = other.sample.as_mut().unwrap();
  sample.retained_indices.reverse();
  sample.cluster_groups.as_mut().unwrap().reverse();
  let b = fit_gamma_poisson(&other, "commune", &Default::default()).unwrap();
  close(a.log_likelihood, b.log_likelihood, "log_likelihood");
  for (a, b) in a.fitted_means.iter().rev().zip(&b.fitted_means) {
    close(*a, *b, "mean");
  }
}
#[test]
fn rejects_invalid_counts_mapping_weights_rank_resource_limits_and_boundary() {
  let (p, _) = problem(&fixture());
  for count in [-1.0, 0.5, 100001.0, f64::NAN] {
    let mut bad = p.clone();
    bad.response[0] = count;
    assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  }
  let mut bad = p.clone();
  bad.response.fill(100000.0);
  assert!(
    fit_gamma_poisson(&bad, "commune", &Default::default())
      .unwrap_err()
      .to_string()
      .contains("group count totals")
  );
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().retained_indices[0] = usize::MAX;
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().retained_indices[0] =
    bad.sample.as_ref().unwrap().retained_indices[1];
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().cluster_groups = None;
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().total_observations = 40001;
  bad.sample.as_mut().unwrap().dropped_observations = 40001 - bad.response.len();
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  bad.include_intercept = false;
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  bad.weights = Some(vec![1.0; p.response.len()]);
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p.clone();
  for row in &mut bad.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  assert!(fit_gamma_poisson(&p, "", &Default::default()).is_err());
  for max_iterations in [0, 1] {
    assert!(fit_gamma_poisson(&p, "commune", &GammaPoissonOptions { max_iterations }).is_err());
  }
  let mut bad = p.clone();
  bad.response.fill(0.0);
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
  let mut bad = p;
  bad.response.fill(1.0);
  assert!(fit_gamma_poisson(&bad, "commune", &Default::default()).is_err());
}
