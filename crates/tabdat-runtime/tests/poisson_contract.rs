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
      "tabdat-poisson-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    // Same independent R reference as the statistical core's multiple case.
    let x = [-2., -1., 0., 1., 2., 3., 0.5, 1.5, -0.5, 2.5, -1.5, 3.5];
    let z = [0., 1., 0., 1., 0., 1., 0., 1., 1., 0., 0.2, 0.7];
    let y = [0, 1, 0, 2, 1, 5, 2, 0, 3, 1, 4, 2];
    let mut csv = String::from("id,y,x,z\n");
    if missing {
      csv.push_str("0,,0,0\n");
    }
    for i in 0..12 {
      csv.push_str(&format!("{},{},{},{}\n", i + 1, y[i], x[i], z[i]));
    }
    if missing {
      csv.push_str("13,0.5,,0\n14,0,NaN,0\n15,Inf,0,0\n");
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
fn close(a: f64, b: f64) {
  assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
fn number(v: &CellValue) -> f64 {
  let CellValue::Float(v) = v else {
    panic!("DOUBLE: {v:?}")
  };
  *v
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

#[test]
fn requires_data_without_initializing_an_engine() {
  let mut s = Session::new();
  assert_eq!(
    fail(&mut s, "poisson y x"),
    RuntimeError::NoActiveDataset { command: "poisson" }
  );
  assert!(s.initialized_capabilities().is_empty());
  assert!(s.state().estimation.is_none());
}

#[test]
fn both_covariances_retain_exact_finite_sample_design_and_normal_delta_inference() {
  let f = Fixture::new(true);
  let (se, effect_se) = (0.152892479974562_f64, 0.261386952568861_f64);
  let mut s = f.load();
  execute(&mut s, "label variable x \"original predictor\"");
  let before = s.state();
  let old = rows(&mut s);
  let ExecutionResult::Poisson(classic) = execute(&mut s, "poisson y x z") else {
    panic!("fit")
  };
  close(classic.coefficients[0].value, 0.168775785473076);
  close(classic.coefficients[1].value, 0.0580678346024785);
  close(classic.coefficients[1].standard_error, 0.129713260011305);
  close(classic.log_likelihood, -20.0771807611182);
  close(classic.pearson_dispersion, 1.64572293378832);
  assert_eq!(
    classic.sample.as_ref().unwrap().retained_indices,
    (1..=12).collect::<Vec<_>>()
  );
  assert_eq!(classic.sample.as_ref().unwrap().total_observations, 16);
  assert_eq!(classic.sample.as_ref().unwrap().dropped_observations, 4);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  same_rows(&rows(&mut s), &old);
  let ExecutionResult::PoissonMarginalEffects(e) = execute(&mut s, "estat margins") else {
    panic!("effects")
  };
  close(e.effects[0].standard_error, 0.22807875092159);
  let ExecutionResult::Poisson(robust) = execute(&mut s, "poisson y x z, robust") else {
    panic!("fit")
  };
  assert!(robust.likelihood_ratio_p_value.is_none());
  assert!(robust.omnibus_wald.is_some());
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
  close(robust.coefficients[1].standard_error, se);
  let ame = execute(&mut s, "estat margins");
  let ExecutionResult::PoissonMarginalEffects(e) = &ame else {
    panic!("effects")
  };
  close(e.effects[0].value, 0.101618710554337);
  close(e.effects[0].standard_error, effect_se);
  let retained = s.state().estimation;
  let state = retained.as_ref().unwrap();
  assert_eq!(state.family(), EstimationFamily::Poisson);
  assert!(matches!(state, EstimationState::PoissonFit(_)));
  assert!(s.last_regression().is_none());
  execute(&mut s, "replace x = x + 100");
  execute(&mut s, "replace y = 0");
  assert_eq!(s.state().estimation, retained);
  assert_eq!(execute(&mut s, "estat margins"), ame);
  execute(&mut s, "drop x");
  let before = s.state();
  assert!(matches!(
    fail(&mut s, "predict bad, n"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state(), before);
  assert_eq!(execute(&mut s, "estat margins"), ame);
}

#[test]
fn current_prediction_needs_only_predictors_preserves_model_and_nulls_nonfinite_or_overflowing_means()
 {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "poisson y x z");
  let retained = s.state().estimation;
  let old = rows(&mut s);
  execute(&mut s, "predict mu, n");
  execute(&mut s, "predict index, xb");
  execute(&mut s, "predict bare");
  let p = rows(&mut s);
  assert_eq!(p.len(), 16);
  for (i, row) in p.iter().enumerate() {
    same_rows(&[row[..4].to_vec()], &[old[i].clone()]);
    assert_eq!(row[5], row[6]);
    if i == 13 || i == 14 {
      assert_eq!(row[4], CellValue::Null);
      assert_eq!(row[5], CellValue::Null);
    } else {
      close(number(&row[4]), number(&row[5]).exp());
    }
  }
  assert!(number(&p[0][4]) > 1.0);
  assert!(number(&p[15][4]) > 1.0);
  execute(&mut s, "drop y");
  execute(&mut s, "replace x = 1000000");
  execute(&mut s, "predict large, n");
  assert!(
    rows(&mut s)
      .iter()
      .all(|r| r.last() == Some(&CellValue::Null))
  );
  execute(&mut s, "replace x = -1000000");
  execute(&mut s, "predict small, n");
  assert!(
    rows(&mut s)
      .iter()
      .all(|r| r.last() == Some(&CellValue::Float(0.0)))
  );
  execute(
    &mut s,
    r#"sql """SELECT id, CAST('1e308' AS DOUBLE) AS x, z FROM active""" into huge_current"#,
  );
  execute(&mut s, "predict finite_index, xb");
  assert!(
    rows(&mut s)
      .iter()
      .all(|r| number(r.last().unwrap()).is_finite())
  );
  assert_eq!(s.state().estimation, retained);
}

#[test]
fn predictions_and_unsupported_postestimation_fail_without_any_mutation() {
  let f = Fixture::new(false);
  let mut s = f.load();
  execute(&mut s, "poisson y x z, robust");
  let before = s.state();
  let old = rows(&mut s);
  for text in [
    "predict bad, pr",
    "predict bad, residuals",
    "predict bad, posterior_predictive interval",
    "predict bad, posterior_predictive saving(\"bad.csv\")",
    "predict y, n",
    "lincom x",
    "test x",
    "estat vif",
  ] {
    fail(&mut s, text);
    assert_eq!(s.state(), before, "{text}");
    same_rows(&rows(&mut s), &old);
  }
  execute(
    &mut s,
    r#"sql """SELECT id, y, 'bad' AS x, z FROM active""" into text_data"#,
  );
  let before = s.state();
  let old = rows(&mut s);
  fail(&mut s, "predict bad, n");
  assert_eq!(s.state(), before);
  assert_eq!(rows(&mut s), old);
}

#[test]
fn new_fit_failures_clear_stale_inference_but_preserve_data_and_labels() {
  let f = Fixture::new(false);
  for (prepare, request) in [
    ("replace y = -1", "poisson y x z"),
    ("replace y = 0.5", "poisson y x z"),
    ("replace y = 0", "poisson y x z"),
    ("replace z = x", "poisson y x z"),
    ("replace y = 9007199254740992", "poisson y x z"),
    ("replace id = 1", "poisson y x z, cluster(id)"),
    ("replace y = y", "poisson y missing"),
  ] {
    let mut s = f.load();
    execute(&mut s, "label variable y \"count outcome\"");
    execute(&mut s, "poisson y x z");
    execute(&mut s, prepare);
    let mut expected = s.state();
    expected.estimation = None;
    let old = rows(&mut s);
    fail(&mut s, request);
    assert_eq!(s.state(), expected, "{request}");
    assert_eq!(rows(&mut s), old);
    assert!(fail(&mut s, "estat margins").to_string().contains("prior"));
  }
}

#[test]
fn exact_decimal_and_wide_counts_cannot_round_into_the_domain() {
  let f = Fixture::new(false);
  for expression in [
    "CAST('1.00000000000000000001' AS DECIMAL(38,20))",
    "CAST('9007199254740993' AS HUGEINT)",
    "CAST('9007199254740993' AS UHUGEINT)",
  ] {
    let mut s = f.load();
    execute(
      &mut s,
      &format!(r#"sql """SELECT id, {expression} AS y, x, z FROM active""" into exact_data"#),
    );
    let before = s.state();
    let old = rows(&mut s);
    assert!(
      fail(&mut s, "poisson y x z")
        .to_string()
        .contains("exact nonnegative integer")
    );
    assert_eq!(s.state(), before);
    assert_eq!(rows(&mut s), old);
  }
  let mut s = f.load();
  execute(
    &mut s,
    r#"sql """SELECT id, CAST(y AS DECIMAL(38,20)) AS y, x, z FROM active""" into exact_data"#,
  );
  let ExecutionResult::Poisson(fit) = execute(&mut s, "poisson y x z") else {
    panic!("fit")
  };
  close(fit.log_likelihood, -20.0771807611182);
}

#[test]
fn noconstant_intercept_only_and_zero_variance_inference_have_explicit_semantics() {
  let f = Fixture::new(false);
  let mut s = f.load();
  let ExecutionResult::Poisson(fit) = execute(&mut s, "poisson y x z, noconstant") else {
    panic!("fit")
  };
  assert!(!fit.include_intercept);
  assert_eq!(fit.coefficients.len(), 2);
  assert_eq!(fit.likelihood_ratio_df, 2);
  execute(&mut s, "poisson y");
  let ExecutionResult::PoissonMarginalEffects(e) = execute(&mut s, "estat margins") else {
    panic!("effects")
  };
  assert!(e.effects.is_empty());
  execute(&mut s, "replace y = 1");
  let ExecutionResult::Poisson(fit) = execute(&mut s, "poisson y, robust") else {
    panic!("fit")
  };
  assert_eq!(fit.coefficients[0].standard_error, 0.0);
  assert!(fit.coefficients[0].p_value.is_none());
  assert!(fit.coefficients[0].confidence_lower.is_none());
  assert!(fit.omnibus_wald.unwrap().statistic.is_none());
  execute(&mut s, "regress y x");
  let before = s.state();
  assert_eq!(
    fail(&mut s, "predict invalid, n"),
    RuntimeError::PredictMeanCountRequiresPoisson
  );
  assert_eq!(s.state(), before);
}
