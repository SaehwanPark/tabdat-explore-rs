use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn count_if_keeps_quoted_and_function_commas_in_predicates_but_rejects_command_options() {
  for source in [
    "count if enabled",
    "COUNT IF x >= 1",
    "count if text == 'a,b'",
    "count if pow(x,2) > 1",
  ] {
    let command = parse_command(source).unwrap();
    assert!(matches!(command, Command::CountQuery { .. }));
    assert_eq!(command.name(), "count");
  }
  assert_eq!(parse_command("count").unwrap(), Command::Count);
  for source in [
    "count if x > 1,",
    "count if x > 1, missing",
    "count if x > 1, unknown",
    "count x if x > 1",
    "count if x > 1 if x < 4",
    "count if x >",
    "count if",
    "count x = 1",
  ] {
    assert!(parse_command(source).is_err(), "{source}");
  }
}
