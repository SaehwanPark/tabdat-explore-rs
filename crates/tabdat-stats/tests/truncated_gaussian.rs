use std::collections::BTreeMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, TruncatedGaussianOptions,
  TruncatedGaussianPredictionKind as Kind, TruncatedGaussianResult, fit_truncated_gaussian,
  truncated_gaussian_average_marginal_effects, truncated_gaussian_predict_row,
};
type Reference = BTreeMap<(usize, String, usize, usize), f64>;
fn reference() -> Reference {
  let mut out = BTreeMap::new();
  for row in include_str!("fixtures/truncated_gaussian_reference.csv")
    .lines()
    .skip(1)
  {
    let c: Vec<_> = row.split(',').collect();
    assert!(
      out
        .insert(
          (
            c[0].parse().unwrap(),
            c[1].into(),
            c[2].parse().unwrap(),
            c[3].parse().unwrap()
          ),
          c[4].parse().unwrap()
        )
        .is_none()
    );
  }
  assert_eq!(out.len(), 153);
  out
}
fn get(r: &Reference, case: usize, q: &str, i: usize, j: usize) -> f64 {
  r[&(case, q.into(), i, j)]
}
fn problem(case: usize) -> EstimationProblem {
  let mut y = Vec::new();
  let mut x = Vec::new();
  for line in include_str!("fixtures/truncated_gaussian_data.csv")
    .lines()
    .skip(1)
  {
    let c: Vec<_> = line.split(',').collect();
    if c[0].parse::<usize>().unwrap() == case {
      y.push(c[2].parse().unwrap());
      x.push(if case == 2 {
        vec![c[3].parse().unwrap(), c[4].parse().unwrap()]
      } else {
        vec![c[3].parse().unwrap()]
      });
    }
  }
  assert_eq!(y.len(), 240);
  EstimationProblem::new(
    "y",
    if case == 2 {
      vec!["x".into(), "z".into()]
    } else {
      vec!["x".into()]
    },
    y,
    x,
  )
  .unwrap()
  .with_intercept(case != 2)
  .with_sample(EstimationSample {
    total_observations: 485,
    dropped_observations: 245,
    retained_indices: (0..240).map(|i| 2 * i + 3).collect(),
    weights: None,
    cluster_groups: None,
  })
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.16e} != {b:.16e}"
  );
}
fn tail(a: f64, b: f64) {
  assert!(
    a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite() && (a / b - 1.0).abs() <= 1e-8,
    "tail {a:.16e} != {b:.16e}"
  );
}
fn fit(p: &EstimationProblem, l: f64) -> TruncatedGaussianResult {
  fit_truncated_gaussian(p, l, &TruncatedGaussianOptions::default()).unwrap()
}

