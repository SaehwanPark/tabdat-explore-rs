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
    let mut csv = "id,y,x,z,person\n".to_owned();
    if missing {
      csv.push_str("0,,0,0,0\n");
    }
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{},{}\n",
        i + 1,
        y,
        -2. + 4. * i as f64 / 35.,
        [0., 1., 0.4][i % 3],
        i % 6
      ));
    }
    if missing {
      csv.push_str("37,0.5,,0,1\n38,100001,0,0,\n");
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
