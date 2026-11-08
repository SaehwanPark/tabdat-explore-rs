use std::path::PathBuf;
use tabdat_domain::{ExecutionResult, RuntimeError};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const BOOT: &str = "bootstrap y x z, model(normal_poisson_re) cluster(id) seed(10001) reps(2)";
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fixture() -> Session {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../tabdat-stats/tests/fixtures/normal_poisson/multivariate/input.csv");
  let mut s = Session::new();
  run(&mut s, &format!("use \"{}\"", path.display()));
  run(&mut s, "poisson y x z");
  run(&mut s, "estimates store pooled");
  run(&mut s, "label variable x \"Retained source label\"");
  run(&mut s, "xtpoisson y x z, re distribution(normal) id(id)");
  s
}
fn rejected(s: &mut Session, text: &str) -> String {
  let before = s.state();
  let rows = run(s, "head 10");
  let error = s.execute(parse_command(text).unwrap()).unwrap_err();
  assert!(matches!(error, RuntimeError::BootstrapFailed(_)), "{error}");
  assert_eq!(s.state(), before);
  assert_eq!(run(s, "head 10"), rows);
  error.to_string()
}
#[test]
fn joint_bootstrap_keeps_owned_fit_data_labels_reports_and_predictions() {
  let mut s = fixture();
  run(&mut s, "predict before_bs, n");
  let before = s.state();
  let rows = run(&mut s, "head 10");
  let first = run(&mut s, BOOT);
  let ExecutionResult::NormalPoissonBootstrap(r) = &first else {
    panic!("wrong model")
  };
  assert_eq!(r.observation_count, 120);
  assert_eq!(r.cluster_labels.len(), 40);
  assert_eq!(r.singleton_group_count, 8);
  assert_eq!(
    (r.sample.total_observations, r.sample.dropped_observations),
    (120, 0)
  );
  assert_eq!(r.replicate_observation_counts, [127, 121]);
  assert_eq!(
    r.covariance_parameter_names,
    ["_cons", "x", "z", "lnsigma2"]
  );
  assert_eq!(r.original_quadrature_nodes, [80, 96]);
  assert_eq!(r.refit_quadrature_nodes, [176, 208]);
  assert_eq!(run(&mut s, BOOT), first);
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 10"), rows);
  run(&mut s, "predict after_bs, n");
  run(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(*)=120 AND MAX(abs(before_bs-after_bs))=0 THEN 1 ELSE error('bootstrap changed owned fit') END AS valid FROM active\"\"\"",
  );
  assert_eq!(s.state().estimation, before.estimation);
  assert_eq!(s.state().labels, before.labels);
}
#[test]
fn rank_losing_copy_and_parser_errors_preserve_stale_owned_model_and_reports() {
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT *, CASE WHEN id=2 THEN 1.0 ELSE 0.0 END AS rare FROM active into rare_data",
  );
  let error = rejected(
    &mut s,
    "bootstrap y x z rare, model(normal_poisson_re) cluster(id) seed(0) reps(2)",
  );
  assert!(error.contains("replicate 1 failed"), "{error}");
  for text in [
    "bootstrap y x, model(normal_poisson_re) cluster(id) seed(1) robust",
    "bootstrap y x, model(normal_poisson_re) cluster(id) seed(1) reps(501)",
  ] {
    let before = s.state();
    assert!(parse_command(text).is_err());
    assert_eq!(s.state(), before);
  }
  run(&mut s, "predict retained, n");
}
#[test]
fn omitted_raw_missing_key_and_wide_count_fail_atomically() {
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT * FROM active UNION ALL SELECT 201,NULL::BIGINT,1975,NULL::DOUBLE,0::DOUBLE,0::DOUBLE into missing_key",
  );
  let error = rejected(&mut s, BOOT);
  assert!(
    error.contains("all raw group keys must be nonmissing"),
    "{error}"
  );
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT source_row,id,year,CASE WHEN source_row=0 THEN CAST(9007199254740993 AS DECIMAL(38,0)) ELSE CAST(y AS DECIMAL(38,0)) END AS y,x,z FROM active into wide_count",
  );
  rejected(&mut s, BOOT);
  run(&mut s, "predict retained, n");
}
#[test]
fn decimal_group_keys_remain_distinct_without_float_roundtrip() {
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT *, CASE id WHEN 1 THEN CAST(9007199254740993.1 AS DECIMAL(38,1)) WHEN 2 THEN CAST(9007199254740993.2 AS DECIMAL(38,1)) ELSE CAST(id AS DECIMAL(38,1)) END AS exact_group FROM active into exact_groups",
  );
  let before = s.state();
  let ExecutionResult::NormalPoissonBootstrap(r) = run(
    &mut s,
    "bootstrap y x z, model(normal_poisson_re) cluster(exact_group) seed(10001) reps(2)",
  ) else {
    panic!()
  };
  assert_eq!(s.state(), before);
  assert_eq!(r.cluster_labels.len(), 40);
  assert!(
    r.cluster_labels
      .contains(&"90071992547409931e-1".to_string())
  );
  assert!(
    r.cluster_labels
      .contains(&"90071992547409932e-1".to_string())
  );
}