#[test]
fn three_independent_truncreg_symbolic_references_cover_joint_covariance_delta_and_all_projections()
{
  let r = reference();
  for case in 1..=3 {
    let p = problem(case);
    let original = p.clone();
    let lower = get(&r, case, "lower_limit", 0, 0);
    let f = fit(&p, lower);
    assert_eq!(p, original);
    assert_eq!(f.sample, p.sample);
    assert_eq!(f.observation_count, 240);
    assert_eq!(f.include_intercept, case != 2);
    assert!(f.max_abs_scaled_score <= 1e-10 && f.max_abs_scaled_newton_step <= 1e-10);
    assert_eq!(
      f.coefficients
        .iter()
        .map(|b| b.name.clone())
        .collect::<Vec<_>>(),
      p.parameter_names()
    );
    for q in [
      "scaled_score",
      "scaled_step",
      "null_scaled_score",
      "null_scaled_step",
    ] {
      assert!(get(&r, case, q, 0, 0).abs() <= 1e-10);
    }
    let effects = truncated_gaussian_average_marginal_effects(&f).unwrap();
    assert_eq!(effects.observation_count, 240);
    assert_eq!(effects.lower_limit, lower);
    assert_eq!(effects.effects.len(), p.predictor_names.len());
    for (prefix, rows) in [("coefficient", &f.coefficients), ("ame", &effects.effects)] {
      for (i, b) in rows.iter().enumerate() {
        for (name, value) in [
          ("value", b.value),
          ("standard_error", b.standard_error),
          ("statistic", b.statistic),
          ("p_value", b.p_value),
          ("confidence_lower", b.confidence_lower),
          ("confidence_upper", b.confidence_upper),
        ] {
          let expected = get(&r, case, &format!("{prefix}_{name}"), i + 1, 0);
          if name == "p_value" {
            tail(value, expected);
          } else {
            close(value, expected);
          }
        }
      }
    }
    for (i, row) in f.covariance.iter().enumerate() {
      for (j, value) in row.iter().enumerate() {
        close(*value, get(&r, case, "joint_covariance", i + 1, j + 1));
      }
    }
    for (i, value) in f.mean_conditional_derivative_gradient.iter().enumerate() {
      close(
        *value,
        get(&r, case, "mean_conditional_derivative_gradient", i + 1, 0),
      );
    }
    for (q, value) in [
      ("log_likelihood", f.log_likelihood),
      ("null_log_likelihood", f.null_log_likelihood),
      ("likelihood_ratio", f.likelihood_ratio),
      ("likelihood_ratio_df", f.likelihood_ratio_df as f64),
      ("sigma", f.scale.value),
      ("log_sigma", f.scale.log_value),
      ("sigma_standard_error", f.scale.standard_error),
      ("log_sigma_standard_error", f.scale.log_standard_error),
      ("sigma_confidence_lower", f.scale.confidence_lower),
      ("sigma_confidence_upper", f.scale.confidence_upper),
      ("mean_conditional_derivative", f.mean_conditional_derivative),
    ] {
      close(value, get(&r, case, q, 0, 0));
    }
    tail(
      f.likelihood_ratio_p_value.unwrap(),
      get(&r, case, "likelihood_ratio_p_value", 0, 0),
    );
    let expected: Vec<_> = include_str!("fixtures/truncated_gaussian_predictions.csv")
      .lines()
      .skip(1)
      .map(|line| line.split(',').collect::<Vec<_>>())
      .filter(|c| c[0].parse::<usize>().unwrap() == case)
      .collect();
    assert_eq!(expected.len(), 240);
    for (i, (row, gold)) in p.design_matrix.iter().zip(expected).enumerate() {
      for (j, kind) in [
        Kind::Latent,
        Kind::RetentionProbability,
        Kind::ConditionalMean,
      ]
      .into_iter()
      .enumerate()
      {
        let value = truncated_gaussian_predict_row(&f, row, kind).unwrap();
        let want = gold[j + 2].parse::<f64>().unwrap();
        if j == 1 {
          assert!((value - want).abs() <= 1e-10);
        } else {
          close(value, want);
        }
        let retained = match kind {
          Kind::Latent => f.fitted_latent_values[i],
          Kind::RetentionProbability => f.fitted_retention_probabilities[i],
          Kind::ConditionalMean => f.fitted_conditional_means[i],
        };
        assert_eq!(value, retained);
      }
    }
  }
}

#[test]
fn outcome_units_and_intercept_shift_preserve_normalized_inference_and_retained_effects() {
  let r = reference();
  let original = problem(1);
  let lower = get(&r, 1, "lower_limit", 0, 0);
  let base = fit(&original, lower);
  let effects = truncated_gaussian_average_marginal_effects(&base).unwrap();
  for (a, offset) in [(1e-9, 0.0), (1000.0, 0.0), (1.0, 1e6)] {
    let mut p = original.clone();
    for y in &mut p.response {
      *y = *y * a + offset;
    }
    let f = fit(&p, lower * a + offset);
    close(f.scale.value / a, base.scale.value);
    close(f.log_likelihood + 240.0 * a.ln(), base.log_likelihood);
    close(
      f.null_log_likelihood + 240.0 * a.ln(),
      base.null_log_likelihood,
    );
    close(f.likelihood_ratio, base.likelihood_ratio);
    for (i, b) in f.coefficients.iter().enumerate() {
      let shift = if i == 0 { offset } else { 0.0 };
      close((b.value - shift) / a, base.coefficients[i].value);
      close(b.standard_error / a, base.coefficients[i].standard_error);
      close(
        (b.confidence_lower - shift) / a,
        base.coefficients[i].confidence_lower,
      );
      close(
        (b.confidence_upper - shift) / a,
        base.coefficients[i].confidence_upper,
      );
    }
    for i in 0..3 {
      for j in 0..3 {
        let scale = if i < 2 { a } else { 1.0 } * if j < 2 { a } else { 1.0 };
        close(f.covariance[i][j] / scale, base.covariance[i][j]);
      }
    }
    let ame = truncated_gaussian_average_marginal_effects(&f).unwrap();
    close(ame.effects[0].value / a, effects.effects[0].value);
    close(
      ame.effects[0].standard_error / a,
      effects.effects[0].standard_error,
    );
    for (i, row) in p.design_matrix.iter().enumerate() {
      close(
        (truncated_gaussian_predict_row(&f, row, Kind::ConditionalMean).unwrap() - offset) / a,
        base.fitted_conditional_means[i],
      );
      close(
        truncated_gaussian_predict_row(&f, row, Kind::RetentionProbability).unwrap(),
        base.fitted_retention_probabilities[i],
      );
    }
  }
}

