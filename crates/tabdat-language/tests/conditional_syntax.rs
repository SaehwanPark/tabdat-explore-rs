use tabdat_language::{
  Command, GenerateBinaryOperator, GenerateExpression, RowFilterAction, SyntaxOption,
  SyntaxOptionValue, parse_command,
};

#[test]
fn condition_and_option_fields_are_retained() {
  let condition = GenerateExpression::Binary {
    left: Box::new(GenerateExpression::Identifier("age".into())),
    operator: GenerateBinaryOperator::GreaterOrEqual,
    right: Box::new(GenerateExpression::Number("18".into())),
  };
  assert_eq!(
    parse_command("keep if age >= 18").unwrap(),
    Command::RowFilter {
      action: RowFilterAction::Keep,
      condition: condition.clone(),
    }
  );
  assert_eq!(
    parse_command("summarize age if age >= 18, detail limit=10").unwrap(),
    Command::SummarizeQuery {
      variables: vec!["age".into()],
      condition: Some(condition),
      options: vec![
        SyntaxOption {
          name: "detail".into(),
          value: SyntaxOptionValue::Flag
        },
        SyntaxOption {
          name: "limit".into(),
          value: SyntaxOptionValue::Number("10".into())
        }
      ],
    }
  );
}

#[test]
fn conditional_commands_parse_without_execution_support() {
  for input in [
    "keep if age >= 18",
    "drop if sex == 'F'",
    "keep if `if` == 1",
    "keep if age > 0,",
    "drop if age > 0,",
    "summarize age, detail",
    "keep if inrange(age, 18, 65)",
    "summarize age if inrange(age, 18, 65), detail",
    "summarize age if age >= 18, detail limit=10",
  ] {
    assert!(parse_command(input).is_ok(), "{input}");
  }
}
