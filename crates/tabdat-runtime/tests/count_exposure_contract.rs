use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const DATA: &str = include_str!("../../../examples/poisson-offset/poisson_offset.csv");
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new(extra: &str) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-count-exposure-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), format!("{DATA}{extra}")).unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    run(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    run(
      &mut session,
      r#"sql """SELECT *, exp(log_time) AS exposure, ln(exp(log_time)) AS log_exposure FROM active""" into exposure_design"#,
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}

fn rows(session: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(preview) = run(session, "head 100") else {
    panic!("head")
  };
  preview.rows
}

fn number(cell: &CellValue) -> f64 {
  let CellValue::Float(value) = cell else {
    panic!("expected DOUBLE, got {cell:?}")
  };
  *value
}

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-8,
    "{actual:.16e} != {expected:.16e}"
  );
}

#[test]
fn poisson_exposure_matches_equivalent_offset_and_uses_current_exposure_only_for_prediction() {
  let fixture = Fixture::new("37,0.5,0,0,,1\n38,0.5,0,0,Inf,1\n39,0.5,0,0,0.2,\n40,,0,0,0.2,1\n");
  let mut session = fixture.load();
  let original_rows = rows(&mut session);
  let ExecutionResult::Poisson(exposure_fit) = run(
    &mut session,
    "poisson y x z, exposure(exposure) cluster(person)",
  ) else {
    panic!("Poisson exposure fit")
  };
  assert_eq!(exposure_fit.offset_variable, None);
  assert_eq!(exposure_fit.exposure_variable.as_deref(), Some("exposure"));
  assert_eq!(
    exposure_fit.sample.as_ref().unwrap().retained_indices,
    (0..36).collect::<Vec<_>>()
  );
  assert_eq!(
    exposure_fit.sample.as_ref().unwrap().dropped_observations,
    4
  );
  assert_eq!(exposure_fit.cluster.as_ref().unwrap().group_count, 6);
  assert_eq!(rows(&mut session), original_rows);
  let ExecutionResult::PoissonOverdispersionScoreTest(score) =
    run(&mut session, "estat overdispersion")
  else {
    panic!("Poisson exposure overdispersion score")
  };
  assert_eq!(score.offset_variable, None);
  assert_eq!(score.exposure_variable.as_deref(), Some("exposure"));

  let ExecutionResult::Poisson(offset_fit) = run(
    &mut session,
    "poisson y x z, offset(log_exposure) cluster(person)",
  ) else {
    panic!("equivalent Poisson offset fit")
  };
  for (exposure, offset) in exposure_fit
    .coefficients
    .iter()
    .zip(&offset_fit.coefficients)
  {
    close(exposure.value, offset.value);
    close(exposure.standard_error, offset.standard_error);
  }
  for (exposure, offset) in exposure_fit
    .covariance
    .iter()
    .flatten()
    .zip(offset_fit.covariance.iter().flatten())
  {
    close(*exposure, *offset);
  }

  let ExecutionResult::Poisson(_) = run(
    &mut session,
    "poisson y x z, exposure(exposure) cluster(person)",
  ) else {
    panic!("refit Poisson exposure")
  };
  let retained_effects = run(&mut session, "estat margins");
  run(&mut session, "estimates store poisson_exposure");
  let ExecutionResult::EstimateTable(table) = run(&mut session, "estimates table poisson_exposure")
  else {
    panic!("Poisson estimate table")
  };
  assert_eq!(
    table.models[0].poisson_exposure_variable.as_deref(),
    Some("exposure")
  );
  assert_eq!(table.models[0].poisson_exposure_coefficient, Some(1.0));
  assert!(table.models[0].poisson_offset_variable.is_none());

  run(&mut session, "predict original_mean, n");
  run(&mut session, "predict original_xb, xb");
  run(&mut session, "predict original_interval, n interval");
  let predicted = rows(&mut session);
  for (index, row) in predicted.iter().take(36).enumerate() {
    close(number(&row[8]), exposure_fit.fitted_means[index]);
    close(number(&row[9]), number(&row[8]).ln());
    close(number(&row[10]), number(&row[8]));
    assert!(number(&row[11]).fract() == 0.0 && number(&row[12]).fract() == 0.0);
  }
  assert_eq!(predicted[36][8], CellValue::Null);
  assert_eq!(predicted[37][8], CellValue::Null);
  assert_eq!(predicted[36][10], CellValue::Null);
  assert_eq!(predicted[36][11], CellValue::Null);
  assert_eq!(predicted[36][12], CellValue::Null);
  assert!(number(&predicted[38][8]).is_finite());
  assert!(number(&predicted[39][8]).is_finite());

  run(&mut session, "replace exposure = exposure * 2");
  run(&mut session, "predict doubled_mean, n");
  let changed = rows(&mut session);
  for row in changed.iter().take(36) {
    close(number(&row[13]), number(&row[8]) * 2.0);
  }
  assert_eq!(run(&mut session, "estat margins"), retained_effects);

  run(
    &mut session,
    r#"sql """SELECT * REPLACE (CASE WHEN id=1 THEN 0.0 ELSE exposure END AS exposure) FROM active""" into invalid_current_exposure"#,
  );
  run(&mut session, "predict invalid_xb, xb");
  let invalid = rows(&mut session);
  assert_eq!(invalid[0].last(), Some(&CellValue::Null));
  assert!(number(invalid[1].last().unwrap()).is_finite());
  run(&mut session, "predict invalid_pmf, pmf(0)");
  let pmf = rows(&mut session);
  assert_eq!(pmf[0].last(), Some(&CellValue::Null));
  assert!(number(pmf[1].last().unwrap()).is_finite());
  run(&mut session, "predict invalid_cdf, cdf(0)");
  let cdf = rows(&mut session);
  assert_eq!(cdf[0].last(), Some(&CellValue::Null));
  assert!(number(cdf[1].last().unwrap()).is_finite());
}

