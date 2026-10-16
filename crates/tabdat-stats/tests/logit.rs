use tabdat_stats::{
  BinaryLogitResult, EstimationProblem, LogitOptions, StatsError, fit_binary_logit,
  fit_binary_logit_with_options, logit_average_marginal_effects,
};

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() < 1e-10,
    "{actual:.17} != {expected:.17}"
  );
}

fn problem(intercept: bool, two: bool) -> EstimationProblem {
  let mut x = Vec::new();
  let mut y = Vec::new();
  for i in 1..=40 {
    let x1 = ((i * 7) % 17) as f64 / 5.0 - 1.6;
    let x2 = ((i * 11) % 19) as f64 / 7.0 - 9.0 / 7.0;
    y.push(f64::from(
      (((i * 13) % 37) as f64) < 18.0 + 4.0 * x1 - 3.0 * x2,
    ));
    x.push(if two { vec![x1, x2] } else { vec![x1] });
  }
  let names = if two {
    vec!["x1".into(), "x2".into()]
  } else {
    vec!["x1".into()]
  };
  EstimationProblem::new("y", names, y, x)
    .unwrap()
    .with_intercept(intercept)
}

struct Reference {
  intercept: bool,
  two: bool,
  coefficients: Vec<f64>,
  coefficient_p: Vec<f64>,
  covariance: Vec<Vec<f64>>,
  probabilities: [f64; 3],
  ll: f64,
  lr: f64,
  lr_p: f64,
  ame: Vec<f64>,
  ame_se: Vec<f64>,
  ame_p: Vec<f64>,
}

#[test]
fn four_independent_r_glm_references_cover_design_and_normal_delta_inference() {
  // R 4.6.1: fixtures/logit_reference.R. Absolute 1e-10 for moderate outputs;
  // relative 1e-10 for probabilities of tests (including small tails elsewhere).
  let references = [
    Reference {
      intercept: true,
      two: true,
      coefficients: vec![0.23169387485895937, 0.6009973792306854, -0.5808443835284844],
      coefficient_p: vec![0.5010183376899813, 0.11031090942408736, 0.22126266227765387],
      covariance: vec![
        vec![
          0.11856134409181962,
          0.00495088100065894,
          -0.00837068490742984,
        ],
        vec![0.00495088100065894, 0.14165946752278716, 0.0386698118821906],
        vec![
          -0.00837068490742984,
          0.0386698118821906,
          0.22549692389923484,
        ],
      ],
      probabilities: [0.4863880794799564, 0.39769955474325164, 0.6747088175781457],
      ll: -24.51495924658521,
      lr: 6.021186603916682,
      lr_p: 0.049262442584721435,
      ame: vec![0.12738548021377377, -0.12311391577108491],
      ame_se: vec![0.06969557366837566, 0.0937844499659613],
      ame_p: vec![0.06758838279473625, 0.1892730291883931],
    },
    Reference {
      intercept: true,
      two: false,
      coefficients: vec![0.23689966240725233, 0.7420884742393362],
      coefficient_p: vec![0.48350100353355285, 0.04396175005735533],
      covariance: vec![
        vec![0.1143104652114983, 0.011012703693837868],
        vec![0.011012703693837868, 0.13570507486831765],
      ],
      probabilities: [0.5221060712941354, 0.4117414115707563, 0.5589494789438502],
      ll: -25.26245043817579,
      lr: 4.526204220735522,
      lr_p: 0.03337957623211646,
      ame: vec![0.16357556876327994],
      ame_se: vec![0.064062078827977],
      ame_p: vec![0.010667950444905238],
    },
    Reference {
      intercept: false,
      two: true,
      coefficients: vec![0.5984493742029906, -0.5717345154485121],
      coefficient_p: vec![0.11254821710022118, 0.21746481279412708],
      covariance: vec![
        vec![0.14222860073560378, 0.03667926956682812],
        vec![0.03667926956682812, 0.2149082871031096],
      ],
      probabilities: [0.42970800210048843, 0.34476922581298214, 0.6201201323380681],
      ll: -24.743324438571683,
      lr: 5.965125567652201,
      lr_p: 0.050662829469522844,
      ame: vec![0.12838058363410956, -0.12264965749993094],
      ame_se: vec![0.07091804919915982, 0.0923581664187681],
      ame_p: vec![0.07025443682068715, 0.18418521612927116],
    },
    Reference {
      intercept: false,
      two: false,
      coefficients: vec![0.728903097698189],
      coefficient_p: vec![0.04580270400383768],
      covariance: vec![vec![0.13319669378730753]],
      probabilities: [0.4636192525758821, 0.35821442840275947, 0.5],
      ll: -25.51034356585805,
      lr: 4.431087313079466,
      lr_p: 0.035290019681896954,
      ame: vec![0.1627034993552953],
      ame_se: vec![0.0644527332560911],
      ame_p: vec![0.011590098897085514],
    },
  ];
  for r in references {
    let p = problem(r.intercept, r.two);
    let fit = fit_binary_logit(&p).unwrap();
    assert_eq!(fit.observation_count, 40);
    assert_eq!(fit.event_count, 22);
    assert_eq!(fit.likelihood_ratio_df, if r.two { 2 } else { 1 });
    close(fit.log_likelihood, r.ll);
    close(
      fit.null_log_likelihood,
      if r.intercept {
        -27.52555254854355
      } else {
        -27.725887222397784
      },
    );
    close(fit.likelihood_ratio, r.lr);
    relative(fit.likelihood_ratio_p_value.unwrap(), r.lr_p);
    close(
      fit.mcfadden_r_squared,
      1.0 - fit.log_likelihood / fit.null_log_likelihood,
    );
    for (i, &b) in r.coefficients.iter().enumerate() {
      let c = &fit.coefficients[i];
      close(c.value, b);
      close(c.standard_error, r.covariance[i][i].sqrt());
      close(c.statistic, b / r.covariance[i][i].sqrt());
      relative(c.p_value, r.coefficient_p[i]);
      close(c.confidence_lower, b - 1.959963984540054 * c.standard_error);
      close(c.confidence_upper, b + 1.959963984540054 * c.standard_error);
      for (j, &v) in r.covariance[i].iter().enumerate() {
        close(fit.covariance[i][j], v);
      }
    }
    for (index, expected) in [0, 19, 39].into_iter().zip(r.probabilities) {
      close(fit.fitted_probabilities[index], expected);
    }
    assert!(fit.max_abs_scaled_score <= 1e-10);
    let ame = logit_average_marginal_effects(&p, &fit).unwrap();
    assert_eq!(ame.effects.len(), r.ame.len());
    for (j, effect) in ame.effects.iter().enumerate() {
      assert_eq!(effect.name, p.predictor_names[j]);
      close(effect.value, r.ame[j]);
      close(effect.standard_error, r.ame_se[j]);
      close(effect.statistic, r.ame[j] / r.ame_se[j]);
      relative(effect.p_value, r.ame_p[j]);
      close(
        effect.confidence_lower,
        r.ame[j] - 1.959963984540054 * r.ame_se[j],
      );
      close(
        effect.confidence_upper,
        r.ame[j] + 1.959963984540054 * r.ame_se[j],
      );
    }
  }
}

