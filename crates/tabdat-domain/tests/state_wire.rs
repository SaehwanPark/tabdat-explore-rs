#![forbid(unsafe_code)]

use tabdat_domain::*;

#[test]
fn domain_exchange_is_versioned_and_lossless() {
  let command = Command::Head {
    limit: RowLimit::new("00000000000000000000000005").unwrap(),
  };
  let value = serde_json::to_value(DomainEnvelope::new(command.clone())).unwrap();
  assert_eq!(
    value,
    serde_json::json!({"schema_version": 1, "data": {"kind": "head", "payload": {"limit": "5"}}})
  );
  assert_eq!(
    serde_json::from_value::<DomainEnvelope<Command>>(value)
      .unwrap()
      .data,
    command
  );
  assert_eq!(
    serde_json::to_value(DomainEnvelope::new(ExecutionResult::Count(CountResult {
      row_count: 7
    })))
    .unwrap(),
    serde_json::json!({"schema_version":1,"data":{"kind":"count","payload":{"row_count":7}}})
  );
  assert!(
    serde_json::from_str::<DomainEnvelope<Command>>(
      r#"{"schema_version":2,"data":{"kind":"count"}}"#
    )
    .is_err()
  );
  assert!(serde_json::from_str::<RowLimit>(r#""-1""#).is_err());
  for cell in [
    CellValue::SignedInteger(i128::MIN),
    CellValue::UnsignedInteger(u128::MAX),
    CellValue::Decimal {
      width: 38,
      scale: 8,
      value: i128::MIN,
    },
    CellValue::Bytes(vec![0xff, 0x00]),
  ] {
    let json = serde_json::to_string(&cell).unwrap();
    assert_eq!(serde_json::from_str::<CellValue>(&json).unwrap(), cell);
  }
}

#[test]
fn old_states_default_missing_report_registry_and_unknown_estimator_without_guessing() {
  let empty = serde_json::to_value(ExecutionState::default()).unwrap();
  assert!(empty.get("stored_estimates").is_none());
  let decoded: ExecutionState = serde_json::from_value(empty).unwrap();
  assert!(decoded.stored_estimates.is_empty());
  let problem = tabdat_stats::EstimationProblem::new(
    "y",
    vec!["x".into()],
    vec![1.0, 3.0, 2.0, 5.0],
    vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]],
  )
  .unwrap();
  let state = LinearRegressionState {
    estimator: Some("ols".into()),
    outcome: "y".into(),
    predictors: vec!["x".into()],
    predictor_design: problem.design_matrix.clone(),
    include_intercept: true,
    result: tabdat_stats::fit_least_squares(&problem).unwrap(),
  };
  let mut old = serde_json::to_value(state).unwrap();
  old.as_object_mut().unwrap().remove("estimator");
  let decoded: LinearRegressionState = serde_json::from_value(old.clone()).unwrap();
  assert_eq!(decoded.estimator, None);
  assert_eq!(serde_json::to_value(decoded).unwrap(), old);
  for (request, effect) in [
    (EstimatesCommand::Store("a".into()), CommandEffect::Control),
    (
      EstimatesCommand::Table(vec!["a".into()]),
      CommandEffect::Read,
    ),
    (
      EstimatesCommand::Drop(vec!["a".into()]),
      CommandEffect::Control,
    ),
  ] {
    let command = Command::Estimates { command: request };
    assert_eq!(command.name(), "estimates");
    assert!(command.requirements().is_empty());
    assert_eq!(command.effects(), &[effect]);
  }
}

#[test]
fn state_has_one_optional_model_and_panel_contract() {
  let mut state = ExecutionState::default();
  assert!(state.estimation.is_none());
  assert!(state.active_dataset.is_none());
  let panel = PanelMetadata::new("id", "time").unwrap();
  assert_eq!(panel.id_variable(), "id");
  assert!(PanelMetadata::new("id", "id").is_err());
  assert!(
    serde_json::from_str::<PanelMetadata>(r#"{"id_variable":"id","time_variable":"id"}"#).is_err()
  );
  state.panel = Some(panel);
  let json = serde_json::to_string(&DomainEnvelope::new(state.clone())).unwrap();
  assert_eq!(
    serde_json::from_str::<DomainEnvelope<ExecutionState>>(&json)
      .unwrap()
      .data,
    state
  );
}

#[test]
fn scalar_nonfinite_values_are_not_confused_with_null() {
  for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0] {
    let wire = serde_json::to_string(&CellValue::Float(value)).unwrap();
    let CellValue::Float(decoded) = serde_json::from_str(&wire).unwrap() else {
      panic!("lost float tag")
    };
    if value.is_nan() {
      assert!(decoded.is_nan());
    } else {
      assert_eq!(decoded.to_bits(), value.to_bits());
    }
  }
}

#[test]
fn located_errors_keep_typed_sources_and_versioned_diagnostics() {
  use std::error::Error;
  let error = TabDatError::from(RuntimeError::ScriptError(ScriptError::new(
    "job.td", 7, "failed",
  )));
  assert!(error.source().is_some());
  let diagnostic = Diagnostic::from(&error);
  assert_eq!(diagnostic.category, ErrorCategory::Script);
  assert_eq!(diagnostic.line, Some(7));
  assert_eq!(diagnostic.message, "job.td:7: failed");
  let wire = serde_json::to_string(&DomainEnvelope::new(diagnostic.clone())).unwrap();
  assert_eq!(
    serde_json::from_str::<DomainEnvelope<Diagnostic>>(&wire)
      .unwrap()
      .data,
    diagnostic
  );
}

#[test]
fn effects_and_capabilities_are_typed_and_recursive() {
  let sql = Command::Sql {
    command: SqlCommand {
      query: "select 1".into(),
      into: None,
    },
  };
  // SQL is conservatively read/write at the command level, matching Python discovery.
  assert_eq!(sql.effects(), &[CommandEffect::Read, CommandEffect::Write]);
  assert_eq!(
    Command::Help { topic: None }.requirements(),
    Vec::<Capability>::new()
  );
  let child = Command::BayesPrefix {
    command: BayesPrefixCommand {
      command: Box::new(Command::Regress {
        command: RegressCommand {
          outcome: "y".into(),
          predictors: vec!["x".into()],
          estimator: RegressEstimator::Ols,
          robust: false,
          cluster_variable: None,
          include_intercept: true,
          weight_variable: None,
        },
      }),
      draws: None,
      burnin: None,
      chains: None,
      thin: None,
      seed: None,
      priors: vec![],
    },
  };
  assert!(child.requirements().contains(&Capability::Bayesian));
  assert!(child.requirements().contains(&Capability::Data));
  assert_eq!(child.name(), "bayes");
}
