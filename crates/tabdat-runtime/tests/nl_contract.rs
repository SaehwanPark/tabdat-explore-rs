use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_nl_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("nl y = a + b*x, params(a b) start(1 2)").expect("nl parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "nl" }
  );
}
