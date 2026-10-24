use tabdat_stats::{
  ConditionalLogitProblem, EstimationProblem, EstimationSample, NestedChoiceProblem,
  NestedLogitOptions, conditional_logit_predict_set, fit_conditional_logit,
  fit_normalized_common_nested_logit, normalized_common_nested_predict_set,
};

fn gold() -> NestedChoiceProblem {
  let cells = [
    ([0., 2., 1., 3.], [1., 0., 1., 0.], 2),
    ([0., 2., 0., 2.], [1., 0., 2., 1.], 3),
    ([0., 2., 0., 2.], [1., 0., 1., 0.], 1),
  ];
  let mut response = Vec::new();
  let mut x = Vec::new();
  let mut cases = Vec::new();
  let mut codes = Vec::new();
  let mut nests = Vec::new();
  // Conditional weights9/16, sqrt inclusive sums5/5k. Exact nest odds
  // 2*3^income/k identify natural utilities ln2/ln3 and common lambda=.5.
  for (p, q, k) in cells {
    for income in 0..=1 {
      let e = 3_usize.pow(income);
      for (chosen, count) in [18 * e, 32 * e, 9 * k, 16 * k].into_iter().enumerate() {
        for _ in 0..count {
          let start = response.len();
          cases.push((start..start + 4).collect());
          for j in 0..4 {
            response.push(f64::from(j == chosen));
            x.push(vec![
              p[j],
              q[j],
              f64::from(j < 2),
              if j < 2 { f64::from(income) } else { 0.0 },
            ]);
            codes.push((j + 1) as i64);
            nests.push(if j < 2 { 1 } else { 2 });
          }
        }
      }
    }
  }
  let n = response.len();
  assert_eq!(n, 3600);
  NestedChoiceProblem {
    choice: ConditionalLogitProblem {
      design: EstimationProblem::new(
        "chosen",
        ["p", "q", "dshore", "dshorey"].map(String::from).to_vec(),
        response,
        x,
      )
      .unwrap()
      .with_intercept(false)
      .with_sample(EstimationSample::new((4..n + 4).collect(), n + 4, 4, None, None).unwrap()),
      group_variable: "id".into(),
      alternative_variable: "alternative".into(),
      case_rows: cases,
      alternative_codes: codes,
    },
    lower_predictor_count: 2,
    nest_variable: "nest".into(),
    nest_codes: nests,
  }
}
fn close(a: f64, b: f64) {
  assert!(
    (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a:.16e} != {b:.16e}"
  );
}

