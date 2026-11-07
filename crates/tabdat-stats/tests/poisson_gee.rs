use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};
use tabdat_stats::{
  EstimationProblem, EstimationSample, PoissonGeeOptions, PoissonGeeResult, fit_poisson_gee,
};
fn fixture(case: &str) -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/poisson_gee")
    .join(case)
}
fn rows(path: &Path) -> Vec<BTreeMap<String, String>> {
  let text = fs::read_to_string(path).unwrap();
  let mut lines = text.lines();
  let names: Vec<_> = lines
    .next()
    .unwrap()
    .split(',')
    .map(str::to_owned)
    .collect();
  lines
    .map(|line| {
      let values: Vec<_> = line.split(',').map(str::to_owned).collect();
      assert_eq!(names.len(), values.len());
      names.iter().cloned().zip(values).collect()
    })
    .collect()
}
fn make(y: Vec<f64>, x: Vec<Vec<f64>>, keys: Vec<String>) -> EstimationProblem {
  let n = y.len();
  let p = x[0].len();
  let sample = EstimationSample::new((0..n).collect(), n, 0, None, Some(keys)).unwrap();
  EstimationProblem::new("y", (1..=p).map(|i| format!("x{i}")).collect(), y, x)
    .unwrap()
    .with_intercept_name("_cons")
    .with_sample(sample)
}
fn problem(case: &str) -> EstimationProblem {
  let data = rows(&fixture(case).join("input.csv"));
  make(
    data.iter().map(|r| r["y"].parse().unwrap()).collect(),
    data
      .iter()
      .map(|r| vec![r["x1"].parse().unwrap(), r["x2"].parse().unwrap()])
      .collect(),
    data.iter().map(|r| r["id"].clone()).collect(),
  )
}
fn near(a: f64, b: f64, label: &str) {
  let tol = if label.ends_with("p_value") && b > 0.0 {
    1e-8 * b
  } else {
    1e-9 + 1e-10 * b.abs()
  };
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= tol,
    "{label}: {a} vs {b} tol={tol}"
  );
}
fn values(fit: &PoissonGeeResult) -> BTreeMap<String, f64> {
  let mut out = BTreeMap::new();
  for (key, value) in [
    ("observations", fit.observation_count as f64),
    ("groups", fit.group_count as f64),
    ("group_size", fit.group_size as f64),
    ("rho", fit.rho),
    ("pearson_normalizer", fit.pearson_normalizer),
    ("working_scale", fit.working_scale),
    ("sandwich_correction", fit.sandwich_correction),
    ("zero_total_groups", fit.zero_total_group_count as f64),
  ] {
    out.insert(key.into(), value);
  }
  for (name, inference) in [("working", &fit.working), ("sandwich", &fit.sandwich)] {
    for (j, c) in inference.coefficients.iter().enumerate() {
      for (field, value) in [
        ("value", c.value),
        ("standard_error", c.standard_error),
        ("statistic", c.statistic),
        ("p_value", c.p_value),
        ("lower", c.confidence_lower),
        ("upper", c.confidence_upper),
      ] {
        out.insert(format!("{name}.{}.{field}", c.name), value);
      }
      for (l, value) in inference.covariance[j].iter().enumerate() {
        out.insert(format!("{name}.covariance.{j}.{l}"), *value);
      }
    }
    out.insert(format!("{name}.wald"), inference.wald_statistic);
    out.insert(format!("{name}.wald_p_value"), inference.wald_p_value);
    assert_eq!(inference.wald_df, fit.predictor_names.len());
  }
  out
}
fn audit(case: &str) {
  let p = problem(case);
  let fit = fit_poisson_gee(&p, "id", &Default::default()).unwrap();
  let actual = values(&fit);
  let reference = rows(&fixture(case).join("reference.csv"));
  assert_eq!(actual.len(), 66);
  assert_eq!(reference.len(), 66);
  for r in reference {
    near(
      actual[&r["field"]],
      r["value"].parse().unwrap(),
      &r["field"],
    );
  }
  assert_eq!(fit.sample, p.sample.clone().unwrap());
  assert!(fit.max_abs_scaled_score <= 1e-10);
  assert!(fit.max_abs_scaled_newton_step <= 1e-12);
  assert!(fit.iterations > 0 && fit.iterations <= 100);
  let prediction = rows(&fixture(case).join("predictions.csv"));
  assert_eq!(prediction.len(), p.response.len());
  for (i, row) in prediction.iter().enumerate() {
    let eta = fit.working.coefficients[0].value
      + p.design_matrix[i]
        .iter()
        .zip(&fit.working.coefficients[1..])
        .map(|(x, b)| x * b.value)
        .sum::<f64>();
    let mu = fit.fitted_means[i];
    for (field, value) in [
      ("mean", mu),
      ("xb", eta),
      ("residual", p.response[i] - mu),
      ("pearson", (p.response[i] - mu) / mu.sqrt()),
    ] {
      near(value, row[field].parse().unwrap(), field);
    }
  }
  let reference = rows(&fixture(case).join("scores.csv"));
  let keys = p.sample.as_ref().unwrap().cluster_groups.as_ref().unwrap();
  for row in reference {
    let group: Vec<_> = keys
      .iter()
      .enumerate()
      .filter(|(_, key)| **key == row["id"])
      .map(|(i, _)| i)
      .collect();
    let residual: Vec<_> = group
      .iter()
      .map(|&i| (p.response[i] - fit.fitted_means[i]) / fit.fitted_means[i].sqrt())
      .collect();
    let sum: f64 = residual.iter().sum();
    let adjustment = fit.rho / (1.0 + (group.len() - 1) as f64 * fit.rho);
    for (j, field) in ["score_cons", "score_x1", "score_x2"].iter().enumerate() {
      let score = group
        .iter()
        .zip(&residual)
        .map(|(&i, e)| {
          fit.fitted_means[i].sqrt()
            * (if j == 0 {
              1.0
            } else {
              p.design_matrix[i][j - 1]
            })
            * (e - adjustment * sum)
            / (1.0 - fit.rho)
        })
        .sum::<f64>();
      let expected: f64 = row[*field].parse().unwrap();
      assert!(
        (score - expected).abs() <= 1e-8 + 1e-10 * expected.abs(),
        "{field}: {score} vs {expected}"
      );
    }
  }
}
#[test]
fn multivariate_dense_r_full_inference_predictions_and_scores() {
  audit("multivariate");
}
#[test]
fn rescaled_dense_r_full_inference_predictions_and_scores() {
  audit("rescaled");
}
#[test]
fn rescaling_and_permutation_preserve_natural_mean_and_covariance() {
  let p = problem("multivariate");
  let base = fit_poisson_gee(&p, "id", &Default::default()).unwrap();
  let scaled = fit_poisson_gee(&problem("rescaled"), "id", &Default::default()).unwrap();
  for (a, b) in base.fitted_means.iter().zip(&scaled.fitted_means) {
    near(*a, *b, "mean");
  }
  for (a, b) in [
    (&base.working, &scaled.working),
    (&base.sandwich, &scaled.sandwich),
  ] {
    for i in 0..3 {
      near(
        a.coefficients[i].value,
        b.coefficients[i].value / (if i == 2 { 1e6 } else { 1.0 }),
        "coefficient",
      );
      for j in 0..3 {
        near(
          a.covariance[i][j],
          b.covariance[i][j] / (if i == 2 { 1e6 } else { 1.0 }) / (if j == 2 { 1e6 } else { 1.0 }),
          "covariance",
        );
      }
    }
  }
  let mut reversed = p.clone();
  reversed.response.reverse();
  reversed.design_matrix.reverse();
  reversed
    .sample
    .as_mut()
    .unwrap()
    .cluster_groups
    .as_mut()
    .unwrap()
    .reverse();
  let fit = fit_poisson_gee(&reversed, "id", &Default::default()).unwrap();
  near(base.rho, fit.rho, "rho");
  for (a, b) in base.fitted_means.iter().rev().zip(&fit.fitted_means) {
    near(*a, *b, "permuted mean");
  }
  for (key, value) in values(&base) {
    near(values(&fit)[&key], value, &key);
  }
}
#[test]
fn exact_sample_mapping_and_zero_total_groups_are_retained() {
  let mut p = problem("multivariate");
  let n = p.response.len();
  let keys = p.sample.as_ref().unwrap().cluster_groups.clone();
  p.sample =
    Some(EstimationSample::new((0..n).map(|i| i * 2).collect(), n * 2, n, None, keys).unwrap());
  let fit = fit_poisson_gee(&p, "id", &Default::default()).unwrap();
  assert_eq!(fit.sample, p.sample.unwrap());
  assert_eq!(fit.group_count, 12);
  assert_eq!(fit.zero_total_group_count, 2);
  assert_eq!(fit.observation_count, 60);
}
#[test]
fn legacy_positive_and_negative_count_gee_match_frozen_r() {
  let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
  for case in ["positive", "negative"] {
    let source = if case == "positive" {
      root.join("mma23_conditional_fe/source.asc")
    } else {
      root.join("mma23_exchangeable_gee/negative/source.asc")
    };
    let text = fs::read_to_string(source).unwrap();
    let tokens: Vec<_> = text.split_whitespace().collect();
    let mut y = Vec::new();
    let mut x = Vec::new();
    let mut keys = Vec::new();
    let (records, remainder) = tokens.as_chunks::<25>();
    assert!(remainder.is_empty());
    for (i, record) in records.iter().enumerate() {
      for t in 0..5 {
        y.push(record[20 + t].parse().unwrap());
        x.push(vec![record[10 + t].parse().unwrap()]);
        keys.push((i + 1).to_string());
      }
    }
    let p = make(y, x, keys);
    let fit = fit_poisson_gee(&p, "id", &Default::default()).unwrap();
    assert_eq!(fit.rho < 0.0, case == "negative");
    let mut actual = BTreeMap::from([
      ("observations".to_owned(), fit.observation_count as f64),
      ("firms".into(), fit.group_count as f64),
      ("intercept".into(), fit.working.coefficients[0].value),
      ("slope".into(), fit.working.coefficients[1].value),
      ("rho".into(), fit.rho),
    ]);
    for (name, inf) in [("working", &fit.working), ("cluster", &fit.sandwich)] {
      for (suffix, i, j) in [("aa", 0, 0), ("ab", 0, 1), ("bb", 1, 1)] {
        actual.insert(format!("{name}_{suffix}"), inf.covariance[i][j]);
      }
      for (label, c) in ["intercept", "slope"].iter().zip(&inf.coefficients) {
        for (field, value) in [
          ("se", c.standard_error),
          ("z", c.statistic),
          ("lo", c.confidence_lower),
          ("hi", c.confidence_upper),
        ] {
          actual.insert(format!("{name}_{field}_{label}"), value);
        }
      }
    }
    let dir = root.join("mma23_exchangeable_gee").join(case);
    let reference = rows(&dir.join("reference.csv"));
    assert_eq!(reference.len(), 27);
    for r in reference {
      near(
        actual[&r["field"]],
        r["value"].parse().unwrap(),
        &r["field"],
      );
    }
    let group_keys = p.sample.as_ref().unwrap().cluster_groups.as_ref().unwrap();
    for row in rows(&dir.join("scores.csv")) {
      let group = group_keys
        .iter()
        .enumerate()
        .filter(|(_, id)| **id == row["id"])
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
      let e = group
        .iter()
        .map(|&i| (p.response[i] - fit.fitted_means[i]) / fit.fitted_means[i].sqrt())
        .collect::<Vec<_>>();
      let adjust = fit.rho / (1.0 + 4.0 * fit.rho) * e.iter().sum::<f64>();
      for (j, field) in ["score_a", "score_b"].iter().enumerate() {
        let score = group
          .iter()
          .zip(&e)
          .map(|(&i, e)| {
            fit.fitted_means[i].sqrt()
              * (if j == 0 { 1.0 } else { p.design_matrix[i][0] })
              * (e - adjust)
              / (1.0 - fit.rho)
          })
          .sum::<f64>();
        let expected: f64 = row[*field].parse().unwrap();
        assert!(
          (score - expected).abs() <= 1e-8 + 1e-10 * expected.abs(),
          "{case}/{field}: {score} vs {expected}"
        );
      }
    }
    for (i, row) in rows(&dir.join("predictions.csv")).iter().enumerate() {
      near(
        fit.fitted_means[i],
        row["gee_mu"].parse().unwrap(),
        "legacy mean",
      );
      near(
        p.response[i] - fit.fitted_means[i],
        row["gee_residual"].parse().unwrap(),
        "legacy residual",
      );
    }
  }
}
#[test]
fn malformed_weighted_unbalanced_and_nonconvergent_problems_fail() {
  let p = problem("multivariate");
  let mut bad = Vec::new();
  for value in [-1.0, 0.5, 100001.0, f64::NAN, f64::INFINITY] {
    let mut q = p.clone();
    q.response[0] = value;
    bad.push(q);
  }
  let mut q = p.clone();
  q.response.fill(0.0);
  bad.push(q);
  let mut q = p.clone();
  q.response.fill(1.0);
  bad.push(q);
  let mut q = p.clone();
  q.include_intercept = false;
  bad.push(q);
  let mut q = p.clone();
  q.weights = Some(vec![1.0; 60]);
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().weights = Some(vec![1.0; 60]);
  bad.push(q);
  let mut q = p.clone();
  q.sample = None;
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().cluster_groups = None;
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().cluster_groups.as_mut().unwrap()[0] = "2".into();
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().cluster_groups = Some((0..60).map(|i| (i % 3).to_string()).collect());
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().retained_indices[0] = 60;
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().retained_indices[0] = 1;
  bad.push(q);
  let mut q = p.clone();
  q.sample.as_mut().unwrap().total_observations = 40001;
  q.sample.as_mut().unwrap().dropped_observations = 39941;
  bad.push(q);
  let mut q = p.clone();
  for row in &mut q.design_matrix {
    row[1] = row[0];
  }
  bad.push(q);
  let mut q = p.clone();
  q.predictor_names[1] = "_cons".into();
  bad.push(q);
  let mut q = p.clone();
  q.predictor_names[1] = q.predictor_names[0].clone();
  bad.push(q);
  assert_eq!(bad.len(), 20);
  for (i, q) in bad.iter().enumerate() {
    assert!(
      fit_poisson_gee(q, "id", &Default::default()).is_err(),
      "bad case{i}"
    );
  }
  for max_iterations in [0, 1, 101] {
    assert!(fit_poisson_gee(&p, "id", &PoissonGeeOptions { max_iterations }).is_err());
  }
  assert!(fit_poisson_gee(&p, " ", &Default::default()).is_err());
}
#[test]
fn resource_and_shape_limits_fail_before_iteration() {
  let mut p = problem("multivariate");
  p.sample.as_mut().unwrap().cluster_groups = Some((0..60).map(|i| i.to_string()).collect());
  let error = fit_poisson_gee(&p, "id", &Default::default()).unwrap_err();
  assert!(error.to_string().contains("T>=2"), "{error}");
  let mut p = problem("multivariate");
  p.predictor_names = (1..=32).map(|i| format!("x{i}")).collect();
  for row in &mut p.design_matrix {
    row.resize(32, 0.0);
  }
  p.sample.as_mut().unwrap().total_observations = 40000;
  p.sample.as_mut().unwrap().dropped_observations = 39940;
  let error = fit_poisson_gee(&p, "id", &Default::default()).unwrap_err();
  assert!(error.to_string().contains("resource bound"), "{error}");
  let p = make(
    vec![1.0; 20002],
    (0..20002).map(|i| vec![i as f64]).collect(),
    (0..20002).map(|i| (i / 2).to_string()).collect(),
  );
  let error = fit_poisson_gee(&p, "id", &Default::default()).unwrap_err();
  assert!(error.to_string().contains("10000 groups"), "{error}");
}

#[test]
fn exact_correlation_boundaries_do_not_become_random_effects_or_clipped_fits() {
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut keys = Vec::new();
  for g in 1..=12 {
    for t in 1..=5 {
      y.push(t as f64);
      x.push(vec![g as f64]);
      keys.push(g.to_string());
    }
  }
  let lower = make(y, x, keys);
  let error = fit_poisson_gee(&lower, "id", &Default::default()).unwrap_err();
  assert!(error.to_string().contains("correlation"), "{error}");
  let mut upper = lower.clone();
  for i in 0..60 {
    upper.response[i] = (i / 5 % 4 + 1) as f64;
    upper.design_matrix[i][0] = (i % 5 + 1) as f64;
  }
  let error = fit_poisson_gee(&upper, "id", &Default::default()).unwrap_err();
  assert!(error.to_string().contains("correlation"), "{error}");
}
