use tabdat_domain::{Command, XtPoissonEstimator};
use tabdat_language::parse_command;
#[test]
fn explicit_grouped_conditional_fe_is_distinct_from_pooled_poisson() {
  let Command::XtPoisson { command } =
    parse_command("xtpoisson visits income age, fe id(commune)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.estimator, XtPoissonEstimator::ConditionalFe);
  assert_eq!(command.outcome, "visits");
  assert_eq!(command.predictors, ["income", "age"]);
  assert_eq!(command.entity_variable, "commune");
  assert!(matches!(
    parse_command("poisson visits income age").unwrap(),
    Command::Poisson { .. }
  ));
}
#[test]
fn explicit_gamma_re_is_typed_and_not_conditional_fe_or_row_nb() {
  let Command::XtPoisson { command } =
    parse_command("xtpoisson visits income age, re id(commune)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.estimator, XtPoissonEstimator::SharedGammaRe);
  assert_eq!(command.predictors, ["income", "age"]);
  assert_eq!(command.entity_variable, "commune");
}

#[test]
fn explicit_normal_re_is_distinct_and_gamma_remains_default() {
  let Command::XtPoisson { command } =
    parse_command("xtpoisson y x z, re distribution(normal) id(i)").unwrap()
  else {
    panic!()
  };
  assert_eq!(command.estimator, XtPoissonEstimator::NormalRe);
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.entity_variable, "i");
  assert_eq!(
    parse_command("xtpoisson y x, re id(i)").unwrap(),
    parse_command("xtpoisson y x, re id(i) distribution(gamma)").unwrap()
  );
}
#[test]
fn unsupported_and_ambiguous_count_panel_forms_are_rejected() {
  for text in [
    "xtpoisson y x",
    "xtpoisson y, fe id(i)",
    "xtpoisson y x, re fe id(i)",
    "xtpoisson y x, re re id(i)",
    "xtpoisson y x, re(1) id(i)",
    "xtpoisson y x, re id(i) robust",
    "xtpoisson y x, fe id(i) distribution(normal)",
    "xtpoisson y x, fe id(i) distribution(gamma)",
    "xtpoisson y x, re id(i) distribution(lognormal)",
    "xtpoisson y x, re id(i) distribution(normal gamma)",
    "xtpoisson y x, re id(i) distribution()",
    "xtpoisson y x, re id(i) distribution",
    "xtpoisson y x, re id(i) distribution(normal) distribution(gamma)",
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