#[test]
fn exact_count_natural_utilities_full_covariance_and_independence_match_99_r_quantities() {
  let p = gold();
  let original = p.clone();
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  let restricted = fit_conditional_logit(&p.choice, &Default::default()).unwrap();
  assert_eq!(fit.observation_count, 3600);
  assert_eq!(fit.choice_set_count, 900);
  assert_eq!(fit.candidate_choice_set_count, 901);
  assert_eq!(fit.excluded_choice_set_count, 1);
  assert_eq!(fit.choice_set_size, 4);
  assert_eq!(fit.chosen_counts, [216, 384, 108, 192]);
  assert_eq!(fit.sample, p.choice.design.sample);
  assert_eq!(fit.alternative_levels, [1, 2, 3, 4]);
  assert_eq!(fit.nest_levels, [1, 2]);
  assert_eq!(fit.alternative_nests, [1, 1, 2, 2]);
  assert_eq!(
    fit.covariance_parameter_names,
    ["p", "q", "dshore", "dshorey", "iv_common"]
  );
  assert!(fit.random_utility_iv_bounds_satisfied);
  assert_eq!(fit.scale_test_null, 1.0);
  assert_eq!(fit.independence_df, 1);
  assert!(fit.converged_starts > 0);
  assert!(fit.max_abs_scaled_score <= 1e-10);
  assert!(fit.max_abs_scaled_newton_step <= 1e-10);
  for (c, b) in fit
    .coefficients
    .iter()
    .zip([2_f64.ln(), 3_f64.ln(), 2_f64.ln(), 3_f64.ln()])
  {
    close(c.value, b);
  }
  close(fit.dissimilarity.value, 0.5);
  close(
    fit.dissimilarity.statistic,
    (0.5 - 1.0) / fit.dissimilarity.standard_error,
  );
  let mut checked = 0;
  for line in include_str!("fixtures/normalized_common_choice_reference.csv")
    .lines()
    .skip(1)
  {
    let c = line.split(',').collect::<Vec<_>>();
    let i = c[2].parse::<usize>().unwrap();
    let j = c[3].parse::<usize>().unwrap();
    let expected = c[4].parse::<f64>().unwrap();
    let normalized = c[0] == "normalized_common";
    let parameter = if i == 0 {
      None
    } else if normalized {
      Some(if i <= 4 {
        &fit.coefficients[i - 1]
      } else {
        &fit.dissimilarity
      })
    } else {
      Some(&restricted.coefficients[i - 1])
    };
    let actual = match c[1] {
      "log_likelihood" => {
        if normalized {
          fit.log_likelihood
        } else {
          restricted.log_likelihood
        }
      }
      "covariance" => {
        if normalized {
          fit.covariance[i - 1][j - 1]
        } else {
          restricted.covariance[i - 1][j - 1]
        }
      }
      "parameter_value" => parameter.unwrap().value,
      "parameter_standard_error" => parameter.unwrap().standard_error,
      "parameter_statistic" => parameter.unwrap().statistic,
      "parameter_p_value" => parameter.unwrap().p_value,
      "parameter_confidence_lower" => parameter.unwrap().confidence_lower,
      "parameter_confidence_upper" => parameter.unwrap().confidence_upper,
      "independence_likelihood_ratio" => fit.independence_likelihood_ratio,
      "independence_p_value" => fit.independence_p_value,
      other => panic!("unknown reference quantity {other}"),
    };
    close(actual, expected);
    if c[1].ends_with("p_value") && expected > 0.0 {
      assert!((actual / expected - 1.0).abs() < 1e-8);
    }
    checked += 1;
  }
  assert_eq!(checked, 99);
  assert_eq!(p, original);
}

#[test]
fn current_probabilities_empirical_odds_common_shift_invariance_and_lambda_one_equivalence() {
  let p = gold();
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  let restricted = fit_conditional_logit(&p.choice, &Default::default()).unwrap();
  let mut independence = fit.clone();
  independence.coefficients = restricted.coefficients.clone();
  independence.dissimilarity.value = 1.0;
  for case in p.choice.case_rows.iter().step_by(47) {
    let rows = case
      .iter()
      .map(|&i| p.choice.design.design_matrix[i].clone())
      .collect::<Vec<_>>();
    let prediction =
      normalized_common_nested_predict_set(&fit, &rows, &[1, 2, 3, 4], &[1, 1, 2, 2]).unwrap();
    let e = 2.0 * 3_f64.powf(rows[0][3]);
    let k = (2_f64.ln() * (rows[2][0] - rows[0][0]) + 3_f64.ln() * (rows[2][1] - rows[0][1])).exp();
    let expected = [9.0 * e, 16.0 * e, 9.0 * k, 16.0 * k].map(|v| v / (25.0 * (e + k)));
    for (j, (&a, b)) in prediction.iter().zip(expected).enumerate() {
      assert!((a - b).abs() < 1e-10);
      assert!((a - fit.fitted_probabilities[case[j]]).abs() < 1e-10);
    }
    assert!((prediction.iter().sum::<f64>() - 1.0).abs() < 1e-14);
    let mut shifted = rows.clone();
    for row in &mut shifted {
      for (x, c) in row.iter_mut().zip([1000.0, -93.0, 7.0, -17.0]) {
        *x += c;
      }
    }
    let shift =
      normalized_common_nested_predict_set(&fit, &shifted, &[1, 2, 3, 4], &[1, 1, 2, 2]).unwrap();
    for (a, b) in shift.iter().zip(&prediction) {
      assert!((a - b).abs() < 1e-10);
    }
    let one =
      normalized_common_nested_predict_set(&independence, &rows, &[1, 2, 3, 4], &[1, 1, 2, 2])
        .unwrap();
    let cl = conditional_logit_predict_set(&restricted, &rows, &[1, 2, 3, 4]).unwrap();
    for (a, b) in one.iter().zip(cl) {
      assert!((a - b).abs() < 1e-12);
    }
  }
}

