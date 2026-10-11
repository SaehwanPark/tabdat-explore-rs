use tabdat_language::{
  Command, GenerateBinaryOperator, GenerateExpression, TestCommand, parse_command,
};
use tabdat_runtime::{RuntimeError, Session};

#[test]
fn parsed_test_requires_prior_estimation() {
  let command = parse_command("test x1 = x2").expect("test parser contract");

  assert_eq!(
    command,
    Command::Test {
      command: TestCommand {
        constraints: vec![GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("x1".to_string())),
          operator: GenerateBinaryOperator::Subtract,
          right: Box::new(GenerateExpression::Identifier("x2".to_string())),
        }],
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::TestRequiresPriorEstimation
  );
}

#[test]
fn parsed_test_with_multiple_constraints_requires_prior_estimation() {
  let command = parse_command("test (x1 = x2) (x3 = 0)").expect("test parser contract");

  assert_eq!(
    command,
    Command::Test {
      command: TestCommand {
        constraints: vec![
          GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x1".to_string())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Identifier("x2".to_string())),
          },
          GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Identifier("x3".to_string())),
            operator: GenerateBinaryOperator::Subtract,
            right: Box::new(GenerateExpression::Number("0".to_string())),
          },
        ],
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err(),
    RuntimeError::TestRequiresPriorEstimation
  );
}
