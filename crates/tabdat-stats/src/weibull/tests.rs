use super::*;

// R4.6.1/survival3.8.6, generated with weibull_reference.R; independent
// 400-row full-author-design exponential control, not original copyrighted data.
const INPUT: &str = include_str!("../../../../tests/fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("../../../../tests/fixtures/weibull/parameter_names.txt");
const MODEL: &str = include_str!("../../../../tests/fixtures/weibull/model_based.csv");
const ROBUST: &str = include_str!("../../../../tests/fixtures/weibull/robust.csv");
const PREDICTIONS: &str = include_str!("../../../../tests/fixtures/weibull/predictions.csv");

fn numeric_csv(text: &str) -> (Vec<&str>, Vec<Vec<f64>>) {
  let mut lines = text.lines();
  let headers = lines.next().unwrap().split(',').collect();
  let rows = lines
    .map(|l| l.split(',').map(|v| v.parse().unwrap()).collect())
    .collect();
  (headers, rows)
}
fn fixture() -> (EstimationProblem, Vec<f64>) {
  let names: Vec<String> = NAMES
    .lines()
    .skip(1)
    .filter(|s| *s != "lnshape")
    .map(str::to_owned)
    .collect();
  let (headers, rows) = numeric_csv(INPUT);
  let at = |row: &[f64], name: &str| row[headers.iter().position(|h| *h == name).unwrap()];
  let design = rows
    .iter()
    .map(|r| {
      names
        .iter()
        .map(|name| match name.as_str() {
          "rr_ui" => at(r, "reprate") * at(r, "ui"),
          "dr_ui" => at(r, "disrate") * at(r, "ui"),
          _ => at(r, name),
        })
        .collect()
    })
    .collect();
  let events = rows.iter().map(|r| at(r, "censor1")).collect();
  let time = rows.iter().map(|r| at(r, "spell")).collect();
  let p = EstimationProblem::new("spell", names, time, design)
    .unwrap()
    .with_sample(EstimationSample::all_retained(rows.len()));
  (p, events)
}
fn close(actual: f64, expected: f64, label: &str) {
  let tolerance = if label.ends_with("p_value") && expected > 0.0 {
    1e-300 + 1e-8 * expected.abs()
  } else {
    1e-9 + 1e-8 * expected.abs()
  };
  assert!(
    (actual - expected).abs() <= tolerance,
    "{label}: actual={actual:.17e}, reference={expected:.17e}"
  );
}
fn check_gates(result: &WeibullResult, reference: &str) {
  let data = serde_json::to_value(result).unwrap();
  let mut count = 0;
  for line in reference.lines().skip(1) {
    let (path, value) = line.split_once(',').unwrap();
    let actual = data
      .pointer(path)
      .unwrap_or_else(|| panic!("absent {path}"))
      .as_f64()
      .unwrap_or_else(|| panic!("not numeric {path}"));
    close(actual, value.parse().unwrap(), path);
    count += 1;
  }
  assert!(count >= 2031);
}

#[test]
fn weibull_full_joint_model_and_robust_inference_match_r_survival() {
  let (problem, events) = fixture();
  let before = problem.clone();
  let (headers, predictions) = numeric_csv(PREDICTIONS);
  for robust in [false, true] {
    let result = fit_weibull(
      &problem,
      "censor1",
      &events,
      &WeibullOptions {
        robust,
        ..Default::default()
      },
    )
    .unwrap();
    check_gates(&result, if robust { ROBUST } else { MODEL });
    assert_eq!(result.sample, problem.sample);
    assert_eq!(result.coefficients.len(), 41);
    assert_eq!(result.covariance_matrix.len(), 42);
    assert_eq!(result.likelihood_ratio_p_value.is_none(), robust);
    assert!(result.max_abs_scaled_score <= 1e-10 && result.max_abs_scaled_newton_step <= 1e-10);
    for (i, row) in problem.design_matrix.iter().enumerate() {
      for (prefix, ui) in [("", None), ("ui0_", Some(0.0)), ("ui1_", Some(1.0))] {
        let mut current = row.clone();
        if let Some(ui) = ui {
          current[2] = ui;
          current[3] = ui * current[0];
          current[4] = ui * current[1];
        }
        for (name, kind) in [
          ("xb", WeibullPredictionKind::Xb),
          ("hazard", WeibullPredictionKind::Hazard),
          ("cumulative_hazard", WeibullPredictionKind::CumulativeHazard),
          ("net_survival", WeibullPredictionKind::Survival),
        ] {
          let name = format!("{prefix}{name}");
          let expected = predictions[i][headers.iter().position(|h| *h == name).unwrap()];
          close(
            weibull_predict_row(&result, &current, Some(problem.response[i]), kind).unwrap(),
            expected,
            &name,
          );
        }
      }
      close(
        result.fitted_xb[i],
        weibull_predict_row(&result, row, None, WeibullPredictionKind::Xb).unwrap(),
        "fitted xb",
      );
    }
  }
  assert_eq!(before, problem);
}

#[test]
fn weibull_natural_score_and_full_cross_hessian_match_finite_differences() {
  let x = vec![
    vec![1.0, -0.4],
    vec![1.0, 0.3],
    vec![1.0, 0.8],
    vec![1.0, -1.0],
  ];
  let t: Vec<f64> = [0.5_f64, 1.2, 2.7, 4.0].iter().map(|t| t.ln()).collect();
  let d = vec![1.0, 0.0, 1.0, 0.0];
  let theta = vec![-1.0, 0.2, 1.3];
  let e = evaluate(&x, &t, &d, 1.0, &theta).unwrap();
  assert!(e.information[0][2].abs() > 0.1 && e.information[1][2].abs() > 0.1);
  for j in 0..3 {
    let step = 1e-5 * (1.0 + theta[j].abs());
    let mut plus = theta.clone();
    plus[j] += step;
    let mut minus = theta.clone();
    minus[j] -= step;
    let ep = evaluate(&x, &t, &d, 1.0, &plus).unwrap();
    let em = evaluate(&x, &t, &d, 1.0, &minus).unwrap();
    assert!(((ep.log_likelihood - em.log_likelihood) / (2.0 * step) - e.score[j]).abs() < 1e-7);
    for i in 0..3 {
      assert!((-(ep.score[i] - em.score[i]) / (2.0 * step) - e.information[i][j]).abs() < 1e-7);
    }
  }
}

#[test]
fn weibull_duration_units_transform_intercept_shape_cross_covariance_and_predictions() {
  let (problem, events) = fixture();
  let options = WeibullOptions {
    robust: true,
    ..Default::default()
  };
  let a = fit_weibull(&problem, "censor1", &events, &options).unwrap();
  let mut changed = problem.clone();
  let c = 8.0_f64;
  for t in &mut changed.response {
    *t *= c;
  }
  let b = fit_weibull(&changed, "censor1", &events, &options).unwrap();
  close(b.shape.value, a.shape.value, "shape units");
  close(
    b.coefficients[0].value,
    a.coefficients[0].value - a.shape.value * c.ln(),
    "intercept units",
  );
  for i in 1..41 {
    close(
      b.coefficients[i].value,
      a.coefficients[i].value,
      "slope units",
    );
  }
  // Unlike exponential/fixed shape, Weibull intercept covariance is NOT
  // invariant to time units: d(beta0-a log c)/d(log a)=-a log c.
  let q = 41;
  let derivative = -a.shape.value * c.ln();
  for i in 0..42 {
    for j in 0..42 {
      let mut v = a.covariance_matrix[i][j];
      if i == 0 {
        v += derivative * a.covariance_matrix[q][j];
      }
      if j == 0 {
        v += derivative * a.covariance_matrix[i][q];
      }
      if i == 0 && j == 0 {
        v += derivative * derivative * a.covariance_matrix[q][q];
      }
      close(b.covariance_matrix[i][j], v, "joint covariance units");
    }
  }
  close(
    b.log_likelihood,
    a.log_likelihood - events.iter().sum::<f64>() * c.ln(),
    "density units",
  );
  close(
    b.normalized_log_likelihood,
    a.normalized_log_likelihood,
    "normalized units",
  );
  for i in 0..400 {
    close(
      b.fitted_cumulative_hazard[i],
      a.fitted_cumulative_hazard[i],
      "H units",
    );
    close(b.fitted_survival[i], a.fitted_survival[i], "S units");
    close(b.fitted_hazard[i], a.fitted_hazard[i] / c, "hazard units");
  }
}

#[test]
fn weibull_invalid_support_rank_bounds_options_and_sample_are_rejected() {
  let (p, d) = fixture();
  let options = WeibullOptions::default();
  for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
    let mut bad = p.clone();
    bad.response[0] = value;
    assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  }
  for value in [-1.0, 0.5, 2.0, f64::NAN, f64::INFINITY] {
    let mut bad = d.clone();
    bad[0] = value;
    assert!(fit_weibull(&p, "event", &bad, &options).is_err());
  }
  assert!(fit_weibull(&p, "event", &vec![0.0; 400], &options).is_err());
  assert!(fit_weibull(&p, "event", &d[1..], &options).is_err());
  assert!(fit_weibull(&p, "", &d, &options).is_err());
  for max_iterations in [0, 1] {
    assert!(
      fit_weibull(
        &p,
        "event",
        &d,
        &WeibullOptions {
          max_iterations,
          ..Default::default()
        }
      )
      .is_err()
    );
  }
  let mut bad = p.clone();
  bad.weights = Some(vec![1.0; 400]);
  assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 400]);
  assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().retained_indices[1] = 0;
  assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  for name in ["reprate", "lnshape"] {
    let mut bad = p.clone();
    bad.predictor_names[1] = name.into();
    assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  }
  let mut bad = p.clone();
  for row in &mut bad.design_matrix {
    row[0] = 1.0;
  }
  assert!(fit_weibull(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.response.truncate(42);
  bad.design_matrix.truncate(42);
  bad.sample = None;
  assert!(fit_weibull(&bad, "event", &d[..42], &options).is_err());
}

#[test]
fn weibull_current_prediction_modes_support_exponential_limit_and_probability_endpoints() {
  let (p, d) = fixture();
  let mut result = fit_weibull(&p, "event", &d, &Default::default()).unwrap();
  result.shape.value = 1.0;
  result.shape.log_value = 0.0;
  let row = &p.design_matrix[0];
  let eta = weibull_predict_row(&result, row, None, WeibullPredictionKind::Xb).unwrap();
  for time in [0.25, 1.0, 10.0] {
    close(
      weibull_predict_row(&result, row, Some(time), WeibullPredictionKind::Hazard).unwrap(),
      eta.exp(),
      "exponential h",
    );
    close(
      weibull_predict_row(
        &result,
        row,
        Some(time),
        WeibullPredictionKind::CumulativeHazard,
      )
      .unwrap(),
      time * eta.exp(),
      "exponential H",
    );
    close(
      weibull_predict_row(&result, row, Some(time), WeibullPredictionKind::Survival).unwrap(),
      (-time * eta.exp()).exp(),
      "exponential S",
    );
  }
  for time in [
    None,
    Some(0.0),
    Some(-1.0),
    Some(f64::NAN),
    Some(f64::INFINITY),
  ] {
    assert!(weibull_predict_row(&result, row, time, WeibullPredictionKind::Hazard).is_err());
    assert_eq!(
      weibull_predict_row(&result, row, time, WeibullPredictionKind::Xb).unwrap(),
      eta
    );
  }
  result.coefficients[0].value = 1000.0;
  assert!(
    weibull_predict_row(
      &result,
      row,
      Some(1.0),
      WeibullPredictionKind::CumulativeHazard
    )
    .is_err()
  );
  assert_eq!(
    weibull_predict_row(&result, row, Some(1.0), WeibullPredictionKind::Survival).unwrap(),
    0.0
  );
  assert!(
    weibull_predict_row(
      &result,
      &row[1..],
      Some(1.0),
      WeibullPredictionKind::Survival
    )
    .is_err()
  );
}
