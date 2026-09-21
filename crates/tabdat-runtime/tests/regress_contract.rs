use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_regress_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("regress cost age bmi").expect("regress parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "regress" }
  );
}
