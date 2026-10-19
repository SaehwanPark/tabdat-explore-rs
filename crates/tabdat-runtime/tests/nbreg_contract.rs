use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(missing: bool) -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-nb2-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let y = [
      0, 0, 2, 0, 0, 3, 0, 3, 1, 3, 8, 3, 0, 0, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 5, 4, 1, 1, 0, 0,
      2, 2, 0, 1, 0,
    ];
    let mut csv = String::from("id,y,x,z\n");
    if missing {
      csv.push_str("0,,0,0\n");
    }
    for (i, y) in y.iter().enumerate() {
      csv.push_str(&format!(
        "{},{},{},{}\n",
        i + 1,
        y,
        -2.0 + 4.0 * i as f64 / 35.0,
        [0., 1., 0.4][i % 3]
      ));
    }
    if missing {
      csv.push_str("37,0.5,,0\n38,100001,NaN,0\n39,Inf,0,0\n");
    }
    fs::write(root.join("data.csv"), csv).unwrap();
    Self(root)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    execute(
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
fn execute(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
fn rows(s: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(p) = execute(s, "head 100") else {
    panic!("preview")
  };
  p.rows
}
fn number(cell: &CellValue) -> f64 {
  let CellValue::Float(n) = cell else {
    panic!("DOUBLE: {cell:?}")
  };
  *n
}
fn same_rows(a: &[Vec<CellValue>], b: &[Vec<CellValue>]) {
  assert_eq!(a.len(), b.len());
  for (a, b) in a.iter().zip(b) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
      if let (CellValue::Float(a), CellValue::Float(b)) = (a, b)
        && a.is_nan()
        && b.is_nan()
      {
        continue;
      }
      assert_eq!(a, b);
    }
  }
}
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
#[test]
fn requires_data_without_initializing_an_engine() {
  let mut s = Session::new();
  assert_eq!(
    fail(&mut s, "nbreg y x"),
    RuntimeError::NoActiveDataset { command: "nbreg" }
  );
  assert!(s.initialized_capabilities().is_empty() && s.state().estimation.is_none());
}
#[test]
fn joint_sample_covariance_dispersion_and_retained_effects_match_independent_r() {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "label variable x \"original predictor\"");
  let before = s.state();
  let old = rows(&mut s);
  let ExecutionResult::Nbreg(classic) = execute(&mut s, "nbreg y x z") else {
    panic!("fit")
  };
  close(classic.coefficients[0].value, -0.168455956875638);
  close(classic.coefficients[1].standard_error, 0.180879427575332);
  close(classic.dispersion.alpha, 0.690042694360566);
  close(classic.covariance[0][3], -0.00961223758125336);
  close(classic.log_likelihood, -56.7885836042116);
  assert_eq!(classic.df_residual, 32);
  assert_eq!(
    classic.sample.as_ref().unwrap().retained_indices,
    (1..=36).collect::<Vec<_>>()
  );
  assert_eq!(classic.sample.as_ref().unwrap().total_observations, 40);
  assert_eq!(classic.sample.as_ref().unwrap().dropped_observations, 4);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  same_rows(&rows(&mut s), &old);
  let ExecutionResult::NbregMarginalEffects(e) = execute(&mut s, "estat margins") else {
    panic!("effects")
  };
  close(e.effects[0].value, -0.114108431828862);
  close(e.effects[0].standard_error, 0.255997093375734);
  let ExecutionResult::Nbreg(robust) = execute(&mut s, "nbreg y x z, robust") else {
    panic!("fit")
  };
  assert!(robust.likelihood_ratio_p_value.is_none());
  assert_eq!(robust.omnibus_wald.as_ref().unwrap().degrees_of_freedom, 2);
  assert_eq!(
    classic
      .coefficients
      .iter()
      .map(|c| c.value)
      .collect::<Vec<_>>(),
    robust
      .coefficients
      .iter()
      .map(|c| c.value)
      .collect::<Vec<_>>()
  );
  assert_eq!(classic.dispersion.alpha, robust.dispersion.alpha);
  assert_ne!(classic.covariance, robust.covariance);
  let margins = execute(&mut s, "estat margins");
  let retained = s.state().estimation;
  assert_eq!(
    retained.as_ref().unwrap().family(),
    EstimationFamily::NegativeBinomial
  );
  assert!(matches!(
    retained,
    Some(EstimationState::NegativeBinomialFit(_))
  ));
  assert!(s.last_regression().is_none());
  execute(&mut s, "replace x = x + 100");
  execute(&mut s, "replace y = 0");
  execute(&mut s, "drop z");
  assert_eq!(s.state().estimation, retained);
  assert_eq!(execute(&mut s, "estat margins"), margins);
  let before = s.state();
  assert!(matches!(
    fail(&mut s, "predict bad, n"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
}
#[test]
fn current_mean_index_and_bare_predictions_ignore_outcome_and_preserve_retained_fit() {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "nbreg y x z, robust");
  let retained = s.state().estimation;
  let old = rows(&mut s);
  execute(&mut s, "predict mu, n");
  execute(&mut s, "predict index, xb");
  execute(&mut s, "predict bare");
  let predicted = rows(&mut s);
  assert_eq!(predicted.len(), 40);
  for (i, row) in predicted.iter().enumerate() {
    same_rows(&[row[..4].to_vec()], &[old[i].clone()]);
    assert_eq!(row[5], row[6]);
    if i == 37 || i == 38 {
      assert_eq!(row[4], CellValue::Null);
      assert_eq!(row[5], CellValue::Null);
    } else {
      close(number(&row[4]), number(&row[5]).exp());
    }
  }
  assert!(
    predicted
      .iter()
      .filter_map(|r| match r[4] {
        CellValue::Float(n) => Some(n),
        _ => None,
      })
      .any(|n| n > 1.0)
  );
  execute(&mut s, "drop y");
  execute(&mut s, "replace x = -1000000");
  execute(&mut s, "predict large, n");
  assert!(
    rows(&mut s)
      .iter()
      .all(|r| r.last() == Some(&CellValue::Null))
  );
  execute(&mut s, "replace x = 1000000");
  execute(&mut s, "predict small, n");
  assert!(
    rows(&mut s)
      .iter()
      .all(|r| r.last() == Some(&CellValue::Float(0.0)))
  );
  assert_eq!(s.state().estimation, retained);
}
#[test]
fn unsupported_postestimation_and_failed_prediction_never_mutate_data_labels_or_model() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "label variable y \"visits\"");
  execute(&mut s, "nbreg y x z");
  let before = s.state();
  let old = rows(&mut s);
  for text in [
    "predict bad, pr",
    "predict bad, residuals",
    "predict bad, posterior_predictive interval",
    "predict y, n",
    "lincom x",
    "test x",
    "estat vif",
    "estimates store counts",
  ] {
    fail(&mut s, text);
    assert_eq!(s.state(), before, "{text}");
    assert_eq!(rows(&mut s), old);
  }
  execute(
    &mut s,
    r#"sql """SELECT id,y,'bad' AS x,z FROM active""" into text_data"#,
  );
  let before = s.state();
  let old = rows(&mut s);
  fail(&mut s, "predict bad, n");
  assert_eq!(s.state(), before);
  assert_eq!(rows(&mut s), old);
}
#[test]
fn failed_new_fits_clear_stale_inference_but_preserve_every_data_and_label_field() {
  let f = Fixture::new(false);
  for (prepare, request) in [
    ("replace y = -1", "nbreg y x z"),
    ("replace y = 0.5", "nbreg y x z"),
    ("replace y = 100001", "nbreg y x z"),
    ("replace y = 0", "nbreg y x z"),
    ("replace y = 1", "nbreg y"),
    ("replace z = x", "nbreg y x z"),
    ("replace y = y", "nbreg y x z, cluster(id)"),
    ("replace y = y", "nbreg y missing"),
  ] {
    let mut s = f.load();
    execute(&mut s, "label variable x \"predictor\"");
    execute(&mut s, "nbreg y x z");
    execute(&mut s, prepare);
    let mut expected = s.state();
    expected.estimation = None;
    let old = rows(&mut s);
    fail(&mut s, request);
    assert_eq!(s.state(), expected, "{request}");
    assert_eq!(rows(&mut s), old);
    fail(&mut s, "estat margins");
  }
}
#[test]
fn decimal_identity_and_wide_integer_bounds_are_checked_before_f64_conversion() {
  let f = Fixture::new(false);
  for expression in [
    "CAST('1.00000000000000000001' AS DECIMAL(38,20))",
    "CAST('100001' AS HUGEINT)",
    "CAST('9007199254740993' AS UHUGEINT)",
  ] {
    let mut s = f.load();
    execute(
      &mut s,
      &format!(r#"sql """SELECT id,{expression} AS y,x,z FROM active""" into exact_data"#),
    );
    let before = s.state();
    let old = rows(&mut s);
    assert!(
      fail(&mut s, "nbreg y x z")
        .to_string()
        .contains("exact nonnegative integer")
    );
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s), old);
  }
  let mut s = f.load();
  execute(
    &mut s,
    r#"sql """SELECT id,CAST(y AS DECIMAL(38,20)) AS y,x,z FROM active""" into exact_data"#,
  );
  let ExecutionResult::Nbreg(fit) = execute(&mut s, "nbreg y x z") else {
    panic!("fit")
  };
  close(fit.log_likelihood, -56.7885836042116);
}
#[test]
fn no_constant_and_intercept_only_keep_distinct_nuisance_null_and_empty_effect_semantics() {
  let f = Fixture::new(false);
  let mut s = f.load();
  let ExecutionResult::Nbreg(fit) = execute(&mut s, "nbreg y x z, noconstant") else {
    panic!("fit")
  };
  assert!(!fit.include_intercept);
  assert_eq!(fit.coefficients.len(), 2);
  assert_eq!(fit.likelihood_ratio_df, 2);
  assert_eq!(fit.covariance.len(), 3);
  let ExecutionResult::Nbreg(fit) = execute(&mut s, "nbreg y, robust") else {
    panic!("fit")
  };
  assert_eq!(fit.likelihood_ratio, 0.0);
  assert!(fit.likelihood_ratio_p_value.is_none());
  assert_eq!(
    fit.omnibus_wald.unwrap().unavailable_reason.as_deref(),
    Some("no_non_intercept_coefficients")
  );
  let ExecutionResult::NbregMarginalEffects(e) = execute(&mut s, "estat margins") else {
    panic!("effects")
  };
  assert!(e.effects.is_empty());
}
