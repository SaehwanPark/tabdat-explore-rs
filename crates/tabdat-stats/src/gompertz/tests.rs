use super::*;

// R4.6.1 integrated time moments + conditional GLM/profile-score root; unlike
// native joint Newton/closed-form moments. Only existing synthetic400-row input.
const INPUT: &str = include_str!("../../../../tests/fixtures/duration_exponential/input.csv");
const NAMES: &str = include_str!("../../../../tests/fixtures/gompertz/parameter_names.txt");
const MODEL: &str = include_str!("../../../../tests/fixtures/gompertz/model_based.csv");
const ROBUST: &str = include_str!("../../../../tests/fixtures/gompertz/robust.csv");
const PREDICTIONS: &str = include_str!("../../../../tests/fixtures/gompertz/predictions.csv");

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
    .filter(|s| *s != "gamma")
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
fn check_gates(result: &GompertzResult, reference: &str) {
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
  let k = result.coefficients.len();
  assert_eq!(
    count,
    6 * k + (k + 1) * (k + 1) + 6 + if result.robust { 13 } else { 14 }
  );
}

#[test]
fn gompertz_full_joint_model_and_robust_inference_match_independent_r_profile() {
  let (problem, events) = fixture();
  let before = problem.clone();
  let (headers, predictions) = numeric_csv(PREDICTIONS);
  for robust in [false, true] {
    let result = fit_gompertz(
      &problem,
      "censor1",
      &events,
      &GompertzOptions {
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
          ("xb", GompertzPredictionKind::Xb),
          ("hazard", GompertzPredictionKind::Hazard),
          (
            "cumulative_hazard",
            GompertzPredictionKind::CumulativeHazard,
          ),
          ("net_survival", GompertzPredictionKind::Survival),
        ] {
          let name = format!("{prefix}{name}");
          let expected = predictions[i][headers.iter().position(|h| *h == name).unwrap()];
          close(
            gompertz_predict_row(&result, &current, Some(problem.response[i]), kind).unwrap(),
            expected,
            &name,
          );
        }
      }
      for (actual, name) in [
        (result.fitted_xb[i], "xb"),
        (result.fitted_hazard[i], "hazard"),
        (result.fitted_cumulative_hazard[i], "cumulative_hazard"),
        (result.fitted_survival[i], "net_survival"),
      ] {
        close(
          actual,
          predictions[i][headers.iter().position(|h| *h == name).unwrap()],
          name,
        );
      }
    }
  }
  assert_eq!(before, problem);
}

#[test]
fn gompertz_no_intercept_joint_inference_and_shape_refitted_null_match_r() {
  const MODEL: &str =
    include_str!("../../../../tests/fixtures/gompertz/no_intercept/model_based.csv");
  const ROBUST: &str = include_str!("../../../../tests/fixtures/gompertz/no_intercept/robust.csv");
  let (problem, events) = fixture();
  let problem = problem.with_intercept(false);
  for robust in [false, true] {
    let result = fit_gompertz(
      &problem,
      "censor1",
      &events,
      &GompertzOptions {
        robust,
        ..Default::default()
      },
    )
    .unwrap();
    check_gates(&result, if robust { ROBUST } else { MODEL });
    assert!(!result.include_intercept);
    assert_eq!(result.coefficients[0].name, "reprate");
    assert_eq!(result.coefficients.len(), 40);
    assert_eq!(result.likelihood_ratio_df, 40);
    assert_eq!(result.omnibus_wald.degrees_of_freedom, 40);
  }
}

#[test]
fn gompertz_time_moments_match_independent_quadrature_at_zero_both_signs_and_tails() {
  let (_, rows) = numeric_csv(include_str!(
    "../../../../tests/fixtures/gompertz/moments.csv"
  ));
  assert_eq!(rows.len(), 15);
  for r in rows {
    let actual = time_moments(r[0]).unwrap();
    for (value, expected) in [
      (actual.log_m0, r[1]),
      (actual.mean_u, r[2]),
      (actual.mean_u2, r[3]),
    ] {
      // Quadrature uses rel.tol1e-12. Absolute bound covers values near zero;
      // relative bound also checks the much smaller negative-tail moments.
      assert!(
        (value - expected).abs() <= 1e-13 + 1e-12 * expected.abs(),
        "z={} actual={value:.17e} reference={expected:.17e}",
        r[0]
      );
    }
  }
  let z = time_moments(0.0).unwrap();
  assert_eq!(z.log_m0, 0.0);
  assert_eq!(z.mean_u, 0.5);
  assert_eq!(z.mean_u2, 1.0 / 3.0);
  for z in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
    assert!(time_moments(z).is_err());
  }
}

