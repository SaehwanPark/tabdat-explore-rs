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
    }
    run(&mut s, "predict row_pmf, pmf(cutoff)");
    run(&mut s, "predict row_cdf, cdf(cutoff)");
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

    let mut missing = f.session();
    run(&mut missing, &format!("{family} y x z"));
    run(&mut missing, "replace cutoff = NULL");
    run(&mut missing, "predict missing_probability, cdf(cutoff)");
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

  let mut too_wide = f.session();
  run(&mut too_wide, "poisson y, offset(log_time)");
  let before = too_wide.state();
  let error = too_wide
    .execute(parse_command("predict too_wide, pmf(cutoff)").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("expanded cutoff work exceeds"));
  assert_eq!(too_wide.state(), before);
}
