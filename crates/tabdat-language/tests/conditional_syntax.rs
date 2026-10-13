use tabdat_language::parse_command;

#[test]
fn conditional_commands_parse_without_execution_support() {
  for input in ["keep if age >= 18", "drop if sex == 'F'", "keep if `if` == 1",
    "summarize age if age >= 18, detail limit=10"] {
    assert!(parse_command(input).is_ok(), "{input}");
  }
}
