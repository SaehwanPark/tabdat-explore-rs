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
      "tabdat-binary-wald-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("binary_wald_data.csv"),
      include_str!("../../tabdat-stats/tests/fixtures/binary_wald_data.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("binary_wald_data.csv").display()),
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
  match covariance {
    "model" => format!("{family} y x1 x2"),
    "robust" => format!("{family} y x1 x2, robust"),
    "cluster" => format!("{family} y x1 x2, cluster(group)"),
    _ => unreachable!(),
  }
}

fn expected(family: &str, covariance: &str, restriction: &str, field: usize) -> f64 {
  include_str!("../../tabdat-stats/tests/fixtures/binary_wald_reference.csv")
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
fn logit_and_probit_tests_use_the_selected_covariance_and_retained_fit() {
  for family in ["logit", "probit"] {
    for covariance in ["model", "robust", "cluster"] {
      let fixture = Fixture::new();
      let mut session = fixture.load();
      execute(&mut session, &fit_command(family, covariance));
      let fitted = session.state();
      let ExecutionResult::Test(one) = execute(&mut session, "test x1 + x2 = 0.2") else {
        panic!("expected a binary chi-square test")
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

      let ExecutionResult::Test(joint) =
        execute(&mut session, "test (x1 + x2 = 0.2) (x1 - x2 = 0.4)")
      else {
        panic!("expected a joint binary chi-square test")
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

      execute(&mut session, "replace x1 = x1 + 100");
      let edited = session.state();
      let ExecutionResult::Test(after_edit) = execute(&mut session, "test x1 + x2 = 0.2") else {
        panic!("expected a retained-fit test after editing data")
      };
      assert_eq!(after_edit, one);
      assert_eq!(session.state(), edited);
    }
  }
}

#[test]
fn invalid_binary_restrictions_leave_fit_and_data_unchanged() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "logit y x1 x2, robust");
  let before = session.state();

  let unknown = fail(&mut session, "test missing = 0");
  assert!(matches!(unknown, RuntimeError::TestFailed { .. }));
  assert!(
    unknown
      .to_string()
      .contains("variable 'missing' not found in active model coefficients")
  );
  assert_eq!(session.state(), before);

  let singular = fail(&mut session, "test (x1 = 0) (x1 = 1)");
  assert!(matches!(singular, RuntimeError::TestFailed { .. }));
  assert!(
    singular
      .to_string()
      .contains("constraints are collinear or singular")
  );
  assert_eq!(session.state(), before);
}
