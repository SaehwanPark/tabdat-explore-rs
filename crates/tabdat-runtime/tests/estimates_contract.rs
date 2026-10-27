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
  let legacy = serde_json::to_value(&saved).unwrap();
  assert!(legacy.get("nls_model").is_none());
  assert!(legacy.get("nls_centered_r_squared").is_none());
  assert!(legacy.get("nls_covariance_correction").is_none());
  assert!(legacy.get("poisson_offset_variable").is_none());
  assert!(legacy.get("poisson_covariance_correction").is_none());
  assert!(legacy.get("poisson_mcfadden_r_squared").is_none());
  assert!(legacy.get("poisson_zero_count").is_none());
  assert!(legacy.get("poisson_count_sum").is_none());
  assert_eq!(
    serde_json::from_value::<tabdat_domain::EstimateSnapshot>(legacy).unwrap(),
    saved
  );
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
  run(&mut session, "regress y x if x > 0, noconstant");
  run(&mut session, "estimates store x_only");
  run(&mut session, "regress x z");
  run(&mut session, "estimates store other_outcome");
  let result = table(&mut session, "other_outcome x_only");
  assert_eq!(result.parameter_names, ["intercept", "z", "x"]);
  assert_eq!(result.models[0].outcome, "x");
  assert_eq!(result.models[1].outcome, "y");
  assert_eq!(result.models[1].coefficients.len(), 1);
  assert_eq!(result.models[1].coefficients[0].name, "x");
  assert_eq!(result.models[0].observation_count, 10);
  assert_eq!(result.models[1].observation_count, 6);
  assert_eq!(result.models[1].sample.total_observations, 10);
  assert_eq!(result.models[1].sample.dropped_observations, 4);
  assert_eq!(result.models[1].sample.retained_indices, [3, 4, 5, 6, 7, 9]);
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

#[test]
fn nls_snapshots_preserve_explicit_formula_covariance_and_state_across_refits() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  for (name, suffix, covariance) in [
    ("classical", "", "working_homoskedastic_gauss_newton"),
    ("robust", " robust", "hc1_gauss_newton"),
  ] {
    let ExecutionResult::Nl(fit) = run(
      &mut session,
      &format!("nl y = a+b*x, params(a b) start(1 0){suffix}"),
    ) else {
      panic!()
    };
    let before = session.state();
    run(&mut session, &format!("estimates store {name}"));
    let mut after = session.state();
    after.stored_estimates = before.stored_estimates.clone();
    assert_eq!(after, before);
    let saved = table(&mut session, name).models.remove(0);
    assert_eq!(saved.nls_model.as_ref(), Some(&fit.model));
    assert_eq!(saved.nls_centered_r_squared, fit.centered_r_squared);
    assert_eq!(
      saved.nls_covariance_correction,
      Some(fit.covariance_correction)
    );
    let json = serde_json::to_value(&saved).unwrap();
    assert_eq!(
      serde_json::from_value::<tabdat_domain::EstimateSnapshot>(json).unwrap(),
      saved
    );
    assert_eq!(saved.sample, fit.sample);
    assert_eq!(saved.covariance, covariance);
    assert_eq!(saved.covariance_matrix, fit.covariance_matrix);
    assert_eq!(saved.covariance_parameter_names, ["a", "b"]);
    assert_eq!(saved.reference_distribution, "approximate_student_t");
    assert_eq!(saved.inference_degrees_of_freedom, Some(8));
    assert!(saved.log_likelihood.is_none() && saved.r_squared.is_none());
    assert!(!saved.include_intercept);
    for (copy, original) in saved.coefficients.iter().zip(fit.coefficients.iter()) {
      assert_eq!(copy.value, original.value);
      assert_eq!(copy.standard_error, Some(original.standard_error));
      assert_eq!(copy.statistic, original.statistic);
      assert_eq!(copy.p_value, original.p_value);
    }
  }
  let saved = table(&mut session, "classical robust");
  assert_eq!(saved.parameter_names, ["a", "b"]);
  run(&mut session, "replace x = x*2");
  run(&mut session, "regress y x");
  assert_eq!(table(&mut session, "classical robust"), saved);
  let before = session.state();
  assert!(error(&mut session, "estimates store classical").contains("already stored"));
  assert_eq!(session.state(), before);
  assert!(error(&mut session, "nl y = a+b*missing, params(a b) start(1 0)").contains("unknown"));
  assert_eq!(table(&mut session, "classical robust"), saved);
}

#[test]
fn nls_snapshot_keeps_transformed_formula_and_excluded_rows() {
  let fixture = Fixture::new();
  let path = fixture.0.join("transformed.csv");
  fs::write(
    &path,
    "y,x,scale\n0.5,0,1\n1.2,1,2\n0.9,2,1\n1.3,,2\n1.7,3,1\n1.1,4,2\n2.4,5,1\n",
  )
  .unwrap();
  let mut session = Session::new();
  run(&mut session, &format!("use \"{}\"", path.display()));
  let ExecutionResult::Nl(fit) = run(
    &mut session,
    "nl y = (a+b*x)/scale, params(a b) start(1 0) robust",
  ) else {
    panic!()
  };
  run(&mut session, "estimates store transformed");
  let saved = table(&mut session, "transformed").models.remove(0);
  assert_eq!(saved.nls_model.as_ref(), Some(&fit.model));
  assert_eq!(saved.predictors, ["x", "scale"]);
  assert_eq!(saved.outcome, "y");
  assert_eq!(saved.sample.retained_indices, [0, 1, 2, 4, 5, 6]);
  assert_eq!(saved.sample.dropped_observations, 1);
  assert!(saved.sample.weights.is_none());
  assert_eq!(saved.covariance, "hc1_gauss_newton");
}

