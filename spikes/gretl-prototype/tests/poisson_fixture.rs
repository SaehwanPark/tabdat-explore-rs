#![forbid(unsafe_code)]
#![cfg(feature = "count-plugin")]

use tabdat_gretl_spike::{
  BinaryLink, EstimationProblem, GretlError, run_binary, run_ols, run_poisson,
};

fn fixture() -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/poisson.csv")
      .lines()
      .skip(1)
      .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}

#[test]
fn poisson_matches_pinned_python_and_independent_r() {
  let result = run_poisson(&fixture()).unwrap();
  assert_eq!(result.sample_rows, (0..240).collect::<Vec<_>>());
  assert_eq!(
    result
      .coefficients
      .iter()
      .map(|c| c.name.as_str())
      .collect::<Vec<_>>(),
    ["intercept", "x1", "x2"]
  );
  assert_eq!(result.covariance.len(), 3);
  assert!(result.covariance.iter().all(|row| row.len() == 3));
  let mut actual: Vec<_> = result.coefficients.iter().map(|c| c.value).collect();
  actual.extend(result.coefficients.iter().map(|c| c.standard_error));
  actual.extend(result.covariance.iter().flatten());
  actual.push(result.log_likelihood);
  println!("native Poisson: {actual:?}");
  for line in include_str!("../fixtures/poisson-reference.csv").lines() {
    let fields: Vec<_> = line.split(',').collect();
    assert_eq!(fields[1], "poisson");
    let expected: Vec<f64> = fields[2..].iter().map(|v| v.parse().unwrap()).collect();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
      assert!(a.is_finite() && e.is_finite());
      assert!(
        (a - e).abs() <= 1e-7 + 1e-6 * e.abs(),
        "{}: {a} != {e}",
        fields[0]
      );
    }
  }
}

#[test]
fn poisson_invalid_inputs_and_native_rank_failure_do_not_poison_later_calls() {
  for bad in [-1.0, 0.5, f64::NAN, f64::INFINITY, 1e20] {
    let mut problem = fixture();
    problem.data[0][0] = bad;
    assert!(matches!(
      run_poisson(&problem),
      Err(GretlError::InvalidProblem(_))
    ));
  }
  let mut problem = fixture();
  problem.data[0].pop();
  assert!(matches!(
    run_poisson(&problem),
    Err(GretlError::InvalidProblem(_))
  ));
  problem = fixture();
  problem.data[0][1] = f64::NAN;
  assert!(matches!(
    run_poisson(&problem),
    Err(GretlError::InvalidProblem(_))
  ));
  problem = fixture();
  problem.outcome = "bad\0name".into();
  assert!(matches!(
    run_poisson(&problem),
    Err(GretlError::InvalidProblem(_))
  ));
  problem = fixture();
  problem.data.truncate(3);
  assert!(matches!(
    run_poisson(&problem),
    Err(GretlError::InvalidProblem(_))
  ));
  problem = fixture();
  for row in &mut problem.data {
    row[2] = row[1];
  }
  assert!(run_poisson(&problem).is_err());
  problem = fixture();
  for row in &mut problem.data {
    row[0] = 0.0;
  }
  assert!(run_poisson(&problem).is_err());
  assert!(run_poisson(&fixture()).is_ok());
  assert!(run_ols(&fixture()).is_ok());
  let mut no_intercept = fixture();
  no_intercept.include_intercept = false;
  let result = run_poisson(&no_intercept).unwrap();
  assert_eq!(result.coefficients.len(), 2);
  assert_eq!(result.coefficients[0].name, "x1");
}

#[test]
fn independent_count_binary_and_ols_calls_share_serialized_core() {
  let workers: Vec<_> = (0..4)
    .map(|_| {
      std::thread::spawn(|| {
        for _ in 0..5 {
          assert!(run_poisson(&fixture()).is_ok());
          assert!(run_ols(&fixture()).is_ok());
          let mut binary = fixture();
          for row in &mut binary.data {
            row[0] = f64::from(row[0] > 0.0);
          }
          assert!(run_binary(&binary, BinaryLink::Logit).is_ok());
        }
      })
    })
    .collect();
  for worker in workers {
    worker.join().unwrap();
  }
}
