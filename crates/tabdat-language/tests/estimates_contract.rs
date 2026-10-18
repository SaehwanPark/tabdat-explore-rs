use tabdat_language::{Command, EstimatesCommand, parse_command};

#[test]
fn explicit_report_requests_parse_and_roundtrip() {
  for (text, request) in [
    ("estimates store M1", EstimatesCommand::Store("M1".into())),
    (
      "ESTIMATES TABLE M1 m1",
      EstimatesCommand::Table(vec!["M1".into(), "m1".into()]),
    ),
    (
      "estimates drop M1 m1",
      EstimatesCommand::Drop(vec!["M1".into(), "m1".into()]),
    ),
  ] {
    let command = parse_command(text).unwrap();
    assert_eq!(command, Command::Estimates { command: request });
    assert_eq!(
      serde_json::from_str::<Command>(&serde_json::to_string(&command).unwrap()).unwrap(),
      command
    );
    assert!(command.requirements().is_empty());
  }
}

#[test]
fn invalid_implicit_and_unsupported_requests_do_not_parse() {
  for text in [
    "estimates",
    "estimates store",
    "estimates store a b",
    "estimates table",
    "estimates table a a",
    "estimates drop",
    "estimates drop a a",
    "estimates table *",
    "estimates store 1name",
    "estimates store Δ",
    "estimates store \"quoted\"",
    "estimates store a, replace",
    "estimates restore a",
    "estimates table a, b",
    "estimates table a b c d e f g h i",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
