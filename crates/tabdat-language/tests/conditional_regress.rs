use tabdat_domain::{
  Command, GenerateBinaryOperator, GenerateExpression, RegressCommand, RegressEstimator,
};
use tabdat_language::parse_command;

fn regress_query(source: &str) -> (RegressCommand, GenerateExpression) {
  match parse_command(source).unwrap() {
    Command::RegressQuery { command, condition } => (command, condition),
    other => panic!("expected RegressQuery, got {other:?}"),
  }
}

#[test]
fn regress_if_accepts_bounded_predicates_with_existing_options() {
  let (command, condition) = regress_query("regress cost age bmi if eligible == 1");
  assert_eq!(command.outcome, "cost");
  assert_eq!(command.predictors, ["age", "bmi"]);
  assert_eq!(command.estimator, RegressEstimator::Ols);
  assert_eq!(command.weight_variable, None);
  assert_eq!(command.cluster_variable, None);
  assert!(!command.robust);
  assert!(command.include_intercept);
  assert_eq!(
    condition,
    GenerateExpression::Binary {
      left: Box::new(GenerateExpression::Identifier("eligible".into())),
      operator: GenerateBinaryOperator::Equal,
      right: Box::new(GenerateExpression::Number("1".into())),
    }
  );

  let (command, _) = regress_query("regress y x if x > 1, robust");
  assert!(command.robust);

  let (command, _) = regress_query("regress y x if eligible == 1, wls(w) noconstant");
  assert_eq!(command.estimator, RegressEstimator::Wls);
  assert_eq!(command.weight_variable.as_deref(), Some("w"));
  assert!(!command.include_intercept);

  let (command, _) = regress_query("regress y x if eligible == 1, gls(sigma)");
  assert_eq!(command.estimator, RegressEstimator::Gls);
  assert_eq!(command.weight_variable.as_deref(), Some("sigma"));

  let (command, _) = regress_query("regress y x if g != 'A', cluster(g)");
  assert_eq!(command.cluster_variable.as_deref(), Some("g"));

  // Quoted commas stay inside the predicate; later commas still open options.
  let (command, _) = regress_query("regress y x if text == 'a,b', noconstant");
  assert!(!command.include_intercept);

  // Bare regress is unchanged.
  assert!(matches!(
    parse_command("regress y x").unwrap(),
    Command::Regress { .. }
  ));
}

#[test]
fn regress_if_rejects_incomplete_or_misplaced_condition_and_option_forms() {
  for (source, expected) in [
    ("regress y x if", "missing expression after if"),
    ("regress y x if x >", "incomplete expression after >"),
    ("regress y x if x > 1 if x < 4", "duplicate if clause"),
    (
      "regress y x, robust if x > 1",
      "option names must be identifiers",
    ),
    (
      "regress y x if x > 1,",
      "comma must be followed by at least one option",
    ),
    (
      "regress y x if x > 1, detail",
      "regress unsupported option: detail",
    ),
    (
      "regress y x if x > 1, robust cluster(g)",
      "regress cannot combine robust and cluster",
    ),
    (
      "regress y x if x > 1, cluster(a b)",
      "regress option cluster expects one variable",
    ),
    (
      "regress if x > 1",
      "regress expects syntax: regress <y> <xvars>",
    ),
    (
      "regress y if x > 1",
      "regress expects syntax: regress <y> <xvars>",
    ),
  ] {
    let error = parse_command(source).unwrap_err();
    assert_eq!(error.message(), expected, "{source}");
  }
}
