use super::*;
use std::collections::BTreeMap;

type Row = BTreeMap<&'static str, &'static str>;
fn rows(text: &'static str) -> Vec<Row> {
  // These frozen numeric fixtures have no embedded quoted commas/newlines; this
  // deliberately is not a parser for user CSV files.
  let mut lines = text.lines();
  let headers: Vec<_> = lines
    .next()
    .unwrap()
    .split(',')
    .map(|s| s.trim_matches('"'))
    .collect();
  lines
    .map(|line| {
      headers
        .iter()
        .copied()
        .zip(line.split(',').map(|s| s.trim_matches('"')))
        .collect()
    })
    .collect()
}
const METADATA: &str = include_str!("../../../../tests/fixtures/nls/metadata.csv");
const INPUTS: &str = include_str!("../../../../tests/fixtures/nls/inputs.csv");
const PARAMETERS: &str = include_str!("../../../../tests/fixtures/nls/parameters.csv");
const COVARIANCE: &str = include_str!("../../../../tests/fixtures/nls/covariance.csv");
const SUMMARIES: &str = include_str!("../../../../tests/fixtures/nls/summaries.csv");
const PROJECTIONS: &str = include_str!("../../../../tests/fixtures/nls/projections.csv");
const DERIVATIVES: &str = include_str!("../../../../tests/fixtures/nls/derivatives.csv");
fn number(row: &Row, key: &str) -> f64 {
  row[key].parse().unwrap()
}
fn close(actual: f64, expected: f64) {
  assert!(
    actual.is_finite() && (actual - expected).abs() <= 1e-11 + 1e-10 * expected.abs(),
    "actual={actual:.17e}, expected={expected:.17e}"
  );
}
fn boxed(e: NlsExpression) -> Box<NlsExpression> {
  Box::new(e)
}
fn parameter(i: usize) -> NlsExpression {
  NlsExpression::Parameter(i)
}
fn predictor() -> NlsExpression {
  NlsExpression::Predictor(0)
}
fn product(a: NlsExpression, b: NlsExpression) -> NlsExpression {
  NlsExpression::Multiply(boxed(a), boxed(b))
}
fn plus(a: NlsExpression, b: NlsExpression) -> NlsExpression {
  NlsExpression::Add(boxed(a), boxed(b))
}
fn model(meta: &Row) -> NlsModel {
  let expression = match meta["expression"] {
    "a+b*x" => plus(parameter(0), product(parameter(1), predictor())),
    "a*a+b*x" => plus(
      product(parameter(0), parameter(0)),
      product(parameter(1), predictor()),
    ),
    "exp(a*x)" => NlsExpression::Exp(boxed(product(parameter(0), predictor()))),
    "exp(a)" => NlsExpression::Exp(boxed(parameter(0))),
    "a/(b+x)" => NlsExpression::Divide(boxed(parameter(0)), boxed(plus(parameter(1), predictor()))),
    "a*a+a*x" => plus(
      product(parameter(0), parameter(0)),
      product(parameter(0), predictor()),
    ),
    "a*x" => product(parameter(0), predictor()),
    other => panic!("unhandled frozen formula {other}"),
  };
  NlsModel {
    outcome: "y".into(),
    predictor_names: if meta["predictors"].is_empty() {
      vec![]
    } else {
      vec!["x".into()]
    },
    parameter_names: meta["parameters"].split(';').map(str::to_owned).collect(),
    expression,
  }
}
fn data(meta: &Row) -> (Vec<f64>, Vec<Vec<f64>>, Vec<f64>) {
  let inputs: Vec<_> = rows(INPUTS)
    .into_iter()
    .filter(|r| r["case"] == meta["case"])
    .collect();
  let y = inputs.iter().map(|r| number(r, "y")).collect();
  let x = inputs
    .iter()
    .map(|r| {
      if meta["predictors"].is_empty() {
        vec![]
      } else {
        vec![number(r, "x")]
      }
    })
    .collect();
  let start = meta["start"]
    .split(';')
    .map(|v| v.parse().unwrap())
    .collect();
  (y, x, start)
}

