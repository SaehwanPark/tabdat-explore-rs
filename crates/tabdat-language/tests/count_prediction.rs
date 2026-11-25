use tabdat_domain::{Command, PredictKind};
use tabdat_language::parse_command;

#[test]
fn count_diagnostic_residual_kinds_are_explicit_and_exclusive() {
  for (text, kind, wire_name) in [
    (
      "predict pearson_resid, pearson",
      PredictKind::PearsonResiduals,
      "pearson_residuals",
    ),
    (
      "predict deviance_resid, deviance",
      PredictKind::DevianceResiduals,
      "deviance_residuals",
    ),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!("predict")
    };
    assert_eq!(command.kind, kind);
    assert_eq!(
      serde_json::to_value(command.kind).unwrap(),
      serde_json::json!(wire_name)
    );
    assert_eq!(command.count_cutoff_variable, None);
    assert!(!command.interval);
  }

  for text in [
    "predict residual, pearson=true",
    "predict residual, deviance=false",
    "predict residual, pearson deviance",
    "predict residual, residuals pearson",
    "predict residual, residuals deviance",
    "predict residual, pearson n",
    "predict residual, deviance interval",
  ] {
    assert!(parse_command(text).is_err(), "{text}");
  }
}

#[test]
fn bounded_count_kinds_are_explicit_and_exclusive() {
  for (text, kind) in [
    ("predict p, pmf(0)", PredictKind::CountPmf(0)),
    (
      "predict p_se, pmf(2) se",
      PredictKind::CountPmfStandardError(2),
    ),
    ("predict p, cdf(99)", PredictKind::CountCdf(99)),
    (
      "predict p_se, cdf(7) se",
      PredictKind::CountCdfStandardError(7),
    ),
    ("predict p, pmf(100000)", PredictKind::CountPmf(100000)),
    ("predict p, pmf(k)", PredictKind::CountPmfVariable),
    (
      "predict p_se, pmf(k) se",
      PredictKind::CountPmfVariableStandardError,
    ),
    (
      "predict p, cdf(`count cutoff`)",
      PredictKind::CountCdfVariable,
    ),
    (
      "predict p_se, cdf(cutoff) se",
      PredictKind::CountCdfVariableStandardError,
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
        PredictKind::CountPmfVariableStandardError => Some("k".to_owned()),
        PredictKind::CountCdfVariable => Some("count cutoff".to_owned()),
        PredictKind::CountCdfVariableStandardError => Some("cutoff".to_owned()),
        _ => None,
      }
    );
  }
  for (text, kind, cutoff, level) in [
    (
      "predict pmf2, pmf(2) interval",
      PredictKind::CountPmf(2),
      None,
      "95.0",
    ),
    (
      "predict cdf2, cdf(2) interval level(90)",
      PredictKind::CountCdf(2),
      None,
      "90",
    ),
    (
      "predict pmf_row, pmf(k) interval",
      PredictKind::CountPmfVariable,
      Some("k"),
      "95.0",
    ),
    (
      "predict cdf_row, cdf(cutoff) interval level(80)",
      PredictKind::CountCdfVariable,
      Some("cutoff"),
      "80",
    ),
    (
      "predict future_count, n interval",
      PredictKind::MeanCount,
      None,
      "95.0",
    ),
    (
      "predict future_count, n interval level(90)",
      PredictKind::MeanCount,
      None,
      "90",
    ),
  ] {
    let Command::Predict { command } = parse_command(text).unwrap() else {
      panic!("predict")
    };
    assert_eq!(command.kind, kind);
    assert_eq!(command.count_cutoff_variable.as_deref(), cutoff);
    assert!(command.interval);
    assert_eq!(command.level, level);
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
    "pmf(0) interval se",
    "cdf(0) se interval",
    "pmf(0) level(95)",
    "cdf(0) interval level(NaN)",
    "cdf(0) interval level(100)",
    "n interval level(100)",
    "xb interval",
    "pmf(0) std",
    "cdf(0) saving(out)",
    "se",
    "pmf(2) se se",
    "pmf(2) se(value)",
  ] {
    assert!(
      parse_command(&format!("predict p, {suffix}")).is_err(),
      "{suffix}"
    );
  }
}
