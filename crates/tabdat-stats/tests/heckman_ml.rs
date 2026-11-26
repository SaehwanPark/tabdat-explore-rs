use std::collections::BTreeMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, HeckmanMlOptions, HeckmanPredictionKind as Kind,
  fit_heckman_ml, heckman_ml_predict_row,
};

type Reference = BTreeMap<(String, String, usize, usize), f64>;

fn cells(line: &str) -> Vec<&str> {
  line
    .split(',')
    .map(|value| value.trim_matches('"'))
    .collect()
}

fn references() -> Reference {
  let mut map = BTreeMap::new();
  for line in include_str!("fixtures/heckman_ml_reference.csv")
    .lines()
    .skip(1)
  {
    let c = cells(line);
    assert_eq!(c.len(), 5);
    assert!(
      map
        .insert(
          (
            c[0].into(),
            c[1].into(),
            c[2].parse().unwrap(),
            c[3].parse().unwrap()
          ),
          c[4].parse().unwrap()
        )
        .is_none()
    );
  }
  assert_eq!(map.len(), 142);
  map
}

fn reference(references: &Reference, case: &str, quantity: &str, i: usize, j: usize) -> f64 {
  *references
    .get(&(case.into(), quantity.into(), i, j))
    .unwrap_or_else(|| panic!("missing {case}/{quantity}/{i}/{j}"))
}

fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite()
      && expected.is_finite()
      && (actual - expected).abs() <= 2e-6 + 2e-6 * expected.abs(),
    "{actual:.16e} != {expected:.16e}"
  );
}

fn problems(case: &str) -> (EstimationProblem, EstimationProblem, Vec<(usize, f64, f64)>) {
  let mut d = Vec::new();
  let mut z = Vec::new();
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut eligible = Vec::new();
  let mut selected = Vec::new();
  let mut prediction_rows = Vec::new();
  for line in include_str!("fixtures/heckman_data.csv").lines().skip(1) {
    let c = cells(line);
    if c[0] != case {
      continue;
    }
    let flag = c[2].parse::<f64>().unwrap();
    let xv = c[4].parse::<f64>().unwrap();
    let zv = c[5].parse::<f64>().unwrap();
    let i = d.len();
    d.push(flag);
    z.push(vec![xv, zv]);
    eligible.push(2 * i + 1);
    prediction_rows.push((c[1].parse().unwrap(), xv, zv));
    if flag == 1.0 {
      y.push(c[3].parse().unwrap());
      x.push(vec![xv]);
      selected.push(2 * i + 1);
    } else {
      assert!(c[3].is_empty());
    }
  }
  assert_eq!(d.len(), 360);
  let constant = case != "no_constant";
  let selection = EstimationProblem::new("d", vec!["x".into(), "z".into()], d, z)
    .unwrap()
    .with_intercept(constant)
    .with_sample(EstimationSample::new(eligible, 720, 360, None, None).unwrap());
  let outcome = EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_intercept(constant)
    .with_sample(
      EstimationSample::new(selected.clone(), 720, 720 - selected.len(), None, None).unwrap(),
    );
  (selection, outcome, prediction_rows)
}

