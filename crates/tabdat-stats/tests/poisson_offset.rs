use serde_json::{Value, json};
use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_poisson, fit_poisson_with_offset,
  poisson_average_marginal_effects,
};

fn problem(design: &str, clustered: bool) -> EstimationProblem {
  let y = vec![
    0., 0., 2., 0., 0., 3., 0., 3., 1., 3., 8., 3., 0., 0., 0., 2., 1., 2., 0., 0., 2., 1., 1., 2.,
    0., 5., 4., 1., 1., 0., 0., 2., 2., 0., 1., 0.,
  ];
  let (names, x) = if design == "intercept" {
    (vec![], vec![vec![]; 36])
  } else {
    (
      vec!["x".into(), "z".into()],
      (0..36)
        .map(|i| vec![-2. + 4. * i as f64 / 35., [0., 1., 0.4][i % 3]])
        .collect(),
    )
  };
  EstimationProblem::new("y", names, y, x)
    .unwrap()
    .with_intercept(design != "noconstant")
    .with_sample(
      EstimationSample::new(
        (0..36).collect(),
        36,
        0,
        None,
        clustered.then(|| (0..36).map(|i| format!("g{}", i % 6)).collect()),
      )
      .unwrap(),
    )
}
fn offsets() -> Vec<f64> {
  (0..36).map(|i| [-1., 0.2, 0.6, 1.2][i % 4]).collect()
}

#[test]
fn fixed_offset_fits_and_inference_match_base_r() {
  // R 4.6.1 glm; independent score/bread sandwich algebra. Conditional-X
  // effects hold offsets fixed. Absolute 1e-9 for fit totals and parameter
  // quantities; relative 1e-9 for tails (never an absolute small-p shortcut).
  let rows: Vec<Vec<_>> = include_str!("fixtures/poisson_offset_reference.csv")
    .lines()
    .skip(1)
    .map(|line| line.split(',').collect())
    .collect();
  assert_eq!(rows.len(), 950);
  let offsets = offsets();
  for design in ["multiple", "noconstant", "intercept"] {
    for covariance in ["model", "robust", "cluster"] {
      let p = problem(design, covariance == "cluster");
      let fit = fit_poisson_with_offset(
        &p,
        &offsets,
        covariance == "robust",
        (covariance == "cluster").then_some("person"),
      )
      .unwrap();
      let effects = poisson_average_marginal_effects(&p, &fit).unwrap();
      let j = serde_json::to_value(&fit).unwrap();
      let e = serde_json::to_value(&effects).unwrap();
      let case = format!("{design}_{covariance}");
      for row in rows.iter().filter(|r| r[0] == case) {
        let (quantity, i, k) = (
          row[1],
          row[2].parse::<usize>().unwrap(),
          row[3].parse::<usize>().unwrap(),
        );
        let actual: Value = if let Some(field) = quantity.strip_prefix("coefficient_") {
          j["coefficients"][i][field].clone()
        } else if let Some(field) = quantity.strip_prefix("effect_") {
          e["effects"][i][field].clone()
        } else {
          match quantity {
            "covariance" => j["covariance"][i][k].clone(),
            "wald_statistic" => j["omnibus_wald"]["statistic"].clone(),
            "wald_p_value" => j["omnibus_wald"]["p_value"].clone(),
            "mean" => j["fitted_means"][i].clone(),
            "xb" => {
              let shift = usize::from(p.include_intercept);
              json!(
                offsets[i]
                  + p.design_matrix[i]
                    .iter()
                    .zip(&fit.coefficients[shift..])
                    .map(|(x, b)| x * b.value)
                    .sum::<f64>()
                  + if shift == 1 {
                    fit.coefficients[0].value
                  } else {
                    0.
                  }
              )
            }
            _ => j[quantity].clone(),
          }
        };
        if row[4] == "NA" {
          assert!(actual.is_null(), "{row:?}");
          continue;
        }
        let expected: f64 = row[4].parse().unwrap();
        let actual = actual.as_f64().unwrap();
        let tolerance = if quantity.ends_with("p_value") {
          1e-9 * expected.abs()
        } else {
          1e-9
        };
        assert!(
          actual.is_finite() && (actual - expected).abs() <= tolerance,
          "{row:?}: {actual:.17e} != {expected:.17e}, tolerance={tolerance}"
        );
      }
    }
  }
}

#[test]
fn zero_offsets_equivalence_fixed_shift_and_invalid_alignment() {
  let p = problem("multiple", false);
  let plain = fit_poisson(&p, true).unwrap();
  assert_eq!(
    plain,
    fit_poisson_with_offset(&p, &[0.; 36], true, None).unwrap()
  );
  let with = fit_poisson_with_offset(&p, &offsets(), false, None).unwrap();
  let shifted: Vec<_> = offsets().iter().map(|v| v + 100.).collect();
  let other = fit_poisson_with_offset(&p, &shifted, false, None).unwrap();
  assert!((other.coefficients[0].value - with.coefficients[0].value + 100.).abs() < 1e-10);
  for (a, b) in with.fitted_means.iter().zip(other.fitted_means) {
    assert!((a - b).abs() < 1e-10);
  }
  for offsets in [
    vec![],
    vec![0.; 35],
    vec![f64::NAN; 36],
    vec![f64::INFINITY; 36],
  ] {
    assert!(fit_poisson_with_offset(&p, &offsets, false, None).is_err());
  }
  assert!(fit_poisson_with_offset(&p, &[0.; 36], true, Some("g")).is_err());
}
