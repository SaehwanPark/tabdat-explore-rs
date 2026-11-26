use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tabdat_domain::{EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-unstructured-gee-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,year,y,x\n");
    for group in 0..80 {
      for period in 0..4 {
        let x = (group as f64 * 0.17).sin() + period as f64 * 0.13;
        let year = if period == 0 && group % 2 == 0 {
          "-0.0".to_owned()
        } else {
          (period as f64).to_string()
        };
        let seed = (group as u64 * 1_000_003 + period as u64 * 97_409 + 1) % 2_147_483_647;
        let uniform = (seed * 48_271 % 2_147_483_647) as f64 / 2_147_483_647.0;
        let noise = 0.5 * (uniform - 0.5);
        csv.push_str(&format!(
          "{group},{year},{:.17},{x:.17}\n",
          1.4 + 0.35 * x + noise
        ));
      }
    }
    std::fs::write(path.join("input.csv"), csv).unwrap();
    Self(path)
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}

fn run(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}

fn rejected(session: &mut Session, text: &str, expected: &str) {
  let before = session.state();
  let error = session
    .execute(parse_command(text).unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains(expected), "{error}");
  assert_eq!(session.state(), before);
}

#[test]
fn unstructured_fit_is_retained_for_prediction_and_named_reports() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let path = fixture.0.join("input.csv");
  run(&mut session, &format!("use \"{}\"", path.display()));
  let ExecutionResult::GaussianGeeUnstructured(result) = run(
    &mut session,
    "xtgee y x, family(gaussian) link(identity) corr(unstructured) id(id) time(year)",
  ) else {
    panic!("expected an unstructured Gaussian GEE result");
  };
  assert_eq!(result.observation_count, 320);
  assert_eq!(result.group_count, 80);
  assert_eq!(result.period_count, 4);
  assert_eq!(result.time_variable, "year");
  assert_eq!(result.time_values, [0.0, 1.0, 2.0, 3.0]);
  assert_eq!(result.time_values[0].to_bits(), 0.0_f64.to_bits());
  assert_eq!(
    session.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::GaussianGee
  );
  assert!(matches!(
    session.state().estimation,
    Some(EstimationState::GaussianGeeUnstructured(_))
  ));
  run(&mut session, "predict mean, xb");
  run(&mut session, "predict error, residuals");
  rejected(&mut session, "predict mean, xb", "already exists");
  run(&mut session, "estimates store pa, covariance(working)");
  run(
    &mut session,
    "estimates store pa_robust, covariance(sandwich)",
  );
  assert_eq!(
    session.state().stored_estimates["pa"].estimator,
    "gaussian_period_standardized_unstructured_gee"
  );
  assert!(matches!(
    run(&mut session, "estimates table pa pa_robust"),
    ExecutionResult::EstimateTable(_)
  ));
  let before = session.state();
  run(
    &mut session,
    "sql SELECT * FROM active WHERE NOT (id=0 AND year=0) into incomplete_panel",
  );
  let incomplete = session.state();
  let error = session
    .execute(
      parse_command(
        "xtgee y x, family(gaussian) link(identity) corr(unstructured) id(id) time(year)",
      )
      .unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(
    error.contains("every entity") || error.contains("complete"),
    "{error}"
  );
  assert_eq!(session.state(), incomplete);
  assert_eq!(before.estimation, incomplete.estimation);
}

#[test]
fn exchangeable_fit_wire_stays_separate_and_unstructured_failures_keep_state() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let path = fixture.0.join("input.csv");
  run(&mut session, &format!("use \"{}\"", path.display()));
  assert!(matches!(
    run(
      &mut session,
      "xtgee y x, family(gaussian) link(identity) corr(exchangeable) id(id)"
    ),
    ExecutionResult::GaussianGee(_)
  ));
  let exchangeable = session.state().estimation.clone();
  rejected(
    &mut session,
    "xtgee y x, family(gaussian) link(identity) corr(unstructured) id(id) time(unknown_year)",
    "unknown time variable",
  );
  assert_eq!(session.state().estimation, exchangeable);
}
