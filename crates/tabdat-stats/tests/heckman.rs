use std::collections::BTreeMap;
use tabdat_stats::{
  EstimationProblem, EstimationSample, HeckmanPredictionKind as Kind, HeckmanTwoStepResult,
  ProbitOptions, fit_heckman_two_step, heckman_predict_row,
};

type Reference = BTreeMap<(String, String, usize, usize), f64>;
fn cells(line: &str) -> Vec<&str> {
  line.split(',').map(|s| s.trim_matches('"')).collect()
}
fn references() -> Reference {
  let mut map = BTreeMap::new();
  for line in include_str!("fixtures/heckman_reference.csv")
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
  assert_eq!(map.len(), 98);
  map
}
fn reference(r: &Reference, case: &str, name: &str, i: usize, j: usize) -> f64 {
  *r.get(&(case.into(), name.into(), i, j))
    .unwrap_or_else(|| panic!("missing {case}/{name}/{i}/{j}"))
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite()
      && expected.is_finite()
      && (actual - expected).abs() <= 1e-9 + 1e-8 * expected.abs(),
    "{actual:.16e} != {expected:.16e}"
  );
}
fn problems(case: &str) -> (EstimationProblem, EstimationProblem) {
  let mut d = Vec::new();
  let mut z = Vec::new();
  let mut y = Vec::new();
  let mut x = Vec::new();
  let mut selected = Vec::new();
  for line in include_str!("fixtures/heckman_data.csv").lines().skip(1) {
    let c = cells(line);
    assert_eq!(c.len(), 6);
    if c[0] != case {
      continue;
    }
    let flag = c[2].parse::<f64>().unwrap();
    let value = c[4].parse::<f64>().unwrap();
    let i = d.len();
    d.push(flag);
    z.push(vec![value, c[5].parse().unwrap()]);
    if flag == 1.0 {
      y.push(c[3].parse().unwrap());
      x.push(vec![value]);
      selected.push(2 * i + 1);
    } else {
      assert!(c[3].is_empty());
    }
  }
  assert_eq!(d.len(), 360);
  let intercept = case != "no_constant";
  let selection = EstimationProblem::new("d", vec!["x".into(), "z".into()], d, z)
    .unwrap()
    .with_intercept(intercept)
    .with_sample(
      EstimationSample::new((0..360).map(|i| 2 * i + 1).collect(), 720, 360, None, None).unwrap(),
    );
  let count = selected.len();
  let outcome = EstimationProblem::new("y", vec!["x".into()], y, x)
    .unwrap()
    .with_intercept(intercept)
    .with_sample(EstimationSample::new(selected, 720, 720 - count, None, None).unwrap());
  (selection, outcome)
}
fn fit(s: &EstimationProblem, o: &EstimationProblem) -> HeckmanTwoStepResult {
  fit_heckman_two_step(s, o, &ProbitOptions::default()).unwrap()
}

#[test]
fn independent_r_glm_lm_corrected_covariance_and_all_five_point_projections() {
  let r = references();
  for case in ["exclusion", "no_constant"] {
    let (s, o) = problems(case);
    let before = (s.clone(), o.clone());
    let f = fit(&s, &o);
    assert_eq!((s.clone(), o.clone()), before);
    assert_eq!(f.selection.sample, s.sample);
    assert_eq!(f.sample, o.sample);
    assert_eq!(f.excluded_selection_predictors, vec!["z"]);
    let positions: Vec<_> = s
      .response
      .iter()
      .enumerate()
      .filter_map(|(i, &d)| (d == 1.0).then_some(i))
      .collect();
    assert_eq!(f.selected_selection_indices, positions);
    assert_eq!(f.selected_count, o.response.len());
    assert_eq!(f.unselected_count, 360 - o.response.len());
    assert!(f.max_abs_scaled_normal_equation <= 1e-10);
    assert!(
      f.selection.max_abs_scaled_score <= 1e-10 && f.selection.max_abs_scaled_newton_step <= 1e-10
    );
    for (group, coefs, covariance) in [
      (
        "selection",
        &f.selection.coefficients,
        &f.selection.covariance,
      ),
      ("outcome", &f.coefficients, &f.covariance),
    ] {
      for (i, b) in coefs.iter().enumerate() {
        for (field, value) in [
          ("value", b.value),
          ("standard_error", b.standard_error),
          ("statistic", b.statistic),
          ("confidence_lower", b.confidence_lower),
          ("confidence_upper", b.confidence_upper),
        ] {
          close(
            value,
            reference(&r, case, &format!("{group}_{field}"), i + 1, 0),
          );
        }
        let expected = reference(&r, case, &format!("{group}_p_value"), i + 1, 0);
        assert!(
          b.p_value > 0.0 && expected > 0.0 && (b.p_value - expected).abs() <= 1e-8 * expected,
          "tail {case}/{group}/{i}: {} != {expected}",
          b.p_value
        );
        for (j, v) in covariance[i].iter().enumerate() {
          close(
            *v,
            reference(&r, case, &format!("{group}_covariance"), i + 1, j + 1),
          );
        }
      }
    }
    for (field, value) in [
      ("n", 360.0),
      ("selected_count", f.selected_count as f64),
      ("sigma", f.sigma),
      ("rho", f.rho),
      ("outcome_sse", f.outcome_sse),
      ("selection_log_likelihood", f.selection.log_likelihood),
    ] {
      close(value, reference(&r, case, field, 0, 0));
    }
    let mut count = 0;
    for line in include_str!("fixtures/heckman_predictions.csv")
      .lines()
      .skip(1)
    {
      let c = cells(line);
      assert_eq!(c.len(), 7);
      if c[0] != case {
        continue;
      }
      let row = c[1].parse::<usize>().unwrap() - 1;
      for (j, kind) in [
        Kind::Xb,
        Kind::Pr,
        Kind::Mills,
        Kind::ConditionalSelectedMean,
        Kind::ObservedContributionMean,
      ]
      .iter()
      .enumerate()
      {
        let value =
          heckman_predict_row(&f, &[s.design_matrix[row][0]], &s.design_matrix[row], *kind)
            .unwrap();
        let expected = c[j + 2].parse::<f64>().unwrap();
        if *kind == Kind::Pr {
          assert!((value - expected).abs() <= 1e-10);
        } else {
          close(value, expected);
        }
        if let Some(k) = positions.iter().position(|&i| i == row) {
          if *kind == Kind::Mills {
            close(f.fitted_selected_mills[k], value);
          }
          if *kind == Kind::ConditionalSelectedMean {
            close(f.fitted_selected_means[k], value);
          }
        }
        count += 1;
      }
    }
    assert_eq!(count, 1800);
  }
}

