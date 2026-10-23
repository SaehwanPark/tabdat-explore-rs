use tabdat_stats::{
  ConditionalLogitOptions, ConditionalLogitProblem, EstimationProblem, EstimationSample,
  conditional_logit_predict_set, fit_binary_logit, fit_conditional_logit,
};

fn dummy_problem(counts: &[usize]) -> ConditionalLogitProblem {
  let j = counts.len();
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut cases = Vec::new();
  let mut codes = Vec::new();
  for (chosen, &count) in counts.iter().enumerate() {
    for _ in 0..count {
      let start = y.len();
      cases.push((start..start + j).collect());
      for alternative in 0..j {
        y.push(f64::from(alternative == chosen));
        codes.push((alternative + 1) as i64);
        x.push((1..j).map(|r| f64::from(alternative == r)).collect());
      }
    }
  }
  let n = y.len();
  let sample =
    EstimationSample::new((2 * j..n + 2 * j).collect(), n + 2 * j, 2 * j, None, None).unwrap();
  ConditionalLogitProblem {
    design: EstimationProblem::new(
      "chosen",
      (1..j).map(|r| format!("alternative{}", r + 1)).collect(),
      y,
      x,
    )
    .unwrap()
    .with_intercept(false)
    .with_sample(sample),
    group_variable: "case".into(),
    alternative_variable: "alternative".into(),
    case_rows: cases,
    alternative_codes: codes,
  }
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn two_three_four_alternative_counts_match_closed_form_conditional_mle_and_joint_information() {
  for counts in [vec![3, 6], vec![2, 4, 6], vec![2, 4, 6, 8]] {
    let problem = dummy_problem(&counts);
    let original = problem.clone();
    let fit = fit_conditional_logit(&problem, &ConditionalLogitOptions::default()).unwrap();
    let g = counts.iter().sum::<usize>();
    let j = counts.len();
    assert_eq!(fit.choice_set_count, g);
    assert_eq!(fit.choice_set_size, j);
    assert_eq!(fit.observation_count, g * j);
    assert_eq!(fit.candidate_choice_set_count, g + 2);
    assert_eq!(fit.excluded_choice_set_count, 2);
    assert_eq!(fit.chosen_counts, counts);
    assert_eq!(fit.sample, problem.design.sample);
    assert_eq!(fit.likelihood_ratio_df, j - 1);
    assert!(fit.max_abs_scaled_score <= 1e-10);
    assert!(fit.max_abs_scaled_newton_step <= 1e-10);
    // Independent closed form for saturated alternative constants: beta_j is
    // log(n_j/n_1); covariance is diag(1/n_j)+(1/n_1)11', not row-based HC1.
    for r in 0..j - 1 {
      close(
        fit.coefficients[r].value,
        (counts[r + 1] as f64 / counts[0] as f64).ln(),
      );
      for s in 0..j - 1 {
        close(
          fit.covariance[r][s],
          1.0 / counts[0] as f64
            + if r == s {
              1.0 / counts[r + 1] as f64
            } else {
              0.0
            },
        );
      }
    }
    let ll = counts
      .iter()
      .map(|&n| n as f64 * (n as f64 / g as f64).ln())
      .sum::<f64>();
    close(fit.log_likelihood, ll);
    close(fit.null_log_likelihood, -(g as f64) * (j as f64).ln());
    for (row, &code) in fit
      .fitted_probabilities
      .iter()
      .zip(&problem.alternative_codes)
    {
      close(*row, counts[code as usize - 1] as f64 / g as f64);
    }
    let first = &problem.case_rows[0];
    let predictors = first
      .iter()
      .map(|&i| problem.design.design_matrix[i].clone())
      .collect::<Vec<_>>();
    let predicted =
      conditional_logit_predict_set(&fit, &predictors, &problem.alternative_codes[..j]).unwrap();
    close(predicted.iter().sum(), 1.0);
    assert_eq!(problem, original);
  }
}

#[test]
fn within_case_shifts_permutations_and_replicated_decisions_preserve_correct_units() {
  let p = dummy_problem(&[2, 4, 6, 8]);
  let fit = fit_conditional_logit(&p, &Default::default()).unwrap();
  let mut shifted = p.clone();
  for (case, rows) in shifted.case_rows.iter().enumerate() {
    for &i in rows {
      for (r, x) in shifted.design.design_matrix[i].iter_mut().enumerate() {
        *x += 2.0 * (case + r) as f64;
      }
    }
  }
  let shifted_fit = fit_conditional_logit(&shifted, &Default::default()).unwrap();
  for (a, b) in fit.coefficients.iter().zip(&shifted_fit.coefficients) {
    close(a.value, b.value);
    close(a.standard_error, b.standard_error);
  }
  for rows in &shifted.case_rows {
    let x = rows
      .iter()
      .map(|&i| shifted.design.design_matrix[i].clone())
      .collect::<Vec<_>>();
    let codes = rows
      .iter()
      .map(|&i| shifted.alternative_codes[i])
      .collect::<Vec<_>>();
    for (&i, prediction) in rows
      .iter()
      .zip(conditional_logit_predict_set(&fit, &x, &codes).unwrap())
    {
      close(prediction, fit.fitted_probabilities[i]);
    }
  }
  let n = p.design.response.len();
  let mut permuted = p.clone();
  permuted.design.response.reverse();
  permuted.design.design_matrix.reverse();
  permuted.alternative_codes.reverse();
  for case in &mut permuted.case_rows {
    for row in case.iter_mut() {
      *row = n - 1 - *row;
    }
    case.reverse();
  }
  permuted.case_rows.reverse();
  let reordered = fit_conditional_logit(&permuted, &Default::default()).unwrap();
  for i in 0..n {
    close(
      reordered.fitted_probabilities[n - 1 - i],
      fit.fitted_probabilities[i],
    );
  }
  let mut doubled = p.clone();
  doubled.design.sample = None;
  doubled
    .design
    .response
    .extend_from_slice(&p.design.response);
  doubled
    .design
    .design_matrix
    .extend_from_slice(&p.design.design_matrix);
  doubled
    .alternative_codes
    .extend_from_slice(&p.alternative_codes);
  doubled.case_rows.extend(
    p.case_rows
      .iter()
      .map(|case| case.iter().map(|i| i + n).collect::<Vec<_>>()),
  );
  let repeat = fit_conditional_logit(&doubled, &Default::default()).unwrap();
  assert_eq!(repeat.choice_set_count, 2 * fit.choice_set_count);
  close(repeat.log_likelihood, 2.0 * fit.log_likelihood);
  for r in 0..fit.coefficients.len() {
    close(repeat.coefficients[r].value, fit.coefficients[r].value);
    for s in 0..fit.coefficients.len() {
      close(repeat.covariance[r][s], fit.covariance[r][s] / 2.0);
    }
  }
}

#[test]
fn two_alternative_difference_design_matches_no_intercept_binary_logit() {
  let mut y = Vec::new();
  let mut x = Vec::new();
  for (g, counts) in [[2, 1], [2, 2], [1, 2]].iter().enumerate() {
    for (choice, &count) in counts.iter().enumerate() {
      for _ in 0..count {
        y.push(choice as f64);
        x.push(vec![g as f64 - 1.0]);
      }
    }
  }
  let binary = EstimationProblem::new("y", vec!["x".into()], y.clone(), x.clone())
    .unwrap()
    .with_intercept(false);
  let logit = fit_binary_logit(&binary).unwrap();
  let mut rows = Vec::new();
  let mut response = Vec::new();
  let mut codes = Vec::new();
  let mut cases = Vec::new();
  for (i, (&choice, row)) in y.iter().zip(&x).enumerate() {
    cases.push(vec![2 * i, 2 * i + 1]);
    rows.extend([vec![0.0], row.clone()]);
    response.extend([1.0 - choice, choice]);
    codes.extend([10, 20]);
  }
  let p = ConditionalLogitProblem {
    design: EstimationProblem::new("chosen", vec!["x".into()], response, rows)
      .unwrap()
      .with_intercept(false),
    group_variable: "id".into(),
    alternative_variable: "alt".into(),
    case_rows: cases,
    alternative_codes: codes,
  };
  let conditional = fit_conditional_logit(&p, &Default::default()).unwrap();
  close(
    conditional.coefficients[0].value,
    logit.coefficients[0].value,
  );
  close(conditional.covariance[0][0], logit.covariance[0][0]);
  close(conditional.log_likelihood, logit.log_likelihood);
  close(conditional.null_log_likelihood, logit.null_log_likelihood);
  for (i, &probability) in logit.fitted_probabilities.iter().enumerate() {
    close(conditional.fitted_probabilities[2 * i + 1], probability);
  }
}

#[test]
fn structural_outcome_weight_rank_convergence_and_prediction_errors_are_explicit() {
  let p = dummy_problem(&[2, 4, 6, 8]);
  let fit = fit_conditional_logit(&p, &Default::default()).unwrap();
  let mut bad = p.clone();
  bad.design.include_intercept = true;
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  bad = p.clone();
  bad.case_rows[0][0] = p.design.response.len();
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  bad = p.clone();
  bad.case_rows[1][0] = bad.case_rows[0][0];
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  bad = p.clone();
  bad.case_rows[0].pop();
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  for code in [2, 9_007_199_254_740_992] {
    bad = p.clone();
    bad.alternative_codes[0] = code;
    assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  }
  for value in [0.0, 1.5] {
    bad = p.clone();
    bad.design.response[0] = value;
    assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  }
  bad = p.clone();
  bad.design.response[1] = 1.0;
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  bad = p.clone();
  bad.design.weights = Some(vec![1.0; bad.design.response.len()]);
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  bad = p.clone();
  for row in &mut bad.design.design_matrix {
    row[0] = 1.0;
  }
  assert!(
    fit_conditional_logit(&bad, &Default::default())
      .unwrap_err()
      .to_string()
      .contains("within-choice-set variation")
  );
  bad = p.clone();
  for row in &mut bad.design.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  for limit in [0, 1] {
    assert!(
      fit_conditional_logit(
        &p,
        &ConditionalLogitOptions {
          max_iterations: limit
        }
      )
      .unwrap_err()
      .to_string()
      .contains("iteration limit")
    );
  }
  assert!(fit_conditional_logit(&dummy_problem(&[0, 30]), &Default::default()).is_err());
  bad = p.clone();
  bad.design.sample.as_mut().unwrap().total_observations = 0;
  assert!(fit_conditional_logit(&bad, &Default::default()).is_err());
  let mut rows = vec![vec![0.0; 3]; 4];
  assert!(conditional_logit_predict_set(&fit, &rows[..3], &[1, 2, 3]).is_err());
  rows[0][0] = f64::NAN;
  assert!(conditional_logit_predict_set(&fit, &rows, &[1, 2, 3, 4]).is_err());
}
