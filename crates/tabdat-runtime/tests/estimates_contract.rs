use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{Command, EstimatesCommand, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-estimates-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), "y,x,z,w,g\n0,-2,0,1,a\n1,-1,1,2,a\n0,0,0,3,b\n1,1,1,1,b\n0,2,0,2,c\n1,3,1,3,c\n1,0.5,0,1,d\n0,1.5,1,2,d\n0,-0.5,1,3,e\n1,2.5,0,1,e\n").unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut session = Session::new();
    run(
      &mut session,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
    );
    session
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn run(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}
fn error(session: &mut Session, text: &str) -> String {
  session
    .execute(parse_command(text).unwrap())
    .unwrap_err()
    .to_string()
}
fn table(session: &mut Session, names: &str) -> tabdat_domain::EstimateTableResult {
  let ExecutionResult::EstimateTable(table) = run(session, &format!("estimates table {names}"))
  else {
    panic!("expected table")
  };
  table
}

#[test]
fn copies_fit_inference_and_sample_without_mutating_active_model_data_or_labels() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "label variable x \"Price score\"");
  let ExecutionResult::Regression(fit) = run(&mut session, "regress y x, robust") else {
    panic!()
  };
  let before = session.state();
  let rows = run(&mut session, "head 20");
  run(&mut session, "estimates store OLS");
  let mut after = session.state();
  after.stored_estimates.clear();
  assert_eq!(before, after);
  let saved = table(&mut session, "OLS").models.remove(0);
  assert_eq!(saved.coefficients, fit.coefficients);
  assert_eq!(saved.covariance_matrix, fit.least_squares.covariance.matrix);
  assert_eq!(saved.sample, fit.least_squares.sample.unwrap());
  assert_eq!(saved.reference_distribution, "student_t");
  assert_eq!(saved.inference_degrees_of_freedom, Some(8));
  assert_eq!(saved.covariance, "robust");
  assert_eq!(
    saved.log_likelihood_convention.as_deref(),
    Some("gaussian_working_model_ml_variance_rss_over_n")
  );
  assert!(saved.mcfadden_r_squared.is_none());
  assert_eq!(run(&mut session, "head 20"), rows);
  assert_eq!(session.state().estimation, before.estimation);
  run(&mut session, "predict predicted");
  let state = session.state();
  table(&mut session, "OLS");
  assert_eq!(session.state(), state);
  run(&mut session, "replace x = x * 2");
  run(&mut session, "probit y x, robust");
  run(&mut session, "estimates store Probit");
  assert_eq!(table(&mut session, "OLS").models[0], saved);
  assert!(error(&mut session, "regress absent x").contains("unknown"));
  assert!(session.state().estimation.is_none());
  assert!(error(&mut session, "estimates store failed").contains("successful current"));
  assert_eq!(table(&mut session, "OLS").models[0], saved);
  run(
    &mut session,
    &format!("use \"{}\"", fixture.0.join("data.csv").display()),
  );
  let before = session.state();
  let mut returned = table(&mut session, "Probit OLS");
  assert_eq!(returned.models[1], saved);
  returned.models[1].coefficients[0].value = 999.0;
  assert_eq!(table(&mut session, "OLS").models[0], saved);
  assert_eq!(session.state(), before);
}

#[test]
fn binary_snapshots_keep_normal_inference_and_clustered_ols_keeps_g_minus_one() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  for family in ["logit", "probit"] {
    let result = run(&mut session, &format!("{family} y x, cluster(g)"));
    let fit = match result {
      ExecutionResult::Logit(fit) | ExecutionResult::Probit(fit) => fit,
      _ => panic!(),
    };
    run(&mut session, &format!("estimates store {family}"));
    let saved = table(&mut session, family).models.remove(0);
    assert_eq!(saved.reference_distribution, "asymptotic_standard_normal");
    assert_eq!(saved.inference_degrees_of_freedom, None);
    assert_eq!(saved.cluster_variable.as_deref(), Some("g"));
    assert_eq!(saved.cluster_count, Some(5));
    assert_eq!(saved.covariance, "cluster_sandwich_g_over_g_minus_1");
    assert_eq!(
      saved.log_likelihood_convention.as_deref(),
      Some("bernoulli")
    );
    assert_eq!(saved.log_likelihood, Some(fit.log_likelihood));
    assert_eq!(saved.covariance_matrix, fit.covariance);
    assert_eq!(saved.sample, fit.sample.unwrap());
    for (copy, original) in saved.coefficients.iter().zip(fit.coefficients.iter()) {
      assert_eq!(copy.value, original.value);
      assert_eq!(copy.p_value, Some(original.p_value));
    }
    let before = session.state();
    run(&mut session, "estat margins");
    assert_eq!(session.state(), before);
  }
  let ExecutionResult::Regression(fit) = run(&mut session, "regress y x, cluster(g)") else {
    panic!()
  };
  run(&mut session, "estimates store linear");
  let saved = table(&mut session, "linear").models.remove(0);
  assert_eq!(saved.inference_degrees_of_freedom, Some(4));
  assert_eq!(saved.df_residual, Some(8));
  assert_eq!(saved.coefficients, fit.coefficients);
  assert_eq!(saved.cluster_count, Some(5));
  assert_eq!(saved.reference_distribution, "student_t");
}

