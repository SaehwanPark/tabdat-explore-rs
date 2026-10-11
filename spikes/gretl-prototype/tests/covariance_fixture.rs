#![forbid(unsafe_code)]

use tabdat_gretl_spike::{
  CovarianceMode, EstimationProblem, run_ols, run_ols_clustered, run_ols_hc1,
};

fn fixture() -> (EstimationProblem, Vec<i32>) {
  let mut data = Vec::new();
  let mut groups = Vec::new();
  for line in include_str!("../fixtures/covariance.csv").lines().skip(1) {
    let row: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
    groups.push(row[3] as i32);
    data.push(row[..3].to_vec());
  }
  (
    EstimationProblem {
      outcome: "y".into(),
      predictors: vec!["x1".into(), "x2".into()],
      include_intercept: true,
      data,
    },
    groups,
  )
}

fn close(actual: f64, expected: f64) {
  assert!(actual.is_finite() && expected.is_finite());
  assert!(
    (actual - expected).abs() <= 1e-10 + 1e-9 * expected.abs(),
    "{actual} != {expected}"
  );
}

#[test]
fn all_covariance_entries_match_python_and_r_sandwich() {
  let (problem, groups) = fixture();
  for line in include_str!("../fixtures/covariance-reference.csv").lines() {
    let fields: Vec<_> = line.split(',').collect();
    let (mode, result) = match fields[1] {
      "classical" => (CovarianceMode::Classical, run_ols(&problem).unwrap()),
      "hc1" => (CovarianceMode::Hc1, run_ols_hc1(&problem).unwrap()),
      "cluster" => (
        CovarianceMode::Clustered,
        run_ols_clustered(&problem, &groups).unwrap(),
      ),
      _ => panic!("unknown fixture mode"),
    };
    assert_eq!(result.nobs, 40);
    assert_eq!(result.ncoeff, 3);
    assert_eq!(result.covariance_mode, mode);
    assert_eq!(result.sample_rows, (0..40).collect::<Vec<_>>());
    assert_eq!(
      result.inference_df,
      if mode == CovarianceMode::Clustered {
        9
      } else {
        37
      }
    );
    assert_eq!(
      result
        .coefficients
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>(),
      ["intercept", "x1", "x2"]
    );
    let mut values: Vec<_> = result.coefficients.iter().map(|c| c.value).collect();
    values.extend(result.coefficients.iter().map(|c| c.standard_error));
    values.extend(result.covariance.iter().flatten());
    let expected: Vec<f64> = fields[2..].iter().map(|v| v.parse().unwrap()).collect();
    assert_eq!(values.len(), expected.len());
    for (actual, expected) in values.into_iter().zip(expected) {
      close(actual, expected);
    }
  }
}

#[test]
fn invalid_groups_and_missing_data_fail_without_poisoning_native_options() {
  let (mut problem, groups) = fixture();
  assert!(run_ols_clustered(&problem, &groups[..39]).is_err());
  assert!(run_ols_clustered(&problem, &[1; 40]).is_err());
  problem.data[0][0] = f64::NAN;
  assert!(run_ols_hc1(&problem).is_err());
  problem.data[0][0] = f64::INFINITY;
  assert!(run_ols_clustered(&problem, &groups).is_err());
  let (problem, groups) = fixture();
  let classical = run_ols(&problem).unwrap();
  let mut singular = problem.clone();
  for row in &mut singular.data {
    row[2] = row[1];
  }
  assert!(run_ols_hc1(&singular).is_err());
  assert!(run_ols_clustered(&singular, &groups).is_err());
  let _ = run_ols_clustered(&problem, &groups).unwrap();
  let _ = run_ols_hc1(&problem).unwrap();
  let after = run_ols(&problem).unwrap();
  assert_eq!(classical.covariance, after.covariance);
}

#[test]
fn concurrent_covariance_modes_do_not_leak_global_settings() {
  let (problem, groups) = fixture();
  let expected = [
    run_ols(&problem).unwrap(),
    run_ols_hc1(&problem).unwrap(),
    run_ols_clustered(&problem, &groups).unwrap(),
  ];
  std::thread::scope(|scope| {
    for mode in 0..3 {
      let problem = &problem;
      let groups = &groups;
      let expected = &expected;
      scope.spawn(move || {
        for _ in 0..10 {
          let result = match mode {
            0 => run_ols(problem).unwrap(),
            1 => run_ols_hc1(problem).unwrap(),
            _ => run_ols_clustered(problem, groups).unwrap(),
          };
          assert_eq!(result.covariance, expected[mode].covariance);
        }
      });
    }
  });
}
