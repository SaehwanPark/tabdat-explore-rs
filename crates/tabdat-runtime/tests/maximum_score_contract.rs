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
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-maxscore-runtime-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "id,y,x\n1,0,0\n2,0,1\n3,1,2\n4,1,3\n5,,4\n6,0,\n",
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    execute(
      &mut session,
      &format!("use {}", self.0.join("data.csv").display()),
    );
    session
  }
}

impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn execute(session: &mut Session, text: &str) -> ExecutionResult {
  session.execute(parse_command(text).unwrap()).unwrap()
}

fn fail(session: &mut Session, text: &str) -> RuntimeError {
  session.execute(parse_command(text).unwrap()).unwrap_err()
}

fn numeric(cell: &CellValue) -> Option<f64> {
  match cell {
    CellValue::Float(value) => Some(*value),
    CellValue::SignedInteger(value) => Some(*value as f64),
    CellValue::UnsignedInteger(value) => Some(*value as f64),
    CellValue::Decimal { value, scale, .. } => Some(*value as f64 / 10_f64.powi(i32::from(*scale))),
    _ => None,
  }
}

#[test]
fn fit_retains_exact_sample_and_current_class_predictions_follow_the_owned_rule() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "label variable x \"relative price\"");
  let before = session.state();
  let ExecutionResult::MaximumScore(fit) = execute(&mut session, "maxscore y x") else {
    panic!("expected maximum-score result");
  };
  assert_eq!(fit.outcome, "y");
  assert_eq!(fit.predictor, "x");
  assert_eq!(fit.sample.retained_indices, [0, 1, 2, 3]);
  assert_eq!(fit.sample.total_observations, 6);
  assert_eq!(fit.sample.dropped_observations, 2);
  assert_eq!(fit.result.observation_count, 4);
  assert_eq!(fit.result.correct_count, 4);
  assert_eq!(fit.result.event_count, 2);
  assert_eq!(fit.result.optimal_regions.len(), 1);
  assert_eq!(session.state().active_dataset, before.active_dataset);
  assert_eq!(session.state().labels, before.labels);
  assert_eq!(
    session.state().estimation.as_ref().unwrap().family(),
    EstimationFamily::MaximumScore
  );
  assert!(matches!(
    session.state().estimation,
    Some(EstimationState::MaximumScoreFit(_))
  ));
  assert!(session.last_regression().is_none());

  execute(&mut session, "replace x = x + 1");
  let retained = session.state().estimation;
  execute(&mut session, "predict fitted_class, class");
  assert_eq!(session.state().estimation, retained);
  let ExecutionResult::Head(preview) = execute(&mut session, "head 20") else {
    panic!("expected rows");
  };
  assert_eq!(preview.rows.len(), 6);
  let fit = match retained.unwrap() {
    EstimationState::MaximumScoreFit(fit) => fit,
    other => panic!("unexpected retained state: {other:?}"),
  };
  for (index, row) in preview.rows.iter().enumerate() {
    let expected = match numeric(&row[2]) {
      Some(value) => {
        let class = tabdat_stats::maximum_score_predict(&fit.result.selected_rule, value).unwrap();
        CellValue::Float(if class { 1.0 } else { 0.0 })
      }
      None if row[2] == CellValue::Null => CellValue::Null,
      None => panic!("unexpected predictor cell: {:?}", row[2]),
    };
    assert_eq!(row[3], expected, "row {index}");
  }
  assert_eq!(preview.rows[4][3], CellValue::Float(1.0));
  assert_eq!(preview.rows[5][3], CellValue::Null);
  let after_prediction = session.state();
  assert!(
    fail(&mut session, "predict fitted_class, class")
      .to_string()
      .contains("exists")
  );
  assert_eq!(session.state(), after_prediction);
  assert!(
    fail(&mut session, "predict xb")
      .to_string()
      .contains("only class")
  );
  assert_eq!(session.state(), after_prediction);
}

#[test]
fn failed_new_fit_clears_the_old_rule_without_changing_data_or_labels() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  execute(&mut session, "label variable y \"choice\"");
  execute(&mut session, "maxscore y x");
  execute(&mut session, "replace y = 2 if id == 1");
  let before = session.state();
  let error = fail(&mut session, "maxscore y x");
  assert!(matches!(error, RuntimeError::MaximumScoreFailed { .. }));
  assert!(session.state().estimation.is_none());
  assert_eq!(session.state().active_dataset, before.active_dataset);
  assert_eq!(session.state().labels, before.labels);
  assert!(
    fail(&mut session, "predict predicted, class")
      .to_string()
      .contains("prior successful")
  );
}

#[test]
fn class_predictions_require_a_maxscore_fit_and_fit_failures_are_explicit() {
  let fixture = Fixture::new();
  let mut session = fixture.load();
  assert!(
    fail(&mut session, "predict predicted, class")
      .to_string()
      .contains("prior successful")
  );
  assert!(matches!(
    fail(&mut session, "maxscore missing x"),
    RuntimeError::MaximumScoreFailed { .. }
  ));
  execute(&mut session, "replace x = 1");
  assert!(matches!(
    fail(&mut session, "maxscore y x"),
    RuntimeError::MaximumScoreFailed { .. }
  ));
  assert!(session.state().estimation.is_none());
}
