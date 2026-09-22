use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_heckman_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("heckman outcome x1, selectdep(selected) select(z1)")
    .expect("heckman parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "heckman" }
  );
}
