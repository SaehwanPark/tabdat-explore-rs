use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_xtdata_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("xtdata wage, within").expect("the bounded xtdata syntax should parse");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "xtdata" }
  );
}