#[test]
fn gompertz_unrestricted_score_and_all_cross_hessian_blocks_match_finite_differences() {
  let x = vec![
    vec![1.0, -0.4],
    vec![1.0, 0.3],
    vec![1.0, 0.8],
    vec![1.0, -1.0],
  ];
  let times = [0.5_f64, 1.2, 2.7, 4.0];
  let log_time: Vec<f64> = times.iter().map(|t| t.ln()).collect();
  let tau: Vec<f64> = times.iter().map(|t| t / 4.0).collect();
  let d = vec![1.0, 0.0, 1.0, 0.0];
  for gamma in [-8.0, -0.4, 0.0, 0.4, 8.0] {
    let theta = vec![-1.0, 0.2, gamma];
    let e = evaluate(&x, &log_time, &tau, &d, &theta).unwrap();
    assert!(e.information[0][2].abs() > 1e-3 && e.information[1][2].abs() > 1e-3);
    for j in 0..3 {
      let step = 1e-5 * (1.0 + theta[j].abs());
      let mut plus = theta.clone();
      plus[j] += step;
      let mut minus = theta.clone();
      minus[j] -= step;
      let ep = evaluate(&x, &log_time, &tau, &d, &plus).unwrap();
      let em = evaluate(&x, &log_time, &tau, &d, &minus).unwrap();
      // Central differences have O(step^2) truncation and subtraction error;
      // this is a derivative certificate, not relaxed native reference gates.
      let numeric = (ep.log_likelihood - em.log_likelihood) / (2.0 * step);
      assert!(
        (numeric - e.score[j]).abs() <= 1e-7 + 1e-7 * e.score[j].abs(),
        "gamma={gamma} score[{j}]"
      );
      for i in 0..3 {
        let numeric = -(ep.score[i] - em.score[i]) / (2.0 * step);
        assert!(
          (numeric - e.information[i][j]).abs() <= 1e-7 + 1e-7 * e.information[i][j].abs(),
          "gamma={gamma} information[{i}][{j}]"
        );
      }
    }
  }
}

fn baseline_problem() -> EstimationProblem {
  EstimationProblem::new("time", vec![], vec![1.0, 2.0, 3.0, 4.0], vec![vec![]; 4]).unwrap()
}

#[test]
fn gompertz_zero_gamma_is_regular_and_matches_exact_exponential_controls() {
  // d=[1,1,0,0]: rate=D/sum(t)=1/5 and gamma score is exactly zero.
  // FULL observed I=[[2,3],[3,20/3]], not the fixed-shape Poisson variance1/2.
  let problem = baseline_problem();
  let events = [1.0, 1.0, 0.0, 0.0];
  let r = fit_gompertz(&problem, "event", &events, &Default::default()).unwrap();
  close(
    r.coefficients[0].value,
    0.2_f64.ln(),
    "exact exponential intercept",
  );
  assert!(r.shape.value.abs() < 1e-12);
  assert!(r.shape.confidence_lower.unwrap() < 0.0 && r.shape.confidence_upper.unwrap() > 0.0);
  for (i, row) in [[20.0 / 13.0, -9.0 / 13.0], [-9.0 / 13.0, 6.0 / 13.0]]
    .iter()
    .enumerate()
  {
    for (j, expected) in row.iter().enumerate() {
      close(
        r.covariance_matrix[i][j],
        *expected,
        "zero gamma joint covariance",
      );
    }
  }
  assert_eq!(r.likelihood_ratio_df, 0);
  assert!(r.likelihood_ratio_p_value.is_none());
  for t in [0.0, 0.5, 1.0, 4.0, 1000.0] {
    for (kind, expected) in [
      (GompertzPredictionKind::Hazard, 0.2),
      (GompertzPredictionKind::CumulativeHazard, 0.2 * t),
      (GompertzPredictionKind::Survival, (-0.2_f64 * t).exp()),
    ] {
      close(
        gompertz_predict_row(&r, &[], Some(t), kind).unwrap(),
        expected,
        "zero gamma point",
      );
    }
  }
}

