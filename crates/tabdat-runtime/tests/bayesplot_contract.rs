use tabdat_language::{BayesPlotCommand, BayesPlotKind, Command, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_bayesplot_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("bayesplot trace").expect("bayesplot parser contract");

  assert_eq!(
    command,
    Command::BayesPlot {
      command: BayesPlotCommand {
        kind: BayesPlotKind::Trace,
        saving: None,
        open_artifact: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bayesplot" }
  );
}

#[test]
fn parsed_bayesplot_with_options_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("bayesplot density, saving(\"out.png\") noopen")
    .expect("bayesplot parser contract");

  assert_eq!(
    command,
    Command::BayesPlot {
      command: BayesPlotCommand {
        kind: BayesPlotKind::Density,
        saving: Some("out.png".to_string()),
        open_artifact: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bayesplot" }
  );
}
