use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn poisson_offset_is_one_named_variable_and_not_supported_for_binary_models() {
  for options in [
    "offset(`log time`)",
    "robust offset(`log time`)",
    "offset(`log time`) cluster(id) noconstant",
  ] {
    let Command::Poisson { command } = parse_command(&format!("poisson y x, {options}")).unwrap()
    else {
      panic!("not poisson");
    };
    assert_eq!(command.offset_variable.as_deref(), Some("log time"));
  }
  for input in [
    "poisson y x, offset",
    "poisson y x, offset()",
    "poisson y x, offset(a b)",
    "poisson y x, offset(a) offset(b)",
    "logit y x, offset(a)",
    "poisson y x, exposure(a)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
