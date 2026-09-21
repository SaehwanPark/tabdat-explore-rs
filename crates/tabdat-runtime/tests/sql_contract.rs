use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_sql_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("sql select 1").expect("sql parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "sql" }
  );
}
