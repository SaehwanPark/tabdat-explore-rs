use tabdat_stats::{
  ConditionalLogitProblem, EstimationProblem, EstimationSample, NonNormalizedNestedLogitProblem,
  conditional_logit_predict_set, fit_conditional_logit, fit_non_normalized_nested_logit,
  non_normalized_nested_predict_set,
};

fn gold() -> NonNormalizedNestedLogitProblem {
  let cells = [
    ([0., 1., 0., 0.], [0., 0., 0., 1.], [6, 12, 16, 48]),
    ([0., 0., 0., 1.], [0., 1., 0., 0.], [8, 24, 9, 18]),
    ([1., 0., 0., 1.], [0., 0., 1., 0.], [12, 6, 75, 50]),
  ];
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut cases = Vec::new();
  let mut codes = Vec::new();
  let mut nests = Vec::new();
  // Exact cell odds: lower weights are integer powers of2/3, upper shore
  // weight is2*3^income*S^2 and boat weight B^3. Every count is positive.
  for (p, q, counts) in cells {
    for income in 0..=1 {
      for (chosen, count) in counts.into_iter().enumerate() {
        let count = count * if chosen < 2 { 3_usize.pow(income) } else { 1 };
        for _ in 0..count {
          let start = y.len();
          cases.push((start..start + 4).collect());
          for j in 0..4 {
            y.push(f64::from(j == chosen));
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
  let n = y.len();
  assert_eq!(n, 2816);
  NonNormalizedNestedLogitProblem {
    choice: ConditionalLogitProblem {
      design: EstimationProblem::new(
        "chosen",
        vec!["p".into(), "q".into(), "dshore".into(), "dshorey".into()],
        y,
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
fn exact_count_mle_full_observed_covariance_and_restriction_match_independent_r() {
  let p = gold();
  let original = p.clone();
  let fit = fit_non_normalized_nested_logit(&p, &Default::default()).unwrap();
  let restricted = fit_conditional_logit(&p.choice, &Default::default()).unwrap();
  assert_eq!(fit.choice_set_count, 704);
  assert_eq!(fit.observation_count, 2816);
  assert_eq!(fit.choice_set_size, 4);
  assert_eq!(fit.candidate_choice_set_count, 705);
  assert_eq!(fit.excluded_choice_set_count, 1);
  assert_eq!(fit.chosen_counts, vec![104, 168, 200, 232]);
  assert_eq!(fit.sample, p.choice.design.sample);
  assert_eq!(fit.nest_levels, vec![1, 2]);
  assert_eq!(fit.alternative_nests, vec![1, 1, 2, 2]);
  assert_eq!(
    fit.covariance_parameter_names,
    vec!["p", "q", "dshore", "dshorey", "iv[1]", "iv[2]"]
  );
  assert!(!fit.random_utility_iv_bounds_satisfied);
  assert!(fit.converged_starts > 0);
  assert!(fit.max_abs_scaled_score <= 1e-10);
  assert!(fit.max_abs_scaled_newton_step <= 1e-10);
  for (c, expected) in fit
    .coefficients
    .iter()
    .zip([2_f64.ln(), 3_f64.ln(), 2_f64.ln(), 3_f64.ln()])
  {
    assert!((c.value - expected).abs() < 1e-9);
  }
  for (d, expected) in fit.dissimilarities.iter().zip([2., 3.]) {
    assert!((d.estimate.value - expected).abs() < 1e-9);
    assert_eq!(d.test_null, 1.0);
    close(
      d.estimate.statistic,
      (expected - 1.) / d.estimate.standard_error,
    );
  }
  let mut checked = 0;
  for line in include_str!("fixtures/nested_choice_reference.csv")
    .lines()
    .skip(1)
  {
    let c = line.split(',').collect::<Vec<_>>();
    let i = c[2].parse::<usize>().unwrap();
    let j = c[3].parse::<usize>().unwrap();
    let expected = c[4].parse::<f64>().unwrap();
    let nested = c[0] == "nested";
    let parameter = if i == 0 {
      None
    } else if nested {
      if i <= 4 {
        Some(&fit.coefficients[i - 1])
      } else {
        Some(&fit.dissimilarities[i - 5].estimate)
      }
    } else {
      Some(&restricted.coefficients[i - 1])
    };
    let actual = match c[1] {
      "log_likelihood" => {
        if nested {
          fit.log_likelihood
        } else {
          restricted.log_likelihood
        }
      }
      "covariance" => {
        if nested {
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
      name => panic!("unrecognized reference quantity {name}"),
    };
    close(actual, expected);
    if c[1].ends_with("p_value") && expected > 0.0 {
      assert!((actual / expected - 1.0).abs() < 1e-8);
    }
    checked += 1;
  }
  assert_eq!(checked, 116);
  close(fit.restricted_log_likelihood, restricted.log_likelihood);
  close(
    fit.independence_likelihood_ratio,
    2.0 * (fit.log_likelihood - restricted.log_likelihood),
  );
  assert_eq!(fit.independence_df, 2);
  for case in &p.choice.case_rows {
    let row = &p.choice.design.design_matrix[case[0]];
    let income = row[3];
    let weights = case
      .iter()
      .map(|&i| {
        let row = &p.choice.design.design_matrix[i];
        2_f64.powf(row[0]) * 3_f64.powf(row[1])
      })
      .collect::<Vec<_>>();
    let s = weights[0] + weights[1];
    let b = weights[2] + weights[3];
    let top_s = 2. * 3_f64.powf(income) * s * s;
    let top_b = b * b * b;
    for (j, &i) in case.iter().enumerate() {
      close(
        fit.fitted_probabilities[i],
        weights[j] / if j < 2 { s } else { b } * if j < 2 { top_s } else { top_b }
          / (top_s + top_b),
      );
    }
  }
  assert_eq!(p, original);
}

#[test]
fn current_predictions_lambda_one_equivalence_and_non_normalized_common_shift_semantics() {
  let p = gold();
  let fit = fit_non_normalized_nested_logit(&p, &Default::default()).unwrap();
  let rows = p.choice.design.design_matrix[..4].to_vec();
  let codes = [1, 2, 3, 4];
  let nests = [1, 1, 2, 2];
  let baseline = non_normalized_nested_predict_set(&fit, &rows, &codes, &nests).unwrap();
  close(baseline.iter().sum(), 1.0);
  for (a, b) in baseline.iter().zip(&fit.fitted_probabilities[..4]) {
    close(*a, *b);
  }
  let shifted = rows
    .iter()
    .map(|r| {
      let mut r = r.clone();
      r[0] += 1.;
      r
    })
    .collect::<Vec<_>>();
  let change = non_normalized_nested_predict_set(&fit, &shifted, &codes, &nests).unwrap();
  close(
    (change[0] + change[1]) / (change[2] + change[3]),
    0.5 * (baseline[0] + baseline[1]) / (baseline[2] + baseline[3]),
  );
  close(change[0] / change[1], baseline[0] / baseline[1]);
  close(change[2] / change[3], baseline[2] / baseline[3]);
  let mut unit = fit.clone();
  for d in &mut unit.dissimilarities {
    d.estimate.value = 1.;
  }
  let mut conditional = fit_conditional_logit(&p.choice, &Default::default()).unwrap();
  conditional.coefficients = fit.coefficients.clone();
  let nested = non_normalized_nested_predict_set(&unit, &rows, &codes, &nests).unwrap();
  let expected = conditional_logit_predict_set(&conditional, &rows, &codes).unwrap();
  for (a, b) in nested.iter().zip(expected) {
    close(*a, b);
  }
  let perm = [3, 0, 2, 1];
  let permuted = perm.iter().map(|&i| rows[i].clone()).collect::<Vec<_>>();
  let predicted = non_normalized_nested_predict_set(
    &fit,
    &permuted,
    &perm.map(|i| codes[i]),
    &perm.map(|i| nests[i]),
  )
  .unwrap();
  for (i, &j) in perm.iter().enumerate() {
    close(predicted[i], baseline[j]);
  }
  let mut bad = rows.clone();
  bad[0][3] += 1.;
  assert!(non_normalized_nested_predict_set(&fit, &bad, &codes, &nests).is_err());
  assert!(non_normalized_nested_predict_set(&fit, &rows, &codes, &[2, 1, 2, 2]).is_err());
  assert!(non_normalized_nested_predict_set(&fit, &rows[..3], &codes[..3], &nests[..3]).is_err());
  bad[0][0] = f64::NAN;
  assert!(non_normalized_nested_predict_set(&fit, &bad, &codes, &nests).is_err());
}

#[test]
fn invalid_layout_upper_design_domains_and_nonidentification_fail_without_mutation() {
  let p = gold();
  for mutation in 0..9 {
    let mut bad = p.clone();
    match mutation {
      0 => bad.nest_codes[0] = 2,
      1 => bad.nest_codes.fill(1),
      2 => bad.choice.design.design_matrix[0][2] = 0.,
      3 => bad.lower_predictor_count = 0,
      4 => bad.choice.design.include_intercept = true,
      5 => bad.choice.design.response[0] = 0.,
      6 => bad.nest_codes[0] = i64::MAX,
      7 => {
        for row in &mut bad.choice.design.design_matrix {
          row[0] = 1.;
        }
      }
      8 => bad.choice.case_rows[1][0] = 0,
      _ => unreachable!(),
    }
    let before = bad.clone();
    assert!(
      fit_non_normalized_nested_logit(&bad, &Default::default()).is_err(),
      "mutation{mutation}"
    );
    assert_eq!(bad, before);
  }
  assert!(
    fit_non_normalized_nested_logit(&p, &tabdat_stats::NestedLogitOptions { max_iterations: 0 })
      .is_err()
  );
}

#[test]
fn nonidentified_scales_at_the_conditional_restriction_do_not_get_a_regular_lr_reference() {
  let mut p = gold();
  p.lower_predictor_count = 1;
  p.choice.design.predictor_names = ["p", "dshore", "dshorey"].map(String::from).to_vec();
  for (row, &alternative) in p
    .choice
    .design
    .design_matrix
    .iter_mut()
    .zip(&p.choice.alternative_codes)
  {
    *row = vec![
      f64::from(alternative == 2 || alternative == 4),
      row[2],
      row[3],
    ];
  }
  // Conditional slopes remain identified, but the nest inclusive values are
  // case-constant and their scale effects alias the upper nest intercept.
  fit_conditional_logit(&p.choice, &Default::default()).unwrap();
  let error = fit_non_normalized_nested_logit(&p, &Default::default()).unwrap_err();
  assert!(
    error.to_string().contains("independence restriction"),
    "{error}"
  );
}

#[test]
fn input_row_case_permutations_and_replicated_decisions_preserve_units_and_covariance() {
  let p = gold();
  let fit = fit_non_normalized_nested_logit(&p, &Default::default()).unwrap();
  let mut permuted = p.clone();
  let n = p.choice.design.response.len();
  permuted.choice.design.sample = None;
  permuted.choice.design.response.reverse();
  permuted.choice.design.design_matrix.reverse();
  permuted.choice.alternative_codes.reverse();
  permuted.nest_codes.reverse();
  for case in &mut permuted.choice.case_rows {
    for row in case {
      *row = n - 1 - *row;
    }
  }
  permuted.choice.case_rows.reverse();
  let alternative = fit_non_normalized_nested_logit(&permuted, &Default::default()).unwrap();
  for (a, b) in fit.coefficients.iter().zip(&alternative.coefficients) {
    close(a.value, b.value);
  }
  for (a, b) in fit
    .covariance
    .iter()
    .flatten()
    .zip(alternative.covariance.iter().flatten())
  {
    close(*a, *b);
  }
  let mut doubled = p.clone();
  doubled.choice.design.sample = None;
  doubled
    .choice
    .design
    .response
    .extend_from_slice(&p.choice.design.response);
  doubled
    .choice
    .design
    .design_matrix
    .extend_from_slice(&p.choice.design.design_matrix);
  doubled
    .choice
    .alternative_codes
    .extend_from_slice(&p.choice.alternative_codes);
  doubled.nest_codes.extend_from_slice(&p.nest_codes);
  doubled.choice.case_rows.extend(
    p.choice
      .case_rows
      .iter()
      .map(|case| case.iter().map(|i| i + n).collect::<Vec<_>>()),
  );
  let replicated = fit_non_normalized_nested_logit(&doubled, &Default::default()).unwrap();
  assert_eq!(replicated.choice_set_count, 1408);
  close(replicated.log_likelihood, 2. * fit.log_likelihood);
  for (a, b) in fit
    .covariance
    .iter()
    .flatten()
    .zip(replicated.covariance.iter().flatten())
  {
    close(*a, 2. * b);
  }
}
