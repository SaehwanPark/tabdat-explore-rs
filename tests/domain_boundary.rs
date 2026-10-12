#![forbid(unsafe_code)]

#[test]
fn parser_and_runtime_use_identical_domain_contracts() {
  let command: tabdat_domain::Command = tabdat_language::parse_command("count").unwrap();
  let mut session = tabdat_runtime::Session::new();
  let error: tabdat_domain::RuntimeError = session.execute(command).unwrap_err();
  assert_eq!(
    error,
    tabdat_domain::RuntimeError::NoActiveDataset { command: "count" }
  );
  let result: tabdat_domain::ExecutionResult =
    tabdat_runtime::ExecutionResult::Count(tabdat_runtime::CountResult { row_count: 3 });
  assert_eq!(
    result,
    tabdat_domain::ExecutionResult::Count(tabdat_domain::CountResult { row_count: 3 })
  );
}
