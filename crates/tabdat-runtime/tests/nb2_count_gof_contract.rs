use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-nb2-count-gof-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let y = [
      0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 5, 4, 1, 1, 0, 0,
      2, 2, 0, 1, 0,
    ];
    let mut csv = "id,y,x,z,person,offset\n".to_owned();
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{},{},0\n",
        i + 1,
        y,
        -2. + 4. * i as f64 / 35.,
        [0., 1., 0.4][i % 3],
        i % 6
      ));
    }
    csv.push_str("37,,0,0,0,0\n38,100001,,0,1,0\n39,0,,0,2,0\n");
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
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn report(session: &mut Session) -> tabdat_domain::CountGoodnessOfFitResult {
  let ExecutionResult::CountGoodnessOfFit(result) = run(session, "estat countgof") else {
    panic!("count goodness of fit")
  };
  *result
}

#[test]
fn countgof_uses_retained_nb2_rows_and_is_read_only_across_current_data_edits() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  run(&mut session, "label variable y \"retained count\"");
  run(&mut session, "nbreg y x z");
  let original = report(&mut session);
  assert_eq!(original.test.observation_count, 36);
  assert_eq!(original.test.tail_from, 10);
  assert_eq!(original.test.degrees_of_freedom, 10);
  assert_eq!(
    original.sample.as_ref().unwrap().retained_indices,
    (0..36).collect::<Vec<_>>()
  );
  assert_eq!(original.sample.as_ref().unwrap().total_observations, 39);
  assert_eq!(original.sample.as_ref().unwrap().dropped_observations, 3);
  assert_eq!(original.test.table.bins.len(), 11);
  assert_eq!(
    original
      .test
      .table
      .bins
      .iter()
      .map(|bin| bin.observed_count)
      .sum::<usize>(),
    36
  );

  run(&mut session, "replace y = 0");
  run(&mut session, "replace x = 100");
  run(&mut session, "drop y x z person offset");
  let state_before = session.state();
  let rows_before = run(&mut session, "head 100");
  assert_eq!(report(&mut session), original);
  assert_eq!(session.state(), state_before);
  assert_eq!(run(&mut session, "head 100"), rows_before);
}

#[test]
fn countgof_rejects_missing_or_unsupported_fits_without_changing_state() {
  let mut empty = Session::new();
  let empty_state = empty.state();
  assert_eq!(
    empty
      .execute(parse_command("estat countgof").unwrap())
      .unwrap_err(),
    RuntimeError::EstatCountGofRequiresPriorNb2
  );
  assert_eq!(empty.state(), empty_state);
  assert!(empty.initialized_capabilities().is_empty());

  let fixture = Fixture::new();
  for fit in [
    "poisson y x z",
    "nbreg y x z, robust",
    "nbreg y x z, offset(offset)",
    "nbreg y x z, offset(offset) cluster(person)",
  ] {
    let mut session = fixture.session();
    run(&mut session, fit);
    let state_before = session.state();
    assert!(
      matches!(
        session.execute(parse_command("estat countgof").unwrap()),
        Err(RuntimeError::EstatCountGofRequiresPriorNb2)
          | Err(RuntimeError::EstatCountGofFailed { .. })
      ),
      "fit={fit}"
    );
    assert_eq!(session.state(), state_before, "fit={fit}");
  }
}

#[test]
fn countgof_uses_cluster_adjustment_and_preserves_session_state() {
  let fixture = Fixture::new();
  let mut session = fixture.session();
  run(&mut session, "nbreg y x z, cluster(person)");
  let state_before = session.state();
  let ExecutionResult::CountGoodnessOfFit(result) = run(&mut session, "estat countgof, max(2)")
  else {
    panic!("clustered count goodness of fit")
  };
  let cluster = result.test.cluster.as_ref().unwrap();
  assert_eq!(cluster.variable, "person");
  assert_eq!(cluster.group_count, 6);
  assert_eq!(cluster.covariance_correction, 1.2);
  assert_eq!(result.test.degrees_of_freedom, 2);
  assert_eq!(result.test.moment_covariance.as_ref().unwrap().len(), 2);
  assert_eq!(session.state(), state_before);
}
