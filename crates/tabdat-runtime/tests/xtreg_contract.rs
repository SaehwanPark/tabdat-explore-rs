use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_xtreg_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("xtreg wage exper, fe").expect("xtreg parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "xtreg" }
  );
}