fn relative(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= expected.abs() * 1e-10 + f64::MIN_POSITIVE,
    "{actual:.17e} != {expected:.17e}"
  );
}

#[test]
fn predictor_units_preserve_likelihood_probabilities_covariance_and_ame() {
  let original = problem(true, true);
  let base = fit_binary_logit(&original).unwrap();
  let base_ame = logit_average_marginal_effects(&original, &base).unwrap();
  let mut scaled = original.clone();
  let scales = [-1e-9, 1e9];
  for row in &mut scaled.design_matrix {
    for j in 0..2 {
      row[j] *= scales[j];
    }
  }
  let fit = fit_binary_logit(&scaled).unwrap();
  close(fit.log_likelihood, base.log_likelihood);
  for (a, b) in fit
    .fitted_probabilities
    .iter()
    .zip(&base.fitted_probabilities)
  {
    close(*a, *b);
  }
  let coefficient_scales = [1.0, scales[0], scales[1]];
  for i in 0..3 {
    close(
      fit.coefficients[i].value * coefficient_scales[i],
      base.coefficients[i].value,
    );
    for j in 0..3 {
      close(
        fit.covariance[i][j] * coefficient_scales[i] * coefficient_scales[j],
        base.covariance[i][j],
      );
    }
  }
  let ame = logit_average_marginal_effects(&scaled, &fit).unwrap();
  for (j, scale) in scales.iter().enumerate() {
    close(ame.effects[j].value * scale, base_ame.effects[j].value);
    close(
      ame.effects[j].standard_error * scale.abs(),
      base_ame.effects[j].standard_error,
    );
    relative(ame.effects[j].p_value, base_ame.effects[j].p_value);
  }
}

#[test]
fn complete_and_quasi_separation_never_publish_a_flat_likelihood_as_a_fit() {
  for (x, y) in [
    (
      vec![-3., -2., -1., 1., 2., 3.],
      vec![0., 0., 0., 1., 1., 1.],
    ),
    (vec![-2., -1., 0., 0., 1., 2.], vec![0., 0., 0., 1., 1., 1.]),
  ] {
    for intercept in [true, false] {
      let p = EstimationProblem::new(
        "y",
        vec!["x".into()],
        y.clone(),
        x.iter().map(|&x| vec![x]).collect(),
      )
      .unwrap()
      .with_intercept(intercept);
      let error = fit_binary_logit(&p).unwrap_err();
      assert!(matches!(error, StatsError::EstimationFailed(_)), "{error}");
      assert!(error.to_string().contains("separation"), "{error}");
    }
  }
}

