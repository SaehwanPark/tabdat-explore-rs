use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;
#[test]
fn bounded_count_kinds_are_explicit_and_exclusive() {
  for (text, kind) in [
    ("predict p, pmf(0)", PredictKind::CountPmf(0)),
    ("predict p, cdf(99)", PredictKind::CountCdf(99)),
    ("predict p, pmf(100000)", PredictKind::CountPmf(100000)),
    ("predict p, pmf(k)", PredictKind::CountPmfVariable),
    (
      "predict p, cdf(`count cutoff`)",
      PredictKind::CountCdfVariable,
    ),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!("predict")
    };
    assert_eq!(command.kind, kind);
    assert_eq!(
      command.count_cutoff_variable,
      match kind {
        PredictKind::CountPmfVariable => Some("k".to_owned()),
        PredictKind::CountCdfVariable => Some("count cutoff".to_owned()),
        _ => None,
      }
    );
  }
  for suffix in [
    "pmf",
    "cdf(true)",
    "pmf(-1)",
    "pmf(100001)",
    "cdf(1.5)",
    "pmf(0) cdf(1)",
    "pmf(0) xb",
    "cdf(1) pr",
    "cdf(1) n",
    "pmf(0) pmf(0)",
    "cdf(0) interval",
    "pmf(0) std",
    "cdf(0) saving(out)",
  ] {
    assert!(
      parse_command(&format!("predict p, {suffix}")).is_err(),
      "{suffix}"
    );
  }
}
