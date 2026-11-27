use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use tabdat_domain::{EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-independent-gee-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    let mut csv = String::from("id,y,x,z\n");
    for (group, size) in [1usize, 2, 4, 3, 5, 2].into_iter().enumerate() {
      for period in 1..=size {
        let g = (group + 1) as f64;
        let t = period as f64;
        let x = (g - 3.0) * 0.3 + (t - 1.0) * 0.2 + (g * t).sin() * 0.13;
        let y = 1.5 + 0.7 * x + g * 0.09 + t * 0.06 + (g * t * 1.31).sin() * 0.2;
        csv.push_str(&format!("{group},{y:.17},{x:.17},{x:.17}\n"));
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

#[test]
fn independent_fit_keeps_unbalanced_singletons_predictions_and_named_inference() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  let path = fixture.0.join("input.csv");
  run(&mut session, &format!("use \"{}\"", path.display()));
  let ExecutionResult::GaussianGee(result) = run(
    &mut session,
    "xtgee y x, family(gaussian) link(identity) corr(independent) id(id)",
  ) else {
    panic!("expected an independent Gaussian GEE result");
  };
  assert_eq!(
    result.correlation,
    tabdat_stats::GaussianGeeCorrelation::Independent
  );
  assert_eq!((result.observation_count, result.group_count), (17, 6));
  assert_eq!(
    (result.minimum_group_size, result.maximum_group_size),
    (1, 5)
  );
  assert_eq!(result.singleton_group_count, 1);
  assert_eq!(result.rho, 0.0);
  assert!((result.phi - 0.017467069899617636).abs() <= 1e-15);
  assert!(matches!(
    session.state().estimation,
    Some(EstimationState::GaussianGee(_))
  ));

  run(&mut session, "predict native_mean, xb");
  run(&mut session, "predict native_residual, residuals");
  run(
    &mut session,
    "estimates store independent_working, covariance(working)",
  );
  run(
    &mut session,
    "estimates store independent_sandwich, covariance(sandwich)",
  );
  let ExecutionResult::EstimateTable(table) = run(
    &mut session,
    "estimates table independent_working independent_sandwich",
  ) else {
    panic!("expected an independent GEE estimate table");
  };
  assert_eq!(table.models.len(), 2);
  assert_eq!(table.models[0].estimator, "gaussian_independent_gee");
  assert_eq!(table.models[0].covariance, "working_gaussian_independent");
  assert_eq!(table.models[1].covariance, "unit_sandwich_uncorrected");
  assert_eq!(table.models[0].observation_count, 17);
  assert_eq!(table.models[0].panel.as_ref().unwrap().group_count, 6);

  let before = session.state();
  let error = session
    .execute(
      parse_command("xtgee y x z, family(gaussian) link(identity) corr(independent) id(id)")
        .unwrap(),
    )
    .unwrap_err()
    .to_string();
  assert!(error.contains("xtgee"), "{error}");
  assert_eq!(session.state(), before);
}
