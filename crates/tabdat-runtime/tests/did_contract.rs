use tabdat_language::{Command, DidCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_did_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("did y, treat(d) post(t)").expect("did parser contract");

  assert_eq!(
    command,
    Command::Did {
      command: DidCommand {
        outcome: "y".to_string(),
        controls: vec![],
        treatment_variable: "d".to_string(),
        post_variable: "t".to_string(),
        robust: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "did" }
  );
}

#[test]
fn parsed_did_with_controls_and_robust_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("did y x1 x2, treat(d) post(t) robust").expect("did parser contract");

  assert_eq!(
    command,
    Command::Did {
      command: DidCommand {
        outcome: "y".to_string(),
        controls: vec!["x1".to_string(), "x2".to_string()],
        treatment_variable: "d".to_string(),
        post_variable: "t".to_string(),
        robust: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "did" }
  );
}