#[test]
fn frozen_analytic_values_gradients_and_hessians_cover_all_522_gates() {
  let metadata = rows(METADATA);
  let parameters = rows(PARAMETERS);
  let references = rows(DERIVATIVES);
  assert_eq!(references.len(), 522);
  for reference in references {
    let meta = metadata
      .iter()
      .find(|r| r["case"] == reference["case"])
      .unwrap();
    let model = model(meta);
    let (_, x, start) = data(meta);
    let theta = if reference["point"] == "solution" && meta["status"] == "fit" {
      model
        .parameter_names
        .iter()
        .map(|name| {
          number(
            parameters
              .iter()
              .find(|r| {
                r["case"] == meta["case"] && r["mode"] == "classical" && r["parameter"] == name
              })
              .unwrap(),
            "estimate",
          )
        })
        .collect()
    } else {
      start
    };
    let row = reference["row"].parse::<usize>().unwrap() - 1;
    let evaluation = model.evaluate(&theta, &x[row]).unwrap();
    let actual = match reference["quantity"] {
      "mean" => {
        close(model.predict(&theta, &x[row]).unwrap(), evaluation.mean);
        evaluation.mean
      }
      "gradient" => {
        let i = model
          .parameter_names
          .iter()
          .position(|n| n == reference["parameter"])
          .unwrap();
        evaluation.gradient[i]
      }
      "hessian" => {
        let i = model
          .parameter_names
          .iter()
          .position(|n| n == reference["parameter"])
          .unwrap();
        let j = model
          .parameter_names
          .iter()
          .position(|n| n == reference["column_parameter"])
          .unwrap();
        evaluation.hessian[i][j]
      }
      other => panic!("unknown quantity {other}"),
    };
    close(actual, number(&reference, "value"));
  }
}

#[test]
fn all_six_frozen_fits_match_full_covariance_inference_and_ordered_projections() {
  let mut fitted = 0;
  for meta in rows(METADATA).into_iter().filter(|r| r["status"] == "fit") {
    let model = model(&meta);
    let (y, x, start) = data(&meta);
    let sample = EstimationSample::all_retained(y.len());
    let mut previous = None;
    for (mode, robust) in [("classical", false), ("hc1", true)] {
      let result = fit_nls(&model, &y, &x, &start, &sample, robust).unwrap();
      assert_eq!(result.model, model);
      assert_eq!(result.sample, sample);
      assert_eq!(result.observation_count, y.len());
      assert_eq!(result.residual_degrees_of_freedom, y.len() - start.len());
      assert!(result.normalized_score <= CONVERGENCE);
      assert!(result.relative_full_gauss_newton_step <= CONVERGENCE);
      assert!(result.iterations <= MAX_ITERATIONS && result.evaluation_work <= MAX_NLS_TOTAL_WORK);
      let coefficients: Vec<_> = result.coefficients.iter().map(|c| c.value).collect();
      if let Some(previous) = previous {
        assert_eq!(coefficients, previous);
      }
      previous = Some(coefficients);
      let mut gates = 0;
      for reference in rows(PARAMETERS)
        .into_iter()
        .filter(|r| r["case"] == meta["case"] && r["mode"] == mode)
      {
        let coefficient = result
          .coefficients
          .iter()
          .find(|c| c.name == reference["parameter"])
          .unwrap();
        close(coefficient.value, number(&reference, "estimate"));
        close(
          coefficient.standard_error,
          number(&reference, "standard_error"),
        );
        close(coefficient.statistic.unwrap(), number(&reference, "t"));
        close(coefficient.p_value.unwrap(), number(&reference, "p_value"));
        close(
          coefficient.confidence_lower.unwrap(),
          number(&reference, "ci_lower"),
        );
        close(
          coefficient.confidence_upper.unwrap(),
          number(&reference, "ci_upper"),
        );
        gates += 1;
      }
      assert_eq!(gates, start.len());
      let mut gates = 0;
      for reference in rows(COVARIANCE)
        .into_iter()
        .filter(|r| r["case"] == meta["case"] && r["mode"] == mode)
      {
        let i = model
          .parameter_names
          .iter()
          .position(|n| n == reference["row_parameter"])
          .unwrap();
        let j = model
          .parameter_names
          .iter()
          .position(|n| n == reference["column_parameter"])
          .unwrap();
        close(result.covariance_matrix[i][j], number(&reference, "value"));
        gates += 1;
      }
      assert_eq!(gates, start.len() * start.len());
      let summary = rows(SUMMARIES)
        .into_iter()
        .find(|r| r["case"] == meta["case"] && r["mode"] == mode)
        .unwrap();
      close(result.residual_sum_of_squares, number(&summary, "rss"));
      close(result.root_mean_squared_error, number(&summary, "rmse"));
      close(
        result.centered_r_squared.unwrap(),
        number(&summary, "r_squared"),
      );
      let mut gates = 0;
      for reference in rows(PROJECTIONS)
        .into_iter()
        .filter(|r| r["case"] == meta["case"])
      {
        let i = reference["row"].parse::<usize>().unwrap() - 1;
        close(result.fitted_values[i], number(&reference, "mean"));
        close(result.residuals[i], number(&reference, "residual"));
        gates += 1;
      }
      assert_eq!(gates, y.len());
      fitted += 1;
    }
  }
  assert_eq!(fitted, 12);
}

