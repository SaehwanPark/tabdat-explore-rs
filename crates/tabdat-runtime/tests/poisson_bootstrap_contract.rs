use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::Session;

const INPUT: &str =
  include_str!("../../../tests/fixtures/mma24_individual_pooled_bootstrap/input.csv");
const COMMAND: &str = "bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(commune) seed(10001) reps(500)";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(text: &str) -> Self {
    let p = std::env::temp_dir().join(format!(
      "tabdat-poisson-bootstrap-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    fs::write(p.join("data.csv"), text).unwrap();
    Self(p)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
    );
    s
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

#[test]
fn read_only_bootstrap_uses_complete_cases_and_preserves_native_poisson_state() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, cluster(commune)",
  );
  run(&mut s, "estimates store pooled");
  run(&mut s, "label variable lnhhexp \"original predictor\"");
  let before = s.state();
  let rows = run(&mut s, "head 5");
  let first = run(&mut s, COMMAND);
  let ExecutionResult::ClusterPoissonBootstrap(result) = &first else {
    panic!("Poisson cluster bootstrap")
  };
  assert_eq!(result.observation_count, 240);
  assert_eq!(result.sample.as_ref().unwrap().dropped_observations, 1);
  assert_eq!(result.cluster_variable.as_deref(), Some("commune"));
  assert_eq!(result.cluster_sizes.len(), 21);
  assert_eq!(result.cluster_sizes.last(), Some(&1));
  assert_eq!(result.replicate_coefficients.len(), 500);
  assert_eq!(first, run(&mut s, COMMAND));
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 5"), rows);
  assert!(
    s.execute(
      parse_command(
        "bootstrap pharvis lnhhexp lnhhexp, model(poisson) cluster(commune) seed(1) reps(2)"
      )
      .unwrap()
    )
    .is_err()
  );
  assert_eq!(s.state(), before);
  run(&mut s, "predict retained_after_bootstrap, n");
  assert_eq!(s.state().estimation, before.estimation);
  assert_eq!(fs::read_to_string(f.0.join("data.csv")).unwrap(), INPUT);
}

#[test]
fn invalid_counts_are_checked_before_float_rounding_after_joint_exclusions() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "poisson pharvis lnhhexp insurance sex age married illdays actdays injury illness educ",
  );
  run(
    &mut s,
    "sql SELECT * REPLACE (CASE WHEN source_row = 1 THEN CAST(9007199254740991.5 AS DECIMAL(38,1)) ELSE CAST(pharvis AS DECIMAL(38,1)) END AS pharvis) FROM active into bad_counts",
  );
  let before = s.state();
  let error = s
    .execute(parse_command(COMMAND).unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains("exact nonnegative integer"), "{error}");
  assert_eq!(s.state(), before);
  run(&mut s, "predict retained_after_failure, n");
  assert_eq!(s.state().estimation, before.estimation);
}

#[test]
fn invalid_count_on_incomplete_row_is_excluded_before_domain_checks() {
  let input = format!("{INPUT}242,1.5,,3,0,1,2,1,0,7,0,0,1\n");
  let f = Fixture::new(&input);
  let mut s = f.load();
  let ExecutionResult::ClusterPoissonBootstrap(result) = run(
    &mut s,
    "bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(commune) seed(10001) reps(2)",
  ) else {
    panic!("Poisson bootstrap")
  };
  assert_eq!(result.observation_count, 240);
  assert_eq!(result.sample.as_ref().unwrap().dropped_observations, 2);
}

#[test]
fn exact_wide_decimal_cluster_labels_remain_distinct() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT *, CASE commune WHEN 1 THEN CAST(9007199254740993.1 AS DECIMAL(38,1)) WHEN 2 THEN CAST(9007199254740993.2 AS DECIMAL(38,1)) ELSE CAST(commune AS DECIMAL(38,1)) END AS exact_group FROM active into exact_groups",
  );
  let ExecutionResult::ClusterPoissonBootstrap(result) = run(
    &mut s,
    "bootstrap pharvis lnhhexp insurance sex age married illdays actdays injury illness educ, model(poisson) cluster(exact_group) seed(10001) reps(2)",
  ) else {
    panic!("Poisson bootstrap")
  };
  assert_eq!(result.cluster_labels.len(), 21);
  assert!(
    result
      .cluster_labels
      .contains(&"90071992547409931e-1".to_string()),
    "{:?}",
    result.cluster_labels
  );
  assert!(
    result
      .cluster_labels
      .contains(&"90071992547409932e-1".to_string())
  );
}

#[test]
fn singular_resample_does_not_clear_original_model() {
  let csv = "y,x,g\n1,0,a\n2,0,a\n1,0,a\n2,0,a\n2,1,b\n3,1,b\n2,1,b\n3,1,b\n";
  let f = Fixture::new(csv);
  let mut s = f.load();
  run(&mut s, "poisson y x");
  let before = s.state();
  let error = s
    .execute(parse_command("bootstrap y x, model(poisson) cluster(g) seed(0) reps(2)").unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains("replicate 1 failed"), "{error}");
  assert_eq!(s.state(), before);
  run(&mut s, "predict after_failure, n");
  assert_eq!(s.state().estimation, before.estimation);
}
