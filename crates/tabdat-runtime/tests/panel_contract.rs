use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_panel_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("panel firm_id year").expect("the bounded panel syntax should parse");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "panel" }
  );
}