#[test]
fn no_constant_units_permutation_and_iid_replication_preserve_model_semantics() {
  let r = reference();
  let mut p = problem(2);
  let lower = get(&r, 2, "lower_limit", 0, 0);
  let base = fit(&p, lower);
  for a in [1e-9, 1e5] {
    let mut changed = p.clone();
    for y in &mut changed.response {
      *y *= a;
    }
    let f = fit(&changed, lower * a);
    for (i, b) in f.coefficients.iter().enumerate() {
      close(b.value / a, base.coefficients[i].value);
      close(b.standard_error / a, base.coefficients[i].standard_error);
    }
    close(f.scale.value / a, base.scale.value);
    close(f.likelihood_ratio, base.likelihood_ratio);
  }
  p.response.reverse();
  p.design_matrix.reverse();
  p.sample.as_mut().unwrap().retained_indices.reverse();
  let permuted = fit(&p, lower);
  for (i, b) in permuted.coefficients.iter().enumerate() {
    close(b.value, base.coefficients[i].value);
  }
  let mut repeated = problem(2);
  repeated.response.extend(repeated.response.clone());
  repeated
    .design_matrix
    .extend(repeated.design_matrix.clone());
  repeated.sample = None;
  let doubled = fit(&repeated, lower);
  for (i, b) in doubled.coefficients.iter().enumerate() {
    close(b.value, base.coefficients[i].value);
    close(
      b.standard_error * 2.0_f64.sqrt(),
      base.coefficients[i].standard_error,
    );
  }
  for i in 0..3 {
    for j in 0..3 {
      close(doubled.covariance[i][j] * 2.0, base.covariance[i][j]);
    }
  }
  close(doubled.log_likelihood, 2.0 * base.log_likelihood);
  close(doubled.likelihood_ratio, 2.0 * base.likelihood_ratio);
  close(doubled.scale.value, base.scale.value);
}

#[test]
fn inactive_extreme_bound_is_gaussian_limit_and_not_a_response_scale() {
  let p = problem(3);
  let base = fit(&p, -1e100);
  let f = fit(&p, -1e300);
  for (i, b) in f.coefficients.iter().enumerate() {
    close(b.value, base.coefficients[i].value);
    close(b.standard_error, base.coefficients[i].standard_error);
  }
  close(f.scale.value, base.scale.value);
  close(f.log_likelihood, base.log_likelihood);
  assert_eq!(f.mean_conditional_derivative, 1.0);
  assert!(
    f.mean_conditional_derivative_gradient
      .iter()
      .all(|g| *g == 0.0)
  );
  let effects = truncated_gaussian_average_marginal_effects(&f).unwrap();
  assert_eq!(effects.effects[0].value, f.coefficients[1].value);
  assert_eq!(
    effects.effects[0].standard_error,
    f.coefficients[1].standard_error
  );
  for (row, latent) in p.design_matrix.iter().zip(&f.fitted_latent_values) {
    assert_eq!(
      truncated_gaussian_predict_row(&f, row, Kind::RetentionProbability).unwrap(),
      1.0
    );
    assert_eq!(
      truncated_gaussian_predict_row(&f, row, Kind::ConditionalMean).unwrap(),
      *latent
    );
  }
}

