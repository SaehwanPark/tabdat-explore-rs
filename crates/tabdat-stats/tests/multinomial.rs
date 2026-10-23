use tabdat_stats::{
  EstimationProblem, EstimationSample, LogitEstimate, MultinomialOptions, fit_binary_logit,
  fit_multinomial_logit, multinomial_marginal_effects, multinomial_predict_row,
};

fn cell_problem() -> EstimationProblem {
  // Three positive-count cells with exact odds 1 : 2*2^x : 3*2^-x.
  let counts = [[1, 1, 6], [2, 4, 6], [4, 16, 6]];
  let mut response = vec![];
  let mut design = vec![];
  for (g, cell) in counts.iter().enumerate() {
    for (category, &count) in cell.iter().enumerate() {
      for _ in 0..count {
        response.push((category + 1) as f64);
        design.push(vec![g as f64 - 1.0]);
      }
    }
  }
  let n = response.len();
  EstimationProblem::new("choice", vec!["x".into()], response, design)
    .unwrap()
    .with_sample(EstimationSample::new((2..n + 2).collect(), n + 2, 2, None, None).unwrap())
}

fn estimate_field(estimate: &LogitEstimate, field: &str) -> f64 {
  match field {
    "value" => estimate.value,
    "standard_error" => estimate.standard_error,
    "statistic" => estimate.statistic,
    "p_value" => estimate.p_value,
    "confidence_lower" => estimate.confidence_lower,
    "confidence_upper" => estimate.confidence_upper,
    other => panic!("unknown field {other}"),
  }
}

