use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;

#[test]
fn count_native_extensions_are_explicit_without_changing_bare_predict_default() {
  for text in ["poisson y", "poisson `count outcome`, robust"] {
    let Command::Poisson { command } = parse_command(text).unwrap() else {
      panic!("poisson")
    };
    assert!(command.predictors.is_empty());
    assert!(command.include_intercept);
  }
  for (text, kind) in [
    ("predict mu, n", PredictKind::MeanCount),
    ("predict index, xb", PredictKind::Xb),
    ("predict bare", PredictKind::Xb),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!("predict")
    };
    assert_eq!(command.kind, kind);
  }
  for text in [
    "predict bad, n pr",
    "predict bad, n xb",
    "predict bad, residuals n",
    "predict bad, n=true",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
  let Command::Poisson { command } = parse_command("poisson y x, offset(log_exposure)").unwrap()
  else {
    panic!("poisson")
  };
  assert_eq!(command.offset_variable.as_deref(), Some("log_exposure"));
  assert!(parse_command("poisson y x, exposure(time)").is_err());
  // The old count parser surface remains accepted; execution owns the explicit rejection.
  assert!(matches!(
    parse_command("poisson y x, cluster(id)").unwrap(),
    Command::Poisson { .. }
  ));
  // NB2's separately installed intercept-only extension (MIG-0018).
  let Command::Nbreg { command } = parse_command("nbreg y").unwrap() else {
    panic!("NB2")
  };
  assert!(command.predictors.is_empty() && command.include_intercept);
}
