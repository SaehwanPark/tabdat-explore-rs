use std::{fs, path::PathBuf};
use tabdat_domain::{CellValue, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

fn execute(s: &mut Session, text: &str) -> ExecutionResult {
  s.execute(parse_command(text).unwrap()).unwrap()
}
fn fail(s: &mut Session, text: &str) -> RuntimeError {
  s.execute(parse_command(text).unwrap()).unwrap_err()
}
struct Fixture(PathBuf);
impl Fixture {
  fn new(missing: bool) -> Self {
    let path =
      std::env::temp_dir().join(format!("tabdat-qreg-{}-{missing}.csv", std::process::id()));
    let input = include_str!("../../tabdat-stats/tests/fixtures/quantile/inputs.csv");
    let mut text = input.to_owned();
    if missing {
      text.push_str(",0,0\n1,,0\n1,NaN,0\nInf,0,0\n");
    }
    fs::write(&path, text).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    execute(&mut s, &format!("use {}", self.0.display()));
    s
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_file(&self.0);
  }
}
#[test]
fn joint_finite_sample_state_and_current_row_predictions_preserve_contracts() {
  let f = Fixture::new(true);
  let mut s = f.load();
  execute(&mut s, "label variable x \"predictor label\"");
  let before = s.state();
  let ExecutionResult::Quantile(fit) = execute(&mut s, "qreg y x z") else {
    panic!("fit")
  };
  let sample = fit.sample.as_ref().unwrap();
  assert_eq!(sample.retained_indices, (0..120).collect::<Vec<_>>());
  assert_eq!(sample.total_observations, 124);
  assert_eq!(sample.dropped_observations, 4);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert!(matches!(
    s.state().estimation,
    Some(EstimationState::QuantileFit(_))
  ));
  assert_eq!(
    s.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::Quantile
  );
  assert!(s.last_regression().is_none());
  let fitted_state = s.state();
  for command in [
    "predict x",
    "predict bad, posterior_predictive std",
    "predict bad, posterior_predictive interval",
    "lincom x",
    "test x = 0",
  ] {
    assert!(
      s.execute(parse_command(command).unwrap()).is_err(),
      "{command}"
    );
    assert_eq!(s.state(), fitted_state);
  }
  execute(&mut s, "replace x = 2");
  execute(&mut s, "predict fitted");
  execute(&mut s, "predict residual, residuals");
  let ExecutionResult::Head(p) = execute(&mut s, "head 200") else {
    panic!("rows")
  };
  assert_eq!(p.rows.len(), 124);
  for row in &p.rows {
    let expected = fit.coefficients[0].value + 2. * fit.coefficients[1].value;
    let CellValue::Float(actual) = row[3] else {
      panic!("prediction")
    };
    let CellValue::Float(z) = row[2] else {
      panic!("z")
    };
    assert!((actual - expected - z * fit.coefficients[2].value).abs() < 1e-10);
  }
  assert_eq!(p.rows[120][4], CellValue::Null);
  let model = s.state().estimation;
  assert!(
    fail(&mut s, "predict fitted")
      .to_string()
      .contains("exists")
  );
  assert_eq!(s.state().estimation, model);
}
#[test]
fn failed_new_fit_clears_inference_but_preserves_data_and_labels() {
  let f = Fixture::new(false);
  let mut s = f.load();
  for command in [
    "qreg y x z, robust",
    "qreg y x z, quantile(0.01)",
    "qreg y missing",
    "qreg y x x",
  ] {
    execute(&mut s, "qreg y x z");
    let before = s.state();
    assert!(
      s.execute(parse_command(command).unwrap()).is_err(),
      "{command}"
    );
    assert!(s.state().estimation.is_none());
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
  }
}

#[test]
fn qregplot_reports_pointwise_bands_and_preserves_fit_data_and_no_clobber() {
  let input = Fixture::new(false);
  let path = std::env::temp_dir().join(format!(
    "tabdat-qregplot-{}-{}.svg",
    std::process::id(),
    std::thread::current().name().unwrap_or("test")
  ));
  let _ = fs::remove_file(&path);
  let mut s = input.load();
  execute(&mut s, "regress y x z");
  let before = s.state();
  let command = format!(
    "qregplot y x z, coefficient(x) quantiles(.2 .5 .8) seed(7) reps(40) saving(\"{}\") noopen",
    path.display()
  );
  let ExecutionResult::QuantileSlopePlot(result) = execute(&mut s, &command) else {
    panic!("quantile slope plot")
  };
  assert_eq!(result.analysis.observation_count, 120);
  assert_eq!(result.analysis.points.len(), 3);
  assert_eq!(result.analysis.coefficient, "x");
  assert_eq!(result.analysis.seed, 7);
  assert!(!result.analysis.simultaneous_coverage);
  assert_eq!(
    result.analysis.sample.as_ref().unwrap().retained_indices,
    (0..120).collect::<Vec<_>>()
  );
  assert_eq!(s.state(), before);
  let svg = fs::read_to_string(&path).unwrap();
  assert!(svg.contains("class=\"quantile-slope-series\""));
  assert!(svg.contains("data-simultaneous-coverage=\"false\""));

  fs::write(&path, "keep existing figure").unwrap();
  let err = fail(&mut s, &command).to_string();
  assert!(err.contains("already exists"), "{err}");
  assert_eq!(fs::read_to_string(&path).unwrap(), "keep existing figure");
  assert_eq!(s.state(), before);

  for invalid in [
    "qregplot y absent, coefficient(absent) quantiles(.2 .5) seed(7) saving(out.svg) noopen",
    "qregplot y x x, coefficient(x) quantiles(.2 .5) seed(7) saving(out.svg) noopen",
  ] {
    assert!(
      s.execute(parse_command(invalid).unwrap()).is_err(),
      "{invalid}"
    );
    assert_eq!(s.state(), before);
  }

  execute(&mut s, "replace z = 2 * x");
  let rank_deficient_state = s.state();
  let rank_path = path.with_file_name(format!("tabdat-qregplot-rank-{}.svg", std::process::id()));
  let _ = fs::remove_file(&rank_path);
  let rank_deficient = format!(
    "qregplot y x z, coefficient(x) quantiles(.2 .5 .8) seed(7) reps(40) saving(\"{}\") noopen",
    rank_path.display()
  );
  let error = fail(&mut s, &rank_deficient).to_string();
  let error = error.to_ascii_lowercase();
  assert!(
    error.contains("rank") || error.contains("singular") || error.contains("linearly dependent"),
    "{error}"
  );
  assert!(!rank_path.exists());
  assert_eq!(s.state(), rank_deficient_state);
  fs::remove_file(path).unwrap();
}
