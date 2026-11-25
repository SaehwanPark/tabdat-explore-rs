use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn nb2_offset_and_exposure_are_single_named_variables_with_covariance_options() {
  for options in [
    "offset(`log time`)",
    "robust offset(`log time`)",
    "offset(`log time`) cluster(id) noconstant",
  ] {
    let Command::Nbreg { command } = parse_command(&format!("nbreg y x, {options}")).unwrap()
    else {
      panic!("NB2")
    };
    assert_eq!(command.offset_variable.as_deref(), Some("log time"));
  }
  for options in [
    "exposure(time)",
    "robust exposure(`person time`)",
    "exposure(time) cluster(id) noconstant",
  ] {
    let Command::Nbreg { command } = parse_command(&format!("nbreg y x, {options}")).unwrap()
    else {
      panic!("NB2")
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
    "nbreg y x, offset",
    "nbreg y x, offset()",
    "nbreg y x, offset(a b)",
    "nbreg y x, offset(a) offset(b)",
    "nbreg y x, exposure",
    "nbreg y x, exposure()",
    "nbreg y x, exposure(a b)",
    "nbreg y x, exposure(a) exposure(b)",
    "nbreg y x, offset(a) exposure(b)",
    "nbreg y x, robust cluster(id) offset(a)",
    "probit y x, offset(a)",
  ] {
    assert!(parse_command(input).is_err(), "{input}");
  }
}
