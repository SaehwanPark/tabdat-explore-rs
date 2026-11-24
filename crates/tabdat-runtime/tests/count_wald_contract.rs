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
      "tabdat-count-wald-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("count_wald_data.csv"),
      include_str!("../../../examples/poisson-offset/poisson_offset.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("count_wald_data.csv").display()),
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

fn expected(family: &str, covariance: &str, restriction: &str, field: usize) -> f64 {
  include_str!("../../tabdat-stats/tests/fixtures/count_wald_reference.csv")
    .lines()
    .skip(1)
    .find_map(|line| {
      let fields = line.split(',').collect::<Vec<_>>();
      (fields[0] == family && fields[1] == covariance && fields[2] == restriction)
        .then(|| fields[field].parse().unwrap())
    })
    .unwrap()
}

fn assert_close(actual: &serde_json::Value, expected: f64) {
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() <= 1e-8,
    "{actual:.16e} != {expected:.16e}"
  );
}

#[test]
fn poisson_and_nb2_use_selected_covariance_and_retained_fit() {
  for family in ["poisson", "nb2"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let mut session = fixture.load();
      execute(&mut session, &fit_command(family, covariance));
      let fitted = session.state();
      let ExecutionResult::Test(one) = execute(&mut session, "test x = 0") else {
        panic!("expected a count Wald test")
      };
      assert_eq!(one.df, 1);
      assert_eq!(one.df_residual, None);
      assert!(one.is_chi2);
      assert_close(
        &serde_json::json!(one.statistic),
        expected(family, covariance, "one", 4),
      );
      assert_close(
        &serde_json::json!(one.p_value),
        expected(family, covariance, "one", 5),
      );
      assert_eq!(session.state(), fitted);

      let ExecutionResult::Test(contrast) = execute(&mut session, "test x = z + 0.25") else {
        panic!("expected a count contrast test")
      };
      assert_eq!(contrast.df, 1);
      assert_eq!(contrast.df_residual, None);
      assert!(contrast.is_chi2);
      assert_close(
        &serde_json::json!(contrast.statistic),
        expected(family, covariance, "contrast", 4),
      );
      assert_close(
        &serde_json::json!(contrast.p_value),
        expected(family, covariance, "contrast", 5),
      );
      assert_eq!(session.state(), fitted);

      let ExecutionResult::Test(joint) = execute(&mut session, "test x z") else {
        panic!("expected a joint count Wald test")
      };
      assert_eq!(joint.df, 2);
      assert_eq!(joint.df_residual, None);
      assert!(joint.is_chi2);
      assert_close(
        &serde_json::json!(joint.statistic),
        expected(family, covariance, "joint", 4),
      );
      assert_close(
        &serde_json::json!(joint.p_value),
        expected(family, covariance, "joint", 5),
      );
      assert_eq!(session.state(), fitted);

      execute(&mut session, "replace x = x + 100");
      let edited = session.state();
      let ExecutionResult::Test(after_edit) = execute(&mut session, "test x = 0") else {
        panic!("expected a retained-fit test after data edits")
      };
      assert_eq!(after_edit, one);
      assert_eq!(session.state(), edited);
    }
  }
}

#[test]
fn count_test_rejects_dispersion_unknown_and_singular_restrictions_atomically() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "nbreg y x z, offset(log_time) robust");
  let before = session.state();

  for command in ["test lnalpha = 0", "test missing = 0"] {
    let error = fail(&mut session, command);
    assert!(matches!(error, RuntimeError::TestFailed { .. }));
    assert_eq!(session.state(), before);
  }
  let singular = fail(&mut session, "test (x = 0) (x = 1)");
  assert!(
    singular
      .to_string()
      .contains("constraints are collinear or singular")
  );
  assert_eq!(session.state(), before);
}
