use tabdat_language::{
  Command, GenerateBinaryOperator, GenerateExpression, LincomCommand, parse_command,
};
use tabdat_runtime::Session;

#[test]
fn parsed_lincom_requires_an_estimation_result() {
  let command = parse_command("lincom x1 - x2").expect("lincom parser contract");

  assert_eq!(
    command,
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("x1".to_string())),
          operator: GenerateBinaryOperator::Subtract,
          right: Box::new(GenerateExpression::Identifier("x2".to_string())),
        },
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err().to_string(),
    "no active estimation results found"
  );
}

#[test]
fn parsed_lincom_with_multiplication_requires_an_estimation_result() {
  let command = parse_command("lincom x1 + 2 * x2").expect("lincom parser contract");

  assert_eq!(
    command,
    Command::Lincom {
      command: LincomCommand {
        expression: GenerateExpression::Binary {
          left: Box::new(GenerateExpression::Identifier("x1".to_string())),
          operator: GenerateBinaryOperator::Add,
          right: Box::new(GenerateExpression::Binary {
            left: Box::new(GenerateExpression::Number("2".to_string())),
            operator: GenerateBinaryOperator::Multiply,
            right: Box::new(GenerateExpression::Identifier("x2".to_string())),
          }),
        },
      },
    }
  );

  let mut session = Session::new();

  assert_eq!(
    session.execute(command).unwrap_err().to_string(),
    "no active estimation results found"
  );
}
