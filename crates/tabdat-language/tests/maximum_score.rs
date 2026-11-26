use tabdat_domain::{Capability, Command, CommandEffect, MaximumScoreCommand, PredictKind};
use tabdat_language::parse_command;

#[test]
fn parses_bounded_maxscore_and_current_row_classification() {
  let command = parse_command("maxscore charter lnrelp").unwrap();
  assert_eq!(
    command,
    Command::MaximumScore {
      command: MaximumScoreCommand {
        outcome: "charter".into(),
        predictor: "lnrelp".into(),
      },
    }
  );
  assert_eq!(command.name(), "maxscore");
  assert_eq!(command.effects(), &[CommandEffect::Read]);
  assert!(
    command
      .requirements()
      .contains(&Capability::ClassicalStatistics)
  );

  let Command::Predict { command } = parse_command("predict predicted_charter, class").unwrap()
  else {
    panic!("expected predict command");
  };
  assert_eq!(command.target_variable, "predicted_charter");
  assert_eq!(command.kind, PredictKind::MaximumScoreClass);
  assert!(!command.interval);
  assert!(!command.std);
  assert!(command.saving.is_none());
}

#[test]
fn rejects_unbounded_maxscore_specifications() {
  for input in [
    "maxscore",
    "maxscore y",
    "maxscore y x z",
    "maxscore y x, robust",
    "maxscore y x if y == 1",
    "maxscore y x = 1",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
