use tabdat_domain::{Command, PredictKind, StregDistribution};
use tabdat_language::parse_command;

#[test]
fn weibull_prediction_flags_are_explicit_exclusive_and_not_probabilities() {
  for (flag, kind) in [
    ("hazard", PredictKind::Hazard),
    ("cumhaz", PredictKind::CumulativeHazard),
    ("survival", PredictKind::Survival),
  ] {
    let Command::Predict { command } = parse_command(&format!("predict result, {flag}")).unwrap()
    else {
      panic!("predict")
    };
    assert_eq!(command.kind, kind);
    assert_eq!(command.target_variable, "result");
    assert!(!command.interval && !command.std && command.count_cutoff_variable.is_none());
    for other in [
      "xb",
      "residuals",
      "pr",
      "mean",
      "n",
      "pmf(0)",
      "cdf(k)",
      "mills",
      "ycond",
      "yexpected",
      "hazard",
      "cumhaz",
      "survival",
    ] {
      if other == flag {
        continue;
      }
      assert!(
        parse_command(&format!("predict p, {flag} {other}")).is_err(),
        "{flag} {other}"
      );
    }
    for option in ["interval", "level(90)", "std", "saving(file)", "outcome(1)"] {
      assert!(
        parse_command(&format!("predict p, {flag} {option}")).is_err(),
        "{flag} {option}"
      );
    }
    for value in ["=true", "(1)", "(time)"] {
      assert!(parse_command(&format!("predict p, {flag}{value}")).is_err());
    }
  }
}

#[test]
fn weibull_ast_preserves_explicit_failure_design_and_covariance_options() {
  let Command::Streg { command } =
    parse_command("streg time x z, failure(event) dist(weibull) robust noconstant").unwrap()
  else {
    panic!("streg")
  };
  assert_eq!(command.time_variable, "time");
  assert_eq!(command.failure_variable, "event");
  assert_eq!(command.predictors, ["x", "z"]);
  assert_eq!(command.distribution, StregDistribution::Weibull);
  assert!(command.robust && !command.include_intercept && command.cluster_variable.is_none());
}
