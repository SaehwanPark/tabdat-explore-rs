use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_ivregress_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("ivregress 2sls y, endog(x) iv(z)")
    .expect("the bounded ivregress syntax should parse");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "ivregress" }
  );
}
