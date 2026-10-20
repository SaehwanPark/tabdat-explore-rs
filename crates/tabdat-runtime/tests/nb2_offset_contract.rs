use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::Session;
const DATA: &str = include_str!("../../../examples/nb2-offset/nb2_offset.csv");
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(extra: &str) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-nb-offset-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), format!("{DATA}{extra}")).unwrap();
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
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = run(s, "head 100") else {
    panic!("head")
  };
  p.rows
}
fn number(cell: &CellValue) -> f64 {
  let CellValue::Float(v) = cell else {
    panic!("{cell:?}")
  };
  *v
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn nb2_offsets_share_exact_joint_count_sample_and_preserve_full_joint_fit() {
  let base = Fixture::new("");
  let f = Fixture::new("37,0.5,0,0,,1\n38,0.5,0,0,Inf,1\n39,0.5,0,0,0.2,\n40,,0,0,0.2,1\n");
  let mut s = f.load();
  let old = rows(&mut s);
  let ExecutionResult::Nbreg(fit) = run(&mut s, "nbreg y x z, offset(log_time) cluster(person)")
  else {
    panic!("fit")
  };
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (0..36).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 40);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(fit.cluster.as_ref().unwrap().group_count, 6);
  assert_eq!(fit.offset_variable.as_deref(), Some("log_time"));
  assert_eq!(fit.df_residual, 32);
  assert_eq!(fit.covariance.len(), 4);
  assert!(fit.covariance[0][3].abs() > 1e-6);
  assert_eq!(rows(&mut s), old);
  let mut plain = base.load();
  let ExecutionResult::Nbreg(expected) =
    run(&mut plain, "nbreg y x z, offset(log_time) cluster(person)")
  else {
    panic!("fit")
  };
  assert_eq!(fit.coefficients, expected.coefficients);
  assert_eq!(fit.dispersion, expected.dispersion);
  assert_eq!(fit.covariance, expected.covariance);
  assert_eq!(fit.null_alpha, expected.null_alpha);
  run(&mut s, "predict mu, n");
  run(&mut s, "predict eta, xb");
  let predicted = rows(&mut s);
  for (row, mu) in predicted.iter().take(36).zip(&fit.fitted_means) {
    close(number(&row[6]), *mu);
    close(number(&row[7]), mu.ln());
  }
  for row in &predicted[36..38] {
    assert_eq!(row[6], CellValue::Null);
    assert_eq!(row[7], CellValue::Null);
  }
  assert!(number(&predicted[38][6]).is_finite());
  assert!(number(&predicted[39][6]).is_finite());
}

#[test]
fn nb2_current_offset_predictions_do_not_change_retained_effects_or_probability_law() {
  let f = Fixture::new("");
  let mut s = f.load();
  run(&mut s, "nbreg y x z, offset(log_time) robust");
  let effects = run(&mut s, "estat margins");
  let probabilities = run(&mut s, "estat countprob, max(5)");
  run(&mut s, "predict original, n");
  run(&mut s, "replace log_time = log_time + 1");
  run(&mut s, "predict shifted, n");
  run(&mut s, "predict eta, xb");
  for row in rows(&mut s) {
    close(number(&row[7]), number(&row[6]) * std::f64::consts::E);
    close(number(&row[8]), number(&row[7]).ln());
  }
  assert_eq!(run(&mut s, "estat margins"), effects);
  assert_eq!(run(&mut s, "estat countprob, max(5)"), probabilities);
  run(&mut s, "drop log_time");
  let before = s.state();
  let old = rows(&mut s);
  assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(rows(&mut s), old);
  assert_eq!(run(&mut s, "estat margins"), effects);
}

#[test]
fn failed_nb2_offset_fits_clear_model_only_and_prediction_failures_publish_nothing() {
  let f = Fixture::new("");
  for (prepare, request) in [
    ("replace y = y", "nbreg y x, offset(absent)"),
    (
      r#"sql """SELECT *, 'bad' AS text FROM active""" into text_data"#,
      "nbreg y x, offset(text)",
    ),
    ("replace y = 100001", "nbreg y x, offset(log_time)"),
    (
      "replace log_time = log_time * 10000",
      "nbreg y x, offset(log_time)",
    ),
  ] {
    let mut s = f.load();
    run(&mut s, "nbreg y x z, offset(log_time)");
    run(&mut s, prepare);
    let mut expected = s.state();
    expected.estimation = None;
    let old = rows(&mut s);
    assert!(
      s.execute(parse_command(request).unwrap()).is_err(),
      "{request}"
    );
    assert_eq!(s.state(), expected);
    assert_eq!(rows(&mut s), old);
  }
  let mut s = f.load();
  run(&mut s, "nbreg y x z, offset(log_time)");
  run(
    &mut s,
    r#"sql """SELECT id,y,x,z,'bad' AS log_time,person FROM active""" into text_data"#,
  );
  let before = s.state();
  let old = rows(&mut s);
  assert!(s.execute(parse_command("predict bad, n").unwrap()).is_err());
  assert_eq!(s.state(), before);
  assert_eq!(rows(&mut s), old);
}
