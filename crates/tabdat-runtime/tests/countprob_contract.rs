use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{Command, EstatCommand, EstatSubcommand, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(missing: bool) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-countprob-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let y = [
      0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 5, 4, 1, 1, 0, 0,
      2, 2, 0, 1, 0,
    ];
    let mut csv = "id,y,x,z,person,cutoff\n".to_owned();
    if missing {
      csv.push_str("0,,0,0,0,2\n");
    }
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{},{},{}\n",
        i + 1,
        y,
        -2. + 4. * i as f64 / 35.,
        [0., 1., 0.4][i % 3],
        i % 6,
        [0, 1, 9, 99][i % 4]
      ));
    }
    if missing {
      csv.push_str("37,0.5,,0,1,1\n38,100001,0,0,,1\n");
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }
  fn session(&self) -> Session {
    let mut session = Session::new();
    run(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
  fn upper_cutoff() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-countprob-upper-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let mut csv = "id,y,cutoff,log_time\n".to_owned();
    for id in 0..101 {
      csv.push_str(&format!("{},{},100000,0\n", id, id % 5));
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn report(s: &mut Session) -> tabdat_domain::CountProbabilityResult {
  let ExecutionResult::CountProbabilities(r) = run(s, "estat countprob, max(2)") else {
    panic!("count probabilities")
  };
  *r
}

#[test]
fn covariance_selection_does_not_change_distributional_plugin_bins_or_mutate_state() {
  let f = Fixture::new(false);
  for family in ["poisson", "nbreg"] {
    let mut tables = Vec::new();
    for option in ["", ", robust", ", cluster(person)"] {
      let mut s = f.session();
      run(&mut s, "label variable y \"retained counts\"");
      run(&mut s, &format!("{family} y x z{option}"));
      let before = s.state();
      let rows = run(&mut s, "head 100");
      let r = report(&mut s);
      assert_eq!(r.outcome, "y");
      assert_eq!(r.table.observation_count, 36);
      assert_eq!(
        r.sample.as_ref().unwrap().retained_indices,
        (0..36).collect::<Vec<_>>()
      );
      assert_eq!(
        r.table.bins.iter().map(|b| b.observed_count).sum::<usize>(),
        36
      );
      assert_eq!(r.table.bins[0].observed_count, 15);
      assert_eq!(r.table.bins[1].observed_count, 7);
      assert_eq!(r.table.bins[2].observed_count, 14);
      assert!((r.table.bins.iter().map(|b| b.fitted_fraction).sum::<f64>() - 1.).abs() < 1e-12);
      assert_eq!(s.state(), before);
      assert_eq!(run(&mut s, "head 100"), rows);
      tables.push(r.table);
    }
    assert_eq!(tables[0], tables[1]);
    assert_eq!(tables[0], tables[2]);
  }
}

#[test]
fn future_count_intervals_use_current_mean_and_publish_atomically() {
  let fixture = Fixture::new(true);
  for family in ["poisson", "nbreg"] {
    let mut covariance_results = Vec::new();
    for covariance_option in ["", " robust", " cluster(person)"] {
      let mut session = fixture.session();
      run(&mut session, "drop if id == 38");
      run(&mut session, "generate log_time = .2 * x");
      run(
        &mut session,
        &format!("{family} y x z, offset(log_time){covariance_option}"),
      );
      let retained = session.state().estimation;
      run(&mut session, "drop y person");
      run(&mut session, "replace x = x + .2");
      run(&mut session, "replace log_time = log_time - .3");
      run(&mut session, "predict future, n interval level(90)");
      run(&mut session, "predict conditional_mean, n");
      let ExecutionResult::Head(table) = run(&mut session, "head 100") else {
        panic!("head")
      };
      let index = |name: &str| {
        table
          .columns
          .iter()
          .position(|column| column == name)
          .unwrap()
      };
      let (mean_index, point_index, lower_index, upper_index) = (
        index("conditional_mean"),
        index("future"),
        index("future_lower"),
        index("future_upper"),
      );
      let mut rows = Vec::new();
      for row in &table.rows {
        let mean = &row[mean_index];
        let (point, lower, upper) = (&row[point_index], &row[lower_index], &row[upper_index]);
        match (point, mean) {
          (tabdat_domain::CellValue::Float(point), tabdat_domain::CellValue::Float(mean)) => {
            assert!((point - mean).abs() < 1e-12)
          }
          (tabdat_domain::CellValue::Null, tabdat_domain::CellValue::Null) => {}
          other => panic!("mean and interval point do not share missingness: {other:?}"),
        }
        match (point, lower, upper) {
          (
            tabdat_domain::CellValue::Float(mean),
            tabdat_domain::CellValue::Float(lower),
            tabdat_domain::CellValue::Float(upper),
          ) => {
            assert!(mean.is_finite() && *lower >= 0.0 && lower.fract() == 0.0);
            assert!(lower <= upper && *upper <= 100000.0 && upper.fract() == 0.0);
          }
          (
            tabdat_domain::CellValue::Null,
            tabdat_domain::CellValue::Null,
            tabdat_domain::CellValue::Null,
          ) => {}
          other => panic!("expected an atomic mean/count interval, got {other:?}"),
        }
        rows.push((point.clone(), lower.clone(), upper.clone()));
      }
      assert!(matches!(
        table.rows[37].get(lower_index),
        Some(tabdat_domain::CellValue::Null)
      ));
      assert_eq!(session.state().estimation, retained);
      covariance_results.push(rows);

      run(&mut session, "generate collision_lower = 1");
      let before = session.state();
      assert!(
        session
          .execute(parse_command("predict collision, n interval").unwrap())
          .is_err()
      );
      assert_eq!(session.state(), before);
      assert!(
        session
          .execute(
            parse_command("predict invalid_level, n interval level(0.000000000000001)").unwrap()
          )
          .is_err()
      );
      assert_eq!(session.state(), before);
    }
    assert_eq!(covariance_results[0], covariance_results[1]);
    assert_eq!(covariance_results[0], covariance_results[2]);
  }

  let mut empty = fixture.session();
  run(&mut empty, "drop if id == 38");
  run(&mut empty, "generate log_time = .2 * x");
  run(&mut empty, "poisson y x z, offset(log_time)");
  run(&mut empty, "drop if id >= 0");
  run(&mut empty, "predict empty_future, n interval");
  assert_eq!(empty.state().active_dataset.as_ref().unwrap().row_count, 0);
  assert!(
    empty
      .state()
      .active_dataset
      .unwrap()
      .columns
      .iter()
      .any(|column| column.name == "empty_future_upper")
  );

  let mut work_limited = fixture.session();
  run(&mut work_limited, "drop if id == 38");
  run(&mut work_limited, "generate log_time = .2 * x");
  run(&mut work_limited, "poisson y x z, offset(log_time)");
  run(&mut work_limited, "replace log_time = 10");
  let before = work_limited.state();
  let error = work_limited
    .execute(parse_command("predict too_wide, n interval").unwrap())
    .unwrap_err();
  assert!(
    error
      .to_string()
      .contains("predictive count interval work exceeds")
  );
  assert_eq!(work_limited.state(), before);

  let mut unsupported = fixture.session();
  run(&mut unsupported, "drop if id == 38");
  run(&mut unsupported, "generate log_time = .2 * x");
  run(&mut unsupported, "poisson y x z, offset(log_time)");
  run(&mut unsupported, "replace log_time = 12");
  let before = unsupported.state();
  let error = unsupported
    .execute(parse_command("predict beyond_support, n interval").unwrap())
    .unwrap_err();
  assert!(
    error
      .to_string()
      .contains("count endpoint exceeds the supported cutoff")
  );
  assert_eq!(unsupported.state(), before);
}

#[test]
fn exact_original_joint_sample_and_outcomes_survive_current_data_edits_and_drop() {
  let f = Fixture::new(true);
  for family in ["poisson", "nbreg"] {
    let mut s = f.session();
    run(&mut s, &format!("{family} y x z, cluster(person)"));
    let original = report(&mut s);
    let sample = original.sample.as_ref().unwrap();
    assert_eq!(sample.retained_indices, (1..=36).collect::<Vec<_>>());
    assert_eq!(sample.total_observations, 39);
    assert_eq!(sample.dropped_observations, 3);
    assert_eq!(sample.cluster_groups.as_ref().unwrap().len(), 36);
    run(&mut s, "replace y = 0");
    run(&mut s, "replace x = 100");
    run(&mut s, "drop y x z person");
    let before = s.state();
    let rows = run(&mut s, "head 100");
    assert_eq!(report(&mut s), original);
    assert_eq!(s.state(), before);
    assert_eq!(run(&mut s, "head 100"), rows);
  }
}

#[test]
fn invalid_direct_requests_and_wrong_fit_fail_read_only_without_lazy_backend_initialization() {
  let mut empty = Session::new();
  let before = empty.state();
  assert_eq!(
    empty
      .execute(parse_command("estat countprob").unwrap())
      .unwrap_err(),
    RuntimeError::EstatCountProbRequiresPriorCount
  );
  assert_eq!(empty.state(), before);
  assert!(empty.initialized_capabilities().is_empty());
  let f = Fixture::new(false);
  let mut s = f.session();
  run(&mut s, "regress y x z");
  let before = s.state();
  assert_eq!(
    s.execute(parse_command("estat countprob").unwrap())
      .unwrap_err(),
    RuntimeError::EstatCountProbRequiresPriorCount
  );
  assert_eq!(s.state(), before);
  run(&mut s, "nbreg y x z");
  let before = s.state();
  let rows = run(&mut s, "head 100");
  for cutoff in [0, 101, usize::MAX] {
    let command = Command::Estat {
      command: EstatCommand {
        subcommand: EstatSubcommand::CountProbabilities { tail_from: cutoff },
      },
    };
    assert!(matches!(
      s.execute(command),
      Err(RuntimeError::EstatCountProbFailed { .. })
    ));
    assert_eq!(s.state(), before);
    assert_eq!(run(&mut s, "head 100"), rows);
  }
}

#[test]
fn current_probabilities_preserve_retained_fit_labels_and_failure_state() {
  let f = Fixture::new(true);
  for family in ["poisson", "nbreg"] {
    let mut s = f.session();
    // Exclude the extra wide NB2 outcome while retaining the missing-y row for prediction.
    run(&mut s, "drop if id >= 37");
    run(&mut s, "label variable x \"original predictor\"");
    run(&mut s, "generate log_time = 0");
    run(&mut s, &format!("{family} y x z, offset(log_time)"));
    let before = s.state();
    run(&mut s, "predict p0, pmf(0)");
    let after = s.state();
    assert_eq!(after.estimation, before.estimation);
    assert_eq!(after.labels, before.labels);
    let ExecutionResult::Head(t) = run(&mut s, "head 1") else {
      panic!("table")
    };
    // First row has missing outcome, but complete predictors and a PMF prediction.
    assert!(
      matches!(t.rows[0].last(),Some(tabdat_domain::CellValue::Float(v)) if *v>0.0 && *v<1.0)
    );
    run(&mut s, "replace log_time = -1000");
    run(&mut s, "predict zero_cdf, cdf(0)");
    let ExecutionResult::Head(t) = run(&mut s, "head 100") else {
      panic!("head")
    };
    assert!(
      t.rows
        .iter()
        .all(|r| r.last() == Some(&tabdat_domain::CellValue::Float(1.0)))
    );
    run(&mut s, "predict zero_pmf, pmf(2)");
    let ExecutionResult::Head(t) = run(&mut s, "head 100") else {
      panic!("head")
    };
    assert!(
      t.rows
        .iter()
        .all(|r| r.last() == Some(&tabdat_domain::CellValue::Float(0.0)))
    );
    run(&mut s, "replace log_time = 1000");
    run(&mut s, "predict extreme, cdf(0)");
    let ExecutionResult::Head(t) = run(&mut s, "head 100") else {
      panic!("head")
    };
    assert!(
      t.rows
        .iter()
        .all(|r| r.last() == Some(&tabdat_domain::CellValue::Null))
    );
    assert_eq!(s.state().estimation, before.estimation);
    let stable = s.state();
    let rows = run(&mut s, "head 100");
    for command in [
      "predict p0, pmf(0)",
      "predict bad, pr",
      "predict bad, residuals",
    ] {
      assert!(s.execute(parse_command(command).unwrap()).is_err());
      assert_eq!(s.state(), stable);
      assert_eq!(run(&mut s, "head 100"), rows);
    }
    run(&mut s, "replace x = NULL");
    run(&mut s, "predict missing, pmf(0)");
    let ExecutionResult::Head(t) = run(&mut s, "head 100") else {
      panic!("head")
    };
    assert!(
      t.rows
        .iter()
        .all(|r| r.last() == Some(&tabdat_domain::CellValue::Null))
    );
    run(&mut s, "drop if id >= 0");
    run(&mut s, "predict empty, cdf(99)");
    let stable = s.state();
    let Command::Predict { mut command } = parse_command("predict invalid, pmf(0)").unwrap() else {
      panic!("predict")
    };
    command.kind = tabdat_domain::PredictKind::CountPmf(usize::MAX);
    assert!(s.execute(Command::Predict { command }).is_err());
    assert_eq!(s.state(), stable);
    run(&mut s, "drop x");
    let stable = s.state();
    assert!(
      s.execute(parse_command("predict bad, cdf(2)").unwrap())
        .is_err()
    );
    assert_eq!(s.state(), stable);
  }
}

#[test]
fn count_probability_standard_errors_use_retained_covariance_and_current_design() {
  let fixture = Fixture::new(true);
  for family in ["poisson", "nbreg"] {
    let mut estimates = Vec::new();
    let mut interval_widths = Vec::new();
    for option in ["", ", robust", ", cluster(person)"] {
      let mut session = fixture.session();
      // Keep an invalid outcome with missing x for a NULL prediction, while
      // dropping the separate out-of-domain count before model fitting.
      run(&mut session, "drop if id == 38");
      run(&mut session, &format!("{family} y x z{option}"));
      let retained_fit = session.state().estimation;
      run(&mut session, "drop y person");
      run(&mut session, "predict p2, pmf(2)");
      run(&mut session, "predict p2_se, pmf(2) se");
      run(&mut session, "predict row_se, pmf(cutoff) se");
      run(&mut session, "predict c2, cdf(2)");
      run(&mut session, "predict c2_se, cdf(2) se");
      run(&mut session, "predict row_cdf_se, cdf(cutoff) se");
      run(&mut session, "predict p2_ci, pmf(2) interval");
      run(&mut session, "predict c2_ci, cdf(2) interval level(90)");
      let ExecutionResult::Head(table) = run(&mut session, "head 100") else {
        panic!("head")
      };
      let value = |name: &str| {
        let column = table
          .columns
          .iter()
          .position(|column| column == name)
          .unwrap();
        match &table.rows[0][column] {
          tabdat_domain::CellValue::Float(value) => *value,
          other => panic!("expected numeric prediction {name}, got {other:?}"),
        }
      };
      let point = value("p2");
      let standard_error = value("p2_se");
      let row_specific_se = value("row_se");
      let cdf = value("c2");
      let cdf_standard_error = value("c2_se");
      let row_specific_cdf_se = value("row_cdf_se");
      let pmf_interval = (value("p2_ci_lower"), value("p2_ci"), value("p2_ci_upper"));
      let cdf_interval = (value("c2_ci_lower"), value("c2_ci"), value("c2_ci_upper"));
      assert!(point > 0.0 && point < 1.0);
      assert!(standard_error.is_finite() && standard_error > 0.0);
      assert!((standard_error - row_specific_se).abs() < 1e-12);
      assert!(cdf > 0.0 && cdf < 1.0);
      assert!(cdf_standard_error.is_finite() && cdf_standard_error > 0.0);
      assert!(row_specific_cdf_se.is_finite() && row_specific_cdf_se > 0.0);
      assert!((row_specific_cdf_se - cdf_standard_error).abs() < 1e-12);
      assert!((pmf_interval.1 - point).abs() < 1e-12);
      assert!((cdf_interval.1 - cdf).abs() < 1e-12);
      assert!(0.0 <= pmf_interval.0 && pmf_interval.0 <= point && point <= pmf_interval.2);
      assert!(0.0 <= cdf_interval.0 && cdf_interval.0 <= cdf && cdf <= cdf_interval.2);
      assert!(pmf_interval.2 <= 1.0 && cdf_interval.2 <= 1.0);
      interval_widths.push((
        pmf_interval.2 - pmf_interval.0,
        cdf_interval.2 - cdf_interval.0,
      ));
      // Row zero has a missing fitted outcome but usable current x; row 37 has
      // a missing predictor and therefore no point probability or SE.
      assert!(matches!(
        table.rows[0].last(),
        Some(tabdat_domain::CellValue::Float(_))
      ));
      assert_eq!(table.rows[37].last(), Some(&tabdat_domain::CellValue::Null));
      assert_eq!(
        table.rows[37].get(
          table
            .columns
            .iter()
            .position(|name| name == "p2_ci_lower")
            .unwrap()
        ),
        Some(&tabdat_domain::CellValue::Null)
      );
      assert_eq!(
        table.rows[37].get(
          table
            .columns
            .iter()
            .position(|name| name == "p2_ci_upper")
            .unwrap()
        ),
        Some(&tabdat_domain::CellValue::Null)
      );
      assert_eq!(session.state().estimation, retained_fit);
      let before = session.state();
      assert!(
        session
          .execute(parse_command("predict p2_se, pmf(2) se").unwrap())
          .is_err()
      );
      assert!(
        session
          .execute(parse_command("predict c2_se, cdf(2) se").unwrap())
          .is_err()
      );
      assert_eq!(session.state(), before);
      run(&mut session, "generate ci_collision_lower = 1");
      let before = session.state();
      assert!(
        session
          .execute(parse_command("predict ci_collision, cdf(2) interval").unwrap())
          .is_err()
      );
      assert_eq!(session.state(), before);
      estimates.push((point, standard_error, cdf, cdf_standard_error));
    }
    let point = estimates[0].0;
    assert!(
      estimates
        .iter()
        .all(|(value, _, _, _)| (value - point).abs() < 1e-12)
    );
    assert!(
      estimates
        .windows(2)
        .any(|pair| (pair[0].1 - pair[1].1).abs() > 1e-8)
    );
    let cdf = estimates[0].2;
    assert!(
      estimates
        .iter()
        .all(|(_, _, value, _)| (value - cdf).abs() < 1e-12)
    );
    assert!(
      estimates
        .windows(2)
        .any(|pair| (pair[0].3 - pair[1].3).abs() > 1e-8)
    );
    assert!(
      interval_widths
        .windows(2)
        .any(|pair| (pair[0].0 - pair[1].0).abs() > 1e-8)
    );
    assert!(
      interval_widths
        .windows(2)
        .any(|pair| (pair[0].1 - pair[1].1).abs() > 1e-8)
    );
  }

  for (family, term, generated) in [
    ("poisson", "offset(log_time)", "generate log_time = 0.1"),
    (
      "nbreg",
      "exposure(time_at_risk)",
      "generate time_at_risk = 1.5",
    ),
  ] {
    let mut session = fixture.session();
    run(&mut session, "drop if id == 38");
    run(&mut session, generated);
    run(&mut session, &format!("{family} y x z, {term}"));
    run(&mut session, "drop y person");
    run(&mut session, "predict p0_se, pmf(0) se");
    run(&mut session, "predict cdf0_se, cdf(0) se");
    run(&mut session, "predict exposure_cdf, cdf(2) interval");
    let ExecutionResult::Head(table) = run(&mut session, "head 100") else {
      panic!("head")
    };
    assert!(matches!(
      table.rows[0].get(table.columns.iter().position(|name| name == "cdf0_se").unwrap()),
      Some(tabdat_domain::CellValue::Float(value)) if *value > 0.0
    ));
    let estimate_index = table
      .columns
      .iter()
      .position(|name| name == "exposure_cdf")
      .unwrap();
    let lower_index = table
      .columns
      .iter()
      .position(|name| name == "exposure_cdf_lower")
      .unwrap();
    let upper_index = table
      .columns
      .iter()
      .position(|name| name == "exposure_cdf_upper")
      .unwrap();
    let (
      Some(tabdat_domain::CellValue::Float(estimate)),
      Some(tabdat_domain::CellValue::Float(lower)),
      Some(tabdat_domain::CellValue::Float(upper)),
    ) = (
      table.rows[0].get(estimate_index),
      table.rows[0].get(lower_index),
      table.rows[0].get(upper_index),
    )
    else {
      panic!("expected an offset/exposure CDF interval")
    };
    assert!(lower <= estimate && estimate <= upper && *lower >= 0.0 && *upper <= 1.0);
    assert_eq!(table.rows[37].last(), Some(&tabdat_domain::CellValue::Null));
  }
}

#[test]
fn unrepresentable_count_interval_levels_fail_atomically() {
  let fixture = Fixture::new(true);
  for family in ["poisson", "nbreg"] {
    let mut session = fixture.session();
    run(&mut session, "drop if id == 38");
    run(&mut session, &format!("{family} y x z"));
    let before = session.state();
    assert!(
      session
        .execute(
          parse_command("predict invalid_level, pmf(0) interval level(0.000000000000001)").unwrap()
        )
        .is_err()
    );
    assert_eq!(session.state(), before);
  }
}

#[test]
fn row_specific_cutoffs_match_scalar_predictions_and_invalid_rows_publish_nothing() {
  let f = Fixture::new(false);
  for family in ["poisson", "nbreg"] {
    let mut s = f.session();
    run(&mut s, &format!("{family} y x z"));
    let retained = s.state().estimation;
    for cutoff in [0, 1, 9, 99] {
      run(
        &mut s,
        &format!("predict scalar_pmf_{cutoff}, pmf({cutoff})"),
      );
      run(
        &mut s,
        &format!("predict scalar_cdf_{cutoff}, cdf({cutoff})"),
      );
      run(
        &mut s,
        &format!("predict scalar_pmf_interval_{cutoff}, pmf({cutoff}) interval"),
      );
      run(
        &mut s,
        &format!("predict scalar_cdf_interval_{cutoff}, cdf({cutoff}) interval"),
      );
    }
    run(&mut s, "predict row_pmf, pmf(cutoff)");
    run(&mut s, "predict row_cdf, cdf(cutoff)");
    run(&mut s, "predict row_pmf_interval, pmf(cutoff) interval");
    run(&mut s, "predict row_cdf_interval, cdf(cutoff) interval");
    let ExecutionResult::Head(table) = run(&mut s, "head 100") else {
      panic!("head")
    };
    let column = |name: &str| {
      table
        .columns
        .iter()
        .position(|header| header == name)
        .unwrap()
    };
    let value = |row: usize, column: usize| match &table.rows[row][column] {
      tabdat_domain::CellValue::Float(value) => *value,
      tabdat_domain::CellValue::SignedInteger(value) => *value as f64,
      tabdat_domain::CellValue::UnsignedInteger(value) => *value as f64,
      cell => panic!("expected numeric prediction, found {cell:?}"),
    };
    for row in 0..table.rows.len() {
      let cutoff = [0, 1, 9, 99][row % 4];
      assert_eq!(
        value(row, column("row_pmf")),
        value(row, column(&format!("scalar_pmf_{cutoff}")))
      );
      assert_eq!(
        value(row, column("row_cdf")),
        value(row, column(&format!("scalar_cdf_{cutoff}")))
      );
      for (row_prefix, scalar_prefix) in [
        ("row_pmf_interval", "scalar_pmf_interval"),
        ("row_cdf_interval", "scalar_cdf_interval"),
      ] {
        for suffix in ["", "_lower", "_upper"] {
          assert_eq!(
            value(row, column(&format!("{row_prefix}{suffix}"))),
            value(row, column(&format!("{scalar_prefix}_{cutoff}{suffix}")))
          );
        }
      }
    }
    assert_eq!(s.state().estimation, retained);

    for invalid_cutoff in ["-1", "100001"] {
      let mut invalid = f.session();
      run(&mut invalid, &format!("{family} y x z"));
      run(&mut invalid, &format!("replace cutoff = {invalid_cutoff}"));
      let before = invalid.state();
      let rows = run(&mut invalid, "head 100");
      assert!(
        invalid
          .execute(parse_command("predict invalid_probability, pmf(cutoff)").unwrap())
          .is_err()
      );
      assert_eq!(invalid.state(), before);
      assert!(
        invalid
          .execute(parse_command("predict invalid_probability_se, pmf(cutoff) se").unwrap())
          .is_err()
      );
      assert_eq!(invalid.state(), before);
      assert!(
        invalid
          .execute(parse_command("predict invalid_cdf_se, cdf(cutoff) se").unwrap())
          .is_err()
      );
      assert_eq!(invalid.state(), before);
      assert!(
        invalid
          .execute(parse_command("predict invalid_interval, cdf(cutoff) interval").unwrap())
          .is_err()
      );
      assert_eq!(invalid.state(), before);
      assert_eq!(run(&mut invalid, "head 100"), rows);
    }

    let mut fractional = f.session();
    run(&mut fractional, &format!("{family} y x z"));
    run(&mut fractional, "generate fractional_cutoff = cutoff + .5");
    let before = fractional.state();
    assert!(
      fractional
        .execute(parse_command("predict invalid_probability, pmf(fractional_cutoff)").unwrap())
        .is_err()
    );
    assert_eq!(fractional.state(), before);
    assert!(
      fractional
        .execute(
          parse_command("predict invalid_probability_se, pmf(fractional_cutoff) se").unwrap()
        )
        .is_err()
    );
    assert_eq!(fractional.state(), before);
    assert!(
      fractional
        .execute(parse_command("predict invalid_cdf_se, cdf(fractional_cutoff) se").unwrap())
        .is_err()
    );
    assert_eq!(fractional.state(), before);
    assert!(
      fractional
        .execute(
          parse_command("predict invalid_interval, cdf(fractional_cutoff) interval").unwrap()
        )
        .is_err()
    );
    assert_eq!(fractional.state(), before);

    // Invalid cutoff values remain errors even on rows whose model predictors are missing.
    let mut masked_invalid = f.session();
    run(&mut masked_invalid, &format!("{family} y x z"));
    run(&mut masked_invalid, "replace x = NULL");
    run(&mut masked_invalid, "replace cutoff = 100001");
    let before = masked_invalid.state();
    assert!(
      masked_invalid
        .execute(parse_command("predict invalid_probability, pmf(cutoff)").unwrap())
        .is_err()
    );
    assert_eq!(masked_invalid.state(), before);
    assert!(
      masked_invalid
        .execute(parse_command("predict invalid_probability_se, pmf(cutoff) se").unwrap())
        .is_err()
    );
    assert_eq!(masked_invalid.state(), before);
    assert!(
      masked_invalid
        .execute(parse_command("predict invalid_cdf_se, cdf(cutoff) se").unwrap())
        .is_err()
    );
    assert_eq!(masked_invalid.state(), before);
    assert!(
      masked_invalid
        .execute(parse_command("predict invalid_interval, cdf(cutoff) interval").unwrap())
        .is_err()
    );
    assert_eq!(masked_invalid.state(), before);

    let mut missing = f.session();
    run(&mut missing, &format!("{family} y x z"));
    run(&mut missing, "replace cutoff = NULL");
    run(&mut missing, "predict missing_probability, cdf(cutoff)");
    run(&mut missing, "predict missing_se, pmf(cutoff) se");
    run(&mut missing, "predict missing_cdf_se, cdf(cutoff) se");
    run(
      &mut missing,
      "predict missing_cdf_interval, cdf(cutoff) interval",
    );
    let ExecutionResult::Head(missing_table) = run(&mut missing, "head 100") else {
      panic!("head")
    };
    let index = missing_table
      .columns
      .iter()
      .position(|name| name == "missing_probability")
      .unwrap();
    assert!(
      missing_table
        .rows
        .iter()
        .all(|row| row[index] == tabdat_domain::CellValue::Null)
    );
    let se_index = missing_table
      .columns
      .iter()
      .position(|name| name == "missing_se")
      .unwrap();
    assert!(
      missing_table
        .rows
        .iter()
        .all(|row| row[se_index] == tabdat_domain::CellValue::Null)
    );
    let cdf_se_index = missing_table
      .columns
      .iter()
      .position(|name| name == "missing_cdf_se")
      .unwrap();
    assert!(
      missing_table
        .rows
        .iter()
        .all(|row| row[cdf_se_index] == tabdat_domain::CellValue::Null)
    );
    for name in [
      "missing_cdf_interval",
      "missing_cdf_interval_lower",
      "missing_cdf_interval_upper",
    ] {
      let index = missing_table
        .columns
        .iter()
        .position(|column| column == name)
        .unwrap();
      assert!(
        missing_table
          .rows
          .iter()
          .all(|row| row[index] == tabdat_domain::CellValue::Null)
      );
    }
  }
}

#[test]
fn large_current_row_cutoffs_match_r_and_expansion_budget_fails_atomically() {
  let f = Fixture::upper_cutoff();
  let mut s = f.session();
  run(&mut s, "poisson y, offset(log_time)");
  let retained = s.state().estimation.clone();
  run(&mut s, "keep if id == 0");
  let intercept = match &s.state().estimation {
    Some(tabdat_domain::EstimationState::PoissonFit(fit)) => fit.result.coefficients[0].value,
    other => panic!("expected retained Poisson fit, found {other:?}"),
  };
  let offset = 100000_f64.ln() - intercept;
  run(&mut s, &format!("replace log_time = {offset:.17}"));
  run(&mut s, "predict high_pmf, pmf(100000)");
  run(&mut s, "predict high_cdf, cdf(cutoff)");
  run(&mut s, "predict high_cdf_se, cdf(cutoff) se");
  assert_eq!(s.state().estimation, retained);
  let ExecutionResult::Head(table) = run(&mut s, "head 1") else {
    panic!("head")
  };
  let value = |name: &str| {
    let column = table
      .columns
      .iter()
      .position(|candidate| candidate == name)
      .unwrap();
    match &table.rows[0][column] {
      tabdat_domain::CellValue::Float(value) => *value,
      other => panic!("expected a numeric probability, found {other:?}"),
    }
  };
  let expected: Vec<_> =
    include_str!("../../tabdat-stats/tests/fixtures/count_point_large_reference.csv")
      .lines()
      .find(|line| line.starts_with("poisson,NA,1e+05,1e+05,"))
      .unwrap()
      .split(',')
      .collect();
  for (name, reference) in [
    ("high_pmf", expected[4].parse::<f64>().unwrap()),
    ("high_cdf", expected[5].parse::<f64>().unwrap()),
  ] {
    let actual = value(name);
    assert!(
      (actual - reference).abs() <= reference.abs() * 1e-10 + f64::MIN_POSITIVE,
      "{name}: {actual} != {reference}"
    );
  }
  assert!(value("high_cdf_se").is_finite() && value("high_cdf_se") > 0.0);

  let mut too_wide = f.session();
  run(&mut too_wide, "poisson y, offset(log_time)");
  let before = too_wide.state();
  let error = too_wide
    .execute(parse_command("predict too_wide, pmf(cutoff)").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("expanded cutoff work exceeds"));
  assert_eq!(too_wide.state(), before);
  let error = too_wide
    .execute(parse_command("predict too_wide_se, pmf(cutoff) se").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("expanded cutoff work exceeds"));
  assert_eq!(too_wide.state(), before);
  let error = too_wide
    .execute(parse_command("predict too_wide_cdf_se, cdf(cutoff) se").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("expanded cutoff work exceeds"));
  assert_eq!(too_wide.state(), before);
  let error = too_wide
    .execute(parse_command("predict too_wide_interval, cdf(cutoff) interval").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("expanded cutoff work exceeds"));
  assert_eq!(too_wide.state(), before);
}