#[test]
fn iteration_exhaustion_and_invalid_specifications_are_explicit() {
  let p = problem(true, true);
  let error = fit_binary_logit_with_options(&p, &LogitOptions { max_iterations: 1 }).unwrap_err();
  assert!(error.to_string().contains("did not converge"));
  assert!(fit_binary_logit_with_options(&p, &LogitOptions { max_iterations: 0 }).is_err());
  for invalid_value in [-1.0, 0.2, 2.0] {
    let mut invalid = p.clone();
    invalid.response[0] = invalid_value;
    assert!(
      fit_binary_logit(&invalid)
        .unwrap_err()
        .to_string()
        .contains("exactly 0 or 1")
    );
  }
  for outcome in [0.0, 1.0] {
    let mut invalid = p.clone();
    invalid.response.fill(outcome);
    assert!(
      fit_binary_logit(&invalid)
        .unwrap_err()
        .to_string()
        .contains("both outcome classes")
    );
  }
  let mut weighted = p.clone();
  weighted.weights = Some(vec![1.0; 40]);
  assert!(
    fit_binary_logit(&weighted)
      .unwrap_err()
      .to_string()
      .contains("weights")
  );
  let mut duplicate_names = p.clone();
  duplicate_names.predictor_names[1] = "x1".into();
  assert!(fit_binary_logit(&duplicate_names).is_err());
  let mut collinear = p.clone();
  for row in &mut collinear.design_matrix {
    row[1] = row[0];
  }
  assert!(matches!(
    fit_binary_logit(&collinear),
    Err(StatsError::SingularMatrix(_))
  ));
  let mut nonfinite = p.clone();
  nonfinite.design_matrix[0][0] = f64::INFINITY;
  assert!(matches!(
    fit_binary_logit(&nonfinite),
    Err(StatsError::NonFiniteValue(_))
  ));
  let saturated = EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![0., 1.],
    vec![vec![0.], vec![1.]],
  )
  .unwrap();
  assert!(matches!(
    fit_binary_logit(&saturated),
    Err(StatsError::InsufficientObservations(_))
  ));
}

#[test]
fn intercept_only_null_model_has_no_slope_test_or_marginal_effect() {
  let p = EstimationProblem::new(
    "y",
    vec![],
    vec![0., 0., 0., 0., 0., 0., 0., 1., 1., 1.],
    vec![vec![]; 10],
  )
  .unwrap();
  let fit = fit_binary_logit(&p).unwrap();
  close(fit.coefficients[0].value, (3.0_f64 / 7.0).ln());
  close(fit.covariance[0][0], 1.0 / 2.1);
  close(fit.likelihood_ratio, 0.0);
  assert_eq!(fit.likelihood_ratio_p_value, None);
  assert!(
    logit_average_marginal_effects(&p, &fit)
      .unwrap()
      .effects
      .is_empty()
  );
}

#[test]
fn saturated_outlier_probabilities_do_not_imply_separation_in_an_identified_model() {
  let x = vec![-2., -2., -1., -1., 0., 0., 1., 1., 2., 2., -1000., 1000.];
  let y = vec![0., 0., 0., 1., 0., 1., 0., 1., 1., 1., 0., 1.];
  let p = EstimationProblem::new(
    "y",
    vec!["x".into()],
    y,
    x.into_iter().map(|v| vec![v]).collect(),
  )
  .unwrap();
  let fit = fit_binary_logit(&p).unwrap();
  assert!(fit.coefficients[1].value > 0.5 && fit.coefficients[1].value < 2.0);
  close(fit.coefficients[0].value, 0.0);
  assert_eq!(fit.fitted_probabilities[10], 0.0);
  assert_eq!(fit.fitted_probabilities[11], 1.0);
  assert!(fit.covariance.iter().flatten().all(|v| v.is_finite()));
  assert!(logit_average_marginal_effects(&p, &fit).is_ok());
}

#[test]
fn mismatched_or_corrupt_owned_fit_is_not_used_for_marginal_effects() {
  let p = problem(true, true);
  let fit = fit_binary_logit(&p).unwrap();
  let mut other = p.clone();
  other.predictor_names.swap(0, 1);
  assert!(logit_average_marginal_effects(&other, &fit).is_err());
  let mut corrupt: BinaryLogitResult = fit.clone();
  corrupt.covariance.pop();
  assert!(logit_average_marginal_effects(&p, &corrupt).is_err());
  let mut corrupt = fit;
  corrupt.fitted_weights[0] = f64::NAN;
  assert!(logit_average_marginal_effects(&p, &corrupt).is_err());
}
