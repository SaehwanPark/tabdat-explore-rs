use tabdat_language::{Command, XtLogitCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_xtlogit_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("xtlogit y x, fe").expect("xtlogit parser contract");

  assert_eq!(
    command,
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_string(),
        predictors: vec!["x".to_string()],
        robust: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "xtlogit" }
  );
}

#[test]
fn parsed_xtlogit_with_multiple_predictors_and_robust_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("xtlogit y x1 x2, fe robust").expect("xtlogit parser contract");

  assert_eq!(
    command,
    Command::XtLogit {
      command: XtLogitCommand {
        outcome: "y".to_string(),
        predictors: vec!["x1".to_string(), "x2".to_string()],
        robust: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "xtlogit" }
  );
}
