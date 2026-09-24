use tabdat_language::{BarCommand, Command, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_bar_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("bar sex").expect("bar parser contract");

  assert_eq!(
    command,
    Command::Bar {
      command: BarCommand {
        variable: "sex".to_string(),
        saving: None,
        include_missing: false,
        open_artifact: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bar" }
  );
}

#[test]
fn parsed_bar_with_options_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("bar sex, saving(\"out.png\") missing noopen").expect("bar parser contract");

  assert_eq!(
    command,
    Command::Bar {
      command: BarCommand {
        variable: "sex".to_string(),
        saving: Some("out.png".to_string()),
        include_missing: true,
        open_artifact: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bar" }
  );
}
