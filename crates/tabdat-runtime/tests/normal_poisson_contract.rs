use std::path::PathBuf;
use tabdat_domain::{EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const FIT: &str = "xtpoisson y x z, re distribution(normal) id(id)";
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fixture() -> Session {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../tabdat-stats/tests/fixtures/normal_poisson/multivariate/input.csv");
  let mut s = Session::new();
  run(&mut s, &format!("use \"{}\"", path.display()));
  s
}
fn fit(s: &mut Session) -> tabdat_stats::NormalPoissonResult {
  let ExecutionResult::NormalPoisson(r) = run(s, FIT) else {
    panic!("not owned normal RE")
  };
  *r
}
fn rejected(s: &mut Session, text: &str) {
  let before = s.state();
  if let Ok(command) = parse_command(text) {
    assert!(s.execute(command).is_err(), "{text}");
  }
  assert_eq!(s.state(), before, "{text}");
}
#[test]
fn owned_fit_population_prediction_and_current_data_without_keys_or_outcome() {
  let mut s = fixture();
  let r = fit(&mut s);
  assert_eq!(
    (r.observation_count, r.group_count, r.singleton_group_count),
    (120, 40, 8)
  );
  assert_eq!(
    (r.sample.total_observations, r.sample.dropped_observations),
    (120, 0)
  );
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::NormalPoisson
  );
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::NormalPoisson(_))
  ));
  let reference =
    include_str!("../../tabdat-stats/tests/fixtures/normal_poisson/multivariate/predictions.csv");
  for (mean, line) in r.fitted_means.iter().zip(reference.lines().skip(1)) {
    let value: f64 = line.split(',').nth(6).unwrap().parse().unwrap();
    assert!((mean - value).abs() <= 1e-9 + 1e-10 * value.abs());
  }
  run(&mut s, "label variable x \"Keep normal source label\"");
  let before = s.state();
  for command in [
    "predict mu, n",
    "predict eta, xb",
    "predict resid, residuals",
  ] {
    run(&mut s, command);
    assert_eq!(s.state().estimation, before.estimation);
    assert_eq!(s.state().labels, before.labels);
  }
  let shift = r.ln_variance.value.exp() / 2.;
  run(
    &mut s,
    &format!(
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=120 AND MAX(abs(mu-exp(eta+{shift:.17})))<1e-10 AND MAX(abs(resid-y+mu))<1e-10 THEN 1 ELSE error('lost normal population identity') END AS valid FROM active\"\"\""
    ),
  );
  run(
    &mut s,
    "sql SELECT x,z FROM active ORDER BY source_row DESC into forecast",
  );
  let before = s.state();
  run(&mut s, "predict forecast, n");
  run(&mut s, "predict forecast_xb, xb");
  assert_eq!(s.state().estimation, before.estimation);
  rejected(&mut s, "predict no_y, residuals");
  run(
    &mut s,
    &format!(
      "sql \"\"\"SELECT CASE WHEN COUNT(*)=120 AND MAX(abs(forecast-exp(forecast_xb+{shift:.17})))<1e-10 THEN 1 ELSE error('forecast needs no keys/y') END AS valid FROM active\"\"\""
    ),
  );
}
#[test]
fn unsupported_prediction_report_collision_and_overflow_preserve_state_and_data() {
  let mut s = fixture();
  fit(&mut s);
  for text in [
    "predict wrong, pr",
    "predict wrong, n std",
    "predict wrong, n interval",
    "predict y, n",
    "estimates store unsupported",
    "estimates store unsupported, covariance(working)",
  ] {
    rejected(&mut s, text);
  }
  run(&mut s, "predict mu, n");
  rejected(&mut s, "predict mu, n");
  run(
    &mut s,
    "sql SELECT 1e308::DOUBLE AS x, z FROM active into overflow",
  );
  let before = run(&mut s, "head 5");
  rejected(&mut s, "predict overflowing, n");
  assert_eq!(run(&mut s, "head 5"), before);
  run(
    &mut s,
    "sql SELECT NULL::DOUBLE AS x,z FROM active into incomplete",
  );
  run(&mut s, "predict nullable, n");
  run(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(*)=120 AND COUNT(nullable)=0 THEN 1 ELSE error('missing x must be NULL') END AS valid FROM active\"\"\"",
  );
}
#[test]
fn invalid_fit_clears_stale_model_preserving_reports_labels_and_data_even_on_omitted_raw_key() {
  let mut s = fixture();
  run(&mut s, "poisson y x z");
  run(&mut s, "estimates store pooled");
  run(&mut s, "label variable x \"Keep label\"");
  fit(&mut s);
  let mut expected = s.state();
  expected.estimation = None;
  let rows = run(&mut s, "head 5");
  assert!(
    s.execute(parse_command("xtpoisson y x x, re distribution(normal) id(id)").unwrap())
      .is_err()
  );
  assert_eq!(s.state(), expected);
  assert_eq!(run(&mut s, "head 5"), rows);
  run(&mut s, "estimates table pooled");
  fit(&mut s);
  run(
    &mut s,
    "sql SELECT source_row,id,year,CASE WHEN source_row=0 THEN NULL ELSE y END AS y,x,z FROM active into partial",
  );
  let r = fit(&mut s);
  assert_eq!(
    (
      r.observation_count,
      r.sample.total_observations,
      r.sample.dropped_observations
    ),
    (119, 120, 1)
  );
  run(
    &mut s,
    "sql SELECT * FROM active UNION ALL SELECT NULL,NULL,NULL,NULL,NULL,NULL into missing_raw_key",
  );
  let mut expected = s.state();
  expected.estimation = None;
  let rows = run(&mut s, "head 5");
  let error = s
    .execute(parse_command(FIT).unwrap())
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("all raw group keys must be nonmissing"),
    "{error}"
  );
  assert_eq!(s.state(), expected);
  assert_eq!(run(&mut s, "head 5"), rows);
  run(&mut s, "estimates table pooled");
}