fn reversed_problems(
  selection: &EstimationProblem,
  outcome: &EstimationProblem,
) -> (EstimationProblem, EstimationProblem) {
  let selection_sample = selection.sample.as_ref().unwrap();
  let outcome_sample = outcome.sample.as_ref().unwrap();
  let order: Vec<usize> = (0..selection.observation_count()).rev().collect();
  let mut selection_response = Vec::with_capacity(order.len());
  let mut selection_design = Vec::with_capacity(order.len());
  let mut selection_positions = Vec::with_capacity(order.len());
  let mut outcome_response = Vec::with_capacity(outcome.observation_count());
  let mut outcome_design = Vec::with_capacity(outcome.observation_count());
  let mut outcome_positions = Vec::with_capacity(outcome.observation_count());

  for (permuted_row, &selection_row) in order.iter().enumerate() {
    selection_response.push(selection.response[selection_row]);
    selection_design.push(selection.design_matrix[selection_row].clone());
    let original_position = selection_sample.retained_indices[selection_row];
    let permuted_position = 2 * permuted_row + 1;
    selection_positions.push(permuted_position);
    if selection.response[selection_row] == 1.0 {
      let outcome_row = outcome_sample
        .retained_indices
        .iter()
        .position(|&position| position == original_position)
        .unwrap();
      outcome_response.push(outcome.response[outcome_row]);
      outcome_design.push(outcome.design_matrix[outcome_row].clone());
      outcome_positions.push(permuted_position);
    }
  }

  let permuted_selection = EstimationProblem::new(
    selection.outcome.clone(),
    selection.predictor_names.clone(),
    selection_response,
    selection_design,
  )
  .unwrap()
  .with_intercept(selection.include_intercept)
  .with_sample(
    EstimationSample::new(
      selection_positions,
      selection_sample.total_observations,
      selection_sample.dropped_observations,
      None,
      None,
    )
    .unwrap(),
  );
  let permuted_outcome = EstimationProblem::new(
    outcome.outcome.clone(),
    outcome.predictor_names.clone(),
    outcome_response,
    outcome_design,
  )
  .unwrap()
  .with_intercept(outcome.include_intercept)
  .with_sample(
    EstimationSample::new(
      outcome_positions,
      outcome_sample.total_observations,
      outcome_sample.dropped_observations,
      None,
      None,
    )
    .unwrap(),
  );
  (permuted_selection, permuted_outcome)
}

#[test]
fn base_r_joint_likelihood_covariance_and_all_five_point_projections_match() {
  let references = references();
  for case in ["exclusion", "no_constant"] {
    let (selection, outcome, prediction_rows) = problems(case);
    let before = (selection.clone(), outcome.clone());
    let result = fit_heckman_ml(&selection, &outcome, &HeckmanMlOptions::default()).unwrap();
    assert_eq!((selection.clone(), outcome.clone()), before);
    assert_eq!(result.observation_count, 360);
    assert_eq!(result.selected_count, outcome.observation_count());
    assert_eq!(result.unselected_count, 360 - outcome.observation_count());
    assert_eq!(result.selection_sample, selection.sample);
    assert_eq!(result.outcome_sample, outcome.sample);
    assert_eq!(result.excluded_selection_predictors, vec!["z"]);
    assert!(result.iterations <= 200);
    assert!(result.max_abs_scaled_score <= 1e-8);
    assert!(result.max_abs_scaled_step <= 1e-8);
    assert_eq!(
      result.covariance_parameter_names.len(),
      result.covariance.len()
    );
    let mut expected_parameter_names = Vec::new();
    if result.include_intercept {
      expected_parameter_names.push("outcome:intercept".to_string());
    }
    expected_parameter_names.push("outcome:x".to_string());
    if result.selection_include_intercept {
      expected_parameter_names.push("selection:intercept".to_string());
    }
    expected_parameter_names.extend(["selection:x", "selection:z"].map(str::to_string));
    expected_parameter_names.extend(["lnsigma", "athrho"].map(str::to_string));
    assert_eq!(result.covariance_parameter_names, expected_parameter_names);
    assert!(result.log_likelihood >= result.independent_log_likelihood);
    close(
      result.independence_lr_chi_square,
      2.0 * (result.log_likelihood - result.independent_log_likelihood),
    );

    for (quantity, estimates) in [
      ("outcome", &result.outcome_coefficients),
      ("selection", &result.selection_coefficients),
    ] {
      for (i, estimate) in estimates.iter().enumerate() {
        close(
          estimate.value,
          reference(&references, case, &format!("{quantity}_value"), i + 1, 0),
        );
        close(
          estimate.standard_error,
          reference(
            &references,
            case,
            &format!("{quantity}_standard_error"),
            i + 1,
            0,
          ),
        );
      }
    }
    for (i, row) in result.covariance.iter().enumerate() {
      for (j, covariance) in row.iter().enumerate() {
        close(
          *covariance,
          reference(&references, case, "covariance", i + 1, j + 1),
        );
      }
    }
    for (quantity, actual) in [
      ("log_likelihood", result.log_likelihood),
      (
        "independent_log_likelihood",
        result.independent_log_likelihood,
      ),
      (
        "independence_lr_chi_square",
        result.independence_lr_chi_square,
      ),
      ("sigma", result.sigma),
      ("rho", result.rho),
      ("lambda", result.lambda),
    ] {
      close(actual, reference(&references, case, quantity, 0, 0));
    }

    for row_number in [1, 10, 100, 359] {
      let (_, x, z) = prediction_rows
        .iter()
        .find(|(row, _, _)| *row == row_number)
        .copied()
        .unwrap();
      for (kind, name) in [
        (Kind::Xb, "xb"),
        (Kind::Pr, "pr"),
        (Kind::Mills, "mills"),
        (Kind::ConditionalSelectedMean, "ycond"),
        (Kind::ObservedContributionMean, "yexpected"),
      ] {
        close(
          heckman_ml_predict_row(&result, &[x], &[x, z], kind).unwrap(),
          reference(
            &references,
            case,
            &format!("prediction_{name}"),
            row_number,
            0,
          ),
        );
      }
    }
    assert!(heckman_ml_predict_row(&result, &[f64::NAN], &[0.2, 0.3], Kind::Pr).is_ok());
    assert!(heckman_ml_predict_row(&result, &[0.2], &[f64::NAN, 0.3], Kind::Xb).is_ok());
  }
}

