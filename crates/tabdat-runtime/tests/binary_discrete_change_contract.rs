use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{ExecutionResult, RuntimeError};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-discrete-change-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      include_str!("../../../examples/binary-discrete-change/binary_discrete_change.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn execute(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn fail(session: &mut Session, command: &str) -> RuntimeError {
  session
    .execute(parse_command(command).unwrap())
    .unwrap_err()
}

fn close(actual: f64, expected: f64) {
  assert!((actual - expected).abs() < 2e-10, "{actual} != {expected}");
}

#[test]
fn discrete_probability_changes_use_the_retained_logit_or_probit_design() {
  let fixture = Fixture::new();
  for (fit_command, expected, standard_error) in [
    (
      "logit y x treated, cluster(group)",
      0.183329669025602,
      0.0882884728319724,
    ),
    (
      "probit y x treated, robust",
      0.182161202616056,
      0.0874502399969682,
    ),
  ] {
    let mut session = fixture.load();
    assert!(matches!(
      fail(&mut session, "estat margins, discrete(treated)"),
      RuntimeError::EstatMarginsFailed { .. }
    ));
    execute(&mut session, fit_command);
    let before = session.state().estimation;
    let ExecutionResult::BinaryDiscreteChange(result) =
      execute(&mut session, "estat margins, discrete(treated)")
    else {
      panic!("expected discrete binary probability change")
    };
    close(result.effect.value, expected);
    close(result.effect.standard_error, standard_error);
    assert_eq!(result.observation_count, 120);
    assert_eq!((result.from_value, result.to_value), (0, 1));
    assert_eq!(session.state().estimation, before);

    // Current data edits must not change post-estimation effects computed from
    // the owned fit design; a failed request must leave that fit available.
    execute(&mut session, "replace treated = 1 - treated");
    assert!(
      fail(&mut session, "estat margins, discrete(missing)")
        .to_string()
        .contains("not in the retained model")
    );
    assert!(
      fail(&mut session, "estat margins, discrete(x)")
        .to_string()
        .contains("only 0 and 1")
    );
    let ExecutionResult::BinaryDiscreteChange(after_edit) =
      execute(&mut session, "estat margins, discrete(treated)")
    else {
      panic!("expected discrete binary probability change after recovery")
    };
    close(after_edit.effect.value, expected);
    close(after_edit.effect.standard_error, standard_error);
  }
}

#[test]
fn bare_continuous_margins_keeps_its_existing_binary_fit_contract() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "logit y x treated");
  assert!(matches!(
    execute(&mut session, "estat margins"),
    ExecutionResult::LogitMarginalEffects(_)
  ));
}
