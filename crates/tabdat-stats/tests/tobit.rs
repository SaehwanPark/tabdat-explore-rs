use std::collections::BTreeMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, TobitOptions, fit_tobit, tobit_average_marginal_effects,
  tobit_predict_row,
};

type Reference = BTreeMap<(usize, String, usize, usize), f64>;
fn reference() -> Reference {
  let mut map = BTreeMap::new();
  for row in include_str!("fixtures/tobit_reference.csv").lines().skip(1) {
    let fields: Vec<_> = row.split(',').collect();
    assert_eq!(fields.len(), 5);
    let key = (
      fields[0].parse().unwrap(),
      fields[1].to_string(),
      fields[2].parse().unwrap(),
      fields[3].parse().unwrap(),
    );
    assert!(map.insert(key, fields[4].parse().unwrap()).is_none());
  }
  assert!(!map.is_empty());
  map
}
fn get(r: &Reference, model: usize, q: &str, i: usize, j: usize) -> f64 {
  *r.get(&(model, q.to_string(), i, j))
    .unwrap_or_else(|| panic!("missing reference {model}/{q}/{i}/{j}"))
}
fn problem(r: &Reference, model: usize) -> EstimationProblem {
  let y = (1..=36).map(|i| get(r, model, "input_y", i, 0)).collect();
  let x = (1..=36)
    .map(|i| (1..=2).map(|j| get(r, model, "input_x", i, j)).collect())
    .collect();
  EstimationProblem::new("y", vec!["x1".into(), "x2".into()], y, x)
    .unwrap()
    .with_intercept(get(r, model, "intercept", 0, 0) == 1.0)
}
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.16e} != {b:.16e}"
  );
}
fn tail(a: f64, b: f64) {
  assert!(
    a > 0.0 && b > 0.0 && a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-8 * b,
    "tail {a:.16e} != {b:.16e}"
  );
}

#[test]
fn five_independent_survreg_deriv3_references_include_full_scale_cross_covariance() {
  let r = reference();
  for model in 1..=5 {
    let p = problem(&r, model);
    let original = p.clone();
    let lower = get(&r, model, "lower", 0, 0);
    let f = fit_tobit(&p, lower, &TobitOptions::default()).unwrap();
    assert_eq!(p, original);
    assert_eq!(f.observation_count, 36);
    assert_eq!(
      f.left_censored_count,
      get(&r, model, "censored", 0, 0) as usize
    );
    assert_eq!(
      f.uncensored_count,
      get(&r, model, "uncensored", 0, 0) as usize
    );
    assert!(f.max_abs_scaled_score <= 1e-10 && f.max_abs_scaled_newton_step <= 1e-10);
    assert_eq!(
      f.coefficients
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>(),
      p.parameter_names()
    );
    for (j, c) in f.coefficients.iter().enumerate() {
      for (name, value) in [
        ("value", c.value),
        ("standard_error", c.standard_error),
        ("statistic", c.statistic),
        ("confidence_lower", c.confidence_lower),
        ("confidence_upper", c.confidence_upper),
      ] {
        close(
          value,
          get(&r, model, &format!("coefficient_{name}"), j + 1, 0),
        );
      }
      tail(c.p_value, get(&r, model, "coefficient_p_value", j + 1, 0));
    }
    for (i, row) in f.covariance.iter().enumerate() {
      for (j, value) in row.iter().enumerate() {
        close(*value, get(&r, model, "covariance", i + 1, j + 1));
      }
    }
    for (q, v) in [
      ("log_likelihood", f.log_likelihood),
      ("null_log_likelihood", f.null_log_likelihood),
      ("likelihood_ratio", f.likelihood_ratio),
      ("sigma", f.scale.value),
      ("log_sigma", f.scale.log_value),
      ("sigma_standard_error", f.scale.standard_error),
      ("log_sigma_standard_error", f.scale.log_standard_error),
      ("sigma_confidence_lower", f.scale.confidence_lower),
      ("sigma_confidence_upper", f.scale.confidence_upper),
      ("mean_probability", f.mean_uncensored_probability),
    ] {
      close(v, get(&r, model, q, 0, 0));
    }
    assert_eq!(f.likelihood_ratio_df, 2);
    tail(
      f.likelihood_ratio_p_value.unwrap(),
      get(&r, model, "likelihood_ratio_p_value", 0, 0),
    );
    for (j, v) in f.mean_probability_gradient.iter().enumerate() {
      close(*v, get(&r, model, "mean_probability_gradient", j + 1, 0));
    }
    for i in 0..36 {
      let point = tobit_predict_row(&f, &p.design_matrix[i]).unwrap();
      for (q, a, b) in [
        ("latent", point.0, f.fitted_latent_values[i]),
        ("probability", point.1, f.fitted_uncensored_probabilities[i]),
        ("mean", point.2, f.fitted_observed_means[i]),
      ] {
        close(a, b);
        close(a, get(&r, model, q, i + 1, 0));
      }
    }
    let effects = tobit_average_marginal_effects(&f).unwrap();
    assert_eq!(effects.observation_count, 36);
    for (j, c) in effects.effects.iter().enumerate() {
      assert_eq!(c.name, p.predictor_names[j]);
      for (name, value) in [
        ("value", c.value),
        ("standard_error", c.standard_error),
        ("statistic", c.statistic),
        ("confidence_lower", c.confidence_lower),
        ("confidence_upper", c.confidence_upper),
      ] {
        close(value, get(&r, model, &format!("ame_{name}"), j + 1, 0));
      }
      tail(c.p_value, get(&r, model, "ame_p_value", j + 1, 0));
    }
  }
}