#[test]
fn invalid_iteration_bounds_and_nonconverged_fits_fail() {
  let (selection, outcome, _) = problems("exclusion");
  assert!(
    fit_heckman_ml(
      &selection,
      &outcome,
      &HeckmanMlOptions { max_iterations: 0 }
    )
    .is_err()
  );
  assert!(
    fit_heckman_ml(
      &selection,
      &outcome,
      &HeckmanMlOptions { max_iterations: 1 }
    )
    .is_err()
  );
}

#[test]
fn row_permutation_and_affine_outcome_units_preserve_joint_fit_contracts() {
  let (selection, outcome, _) = problems("exclusion");
  let original = fit_heckman_ml(&selection, &outcome, &HeckmanMlOptions::default()).unwrap();

  let (reversed_selection, reversed_outcome) = reversed_problems(&selection, &outcome);
  let reversed = fit_heckman_ml(
    &reversed_selection,
    &reversed_outcome,
    &HeckmanMlOptions::default(),
  )
  .unwrap();
  assert_eq!(
    reversed.selection_sample.as_ref().unwrap().retained_indices,
    reversed_selection.sample.as_ref().unwrap().retained_indices
  );
  assert_eq!(
    reversed.outcome_sample.as_ref().unwrap().retained_indices,
    reversed_outcome.sample.as_ref().unwrap().retained_indices
  );
  let selected_original_rows: Vec<_> = reversed
    .selected_selection_indices
    .iter()
    .map(|&i| reversed.selection_sample.as_ref().unwrap().retained_indices[i])
    .collect();
  assert_eq!(
    selected_original_rows,
    reversed.outcome_sample.as_ref().unwrap().retained_indices
  );
  for (left, right) in original
    .outcome_coefficients
    .iter()
    .chain(&original.selection_coefficients)
    .zip(
      reversed
        .outcome_coefficients
        .iter()
        .chain(&reversed.selection_coefficients),
    )
  {
    close(left.value, right.value);
    close(left.standard_error, right.standard_error);
  }
  close(original.log_likelihood, reversed.log_likelihood);
  for (left, right) in original.covariance.iter().zip(&reversed.covariance) {
    for (left, right) in left.iter().zip(right) {
      close(*left, *right);
    }
  }

  let shift = 3.25;
  let scale = 2.75;
  let scaled_outcome = EstimationProblem::new(
    outcome.outcome.clone(),
    outcome.predictor_names.clone(),
    outcome
      .response
      .iter()
      .map(|value| shift + scale * value)
      .collect(),
    outcome.design_matrix.clone(),
  )
  .unwrap()
  .with_intercept(outcome.include_intercept)
  .with_sample(outcome.sample.clone().unwrap());
  let scaled = fit_heckman_ml(&selection, &scaled_outcome, &HeckmanMlOptions::default()).unwrap();
  for (i, (before, after)) in original
    .outcome_coefficients
    .iter()
    .zip(&scaled.outcome_coefficients)
    .enumerate()
  {
    close(
      after.value,
      if i == 0 {
        shift + scale * before.value
      } else {
        scale * before.value
      },
    );
  }
  for (before, after) in original
    .selection_coefficients
    .iter()
    .zip(&scaled.selection_coefficients)
  {
    close(before.value, after.value);
  }
  close(scaled.sigma, scale * original.sigma);
  close(scaled.rho, original.rho);
  close(scaled.lambda, scale * original.lambda);
  close(
    scaled.log_likelihood,
    original.log_likelihood - original.selected_count as f64 * scale.ln(),
  );
  close(
    scaled.independent_log_likelihood,
    original.independent_log_likelihood - original.selected_count as f64 * scale.ln(),
  );
  close(
    scaled.independence_lr_chi_square,
    original.independence_lr_chi_square,
  );
  let p = original.outcome_coefficients.len();
  let mut parameter_scales = vec![scale; p];
  parameter_scales.resize(original.covariance.len(), 1.0);
  for (i, row) in original.covariance.iter().enumerate() {
    for (j, covariance) in row.iter().enumerate() {
      close(
        scaled.covariance[i][j],
        covariance * parameter_scales[i] * parameter_scales[j],
      );
    }
  }
}

