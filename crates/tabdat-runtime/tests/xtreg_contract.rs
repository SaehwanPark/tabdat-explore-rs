use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn legacy_bare_random_effects_remains_unsupported_without_explicit_keys() {
  let command = parse_command("xtreg wage exper, re").expect("xtreg parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "xtreg" }
  );
}
