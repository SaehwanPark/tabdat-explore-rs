use tabdat_language::{Command, parse_command};

#[test]
fn nested_choice_requires_explicit_non_normalized_opt_in_and_structural_roles() {
  let Command::Nlogit {command}=parse_command("nlogit chosen p q, group(id) alternative(alt) nest(branch) upper(shore shore_income) nonnormalized").unwrap() else {panic!()};
  assert_eq!(command.outcome, "chosen");
  assert_eq!(command.predictors, ["p", "q"]);
  assert_eq!(command.upper_predictors, ["shore", "shore_income"]);
  assert_eq!(command.group_variable, "id");
  assert_eq!(command.alternative_variable, "alt");
  assert_eq!(command.nest_variable, "branch");
  let Command::Nlogit { command } =
    parse_command("nlogit chosen p, nonnormalized nest(branch) alternative(alt) group(id)")
      .unwrap()
  else {
    panic!()
  };
  assert!(command.upper_predictors.is_empty());
  for text in [
    "nlogit chosen p",
    "nlogit chosen, group(id) alternative(alt) nest(branch) nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch)",
    "nlogit chosen p, group(id) alternative(alt) nonnormalized",
    "nlogit chosen p, alternative(alt) nest(branch) nonnormalized",
    "nlogit chosen p, group(id) nest(branch) nonnormalized",
    "nlogit chosen p if id>0, group(id) alternative(alt) nest(branch) nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) upper() nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(1) nonnormalized",
    "nlogit chosen p, group(id other) alternative(alt) nest(branch) nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nest(other) nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) upper(x) upper(y) nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nonnormalized nonnormalized",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nonnormalized(1)",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nonnormalized robust",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nonnormalized cluster(id)",
    "nlogit chosen p, group(id) alternative(alt) nest(branch) nonnormalized normalized",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
