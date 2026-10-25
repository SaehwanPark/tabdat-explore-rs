use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn native_weibull_requires_an_active_dataset_after_parsing() {
  let command = parse_command("streg time age income, failure(died) dist(weibull)")
    .expect("streg parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::NoActiveDataset { command: "streg" }
  );
}