#[test]
fn response_scale_bound_shift_order_replication_and_retained_sample_contracts() {
  let r = reference();
  let p = problem(&r, 1);
  let f = fit_tobit(&p, 0.0, &TobitOptions::default()).unwrap();
  let mut reordered = p.clone();
  reordered.response.reverse();
  reordered.design_matrix.reverse();
  let reversed = fit_tobit(&reordered, 0.0, &TobitOptions::default()).unwrap();
  for (a, b) in f.coefficients.iter().zip(&reversed.coefficients) {
    close(a.value, b.value);
    close(a.standard_error, b.standard_error);
  }
  let mut repeated = p.clone();
  repeated.response.extend(p.response.clone());
  repeated.design_matrix.extend(p.design_matrix.clone());
  repeated.sample = Some(EstimationSample::all_retained(72));
  let twice = fit_tobit(&repeated, 0.0, &TobitOptions::default()).unwrap();
  assert_eq!(twice.sample, repeated.sample);
  close(twice.log_likelihood, 2.0 * f.log_likelihood);
  for (i, row) in f.covariance.iter().enumerate() {
    for (j, v) in row.iter().enumerate() {
      close(twice.covariance[i][j], v / 2.0);
    }
  }
  let shifted = fit_tobit(&problem(&r, 4), 5.0, &TobitOptions::default()).unwrap();
  close(shifted.coefficients[0].value, f.coefficients[0].value + 5.0);
  close(shifted.scale.value, f.scale.value);
  close(shifted.log_likelihood, f.log_likelihood);
  let scaled = fit_tobit(&problem(&r, 5), 0.0, &TobitOptions::default()).unwrap();
  for (a, b) in f.coefficients.iter().zip(&scaled.coefficients) {
    close(b.value, a.value * 1000.0);
    close(b.standard_error, a.standard_error * 1000.0);
  }
  close(scaled.scale.value, f.scale.value * 1000.0);
  close(scaled.likelihood_ratio, f.likelihood_ratio);
  close(
    scaled.log_likelihood,
    f.log_likelihood - f.uncensored_count as f64 * 1000.0_f64.ln(),
  );
  for (a, b) in f
    .fitted_uncensored_probabilities
    .iter()
    .zip(&scaled.fitted_uncensored_probabilities)
  {
    close(*a, *b);
  }
}

#[test]
fn explicit_invalid_design_iteration_scale_and_inference_failures() {
  let r = reference();
  let p = problem(&r, 1);
  let o = TobitOptions::default();
  let mut bad = p.clone();
  bad.response.fill(0.0);
  assert!(fit_tobit(&bad, 0.0, &o).is_err());
  let mut bad = p.clone();
  for row in &mut bad.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_tobit(&bad, 0.0, &o).is_err());
  let mut bad = p.clone();
  bad.predictor_names[1] = bad.predictor_names[0].clone();
  assert!(fit_tobit(&bad, 0.0, &o).is_err());
  let mut bad = p.clone();
  bad.weights = Some(vec![1.0; 36]);
  assert!(fit_tobit(&bad, 0.0, &o).is_err());
  let mut bad = p.clone();
  let mut sample = EstimationSample::all_retained(36);
  sample.cluster_groups = Some(vec!["one".into(); 36]);
  bad.sample = Some(sample);
  assert!(fit_tobit(&bad, 0.0, &o).is_err());
  assert!(fit_tobit(&p, f64::NAN, &o).is_err());
  assert!(fit_tobit(&p, 0.0, &TobitOptions { max_iterations: 0 }).is_err());
  assert!(fit_tobit(&p, 0.0, &TobitOptions { max_iterations: 1 }).is_err());
  let perfect = EstimationProblem::new(
    "y",
    vec!["x".into()],
    (0..20).map(|i| 3.0 + i as f64).collect(),
    (0..20).map(|i| vec![i as f64]).collect(),
  )
  .unwrap();
  assert!(fit_tobit(&perfect, -1.0, &o).is_err());
  let f = fit_tobit(&p, 0.0, &o).unwrap();
  assert!(tobit_predict_row(&f, &[1.0]).is_err());
  assert!(tobit_predict_row(&f, &[1.0, f64::NAN]).is_err());
  let mut bad = f.clone();
  bad.mean_probability_gradient.pop();
  assert!(tobit_average_marginal_effects(&bad).is_err());
  let mut bad = f.clone();
  bad.covariance[0].pop();
  assert!(tobit_average_marginal_effects(&bad).is_err());
}

#[test]
fn current_prediction_and_continuous_mean_derivative_are_not_latent_slopes() {
  let r = reference();
  let p = problem(&r, 1);
  let f = fit_tobit(&p, 0.0, &TobitOptions::default()).unwrap();
  let row = [0.3, -0.2];
  let h = 1e-5;
  let (latent, probability, mean) = tobit_predict_row(&f, &row).unwrap();
  assert!(mean >= 0.0 && (0.0..=1.0).contains(&probability));
  assert_ne!(latent, mean);
  for j in 0..2 {
    let mut plus = row;
    let mut minus = row;
    plus[j] += h;
    minus[j] -= h;
    let derivative = (tobit_predict_row(&f, &plus).unwrap().2
      - tobit_predict_row(&f, &minus).unwrap().2)
      / (2.0 * h);
    close(derivative, f.coefficients[j + 1].value * probability);
  }
}