#[test]
fn affine_outcome_units_and_large_intercept_shift_preserve_two_step_geometry() {
  let (s, o) = problems("exclusion");
  let f = fit(&s, &o);
  for (scale, shift) in [(1000.0, 0.0), (1e-9, 0.0), (1.0, 1e6)] {
    let mut p = o.clone();
    for y in &mut p.response {
      *y = *y * scale + shift;
    }
    let transformed = fit(&s, &p);
    close(transformed.sigma, f.sigma * scale);
    close(transformed.rho, f.rho);
    for (i, b) in f.coefficients.iter().enumerate() {
      close(
        transformed.coefficients[i].value,
        b.value * scale + if i == 0 { shift } else { 0.0 },
      );
      for (j, v) in f.covariance[i].iter().enumerate() {
        close(transformed.covariance[i][j], v * scale * scale);
      }
    }
    close(transformed.outcome_sse, f.outcome_sse * scale * scale);
  }
}

#[test]
fn prediction_uses_only_relevant_equation_and_handles_unrepresented_mills_explicitly() {
  let (s, o) = problems("exclusion");
  let f = fit(&s, &o);
  close(
    heckman_predict_row(&f, &[0.3], &[f64::NAN], Kind::Xb).unwrap(),
    f.coefficients[0].value + 0.3 * f.coefficients[1].value,
  );
  assert!(heckman_predict_row(&f, &[f64::NAN], &[0.3, 0.4], Kind::Pr).unwrap() > 0.0);
  assert!(heckman_predict_row(&f, &[], &[0.3, 0.4], Kind::Mills).unwrap() > 0.0);
  assert!(heckman_predict_row(&f, &[], &[0.3, 0.4], Kind::ConditionalSelectedMean).is_err());
  let z = (-1000.0 - f.selection.coefficients[0].value) / f.selection.coefficients[2].value;
  assert_eq!(
    heckman_predict_row(&f, &[], &[0.0, z], Kind::Pr).unwrap(),
    0.0
  );
  assert_eq!(
    heckman_predict_row(&f, &[0.0], &[0.0, z], Kind::ObservedContributionMean).unwrap(),
    0.0
  );
  assert!(heckman_predict_row(&f, &[], &[0.0, z], Kind::Mills).is_err());
}

#[test]
fn exact_stage_sample_identities_and_unweighted_boundary_are_required() {
  let (s, o) = problems("exclusion");
  let mut bad = o.clone();
  bad.sample.as_mut().unwrap().retained_indices[0] = 0;
  assert!(
    fit_heckman_two_step(&s, &bad, &ProbitOptions::default())
      .unwrap_err()
      .to_string()
      .contains("exact original-row")
  );
  bad = o.clone();
  bad.sample = None;
  assert!(fit_heckman_two_step(&s, &bad, &ProbitOptions::default()).is_err());
  let mut bad_s = s.clone();
  bad_s.sample.as_mut().unwrap().retained_indices[0] = usize::MAX;
  assert!(fit_heckman_two_step(&bad_s, &o, &ProbitOptions::default()).is_err());
  bad = o.clone();
  bad.weights = Some(vec![1.0; bad.response.len()]);
  assert!(fit_heckman_two_step(&s, &bad, &ProbitOptions::default()).is_err());
  bad_s = s.clone();
  bad_s.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 360]);
  assert!(fit_heckman_two_step(&bad_s, &o, &ProbitOptions::default()).is_err());
  bad_s = s.clone();
  bad_s.sample = None;
  bad = o.clone();
  bad.sample = None;
  assert!(fit_heckman_two_step(&bad_s, &bad, &ProbitOptions::default()).is_ok());
}

