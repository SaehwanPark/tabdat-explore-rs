use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{EstimationState, ExecutionResult, RandomEffectsState};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(csv: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-panel-hausman-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), csv).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn execute(s: &mut Session, command: &str) -> ExecutionResult {
  s.execute(parse_command(command).unwrap())
    .unwrap_or_else(|e| panic!("{command}: {e}"))
}

#[test]
fn retained_design_survives_data_changes_and_legacy_state_deserializes() {
  let fixture = Fixture::new(include_str!(
    "../../tabdat-stats/tests/fixtures/panel_classical_hausman/data.csv"
  ));
  let mut session = fixture.load();
  assert!(
    session
      .execute(parse_command("estat hausman").unwrap())
      .is_err()
  );
  let ExecutionResult::RandomEffects(state) =
    execute(&mut session, "xtreg y x, re id(id) time(year)")
  else {
    panic!()
  };
  assert!(state.hausman_problem.is_some());
  let mut old = serde_json::to_value(&state).unwrap();
  old.as_object_mut().unwrap().remove("hausman_problem");
  let legacy: RandomEffectsState = serde_json::from_value(old).unwrap();
  assert!(legacy.hausman_problem.is_none());
  let ExecutionResult::ClassicalPanelHausman(reference) = execute(&mut session, "estat hausman")
  else {
    panic!()
  };
  assert_eq!(reference.observation_count, 200);
  assert_eq!(reference.group_count, 40);
  assert!(reference.variance_difference > 0.0);
  execute(&mut session, "replace x = x + 1");
  execute(&mut session, "drop id year y");
  let before = session.state();
  let ExecutionResult::ClassicalPanelHausman(actual) = execute(&mut session, "estat hausman")
  else {
    panic!()
  };
  assert_eq!(actual, reference);
  assert_eq!(session.state(), before);
  execute(&mut session, "predict current, xb");
}

#[test]
fn joint_finite_sample_includes_only_original_matched_rows() {
  let mut csv =
    include_str!("../../tabdat-stats/tests/fixtures/panel_classical_hausman/data.csv").to_owned();
  for id in 1..=40 {
    csv.push_str(&format!("{id},6,,1\n"));
  }
  csv.push_str("41,1,,1\n");
  let fixture = Fixture::new(&csv);
  let mut session = fixture.load();
  let ExecutionResult::RandomEffects(state) =
    execute(&mut session, "xtreg y x, re id(id) time(year)")
  else {
    panic!()
  };
  let sample = state
    .hausman_problem
    .as_ref()
    .unwrap()
    .sample
    .as_ref()
    .unwrap();
  assert_eq!(sample.total_observations, 241);
  assert_eq!(sample.dropped_observations, 41);
  assert_eq!(sample.retained_indices, (0..200).collect::<Vec<_>>());
  let ExecutionResult::ClassicalPanelHausman(result) = execute(&mut session, "estat hausman")
  else {
    panic!()
  };
  assert_eq!(result.observation_count, 200);
  assert_eq!(result.group_count, 40);
  assert!((result.statistic - 13.1737660105836).abs() < 1e-9);
}

#[test]
fn unsupported_multiple_slopes_and_unbalanced_panels_fail_atomically() {
  let fixture = Fixture::new(include_str!(
    "../../tabdat-stats/tests/fixtures/panel_classical_hausman/data.csv"
  ));
  let mut session = fixture.load();
  execute(&mut session, "generate z = year * id");
  execute(&mut session, "xtreg y x z, re id(id) time(year)");
  let before = session.state();
  assert!(
    session
      .execute(parse_command("estat hausman").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), before);
  execute(&mut session, "drop if id * 10 + year == 11");
  execute(&mut session, "xtreg y x, re id(id) time(year)");
  let before = session.state();
  let error = session
    .execute(parse_command("estat hausman").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("balanced"), "{error}");
  assert_eq!(session.state(), before);
  assert!(parse_command("estat hausman, robust").is_err());
  assert_eq!(session.state(), before);
}

#[test]
fn unsupported_covariance_and_negative_difference_fail_without_state_mutation() {
  let fixture = Fixture::new(include_str!(
    "../../tabdat-stats/tests/fixtures/panel_classical_hausman/data.csv"
  ));
  let mut session = fixture.load();
  execute(&mut session, "xtreg y x, re id(id) time(year) cluster(id)");
  let before = session.state();
  let error = session
    .execute(parse_command("estat hausman").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("nonclustered"), "{error}");
  assert_eq!(session.state(), before);
  execute(&mut session, "xtreg y x, mle id(id) time(year)");
  let before = session.state();
  assert!(
    session
      .execute(parse_command("estat hausman").unwrap())
      .is_err()
  );
  assert_eq!(session.state(), before);
  assert!(matches!(
    before.estimation,
    Some(EstimationState::RandomEffects(_))
  ));
  let fixture = Fixture::new(include_str!(
    "../../tabdat-stats/tests/fixtures/panel_hausman/inputs.csv"
  ));
  let mut session = fixture.load();
  execute(&mut session, "xtreg lnhr lnwg, re id(id) time(year)");
  let before = session.state();
  let error = session
    .execute(parse_command("estat hausman").unwrap())
    .unwrap_err();
  assert!(error.to_string().contains("nonpositive"), "{error}");
  assert_eq!(session.state(), before);
}
