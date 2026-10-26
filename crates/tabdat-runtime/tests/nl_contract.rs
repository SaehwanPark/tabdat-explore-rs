use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_nl_runtime_requires_data_and_rejects_uninstalled_math_explicitly() {
  let command =
    parse_command("nl y = a + b*x, params(a b) start(1 2)").expect("nl parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "nl" }
  );
  let unsupported = parse_command("nl y = log(a+b*x), params(a b) start(1 2)").unwrap();
  let RuntimeError::NlFailed(message) = session.execute(unsupported).unwrap_err() else {
    panic!("NLS must reject unsupported math explicitly, not report a deferred command")
  };
  assert!(message.contains("numeric arithmetic and one-argument exp only"));
  assert!(session.state().estimation.is_none());
}
