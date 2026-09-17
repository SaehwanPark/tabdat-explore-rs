use tabdat_language::{Command, RowLimit, SettingName, parse_command};

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

#[test]
fn inspection_commands_are_public_syntax_only_values() {
  assert_eq!(parse_command("count").unwrap(), Command::Count);
  assert_eq!(
    parse_command("head").unwrap(),
    Command::Head {
      limit: RowLimit::default(),
    }
  );
  let command = parse_command("tail 00042").expect("valid tail limit");
  match command {
    Command::Tail { limit } => assert_eq!(limit.as_decimal(), "42"),
    other => panic!("unexpected command: {other:?}"),
  }
}

#[test]
fn describe_is_a_public_syntax_only_command() {
  assert_eq!(parse_command(" DESCRIBE ").unwrap(), Command::Describe);
  assert_eq!(
    parse_command("describe age").unwrap_err().message(),
    "describe does not accept arguments, if clauses, or options"
  );
}

#[test]
fn doctor_is_a_public_syntax_only_command() {
  assert_eq!(parse_command(" DOCTOR ").unwrap(), Command::Doctor);
  assert_eq!(
    parse_command("doctor if age > 18").unwrap_err().message(),
    "doctor does not accept arguments, if clauses, options, or assignment syntax"
  );
}

#[test]
fn set_is_a_public_syntax_only_command() {
  assert_eq!(
    parse_command("set graph_format png").unwrap(),
    Command::Set {
      name: SettingName::GraphFormat,
      value: "png".to_owned(),
    }
  );
  assert_eq!(
    parse_command("set artifact_dir \"my plots\"").unwrap(),
    Command::Set {
      name: SettingName::ArtifactDir,
      value: "my plots".to_owned(),
    }
  );
}