#[test]
fn stationary_maximum_zero_variance_rank_and_unattained_infimum_are_rejected() {
  let expected = [
    (
      "stationary_nonminimum",
      "not a numerically regular local minimum",
    ),
    (
      "zero_variance",
      "zero or nonrepresentable residual variance",
    ),
    ("rank_deficient", "linearly dependent"),
    ("infimum", "iteration limit reached"),
  ];
  for (name, message) in expected {
    let meta = rows(METADATA)
      .into_iter()
      .find(|r| r["case"] == name)
      .unwrap();
    let model = model(&meta);
    let (y, x, start) = data(&meta);
    let original = (model.clone(), y.clone(), x.clone(), start.clone());
    for robust in [false, true] {
      let error = fit_nls(
        &model,
        &y,
        &x,
        &start,
        &EstimationSample::all_retained(y.len()),
        robust,
      )
      .unwrap_err();
      assert!(error.to_string().contains(message), "{name}: {error}");
    }
    assert_eq!((model, y, x, start), original);
  }
}

#[test]
fn curvature_rescaling_must_not_erase_a_scaled_stationary_maximum_or_ban_zero_hc1() {
  let model = NlsModel {
    outcome: "y".into(),
    predictor_names: vec!["x".into()],
    parameter_names: vec!["a".into()],
    expression: plus(
      product(
        NlsExpression::Constant(1e-250),
        product(parameter(0), parameter(0)),
      ),
      product(parameter(0), predictor()),
    ),
  };
  let y = [0.0, 1e100];
  let x = vec![vec![1e-100], vec![0.0]];
  let sample = EstimationSample::all_retained(2);
  let evaluation = model.evaluate(&[0.0], &x[1]).unwrap();
  assert_eq!(evaluation.hessian[0][0] / 2e-250, 1.0);
  // Raw half-loss curvature=1e-200-2e-150<0. Early h/response_scale
  // underflows, but the correctly scaled average curvature is about -1e50.
  let error = fit_nls(&model, &y, &x, &[0.0], &sample, true).unwrap_err();
  assert!(error.to_string().contains("curvature"), "{error}");

  let regular = NlsModel {
    expression: product(parameter(0), predictor()),
    ..model
  };
  let fit = fit_nls(
    &regular,
    &[1.0, 2.0],
    &[vec![1.0], vec![0.0]],
    &[1.0],
    &sample,
    true,
  )
  .unwrap();
  assert_eq!(fit.covariance_matrix, vec![vec![0.0]]);
  assert_eq!(fit.coefficients[0].standard_error, 0.0);
  assert_eq!(fit.coefficients[0].statistic, None);
  assert_eq!(fit.coefficients[0].confidence_lower, None);
}

