use super::*;
const INPUT: &str = include_str!("../../../../tests/fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("../../../../tests/fixtures/cox/parameter_names.txt");
const MODEL: &str = include_str!("../../../../tests/fixtures/cox/model_based.csv");
const ROBUST: &str = include_str!("../../../../tests/fixtures/cox/robust.csv");
const PREDICTIONS: &str = include_str!("../../../../tests/fixtures/cox/predictions.csv");
const SCORES: &str = include_str!("../../../../tests/fixtures/cox/score_residuals.csv");
fn csv(text: &str) -> (Vec<&str>, Vec<Vec<f64>>) {
  let mut l = text.lines();
  (
    l.next().unwrap().split(',').collect(),
    l.map(|r| r.split(',').map(|v| v.parse().unwrap()).collect())
      .collect(),
  )
}
fn close(a: f64, b: f64, label: &str) {
  let tol = if b > 0.0 && (label.ends_with("p_value") || label.contains("prediction_tail")) {
    1e-300 + 1e-8 * b.abs()
  } else {
    1e-9 + 1e-8 * b.abs()
  };
  assert!(
    (a - b).abs() <= tol,
    "{label}: actual={a:.17e} expected={b:.17e}"
  );
}
fn fixture() -> CoxProblem {
  let (h, r) = csv(INPUT);
  let names: Vec<String> = NAMES.lines().map(str::to_owned).collect();
  let at = |r: &[f64], n: &str| r[h.iter().position(|v| *v == n).unwrap()];
  CoxProblem {
    time_variable: "spell".into(),
    failure_variable: "censor1".into(),
    predictor_names: names.clone(),
    time: r.iter().map(|r| at(r, "spell")).collect(),
    events: r.iter().map(|r| at(r, "censor1")).collect(),
    design_matrix: r
      .iter()
      .map(|r| {
        names
          .iter()
          .map(|n| match n.as_str() {
            "rr_ui" => at(r, "reprate") * at(r, "ui"),
            "dr_ui" => at(r, "disrate") * at(r, "ui"),
            _ => at(r, n),
          })
          .collect()
      })
      .collect(),
    sample: Some(EstimationSample::all_retained(r.len())),
  }
}
fn fit(p: &CoxProblem, robust: bool) -> CoxResult {
  fit_cox(
    p,
    &CoxOptions {
      robust,
      ..Default::default()
    },
  )
  .unwrap()
}
fn gates(result: &CoxResult, expected: &str) {
  let data = serde_json::to_value(result).unwrap();
  let mut count = 0;
  for line in expected.lines().skip(1) {
    let (path, v) = line.split_once(',').unwrap();
    close(
      data
        .pointer(path)
        .unwrap_or_else(|| panic!("missing {path}"))
        .as_f64()
        .unwrap(),
      v.parse().unwrap(),
      path,
    );
    count += 1;
  }
  let k = result.coefficients.len();
  assert_eq!(
    count,
    7 * k + k * k + 8 * result.baseline.len() + 12 + usize::from(!result.robust)
  );
}

#[test]
fn cox_full_inference_baseline_and_current_points_match_r_survival() {
  let p = fixture();
  let before = p.clone();
  let (headers, expected) = csv(PREDICTIONS);
  for robust in [false, true] {
    let result = fit(&p, robust);
    gates(&result, if robust { ROBUST } else { MODEL });
    assert_eq!(result.sample, p.sample);
    assert_eq!(result.coefficients.len(), 40);
    assert!(result.iterations > 0);
    assert!(result.max_abs_scaled_score <= 1e-10 && result.max_abs_scaled_newton_step <= 1e-10);
    for i in 0..400 {
      for (prefix, ui) in [("", None), ("ui0_", Some(0.0)), ("ui1_", Some(1.0))] {
        let mut x = p.design_matrix[i].clone();
        if let Some(ui) = ui {
          x[2] = ui;
          x[3] = x[0] * ui;
          x[4] = x[1] * ui;
        }
        for (kind, name) in [
          (CoxPredictionKind::Xb, "xb"),
          (CoxPredictionKind::CumulativeHazard, "cumulative_hazard"),
          (CoxPredictionKind::Survival, "net_survival"),
        ] {
          let column = headers
            .iter()
            .position(|h| *h == format!("{prefix}{name}"))
            .unwrap();
          let value = expected[i][column];
          close(
            cox_predict_row(&result, &x, Some(p.time[i]), kind).unwrap(),
            value,
            if name == "xb" {
              "point"
            } else {
              "prediction_tail"
            },
          );
        }
      }
      let c = headers
        .iter()
        .position(|h| *h == "cumulative_hazard")
        .unwrap();
      close(result.fitted_cumulative_hazard[i], expected[i][c], "fitted");
      close(
        result.fitted_martingale[i],
        p.events[i] - expected[i][c],
        "martingale",
      );
    }
    for i in 0..40 {
      for j in 0..40 {
        assert_eq!(
          result.covariance_matrix[i][j].to_bits(),
          result.covariance_matrix[j][i].to_bits()
        );
      }
    }
  }
  assert_eq!(p, before);
}

#[test]
fn cox_full_efficient_row_scores_match_r_not_naive_martingale_meat() {
  let p = fixture();
  let result = fit(&p, true);
  let beta: Vec<f64> = result.coefficients.iter().map(|c| c.value).collect();
  let z: Vec<Vec<f64>> = p
    .design_matrix
    .iter()
    .map(|r| {
      r.iter()
        .zip(&result.baseline_reference_values)
        .map(|(x, m)| x - m)
        .collect()
    })
    .collect();
  let mut order: Vec<usize> = (0..400).collect();
  order.sort_by(|&i, &j| p.time[i].total_cmp(&p.time[j]));
  let (e, steps) = evaluate(&z, &p.time, &p.events, &order, &beta, true).unwrap();
  let (_, steps) = baseline(steps).unwrap();
  let (_, expected) = csv(SCORES);
  let mut wrong_difference = 0.0;
  for i in 0..400 {
    let (score, h) =
      efficient_score(&z[i], dot(&z[i], &beta), p.time[i], p.events[i], &steps).unwrap();
    for j in 0..40 {
      close(score[j], expected[i][j], "R efficient score");
      wrong_difference += (score[j] - z[i][j] * (p.events[i] - h)).abs();
    }
  }
  assert!(
    wrong_difference > 1.0,
    "baseline influence cannot be omitted"
  );
  assert!(e.score.iter().all(|v| v.abs() < 1e-7));
}

#[test]
fn cox_score_information_finite_differences_and_dynamic_late_risks() {
  let z = vec![
    vec![0.1, 1.0],
    vec![1.0, -0.4],
    vec![-1.0, 0.2],
    vec![0.7, 0.1],
    vec![0.2, 1.4],
    vec![-0.5, -0.7],
  ];
  let t = vec![1.0, 2.0, 2.0, 3.0, 4.0, 5.0];
  let d = vec![1.0, 0.0, 1.0, 0.0, 1.0, 1.0];
  let order = vec![0, 1, 2, 3, 4, 5];
  let theta = vec![0.2, -0.3];
  let e = evaluate(&z, &t, &d, &order, &theta, false).unwrap().0;
  let h = 1e-5;
  for j in 0..2 {
    let mut plus = theta.clone();
    let mut minus = theta.clone();
    plus[j] += h;
    minus[j] -= h;
    let a = evaluate(&z, &t, &d, &order, &plus, false).unwrap().0;
    let b = evaluate(&z, &t, &d, &order, &minus, false).unwrap().0;
    assert!(((a.log_likelihood - b.log_likelihood) / (2.0 * h) - e.score[j]).abs() < 1e-8);
    for l in 0..2 {
      assert!(((a.score[l] - b.score[l]) / (2.0 * h) + e.information[l][j]).abs() < 1e-8);
    }
  }
  // A single global eta maximum makes BOTH late weights zero here.
  let (e, steps) = evaluate(
    &[vec![1.0], vec![1.0], vec![0.0], vec![0.0]],
    &[1.0, 1.0, 2.0, 3.0],
    &[1.0, 0.0, 1.0, 0.0],
    &[0, 1, 2, 3],
    &[1000.0],
    true,
  )
  .unwrap();
  close(e.log_likelihood, -2.0 * 2.0_f64.ln(), "adaptive risk LL");
  assert_eq!(steps[1].risk_count, 2);
  close(steps[1].log_sum, 2.0_f64.ln(), "late risk denominator");
}

#[test]
fn cox_baseline_only_is_nelson_aalen_not_kaplan_meier_and_ties_include_censors() {
  let p = CoxProblem {
    time_variable: "t".into(),
    failure_variable: "d".into(),
    predictor_names: vec![],
    time: vec![1., 1., 2., 3., 3., 4.],
    events: vec![1., 0., 1., 1., 0., 0.],
    design_matrix: vec![vec![]; 6],
    sample: None,
  };
  for robust in [false, true] {
    let r = fit(&p, robust);
    assert!(r.coefficients.is_empty() && r.covariance_matrix.is_empty());
    assert_eq!(r.iterations, 0);
    assert!(r.omnibus_wald.p_value.is_none() && r.likelihood_ratio_p_value.is_none());
    for (i, (risk, cum)) in [(6, 1.0 / 6.0), (4, 5.0 / 12.0), (3, 0.75)]
      .into_iter()
      .enumerate()
    {
      assert_eq!(r.baseline[i].risk_count, risk);
      assert_eq!(r.baseline[i].event_count, 1);
      close(r.baseline[i].cumulative_hazard.unwrap(), cum, "NA baseline");
    }
    close(
      cox_predict_row(&r, &[], Some(4.0), CoxPredictionKind::Survival).unwrap(),
      (-0.75_f64).exp(),
      "exp negative NA",
    );
    assert!(((-0.75_f64).exp() - 5.0 / 12.0).abs() > 0.01);
    close(
      r.fitted_martingale.iter().sum(),
      0.0,
      "baseline martingale sum",
    );
  }
  let mut tied = p;
  tied.events = vec![1., 1., 1., 0., 0., 0.];
  let r = fit(&tied, false);
  assert_eq!(r.baseline.len(), 2);
  assert_eq!(r.baseline[0].risk_count, 6);
  assert_eq!(r.baseline[0].event_count, 2);
  close(
    r.baseline[0].increment.unwrap(),
    1.0 / 3.0,
    "tied NA increment",
  );
}

#[test]
fn cox_row_permutation_time_units_and_affine_design_preserve_fit() {
  let p = fixture();
  let r = fit(&p, true);
  let mut q = p.clone();
  q.time.reverse();
  q.events.reverse();
  q.design_matrix.reverse();
  q.sample.as_mut().unwrap().retained_indices.reverse();
  let s = fit(&q, true);
  assert_eq!(s.sample, q.sample);
  close(r.log_likelihood, s.log_likelihood, "permutation LL");
  for i in 0..40 {
    close(
      r.coefficients[i].value,
      s.coefficients[i].value,
      "permuted beta",
    );
    for j in 0..40 {
      close(
        r.covariance_matrix[i][j],
        s.covariance_matrix[i][j],
        "permuted covariance",
      );
    }
  }
  for i in 0..400 {
    close(
      r.fitted_cumulative_hazard[i],
      s.fitted_cumulative_hazard[399 - i],
      "permuted points",
    );
  }
  let mut swapped = p.clone();
  swapped.predictor_names.swap(3, 39);
  for row in &mut swapped.design_matrix {
    row.swap(3, 39);
  }
  let unrestricted = fit(&swapped, true);
  assert!(unrestricted.coefficients[39].value < 0.0);
  close(
    unrestricted.coefficients[39].value,
    r.coefficients[3].value,
    "unrestricted last coordinate",
  );
  let mut q = p.clone();
  for t in &mut q.time {
    *t /= 8.0;
  }
  for row in &mut q.design_matrix {
    row[0] = 10.0 * row[0] + 100.0;
  }
  let s = fit(&q, true);
  close(r.log_likelihood, s.log_likelihood, "units LL");
  for i in 0..40 {
    let scale = if i == 0 { 10.0 } else { 1.0 };
    close(
      r.coefficients[i].value / scale,
      s.coefficients[i].value,
      "coordinate beta",
    );
    for j in 0..40 {
      let scale2 = if j == 0 { 10.0 } else { 1.0 };
      close(
        r.covariance_matrix[i][j] / scale / scale2,
        s.covariance_matrix[i][j],
        "coordinate covariance",
      );
    }
  }
  for i in 0..400 {
    close(
      r.fitted_cumulative_hazard[i],
      s.fitted_cumulative_hazard[i],
      "time/coordinate points",
    );
  }
}

#[test]
fn cox_invalid_support_rank_options_sample_and_separation_fail() {
  let p = fixture();
  let opts = CoxOptions::default();
  assert!(
    fit_cox(
      &p,
      &CoxOptions {
        max_iterations: 0,
        ..opts.clone()
      }
    )
    .is_err()
  );
  for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
    let mut q = p.clone();
    q.time[0] = value;
    assert!(fit_cox(&q, &opts).is_err());
  }
  for value in [0.5, -1.0, 2.0, f64::NAN] {
    let mut q = p.clone();
    q.events[0] = value;
    assert!(fit_cox(&q, &opts).is_err());
  }
  let mut q = p.clone();
  q.events.fill(0.0);
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.events.pop();
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.design_matrix[0][0] = f64::INFINITY;
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  for row in &mut q.design_matrix {
    row[0] = 1.0;
  }
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  for row in &mut q.design_matrix {
    row[1] = 2.0 * row[0] + 3.0;
  }
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.sample.as_mut().unwrap().retained_indices[0] = 1;
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.predictor_names[0] = q.predictor_names[1].clone();
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.predictor_names = vec!["x".into(); 65];
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.time = vec![1.0; 10001];
  assert!(
    fit_cox(&q, &opts)
      .unwrap_err()
      .to_string()
      .contains("N<=10000")
  );
  let mut q = p.clone();
  q.time = vec![1.0; 6000];
  q.predictor_names = vec!["x".into(); 64];
  assert!(
    fit_cox(&q, &opts)
      .unwrap_err()
      .to_string()
      .contains("N*K^2")
  );
  let mut q = p.clone();
  q.sample.as_mut().unwrap().weights = Some(vec![1.0; 400]);
  assert!(fit_cox(&q, &opts).is_err());
  let mut q = p.clone();
  q.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 400]);
  assert!(fit_cox(&q, &opts).is_err());
  let q = CoxProblem {
    time_variable: "t".into(),
    failure_variable: "d".into(),
    predictor_names: vec!["x".into()],
    time: vec![1., 2., 3., 4.],
    events: vec![1.; 4],
    design_matrix: vec![vec![4.], vec![3.], vec![2.], vec![1.]],
    sample: None,
  };
  assert!(fit_cox(&q, &opts).is_err());
}