#[test]
fn gompertz_negative_gamma_fit_full_joint_covariance_and_nonzero_tail_match_r() {
  // Independent R direct integration over [0,t], analytic conditional intercept
  // and scalar profile root; reproducible snippet in fixtures/gompertz/README.md.
  let problem = baseline_problem();
  let events = [1.0, 0.0, 0.0, 0.0];
  for (robust, expected) in [
    (
      false,
      [
        [2.428_300_983_839_824, -1.4283009838398237],
        [-1.4283009838398235, 1.4283009838398233],
      ],
    ),
    (
      true,
      [
        [0.953_646_171_321_643_5, 0.137_247_823_199_049_88],
        [0.137_247_823_199_050_3, 0.052_753_919_024_554_51],
      ],
    ),
  ] {
    let r = fit_gompertz(
      &problem,
      "event",
      &events,
      &GompertzOptions {
        robust,
        ..Default::default()
      },
    )
    .unwrap();
    close(
      r.coefficients[0].value,
      -1.613_457_975_074_667_4,
      "negative gamma intercept",
    );
    close(r.shape.value, -0.559_430_528_002_073_2, "negative gamma");
    close(
      r.log_likelihood,
      -3.172_888_503_076_740_6,
      "negative gamma density",
    );
    for i in 0..2 {
      for j in 0..2 {
        close(
          r.covariance_matrix[i][j],
          expected[i][j],
          "negative gamma joint covariance",
        );
      }
    }
    let limit = (-r.coefficients[0].value.exp() / (-r.shape.value)).exp();
    assert!(limit > 0.5 && limit < 1.0);
    close(
      gompertz_predict_row(&r, &[], Some(1e6), GompertzPredictionKind::Survival).unwrap(),
      limit,
      "negative gamma positive tail",
    );
    assert_eq!(
      gompertz_predict_row(&r, &[], Some(1e6), GompertzPredictionKind::Hazard).unwrap(),
      0.0
    );
  }
}

#[test]
fn gompertz_duration_units_transform_natural_gamma_and_full_cross_covariance() {
  let (problem, events) = fixture();
  let c = 8.0_f64;
  let mut changed = problem.clone();
  for t in &mut changed.response {
    *t *= c;
  }
  for robust in [false, true] {
    let options = GompertzOptions {
      robust,
      ..Default::default()
    };
    let a = fit_gompertz(&problem, "event", &events, &options).unwrap();
    let b = fit_gompertz(&changed, "event", &events, &options).unwrap();
    close(b.shape.value, a.shape.value / c, "gamma time units");
    close(
      b.coefficients[0].value,
      a.coefficients[0].value - c.ln(),
      "intercept time units",
    );
    for i in 1..41 {
      close(
        b.coefficients[i].value,
        a.coefficients[i].value,
        "slope time units",
      );
    }
    for i in 0..42 {
      for j in 0..42 {
        let divisor = if i == 41 { c } else { 1.0 } * if j == 41 { c } else { 1.0 };
        close(
          b.covariance_matrix[i][j],
          a.covariance_matrix[i][j] / divisor,
          "full covariance time units",
        );
      }
    }
    close(
      b.shape.standard_error,
      a.shape.standard_error / c,
      "gamma SE time units",
    );
    close(
      b.log_likelihood,
      a.log_likelihood - events.iter().sum::<f64>() * c.ln(),
      "density time units",
    );
    close(
      b.normalized_log_likelihood,
      a.normalized_log_likelihood,
      "normalized time units",
    );
    for i in 0..400 {
      close(b.fitted_xb[i], a.fitted_xb[i] - c.ln(), "xb time units");
      close(
        b.fitted_cumulative_hazard[i],
        a.fitted_cumulative_hazard[i],
        "H time units",
      );
      close(b.fitted_survival[i], a.fitted_survival[i], "S time units");
      close(b.fitted_hazard[i], a.fitted_hazard[i] / c, "h time units");
    }
  }
}

