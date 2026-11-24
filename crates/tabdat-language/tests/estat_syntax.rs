use tabdat_language::parse_command;

#[test]
fn remaining_no_option_estat_forms_parse_without_a_backend() {
  for name in [
    "residuals",
    "ovtest",
    "margins",
    "overdispersion",
    "gof",
    "did",
    "drdid",
    "dml",
    "bayes",
  ] {
    let command = parse_command(&format!("estat {name}")).unwrap();
    let value = serde_json::to_value(command).unwrap();
    assert_eq!(value["kind"], "estat");
    assert_eq!(value["payload"]["command"]["subcommand"], name);
    assert_eq!(
      parse_command(&format!("estat {name}, detail"))
        .unwrap_err()
        .message(),
      format!("estat {name} does not support options")
    );
  }
}

#[test]
fn binary_discrete_margins_parse_only_one_named_predictor() {
  let command = parse_command("estat margins, discrete(treated)").unwrap();
  let value = serde_json::to_value(command).unwrap();
  assert_eq!(
    value["payload"]["command"]["subcommand"]["discrete_margins"]["variable"],
    "treated"
  );

  for invalid in [
    "estat margins, robust",
    "estat margins, discrete",
    "estat margins, discrete(x y)",
    "estat margins, discrete(x) discrete(y)",
    "estat margins, discrete(x) robust",
  ] {
    assert!(
      parse_command(invalid).is_err(),
      "unexpectedly parsed {invalid}"
    );
  }
}