#[test]
fn poisson_snapshots_preserve_offsets_covariance_sample_and_active_fit() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  for (name, options, covariance, correction) in [
    ("model", "offset(w)", "model_based_poisson_information", 1.0),
    (
      "robust_count",
      "offset(w) robust",
      "robust_sandwich_n_over_n_minus_1",
      10.0 / 9.0,
    ),
    (
      "clustered",
      "offset(w) cluster(g)",
      "cluster_sandwich_g_over_g_minus_1",
      5.0 / 4.0,
    ),
  ] {
    let ExecutionResult::Poisson(fit) = run(&mut session, &format!("poisson y x, {options}"))
    else {
      panic!()
    };
    let before = session.state();
    run(&mut session, &format!("estimates store {name}"));
    let mut after = session.state();
    after.stored_estimates = before.stored_estimates.clone();
    assert_eq!(after, before);
    let saved = table(&mut session, name).models.remove(0);
    assert_eq!(saved.estimator, "poisson");
    assert_eq!(saved.poisson_offset_variable.as_deref(), Some("w"));
    assert_eq!(saved.poisson_covariance_correction, Some(correction));
    assert_eq!(
      saved.poisson_mcfadden_r_squared,
      Some(fit.mcfadden_r_squared)
    );
    assert_eq!(saved.poisson_zero_count, Some(fit.zero_count));
    assert_eq!(saved.poisson_count_sum, Some(fit.count_sum));
    assert_eq!(saved.covariance, covariance);
    assert_eq!(saved.covariance_matrix, fit.covariance);
    assert_eq!(saved.sample, fit.sample.clone().unwrap());
    assert_eq!(saved.reference_distribution, "asymptotic_standard_normal");
    assert_eq!(saved.inference_degrees_of_freedom, None);
    assert_eq!(saved.log_likelihood, Some(fit.log_likelihood));
    assert_eq!(
      saved.log_likelihood_convention.as_deref(),
      Some("poisson_count_pmf")
    );
    assert_eq!(saved.mcfadden_r_squared, None);
    assert_eq!(
      saved.cluster_count,
      fit.cluster.as_ref().map(|c| c.group_count)
    );
    for (copy, original) in saved.coefficients.iter().zip(fit.coefficients.iter()) {
      assert_eq!(copy.value, original.value);
      assert_eq!(copy.standard_error, Some(original.standard_error));
      assert_eq!(copy.statistic, original.statistic);
      assert_eq!(copy.p_value, original.p_value);
    }
    let serialized = serde_json::to_value(&saved).unwrap();
    assert_eq!(
      serde_json::from_value::<tabdat_domain::EstimateSnapshot>(serialized).unwrap(),
      saved
    );
  }
  let saved = table(&mut session, "model robust_count clustered");
  let active = session.state();
  let before = run(&mut session, "estat margins");
  assert_eq!(session.state(), active);
  assert_eq!(run(&mut session, "estat margins"), before);
  run(&mut session, "predict expected, n");
  run(&mut session, "poisson y x");
  run(&mut session, "estimates store no_offset");
  let no_offset = table(&mut session, "no_offset").models.remove(0);
  assert_eq!(no_offset.poisson_offset_variable, None);
  assert_eq!(no_offset.poisson_covariance_correction, Some(1.0));
  assert!(
    serde_json::to_value(&no_offset)
      .unwrap()
      .get("poisson_offset_variable")
      .is_none()
  );
  run(&mut session, "replace w = w+1");
  run(&mut session, "regress y x");
  assert_eq!(table(&mut session, "model robust_count clustered"), saved);
  let state = session.state();
  assert!(error(&mut session, "estimates store model").contains("already stored"));
  assert_eq!(session.state(), state);
  assert!(error(&mut session, "poisson absent x").contains("unknown"));
  assert!(session.state().estimation.is_none());
  assert_eq!(table(&mut session, "model robust_count clustered"), saved);
}

#[test]
fn poisson_named_snapshot_preserves_missing_offset_and_cluster_sample() {
  let fixture = Fixture::new();
  let path = fixture.0.join("missing_offset.csv");
  fs::write(&path, "y,x,w,g\n0,-2,0,a\n1,-1,1,a\n0,0,,b\n1,1,0,b\n0,2,1,c\n1,3,0,c\n1,0.5,1,d\n0,1.5,0,d\n0,-0.5,1,e\n1,2.5,0,e\n").unwrap();
  let mut session = Session::new();
  run(&mut session, &format!("use \"{}\"", path.display()));
  let ExecutionResult::Poisson(fit) = run(&mut session, "poisson y x, offset(w) cluster(g)") else {
    panic!()
  };
  run(&mut session, "estimates store incomplete");
  let saved = table(&mut session, "incomplete").models.remove(0);
  assert_eq!(saved.sample, fit.sample.unwrap());
  assert_eq!(saved.sample.retained_indices, [0, 1, 3, 4, 5, 6, 7, 8, 9]);
  assert_eq!(saved.sample.dropped_observations, 1);
  assert_eq!(saved.sample.cluster_groups.as_ref().unwrap().len(), 9);
  assert_eq!(saved.cluster_count, Some(5));
  assert_eq!(saved.poisson_offset_variable.as_deref(), Some("w"));
}
