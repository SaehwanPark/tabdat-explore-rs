use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;

#[test]
fn lower_censored_fit_and_observed_mean_prediction_are_explicit() {
  assert!(parse_command("tobit y, ll(0)").is_err());
  let Command::Tobit { command } = parse_command("tobit y x1 x2, ll(-0.8) noconstant").unwrap()
  else {
    panic!("tobit");
  };
  assert_eq!(command.lower_limit, "-0.8");
  assert!(!command.include_intercept);
  assert!(!command.robust && command.cluster_variable.is_none() && command.upper_limit.is_none());
  let Command::Predict { command } = parse_command("predict expected, mean").unwrap() else {
    panic!("predict");
  };
  assert_eq!(command.kind, PredictKind::TobitMean);
  assert_eq!(command.target_variable, "expected");
  assert!(!command.interval && !command.std && command.saving.is_none());
  assert!(command.count_cutoff_variable.is_none());
}

#[test]
fn observed_mean_cannot_be_confused_with_other_prediction_kinds_or_options() {
  for kind in [
    "xb",
    "pr",
    "n",
    "pmf(0)",
    "cdf(k)",
    "xbu",
    "u",
    "residuals",
    "posterior_predictive",
  ] {
    assert!(
      parse_command(&format!("predict expected, mean {kind}")).is_err(),
      "{kind}"
    );
  }
  for option in [
    "mean(1)",
    "mean=1",
    "mean interval",
    "mean level(90)",
    "mean std",
    "mean saving(\"x.csv\")",
    "mean outcome(1)",
  ] {
    assert!(
      parse_command(&format!("predict expected, {option}")).is_err(),
      "{option}"
    );
  }
  // Legacy parser acceptance is retained; runtime must reject unsupported inference.
  for options in ["robust", "cluster(id)", "ul(10)"] {
    assert!(matches!(
      parse_command(&format!("tobit y x, ll(0) {options}")).unwrap(),
      Command::Tobit { .. }
    ));
  }
}
