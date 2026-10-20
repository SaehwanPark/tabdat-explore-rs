use tabdat_domain::{Command, PredictKind, XtRegEstimator};
use tabdat_language::parse_command;

#[test]
fn explicit_fe_keys_and_prediction_kinds_preserve_legacy_forms() {
  for (text, robust, cluster) in [
    ("xtreg y x z, fe id(person) time(year)", false, None),
    ("xtreg y x z, fe id(person) time(year) robust", true, None),
    (
      "xtreg y x z, fe id(person) time(year) cluster(person)",
      false,
      Some("person"),
    ),
  ] {
    let Command::XtReg { command } = parse_command(text).unwrap() else {
      panic!("wrong command");
    };
    assert_eq!(command.estimator, XtRegEstimator::FixedEffects);
    assert_eq!(command.predictors, ["x", "z"]);
    assert_eq!(command.id_variable.as_deref(), Some("person"));
    assert_eq!(command.time_variable.as_deref(), Some("year"));
    assert_eq!(command.robust, robust);
    assert_eq!(command.cluster_variable.as_deref(), cluster);
  }
  for (text, kind) in [
    ("predict p, xbu", PredictKind::FixedEffectsResponse),
    ("predict p, u", PredictKind::FixedEffect),
    ("predict p, xb", PredictKind::Xb),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!("wrong command");
    };
    assert_eq!(command.kind, kind);
  }
  let old = parse_command("xtreg y x, fe").unwrap();
  let json = serde_json::to_value(old).unwrap();
  assert!(json.to_string().find("id_variable").is_none());
  assert!(parse_command("xtreg y x, re").is_ok());
}
#[test]
fn malformed_fe_keys_and_conflicting_prediction_modes_are_rejected() {
  for text in [
    "xtreg y x, fe id(i)",
    "xtreg y x, fe time(t)",
    "xtreg y x, fe id(i) time(i)",
    "xtreg y x, fe id(i i2) time(t)",
    "xtreg y x, fe id(i) time(t t2)",
    "xtreg y x, fe id() time(t)",
    "xtreg y x, fe id(i) id(j) time(t)",
    "xtreg y x, fe id(i) time(t) time(q)",
    "predict p, xb xbu",
    "predict p, u residuals",
    "predict p, u pr",
    "predict p, u(1)",
    "predict p, xbu(1)",
  ] {
    assert!(parse_command(text).is_err(), "accepted {text}");
  }
}
