use tabdat_language::parse_command;

#[test]
fn grouping_and_empty_prefix_diagnostics_match_python() {
  for (input, expected) in [
    ("by: summarize age", "unknown command: by"),
    (
      "collapse mean age, by()",
      "option by expects at least one value",
    ),
    (
      "reshape long income, i() j(year)",
      "option i expects at least one value",
    ),
  ] {
    assert_eq!(parse_command(input).unwrap_err().message(), expected);
  }
}
