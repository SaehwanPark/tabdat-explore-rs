use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, Command, ExecutionResult, PoissonCommand};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(missing: bool) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-poisson-cluster-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let y = [
      0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 5, 4, 1, 1, 0, 0,
      2, 2, 0, 1, 0,
    ];
    let mut csv = "id,y,x,z,person\n".to_owned();
    if missing {
      csv.push_str("0,,0,0,0\n");
    }
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{},{}\n",
        i + 1,
        y,
        -2. + 4. * i as f64 / 35.,
        [0., 1., 0.4][i % 3],
        i % 6
      ));
    }
    if missing {
      csv.push_str("37,0.5,0,0,\n38,0.5,,0,1\n39,0.5,0,0,Inf\n");
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
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
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = run(s, "head 100") else {
    panic!("preview")
  };
  p.rows
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
fn number(v: &CellValue) -> f64 {
  let CellValue::Float(n) = v else {
    panic!("DOUBLE {v:?}")
  };
  *n
}
#[test]
fn joint_model_and_cluster_selection_precedes_count_validation_and_matches_r() {
  let f = Fixture::new(true);
  let mut s = f.load();
  run(&mut s, "label variable x \"original predictor\"");
  let before = s.state();
  let old = rows(&mut s);
  let ExecutionResult::Poisson(fit) = run(&mut s, "poisson y x z, cluster(person)") else {
    panic!("fit")
  };
  assert_eq!(fit.observation_count, 36);
  assert!(fit.robust && fit.likelihood_ratio_p_value.is_none());
  assert_eq!(fit.cluster.as_ref().unwrap().variable, "person");
  assert_eq!(fit.cluster.as_ref().unwrap().group_count, 6);
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (1..=36).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 40);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(
    sample.cluster_groups.as_ref().unwrap(),
    &(0..36).map(|i| (i % 6).to_string()).collect::<Vec<_>>()
  );
  close(fit.coefficients[0].value, -0.120739179664192);
  close(fit.coefficients[1].standard_error, 0.181649932874773);
  close(fit.covariance[0][2], -0.203478510401248);
  close(
    fit.omnibus_wald.unwrap().statistic.unwrap(),
    3.07142902408653,
  );
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert_eq!(rows(&mut s), old);
  let ExecutionResult::PoissonMarginalEffects(e) = run(&mut s, "estat margins") else {
    panic!("margins")
  };
  close(e.effects[0].value, -0.0862729294154299);
  close(e.effects[0].standard_error, 0.257217825584203);
  assert_eq!(e.cluster, fit.cluster);
}
#[test]
fn retained_cluster_effects_survive_edits_and_current_predictions_do_not_need_ids_or_outcome() {
  let f = Fixture::new(false);
  let mut s = f.load();
  run(&mut s, "poisson y x z, cluster(person)");
  let margins = run(&mut s, "estat margins");
  let retained = s.state().estimation;
  run(&mut s, "drop y person");
  run(&mut s, "replace x = x + 10");
  assert_eq!(run(&mut s, "estat margins"), margins);
  run(&mut s, "predict mu, n");
  run(&mut s, "predict index, xb");
  run(&mut s, "predict bare");
  for row in rows(&mut s) {
    assert_eq!(row[4], row[5]);
    close(number(&row[3]), number(&row[4]).exp());
  }
  assert_eq!(s.state().estimation, retained);
  let before = s.state();
  let old = rows(&mut s);
  for text in [
    "predict x, n",
    "predict bad, pr",
    "predict bad, residuals",
    "lincom x",
    "test x",
    "estat vif",
    "estimates store counts",
  ] {
    fail(&mut s, text);
    assert_eq!(s.state(), before, "{text}");
    assert_eq!(rows(&mut s), old);
  }
}
#[test]
fn wide_exact_cluster_ids_and_numeric_text_boolean_partitions_are_preserved() {
  let f = Fixture::new(false);
  for expression in [
    "CAST(id % 2 AS BOOLEAN)",
    "CAST(id % 2 AS VARCHAR)",
    "CASE WHEN id % 2=0 THEN CAST('9007199254740992' AS DECIMAL(38,0)) ELSE CAST('9007199254740993' AS DECIMAL(38,0)) END",
    "CASE WHEN id % 2=0 THEN CAST('18446744073709551616' AS UHUGEINT) ELSE CAST('18446744073709551617' AS UHUGEINT) END",
  ] {
    let mut s = f.load();
    run(
      &mut s,
      &format!(r#"sql """SELECT id,y,x,z,{expression} AS person FROM active""" into exact_groups"#),
    );
    let before = s.state();
    let old = rows(&mut s);
    let ExecutionResult::Poisson(fit) = run(&mut s, "poisson y x z, cluster(person)") else {
      panic!("fit")
    };
    assert_eq!(fit.cluster.unwrap().group_count, 2);
    assert_eq!(
      fit
        .sample
        .unwrap()
        .cluster_groups
        .unwrap()
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len(),
      2
    );
    assert_eq!(
      fit.omnibus_wald.unwrap().unavailable_reason.as_deref(),
      Some("insufficient_cluster_rank")
    );
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(rows(&mut s), old);
  }
}
#[test]
fn failed_new_cluster_requests_clear_only_stale_fit_and_nb2_remains_unsupported() {
  let f = Fixture::new(false);
  for (prepare, request) in [
    ("replace person = 0", "poisson y x z, cluster(person)"),
    ("replace person = person", "poisson y x z, cluster(missing)"),
    ("replace y = 0.5", "poisson y x z, cluster(person)"),
    ("replace z = x", "poisson y x z, cluster(person)"),
    (
      r#"sql """SELECT id,y,x,z,CAST(NULL AS DOUBLE) AS person FROM active""" into no_groups"#,
      "poisson y x z, cluster(person)",
    ),
    (
      r#"sql """SELECT id,y,x,z,CAST('bad' AS BLOB) AS person FROM active""" into bad_groups"#,
      "poisson y x z, cluster(person)",
    ),
    ("replace person = person", "nbreg y x z, cluster(person)"),
  ] {
    let mut s = f.load();
    run(&mut s, "label variable x \"predictor\"");
    run(&mut s, "poisson y x z, cluster(person)");
    run(&mut s, prepare);
    let mut expected = s.state();
    expected.estimation = None;
    let old = rows(&mut s);
    fail(&mut s, request);
    assert_eq!(s.state(), expected, "{request}");
    assert_eq!(rows(&mut s), old);
    fail(&mut s, "estat margins");
  }
  let mut s = f.load();
  run(&mut s, "poisson y x z, cluster(person)");
  let mut expected = s.state();
  expected.estimation = None;
  let request = Command::Poisson {
    command: PoissonCommand {
      outcome: "y".into(),
      predictors: vec!["x".into(), "z".into()],
      robust: true,
      cluster_variable: Some("person".into()),
      include_intercept: true,
    },
  };
  assert!(
    s.execute(request)
      .unwrap_err()
      .to_string()
      .contains("cannot combine robust and cluster")
  );
  assert_eq!(s.state(), expected);
  let mut empty = Session::new();
  assert_eq!(
    fail(&mut empty, "poisson y x, cluster(id)"),
    RuntimeError::NoActiveDataset { command: "poisson" }
  );
  assert!(empty.initialized_capabilities().is_empty());
}
