use tabdat_language::{Command, DmlCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_dml_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("dml linear y x1 x2, treat(d)").expect("dml parser contract");

  assert_eq!(
    command,
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_string(),
        controls: vec!["x1".to_string(), "x2".to_string()],
        treatment_variable: "d".to_string(),
        folds: 5,
        alpha: "1.0".to_string(),
        robust: false,
        seed: None,
        include_intercept: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "dml" }
  );
}

#[test]
fn parsed_dml_with_all_options_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("dml linear y x1, treat(d) folds(10) alpha(0.5) robust seed(42) noconstant")
      .expect("dml parser contract");

  assert_eq!(
    command,
    Command::Dml {
      command: DmlCommand {
        outcome: "y".to_string(),
        controls: vec!["x1".to_string()],
        treatment_variable: "d".to_string(),
        folds: 10,
        alpha: "0.5".to_string(),
        robust: true,
        seed: Some(42),
        include_intercept: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "dml" }
  );
}
