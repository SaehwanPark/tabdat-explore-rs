use tabdat_stats::{
  BinaryModel, EstimationProblem, EstimationSample, binary_average_marginal_effects,
  fit_binary_logit, fit_binary_probit, fit_binary_robust,
};

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
  EstimationProblem::new(
    "y",
    if two {
      vec!["x1".into(), "x2".into()]
    } else {
      vec!["x1".into()]
    },
    y,
    x,
  )
  .unwrap()
  .with_intercept(intercept)
  .with_sample(EstimationSample::new((0..40).map(|i| 2 * i).collect(), 80, 40, None, None).unwrap())
}

#[test]
fn independent_r_references_cover_robust_covariance_and_full_normal_delta_inference() {
  // R 4.6.1, fixtures/binary_robust_reference.R. Same sample/design as the
  // classical reference; N/(N-1), not HC1's N/(N-K). 1e-10 absolute for
  // moderate outputs and 1e-10 relative for two-sided test probabilities.
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    for intercept in [true, false] {
      for two in [true, false] {
        let p = problem(intercept, two);
        let fit = fit_binary_robust(&p, model).unwrap();
        let classical = match model {
          BinaryModel::Logit => fit_binary_logit(&p),
          BinaryModel::Probit => fit_binary_probit(&p),
        }
        .unwrap();
        assert!(fit.robust);
        assert!(!classical.robust);
        assert_eq!(fit.sample, classical.sample);
        assert_eq!(fit.fitted_probabilities, classical.fitted_probabilities);
        assert_eq!(fit.log_likelihood, classical.log_likelihood);
        assert_eq!(fit.likelihood_ratio, classical.likelihood_ratio);
        assert!(fit.likelihood_ratio_p_value.is_none());
        assert!(classical.likelihood_ratio_p_value.is_some());
        let ame = binary_average_marginal_effects(&p, &fit).unwrap();
        assert!(ame.robust);
        let classical_ame = binary_average_marginal_effects(&p, &classical).unwrap();
        for (a, b) in ame.effects.iter().zip(classical_ame.effects) {
          assert_eq!(a.value, b.value);
        }
        let family = if model == BinaryModel::Logit {
          "logit"
        } else {
          "probit"
        };
        let mut checked = 0;
        for row in include_str!("fixtures/binary_robust_reference.csv")
          .lines()
          .skip(1)
        {
          let fields: Vec<_> = row.split(',').collect();
          if fields[0] != family
            || (fields[1] == "TRUE") != intercept
            || (fields[2] == "TRUE") != two
          {
            continue;
          }
          let i: usize = fields[4].parse().unwrap();
          let j: usize = fields[5].parse().unwrap();
          let expected: f64 = fields[6].parse().unwrap();
          let c = &fit.coefficients[i];
          let actual = match fields[3] {
            "coefficient" => c.value,
            "se" => c.standard_error,
            "z" => c.statistic,
            "p" => c.p_value,
            "lower" => c.confidence_lower,
            "upper" => c.confidence_upper,
            "covariance" => fit.covariance[i][j],
            q => {
              let a = &ame.effects[i];
              match q {
                "ame" => a.value,
                "ame_se" => a.standard_error,
                "ame_z" => a.statistic,
                "ame_p" => a.p_value,
                "ame_lower" => a.confidence_lower,
                "ame_upper" => a.confidence_upper,
                _ => panic!("unknown reference quantity: {q}"),
              }
            }
          };
          let tolerance = if fields[3] == "p" || fields[3] == "ame_p" {
            expected.abs() * 1e-10 + f64::MIN_POSITIVE
          } else {
            1e-10
          };
          assert!(
            (actual - expected).abs() < tolerance,
            "{row}: actual {actual:.17}"
          );
          checked += 1;
        }
        let k = p.parameter_count();
        assert_eq!(checked, 6 * k + k * k + 6 * p.predictor_names.len());
      }
    }
  }
}

#[test]
fn robust_fits_keep_identification_domain_and_cluster_weight_guards() {
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    let mut p = problem(true, false);
    p.response[0] = 2.0;
    assert!(fit_binary_robust(&p, model).is_err());
    p.response.fill(1.0);
    assert!(fit_binary_robust(&p, model).is_err());
    let mut p = problem(true, false);
    for (y, x) in p.response.iter_mut().zip(&p.design_matrix) {
      *y = f64::from(x[0] > 0.0);
    }
    assert!(fit_binary_robust(&p, model).is_err());
    let mut p = problem(true, false);
    p.sample.as_mut().unwrap().cluster_groups = Some(vec!["one".into(); 40]);
    assert!(fit_binary_robust(&p, model).is_err());
    let mut p = problem(true, false);
    p.weights = Some(vec![1.0; 40]);
    assert!(fit_binary_robust(&p, model).is_err());
  }
}
