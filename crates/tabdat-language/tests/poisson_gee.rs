use tabdat_domain::Command;
use tabdat_language::parse_command;
#[test]
fn poisson_log_pair_is_typed_separately_with_explicit_options() {
  for text in [
    "xtgee y x z, family(poisson) link(log) corr(exchangeable) id(i)",
    "xtgee y x z, id(i) corr(EXCHANGEABLE) link(LOG) family(POISSON)",
  ] {
    let Command::XtGeePoisson { command } = parse_command(text).unwrap() else {
      panic!("wrong family")
    };
    assert_eq!(command.outcome, "y");
    assert_eq!(command.predictors, ["x", "z"]);
    assert_eq!(command.entity_variable, "i");
  }
}
#[test]
fn unsupported_missing_duplicate_and_cross_family_forms_fail() {
  for text in [
    "xtgee y x, family(poisson) link(identity) corr(exchangeable) id(i)",
    "xtgee y x, family(gaussian) link(log) corr(exchangeable) id(i)",
    "xtgee y x, family(poisson) link(log) corr(independence) id(i)",
    "xtgee y x, family(poisson) link(log) id(i)",
    "xtgee y x, family(poisson) link(log) corr(exchangeable) id(i j)",
    "xtgee y x, family(poisson) link(log) corr(exchangeable) id(i) robust",
    "xtgee y x, family(poisson) family(poisson) link(log) corr(exchangeable) id(i)",
    "xtgee y x if x>0, family(poisson) link(log) corr(exchangeable) id(i)",
    "xtgee y, family(poisson) link(log) corr(exchangeable) id(i)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
