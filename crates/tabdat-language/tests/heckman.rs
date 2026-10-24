use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;

#[test]
fn explicit_twostep_is_distinct_from_deferred_default_and_shared_noconstant() {
  for (text, method, intercept) in [
    ("heckman y x, selectdep(d) select(x z)", false, true),
    ("heckman y x, selectdep(d) select(x z) twostep", true, true),
    (
      "HECKMAN `y var` x, selectdep(d) select(z) twostep noconstant",
      true,
      false,
    ),
  ] {
    let Command::Heckman { command } = parse_command(text).unwrap() else {
      panic!()
    };
    assert_eq!(command.twostep, method);
    assert_eq!(command.include_intercept, intercept);
  }
  for text in [
    "heckman y x, selectdep(d) select(z) twostep(1)",
    "heckman y, selectdep(d) select(z) twostep",
    "heckman y x, selectdep(d) select() twostep",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}

#[test]
fn selection_point_kinds_are_typed_flag_only_and_mutually_exclusive() {
  for (name, kind) in [
    ("mills", PredictKind::SelectionMills),
    ("ycond", PredictKind::SelectionConditionalMean),
    ("yexpected", PredictKind::SelectionObservedContribution),
  ] {
    let Command::Predict { command } = parse_command(&format!("predict p, {name}")).unwrap() else {
      panic!()
    };
    assert_eq!(command.kind, kind);
    assert!(parse_command(&format!("predict p, {name}(1)")).is_err());
    for other in [
      "xb",
      "pr",
      "mean",
      "n",
      "residuals",
      "mills",
      "ycond",
      "yexpected",
      "pmf(1)",
      "xbu",
      "u",
      "cdf(k)",
    ] {
      if other != name {
        assert!(
          parse_command(&format!("predict p, {name} {other}")).is_err(),
          "{name}/{other}"
        );
      }
    }
  }
}