#[test]
fn nb2_exposure_preserves_joint_inference_and_rejects_unsupported_countgof() {
  let fixture = Fixture::new("");
  let mut session = fixture.load();
  let ExecutionResult::Nbreg(exposure_fit) =
    run(&mut session, "nbreg y x z, exposure(exposure) robust")
  else {
    panic!("NB2 exposure fit")
  };
  assert_eq!(exposure_fit.offset_variable, None);
  assert_eq!(exposure_fit.exposure_variable.as_deref(), Some("exposure"));
  assert!(exposure_fit.covariance[0][3].abs() > 1e-6);
  let ExecutionResult::EstimateStore(_) = run(&mut session, "estimates store nb2_exposure") else {
    panic!("NB2 estimate store")
  };
  let ExecutionResult::EstimateTable(table) = run(&mut session, "estimates table nb2_exposure")
  else {
    panic!("NB2 estimate table")
  };
  assert_eq!(
    table.models[0]
      .nb2
      .as_ref()
      .unwrap()
      .exposure_variable
      .as_deref(),
    Some("exposure")
  );
  assert!(
    session
      .execute(parse_command("estat countgof").unwrap())
      .is_err()
  );

  let ExecutionResult::Nbreg(offset_fit) =
    run(&mut session, "nbreg y x z, offset(log_exposure) robust")
  else {
    panic!("equivalent NB2 offset fit")
  };
  for (exposure, offset) in exposure_fit
    .coefficients
    .iter()
    .zip(&offset_fit.coefficients)
  {
    close(exposure.value, offset.value);
    close(exposure.standard_error, offset.standard_error);
  }
  close(exposure_fit.dispersion.alpha, offset_fit.dispersion.alpha);
  for (exposure, offset) in exposure_fit
    .covariance
    .iter()
    .flatten()
    .zip(offset_fit.covariance.iter().flatten())
  {
    close(*exposure, *offset);
  }

  let ExecutionResult::Nbreg(_) = run(&mut session, "nbreg y x z, exposure(exposure) robust")
  else {
    panic!("refit NB2 exposure")
  };
  let retained_effects = run(&mut session, "estat margins");
  run(&mut session, "predict original_mean, n");
  run(&mut session, "predict original_xb, xb");
  run(&mut session, "predict original_interval, n interval");
  let original = rows(&mut session);
  for (index, row) in original.iter().enumerate() {
    close(number(&row[8]), exposure_fit.fitted_means[index]);
    close(number(&row[9]), number(&row[8]).ln());
    close(number(&row[10]), number(&row[8]));
    assert!(number(&row[11]).fract() == 0.0 && number(&row[12]).fract() == 0.0);
  }
  run(&mut session, "replace exposure = exposure * 2");
  run(&mut session, "predict doubled_mean, n");
  for row in rows(&mut session) {
    close(number(&row[13]), number(&row[8]) * 2.0);
  }
  assert_eq!(run(&mut session, "estat margins"), retained_effects);
}

#[test]
fn finite_nonpositive_exposure_fails_fit_and_missing_exposure_rows_are_excluded() {
  let fixture = Fixture::new("37,1,0.1,0,0.2,1\n");
  let mut session = fixture.load();
  run(
    &mut session,
    r#"sql """SELECT * REPLACE (CASE WHEN id=37 THEN NULL::DOUBLE ELSE exposure END AS exposure) FROM active""" into missing_exposure"#,
  );
  let ExecutionResult::Poisson(fit) = run(&mut session, "poisson y x z, exposure(exposure)") else {
    panic!("fit with missing exposure")
  };
  assert_eq!(fit.observation_count, 36);
  assert_eq!(fit.sample.as_ref().unwrap().dropped_observations, 1);

  run(
    &mut session,
    r#"sql """SELECT * REPLACE (CASE WHEN id=1 THEN 0.0 ELSE exposure END AS exposure) FROM active""" into nonpositive_exposure"#,
  );
  let mut expected = session.state();
  expected.estimation = None;
  let unchanged_rows = rows(&mut session);
  assert!(
    session
      .execute(parse_command("poisson y x z, exposure(exposure)").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), expected);
  assert_eq!(rows(&mut session), unchanged_rows);
}

#[test]
fn exposure_column_can_be_a_predictor_without_adding_a_log_chain_rule_to_margins() {
  let fixture = Fixture::new("");
  let mut session = fixture.load();
  let ExecutionResult::Poisson(fit) = run(&mut session, "poisson y x exposure, exposure(exposure)")
  else {
    panic!("Poisson fit")
  };
  let ExecutionResult::PoissonMarginalEffects(effects) = run(&mut session, "estat margins") else {
    panic!("Poisson margins")
  };
  assert_eq!(effects.exposure_variable.as_deref(), Some("exposure"));
  let exposure_coefficient = fit
    .coefficients
    .iter()
    .find(|estimate| estimate.name == "exposure")
    .unwrap()
    .value;
  let expected =
    exposure_coefficient * fit.fitted_means.iter().sum::<f64>() / fit.fitted_means.len() as f64;
  close(effects.effects[1].value, expected);
}
