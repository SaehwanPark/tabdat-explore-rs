use std::{
  fs,
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_domain::{CellValue, ExecutionResult};
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};
use tabdat_stats::BinaryModel;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-binary-probability-inference-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      include_str!("../../../examples/binary-factor-design/binary_factor_data.csv"),
    )
    .unwrap();
    Self(root)
  }

  fn load(&self) -> Session {
    let mut session = Session::new();
    run(
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

fn run(session: &mut Session, command: &str) -> ExecutionResult {
  session.execute(parse_command(command).unwrap()).unwrap()
}

fn fail(session: &mut Session, command: &str) -> RuntimeError {
  session
    .execute(parse_command(command).unwrap())
    .unwrap_err()
}

fn preview(session: &mut Session) -> Vec<Vec<CellValue>> {
  let ExecutionResult::Head(result) = run(session, "head 300") else {
    panic!("expected head result")
  };
  result.rows
}

fn reference(
  model: &str,
  design: &str,
  covariance: &str,
  id: usize,
  level: usize,
) -> Vec<&'static str> {
  include_str!(
    "../../../examples/binary-factor-design/binary_factor_probability_inference_reference.csv"
  )
  .lines()
  .skip(1)
  .map(|line| line.split(',').collect::<Vec<_>>())
  .find(|fields| {
    fields[0].trim_matches('"') == model
      && fields[1].trim_matches('"') == design
      && fields[2].trim_matches('"') == covariance
      && fields[3].parse::<usize>().unwrap() == id
      && fields[4].parse::<usize>().unwrap() == level
  })
  .unwrap()
}

fn close(actual: f64, expected: f64, label: &str) {
  assert!(
    (actual - expected).abs() <= 2e-8,
    "{label}: {actual:.17} != {expected:.17}"
  );
}

fn compare_reference(row: &[CellValue], actual_index: usize, expected: &str, label: &str) {
  if expected.is_empty() {
    assert_eq!(row[actual_index], CellValue::Null, "{label}");
  } else {
    let CellValue::Float(actual) = row[actual_index] else {
      panic!("expected numeric {label}, got {:?}", row[actual_index]);
    };
    close(actual, expected.parse().unwrap(), label);
  }
}

#[test]
fn binary_probability_se_and_link_scale_intervals_match_r_for_each_covariance() {
  let fixture = Fixture::new();
  for (model, model_name) in [
    (BinaryModel::Logit, "logit"),
    (BinaryModel::Probit, "probit"),
  ] {
    for (covariance, option) in [
      ("model_based", ""),
      ("robust", ", robust"),
      ("cluster", ", cluster(cluster)"),
    ] {
      let mut session = fixture.load();
      let fit_command = format!("{model_name} y x i.region{option}");
      run(&mut session, &fit_command);
      let retained_fit = session.state().estimation;
      let labels = session.state().labels;
      run(&mut session, "predict probability_se, pr se");
      run(&mut session, "predict probability95, pr interval");
      run(&mut session, "predict probability90, pr interval level(90)");
      assert_eq!(session.state().estimation, retained_fit);
      assert_eq!(session.state().labels, labels);
      let predicted = preview(&mut session);
      assert_eq!(predicted.len(), 240);
      assert_eq!(predicted[0].len(), 14);
      for (index, row) in predicted.iter().enumerate() {
        let id = index + 1;
        let expected95 = reference(model_name, "default", covariance, id, 95);
        let expected90 = reference(model_name, "default", covariance, id, 90);
        compare_reference(row, 7, expected95[6], "probability SE");
        compare_reference(row, 8, expected95[5], "probability");
        compare_reference(row, 9, expected95[7], "95% lower");
        compare_reference(row, 10, expected95[8], "95% upper");
        compare_reference(row, 11, expected90[5], "90% probability");
        compare_reference(row, 12, expected90[7], "90% lower");
        compare_reference(row, 13, expected90[8], "90% upper");
      }

      if model == BinaryModel::Logit && covariance == "model_based" {
        let before = session.state();
        let before_rows = predicted.clone();
        assert_eq!(
          fail(&mut session, "predict probability95, pr interval"),
          RuntimeError::PredictFailed
        );
        assert_eq!(session.state(), before);
        assert_eq!(preview(&mut session), before_rows);

        run(&mut session, "generate blocked_lower = 1");
        let before_collision = session.state();
        let before_collision_rows = preview(&mut session);
        assert_eq!(
          fail(&mut session, "predict blocked, pr interval"),
          RuntimeError::PredictFailed
        );
        assert_eq!(session.state(), before_collision);
        assert_eq!(preview(&mut session), before_collision_rows);

        run(&mut session, "replace x = x + 1 if id == 1");
        run(&mut session, "predict shifted, pr interval");
        let shifted = preview(&mut session);
        assert_ne!(shifted[0][15], before_rows[0][8]);
        assert_eq!(session.state().estimation, retained_fit);
        run(&mut session, "replace region = \"South\" if id == 2");
        run(&mut session, "predict unseen_se, pr se");
        let unseen = preview(&mut session);
        assert_eq!(unseen[1].last(), Some(&CellValue::Null));
        assert_eq!(session.state().estimation, retained_fit);
      }
    }
  }
}

#[test]
fn continuous_only_predictions_ignore_irrelevant_factor_columns() {
  let fixture = Fixture::new();
  for (_model, model_name) in [
    (BinaryModel::Logit, "logit"),
    (BinaryModel::Probit, "probit"),
  ] {
    for (covariance, option) in [
      ("model_based", ""),
      ("robust", ", robust"),
      ("cluster", ", cluster(cluster)"),
    ] {
      let mut session = fixture.load();
      run(&mut session, &format!("{model_name} y x{option}"));
      run(&mut session, "predict probability_se, pr se");
      run(&mut session, "predict probability95, pr interval");
      run(&mut session, "predict probability90, pr interval level(90)");
      let predicted = preview(&mut session);
      assert_eq!(predicted.len(), 240);
      for (index, row) in predicted.iter().enumerate() {
        let id = index + 1;
        let expected95 = reference(model_name, "continuous", covariance, id, 95);
        let expected90 = reference(model_name, "continuous", covariance, id, 90);
        compare_reference(row, 7, expected95[6], "continuous probability SE");
        compare_reference(row, 8, expected95[5], "continuous probability");
        compare_reference(row, 9, expected95[7], "continuous 95% lower");
        compare_reference(row, 10, expected95[8], "continuous 95% upper");
        compare_reference(row, 11, expected90[5], "continuous 90% probability");
        compare_reference(row, 12, expected90[7], "continuous 90% lower");
        compare_reference(row, 13, expected90[8], "continuous 90% upper");
      }
      // Missing region is irrelevant for the continuous-only fit; missing x is not.
      assert!(matches!(predicted[4][8], CellValue::Float(_)));
      assert_eq!(predicted[11][8], CellValue::Null);
      // Missing y does not prevent current-row scoring.
      assert!(matches!(predicted[17][8], CellValue::Float(_)));
    }
  }
}
