use std::path::PathBuf;
use tabdat_domain::{EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const FIT: &str = "xtgee y x1 x2, family(poisson) link(log) corr(exchangeable) id(id)";
fn run(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fixture() -> Session {
  let mut s = Session::new();
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../tabdat-stats/tests/fixtures/poisson_gee/multivariate/input.csv");
  run(&mut s, &format!("use \"{}\"", path.display()));
  s
}
fn fit(s: &mut Session) -> tabdat_stats::PoissonGeeResult {
  let ExecutionResult::PoissonGee(r) = run(s, FIT) else {
    panic!("not Poisson GEE")
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
fn native_fit_is_owned_and_predictions_agree_with_frozen_r() {
  let mut s = fixture();
  let r = fit(&mut s);
  assert_eq!(r.observation_count, 60);
  assert_eq!(r.group_count, 12);
  assert_eq!(r.zero_total_group_count, 2);
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::PoissonGee
  );
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::PoissonGee(_))
  ));
  let reference = std::fs::read_to_string(
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join("../tabdat-stats/tests/fixtures/poisson_gee/multivariate/predictions.csv"),
  )
  .unwrap();
  for (mean, line) in r.fitted_means.iter().zip(reference.lines().skip(1)) {
    let expected: f64 = line.split(',').nth(3).unwrap().parse().unwrap();
    assert!((mean - expected).abs() <= 1e-9 + 1e-10 * expected.abs());
  }
  run(&mut s, "label variable x1 \"Owned source predictor\"");
  let before = s.state();
  for command in [
    "predict mu, n",
    "predict repeated, n",
    "predict eta, xb",
    "predict resid, residuals",
  ] {
    run(&mut s, command);
    assert_eq!(s.state().estimation, before.estimation);
    assert_eq!(s.state().labels, before.labels);
  }
  run(
    &mut s,
    "sql \"\"\"SELECT CASE WHEN COUNT(*)=60 AND MAX(abs(mu-repeated))<1e-12 AND MAX(abs(mu-exp(eta)))<1e-12 AND MAX(abs(resid-y+mu))<1e-12 THEN 1 ELSE error('lost GEE prediction identity') END AS valid FROM active\"\"\"",
  );
}
#[test]
fn parser_prediction_and_unsupported_store_errors_preserve_owned_state() {
  let mut s = fixture();
  fit(&mut s);
  let before = s.state();
  assert!(
    parse_command("xtgee y x1, family(poisson) link(identity) corr(exchangeable) id(id)").is_err()
  );
  assert_eq!(s.state(), before);
  for command in [
    "predict wrong, pr",
    "predict wrong, n std",
    "predict wrong, n interval",
    "predict y, n",
    "estimates store unsupported",
    "estimates store unsupported, covariance(working)",
  ] {
    rejected(&mut s, command);
  }
  run(&mut s, "predict mu, n");
  rejected(&mut s, "predict mu, n");
}
#[test]
fn failed_new_fit_clears_stale_estimation_but_preserves_data_labels_and_reports() {
  let mut s = fixture();
  run(&mut s, "poisson y x1 x2");
  run(&mut s, "estimates store starter");
  run(&mut s, "label variable x1 \"Keep label\"");
  fit(&mut s);
  let mut expected = s.state();
  expected.estimation = None;
  assert!(
    s.execute(
      parse_command("xtgee y x1 x1, family(poisson) link(log) corr(exchangeable) id(id)").unwrap()
    )
    .is_err()
  );
  assert_eq!(s.state(), expected);
  assert!(
    s.execute(parse_command("predict denied, n").unwrap())
      .is_err()
  );
  run(&mut s, "estimates table starter");
}
#[test]
fn selected_counts_and_balanced_joint_finite_membership_are_explicit() {
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT id,period,CASE WHEN period=1 THEN NULL ELSE x1 END AS x1,x2,CASE WHEN period=1 THEN 0.5 ELSE y END AS y FROM active ORDER BY id,period into partial_cohort",
  );
  let r = fit(&mut s);
  assert_eq!(r.observation_count, 48);
  assert_eq!(r.group_count, 12);
  assert_eq!(r.group_size, 4);
  assert_eq!(r.sample.total_observations, 60);
  assert_eq!(r.sample.dropped_observations, 12);
  assert_eq!(
    r.sample.retained_indices,
    (0..60).filter(|i| i % 5 != 0).collect::<Vec<_>>()
  );
  assert_eq!(r.zero_total_group_count, 2);
  run(&mut s, "sql SELECT x1,x2 FROM active into predictors_only");
  let owned = s.state().estimation;
  run(&mut s, "predict mean_without_outcome, n");
  assert_eq!(s.state().estimation, owned);
  rejected(&mut s, "predict denied_residual, residuals");
}

#[test]
fn one_missing_outcome_cannot_silently_select_complete_firms() {
  let mut s = fixture();
  run(
    &mut s,
    "sql SELECT id,period,x1,x2,CASE WHEN id=1 AND period=1 THEN NULL ELSE y END AS y FROM active ORDER BY id,period into unbalanced_cohort",
  );
  let before = s.state();
  let error = s.execute(parse_command(FIT).unwrap()).unwrap_err();
  assert!(
    error.to_string().contains("balanced selected groups"),
    "{error}"
  );
  assert_eq!(s.state(), before);
}

#[test]
fn current_ids_are_not_needed_and_failed_overflow_is_atomic() {
  let mut s = fixture();
  fit(&mut s);
  let owned = s.state().estimation;
  run(
    &mut s,
    "sql SELECT y,x1,x2 FROM active into prediction_frame",
  );
  assert_eq!(s.state().estimation, owned);
  run(&mut s, "predict mu, n");
  run(
    &mut s,
    "sql SELECT y,1000000000.0 AS x1,x2 FROM active into extreme_frame",
  );
  rejected(&mut s, "predict overflow, n");
}
