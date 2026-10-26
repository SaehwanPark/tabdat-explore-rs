use tabdat_domain::{Command, StregDistribution};
use tabdat_language::parse_command;

#[test]
fn gompertz_ast_preserves_names_design_and_explicit_inference() {
  for (text, robust, intercept) in [
    ("streg time x z, failure(event) dist(gompertz)", false, true),
    (
      "STREG time x z, failure(event) dist(GoMpErTz) robust noconstant",
      true,
      false,
    ),
  ] {
    let Command::Streg { command } = parse_command(text).unwrap() else {
      panic!("streg")
    };
    assert_eq!(command.time_variable, "time");
    assert_eq!(command.predictors, ["x", "z"]);
    assert_eq!(command.failure_variable, "event");
    assert_eq!(command.distribution, StregDistribution::Gompertz);
    assert_eq!(command.robust, robust);
    assert_eq!(command.include_intercept, intercept);
    assert!(command.cluster_variable.is_none());
  }
  let Command::Streg { command } =
    parse_command("streg `time name` `x name`, failure(`event name`) dist(gompertz) cluster(id)")
      .unwrap()
  else {
    panic!("streg")
  };
  assert_eq!(command.time_variable, "time name");
  assert_eq!(command.predictors, ["x name"]);
  assert_eq!(command.failure_variable, "event name");
  assert_eq!(command.cluster_variable.as_deref(), Some("id"));
}

#[test]
fn gompertz_retains_explicit_predictor_and_option_grammar_bounds() {
  for text in [
    "streg time, failure(event) dist(gompertz)",
    "streg time x, failure(event) dist(gompertz) robust cluster(id)",
    "streg time x, failure(event) dist(gompertz) dist(weibull)",
    "streg time x, failure(event) dist(gompertz) robust=true",
    "streg time x, failure(event) dist(gompertz) foo",
    "streg time x if x>0, failure(event) dist(gompertz)",
    "streg time x, failure(event) dist(gompertz) [weight=w]",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
