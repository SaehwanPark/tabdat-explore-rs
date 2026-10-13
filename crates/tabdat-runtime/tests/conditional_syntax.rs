#![forbid(unsafe_code)]
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn new_conditional_syntax_does_not_accidentally_execute_column_commands() {
  let mut session = Session::new();
  for (input, name) in [
    ("keep if age > 18", "keep"),
    ("drop if age > 18", "drop"),
    ("summarize age, detail", "summarize"),
  ] {
    assert!(matches!(session.execute(parse_command(input).unwrap()),
      Err(RuntimeError::UnsupportedCommand { name: actual }) if actual == name));
    assert!(session.initialized_capabilities().is_empty());
    assert!(session.active_dataset().is_none());
  }
}
