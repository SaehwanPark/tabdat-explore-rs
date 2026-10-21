use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{Command, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(text: &str) -> Self {
    let p = std::env::temp_dir().join(format!(
      "tabdat-bootstrap-{}-{}",
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
      &format!("use {}", self.0.join("data.csv").display()),
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
fn complete_case_order_seeds_and_every_success_failure_preserve_the_existing_model() {
  let raw = include_str!("../../tabdat-stats/tests/fixtures/bootstrap/inputs.csv");
  let mut csv = String::from("y,x,z\n,0,0\n");
  csv.push_str(&raw.lines().skip(1).collect::<Vec<_>>().join("\n"));
  csv.push_str("\n0,,0\nInf,0,0\n");
  let f = Fixture::new(&csv);
  let mut s = f.load();
  run(&mut s, "regress y x z");
  run(&mut s, "estimates store original");
  run(&mut s, "label variable x \"original x\"");
  let before = s.state();
  let rows = run(&mut s, "head 100");
  let a = run(
    &mut s,
    "bootstrap y x z, seed(18446744073709551615) reps(40)",
  );
  let ExecutionResult::Bootstrap(r) = &a else {
    panic!("bootstrap")
  };
  assert_eq!(r.observation_count, 12);
  assert_eq!(
    r.sample.as_ref().unwrap().retained_indices,
    (1..=12).collect::<Vec<_>>()
  );
  assert_eq!(r.sample.as_ref().unwrap().dropped_observations, 3);
  assert_eq!(
    a,
    run(
      &mut s,
      "bootstrap y x z, seed(18446744073709551615) reps(40)"
    )
  );
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 100"), rows);
  for command in ["bootstrap absent x, seed(1)", "bootstrap y x x, seed(1)"] {
    assert!(s.execute(parse_command(command).unwrap()).is_err());
    assert_eq!(s.state(), before);
  }
  let Command::Bootstrap { mut command } = parse_command("bootstrap y x, seed(1)").unwrap() else {
    panic!("bootstrap")
  };
  command.regression.robust = true;
  assert!(s.execute(Command::Bootstrap { command }).is_err());
  assert_eq!(s.state(), before);
  run(&mut s, "replace y = y + 100");
  let changed = s.state();
  let ExecutionResult::Bootstrap(current) = run(
    &mut s,
    "bootstrap y x z, seed(18446744073709551615) reps(40)",
  ) else {
    panic!("bootstrap")
  };
  assert!((current.coefficients[0].estimate - r.coefficients[0].estimate - 100.0).abs() < 1e-9);
  assert_eq!(s.state(), changed);
  run(&mut s, "predict preserved");
}
#[test]
fn a_failed_resample_does_not_clear_the_prior_fit_or_publish_any_dataset_change() {
  let f = Fixture::new("y,x\n1,0\n4,0\n2,0\n8,1\n");
  let mut s = f.load();
  run(&mut s, "regress y x");
  let before = s.state();
  let error = s
    .execute(parse_command("bootstrap y x, seed(1) reps(100)").unwrap())
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  assert_eq!(s.state(), before);
  run(&mut s, "predict after_failure");
}

#[test]
fn whole_clusters_use_current_complete_cases_and_exact_decimal_keys_without_state_changes() {
  let raw = include_str!("../../tabdat-stats/tests/fixtures/cluster-bootstrap/inputs.csv");
  let mut csv = String::from("y,x,z,balanced,unbalanced\n,0,0,excluded,excluded\n");
  csv.push_str(&raw.lines().skip(1).collect::<Vec<_>>().join("\n"));
  csv.push_str("\n1,2,3,,\nInf,0,0,excluded,excluded\n");
  let f = Fixture::new(&csv);
  let mut s = f.load();
  run(&mut s, "regress y x z");
  run(&mut s, "estimates store original");
  run(&mut s, "label variable x \"preserved x\"");
  run(
    &mut s,
    "sql SELECT *, CAST(CASE unbalanced WHEN 'C' THEN '9007199254740993.1' WHEN 'A' THEN '9007199254740993.2' WHEN 'B' THEN '9007199254740993.3' WHEN 'D' THEN '9007199254740993.4' END AS DECIMAL(38,1)) AS exact_key FROM active into exact_keys",
  );
  let before = s.state();
  let rows = run(&mut s, "head 100");
  let fit = run(
    &mut s,
    "bootstrap y x z, cluster(exact_key) seed(7) reps(40)",
  );
  let ExecutionResult::ClusterBootstrap(r) = &fit else {
    panic!("cluster bootstrap")
  };
  assert_eq!(r.cluster_count, 4);
  assert_eq!(r.cluster_sizes, vec![3, 4, 6, 7]);
  let sample = r.bootstrap.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (1..=20).collect::<Vec<_>>());
  assert_eq!(sample.dropped_observations, 3);
  assert_eq!(
    sample.cluster_groups.as_ref().unwrap()[0],
    "9007199254740993.1"
  );
  assert_eq!(
    fit,
    run(
      &mut s,
      "bootstrap y x z, cluster(exact_key) seed(7) reps(40)"
    )
  );
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 100"), rows);
  for text in [
    "bootstrap y x x, cluster(exact_key) seed(7)",
    "bootstrap y x, cluster(absent) seed(7)",
    "bootstrap y x, cluster(exact_key) reps(5001) seed(7)",
  ] {
    if let Ok(command) = parse_command(text) {
      assert!(s.execute(command).is_err());
    }
    assert_eq!(s.state(), before);
  }
  run(
    &mut s,
    "sql SELECT *, DATE '2020-01-01' AS date_key FROM active into unsupported_keys",
  );
  let unsupported = s.state();
  let error = s
    .execute(parse_command("bootstrap y x z, cluster(date_key) seed(7)").unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains("numeric, text or boolean"), "{error}");
  assert_eq!(s.state(), unsupported);
  run(&mut s, "predict preserved_after_cluster");
}
#[test]
fn cluster_replicate_failure_is_atomic_and_nonfinite_keys_are_excluded() {
  let f = Fixture::new("y,x,g\n1,0,1\n4,0,1\n2,0,2\n8,1,2\n3,1,Inf\n");
  let mut s = f.load();
  run(&mut s, "regress y x");
  let before = s.state();
  let error = s
    .execute(parse_command("bootstrap y x, cluster(g) seed(1) reps(100)").unwrap())
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("replicate") && error.contains("no draws were discarded or retried"),
    "{error}"
  );
  assert_eq!(s.state(), before);
  run(&mut s, "predict preserved_after_failed_cluster");
}

#[test]
fn missing_nonfinite_and_signed_zero_float_groups_have_explicit_sample_semantics() {
  let f = Fixture::new("y,x,g\n1,0,0.0\n3,1,-0.0\n2,2,1.0\n5,3,1.0\n9,4,Inf\n8,5,NaN\n7,6,\n");
  let mut s = f.load();
  let before = s.state();
  let ExecutionResult::ClusterBootstrap(r) =
    run(&mut s, "bootstrap y x, cluster(g) seed(7) reps(40)")
  else {
    panic!("cluster bootstrap")
  };
  assert_eq!(r.cluster_count, 2);
  assert_eq!(r.cluster_sizes, vec![2, 2]);
  assert_eq!(
    r.bootstrap.sample.as_ref().unwrap().retained_indices,
    vec![0, 1, 2, 3]
  );
  assert_eq!(r.bootstrap.sample.as_ref().unwrap().dropped_observations, 3);
  assert_eq!(s.state(), before);
}
