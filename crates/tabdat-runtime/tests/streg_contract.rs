use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_streg_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("streg time age income, failure(died) dist(weibull)")
    .expect("streg parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "streg" }
  );
}
