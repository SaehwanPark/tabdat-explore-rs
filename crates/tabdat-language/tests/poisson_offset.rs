use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn poisson_offset_and_exposure_are_single_named_variables_and_not_supported_for_binary_models() {
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
  for options in [
    "exposure(time)",
    "robust exposure(`person time`)",
    "exposure(time) cluster(id) noconstant",
  ] {
    let Command::Poisson { command } = parse_command(&format!("poisson y x, {options}")).unwrap()
    else {
      panic!("not poisson");
    };
    assert_eq!(
      command.exposure_variable.as_deref(),
      Some(if options.contains("person time") {
        "person time"
      } else {
        "time"
      })
    );
    assert!(command.offset_variable.is_none());
  }
  for input in [
    "poisson y x, offset",
    "poisson y x, offset()",
    "poisson y x, offset(a b)",
    "poisson y x, offset(a) offset(b)",
    "poisson y x, exposure",
    "poisson y x, exposure()",
    "poisson y x, exposure(a b)",
    "poisson y x, exposure(a) exposure(b)",
    "poisson y x, offset(a) exposure(b)",
    "logit y x, offset(a)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