#[test]
fn erased_nonzero_derivative_and_covariance_terms_fail_closed() {
  let model = NlsModel {
    outcome: "y".into(),
    predictor_names: vec!["x".into()],
    parameter_names: vec!["a".into()],
    expression: plus(
      product(
        NlsExpression::Constant(1e-200),
        product(
          NlsExpression::Constant(1e-200),
          product(parameter(0), parameter(0)),
        ),
      ),
      product(parameter(0), predictor()),
    ),
  };
  // The ideal mean Hessian2e-400 cannot be stored: do not turn it into a
  // fake zero before a large residual could restore negative loss curvature.
  assert!(
    model
      .evaluate(&[0.0], &[1e-150])
      .unwrap_err()
      .to_string()
      .contains("underflow")
  );
  assert!(
    fit_nls(
      &model,
      &[0.0, 1e150],
      &[vec![1e-150], vec![0.0]],
      &[0.0],
      &EstimationSample::all_retained(2),
      true
    )
    .is_err()
  );

  let regular = NlsModel {
    expression: product(parameter(0), predictor()),
    ..model
  };
  let mut x = vec![vec![1e-200]; 10000];
  x[9999][0] = 0.0;
  let mut y = vec![0.0; 10000];
  y[0] = 1e-223;
  y[1] = -1e-223;
  y[9999] = 1e100;
  // Exact stationary minimum, but HC1 is NONZERO: each of the two informative
  // residual influences is +/-1e-223/(9999*1e-200), giving about2e-54 variance.
  // Multiplication before column rescaling must not silently publish zero HC1.
  let influence = 1e-223 / (9999.0 * 1e-200);
  assert!(2.0 * influence * influence > 1e-54);
  let error = fit_nls(
    &regular,
    &y,
    &x,
    &[0.0],
    &EstimationSample::all_retained(10000),
    true,
  )
  .unwrap_err();
  assert!(error.to_string().contains("underflow"), "{error}");
}

#[test]
fn original_sample_indices_are_owned_ordered_and_unweighted() {
  let meta = rows(METADATA).remove(0);
  let (y, x, start) = data(&meta);
  let model = model(&meta);
  let sample = EstimationSample::new(vec![1, 4, 6, 9, 11, 13, 14], 15, 8, None, None).unwrap();
  assert_eq!(
    fit_nls(&model, &y, &x, &start, &sample, false)
      .unwrap()
      .sample,
    sample
  );
  let mut out_of_range = sample.clone();
  out_of_range.retained_indices[0] = 15;
  let mut duplicate = sample.clone();
  duplicate.retained_indices[1] = 1;
  let mut weighted = sample.clone();
  weighted.weights = Some(vec![1.0; y.len()]);
  let mut clustered = sample.clone();
  clustered.cluster_groups = Some(vec!["g".into(); y.len()]);
  let mut excessive = sample.clone();
  excessive.total_observations = 10001;
  excessive.dropped_observations = 10001 - y.len();
  for invalid_sample in [out_of_range, duplicate, weighted, clustered, excessive] {
    assert!(fit_nls(&model, &y, &x, &start, &invalid_sample, false).is_err());
  }
}

#[test]
fn global_response_units_preserve_normalized_linear_results_including_tiny_signals() {
  let meta = rows(METADATA).remove(0);
  let (y, x, start) = data(&meta);
  let model = model(&meta);
  let sample = EstimationSample::all_retained(y.len());
  for robust in [false, true] {
    let baseline = fit_nls(&model, &y, &x, &start, &sample, robust).unwrap();
    for scale in [1e-100, 1e100] {
      let scaled_y: Vec<_> = y.iter().map(|v| v * scale).collect();
      let result = fit_nls(&model, &scaled_y, &x, &start, &sample, robust).unwrap();
      for (i, coefficient) in result.coefficients.iter().enumerate() {
        close(coefficient.value / scale, baseline.coefficients[i].value);
        close(
          coefficient.standard_error / scale,
          baseline.coefficients[i].standard_error,
        );
        for j in 0..start.len() {
          close(
            result.covariance_matrix[i][j] / scale / scale,
            baseline.covariance_matrix[i][j],
          );
        }
      }
      for i in 0..y.len() {
        close(result.fitted_values[i] / scale, baseline.fitted_values[i]);
        close(result.residuals[i] / scale, baseline.residuals[i]);
      }
    }
  }
}

