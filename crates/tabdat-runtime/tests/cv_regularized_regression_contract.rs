use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_cvlasso_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("cvlasso linear y x1 x2").expect("cvlasso parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "cvlasso" }
  );
}

#[test]
fn parsed_cvridge_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("cvridge linear y x1 x2, cv(10)").expect("cvridge parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "cvridge" }
  );
}

#[test]
fn parsed_cvelasticnet_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("cvelasticnet linear y x1 x2, cv(5) l1_ratio(0.1 0.5 0.9)")
    .expect("cvelasticnet parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand {
      name: "cvelasticnet"
    }
  );
}
