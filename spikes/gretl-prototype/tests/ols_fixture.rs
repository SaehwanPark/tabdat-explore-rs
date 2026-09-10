//! OLS fixture: three-way validation of the libgretl OLS boundary.
//!
//! Runs OLS on the NIST Longley dataset through the Rust facade and compares
//! the coefficients, standard errors, and fit statistics against:
//!
//! 1. The NIST/ITL certified reference values (trusted external reference), and
//! 2. The pinned Python TabDat oracle (`regress y x1..x6` on the same data).
//!
//! The tolerance is a relative 1e-9, well below the observed agreement
//! (libgretl matches both references to ~1e-11..1e-13 relative) so that any
//! real regression is caught. Tolerances are not widened to hide disagreement.

use tabdat_gretl_spike::{EstimationProblem, run_ols};

/// Relative tolerance for coefficient/standard-error/fit-statistic comparison.
///
/// Justification: libgretl OLS on Longley agrees with the NIST certified
/// values and the Python oracle to ~1e-11..1e-13 relative. 1e-9 is three
/// orders of magnitude looser than that, so it cannot mask a real regression
/// while absorbing harmless last-bit floating-point differences across BLAS
/// backends.
const REL_TOL: f64 = 1e-9;

fn rel_close(a: f64, b: f64) -> bool {
  let denom = a.abs().max(b.abs()).max(1e-300);
  (a - b).abs() / denom <= REL_TOL
}

fn assert_rel_close(label: &str, got: f64, expected: f64) {
  assert!(
    rel_close(got, expected),
    "{label}: got {got}, expected {expected} (rel diff {})",
    ((got - expected).abs() / expected.abs().max(1e-300))
  );
}

/// Parse the Longley CSV fixture into an EstimationProblem.
fn load_longley() -> EstimationProblem {
  let csv = include_str!("../fixtures/longley.csv");
  let mut lines = csv.lines();
  let header = lines.next().expect("header line").to_string();
  let cols: Vec<&str> = header.split(',').collect();
  let outcome = cols[0].to_string();
  let predictors: Vec<String> = cols[1..].iter().map(|s| s.to_string()).collect();

  let mut data = Vec::new();
  for line in lines {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }
    let row: Vec<f64> = line
      .split(',')
      .map(|s| s.parse().expect("numeric cell"))
      .collect();
    data.push(row);
  }

  EstimationProblem {
    outcome,
    predictors,
    include_intercept: true,
    data,
  }
}

#[test]
fn ols_longley_matches_nist_and_python_oracle() {
  let problem = load_longley();
  let result = run_ols(&problem).expect("OLS should succeed");

  // --- Structural checks ---
  assert_eq!(result.nobs, 16, "Longley has 16 observations");
  assert_eq!(result.ncoeff, 7, "intercept + 6 predictors");
  assert_eq!(result.dfn, 6, "numerator df = k");
  assert_eq!(result.dfd, 9, "denominator df = n - k - 1");

  // Coefficient names: intercept then x1..x6.
  let names: Vec<&str> = result
    .coefficients
    .iter()
    .map(|c| c.name.as_str())
    .collect();
  assert_eq!(names, vec!["intercept", "x1", "x2", "x3", "x4", "x5", "x6"]);

  // --- NIST/ITL certified reference values (trusted external reference) ---
  // Source: gretl 2026b tests/Longley.dat "Certified Regression Statistics".
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
    assert_rel_close(&format!("NIST coeff B{i}"), c.value, nist_coeff[i]);
    assert_rel_close(&format!("NIST se B{i}"), c.standard_error, nist_se[i]);
  }
  assert_rel_close("NIST R^2", result.r_squared, 0.995479004577296);
  assert_rel_close("NIST residual sd", result.sigma, 304.854073561965);
  assert_rel_close("NIST F", result.f_statistic, 330.285339234588);

  // --- Pinned Python TabDat oracle (regress y x1..x6, --json) ---
  // Source: tabdat-explore @ 16b45d9 (v0.25.0), `tabdat --json -f longley.td`.
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
    assert_rel_close(&format!("Python coeff B{i}"), c.value, py_coeff[i]);
    assert_rel_close(&format!("Python se B{i}"), c.standard_error, py_se[i]);
  }
  assert_rel_close("Python R^2", result.r_squared, 0.9954790045772965);
  assert_rel_close("Python adj R^2", result.adj_r_squared, 0.9924650076288276);
  assert_rel_close("Python root_mse", result.sigma, 304.85407356193167);
}

#[test]
fn ols_rejects_ragged_data() {
  let problem = EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into()],
    include_intercept: true,
    data: vec![vec![1.0, 2.0], vec![3.0]], // second row is short
  };
  let err = run_ols(&problem).unwrap_err();
  assert!(matches!(
    err,
    tabdat_gretl_spike::GretlError::InvalidProblem(_)
  ));
}

#[test]
fn ols_rejects_empty_predictors() {
  let problem = EstimationProblem {
    outcome: "y".into(),
    predictors: vec![],
    include_intercept: true,
    data: vec![vec![1.0], vec![2.0]],
  };
  let err = run_ols(&problem).unwrap_err();
  assert!(matches!(
    err,
    tabdat_gretl_spike::GretlError::InvalidProblem(_)
  ));
}
