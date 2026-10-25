use std::{
  collections::BTreeMap,
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{Command, EstimationFamily, EstimationState, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new(case: usize) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-truncreg-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let mut csv = String::from("source_row,y,x,z\n0,,0,0\n");
    for line in include_str!("../../tabdat-stats/tests/fixtures/truncated_gaussian_data.csv")
      .lines()
      .skip(1)
    {
      let c: Vec<_> = line.split(',').collect();
      if c[0].parse::<usize>().unwrap() == case {
        csv.push_str(&format!("{},{},{},{}\n", c[1], c[2], c[3], c[4]));
      }
    }
    csv.push_str("241,-10,,0\n242,Inf,0,0\n243,2,NaN,0\n");
    fs::write(path.join("data.csv"), csv).unwrap();
    Self(path)
  }
  fn load(&self) -> Session {
    let mut s = Session::new();
    run(
      &mut s,
      &format!("use \"{}\"", self.0.join("data.csv").display()),
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
fn close(a: f64, b: f64) {
  assert!(
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 + 1e-8 * b.abs(),
    "{a} != {b}"
  );
}
fn gold(case: usize) -> BTreeMap<(String, usize, usize), f64> {
  include_str!("../../tabdat-stats/tests/fixtures/truncated_gaussian_reference.csv")
    .lines()
    .skip(1)
    .filter_map(|line| {
      let c: Vec<_> = line.split(',').collect();
      (c[0].parse::<usize>().unwrap() == case).then(|| {
        (
          (c[1].into(), c[2].parse().unwrap(), c[3].parse().unwrap()),
          c[4].parse().unwrap(),
        )
      })
    })
    .collect()
}
fn rows(s: &mut Session) -> String {
  format!("{:?}", run(s, "head 500"))
}

#[test]
fn independent_reference_both_intercept_conventions_full_covariance_and_original_sample() {
  for case in [1, 2] {
    let f = Fixture::new(case);
    let mut s = f.load();
    let before = rows(&mut s);
    let g = gold(case);
    let cmd = if case == 1 {
      "truncreg y x, ll(-0.6)"
    } else {
      "truncreg y x z, ll(0.8) noconstant"
    };
    let ExecutionResult::TruncatedGaussian(r) = run(&mut s, cmd) else {
      panic!("truncreg");
    };
    assert_eq!(r.observation_count, 240);
    let sample = r.sample.as_ref().unwrap();
    assert_eq!(sample.total_observations, 244);
    assert_eq!(sample.dropped_observations, 4);
    assert_eq!(sample.retained_indices, (1..=240).collect::<Vec<_>>());
    assert_eq!(before, rows(&mut s));
    for (i, b) in r.coefficients.iter().enumerate() {
      close(b.value, g[&("coefficient_value".into(), i + 1, 0)]);
      close(
        b.standard_error,
        g[&("coefficient_standard_error".into(), i + 1, 0)],
      );
    }
    for (i, row) in r.covariance.iter().enumerate() {
      for (j, value) in row.iter().enumerate() {
        close(*value, g[&("joint_covariance".into(), i + 1, j + 1)]);
      }
    }
    close(r.log_likelihood, g[&("log_likelihood".into(), 0, 0)]);
    close(
      r.null_log_likelihood,
      g[&("null_log_likelihood".into(), 0, 0)],
    );
    close(r.likelihood_ratio, g[&("likelihood_ratio".into(), 0, 0)]);
    close(r.scale.value, g[&("sigma".into(), 0, 0)]);
    assert!(matches!(
      s.state().estimation,
      Some(EstimationState::TruncatedGaussianFit(_))
    ));
    assert_eq!(
      s.state().estimation.unwrap().family(),
      EstimationFamily::TruncatedGaussian
    );
    let ExecutionResult::TruncatedGaussianMarginalEffects(e) = run(&mut s, "estat margins") else {
      panic!("margins");
    };
    for (i, b) in e.effects.iter().enumerate() {
      close(b.value, g[&("ame_value".into(), i + 1, 0)]);
      close(
        b.standard_error,
        g[&("ame_standard_error".into(), i + 1, 0)],
      );
    }
  }
}

#[test]
fn projections_need_only_current_x_preserve_identity_labels_fit_and_retained_effects() {
  let f = Fixture::new(1);
  let mut s = f.load();
  run(&mut s, "label variable x \"covariate label\"");
  let ExecutionResult::TruncatedGaussian(r) = run(&mut s, "truncreg y x, ll(-0.6)") else {
    panic!("fit");
  };
  let before = s.state();
  let margins = run(&mut s, "estat margins");
  for command in [
    "predict latent, xb",
    "predict retention, pr",
    "predict conditional, mean",
  ] {
    run(&mut s, command);
    assert_eq!(s.state().estimation, before.estimation);
    assert_eq!(s.state().labels, before.labels);
  }
  let ExecutionResult::Head(h) = run(&mut s, "head 500") else {
    panic!("head");
  };
  assert_eq!(h.rows.len(), 244);
  let inputs: Vec<_> =
    include_str!("../../tabdat-stats/tests/fixtures/truncated_gaussian_data.csv")
      .lines()
      .skip(1)
      .map(|l| l.split(',').collect::<Vec<_>>())
      .filter(|c| c[0] == "1")
      .collect();
  for (i, input) in inputs.iter().enumerate() {
    let x = [input[3].parse::<f64>().unwrap()];
    for (j, kind) in [
      tabdat_stats::TruncatedGaussianPredictionKind::Latent,
      tabdat_stats::TruncatedGaussianPredictionKind::RetentionProbability,
      tabdat_stats::TruncatedGaussianPredictionKind::ConditionalMean,
    ]
    .into_iter()
    .enumerate()
    {
      let tabdat_domain::CellValue::Float(actual) = h.rows[i + 1][j + 4] else {
        panic!("numeric prediction");
      };
      close(
        actual,
        tabdat_stats::truncated_gaussian_predict_row(&r, &x, kind).unwrap(),
      );
    }
  }
  for i in [241, 243] {
    for j in 4..7 {
      assert_eq!(h.rows[i][j], tabdat_domain::CellValue::Null);
    }
  }
  run(&mut s, "drop y");
  run(&mut s, "predict without_y, mean");
  run(&mut s, "replace x = x + 2");
  assert_eq!(run(&mut s, "estat margins"), margins);
  run(&mut s, "drop x");
  let state = s.state();
  assert!(matches!(
    fail(&mut s, "predict bad, mean"),
    RuntimeError::PredictUnknownVariable { .. }
  ));
  assert_eq!(s.state().estimation, state.estimation);
  assert_eq!(s.state().active_dataset, state.active_dataset);
}

#[test]
fn mode_specific_extreme_tail_scores_xb_and_probability_but_mean_failure_publishes_nothing() {
  let f = Fixture::new(1);
  let mut s = f.load();
  run(&mut s, "truncreg y x, ll(-0.6)");
  run(&mut s, "replace x = 0 - 100000000");
  run(&mut s, "predict extreme_latent, xb");
  run(&mut s, "predict extreme_probability, pr");
  let ExecutionResult::Head(h) = run(&mut s, "head 500") else {
    panic!("head");
  };
  for row in h.rows {
    let tabdat_domain::CellValue::Float(index) = row[4] else {
      panic!("latent");
    };
    assert!(index.is_finite() && index < -1000000.0);
    assert_eq!(row[5], tabdat_domain::CellValue::Float(0.0));
  }
  let before = s.state();
  let data = rows(&mut s);
  assert!(matches!(
    fail(&mut s, "predict bad, mean"),
    RuntimeError::TruncregFailed(_)
  ));
  assert_eq!(s.state().estimation, before.estimation);
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert_eq!(rows(&mut s), data);
}

#[test]
fn support_fit_failure_clears_only_estimation_while_prediction_failure_is_atomic() {
  let f = Fixture::new(1);
  let mut s = f.load();
  run(&mut s, "truncreg y x, ll(-0.6)");
  run(&mut s, "label variable x \"keep label\"");
  for text in [
    "predict x, mean",
    "predict bad, residuals",
    "predict bad, n",
  ] {
    let before = s.state();
    let data = rows(&mut s);
    let _ = fail(&mut s, text);
    assert_eq!(s.state().estimation, before.estimation);
    assert_eq!(s.state().active_dataset, before.active_dataset);
    assert_eq!(s.state().labels, before.labels);
    assert_eq!(rows(&mut s), data);
  }
  let Command::Predict { mut command } = parse_command("predict bad").unwrap() else {
    panic!("predict");
  };
  command.std = true;
  let before = s.state();
  assert!(s.execute(Command::Predict { command }).is_err());
  assert_eq!(s.state().estimation, before.estimation);
  run(&mut s, "replace y = 0 - 10");
  let before = s.state();
  let data = rows(&mut s);
  assert!(matches!(
    fail(&mut s, "truncreg y x, ll(-0.6)"),
    RuntimeError::TruncregFailed(_)
  ));
  assert!(s.state().estimation.is_none());
  assert_eq!(s.state().active_dataset, before.active_dataset);
  assert_eq!(s.state().labels, before.labels);
  assert_eq!(rows(&mut s), data);
}