#[test]
fn exact_three_class_cell_odds_match_independent_r_nnet_and_delta_reference() {
  let problem = cell_problem();
  let before = problem.clone();
  let fit = fit_multinomial_logit(&problem, &MultinomialOptions::default()).unwrap();
  let margins = multinomial_marginal_effects(&problem, &fit).unwrap();
  assert_eq!(fit.category_levels, [1, 2, 3]);
  assert_eq!(fit.base_outcome, 1);
  assert_eq!(fit.category_counts, [7, 21, 18]);
  assert_eq!(fit.observation_count, 46);
  assert_eq!(fit.sample, problem.sample);
  assert_eq!(
    fit
      .coefficients
      .iter()
      .map(|c| (c.category, c.estimate.name.as_str()))
      .collect::<Vec<_>>(),
    [(2, "intercept"), (2, "x"), (3, "intercept"), (3, "x")]
  );
  assert_eq!(fit.likelihood_ratio_df, 2);
  assert!(fit.max_abs_scaled_score <= 1e-10);
  assert!(fit.max_abs_scaled_newton_step <= 1e-10);
  let mut checked = 0;
  // R4.6.1/nnet7.3.20: exact cell-odds MLE, package Fisher curvature and
  // independently Richardson-differenced AME gradients; no native replay.
  for row in include_str!("fixtures/multinomial_reference.csv")
    .lines()
    .skip(1)
  {
    let parts = row.split(',').collect::<Vec<_>>();
    let i: usize = parts[1].parse().unwrap();
    let j: usize = parts[2].parse().unwrap();
    let expected: f64 = parts[3].parse().unwrap();
    let actual = if let Some(field) = parts[0].strip_prefix("coefficient_") {
      estimate_field(&fit.coefficients[i].estimate, field)
    } else if let Some(field) = parts[0].strip_prefix("effect_") {
      estimate_field(&margins.effects[i].estimate, field)
    } else {
      match parts[0] {
        "covariance" => fit.covariance[i][j],
        "log_likelihood" => fit.log_likelihood,
        "null_log_likelihood" => fit.null_log_likelihood,
        "likelihood_ratio" => fit.likelihood_ratio,
        "likelihood_ratio_p_value" => fit.likelihood_ratio_p_value.unwrap(),
        "mcfadden_r_squared" => fit.mcfadden_r_squared,
        other => panic!("unknown quantity {other}"),
      }
    };
    assert!(
      (actual - expected).abs() < 1e-9,
      "{}[{i},{j}]: {actual} != {expected}",
      parts[0]
    );
    checked += 1;
  }
  assert_eq!(checked, 63);
  for (row, probabilities) in problem.design_matrix.iter().zip(&fit.fitted_probabilities) {
    let weights = [1.0, 2.0 * 2.0_f64.powf(row[0]), 3.0 * 2.0_f64.powf(-row[0])];
    let denominator: f64 = weights.iter().sum();
    for j in 0..3 {
      assert!((probabilities[j] - weights[j] / denominator).abs() < 1e-11);
    }
    assert!((probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-14);
    let (p, eta) = multinomial_predict_row(&fit, row).unwrap();
    assert_eq!(&p, probabilities);
    assert_eq!(eta[0], 0.0);
  }
  assert!(
    margins
      .effects
      .iter()
      .map(|r| r.estimate.value)
      .sum::<f64>()
      .abs()
      < 1e-14
  );
  assert_eq!(problem, before);
}

#[test]
fn nonlowest_reference_reparameterizes_full_covariance_without_changing_probabilities_or_effects() {
  let problem = cell_problem();
  let first = fit_multinomial_logit(&problem, &MultinomialOptions::default()).unwrap();
  let other = fit_multinomial_logit(
    &problem,
    &MultinomialOptions {
      base_outcome: Some(3),
      ..Default::default()
    },
  )
  .unwrap();
  let transform = [
    [0.0, 0.0, -1.0, 0.0],
    [0.0, 0.0, 0.0, -1.0],
    [1.0, 0.0, -1.0, 0.0],
    [0.0, 1.0, 0.0, -1.0],
  ];
  for a in 0..4 {
    let value: f64 = transform[a]
      .iter()
      .zip(&first.coefficients)
      .map(|(t, c)| t * c.estimate.value)
      .sum();
    assert!((value - other.coefficients[a].estimate.value).abs() < 1e-10);
    for b in 0..4 {
      let mut covariance = 0.0;
      for s in 0..4 {
        for t in 0..4 {
          covariance += transform[a][s] * first.covariance[s][t] * transform[b][t];
        }
      }
      assert!((covariance - other.covariance[a][b]).abs() < 1e-10);
    }
  }
  for (a, b) in first
    .fitted_probabilities
    .iter()
    .flatten()
    .zip(other.fitted_probabilities.iter().flatten())
  {
    assert!((a - b).abs() < 1e-11);
  }
  let first_effects = multinomial_marginal_effects(&problem, &first).unwrap();
  let other_effects = multinomial_marginal_effects(&problem, &other).unwrap();
  for (a, b) in first_effects.effects.iter().zip(&other_effects.effects) {
    assert_eq!(a.category, b.category);
    assert!((a.estimate.value - b.estimate.value).abs() < 1e-10);
    assert!((a.estimate.standard_error - b.estimate.standard_error).abs() < 1e-10);
  }
}

#[test]
fn two_class_reduction_matches_binary_logit_with_and_without_an_intercept() {
  let mut response = vec![];
  let mut design = vec![];
  for (g, counts) in [[2, 1], [2, 2], [1, 2]].iter().enumerate() {
    for (c, &n) in counts.iter().enumerate() {
      for _ in 0..n {
        response.push((c + 1) as f64);
        design.push(vec![g as f64 - 1.0]);
      }
    }
  }
  for intercept in [true, false] {
    let problem = EstimationProblem::new("y", vec!["x".into()], response.clone(), design.clone())
      .unwrap()
      .with_intercept(intercept);
    let mnl = fit_multinomial_logit(&problem, &MultinomialOptions::default()).unwrap();
    let mut binary = problem.clone();
    for y in &mut binary.response {
      *y -= 1.0;
    }
    let logit = fit_binary_logit(&binary).unwrap();
    for (a, b) in mnl.coefficients.iter().zip(&logit.coefficients) {
      assert!((a.estimate.value - b.value).abs() < 1e-9);
      assert!((a.estimate.standard_error - b.standard_error).abs() < 1e-9);
    }
    assert!((mnl.log_likelihood - logit.log_likelihood).abs() < 1e-10);
    assert!((mnl.null_log_likelihood - logit.null_log_likelihood).abs() < 1e-10);
    assert_eq!(
      mnl.likelihood_ratio_df,
      usize::from(intercept) + usize::from(!intercept)
    );
    for (p, q) in mnl
      .fitted_probabilities
      .iter()
      .zip(&logit.fitted_probabilities)
    {
      assert!((p[1] - q).abs() < 1e-10);
    }
  }
}

#[test]
fn domain_rank_convergence_weights_bounds_and_mismatched_postestimation_fail() {
  let problem = cell_problem();
  let fit = fit_multinomial_logit(&problem, &MultinomialOptions::default()).unwrap();
  let mut bad = problem.clone();
  bad.response[0] = 1.5;
  assert!(
    fit_multinomial_logit(&bad, &MultinomialOptions::default())
      .unwrap_err()
      .to_string()
      .contains("integer codes")
  );
  bad.response[0] = 9_007_199_254_740_992.0;
  assert!(fit_multinomial_logit(&bad, &MultinomialOptions::default()).is_err());
  assert!(
    fit_multinomial_logit(
      &problem,
      &MultinomialOptions {
        base_outcome: Some(9),
        ..Default::default()
      }
    )
    .unwrap_err()
    .to_string()
    .contains("absent")
  );
  bad = problem.clone();
  bad.response.fill(1.0);
  assert!(fit_multinomial_logit(&bad, &MultinomialOptions::default()).is_err());
  bad = problem.clone();
  bad.predictor_names.push("duplicate".into());
  for row in &mut bad.design_matrix {
    row.push(row[0]);
  }
  assert!(fit_multinomial_logit(&bad, &MultinomialOptions::default()).is_err());
  for limit in [0, 1] {
    assert!(
      fit_multinomial_logit(
        &problem,
        &MultinomialOptions {
          max_iterations: limit,
          ..Default::default()
        }
      )
      .unwrap_err()
      .to_string()
      .contains("iteration limit")
    );
  }
  bad = problem.clone();
  for (y, row) in bad.response.iter_mut().zip(&bad.design_matrix) {
    *y = if row[0] < 0.0 { 1.0 } else { 2.0 };
  }
  assert!(fit_multinomial_logit(&bad, &MultinomialOptions::default()).is_err());
  bad = problem.clone().with_weights(vec![1.0; 46]);
  assert!(
    fit_multinomial_logit(&bad, &MultinomialOptions::default())
      .unwrap_err()
      .to_string()
      .contains("weights")
  );
  bad = problem.clone();
  bad.predictor_names = (0..32).map(|i| format!("x{i}")).collect();
  for row in &mut bad.design_matrix {
    *row = vec![row[0]; 32];
  }
  assert!(
    fit_multinomial_logit(&bad, &MultinomialOptions::default())
      .unwrap_err()
      .to_string()
      .contains("K<=32")
  );
  assert!(multinomial_predict_row(&fit, &[]).is_err());
  assert!(multinomial_predict_row(&fit, &[f64::NAN]).is_err());
  bad = problem.clone();
  bad.outcome = "other".into();
  assert!(multinomial_marginal_effects(&bad, &fit).is_err());
  let mut malformed = fit;
  malformed.covariance.clear();
  assert!(multinomial_marginal_effects(&problem, &malformed).is_err());
}
