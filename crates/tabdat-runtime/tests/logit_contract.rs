use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_logit_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("logit outcome x1 x2").expect("logit parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "logit" }
  );
}
