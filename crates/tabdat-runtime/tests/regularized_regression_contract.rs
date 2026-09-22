use tabdat_language::parse_command;
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_lasso_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("lasso linear y x1 x2").expect("lasso parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "lasso" }
  );
}

#[test]
fn parsed_postlasso_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("postlasso linear y x1 x2, robust").expect("postlasso parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "postlasso" }
  );
}

#[test]
fn parsed_ridge_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("ridge linear y x1 x2, alpha(2.0)").expect("ridge parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "ridge" }
  );
}

#[test]
fn parsed_elasticnet_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("elasticnet linear y x1 x2, alpha(1.5) l1_ratio(0.7)")
    .expect("elasticnet parser contract");
  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "elasticnet" }
  );
}
