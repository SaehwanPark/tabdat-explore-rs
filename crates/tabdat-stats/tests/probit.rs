// Reference literals below are printed by R at digits=17; the extra digit
// does not change the parsed f64 value, so clippy's excessive-precision lint
// is silenced for these opaque fixture values.
#![allow(clippy::excessive_precision)]

use tabdat_stats::{
  BinaryModel, EstimationProblem, EstimationSample, LogitMarginalEffects, ProbitOptions,
  StatsError, binary_average_marginal_effects, fit_binary_logit, fit_binary_probit,
  fit_binary_probit_with_options, logit_average_marginal_effects,
};

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() < 1e-10,
    "{actual:.17} != {expected:.17}"
  );
}

fn relative(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= expected.abs() * 1e-10 + f64::MIN_POSITIVE,
    "{actual:.17e} != {expected:.17e}"
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
fn four_independent_r_references_cover_design_observed_information_and_delta_inference() {
  // R 4.6.1 references: crates/tabdat-stats/tests/fixtures/probit_reference.R
  // regenerates all values (glm IRLS first solution, then a Newton polish on
  // the analytic observed information until the gradient norm is < 1e-13,
  // because glm's deviance-change stop leaves probit coefficients ~1e-9 from
  // the MLE on these designs). Covariance = inverse OIM at that fixed point;
  // AME = probit delta method averaged over all retained rows, with the OIM
  // delta-method standard errors. Absolute 1e-10 for moderate outputs;
  // relative 1e-10 for small tail probabilities.
  let references = [
    Reference {
      intercept: true,
      two: true,
      coefficients: vec![
        0.13364045759262577,
        0.37200554216777859,
        -0.35371618791237436,
      ],
      coefficient_p: vec![
        0.52127297596846967,
        0.10945834969820845,
        0.22330122196479868,
      ],
      covariance: vec![
        vec![
          0.043414933864685645,
          0.00056155307533640216,
          -0.00019614358397441824,
        ],
        vec![
          0.00056155307533640227,
          0.054015149635511933,
          0.018487237266294795,
        ],
        vec![
          -0.00019614358397441832,
          0.018487237266294795,
          0.084364600567286965,
        ],
      ],
      probabilities: [
        0.48332013150794201,
        0.39549481488702537,
        0.66888114268343024,
      ],
      ll: -24.51228107498369,
      lr: 6.0265429471197223,
      lr_p: 0.049130685822332425,
      ame: vec![0.12987211997972109, -0.12348706131535794],
      ame_se: vec![0.072857744094171742, 0.095838874529783846],
      ame_p: vec![0.074660595325425905, 0.19757679115102303],
    },
    Reference {
      intercept: true,
      two: false,
      coefficients: vec![0.13907349296591914, 0.46083423873144186],
      coefficient_p: vec![0.49936611198985931, 0.038928422029420373],
      covariance: vec![
        vec![0.0423890940914292, 0.0014591946203885364],
        vec![0.0014591946203885359, 0.049805059833710502],
      ],
      probabilities: [
        0.51870618409676517,
        0.40920367445066985,
        0.55530396290225026,
      ],
      ll: -25.256133886819303,
      lr: 4.5388373234484973,
      lr_p: 0.033134090103371698,
      ame: vec![0.16608333656305493],
      ame_se: vec![0.066131861571066902],
      ame_p: vec![0.012025448740446724],
    },
    Reference {
      intercept: false,
      two: true,
      coefficients: vec![0.37191612277112351, -0.35469079185176078],
      coefficient_p: vec![0.10958647738419286, 0.2169671597339568],
      covariance: vec![
        vec![0.054028190271852239, 0.017613191221968964],
        vec![0.017613191221968961, 0.082532336100792178],
      ],
      probabilities: [
        0.43025560545243918,
        0.34499334645845103,
        0.61944393149124544,
      ],
      ll: -24.718496052266925,
      lr: 6.0147823402617746,
      lr_p: 0.049420440249691562,
      ame: vec![0.13078096444590281, -0.12472383152639122],
      ame_se: vec![0.073404442319312119, 0.095076981876603447],
      ame_p: vec![0.074806459161909256, 0.18958106976391298],
    },
    Reference {
      intercept: false,
      two: false,
      coefficients: vec![0.45819975441693372],
      coefficient_p: vec![0.040186253915323433],
      covariance: vec![vec![0.0498686080475415]],
      probabilities: [0.46349205438295377, 0.35697369810892454, 0.5],
      ll: -25.484996463774177,
      lr: 4.4817815172472706,
      lr_p: 0.03425799335150144,
      ame: vec![0.1664791310528749],
      ame_se: vec![0.066571405899697164],
      ame_p: vec![0.012392701350499158],
    },
  ];
  for r in references {
    let p = problem(r.intercept, r.two);
    let fit = fit_binary_probit(&p).unwrap();
    assert_eq!(fit.model, BinaryModel::Probit);
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
    let ame = binary_average_marginal_effects(&p, &fit).unwrap();
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

#[test]
fn predictor_units_preserve_likelihood_probabilities_covariance_and_ame() {
  let original = problem(true, true);
  let base = fit_binary_probit(&original).unwrap();
  let base_ame = binary_average_marginal_effects(&original, &base).unwrap();
  let mut scaled = original.clone();
  let scales = [-1e-9, 1e9];
  for row in &mut scaled.design_matrix {
    for j in 0..2 {
      row[j] *= scales[j];
    }
  }
  let fit = fit_binary_probit(&scaled).unwrap();
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
  let ame = binary_average_marginal_effects(&scaled, &fit).unwrap();
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
fn complete_and_quasi_separation_never_publish_a_bounded_gain_as_a_fit() {
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
      let error = fit_binary_probit(&p).unwrap_err();
      assert!(matches!(error, StatsError::EstimationFailed(_)), "{error}");
      assert!(error.to_string().contains("separation"), "{error}");
    }
  }
}

#[test]
fn large_identified_coefficient_cannot_hide_a_separating_direction() {
  let base_x = [-2., -2., -1., -1., 0., 0., 1., 1., 2., 2., -1e11, 1e11];
  let mut y = vec![0., 0., 0., 1., 0., 1., 0., 1., 1., 1., 0., 1.];
  let mut x: Vec<_> = base_x.into_iter().map(|v| vec![v, 0.0]).collect();
  x.extend([vec![0.0, -1.0], vec![0.0, 1.0]]);
  y.extend([0.0, 1.0]);
  for rotate in [false, true] {
    let mut design = x.clone();
    if rotate {
      for row in &mut design {
        let finite = row[0] / 1e11;
        let separating = row[1];
        row[0] = finite + separating;
        row[1] = finite - separating;
      }
    }
    let p = EstimationProblem::new("y", vec!["x".into(), "z".into()], y.clone(), design).unwrap();
    let result = fit_binary_probit(&p);
    assert!(
      result.is_err(),
      "rotate={rotate}: quasi-separated model falsely converged: {result:?}"
    );
  }
}

#[test]
fn iteration_exhaustion_and_invalid_specifications_are_explicit() {
  let p = problem(true, true);
  let error = fit_binary_probit_with_options(&p, &ProbitOptions { max_iterations: 1 }).unwrap_err();
  assert!(error.to_string().contains("did not converge"));
  assert!(fit_binary_probit_with_options(&p, &ProbitOptions { max_iterations: 0 }).is_err());
  for invalid_value in [-1.0, 0.2, 2.0] {
    let mut invalid = p.clone();
    invalid.response[0] = invalid_value;
    assert!(
      fit_binary_probit(&invalid)
        .unwrap_err()
        .to_string()
        .contains("exactly 0 or 1")
    );
  }
  for outcome in [0.0, 1.0] {
    let mut invalid = p.clone();
    invalid.response.fill(outcome);
    assert!(
      fit_binary_probit(&invalid)
        .unwrap_err()
        .to_string()
        .contains("both outcome classes")
    );
  }
  let mut weighted = p.clone();
  weighted.weights = Some(vec![1.0; 40]);
  assert!(
    fit_binary_probit(&weighted)
      .unwrap_err()
      .to_string()
      .contains("weights")
  );
  let mut duplicate_names = p.clone();
  duplicate_names.predictor_names[1] = "x1".into();
  assert!(fit_binary_probit(&duplicate_names).is_err());
  let mut collinear = p.clone();
  for row in &mut collinear.design_matrix {
    row[1] = row[0];
  }
  assert!(matches!(
    fit_binary_probit(&collinear),
    Err(StatsError::SingularMatrix(_))
  ));
  let mut nonfinite = p.clone();
  nonfinite.design_matrix[0][0] = f64::INFINITY;
  assert!(matches!(
    fit_binary_probit(&nonfinite),
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
    fit_binary_probit(&saturated),
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
  let fit = fit_binary_probit(&p).unwrap();
  // The intercept-only probit MLE inverts the normal link; the observed
  // information of the non-canonical link is not n*p(1-p).
  close(fit.coefficients[0].value, -0.52440051270804078);
  close(fit.covariance[0][0], 0.17371141743550259);
  close(fit.likelihood_ratio, 0.0);
  assert_eq!(fit.likelihood_ratio_p_value, None);
  assert!(
    binary_average_marginal_effects(&p, &fit)
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
  let fit = fit_binary_probit(&p).unwrap();
  assert!(fit.coefficients[1].value > 0.5 && fit.coefficients[1].value < 2.0);
  close(fit.coefficients[0].value, 0.0);
  assert_eq!(fit.fitted_probabilities[10], 0.0);
  assert_eq!(fit.fitted_probabilities[11], 1.0);
  assert!(fit.covariance.iter().flatten().all(|v| v.is_finite()));
  // Observed-information SEs on the ten identified rows (the saturated pair
  // drops out exactly), and delta-method AME averaged over all twelve.
  close(fit.coefficients[1].value, 0.63342757035981667);
  close(fit.covariance[0][0], 0.2040696922088259);
  close(fit.covariance[1][1], 0.14019905759524603);
  close(fit.covariance[1][0], 0.0);
  let ame = binary_average_marginal_effects(&p, &fit).unwrap();
  close(ame.effects[0].value, 0.14879472907404659);
  close(ame.effects[0].standard_error, 0.035790071052071652);
  relative(ame.effects[0].p_value, 3.218472974562866e-05);
}

#[test]
fn mismatched_family_or_corrupt_owned_fit_is_not_used_for_marginal_effects() {
  let p = problem(true, true);
  let fit = fit_binary_probit(&p).unwrap();
  let mut other = p.clone();
  other.predictor_names.swap(0, 1);
  assert!(binary_average_marginal_effects(&other, &fit).is_err());
  let mut corrupt = fit.clone();
  corrupt.covariance.pop();
  assert!(binary_average_marginal_effects(&p, &corrupt).is_err());
  let mut corrupt = fit.clone();
  corrupt.fitted_weights[0] = f64::NAN;
  assert!(binary_average_marginal_effects(&p, &corrupt).is_err());
  // A fit's sample identity must stay the exact sample the caller computes
  // against (mutating either side invalidates the paired computation).
  let mut tracked = p.clone();
  tracked.sample = Some(EstimationSample::new((0..40).collect(), 40, 0, None, None).unwrap());
  let tracked_fit = fit_binary_probit(&tracked).unwrap();
  assert!(binary_average_marginal_effects(&tracked, &tracked_fit).is_ok());
  let mut corrupt = tracked_fit.clone();
  corrupt.sample.as_mut().unwrap().retained_indices[0] += 1;
  assert!(binary_average_marginal_effects(&tracked, &corrupt).is_err());
  // The prior-name alias stays name-checked against the fitted family.
  assert!(
    logit_average_marginal_effects(&p, &fit)
      .unwrap_err()
      .to_string()
      .contains("requires a logit result")
  );
  let logit_p = problem(true, true);
  let logit_fit = fit_binary_logit(&logit_p).unwrap();
  let LogitMarginalEffects {
    observation_count, ..
  } = logit_average_marginal_effects(&logit_p, &logit_fit).unwrap();
  assert_eq!(observation_count, 40);
}
