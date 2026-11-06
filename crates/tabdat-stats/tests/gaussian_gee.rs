use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_stats::{EstimationProblem, EstimationSample, GaussianGeeOptions, fit_gaussian_gee};

const PREDICTORS: [&str; 6] = ["lnhhexp", "age", "sex", "hhsize", "farm", "educ"];
fn fixture(name: &str) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("../../tests/fixtures/mma24_household_gee")
    .join(name)
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
        .map(|v| v.trim_matches('"').to_owned())
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
    if ["lnexp12m", "commune"]
      .iter()
      .chain(PREDICTORS.iter())
      .any(|key| row[*key].is_empty())
    {
      continue;
    }
    y.push(row["lnexp12m"].parse().unwrap());
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
    EstimationProblem::new("lnexp12m", PREDICTORS.map(str::to_owned).to_vec(), y, x)
      .unwrap()
      .with_intercept_name("_cons")
      .with_sample(sample),
    ids,
  )
}
fn close(actual: f64, expected: f64, path: &str) {
  let tol = if path.ends_with("p_value") {
    expected.abs() * 1e-8
  } else {
    1e-9
  };
  assert!(
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= tol,
    "{path}: {actual} vs {expected}, tolerance {tol}"
  );
}
fn audit(root: &Path) -> f64 {
  let (p, ids) = problem(root);
  let fit = fit_gaussian_gee(&p, "commune", &Default::default()).unwrap();
  let mut actual = BTreeMap::<String, f64>::new();
  for (key, val) in [
    ("source_count", fit.sample.total_observations as f64),
    ("observation_count", fit.observation_count as f64),
    ("group_count", fit.group_count as f64),
    ("minimum_group_size", fit.minimum_group_size as f64),
    ("maximum_group_size", fit.maximum_group_size as f64),
    ("singleton_group_count", fit.singleton_group_count as f64),
    ("parameter_count", fit.working.coefficients.len() as f64),
    ("ordered_pair_count", fit.ordered_pair_count as f64),
    ("phi", fit.phi),
    ("rho", fit.rho),
  ] {
    actual.insert(key.into(), val);
  }
  for (kind, inf) in [("working", &fit.working), ("sandwich", &fit.sandwich)] {
    for (i, c) in inf.coefficients.iter().enumerate() {
      for (field, v) in [
        ("estimate", c.value),
        ("standard_error", c.standard_error),
        ("statistic", c.statistic),
        ("p_value", c.p_value),
        ("ci_lower", c.confidence_lower),
        ("ci_upper", c.confidence_upper),
      ] {
        actual.insert(format!("{kind}.{}.{field}", c.name), v);
      }
      for (j, other) in inf.coefficients.iter().enumerate() {
        actual.insert(
          format!("{kind}.covariance.{}.{}", c.name, other.name),
          inf.covariance[i][j],
        );
      }
    }
    for (key, v) in [
      ("wald_statistic", inf.wald_statistic),
      ("wald_df", inf.wald_df as f64),
      ("wald_p_value", inf.wald_p_value),
    ] {
      actual.insert(format!("{kind}.{key}"), v);
    }
  }
  let reference = rows(&root.join("reference.csv"));
  assert_eq!(reference.len(), 198);
  assert_eq!(actual.len(), 198);
  for row in reference {
    let path = &row["path"];
    close(
      *actual.get(path).unwrap_or_else(|| panic!("missing {path}")),
      row["value"].parse().unwrap(),
      path,
    );
  }
  let predictions = rows(&root.join("predictions.csv"))
    .into_iter()
    .map(|r| (r["source_row"].clone(), r))
    .collect::<BTreeMap<_, _>>();
  assert_eq!(ids.len(), fit.fitted_means.len());
  assert_eq!(predictions.len(), fit.sample.total_observations);
  let retained = ids.iter().collect::<std::collections::BTreeSet<_>>();
  for (id, row) in &predictions {
    assert_eq!(
      row["selected"] == "1",
      retained.contains(id),
      "sample row {id}"
    );
  }
  for (i, id) in ids.iter().enumerate() {
    let r = &predictions[id];
    assert_eq!(r["selected"], "1");
    close(fit.fitted_means[i], r["mean"].parse().unwrap(), "mean");
    close(
      p.response[i] - fit.fitted_means[i],
      r["residual"].parse().unwrap(),
      "residual",
    );
  }
  assert_eq!(&fit.sample, p.sample.as_ref().unwrap());
  assert!(fit.max_abs_scaled_score <= 1e-10);
  fit.rho
}
#[test]
fn independently_validated_positive_and_negative_exchangeable_panels() {
  assert!(audit(&fixture("positive")) > 0.0);
  assert!(audit(&fixture("negative")) < 0.0);
  if let Some(root) = std::env::var_os("TABDAT_GAUSSIAN_GEE_REFERENCE_ROOT") {
    audit(Path::new(&root));
  }
}
#[test]
fn ordering_scaling_and_explicit_failures() {
  let (p, _) = problem(&fixture("negative"));
  let fit = fit_gaussian_gee(&p, "commune", &Default::default()).unwrap();
  let mut reversed = p.clone();
  reversed.response.reverse();
  reversed.design_matrix.reverse();
  let sample = reversed.sample.as_mut().unwrap();
  sample.retained_indices.reverse();
  sample.cluster_groups.as_mut().unwrap().reverse();
  let other = fit_gaussian_gee(&reversed, "commune", &Default::default()).unwrap();
  close(other.rho, fit.rho, "reverse rho");
  for (a, b) in other
    .working
    .coefficients
    .iter()
    .zip(&fit.working.coefficients)
  {
    close(a.value, b.value, "reverse coefficient");
  }
  let mut scaled = p.clone();
  for row in &mut scaled.design_matrix {
    row[0] *= 1000.0;
  }
  let scaled = fit_gaussian_gee(&scaled, "commune", &Default::default()).unwrap();
  close(
    scaled.working.coefficients[1].value * 1000.0,
    fit.working.coefficients[1].value,
    "scaled slope",
  );
  close(scaled.rho, fit.rho, "scaled rho");
  let mut bad = p.clone();
  for row in &mut bad.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
  assert!(fit_gaussian_gee(&p, "commune", &GaussianGeeOptions { max_iterations: 1 }).is_err());
  bad = p.clone();
  bad.sample.as_mut().unwrap().cluster_groups =
    Some((0..bad.response.len()).map(|i| i.to_string()).collect());
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
  bad = p.clone();
  bad.response = bad.design_matrix.iter().map(|row| 3.0 + row[0]).collect();
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
  bad = p.clone();
  bad.sample.as_mut().unwrap().retained_indices[0] =
    bad.sample.as_ref().unwrap().retained_indices[1];
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
  bad = p.clone();
  bad.sample.as_mut().unwrap().total_observations = 40_001;
  bad.sample.as_mut().unwrap().dropped_observations = 40_001 - bad.response.len();
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
  bad = p.clone().with_intercept(false);
  assert!(fit_gaussian_gee(&bad, "commune", &Default::default()).is_err());
}
