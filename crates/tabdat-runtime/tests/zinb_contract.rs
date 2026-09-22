use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_zinb_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("zinb outcome x1 x2, inflate(z1 z2)").expect("zinb parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "zinb" }
  );
}