#[test]
fn fit_is_invariant_to_case_common_design_shifts_and_case_alternative_order() {
  let p = gold();
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  let mut shifted = p.clone();
  for (g, case) in shifted.choice.case_rows.iter_mut().enumerate() {
    for &i in case.iter() {
      for (x, c) in shifted.choice.design.design_matrix[i]
        .iter_mut()
        .zip([3.0, -5.0, 2.0, -1.0])
      {
        *x += c * (g % 7) as f64;
      }
    }
    case.reverse();
  }
  shifted.choice.case_rows.reverse();
  let alternative = fit_normalized_common_nested_logit(&shifted, &Default::default()).unwrap();
  close(fit.log_likelihood, alternative.log_likelihood);
  for (a, b) in fit.coefficients.iter().zip(&alternative.coefficients) {
    close(a.value, b.value);
  }
  close(fit.dissimilarity.value, alternative.dissimilarity.value);
  for (a, b) in fit
    .covariance
    .iter()
    .flatten()
    .zip(alternative.covariance.iter().flatten())
  {
    close(*a, *b);
  }
  for (a, b) in fit
    .fitted_probabilities
    .iter()
    .zip(&alternative.fitted_probabilities)
  {
    assert!((a - b).abs() < 1e-10);
  }
}

#[test]
fn independent_decision_replication_halves_full_joint_covariance() {
  let p = gold();
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  let mut twice = p.clone();
  let n = p.choice.design.response.len();
  twice
    .choice
    .design
    .response
    .extend(&p.choice.design.response);
  twice
    .choice
    .design
    .design_matrix
    .extend(p.choice.design.design_matrix.clone());
  twice.choice.design.sample = None;
  twice
    .choice
    .alternative_codes
    .extend(&p.choice.alternative_codes);
  twice.nest_codes.extend(&p.nest_codes);
  twice.choice.case_rows.extend(
    p.choice
      .case_rows
      .iter()
      .map(|case| case.iter().map(|i| i + n).collect::<Vec<_>>()),
  );
  let replicated = fit_normalized_common_nested_logit(&twice, &Default::default()).unwrap();
  close(replicated.log_likelihood, 2.0 * fit.log_likelihood);
  for (a, b) in replicated
    .covariance
    .iter()
    .flatten()
    .zip(fit.covariance.iter().flatten())
  {
    close(2.0 * a, *b);
  }
  assert_eq!(replicated.choice_set_count, 1800);
}

#[test]
fn malformed_maps_upper_features_dimensions_and_iteration_failure_are_errors() {
  let p = gold();
  for what in 0..10 {
    let mut bad = p.clone();
    match what {
      0 => bad.nest_codes[4] = 2,
      1 => bad.nest_codes.fill(1),
      2 => bad.nest_codes.pop().map(|_| ()).unwrap(),
      3 => bad.choice.design.design_matrix[1][2] = 0.0,
      4 => bad.lower_predictor_count = 0,
      5 => bad.lower_predictor_count = 5,
      6 => bad.choice.design.predictor_names[0] = "iv_common".into(),
      7 => bad.choice.case_rows[1][0] = 0,
      8 => bad.nest_codes[0] = 9_007_199_254_740_992,
      9 => bad.choice.design.design_matrix[0][0] = f64::NAN,
      _ => unreachable!(),
    }
    assert!(
      fit_normalized_common_nested_logit(&bad, &Default::default()).is_err(),
      "case {what}"
    );
  }
  assert!(
    fit_normalized_common_nested_logit(&p, &NestedLogitOptions { max_iterations: 0 })
      .unwrap_err()
      .to_string()
      .contains("iteration limit")
  );
}

