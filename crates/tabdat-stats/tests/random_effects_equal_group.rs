use tabdat_stats::{
  EstimationProblem, EstimationSample, RandomEffectsComponentMethod, fit_random_effects,
  fit_random_effects_equal_group_between,
};

fn problem(zero_within_variance: bool) -> EstimationProblem {
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut groups = Vec::new();
  for g in 0..12 {
    for t in 0..4 {
      let v = (g as f64) / 3.0 + (t as f64 - 1.5) * 0.7;
      x.push(vec![v]);
      y.push(if zero_within_variance {
        (g % 3) as f64
      } else {
        1.0 + 0.4 * v + (g % 3) as f64 + ((g * 5 + t * 3) % 7) as f64 / 5.0
      });
      groups.push(g.to_string());
    }
  }
  let n = y.len();
  EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_sample(EstimationSample::new((0..n).collect(), n, 0, None, Some(groups)).unwrap())
}

#[test]
fn balanced_formula_reduces_to_existing_swamy_arora_without_changing_inference() {
  let p = problem(false);
  let swar = fit_random_effects(&p, "id").unwrap();
  let equal = fit_random_effects_equal_group_between(&p, "id").unwrap();
  assert_eq!(
    equal.component_method,
    RandomEffectsComponentMethod::EqualGroupBetween
  );
  assert_eq!(
    swar.component_method,
    RandomEffectsComponentMethod::SwamyArora
  );
  assert!((equal.raw_sigma_u_squared - swar.raw_sigma_u_squared).abs() < 1e-12);
  assert_eq!(equal.sigma_e_squared, swar.sigma_e_squared);
  for (a, b) in equal.gls.coefficients.iter().zip(&swar.gls.coefficients) {
    assert!((a.value - b.value).abs() < 1e-12);
    assert!((a.standard_error.unwrap() - b.standard_error.unwrap()).abs() < 1e-12);
    assert!((a.p_value.unwrap() - b.p_value.unwrap()).abs() < 1e-12);
  }
  for (a, b) in equal
    .gls
    .covariance
    .matrix
    .iter()
    .flatten()
    .zip(swar.gls.covariance.matrix.iter().flatten())
  {
    assert!((a - b).abs() < 1e-12);
  }
  assert_eq!(equal.group_size, 4);
  assert!((equal.theta - swar.theta).abs() < 1e-12);
}

#[test]
fn invalid_identification_is_not_a_zero_variance_fallback() {
  let mut p = problem(false);
  for row in &mut p.design_matrix {
    row[0] = 1.0;
  }
  assert!(fit_random_effects_equal_group_between(&p, "id").is_err());
  assert!(fit_random_effects_equal_group_between(&problem(true), "id").is_err());
}
