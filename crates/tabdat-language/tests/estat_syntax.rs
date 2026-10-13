use tabdat_language::parse_command;

#[test]
fn remaining_no_option_estat_forms_parse_without_a_backend() {
  for name in ["residuals", "ovtest", "margins", "gof", "did", "drdid", "dml", "bayes"] {
    let command = parse_command(&format!("estat {name}")).unwrap();
    let value = serde_json::to_value(command).unwrap();
    assert_eq!(value["kind"], "estat");
    assert_eq!(value["payload"]["command"]["subcommand"], name);
    assert_eq!(parse_command(&format!("estat {name}, detail")).unwrap_err().message(),
      format!("estat {name} does not support options"));
  }
}
