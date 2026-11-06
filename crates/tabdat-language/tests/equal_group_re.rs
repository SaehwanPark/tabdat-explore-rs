use tabdat_domain::{Command, XtRegEstimator};
use tabdat_language::parse_command;

#[test]
fn named_component_method_is_opt_in_and_default_serialization_is_unchanged() {
  let original = parse_command("xtreg y x, re id(i) time(t)").unwrap();
  assert_eq!(
    original,
    parse_command("xtreg y x, re id(i) time(t) vcmethod(swamy_arora)").unwrap()
  );
  let Command::XtReg { command } =
    parse_command("xtreg y x, re id(i) time(t) vcmethod(equal_group_between)").unwrap()
  else {
    panic!();
  };
  assert_eq!(
    command.estimator,
    XtRegEstimator::RandomEffectsEqualGroupBetween
  );
  let value = serde_json::to_value(command).unwrap();
  assert_eq!(value["estimator"], "random_effects_equal_group_between");
}

#[test]
fn invalid_or_unsupported_method_combinations_are_actionable_errors() {
  for text in [
    "xtreg y x, fe vcmethod(equal_group_between)",
    "xtreg y x, mle vcmethod(equal_group_between)",
    "xtreg y x, fe vcmethod(swamy_arora)",
    "xtreg y x, re vcmethod(unknown)",
    "xtreg y x, re vcmethod(equal_group_between swamy_arora)",
    "xtreg y x, re vcmethod()",
    "xtreg y x, re vcmethod",
    "xtreg y x, re vcmethod(1)",
    "xtreg y x, re vcmethod(equal_group_between) vcmethod(swamy_arora)",
    "xtreg y x, re vcmethod(equal_group_between) robust",
    "xtreg y x, re vcmethod(equal_group_between) cluster(i)",
  ] {
    let error = parse_command(text).unwrap_err().to_string();
    assert!(error.contains("vcmethod"), "{text}: {error}");
  }
}
