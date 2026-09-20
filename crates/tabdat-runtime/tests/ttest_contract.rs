use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_ttest_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("ttest wage == 0").expect("ttest parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "ttest" }
  );
}
