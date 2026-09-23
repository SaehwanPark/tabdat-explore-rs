use tabdat_language::{Command, DrDidCommand, DrDidMethod, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_drdid_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("drdid y, treat(d) post(t)").expect("drdid parser contract");

  assert_eq!(
    command,
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_string(),
        covariates: vec![],
        treatment_variable: "d".to_string(),
        post_variable: "t".to_string(),
        method: DrDidMethod::Aipw,
        robust: false,
        bootstrap: None,
        seed: None,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "drdid" }
  );
}

#[test]
fn parsed_drdid_with_covariates_method_and_bootstrap_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("drdid y x1 x2, treat(d) post(t) method(or) robust bootstrap(100) seed(42)")
      .expect("drdid parser contract");

  assert_eq!(
    command,
    Command::DrDid {
      command: DrDidCommand {
        outcome: "y".to_string(),
        covariates: vec!["x1".to_string(), "x2".to_string()],
        treatment_variable: "d".to_string(),
        post_variable: "t".to_string(),
        method: DrDidMethod::Or,
        robust: true,
        bootstrap: Some(100),
        seed: Some(42),
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "drdid" }
  );
}
