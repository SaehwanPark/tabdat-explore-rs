use tabdat_language::{Command, parse_command};

#[test]
fn conditional_choice_requires_explicit_group_and_alternative_variables() {
  let Command::Clogit { command } =
    parse_command("clogit chosen p q, group(id) alternative(alt)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.outcome, "chosen");
  assert_eq!(command.predictors, ["p", "q"]);
  assert_eq!(command.group_variable, "id");
  assert_eq!(command.alternative_variable, "alt");
  for text in [
    "clogit chosen p",
    "clogit chosen, group(id) alternative(alt)",
    "clogit chosen p, group(id)",
    "clogit chosen p, alternative(alt)",
    "clogit chosen p, group(id) group(other) alternative(alt)",
    "clogit chosen p, group(id) alternative(alt) alternative(other)",
    "clogit chosen p, group(id other) alternative(alt)",
    "clogit chosen p, group(id) alternative(1)",
    "clogit chosen p, group(id) alternative(alt) robust",
    "clogit chosen p, group(id) alternative(alt) cluster(unit)",
    "clogit chosen p, group(id) alternative(alt) noconstant",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
