use tabdat_language::{Command, TtestCommand, parse_command};

#[test]
fn ttest_parses_value_pair_and_group_forms() {
  assert_eq!(
    parse_command("ttest wage == -1.5").unwrap(),
    Command::Ttest {
      command: TtestCommand {
        varname1: "wage".to_owned(),
        varname2: None,
        value: Some("-1.5".to_owned()),
        by_variable: None,
        welch: false,
      },
    }
  );
  assert_eq!(
    parse_command("ttest wage == + 2").unwrap(),
    Command::Ttest {
      command: TtestCommand {
        varname1: "wage".to_owned(),
        varname2: None,
        value: Some("+2".to_owned()),
        by_variable: None,
        welch: false,
      },
    }
  );
  assert_eq!(
    parse_command("ttest `wage value` == `exposure value`").unwrap(),
    Command::Ttest {
      command: TtestCommand {
        varname1: "wage value".to_owned(),
        varname2: Some("exposure value".to_owned()),
        value: None,
        by_variable: None,
        welch: false,
      },
    }
  );
  assert_eq!(
    parse_command("TTEST wage, by(`group value`) unequal welch").unwrap(),
    Command::Ttest {
      command: TtestCommand {
        varname1: "wage".to_owned(),
        varname2: None,
        value: None,
        by_variable: Some("group value".to_owned()),
        welch: true,
      },
    }
  );
}

#[test]
fn ttest_preserves_bounded_parser_diagnostics() {
  let cases = [
    (
      "ttest",
      "ttest command expects a variable comparison or a variable with by() option",
    ),
    (
      "ttest wage",
      "ttest command expects comparison (e.g. ttest var == value) or by() option",
    ),
    (
      "ttest wage == 0 == 1",
      "ttest command: multiple comparisons",
    ),
    (
      "ttest 1 == 0",
      "ttest command: LHS must be a single variable name",
    ),
    (
      "ttest wage == \"text\"",
      "ttest command: RHS must be a variable name or a numeric value",
    ),
    (
      "ttest wage == 1 2",
      "ttest command: RHS must be a single variable name or a numeric value",
    ),
    (
      "ttest wage, by(group), welch",
      "ttest command: duplicate comma",
    ),
    (
      "ttest wage, by(group) by(other)",
      "ttest option by may only be supplied once",
    ),
    (
      "ttest wage, by()",
      "ttest command requires option by(<variable>)",
    ),
    (
      "ttest wage, by(group other)",
      "ttest command requires option by(<variable>)",
    ),
    ("ttest wage, by=group", "ttest option by expects variables"),
    (
      "ttest wage, by(group) robust",
      "ttest unsupported option: robust",
    ),
    (
      "ttest wage, by(group) unequal=true",
      "ttest option unequal does not accept a value",
    ),
    (
      "ttest wage == .",
      "ttest command: RHS must be a variable name or a numeric value",
    ),
    ("ttest wage == 1.2.3", "malformed number: 1.2.3"),
  ];

  for (input, expected) in cases {
    assert_eq!(
      parse_command(input).unwrap_err().message(),
      expected,
      "{input:?}"
    );
  }
}
