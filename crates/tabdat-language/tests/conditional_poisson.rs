use tabdat_domain::Command;
use tabdat_language::parse_command;
#[test]
fn explicit_grouped_conditional_fe_is_distinct_from_pooled_poisson() {
  let Command::XtPoisson { command } =
    parse_command("xtpoisson visits income age, fe id(commune)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.outcome, "visits");
  assert_eq!(command.predictors, ["income", "age"]);
  assert_eq!(command.entity_variable, "commune");
  assert!(matches!(
    parse_command("poisson visits income age").unwrap(),
    Command::Poisson { .. }
  ));
}
#[test]
fn unsupported_and_ambiguous_count_panel_forms_are_rejected() {
  for text in [
    "xtpoisson y x",
    "xtpoisson y, fe id(i)",
    "xtpoisson y x, re id(i)",
    "xtpoisson y x, fe",
    "xtpoisson y x, fe id()",
    "xtpoisson y x, fe id(i j)",
    "xtpoisson y x, fe id(i) id(j)",
    "xtpoisson y x, fe fe id(i)",
    "xtpoisson y x, fe(1) id(i)",
    "xtpoisson y x, fe id(i) robust",
    "xtpoisson y x, fe id(i) cluster(i)",
    "xtpoisson y x, fe id(i) offset(z)",
    "xtpoisson y x if z>0, fe id(i)",
    "xtpoisson y x, fe id(i) time(t)",
  ] {
    assert!(parse_command(text).is_err(), "accepted {text}");
  }
}
