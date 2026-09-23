use tabdat_language::{Command, LowessCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_lowess_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("lowess y x, gen(y_hat)").expect("lowess parser contract");

  assert_eq!(
    command,
    Command::Lowess {
      command: LowessCommand {
        outcome: "y".to_string(),
        predictor: "x".to_string(),
        target_variable: "y_hat".to_string(),
        bandwidth: (2.0f64 / 3.0f64).to_string(),
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "lowess" }
  );
}

#[test]
fn parsed_lowess_with_explicit_bandwidth_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("lowess y x, gen(y_hat) bandwidth=0.5").expect("lowess parser contract");

  assert_eq!(
    command,
    Command::Lowess {
      command: LowessCommand {
        outcome: "y".to_string(),
        predictor: "x".to_string(),
        target_variable: "y_hat".to_string(),
        bandwidth: "0.5".to_string(),
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "lowess" }
  );
}
