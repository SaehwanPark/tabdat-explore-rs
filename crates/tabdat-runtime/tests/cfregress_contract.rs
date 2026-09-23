use tabdat_language::{CfRegressCommand, Command, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_cfregress_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("cfregress cost age bmi, endog(hours) iv(distance policy)")
    .expect("cfregress parser contract");

  assert_eq!(
    command,
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost".to_string(),
        exogenous: vec!["age".to_string(), "bmi".to_string()],
        endogenous: "hours".to_string(),
        instruments: vec!["distance".to_string(), "policy".to_string()],
        robust: false,
        cluster_variable: None,
        include_intercept: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "cfregress" }
  );
}

#[test]
fn parsed_cfregress_with_cluster_and_noconstant_remains_explicitly_deferred_at_runtime() {
  let command =
    parse_command("cfregress cost, endog(hours) iv(distance) cluster(group_id) noconstant")
      .expect("cfregress parser contract");

  assert_eq!(
    command,
    Command::CfRegress {
      command: CfRegressCommand {
        outcome: "cost".to_string(),
        exogenous: vec![],
        endogenous: "hours".to_string(),
        instruments: vec!["distance".to_string()],
        robust: false,
        cluster_variable: Some("group_id".to_string()),
        include_intercept: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "cfregress" }
  );
}
