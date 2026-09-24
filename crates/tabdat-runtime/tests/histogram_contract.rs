use tabdat_language::{Command, HistogramCommand, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_histogram_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("histogram price").expect("histogram parser contract");

  assert_eq!(
    command,
    Command::Histogram {
      command: HistogramCommand {
        variable: "price".to_string(),
        bins: None,
        saving: None,
        open_artifact: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "histogram" }
  );
}

#[test]
fn parsed_histogram_with_options_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("histogram weight, bins=25 saving(\"out.png\") noopen")
    .expect("histogram parser contract");

  assert_eq!(
    command,
    Command::Histogram {
      command: HistogramCommand {
        variable: "weight".to_string(),
        bins: Some(25),
        saving: Some("out.png".to_string()),
        open_artifact: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "histogram" }
  );
}
