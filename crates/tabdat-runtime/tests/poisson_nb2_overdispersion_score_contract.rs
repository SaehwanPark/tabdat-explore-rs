use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

const DATA: &str = include_str!("../../../examples/nb2-overdispersion-score/score_reference.csv");
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-nb2-score-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("score_reference.csv"), DATA).unwrap();
    Self(root)
  }

  fn session(&self) -> Session {
    let mut session = Session::new();
    run(
      &mut session,
      &format!("use {}", self.0.join("score_reference.csv").display()),
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

#[test]
fn retained_poisson_score_uses_fit_time_outcomes_offsets_and_groups() {
  let fixture = Fixture::new();
  for (fit, expected_covariance, expected_cluster) in [
    (
      "poisson y_independent x, offset(log_exposure)",
      "poisson_information_nb2_score",
      false,
    ),
    (
      "poisson y_independent x, offset(log_exposure) robust",
      "robust_row_nb2_score_n_over_n_minus_1",
      false,
    ),
    (
      "poisson y_clustered x, offset(log_exposure) cluster(group)",
      "cluster_nb2_score_g_over_g_minus_1",
      true,
    ),
  ] {
    let mut session = fixture.session();
    let fit_result = run(&mut session, fit);
    assert!(matches!(fit_result, ExecutionResult::Poisson(_)));
    let result = run(&mut session, "estat overdispersion");
    let ExecutionResult::PoissonOverdispersionScoreTest(test) = result.clone() else {
      panic!("unexpected result: {result:?}")
    };
    assert_eq!(test.covariance, expected_covariance);
    assert_eq!(test.observation_count, 180);
    assert_eq!(test.offset_variable.as_deref(), Some("log_exposure"));
    assert_eq!(test.cluster.is_some(), expected_cluster);
    if expected_cluster {
      assert_eq!(test.cluster.as_ref().unwrap().group_count, 30);
    }

    // Current values and the named source columns are no longer consulted.
    run(&mut session, "replace x = x + 1");
    run(&mut session, "replace log_exposure = log_exposure + 2");
    run(&mut session, "replace y_independent = y_independent + 10");
    run(&mut session, "replace y_clustered = y_clustered + 10");
    run(&mut session, "replace group = group + 1000");
    assert_eq!(run(&mut session, "estat overdispersion"), result);
  }
}

#[test]
fn requires_a_retained_poisson_fit_and_preserves_state_on_failure() {
  let fixture = Fixture::new();
  let mut empty = Session::new();
  let before = empty.state();
  assert_eq!(
    empty
      .execute(parse_command("estat overdispersion").unwrap())
      .unwrap_err(),
    RuntimeError::EstatOverdispersionRequiresPriorPoisson
  );
  assert_eq!(empty.state(), before);

  let mut session = fixture.session();
  run(&mut session, "regress y_independent x");
  let before = session.state();
  assert_eq!(
    session
      .execute(parse_command("estat overdispersion").unwrap())
      .unwrap_err(),
    RuntimeError::EstatOverdispersionRequiresPriorPoisson
  );
  assert_eq!(session.state(), before);

  run(
    &mut session,
    "poisson y_independent x, offset(log_exposure) robust",
  );
  let previous = run(&mut session, "estat overdispersion");
  run(&mut session, "nbreg y_independent x, offset(log_exposure)");
  let before = session.state();
  assert_eq!(
    session
      .execute(parse_command("estat overdispersion").unwrap())
      .unwrap_err(),
    RuntimeError::EstatOverdispersionRequiresPriorPoisson
  );
  assert_eq!(session.state(), before);

  run(
    &mut session,
    "poisson y_independent x, offset(log_exposure)",
  );
  assert!(matches!(
    run(&mut session, "estat overdispersion"),
    ExecutionResult::PoissonOverdispersionScoreTest(_)
  ));
  assert_ne!(previous, run(&mut session, "estat overdispersion"));
}
