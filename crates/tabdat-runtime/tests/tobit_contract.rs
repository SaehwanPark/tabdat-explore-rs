use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_tobit_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("tobit outcome x1, ll(0)").expect("tobit parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "tobit" }
  );
}