#[test]
fn different_designs_align_by_first_seen_exact_names_and_do_not_fill_missing_coefficients() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "regress y x, noconstant");
  run(&mut session, "estimates store x_only");
  run(&mut session, "regress x z");
  run(&mut session, "estimates store other_outcome");
  let result = table(&mut session, "other_outcome x_only");
  assert_eq!(result.parameter_names, ["intercept", "z", "x"]);
  assert_eq!(result.models[0].outcome, "x");
  assert_eq!(result.models[1].outcome, "y");
  assert_eq!(result.models[1].coefficients.len(), 1);
  assert_eq!(result.models[1].coefficients[0].name, "x");
}

#[test]
fn all_registry_errors_are_atomic_and_multi_drop_does_not_remove_known_prefix() {
  let mut empty = Session::new();
  assert!(error(&mut empty, "estimates store absent").contains("successful current"));
  assert!(error(&mut empty, "estimates table missing").contains("unknown stored"));
  assert!(empty.initialized_capabilities().is_empty());
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "regress y x");
  run(&mut session, "estimates store good");
  let state = session.state();
  for text in [
    "estimates store good",
    "estimates drop good missing",
    "estimates table missing",
  ] {
    assert!(!error(&mut session, text).is_empty());
    assert_eq!(session.state(), state);
  }
  for request in [
    EstimatesCommand::Store("bad name".into()),
    EstimatesCommand::Drop(vec!["good".into(), "good".into()]),
    EstimatesCommand::Table(vec!["good".into(); 9]),
  ] {
    assert!(
      session
        .execute(Command::Estimates { command: request })
        .is_err()
    );
    assert_eq!(session.state(), state);
  }
  for request in [
    "regress y x, wls(w)",
    "regress y x, gls(w)",
    "ivregress 2sls y, endog(x) iv(z)",
  ] {
    run(&mut session, request);
    let before = session.state();
    assert!(error(&mut session, "estimates store unsupported").contains("supports"));
    assert_eq!(session.state(), before);
  }
  run(&mut session, "regress y x");
  for i in 1..32 {
    run(&mut session, &format!("estimates store m{i}"));
  }
  let before = session.state();
  assert!(error(&mut session, "estimates store overflow").contains("at most 32"));
  assert_eq!(session.state(), before);
  run(&mut session, "estimates drop good m1");
  assert_eq!(session.state().stored_estimates.len(), 30);
  assert_eq!(session.state().estimation, before.estimation);
  run(&mut session, "estimates store good");
}

#[test]
fn ambiguous_parameter_names_and_overwide_reports_or_unions_are_rejected() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  run(&mut session, "rename x intercept");
  run(&mut session, "regress y intercept");
  let before = session.state();
  assert!(error(&mut session, "estimates store ambiguous").contains("ambiguous"));
  assert_eq!(session.state(), before);
  let names: Vec<_> = (0..65).map(|i| format!("x{i}")).collect();
  let mut csv = format!("y,{}\n", names.join(","));
  for i in 0..70 {
    csv.push_str(&format!(
      "{},{}\n",
      i % 7 + 1,
      (0..65)
        .map(|j| if i == j { "1" } else { "0" })
        .collect::<Vec<_>>()
        .join(",")
    ));
  }
  let path = fixture.0.join("wide.csv");
  fs::write(&path, csv).unwrap();
  run(&mut session, &format!("use \"{}\"", path.display()));
  run(
    &mut session,
    &format!("regress y {}, noconstant", names.join(" ")),
  );
  assert!(error(&mut session, "estimates store wide").contains("64 coefficients"));
  for (label, slice) in [("left", &names[..32]), ("right", &names[32..])] {
    run(
      &mut session,
      &format!("regress y {}, noconstant", slice.join(" ")),
    );
    run(&mut session, &format!("estimates store {label}"));
  }
  let before = session.state();
  assert!(error(&mut session, "estimates table left right").contains("union exceeds"));
  assert_eq!(session.state(), before);
}
