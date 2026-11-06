use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::ExecutionResult;
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const INPUT: &str = include_str!("../../../tests/fixtures/mma24_individual_gamma_re/input.csv");
const PRED: &str = "lnhhexp insurance sex age married illdays actdays injury illness educ";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(input: &str) -> Self {
    let p = std::env::temp_dir().join(format!(
      "tabdat-gamma-bootstrap-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&p).unwrap();
    fs::write(p.join("data.csv"), input).unwrap();
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
fn resampling_retains_zero_groups_and_original_gamma_fit_data_and_named_report() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT * FROM active WHERE commune IS NOT NULL into cohort",
  );
  run(&mut s, &format!("poisson pharvis {PRED}"));
  run(&mut s, "estimates store pooled");
  run(&mut s, "label variable lnhhexp \"label\"");
  run(&mut s, &format!("xtpoisson pharvis {PRED}, re id(commune)"));
  let before = s.state();
  let rows = run(&mut s, "head 5");
  let cmd = format!(
    "bootstrap pharvis {PRED}, model(gamma_poisson_re) cluster(commune) seed(10001) reps(10)"
  );
  let first = run(&mut s, &cmd);
  let ExecutionResult::GammaPoissonBootstrap(r) = &first else {
    panic!("shared gamma bootstrap")
  };
  assert_eq!(r.observation_count, 278);
  assert_eq!(r.cluster_labels.len(), 48);
  assert_eq!(r.zero_total_group_count, 4);
  assert_eq!(r.singleton_group_count, 2);
  assert_eq!(r.sample.total_observations, 278);
  assert_eq!(r.sample.dropped_observations, 0);
  assert_eq!(r.covariance_parameter_names[0], "_cons");
  assert_eq!(r.covariance_parameter_names[11], "lnalpha");
  assert_eq!(run(&mut s, &cmd), first);
  assert_eq!(s.state(), before);
  assert_eq!(run(&mut s, "head 5"), rows);
  run(&mut s, "predict retained_mean, n");
  assert_eq!(s.state().estimation, before.estimation);
  assert_eq!(fs::read_to_string(f.0.join("data.csv")).unwrap(), INPUT);
}
#[test]
fn failing_singular_copy_and_missing_raw_key_preserve_original_fit() {
  let f = Fixture::new("y,x,g\n0,0,a\n0,0,a\n10,1,b\n12,1,b\n1,0,c\n2,0,c\n8,0,d\n10,0,d\n");
  let mut s = f.load();
  run(&mut s, "xtpoisson y x, re id(g)");
  let before = s.state();
  let err = s
    .execute(
      parse_command("bootstrap y x, model(gamma_poisson_re) cluster(g) seed(0) reps(2)").unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(err.contains("replicate 1 failed"), "{err}");
  assert_eq!(s.state(), before);
  run(
    &mut s,
    "sql SELECT * FROM active UNION ALL SELECT NULL::DOUBLE, 1::DOUBLE, NULL::VARCHAR into bad_key",
  );
  let current = s.state();
  let err = s
    .execute(
      parse_command("bootstrap y x, model(gamma_poisson_re) cluster(g) seed(0) reps(2)").unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(
    err.contains("all raw group keys must be nonmissing"),
    "{err}"
  );
  assert_eq!(s.state(), current);
  run(&mut s, "predict retained_after_failure, n");
  assert_eq!(s.state().estimation, before.estimation);
}
#[test]
fn dispersion_boundary_copy_does_not_replace_a_retained_model() {
  let f = Fixture::new("y,x,g\n0,0,a\n0,1,a\n10,0,b\n12,1,b\n1,0,c\n2,1,c\n8,0,d\n10,1,d\n");
  let mut s = f.load();
  run(&mut s, "xtpoisson y x, re id(g)");
  let before = s.state();
  let err = s
    .execute(
      parse_command("bootstrap y x, model(gamma_poisson_re) cluster(g) seed(95) reps(2)").unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(
    err.contains("replicate 1 failed") && (err.contains("boundary") || err.contains("ln(alpha)")),
    "{err}"
  );
  assert_eq!(s.state(), before);
  run(&mut s, "predict retained_after_boundary, n");
  assert_eq!(s.state().estimation, before.estimation);
}
#[test]
fn wide_decimal_group_keys_remain_exact() {
  let f = Fixture::new(INPUT);
  let mut s = f.load();
  run(
    &mut s,
    "sql SELECT * FROM active WHERE commune IS NOT NULL into cohort",
  );
  run(
    &mut s,
    "sql SELECT *, CASE commune WHEN 1 THEN CAST(9007199254740993.1 AS DECIMAL(38,1)) WHEN 2 THEN CAST(9007199254740993.2 AS DECIMAL(38,1)) ELSE CAST(commune AS DECIMAL(38,1)) END AS exact_group FROM active into exact_groups",
  );
  let ExecutionResult::GammaPoissonBootstrap(r) = run(
    &mut s,
    &format!(
      "bootstrap pharvis {PRED}, model(gamma_poisson_re) cluster(exact_group) seed(10001) reps(2)"
    ),
  ) else {
    panic!("shared gamma bootstrap")
  };
  assert_eq!(r.cluster_labels.len(), 48);
  assert!(
    r.cluster_labels
      .contains(&"90071992547409931e-1".to_string())
  );
  assert!(
    r.cluster_labels
      .contains(&"90071992547409932e-1".to_string())
  );
}