#[test]
fn gompertz_affine_x_reparameterization_maps_all_joint_covariance_and_points() {
  let (p, d) = fixture();
  let options = GompertzOptions {
    robust: true,
    ..Default::default()
  };
  let a = fit_gompertz(&p, "event", &d, &options).unwrap();
  let mut p = p;
  let shift = 3.0;
  let scale = 2.0;
  let j = 5;
  for row in &mut p.design_matrix {
    row[j] = shift + scale * row[j];
  }
  let b = fit_gompertz(&p, "event", &d, &options).unwrap();
  let j = j + 1;
  let n = a.covariance_matrix.len();
  close(
    b.coefficients[0].value,
    a.coefficients[0].value - shift / scale * a.coefficients[j].value,
    "affine intercept",
  );
  close(
    b.coefficients[j].value,
    a.coefficients[j].value / scale,
    "affine slope",
  );
  let mut jacobian = vec![vec![0.0; n]; n];
  for i in 0..n {
    jacobian[i][i] = 1.0;
  }
  jacobian[0][j] = -shift / scale;
  jacobian[j][j] = 1.0 / scale;
  let left: Vec<Vec<f64>> = (0..n)
    .map(|i| {
      (0..n)
        .map(|j| {
          (0..n)
            .map(|k| jacobian[i][k] * a.covariance_matrix[k][j])
            .sum::<f64>()
        })
        .collect()
    })
    .collect();
  for i in 0..n {
    for j in 0..n {
      let expected = (0..n).map(|k| left[i][k] * jacobian[j][k]).sum();
      close(
        b.covariance_matrix[i][j],
        expected,
        "affine full joint covariance",
      );
    }
  }
  for i in 0..400 {
    close(b.fitted_xb[i], a.fitted_xb[i], "affine xb");
    close(b.fitted_survival[i], a.fitted_survival[i], "affine S");
  }
}

#[test]
fn gompertz_row_permutation_preserves_inference_and_original_sample_identity() {
  let (p, mut d) = fixture();
  let options = GompertzOptions {
    robust: true,
    ..Default::default()
  };
  let a = fit_gompertz(&p, "event", &d, &options).unwrap();
  let mut changed = p.clone();
  changed.response.reverse();
  changed.design_matrix.reverse();
  d.reverse();
  changed.sample.as_mut().unwrap().retained_indices.reverse();
  let b = fit_gompertz(&changed, "event", &d, &options).unwrap();
  close(a.shape.value, b.shape.value, "permuted gamma");
  close(a.log_likelihood, b.log_likelihood, "permuted density");
  for i in 0..41 {
    close(
      a.coefficients[i].value,
      b.coefficients[i].value,
      "permuted beta",
    );
  }
  for i in 0..42 {
    for j in 0..42 {
      close(
        a.covariance_matrix[i][j],
        b.covariance_matrix[i][j],
        "permuted full covariance",
      );
    }
  }
  assert_eq!(b.sample, changed.sample);
  assert_eq!(b.sample.as_ref().unwrap().retained_indices[0], 399);
  for i in 0..400 {
    close(
      b.fitted_cumulative_hazard[i],
      a.fitted_cumulative_hazard[399 - i],
      "permuted H",
    );
  }
}

