use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(input: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-fe-bootstrap-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), input).unwrap();
    Self(path)
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
fn fixture() -> String {
  let mut csv = String::from("y,x,g\n");
  for g in 0..20 {
    csv.push_str(&format!("1,0,g{g}\n{},1,g{g}\n", 2 + g % 3));
  }
  csv.push_str("3,7,singleton\n0,0,zero\n0,1,zero\n,1,missing_response\n");
  csv
}
#[test]
fn report_samples_positive_groups_and_preserves_fe_state_and_data() {
  let input = fixture();
  let f = Fixture::new(&input);
  let mut s = f.load();
  run(&mut s, "poisson y x");
  run(&mut s, "estimates store pooled");
  run(&mut s, "label variable x \"predictor label\"");
  run(&mut s, "xtpoisson y x, fe id(g)");
  let before = s.state();
  let rows = run(&mut s, "head 5");
  let cmd = "bootstrap y x, model(conditional_poisson) cluster(g) seed(10001) reps(100)";
  let result = run(&mut s, cmd);
  let ExecutionResult::ConditionalPoissonBootstrap(r) = &result else {
    panic!("conditional bootstrap")
  };
  assert_eq!(
    (r.joint_finite_observation_count, r.observation_count),
    (43, 41)
  );
  assert_eq!(
    (
      r.candidate_group_count,
      r.zero_total_group_count,
      r.zero_total_observation_count
    ),
    (22, 1, 2)
  );
  assert_eq!(r.sample.total_observations, 44);
  assert_eq!(r.sample.dropped_observations, 3);
  assert_eq!(r.cluster_labels.len(), 21);
  assert_eq!(r.cluster_labels.last().unwrap(), "singleton");
  assert!(!r.cluster_labels.iter().any(|g| g == "zero"));
  assert_eq!(r.replicate_coefficients.len(), 100);
  assert_eq!(run(&mut s, cmd), result);
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 5"), rows);
  run(&mut s, "predict retained_mean, n");
  assert_eq!(s.state().estimation, before.estimation);
  assert_eq!(fs::read_to_string(f.0.join("data.csv")).unwrap(), input);
}
#[test]
fn exact_wide_decimal_group_keys_remain_distinct() {
  let f = Fixture::new(&fixture());
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT *, CASE WHEN g='g0' THEN CAST(9007199254740993.1 AS DECIMAL(38,1)) WHEN g='g1' THEN CAST(9007199254740993.2 AS DECIMAL(38,1)) WHEN g='singleton' THEN CAST(100 AS DECIMAL(38,1)) WHEN g='zero' THEN CAST(101 AS DECIMAL(38,1)) WHEN g='missing_response' THEN CAST(102 AS DECIMAL(38,1)) ELSE CAST(substr(g,2) AS DECIMAL(38,1)) END AS exact_group FROM active into exact_groups",
  );
  let ExecutionResult::ConditionalPoissonBootstrap(r) = run(
    &mut s,
    "bootstrap y x, model(conditional_poisson) cluster(exact_group) seed(10001) reps(2)",
  ) else {
    panic!("conditional bootstrap")
  };
  assert_eq!(r.cluster_labels.len(), 21);
  assert!(
    r.cluster_labels
      .contains(&"90071992547409931e-1".to_string())
  );
  assert!(
    r.cluster_labels
      .contains(&"90071992547409932e-1".to_string())
  );
}
#[test]
fn singular_resample_and_invalid_raw_keys_leave_original_fit_intact() {
  let f = Fixture::new("y,x,g\n1,0,a\n2,1,a\n3,7,b\n0,0,z\n");
  let mut s = f.load();
  run(&mut s, "xtpoisson y x, fe id(g)");
  let before = s.state();
  let err = s
    .execute(
      parse_command("bootstrap y x, model(conditional_poisson) cluster(g) seed(0) reps(2)")
        .unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(err.contains("replicate 1 failed"), "{err}");
  assert_eq!(s.state(), before);
  run(
    &mut s,
    "sql SELECT * FROM active UNION ALL SELECT NULL::DOUBLE, 1::DOUBLE, NULL::VARCHAR into bad_key",
  );
  let after_sql = s.state();
  let err = s
    .execute(
      parse_command("bootstrap y x, model(conditional_poisson) cluster(g) seed(0) reps(2)")
        .unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(
    err.contains("all raw group keys must be nonmissing"),
    "{err}"
  );
  assert_eq!(s.state(), after_sql);
  run(&mut s, "predict retained_after_failure, n");
  assert_eq!(s.state().estimation, before.estimation);
}