#[test]
fn invalid_flags_rank_iteration_names_and_resource_limits_fail_without_mutation() {
  let (s, o) = problems("exclusion");
  let before = (s.clone(), o.clone());
  let mut bad = s.clone();
  bad.response[0] = 2.0;
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  bad = s.clone();
  bad.response.fill(1.0);
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  bad = s.clone();
  bad.response.fill(0.0);
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  bad = s.clone();
  for row in &mut bad.design_matrix {
    row[1] = row[0];
  }
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  bad = s.clone();
  for (row, flag) in bad.design_matrix.iter_mut().zip(&bad.response) {
    row[1] = if *flag == 1.0 { 1.0 } else { -1.0 };
  }
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  let mut bad_o = o.clone();
  bad_o.predictor_names[0] = "mills_coefficient".into();
  assert!(fit_heckman_two_step(&s, &bad_o, &ProbitOptions::default()).is_err());
  bad_o = o.clone();
  for row in &mut bad_o.design_matrix {
    row[0] = 0.0;
  }
  assert!(fit_heckman_two_step(&s, &bad_o, &ProbitOptions::default()).is_err());
  assert!(fit_heckman_two_step(&s, &o, &ProbitOptions { max_iterations: 0 }).is_err());
  bad = s.clone();
  bad.predictor_names = (0..33).map(|i| format!("z{i}")).collect();
  bad.design_matrix = vec![vec![1.0; 33]; 360];
  assert!(fit_heckman_two_step(&bad, &o, &ProbitOptions::default()).is_err());
  assert_eq!((s, o), before);
}

#[test]
fn degenerate_scale_and_out_of_range_rho_reject_instead_of_clipping() {
  let (s, o) = problems("exclusion");
  let f = fit(&s, &o);
  let mut bad = o.clone();
  bad.response.fill(2.0);
  assert!(fit_heckman_two_step(&s, &bad, &ProbitOptions::default()).is_err());
  bad = o.clone();
  for (y, x) in bad.response.iter_mut().zip(&bad.design_matrix) {
    *y = 2.0 + 0.8 * x[0];
  }
  assert!(fit_heckman_two_step(&s, &bad, &ProbitOptions::default()).is_err());
  bad = o.clone();
  for (i, y) in bad.response.iter_mut().enumerate() {
    *y = 2.0 + 0.8 * bad.design_matrix[i][0] + 4.0 * f.fitted_selected_mills[i];
  }
  let error = fit_heckman_two_step(&s, &bad, &ProbitOptions::default())
    .unwrap_err()
    .to_string();
  assert!(error.contains("clipping/fallback"), "{error}");
}

#[test]
fn row_permutation_and_iid_replication_preserve_points_and_scale_covariance() {
  let (mut s, mut o) = problems("exclusion");
  s.sample = None;
  o.sample = None;
  let f = fit(&s, &o);
  let mut permuted_s = s.clone();
  permuted_s.response.reverse();
  permuted_s.design_matrix.reverse();
  let mut permuted_o = o.clone();
  permuted_o.response.reverse();
  permuted_o.design_matrix.reverse();
  let permuted = fit(&permuted_s, &permuted_o);
  for (a, b) in f.coefficients.iter().zip(&permuted.coefficients) {
    close(a.value, b.value);
    close(a.standard_error, b.standard_error);
  }
  close(f.sigma, permuted.sigma);
  close(f.rho, permuted.rho);
  let mut repeated_s = s.clone();
  repeated_s.response.extend_from_slice(&s.response);
  repeated_s.design_matrix.extend_from_slice(&s.design_matrix);
  let mut repeated_o = o.clone();
  repeated_o.response.extend_from_slice(&o.response);
  repeated_o.design_matrix.extend_from_slice(&o.design_matrix);
  let repeated = fit(&repeated_s, &repeated_o);
  for (i, b) in f.coefficients.iter().enumerate() {
    close(b.value, repeated.coefficients[i].value);
    for (j, v) in f.covariance[i].iter().enumerate() {
      close(*v, 2.0 * repeated.covariance[i][j]);
    }
  }
  for (i, row) in f.selection.covariance.iter().enumerate() {
    for (j, v) in row.iter().enumerate() {
      close(*v, 2.0 * repeated.selection.covariance[i][j]);
    }
  }
  close(f.sigma, repeated.sigma);
  close(f.rho, repeated.rho);
}
