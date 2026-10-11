#![forbid(unsafe_code)]
use tabdat_gretl_spike::{BinaryLink, EstimationProblem, run_binary};

fn fixture() -> EstimationProblem {
  EstimationProblem {
    outcome: "y".into(),
    predictors: vec!["x1".into(), "x2".into()],
    include_intercept: true,
    data: include_str!("../fixtures/binary.csv")
      .lines()
      .skip(1)
      .map(|line| line.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  }
}

#[test]
fn binary_mle_matches_python_and_r_observed_information() {
  for line in include_str!("../fixtures/binary-reference.csv").lines() {
    let fields: Vec<_> = line.split(',').collect();
    let link = if fields[1] == "logit" {
      BinaryLink::Logit
    } else {
      BinaryLink::Probit
    };
    let result = run_binary(&fixture(), link).unwrap();
    assert_eq!(result.link, link);
    assert_eq!(result.sample_rows, (0..200).collect::<Vec<_>>());
    assert_eq!(
      result
        .coefficients
        .iter()
        .map(|c| c.name.as_str())
        .collect::<Vec<_>>(),
      ["intercept", "x1", "x2"]
    );
    let mut actual: Vec<_> = result.coefficients.iter().map(|c| c.value).collect();
    actual.extend(result.coefficients.iter().map(|c| c.standard_error));
    actual.extend(result.covariance.iter().flatten());
    actual.push(result.log_likelihood);
    let expected: Vec<f64> = fields[2..].iter().map(|v| v.parse().unwrap()).collect();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.into_iter().zip(expected) {
      assert!(a.is_finite() && e.is_finite());
      assert!((a - e).abs() <= 1e-7 + 1e-6 * e.abs(), "{a} != {e}");
    }
  }
}

#[test]
fn binary_rejects_invalid_outcomes_missing_data_and_separation() {
  for link in [BinaryLink::Logit, BinaryLink::Probit] {
    let mut problem = fixture();
    problem.data[0][0] = 2.0;
    assert!(run_binary(&problem, link).is_err());
    problem = fixture();
    problem.data[0][1] = f64::NAN;
    assert!(run_binary(&problem, link).is_err());
    problem = fixture();
    problem.data[0].pop();
    assert!(run_binary(&problem, link).is_err());
    problem = fixture();
    for row in &mut problem.data {
      row[0] = f64::from(row[1] > 0.0);
    }
    assert!(run_binary(&problem, link).is_err());
    assert!(run_binary(&fixture(), link).is_ok());
  }
}
