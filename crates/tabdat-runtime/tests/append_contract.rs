use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_append_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("append followup").expect("the bounded append syntax should parse");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "append" }
  );
}
