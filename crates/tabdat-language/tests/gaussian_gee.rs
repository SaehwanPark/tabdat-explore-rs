use tabdat_domain::Command;
use tabdat_language::parse_command;

#[test]
fn gaussian_identity_exchangeable_requires_explicit_group() {
  let Command::XtGee { command } =
    parse_command("xtgee y x z, family(gaussian) link(identity) corr(exchangeable) id(commune)")
      .unwrap()
  else {
    panic!()
  };
  assert_eq!(command.outcome, "y");
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.entity_variable, "commune");
  assert!(
    parse_command("xtgee y x, id(i) corr(EXCHANGEABLE) link(IDENTITY) family(GAUSSIAN)").is_ok()
  );
}

#[test]
fn incomplete_or_unsupported_gee_forms_are_rejected() {
  for text in [
    "xtgee y x",
    "xtgee y, family(gaussian) link(identity) corr(exchangeable) id(i)",
    "xtgee y x, link(identity) corr(exchangeable) id(i)",
    "xtgee y x, family(poisson) link(identity) corr(exchangeable) id(i)",
    "xtgee y x, family(gaussian) link(log) corr(exchangeable) id(i)",
    "xtgee y x, family(gaussian) link(identity) corr(independence) id(i)",
    "xtgee y x, family(gaussian) link(identity) corr(exchangeable)",
    "xtgee y x, family(gaussian) link(identity) corr(exchangeable) id(i j)",
    "xtgee y x, family(gaussian) family(gaussian) link(identity) corr(exchangeable) id(i)",
    "xtgee y x, family(gaussian) link(identity) corr(exchangeable) id(i) robust",
    "xtgee y x, family(gaussian) link(identity) corr(exchangeable) id(i) time(t)",
    "xtgee y x if z>0, family(gaussian) link(identity) corr(exchangeable) id(i)",
  ] {
    assert!(parse_command(text).is_err(), "accepted {text}");
  }
}
