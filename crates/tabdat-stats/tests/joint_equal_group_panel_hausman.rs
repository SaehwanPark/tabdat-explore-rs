use tabdat_stats::{
  EstimationProblem, EstimationSample, fit_random_effects_equal_group_between,
  joint_equal_group_panel_hausman,
};

fn problem(data: &str) -> EstimationProblem {
  let rows: Vec<_> = data
    .lines()
    .skip(1)
    .map(|l| l.split(',').collect::<Vec<_>>())
    .collect();
  let mut response = Vec::new();
  let mut design = Vec::new();
  let mut groups = Vec::new();
  let mut retained = Vec::new();
  for (i, row) in rows.iter().enumerate() {
    if let Ok(y) = row[1].parse::<f64>() {
      response.push(y);
      design.push([7, 2, 3, 4, 5, 6].map(|j| row[j].parse().unwrap()).to_vec());
      groups.push(row[8].to_string());
      retained.push(i);
    }
  }
  let n = response.len();
  let sample =
    EstimationSample::new(retained, rows.len(), rows.len() - n, None, Some(groups)).unwrap();
  EstimationProblem::new(
    "lnexp12m",
    ["lnhhexp", "age", "sex", "hhsize", "farm", "educ"]
      .map(str::to_owned)
      .to_vec(),
    response,
    design,
  )
  .unwrap()
  .with_sample(sample)
}
fn check(data: &str, reference: &str, balanced: bool) {
  let p = problem(data);
  let re = fit_random_effects_equal_group_between(&p, "commune").unwrap();
  let saved = re.clone();
  let h = joint_equal_group_panel_hausman(&p, &re).unwrap();
  assert_eq!(re, saved);
  assert_eq!(h.minimum_group_size == h.maximum_group_size, balanced);
  assert_eq!(h.candidate_observation_count, data.lines().count() - 1);
  assert_eq!(h.dropped_observation_count, usize::from(!balanced));
  assert_eq!(h.df, 6);
  let value = serde_json::to_value(&h).unwrap();
  let mut count = 0;
  for line in reference.lines().skip(1) {
    let (path, expected) = line.split_once(',').unwrap();
    let expected: f64 = expected.parse().unwrap();
    let actual = value.pointer(path).unwrap().as_f64().unwrap();
    let tolerance = if path == "/p_value" {
      expected.abs() * 1e-8
    } else {
      1e-9
    };
    assert!(
      (actual - expected).abs() <= tolerance,
      "{path}: {actual} != {expected}"
    );
    count += 1;
  }
  assert_eq!(count, 139);
}
#[test]
fn balanced_full_joint_covariance_matches_independent_qr() {
  check(
    include_str!("fixtures/panel_joint_equal_group_hausman/balanced_input.csv"),
    include_str!("fixtures/panel_joint_equal_group_hausman/balanced_reference.csv"),
    true,
  );
}
#[test]
fn unbalanced_singleton_and_exact_sample_match_independent_qr() {
  check(
    include_str!("fixtures/panel_joint_equal_group_hausman/unbalanced_input.csv"),
    include_str!("fixtures/panel_joint_equal_group_hausman/unbalanced_reference.csv"),
    false,
  );
}
#[test]
fn predictor_permutation_and_units_preserve_the_joint_test() {
  let p = problem(include_str!(
    "fixtures/panel_joint_equal_group_hausman/unbalanced_input.csv"
  ));
  let re = fit_random_effects_equal_group_between(&p, "commune").unwrap();
  let expected = joint_equal_group_panel_hausman(&p, &re).unwrap();
  let mut transformed = p.clone();
  transformed.predictor_names.reverse();
  let scales = [1e-4, -1e4, 2.0, 0.5, 8.0, -0.25];
  for row in &mut transformed.design_matrix {
    row.reverse();
    for (v, scale) in row.iter_mut().zip(scales) {
      *v *= scale;
    }
  }
  let re = fit_random_effects_equal_group_between(&transformed, "commune").unwrap();
  let actual = joint_equal_group_panel_hausman(&transformed, &re).unwrap();
  assert!((actual.statistic - expected.statistic).abs() < 1e-9);
  assert!((actual.p_value - expected.p_value).abs() < 1e-9);
}
#[test]
fn retained_fit_forgery_and_unsupported_methods_are_rejected() {
  let p = problem(include_str!(
    "fixtures/panel_joint_equal_group_hausman/unbalanced_input.csv"
  ));
  let re = fit_random_effects_equal_group_between(&p, "commune").unwrap();
  let mut forged = re.clone();
  forged.gls.coefficients[0].value += 0.01;
  assert!(
    joint_equal_group_panel_hausman(&p, &forged)
      .unwrap_err()
      .to_string()
      .contains("does not match")
  );
  let mut forged = re.clone();
  forged.gls.covariance.matrix[1][1] *= 1.01;
  assert!(joint_equal_group_panel_hausman(&p, &forged).is_err());
  let mut forged = re.clone();
  forged.component_method = tabdat_stats::RandomEffectsComponentMethod::SwamyArora;
  assert!(joint_equal_group_panel_hausman(&p, &forged).is_err());
  let mut altered = p.clone();
  altered.response[0] += 1.0;
  assert!(joint_equal_group_panel_hausman(&altered, &re).is_err());
  let mut altered = p.clone();
  altered.include_intercept = false;
  assert!(joint_equal_group_panel_hausman(&altered, &re).is_err());
}
