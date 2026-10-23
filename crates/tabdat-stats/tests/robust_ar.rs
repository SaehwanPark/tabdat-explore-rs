use tabdat_stats::{
  CovarianceType, EstimationSample, InstrumentalVariablesProblem, robust_anderson_rubin_test,
};

fn problem(design: &str, clustered: bool) -> InstrumentalVariablesProblem {
  let rows = include_str!("fixtures/robust_ar_data.csv")
    .lines()
    .skip(1)
    .map(|line| {
      line
        .split(',')
        .map(|x| x.parse::<f64>().ok())
        .collect::<Vec<_>>()
    })
    .filter(|r| r[1..8].iter().all(Option::is_some) && (!clustered || r[8].is_some()))
    .collect::<Vec<_>>();
  InstrumentalVariablesProblem {
    outcome: "y".into(),
    exogenous_names: if design == "noexogenous" {
      vec![]
    } else {
      vec!["w".into()]
    },
    endogenous_names: vec!["x1".into(), "x2".into()],
    instrument_names: vec!["z1".into(), "z2".into(), "z3".into()],
    response: rows.iter().map(|r| r[1].unwrap()).collect(),
    exogenous: rows
      .iter()
      .map(|r| {
        if design == "noexogenous" {
          vec![]
        } else {
          vec![r[2].unwrap()]
        }
      })
      .collect(),
    endogenous: rows
      .iter()
      .map(|r| vec![r[3].unwrap(), r[4].unwrap()])
      .collect(),
    instruments: rows
      .iter()
      .map(|r| r[5..8].iter().map(|v| v.unwrap()).collect())
      .collect(),
    include_intercept: design != "nointercept",
    sample: Some(
      EstimationSample::new(
        rows.iter().map(|r| r[0].unwrap() as usize - 1).collect(),
        72,
        72 - rows.len(),
        None,
        clustered.then(|| rows.iter().map(|r| r[8].unwrap().to_string()).collect()),
      )
      .unwrap(),
    ),
  }
}

#[test]
fn robust_joint_ar_matches_independent_r_under_three_designs_and_two_covariances() {
  // base R4.6.1: fixtures/robust_ar_reference.R. Nonconsecutive omissions,
  // weak first stages, overidentification (K_aux != K_iv), nonzero joint null.
  for line in include_str!("fixtures/robust_ar_reference.csv")
    .lines()
    .skip(1)
  {
    let fields = line.split(',').collect::<Vec<_>>();
    let clustered = fields[1] == "cluster";
    let covariance = if clustered {
      CovarianceType::Cluster("g".into())
    } else {
      CovarianceType::RobustHc1
    };
    let p = problem(fields[0], clustered);
    let before = p.clone();
    let r = robust_anderson_rubin_test(&p, &[1.7, -0.6], &covariance).unwrap();
    assert_eq!(p, before);
    assert_eq!(r.endogenous, ["x1", "x2"]);
    assert_eq!(r.excluded_instruments, ["z1", "z2", "z3"]);
    assert_eq!(r.null_values, [1.7, -0.6]);
    assert_eq!(r.degrees_of_freedom, 3);
    assert_eq!(r.covariance_type, covariance);
    assert_eq!(r.observation_count, fields[2].parse::<usize>().unwrap());
    assert_eq!(
      r.auxiliary_parameter_count,
      fields[3].parse::<usize>().unwrap()
    );
    assert_eq!(
      r.auxiliary_residual_df,
      r.observation_count - r.auxiliary_parameter_count
    );
    assert_eq!(r.cluster_group_count, clustered.then_some(12));
    for (actual, index) in [
      (r.covariance_correction, 5),
      (r.statistic, 6),
      (r.p_value, 7),
    ] {
      let expected = fields[index].parse::<f64>().unwrap();
      assert!(
        (actual - expected).abs() <= 1e-10 * expected.abs().max(1e-30),
        "{line}: {actual} != {expected}"
      );
    }
    let retained = &p.sample.as_ref().unwrap().retained_indices;
    let expected = (0..72)
      .filter(|i| ![1, 7].contains(i) && (!clustered || *i != 16))
      .collect::<Vec<_>>();
    assert_eq!(*retained, expected);
    // Invariant to instrument units and reversed source/group order.
    let mut scaled = p.clone();
    for row in &mut scaled.instruments {
      row[0] *= 1e5;
      row[1] *= 1e-4;
    }
    scaled.response.reverse();
    scaled.endogenous.reverse();
    scaled.exogenous.reverse();
    scaled.instruments.reverse();
    let sample = scaled.sample.as_mut().unwrap();
    sample.retained_indices.reverse();
    if let Some(groups) = &mut sample.cluster_groups {
      groups.reverse();
    }
    let changed = robust_anderson_rubin_test(&scaled, &[1.7, -0.6], &covariance).unwrap();
    assert!((r.statistic - changed.statistic).abs() < 1e-9 * r.statistic);
  }
}

#[test]
fn robust_ar_rejects_undefined_joint_null_design_or_cluster_rank() {
  let covariance = CovarianceType::Cluster("g".into());
  let p = problem("controlled", true);
  for null in [vec![0.], vec![f64::NAN, 0.], vec![f64::MAX, f64::MAX]] {
    assert!(robust_anderson_rubin_test(&p, &null, &covariance).is_err());
  }
  assert!(robust_anderson_rubin_test(&p, &[0., 0.], &CovarianceType::NonRobust).is_err());
  let mut absent = p.clone();
  absent.sample = None;
  assert!(robust_anderson_rubin_test(&absent, &[0., 0.], &covariance).is_err());
  for groups in [1, 3] {
    let mut limited = p.clone();
    limited.sample.as_mut().unwrap().cluster_groups = Some(
      (0..p.response.len())
        .map(|i| (i % groups).to_string())
        .collect(),
    );
    let error = robust_anderson_rubin_test(&limited, &[1.7, -0.6], &covariance)
      .unwrap_err()
      .to_string();
    assert!(
      error.contains(if groups == 1 {
        "at least 2"
      } else {
        "insufficient_cluster_rank"
      }),
      "{error}"
    );
  }
  let mut rank = p.clone();
  for row in &mut rank.instruments {
    row[2] = row[0];
  }
  assert!(robust_anderson_rubin_test(&rank, &[0., 0.], &covariance).is_err());
  let mut perfect = p.clone();
  perfect.response = vec![0.; p.response.len()];
  assert!(robust_anderson_rubin_test(&perfect, &[0., 0.], &CovarianceType::RobustHc1).is_err());
}