#[test]
fn unidentified_common_scale_at_the_conditional_restriction_is_not_a_regular_lr() {
  let mut p = gold();
  // Both alternatives in each nest have identical lower features. Equal-sized
  // nests then contribute the same lambda*log(2), which cancels at the top.
  // Between-nest covariates/upper variables still identify the conditional fit.
  for case in &p.choice.case_rows {
    for (a, b) in [(case[0], case[1]), (case[2], case[3])] {
      for r in 0..2 {
        let average =
          (p.choice.design.design_matrix[a][r] + p.choice.design.design_matrix[b][r]) / 2.0;
        p.choice.design.design_matrix[a][r] = average;
        p.choice.design.design_matrix[b][r] = average;
      }
    }
  }
  assert!(fit_conditional_logit(&p.choice, &Default::default()).is_ok());
  let error = fit_normalized_common_nested_logit(&p, &Default::default())
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("common scale is not locally identified"),
    "{error}"
  );
}

#[test]
fn no_upper_predictors_can_identify_natural_utilities_and_the_shared_scale() {
  let mut p = gold();
  p.choice.design.response.clear();
  p.choice.design.design_matrix.clear();
  p.choice.design.predictor_names = vec!["p".into(), "q".into()];
  p.choice.design.sample = None;
  p.choice.case_rows.clear();
  p.choice.alternative_codes.clear();
  p.nest_codes.clear();
  for (px, qx, k) in [
    ([0., 2., 1., 3.], [1., 0., 1., 0.], 2),
    ([0., 2., 0., 2.], [1., 0., 2., 1.], 3),
    ([0., 2., 0., 2.], [1., 0., 1., 0.], 1),
  ] {
    for (chosen, count) in [9, 16, 9 * k, 16 * k].into_iter().enumerate() {
      for _ in 0..count {
        let start = p.choice.design.response.len();
        p.choice.case_rows.push((start..start + 4).collect());
        for j in 0..4 {
          p.choice.design.response.push(f64::from(j == chosen));
          p.choice.design.design_matrix.push(vec![px[j], qx[j]]);
          p.choice.alternative_codes.push((j + 1) as i64);
          p.nest_codes.push(if j < 2 { 1 } else { 2 });
        }
      }
    }
  }
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  assert_eq!(fit.choice_set_count, 225);
  assert_eq!(fit.covariance.len(), 3);
  close(fit.coefficients[0].value, 2_f64.ln());
  close(fit.coefficients[1].value, 3_f64.ln());
  close(fit.dissimilarity.value, 0.5);
  assert!(fit.random_utility_iv_bounds_satisfied);
}

#[test]
fn prediction_rejects_changed_maps_nonconstant_upper_features_and_invalid_parameters() {
  let p = gold();
  let fit = fit_normalized_common_nested_logit(&p, &Default::default()).unwrap();
  let rows = p.choice.design.design_matrix[..4].to_vec();
  for (codes, nests) in [
    ([1, 2, 3, 3], [1, 1, 2, 2]),
    ([1, 2, 3, 5], [1, 1, 2, 2]),
    ([1, 2, 3, 4], [1, 2, 2, 2]),
  ] {
    assert!(normalized_common_nested_predict_set(&fit, &rows, &codes, &nests).is_err());
  }
  let mut bad = rows.clone();
  bad[1][2] = 0.0;
  assert!(normalized_common_nested_predict_set(&fit, &bad, &[1, 2, 3, 4], &[1, 1, 2, 2]).is_err());
  bad = rows.clone();
  bad[0][0] = f64::INFINITY;
  assert!(normalized_common_nested_predict_set(&fit, &bad, &[1, 2, 3, 4], &[1, 1, 2, 2]).is_err());
  for lambda in [0.0, -1.0, f64::NAN, f64::INFINITY] {
    let mut bad = fit.clone();
    bad.dissimilarity.value = lambda;
    assert!(
      normalized_common_nested_predict_set(&bad, &rows, &[1, 2, 3, 4], &[1, 1, 2, 2]).is_err()
    );
  }
}
