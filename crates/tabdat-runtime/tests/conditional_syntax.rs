#![forbid(unsafe_code)]
use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn new_conditional_syntax_does_not_accidentally_execute_column_commands() {
  let mut session = Session::new();
  for (input, command) in [("keep if age > 18", "keep"), ("drop if age > 18", "drop")] {
    assert_eq!(
      session.execute(parse_command(input).unwrap()).unwrap_err(),
      RuntimeError::NoActiveDataset { command }
    );
    assert!(session.initialized_capabilities().is_empty());
  }
  assert_eq!(
    session
      .execute(parse_command("summarize age, detail").unwrap())
      .unwrap_err(),
    RuntimeError::NoActiveDataset {
      command: "summarize"
    }
  );
  assert!(session.initialized_capabilities().is_empty());
  assert!(session.active_dataset().is_none());
}
