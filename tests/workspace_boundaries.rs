#![forbid(unsafe_code)]

use std::process::Command;

#[test]
fn workspace_dependencies_keep_contracts_and_surfaces_backend_independent() {
  let output = Command::new(env!("CARGO"))
    .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
    .output()
    .unwrap();
  assert!(output.status.success());
  let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  let expected = [
    "tabdat-domain",
    "tabdat-language",
    "tabdat-execution",
    "tabdat-data",
    "tabdat-io",
    "tabdat-stats",
    "tabdat-reporting",
    "tabdat-shell",
    "tabdat-mcp",
    "tabdat-runtime",
  ];
  for name in expected {
    let package = metadata["packages"]
      .as_array()
      .unwrap()
      .iter()
      .find(|p| p["name"] == name)
      .unwrap_or_else(|| panic!("missing {name}"));
    let manifest = std::path::Path::new(package["manifest_path"].as_str().unwrap());
    let source = std::fs::read_to_string(manifest.parent().unwrap().join("src/lib.rs")).unwrap();
    assert!(source.contains("#![forbid(unsafe_code)]"), "{name}");
    let dependencies = package["dependencies"]
      .as_array()
      .unwrap()
      .iter()
      .filter(|d| d["kind"].is_null())
      .map(|d| d["name"].as_str().unwrap())
      .collect::<Vec<_>>();
    if name == "tabdat-domain" {
      assert!(
        dependencies
          .iter()
          .all(|d| ["serde", "tabdat-stats"].contains(d))
      );
    }
    if [
      "tabdat-language",
      "tabdat-io",
      "tabdat-reporting",
      "tabdat-stats",
    ]
    .contains(&name)
    {
      assert!(
        !dependencies.iter().any(|d| [
          "duckdb",
          "tabdat-runtime",
          "tabdat-execution",
          "tabdat-data"
        ]
        .contains(d)),
        "{name}: {dependencies:?}"
      );
    }
    if name == "tabdat-execution" {
      assert!(!dependencies.contains(&"duckdb"));
    }
  }
}

#[test]
fn reporting_and_mcp_share_typed_domain_success_without_a_server() {
  let result = tabdat_domain::ExecutionResult::Count(tabdat_domain::CountResult { row_count: 3 });
  let cli = tabdat_reporting::json_results::serialize_execution_result(&result).unwrap();
  let reply = tabdat_mcp::ToolReply::Executed(Box::new(result));
  assert_eq!(reply.execution_json().unwrap(), cli);
  let mut session = tabdat_execution::Session::new();
  let reply = tabdat_mcp::dispatch(
    &mut session,
    tabdat_mcp::ToolRequest::Explain {
      command: "count".into(),
    },
  )
  .unwrap();
  assert!(matches!(reply, tabdat_mcp::ToolReply::Explained(_)));
  assert_eq!(session.state(), tabdat_domain::ExecutionState::default());
  assert!(session.initialized_capabilities().is_empty());
}
