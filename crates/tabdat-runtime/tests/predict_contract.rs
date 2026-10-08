use tabdat_language::{Command, PredictCommand, PredictKind, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_predict_requires_an_active_dataset_before_model_lookup() {
  let command = parse_command("predict cost_hat").expect("predict parser contract");

  assert_eq!(
    command,
    Command::Predict {
      command: PredictCommand {
        target_variable: "cost_hat".to_string(),
        kind: PredictKind::Xb,
        interval: false,
        level: "95.0".to_string(),
        std: false,
        saving: None,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "predict" }
  );
}

#[test]
fn parsed_posterior_predictive_also_requires_an_active_dataset_first() {
  let command = parse_command("predict y_pp, posterior_predictive std interval level(90)")
    .expect("predict parser contract");

  assert_eq!(
    command,
    Command::Predict {
      command: PredictCommand {
        target_variable: "y_pp".to_string(),
        kind: PredictKind::PosteriorPredictive,
        interval: true,
        level: "90".to_string(),
        std: true,
        saving: None,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "predict" }
  );
}