#[test]
fn invalid_support_resources_rank_and_boundary_fits_fail_without_mutating_problem() {
  let r = reference();
  let base = problem(1);
  let lower = get(&r, 1, "lower_limit", 0, 0);
  for value in [lower, lower - 1.0, f64::NAN] {
    let mut p = base.clone();
    p.response[0] = value;
    assert!(fit_truncated_gaussian(&p, lower, &TruncatedGaussianOptions::default()).is_err());
    if value.is_finite() {
      assert_eq!(p.response[0], value);
    }
  }
  for limit in [f64::NAN, f64::INFINITY] {
    assert!(fit_truncated_gaussian(&base, limit, &TruncatedGaussianOptions::default()).is_err());
  }
  for max_iterations in [0, 1] {
    assert!(
      fit_truncated_gaussian(&base, lower, &TruncatedGaussianOptions { max_iterations }).is_err()
    );
  }
  let mut weighted = base.clone();
  weighted.weights = Some(vec![1.0; 240]);
  assert!(fit_truncated_gaussian(&weighted, lower, &TruncatedGaussianOptions::default()).is_err());
  let mut names = base.clone();
  names.predictor_names[0] = "lnsigma".into();
  assert!(fit_truncated_gaussian(&names, lower, &TruncatedGaussianOptions::default()).is_err());
  let mut rank = base.clone();
  rank.predictor_names.push("duplicate".into());
  for row in &mut rank.design_matrix {
    row.push(row[0]);
  }
  assert!(fit_truncated_gaussian(&rank, lower, &TruncatedGaussianOptions::default()).is_err());
  let mut sample = base.clone();
  sample.sample.as_mut().unwrap().retained_indices[0] =
    sample.sample.as_ref().unwrap().retained_indices[1];
  assert!(fit_truncated_gaussian(&sample, lower, &TruncatedGaussianOptions::default()).is_err());
  let large = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0; 10001],
    vec![vec![1.0]; 10001],
  )
  .unwrap();
  assert!(fit_truncated_gaussian(&large, 0.0, &TruncatedGaussianOptions::default()).is_err());
  let wide = EstimationProblem::new(
    "y",
    (0..33).map(|i| format!("x{i}")).collect(),
    vec![1.0; 240],
    vec![vec![1.0; 33]; 240],
  )
  .unwrap();
  assert!(fit_truncated_gaussian(&wide, 0.0, &TruncatedGaussianOptions::default()).is_err());
  let mut constant = base.clone();
  constant.response.fill(2.0);
  assert!(fit_truncated_gaussian(&constant, 0.0, &TruncatedGaussianOptions::default()).is_err());
  // Positive exponential closure: empirical second moment exceeds twice the
  // squared mean, so the concave precision likelihood maximizes at v=0.
  let mut y = vec![0.01; 40];
  y[39] = 100.0;
  let boundary = EstimationProblem::new("y", vec![], y, vec![vec![]; 40]).unwrap();
  assert!(fit_truncated_gaussian(&boundary, 0.0, &TruncatedGaussianOptions::default()).is_err());
  assert_eq!(base, problem(1));
}

#[test]
fn projection_modes_do_not_need_outcome_or_unnecessary_mills_and_validate_shapes() {
  let mut f = fit(&problem(1), -0.6);
  f.coefficients[0].value = -1e6;
  f.coefficients[1].value = 0.0;
  f.scale.value = 1.0;
  f.lower_limit = 0.0;
  assert_eq!(
    truncated_gaussian_predict_row(&f, &[0.0], Kind::Latent).unwrap(),
    -1e6
  );
  assert_eq!(
    truncated_gaussian_predict_row(&f, &[0.0], Kind::RetentionProbability).unwrap(),
    0.0
  );
  assert!(truncated_gaussian_predict_row(&f, &[0.0], Kind::ConditionalMean).is_err());
  assert!(truncated_gaussian_predict_row(&f, &[], Kind::Latent).is_err());
  assert!(truncated_gaussian_predict_row(&f, &[f64::NAN], Kind::Latent).is_err());
  f.scale.value = 0.0;
  assert!(truncated_gaussian_predict_row(&f, &[0.0], Kind::Latent).is_err());
  let mut malformed = fit(&problem(1), -0.6);
  malformed.mean_conditional_derivative_gradient.pop();
  assert!(truncated_gaussian_average_marginal_effects(&malformed).is_err());
}
