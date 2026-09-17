use tabdat_language::{Command, parse_command};

#[test]
fn public_parser_returns_owned_typed_commands() {
  let command = parse_command("help summarize").expect("valid help command");

  assert_eq!(
    command,
    Command::Help {
      topic: Some("summarize".to_owned()),
    }
  );
}

#[test]
fn parse_error_exposes_a_stable_message() {
  let error = parse_command("status now").expect_err("status has no arguments");

  assert_eq!(
    error.message(),
    "status does not accept arguments, if clauses, options, or assignment syntax"
  );
  assert_eq!(error.to_string(), error.message());
}
