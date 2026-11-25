use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{EstimationState, ExecutionResult, RuntimeError};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
use tabdat_stats::BinaryModel;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-binary-gof-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      include_str!("../../../examples/binary-hosmer-lemeshow/binary_gof_data.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
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

fn fail(session: &mut Session, command: &str) -> RuntimeError {
  session
    .execute(parse_command(command).unwrap())
    .unwrap_err()
}

fn close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 2e-10,
    "{actual:.17} != {expected:.17}"
  );
}

fn close_expected_count(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-9,
    "{actual:.17} != {expected:.17}"
  );
}

fn reference_rows(model: &str) -> Vec<Vec<f64>> {
  include_str!("../../../examples/binary-hosmer-lemeshow/binary_gof_reference.csv")
    .lines()
    .skip(1)
    .filter_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == model).then(|| {
        fields[1..]
          .iter()
          .map(|value| value.parse::<f64>().unwrap())
          .collect::<Vec<_>>()
      })
    })
    .collect()
}

#[test]
fn logit_and_probit_grouped_tests_match_base_r_and_retain_the_original_sample() {
  let fixture = Fixture::new();
  for (model, fit) in [
    (BinaryModel::Logit, "logit y x1 x2"),
    (BinaryModel::Probit, "probit y x1 x2"),
  ] {
    let mut session = fixture.load();
    run(&mut session, fit);
    let retained_state = session.state().estimation;
    let ExecutionResult::BinaryGoodnessOfFit(result) = run(&mut session, "estat gof, group(10)")
    else {
      panic!("expected grouped binary goodness of fit")
    };
    assert_eq!(result.model, model);
    assert_eq!(result.outcome, "y");
    let sample = result.sample.as_ref().unwrap();
    assert_eq!(sample.retained_indices, (1..160).collect::<Vec<_>>());
    assert_eq!(sample.total_observations, 160);
    assert_eq!(sample.dropped_observations, 1);
    assert_eq!(result.test.requested_group_count, 10);
    assert_eq!(result.test.group_count, 4);
    assert_eq!(result.test.degrees_of_freedom, 2);
    assert_eq!(result.test.groups.len(), 4);
    assert_eq!(session.state().estimation, retained_state);

    let reference = reference_rows(if model == BinaryModel::Logit {
      "logit"
    } else {
      "probit"
    });
    assert_eq!(reference.len(), 4);
    close(result.test.statistic, reference[0][3]);
    close(result.test.p_value, reference[0][4]);
    for (group, expected) in result.test.groups.iter().zip(reference) {
      assert_eq!(group.group, expected[5] as usize);
      close(group.probability_upper_bound, expected[6]);
      assert_eq!(group.observation_count, expected[7] as usize);
      assert_eq!(group.observed_events, expected[8] as usize);
      close_expected_count(group.expected_events, expected[9]);
      assert_eq!(group.observed_nonevents, expected[10] as usize);
      close_expected_count(group.expected_nonevents, expected[11]);
    }

    // The diagnostic uses retained outcomes/probabilities even after those
    // columns are changed or removed from the active DuckDB relation.
    run(&mut session, "replace y = 1 - y");
    run(&mut session, "replace x1 = 0");
    run(&mut session, "drop y x1 x2 cluster");
    assert_eq!(
      run(&mut session, "estat gof, group(10)"),
      ExecutionResult::BinaryGoodnessOfFit(result.clone())
    );
    assert_eq!(session.state().estimation, retained_state);
  }
}

#[test]
fn grouped_gof_rejects_missing_robust_clustered_and_insufficient_tie_fits_atomically() {
  let mut empty = Session::new();
  let empty_state = empty.state();
  assert_eq!(
    fail(&mut empty, "estat gof, group(10)"),
    RuntimeError::EstatGofRequiresPriorBinary
  );
  assert_eq!(empty.state(), empty_state);

  let fixture = Fixture::new();
  for fit in ["logit y x1 x2, robust", "probit y x1 x2, cluster(cluster)"] {
    let mut session = fixture.load();
    run(&mut session, fit);
    let state_before = session.state();
    assert_eq!(
      fail(&mut session, "estat gof, group(10)"),
      RuntimeError::EstatGofRequiresModelBasedBinary,
      "fit={fit}"
    );
    assert_eq!(session.state(), state_before, "fit={fit}");
  }

  let mut session = fixture.load();
  run(&mut session, "logit y x1");
  let state_before = session.state();
  assert!(matches!(
    fail(&mut session, "estat gof, group(10)"),
    RuntimeError::EstatGofFailed { .. }
  ));
  assert_eq!(session.state(), state_before);
}

#[test]
fn bare_gof_remains_outside_this_binary_grouped_slice() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "logit y x1 x2");
  let state_before = session.state();
  assert!(matches!(
    fail(&mut session, "estat gof"),
    RuntimeError::UnsupportedCommand { .. }
  ));
  assert_eq!(session.state(), state_before);
}

#[test]
fn saturated_probability_uses_the_retained_stable_variance_and_checks_design_identity() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "logit y x1 x2");
  let Some(EstimationState::Logit(state)) = session.state().estimation else {
    panic!("expected retained logit state")
  };
  let mut fit = state.result.clone();
  for observation in 119..159 {
    fit.fitted_probabilities[observation] = 1.0;
    fit.fitted_weights[observation] = 1e-16;
  }
  let test = tabdat_stats::binary_hosmer_lemeshow_test(&state.problem, &fit, 10).unwrap();
  assert_eq!(test.group_count, 4);
  assert_eq!(test.groups.last().unwrap().probability_upper_bound, 1.0);
  assert!(test.groups.last().unwrap().expected_nonevents > 0.0);
  assert!(test.statistic.is_finite());
  assert!(test.p_value.is_finite());

  let mut unrecoverable_tail = fit.clone();
  for observation in 119..159 {
    unrecoverable_tail.fitted_weights[observation] = 0.0;
  }
  assert!(
    tabdat_stats::binary_hosmer_lemeshow_test(&state.problem, &unrecoverable_tail, 10).is_err()
  );

  let mut mismatched_design = state.problem.clone();
  mismatched_design.predictor_names[0] = "different_predictor".into();
  assert!(tabdat_stats::binary_hosmer_lemeshow_test(&mismatched_design, &fit, 10).is_err());
}
