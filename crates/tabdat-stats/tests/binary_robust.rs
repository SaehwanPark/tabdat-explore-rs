use tabdat_stats::{
  BinaryModel, EstimationProblem, EstimationSample, binary_average_marginal_effects,
  fit_binary_clustered, fit_binary_logit, fit_binary_probit, fit_binary_robust,
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
  for (clustered, references) in [
    (false, include_str!("fixtures/binary_robust_reference.csv")),
    (true, include_str!("fixtures/binary_cluster_reference.csv")),
  ] {
    for model in [BinaryModel::Logit, BinaryModel::Probit] {
      for intercept in [true, false] {
        for two in [true, false] {
          let mut p = problem(intercept, two);
          let classical = match model {
            BinaryModel::Logit => fit_binary_logit(&p),
            BinaryModel::Probit => fit_binary_probit(&p),
          }
          .unwrap();
          let classical_problem = p.clone();
          if clustered {
            p.sample.as_mut().unwrap().cluster_groups =
              Some((1..=40).map(|i| ((i * 7) % 9).to_string()).collect());
          }
          let fit = if clustered {
            fit_binary_clustered(&p, model, "group")
          } else {
            fit_binary_robust(&p, model)
          }
          .unwrap();
          assert_eq!(
            fit
              .cluster
              .as_ref()
              .map(|c| (c.variable.as_str(), c.group_count)),
            clustered.then_some(("group", 9))
          );
          assert!(fit.robust);
          assert!(!classical.robust);
          assert_eq!(fit.sample, p.sample);
          assert_eq!(
            fit.sample.as_ref().unwrap().retained_indices,
            classical.sample.as_ref().unwrap().retained_indices
          );
          assert_eq!(fit.fitted_probabilities, classical.fitted_probabilities);
          assert_eq!(fit.log_likelihood, classical.log_likelihood);
          assert_eq!(fit.likelihood_ratio, classical.likelihood_ratio);
          assert!(fit.likelihood_ratio_p_value.is_none());
          assert!(classical.likelihood_ratio_p_value.is_some());
          let ame = binary_average_marginal_effects(&p, &fit).unwrap();
          assert!(ame.robust);
          assert_eq!(ame.cluster, fit.cluster);
          let classical_ame =
            binary_average_marginal_effects(&classical_problem, &classical).unwrap();
          for (a, b) in ame.effects.iter().zip(classical_ame.effects) {
            assert_eq!(a.value, b.value);
          }
          let family = if model == BinaryModel::Logit {
            "logit"
          } else {
            "probit"
          };
          let mut checked = 0;
          for row in references.lines().skip(1) {
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
}

#[test]
fn robust_intercept_only_and_predictor_units_have_explicit_covariance_semantics() {
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    let p = EstimationProblem::new(
      "y",
      vec![],
      vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
      vec![vec![]; 10],
    )
    .unwrap();
    let fit = fit_binary_robust(&p, model).unwrap();
    let expected_variance = match model {
      BinaryModel::Logit => 1.0 / (9.0 * 0.3 * 0.7),
      BinaryModel::Probit => {
        let density = tabdat_stats::normal_pdf(tabdat_stats::normal_quantile(0.3));
        0.3 * 0.7 / (9.0 * density * density)
      }
    };
    assert!((fit.covariance[0][0] - expected_variance).abs() < 1e-10);
    assert_eq!(fit.likelihood_ratio_df, 0);
    assert!(fit.likelihood_ratio_p_value.is_none());
    let ame = binary_average_marginal_effects(&p, &fit).unwrap();
    assert!(ame.robust && ame.effects.is_empty());

    let p = problem(true, true);
    let fit = fit_binary_robust(&p, model).unwrap();
    let ame = binary_average_marginal_effects(&p, &fit).unwrap();
    let mut scaled = p.clone();
    for row in &mut scaled.design_matrix {
      row[0] *= 1e5;
      row[1] *= 0.03;
    }
    let other = fit_binary_robust(&scaled, model).unwrap();
    let other_ame = binary_average_marginal_effects(&scaled, &other).unwrap();
    let factors = [1.0, 1e5, 0.03];
    for (i, factor) in factors.iter().enumerate() {
      assert!((fit.coefficients[i].value - other.coefficients[i].value * factor).abs() < 1e-10);
      for (j, factor_j) in factors.iter().enumerate() {
        assert!((fit.covariance[i][j] - other.covariance[i][j] * factor * factor_j).abs() < 1e-10);
      }
    }
    for i in 0..2 {
      assert!((ame.effects[i].value - other_ame.effects[i].value * factors[i + 1]).abs() < 1e-10);
      assert!(
        (ame.effects[i].standard_error - other_ame.effects[i].standard_error * factors[i + 1])
          .abs()
          < 1e-10
      );
    }
  }
}

#[test]
fn cluster_group_guards_singletons_and_relabeling_have_explicit_semantics() {
  for model in [BinaryModel::Logit, BinaryModel::Probit] {
    let mut p = problem(true, true);
    assert!(fit_binary_clustered(&p, model, "group").is_err());
    p.sample.as_mut().unwrap().cluster_groups = Some(vec!["one".into(); 40]);
    assert!(fit_binary_clustered(&p, model, "group").is_err());
    p.sample.as_mut().unwrap().cluster_groups = Some(vec!["one".into(); 39]);
    assert!(fit_binary_clustered(&p, model, "group").is_err());
    p.sample.as_mut().unwrap().cluster_groups = None;
    let individual = fit_binary_robust(&p, model).unwrap();
    p.sample.as_mut().unwrap().cluster_groups = Some((0..40).map(|i| i.to_string()).collect());
    let singleton = fit_binary_clustered(&p, model, "group").unwrap();
    for (a, b) in individual
      .covariance
      .iter()
      .flatten()
      .zip(singleton.covariance.iter().flatten())
    {
      assert!((a - b).abs() < 1e-12);
    }
    assert!(fit_binary_clustered(&p, model, "").is_err());
    p.sample.as_mut().unwrap().cluster_groups =
      Some((1..=40).map(|i| ((i * 7) % 9).to_string()).collect());
    let grouped = fit_binary_clustered(&p, model, "group").unwrap();
    for name in p.sample.as_mut().unwrap().cluster_groups.as_mut().unwrap() {
      *name = format!("cluster-{}", 8 - name.parse::<usize>().unwrap());
    }
    let renamed = fit_binary_clustered(&p, model, "renamed").unwrap();
    for (a, b) in grouped
      .covariance
      .iter()
      .flatten()
      .zip(renamed.covariance.iter().flatten())
    {
      assert!((a - b).abs() < 1e-12);
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
