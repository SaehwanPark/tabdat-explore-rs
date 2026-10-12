#![forbid(unsafe_code)]

use tabdat_domain::{Command, DomainEnvelope, EstimationState, ExecutionState};

#[test]
fn representative_commands_roundtrip_the_internal_wire_without_backend_use() {
  let oracle: serde_json::Value =
    serde_json::from_str(include_str!("fixtures/python-command-effects-v1.json")).unwrap();
  for input in [
    "count",
    "help status",
    "head 0005",
    "use data.parquet",
    "use https://example.invalid/data.parquet",
    "panel id time",
    r#"label define yesno 0 "no" 1 "yes""#,
    "label values x yesno",
    "generate z = x + 1",
    "replace x = 2 if x > 0",
    "assert x > 0",
    "recode x (1=2), replace",
    "regress y x, robust",
    "predict fitted, xb",
    "test x = 0",
    "lincom x + 1",
    "by id: count",
    "bayes: regress y x",
    "sql select 1",
    "xtreg y x, fe",
    "ivregress 2sls y x, endog(z) iv(w)",
  ] {
    let command =
      tabdat_language::parse_command(input).unwrap_or_else(|error| panic!("{input}: {error}"));
    let wire = serde_json::to_string(&DomainEnvelope::new(command.clone())).unwrap();
    let decoded: DomainEnvelope<Command> = serde_json::from_str(&wire).unwrap();
    assert_eq!(decoded.data, command, "{input}");
    let expected = oracle["data"]["commands"]
      .as_array()
      .unwrap()
      .iter()
      .find(|entry| entry["name"] == command.name())
      .unwrap();
    assert_eq!(
      serde_json::to_value(command.effects()).unwrap(),
      expected["effects"],
      "{input}"
    );
    assert!(!command.effects().is_empty());
    assert!(command.effects().windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
      command
        .requirements()
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    );
  }
}

#[test]
fn runtime_publishes_domain_state_without_initializing_for_reads() {
  let mut session = tabdat_runtime::Session::new();
  assert_eq!(session.state(), ExecutionState::default());
  assert!(session.execute(Command::Count).is_err());
  assert_eq!(session.state(), ExecutionState::default());
  // The family enum is the state API, not a backend-native model object.
  let state: Option<EstimationState> = session.state().estimation;
  assert!(state.is_none());
}
