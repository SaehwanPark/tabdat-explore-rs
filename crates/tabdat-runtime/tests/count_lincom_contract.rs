use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-count-lincom-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("count_lincom_data.csv"),
      include_str!("../../../examples/poisson-offset/poisson_offset.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("count_lincom_data.csv").display()),
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn execute(session: &mut Session, source: &str) -> ExecutionResult {
  session.execute(parse_command(source).unwrap()).unwrap()
}

fn fail(session: &mut Session, source: &str) -> RuntimeError {
  session.execute(parse_command(source).unwrap()).unwrap_err()
}

fn fit_command(family: &str, covariance: &str) -> String {
  let prefix = if family == "poisson" {
    "poisson"
  } else {
    "nbreg"
  };
  match covariance {
    "model" => format!("{prefix} y x z, offset(log_time)"),
    "robust" => format!("{prefix} y x z, offset(log_time) robust"),
    "cluster" => format!("{prefix} y x z, offset(log_time) cluster(person)"),
    _ => unreachable!(),
  }
}

fn expected(family: &str, covariance: &str, combination: &str, field: usize) -> f64 {
  include_str!("../../tabdat-stats/tests/fixtures/count_lincom_reference.csv")
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == family && fields[1] == covariance && fields[2] == combination)
        .then(|| fields[field].parse().unwrap())
    })
    .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
  assert!(
    (actual - expected).abs() <= 1e-8,
    "{actual:.16e} != {expected:.16e}"
  );
}

#[test]
fn poisson_and_nb2_lincom_use_selected_covariance_and_retained_fit() {
  for family in ["poisson", "nb2"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let mut session = fixture.load();
      execute(&mut session, &fit_command(family, covariance));
      let fitted = session.state();
      for (command, name) in [
        ("lincom x", "slope"),
        ("lincom 2 + x - z", "contrast"),
        ("lincom x + z - 0.25", "sum"),
      ] {
        let ExecutionResult::Lincom(result) = execute(&mut session, command) else {
          panic!("expected a count linear combination")
        };
        assert_eq!(result.df_residual, None);
        for (actual, field) in [
          (result.estimate, 3),
          (result.standard_error, 4),
          (result.statistic, 5),
          (result.p_value, 6),
          (result.ci_lower, 7),
          (result.ci_upper, 8),
          (result.ci_level, 9),
        ] {
          assert_close(actual, expected(family, covariance, name, field));
        }
        assert_eq!(session.state(), fitted);
      }

      let ExecutionResult::Lincom(constant) = execute(&mut session, "lincom 2") else {
        panic!("expected a constant linear combination")
      };
      assert_eq!(constant.estimate, 2.0);
      assert_eq!(constant.standard_error, 0.0);
      assert!(constant.statistic.is_nan());
      assert!(constant.p_value.is_nan());
      assert_eq!((constant.ci_lower, constant.ci_upper), (2.0, 2.0));
      assert_eq!(constant.df_residual, None);

      execute(&mut session, "replace x = x + 100");
      let edited = session.state();
      let ExecutionResult::Lincom(after_edit) = execute(&mut session, "lincom x") else {
        panic!("expected retained-fit lincom after data edits")
      };
      assert_close(
        after_edit.estimate,
        expected(family, covariance, "slope", 3),
      );
      assert_eq!(session.state(), edited);
    }
  }
}

#[test]
fn count_lincom_rejects_unknown_names_and_nonfinite_results_atomically() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "nbreg y x z, offset(log_time) robust");
  let before = session.state();
  for command in ["lincom lnalpha", "lincom missing"] {
    assert!(matches!(
      fail(&mut session, command),
      RuntimeError::LincomFailed { .. }
    ));
    assert_eq!(session.state(), before, "{command}");
  }

  let large_multiplier = format!("1{}", "0".repeat(309));
  let command = format!("lincom {large_multiplier} * x");
  assert!(matches!(
    fail(&mut session, &command),
    RuntimeError::LincomFailed { .. }
  ));
  assert_eq!(session.state(), before, "{command}");
}
