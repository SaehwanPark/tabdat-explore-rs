use tabdat_language::{Command, parse_command};

#[test]
fn normalized_choice_requires_explicit_shared_scale_mode_and_preserves_design_roles() {
  let Command::NlogitNormalizedCommon(command) = parse_command(
    "nlogit chosen p q, group(id) alternative(alt) nest(branch) upper(shore shore_income) normalized common",
  ).unwrap() else { panic!() };
  assert_eq!(command.outcome, "chosen");
  assert_eq!(command.predictors, ["p", "q"]);
  assert_eq!(command.upper_predictors, ["shore", "shore_income"]);
  assert_eq!(command.group_variable, "id");
  assert_eq!(command.alternative_variable, "alt");
  assert_eq!(command.nest_variable, "branch");
  let Command::NlogitNormalizedCommon(command) =
    parse_command("nlogit chosen p, common nest(branch) normalized alternative(alt) group(id)")
      .unwrap()
  else {
    panic!()
  };
  assert!(command.upper_predictors.is_empty());
}

#[test]
fn missing_mixed_duplicated_and_unsupported_normalized_modes_are_errors() {
  let prefix = "nlogit chosen p, group(id) alternative(alt) nest(branch)";
  for options in [
    "",
    "normalized",
    "common",
    "nonnormalized common",
    "nonnormalized normalized",
    "nonnormalized normalized common",
    "normalized normalized common",
    "normalized common common",
    "normalized(1) common",
    "normalized common(x)",
    "normalized common upper()",
    "normalized common robust",
    "normalized common cluster(id)",
    "normalized common weights(w)",
    "normalized common nest(other)",
    "normalized common upper(x) upper(y)",
  ] {
    let text = format!("{prefix} {options}");
    assert!(parse_command(&text).is_err(), "{text}");
  }
  for text in [
    "nlogit chosen p if id>0, group(id) alternative(alt) nest(branch) normalized common",
    "nlogit chosen p, alternative(alt) nest(branch) normalized common",
    "nlogit chosen p, group(id) nest(branch) normalized common",
    "nlogit chosen p, group(id) alternative(alt) normalized common",
    "nlogit chosen, group(id) alternative(alt) nest(branch) normalized common",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