#[test]
fn rank_deficiency_and_expanded_work_bounds_fail_closed() {
  let (selection, outcome, _) = problems("exclusion");
  let rank_deficient_selection = EstimationProblem::new(
    selection.outcome.clone(),
    selection.predictor_names.clone(),
    selection.response.clone(),
    selection
      .design_matrix
      .iter()
      .map(|row| vec![row[0], 2.0 * row[0]])
      .collect(),
  )
  .unwrap()
  .with_sample(selection.sample.clone().unwrap());
  assert!(
    fit_heckman_ml(
      &rank_deficient_selection,
      &outcome,
      &HeckmanMlOptions::default()
    )
    .is_err()
  );

  let n = 6_105;
  let predictors = 30;
  let names: Vec<String> = (0..predictors).map(|i| format!("x{i}")).collect();
  let mut state = 0x9e37_79b9_7f4a_7c15_u64;
  let mut next_normal = || {
    let mut total = 0.0;
    for _ in 0..6 {
      state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
      let uniform = (state >> 11) as f64 / 9_007_199_254_740_992.0;
      total += (uniform - 0.5) * 3.464_101_615_137_754_4;
    }
    total / 6.0_f64.sqrt()
  };
  let mut selection_response = Vec::with_capacity(n);
  let mut selection_design = Vec::with_capacity(n);
  let mut outcome_response = Vec::with_capacity(n / 2);
  let mut outcome_design = Vec::with_capacity(n / 2);
  let mut outcome_indices = Vec::with_capacity(n / 2);
  for i in 0..n {
    let x: Vec<f64> = (0..predictors).map(|_| next_normal()).collect();
    let selection_error = next_normal();
    let outcome_error = next_normal();
    let d = if -0.2 + 0.55 * x[0] - 0.3 * x[1] + selection_error > 0.0 {
      1.0
    } else {
      0.0
    };
    selection_response.push(d);
    selection_design.push(x.clone());
    if d == 1.0 {
      outcome_response.push(4.0 + 0.8 * x[0] - 0.4 * x[1] + outcome_error);
      outcome_design.push(x);
      outcome_indices.push(i);
    }
  }
  let selection_problem =
    EstimationProblem::new("d", names.clone(), selection_response, selection_design)
      .unwrap()
      .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, None).unwrap());
  let outcome_problem = EstimationProblem::new("y", names, outcome_response, outcome_design)
    .unwrap()
    .with_sample(
      EstimationSample::new(
        outcome_indices.clone(),
        n,
        n - outcome_indices.len(),
        None,
        None,
      )
      .unwrap(),
    );
  let error = fit_heckman_ml(
    &selection_problem,
    &outcome_problem,
    &HeckmanMlOptions::default(),
  )
  .unwrap_err();
  assert!(
    error.to_string().contains("bounded likelihood work"),
    "{error}"
  );
}
