#![forbid(unsafe_code)]

//! NIST Longley benchmark: Three-way statistical differential validation.
//!
//! Validates `tabdat-stats` ordinary least squares fitting against:
//! 1. NIST/ITL Certified Reference Values (trusted external reference)
//! 2. Python TabDat oracle (revision 16b45d9)

use tabdat_stats::{
  EstimationProblem, PostEstimationModel, fit_least_squares, predict_linear_response,
};

const REL_TOL: f64 = 1e-9;

fn rel_close(a: f64, b: f64) -> bool {
  let denom = a.abs().max(b.abs()).max(1e-300);
  (a - b).abs() / denom <= REL_TOL
}

fn assert_rel_close(label: &str, got: f64, expected: f64) {
  assert!(
    rel_close(got, expected),
    "{label}: got {got}, expected {expected} (relative diff: {})",
    ((got - expected).abs() / expected.abs().max(1e-300))
  );
}

fn load_longley_problem() -> EstimationProblem {
  let csv = include_str!("../fixtures/longley.csv");
  let mut lines = csv.lines();
  let header = lines.next().expect("header line");
  let cols: Vec<&str> = header.split(',').collect();
  let outcome = cols[0].to_string();
  let predictors: Vec<String> = cols[1..].iter().map(|s| s.to_string()).collect();

  let mut response = Vec::new();
  let mut design = Vec::new();

  for line in lines {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let vals: Vec<f64> = line
      .split(',')
      .map(|s| s.parse().expect("numeric value"))
      .collect();
    response.push(vals[0]);
    design.push(vals[1..].to_vec());
  }

  EstimationProblem::new(outcome, predictors, response, design).expect("valid longley problem")
}

#[test]
fn longley_ols_matches_nist_certified_values_and_python_oracle() {
  let problem = load_longley_problem();
  let result = fit_least_squares(&problem).expect("fit should succeed");

  // Structural checks
  assert_eq!(result.observation_count(), 16);
  assert_eq!(result.parameter_names.len(), 7);
  assert_eq!(result.degrees_of_freedom(), 9);
  assert_eq!(
    result.parameter_names,
    vec!["intercept", "x1", "x2", "x3", "x4", "x5", "x6"]
  );

  // --- 1. NIST/ITL Certified Reference Values ---
  // Source: NIST Statistical Reference Datasets (StRD), Longley dataset.
  let nist_coeff: &[f64] = &[
    -3482258.63459582,   // B0
    15.0618722713733,    // B1
    -0.0358191792925910, // B2
    -2.02022980381683,   // B3
    -1.03322686717359,   // B4
    -0.0511041056535807, // B5
    1829.15146461355,    // B6
  ];
  let nist_se: &[f64] = &[
    890420.383607373,
    84.9149257747669,
    0.0334910077722432,
    0.488399681651699,
    0.214274163161675,
    0.226073200069370,
    455.478499142212,
  ];

  for (i, c) in result.coefficients.iter().enumerate() {
    assert_rel_close(&format!("NIST coefficient B{i}"), c.value, nist_coeff[i]);
    assert_rel_close(
      &format!("NIST standard error B{i}"),
      c.standard_error.expect("SE"),
      nist_se[i],
    );
  }

  assert_rel_close(
    "NIST R^2",
    result.r_squared().expect("R^2"),
    0.995479004577296,
  );
  assert_rel_close(
    "NIST residual sd (root MSE)",
    result.root_mse().expect("root MSE"),
    304.854073561965,
  );
  assert_rel_close(
    "NIST F-statistic",
    result.fit_statistics.f_statistic.expect("F-stat"),
    330.285339234588,
  );

  // --- 2. Python TabDat Oracle (16b45d9) ---
  let py_coeff: &[f64] = &[
    -3482258.6345977746,
    15.06187227159171,
    -0.035819179292643444,
    -2.020229803817365,
    -1.033226867173639,
    -0.05110410565365875,
    1829.1514646145479,
  ];
  let py_se: &[f64] = &[
    890420.3836072548,
    84.9149257748673,
    0.03349100777224425,
    0.4883996816515502,
    0.21427416316161904,
    0.22607320006931417,
    455.4784991421528,
  ];

  for (i, c) in result.coefficients.iter().enumerate() {
    assert_rel_close(&format!("Python coefficient B{i}"), c.value, py_coeff[i]);
    assert_rel_close(
      &format!("Python standard error B{i}"),
      c.standard_error.expect("SE"),
      py_se[i],
    );
  }

  assert_rel_close(
    "Python R^2",
    result.r_squared().expect("R^2"),
    0.9954790045772965,
  );
  assert_rel_close(
    "Python adjusted R^2",
    result.adjusted_r_squared().expect("adj R^2"),
    0.9924650076288276,
  );
  assert_rel_close(
    "Python root MSE",
    result.root_mse().expect("root MSE"),
    304.85407356193167,
  );

  // --- 3. In-Sample Prediction Parity ---
  let mut full_design = Vec::with_capacity(16);
  for row in &problem.design_matrix {
    let mut aug = vec![1.0];
    aug.extend(row.iter().copied());
    full_design.push(aug);
  }
  let beta: Vec<f64> = result.coefficients.iter().map(|c| c.value).collect();
  let predicted = predict_linear_response(&full_design, &beta).expect("prediction");
  for (i, (&p, &f)) in predicted
    .iter()
    .zip(result.fitted_values.iter())
    .enumerate()
  {
    assert_rel_close(&format!("Predicted vs fitted value row {i}"), p, f);
  }

  // --- 4. Post-Estimation State & Hypothesis Testing on Longley ---
  let post = PostEstimationModel::from_least_squares(&result);
  assert_rel_close(
    "PostEstimation x1 parameter",
    post.parameter("x1").expect("x1"),
    result.parameter_value("x1").expect("x1 in result"),
  );

  // Joint test: H0: B1 = 0, B2 = 0
  let r_matrix = vec![
    vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
  ];
  let r_vector = vec![0.0, 0.0];
  let test_result = post
    .test_linear_hypothesis(&r_matrix, &r_vector)
    .expect("joint hypothesis test");
  assert_eq!(test_result.df_num, 2);
  assert_eq!(test_result.df_denom, 9);
  assert!(test_result.f_statistic > 0.0);
  assert!(test_result.chi2_statistic > 0.0);
}