#[test]
fn gompertz_rejects_invalid_support_rank_resources_options_and_sample() {
  let (p, d) = fixture();
  let options = GompertzOptions::default();
  for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
    let mut bad = p.clone();
    bad.response[0] = value;
    assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  }
  for value in [-1.0, 0.5, 2.0, f64::NAN, f64::INFINITY] {
    let mut bad = d.clone();
    bad[0] = value;
    assert!(fit_gompertz(&p, "event", &bad, &options).is_err());
  }
  assert!(fit_gompertz(&p, "event", &vec![0.0; 400], &options).is_err());
  assert!(fit_gompertz(&p, "event", &d[1..], &options).is_err());
  assert!(fit_gompertz(&p, "", &d, &options).is_err());
  for max_iterations in [0, 1] {
    assert!(
      fit_gompertz(
        &p,
        "event",
        &d,
        &GompertzOptions {
          max_iterations,
          ..Default::default()
        }
      )
      .is_err()
    );
  }
  let mut bad = p.clone();
  bad.weights = Some(vec![1.0; 400]);
  assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().cluster_groups = Some(vec!["a".into(); 400]);
  assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().weights = Some(vec![1.0; 400]);
  assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.sample.as_mut().unwrap().retained_indices[1] = 0;
  assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  for name in ["reprate", "gamma", "GAMMA"] {
    let mut bad = p.clone();
    bad.predictor_names[1] = name.into();
    assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  }
  let mut bad = p.clone();
  for row in &mut bad.design_matrix {
    row[0] = 1.0;
  }
  assert!(fit_gompertz(&bad, "event", &d, &options).is_err());
  let mut bad = p.clone();
  bad.response.truncate(42);
  bad.design_matrix.truncate(42);
  bad.sample = None;
  assert!(fit_gompertz(&bad, "event", &d[..42], &options).is_err());
  for (n, xvars) in [(10001, 0), (70, 63), (5000, 62)] {
    let p = EstimationProblem::new(
      "time",
      (0..xvars).map(|j| format!("x{j}")).collect(),
      vec![1.0; n],
      vec![vec![0.0; xvars]; n],
    )
    .unwrap();
    let error = fit_gompertz(&p, "event", &vec![1.0; n], &options)
      .unwrap_err()
      .to_string();
    assert!(error.contains("N<=10000"), "{error}");
  }
  let concentrated = EstimationProblem::new("time", vec![], vec![1.0; 5], vec![vec![]; 5]).unwrap();
  assert!(
    fit_gompertz(&concentrated, "event", &[1.0; 5], &options).is_err(),
    "unbounded equal-time failures must not publish an approximate fit"
  );
}

#[test]
fn gompertz_current_points_respect_time_zero_overflow_and_log_tail_cancellation() {
  let mut r = fit_gompertz(
    &baseline_problem(),
    "event",
    &[1.0, 1.0, 0.0, 0.0],
    &Default::default(),
  )
  .unwrap();
  r.shape.value = 0.0;
  let eta = r.coefficients[0].value;
  for t in [None, Some(-1.0), Some(f64::NAN), Some(f64::INFINITY)] {
    assert!(gompertz_predict_row(&r, &[], t, GompertzPredictionKind::Hazard).is_err());
    assert_eq!(
      gompertz_predict_row(&r, &[], t, GompertzPredictionKind::Xb).unwrap(),
      eta
    );
  }
  assert!(gompertz_predict_row(&r, &[1.0], Some(1.0), GompertzPredictionKind::Survival).is_err());
  r.coefficients[0].value = 1000.0;
  assert!(
    gompertz_predict_row(&r, &[], Some(1.0), GompertzPredictionKind::CumulativeHazard).is_err()
  );
  assert!(gompertz_predict_row(&r, &[], Some(1.0), GompertzPredictionKind::Hazard).is_err());
  assert_eq!(
    gompertz_predict_row(&r, &[], Some(1.0), GompertzPredictionKind::Survival).unwrap(),
    0.0
  );
  assert_eq!(
    gompertz_predict_row(&r, &[], Some(0.0), GompertzPredictionKind::CumulativeHazard).unwrap(),
    0.0
  );
  assert_eq!(
    gompertz_predict_row(&r, &[], Some(0.0), GompertzPredictionKind::Survival).unwrap(),
    1.0
  );
  r.coefficients[0].value = -1e308;
  r.shape.value = 1e308;
  assert_eq!(
    gompertz_predict_row(&r, &[], Some(1.0), GompertzPredictionKind::Hazard).unwrap(),
    1.0
  );
  let h =
    gompertz_predict_row(&r, &[], Some(1.0), GompertzPredictionKind::CumulativeHazard).unwrap();
  assert!(
    (h / 1e-308 - 1.0).abs() < 1e-12,
    "must retain -log(gamma) after eta+gamma*t cancellation"
  );
  assert!(gompertz_predict_row(&r, &[], Some(10.0), GompertzPredictionKind::Survival).is_err());
}
