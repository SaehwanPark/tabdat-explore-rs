use tabdat_language::{BayesCommand, Command, parse_command};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_bayes_linear_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("bayes linear cost age bmi").expect("bayes linear parser contract");

  assert_eq!(
    command,
    Command::Bayes {
      command: BayesCommand {
        outcome: "cost".to_string(),
        predictors: vec!["age".to_string(), "bmi".to_string()],
        n_iter: 300,
        tol: "0.001".to_string(),
        include_intercept: true,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bayes" }
  );
}

#[test]
fn parsed_bayes_linear_with_options_remains_explicitly_deferred_at_runtime() {
  let command = parse_command("bayes linear cost age, n_iter(500) tol(1e-4) noconstant")
    .expect("bayes linear parser contract");

  assert_eq!(
    command,
    Command::Bayes {
      command: BayesCommand {
        outcome: "cost".to_string(),
        predictors: vec!["age".to_string()],
        n_iter: 500,
        tol: "1e-4".to_string(),
        include_intercept: false,
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::UnsupportedCommand { name: "bayes" }
  );
}
