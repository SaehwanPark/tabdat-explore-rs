use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_zip_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("zip outcome x1 x2, inflate(z1 z2)").expect("zip parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "zip" }
  );
}