#[test]
fn cox_log_baseline_preserves_finite_fitted_hazard_when_natural_reference_overflows() {
  let (reported, scoring) = baseline(vec![RiskStep {
    time: 1.0,
    risk_count: 1,
    event_count: 1,
    log_sum: -1000.0,
    mean: vec![2.0],
  }])
  .unwrap();
  assert!(reported[0].increment.is_none() && reported[0].cumulative_hazard.is_none());
  assert_eq!(reported[0].survival, 0.0);
  close(reported[0].log_cumulative_hazard, 1000.0, "log reference");
  let (score, h) = efficient_score(&[2.0], -1000.0, 1.0, 1.0, &scoring).unwrap();
  close(h, 1.0, "finite fitted hazard");
  assert_eq!(score, vec![0.0]);
}

#[test]
fn cox_prediction_support_zero_and_after_last_steps_and_probability_endpoints() {
  let p = fixture();
  let r = fit(&p, false);
  let x = &p.design_matrix[0];
  close(
    cox_predict_row(&r, x, None, CoxPredictionKind::Xb).unwrap(),
    r.fitted_xb[0].unwrap(),
    "xb no time",
  );
  for t in [0.0, r.baseline[0].time / 2.0] {
    assert_eq!(
      cox_predict_row(&r, x, Some(t), CoxPredictionKind::CumulativeHazard).unwrap(),
      0.0
    );
    assert_eq!(
      cox_predict_row(&r, x, Some(t), CoxPredictionKind::Survival).unwrap(),
      1.0
    );
  }
  close(
    cox_predict_row(&r, x, Some(1e6), CoxPredictionKind::Survival).unwrap(),
    cox_predict_row(
      &r,
      x,
      Some(r.baseline.last().unwrap().time),
      CoxPredictionKind::Survival,
    )
    .unwrap(),
    "last step holds",
  );
  for t in [None, Some(-1.0), Some(f64::NAN), Some(f64::INFINITY)] {
    assert!(cox_predict_row(&r, x, t, CoxPredictionKind::Survival).is_err());
    assert!(cox_predict_row(&r, x, t, CoxPredictionKind::Xb).is_ok());
  }
  let mut large = x.clone();
  large[0] = 10000.0 / r.coefficients[0].value;
  assert_eq!(
    cox_predict_row(&r, &large, Some(1e6), CoxPredictionKind::Survival).unwrap(),
    0.0
  );
  assert!(cox_predict_row(&r, &large, Some(1e6), CoxPredictionKind::CumulativeHazard).is_err());
  assert!(cox_predict_row(&r, &x[..39], Some(1.0), CoxPredictionKind::Xb).is_err());
  let mut bad = x.clone();
  bad[0] = f64::NAN;
  assert!(cox_predict_row(&r, &bad, Some(1.0), CoxPredictionKind::Survival).is_err());
}
