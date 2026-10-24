use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;
#[test]
fn fixed_lower_truncation_requires_limit_and_explicit_supported_design() {
  let Command::Truncreg { command } = parse_command("truncreg y x z, ll(0.8) noconstant").unwrap()
  else {
    panic!("truncreg");
  };
  assert_eq!(command.outcome, "y");
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.lower_limit, "0.8");
  assert!(!command.include_intercept);
  let Command::Predict { command } = parse_command("predict conditional, mean").unwrap() else {
    panic!("predict");
  };
  assert_eq!(command.kind, PredictKind::TobitMean);
  assert!(matches!(
    parse_command("truncreg y x, ll(-0.6)").unwrap(),
    Command::Truncreg { .. }
  ));
}
#[test]
fn malformed_and_unsupported_variants_are_not_silently_tobit_or_weights() {
  for text in [
    "truncreg y x",
    "truncreg y, ll(0)",
    "truncreg y x, ll(0) ll(1)",
    "truncreg y x, ll(0) noconstant noconstant",
    "truncreg y x, ll(0) noconstant(1)",
    "truncreg y x, ll(0) ul(2)",
    "truncreg y x, ll(0) robust",
    "truncreg y x [pw=w], ll(0)",
    "truncreg y x, ll(0) cluster(id)",
    "truncreg y x, ll(\"NaN\")",
    "truncreg y x, ll(\"Inf\")",
    "truncreg y x if x>0, ll(0)",
    "truncreg=y x, ll(0)",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