#[test]
fn expression_indices_names_depth_nodes_and_finite_arithmetic_are_validated() {
  let meta = rows(METADATA).remove(0);
  let base = model(&meta);
  let mut bad = base.clone();
  bad.parameter_names[1] = "a".into();
  assert!(bad.validate().is_err());
  let mut bad = base.clone();
  bad.outcome = "a".into();
  assert!(bad.validate().is_err());
  let mut bad = base.clone();
  bad.expression = parameter(2);
  assert!(bad.validate().is_err());
  let mut bad = base.clone();
  bad.expression = NlsExpression::Predictor(2);
  assert!(bad.validate().is_err());
  let mut bad = base.clone();
  bad.expression = parameter(0);
  assert!(bad.validate().is_err());
  let mut bad = base.clone();
  bad.expression = plus(parameter(0), NlsExpression::Constant(f64::INFINITY));
  assert!(bad.validate().is_err());
  let mut depth = parameter(0);
  for _ in 0..MAX_NLS_DEPTH {
    depth = NlsExpression::Negate(boxed(depth));
  }
  let mut bad = base.clone();
  bad.expression = depth;
  assert!(bad.validate().is_err());
  let mut nodes = vec![parameter(0); 65];
  while nodes.len() > 1 {
    let mut next = Vec::new();
    for pair in nodes.chunks(2) {
      next.push(if pair.len() == 2 {
        plus(pair[0].clone(), pair[1].clone())
      } else {
        pair[0].clone()
      });
    }
    nodes = next;
  }
  let mut bad = base.clone();
  bad.expression = nodes.remove(0);
  assert!(bad.validate().is_err());
  let exp = NlsModel {
    outcome: "y".into(),
    predictor_names: vec![],
    parameter_names: vec!["a".into()],
    expression: NlsExpression::Exp(boxed(parameter(0))),
  };
  for a in [-1000.0, 1000.0, f64::NAN] {
    assert!(exp.evaluate(&[a], &[]).is_err());
    assert!(exp.predict(&[a], &[]).is_err());
  }
  let rational_meta = rows(METADATA)
    .into_iter()
    .find(|r| r["case"] == "rational_parameter")
    .unwrap();
  assert!(model(&rational_meta).evaluate(&[3., 0.], &[0.]).is_err());
  assert!(base.evaluate(&[1.], &[0.]).is_err());
}

#[test]
fn start_sample_and_resource_limits_fail_without_partial_fits() {
  let meta = rows(METADATA).remove(0);
  let (y, x, start) = data(&meta);
  let model = model(&meta);
  let sample = EstimationSample::all_retained(y.len());
  for bad in [vec![1.0], vec![1., f64::NAN]] {
    assert!(fit_nls(&model, &y, &x, &bad, &sample, false).is_err());
  }
  let mut bad_y = y.clone();
  bad_y[0] = f64::INFINITY;
  assert!(fit_nls(&model, &bad_y, &x, &start, &sample, false).is_err());
  assert!(
    fit_nls(
      &model,
      &y[..2],
      &x[..2],
      &start,
      &EstimationSample::all_retained(2),
      false
    )
    .is_err()
  );
  let mut expression = parameter(0);
  for i in 1..MAX_NLS_PARAMETERS {
    expression = plus(expression, parameter(i));
  }
  let large = NlsModel {
    outcome: "y".into(),
    predictor_names: vec![],
    parameter_names: (0..MAX_NLS_PARAMETERS).map(|i| format!("a{i}")).collect(),
    expression,
  };
  assert!(
    fit_nls(
      &large,
      &vec![1.; 10000],
      &vec![vec![]; 10000],
      &vec![0.; MAX_NLS_PARAMETERS],
      &EstimationSample::all_retained(10000),
      false
    )
    .unwrap_err()
    .to_string()
    .contains("work within bounds")
  );
  let mut budget = Budget {
    used: MAX_NLS_TOTAL_WORK - 1,
  };
  assert!(budget.spend(2).is_err());
  assert_eq!(budget.used, MAX_NLS_TOTAL_WORK - 1);
  let mut budget = Budget { used: usize::MAX };
  assert!(budget.spend(1).is_err());
}
