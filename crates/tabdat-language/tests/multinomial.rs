use tabdat_language::{Command, PredictKind, parse_command};

#[test]
fn bounded_multinomial_specification_and_explicit_category_predictions_parse() {
  for (text, base, intercept, predictors) in [
    ("mlogit y x", None, true, vec!["x"]),
    (
      "mlogit y x z, baseoutcome(-2) noconstant",
      Some(-2),
      false,
      vec!["x", "z"],
    ),
    ("mlogit y", None, true, vec![]),
  ] {
    let Command::Mlogit { command } = parse_command(text).unwrap() else {
      panic!()
    };
    assert_eq!(command.outcome, "y");
    assert_eq!(command.predictors, predictors);
    assert_eq!(command.base_outcome, base);
    assert_eq!(command.include_intercept, intercept);
  }
  for (text, kind) in [
    ("predict p, pr outcome(1)", PredictKind::MultinomialPr(1)),
    ("predict b, xb outcome(-2)", PredictKind::MultinomialXb(-2)),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!()
    };
    assert_eq!(command.kind, kind);
  }
  let Command::Predict { command } = parse_command("predict p, pr").unwrap() else {
    panic!()
  };
  assert_eq!(command.kind, PredictKind::Pr);
}

#[test]
fn unsupported_estimator_inference_and_ambiguous_category_syntax_fail() {
  for text in [
    "mlogit",
    "mlogit y, noconstant",
    "mlogit y x, robust",
    "mlogit y x, cluster(g)",
    "mlogit y x, baseoutcome(1.5)",
    "mlogit y x, baseoutcome(9007199254740992)",
    "mlogit y x, baseoutcome(1) baseoutcome(2)",
    "mlogit y x, noconstant(1)",
    "predict p, outcome(1)",
    "predict p, residuals outcome(1)",
    "predict p, pr outcome(1) outcome(2)",
    "predict p, pr outcome(1.5)",
    "predict p, pr outcome(2) interval",
    "predict p, pr outcome(2) n",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}
