use tabdat_language::{Command, ScatterCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_scatter_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("scatter price weight").expect("scatter parser contract");

  assert_eq!(
    command,
    Command::Scatter {
      command: ScatterCommand {
        y_variable: "price".to_string(),
        x_variable: "weight".to_string(),
        saving: None,
        open_artifact: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "scatter" }
  );
}

#[test]
fn parsed_scatter_with_options_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("scatter price weight, saving(\"out.png\") noopen")
    .expect("scatter parser contract");

  assert_eq!(
    command,
    Command::Scatter {
      command: ScatterCommand {
        y_variable: "price".to_string(),
        x_variable: "weight".to_string(),
        saving: Some("out.png".to_string()),
        open_artifact: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "scatter" }
  );
}
