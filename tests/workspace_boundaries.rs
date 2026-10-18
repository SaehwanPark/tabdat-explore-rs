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
fn mcp_execute_returns_the_same_owned_count_as_the_coordinator() {
  use std::io::Write;
  use tabdat_domain::{Command as DomainCommand, DataSource, ExecutionMode, ExecutionResult};
  struct OwnedCsv(std::path::PathBuf);
  impl Drop for OwnedCsv {
    fn drop(&mut self) {
      let _ = std::fs::remove_file(&self.0);
    }
  }
  let (fixture, mut file) = (0..1000)
    .find_map(|index| {
      let path = std::env::temp_dir().join(format!(
        "tabdat-phase2-mcp-{}-{index}.csv",
        std::process::id()
      ));
      match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
      {
        Ok(file) => Some((OwnedCsv(path), file)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
        Err(error) => panic!("fixture creation failed: {error}"),
      }
    })
    .expect("exclusive fixture path");
  file.write_all(b"x\n1\n2\n").unwrap();
  drop(file);
  let mut session = tabdat_execution::Session::new();
  session
    .execute(DomainCommand::Use {
      source: DataSource::LocalPath(fixture.0.to_str().unwrap().into()),
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: Some(true),
    })
    .unwrap();
  let expected = session.execute(DomainCommand::Count).unwrap();
  let reply = tabdat_mcp::dispatch(
    &mut session,
    tabdat_mcp::ToolRequest::Execute {
      command: "count".into(),
    },
  )
  .unwrap();
  let tabdat_mcp::ToolReply::Executed(result) = &reply else {
    panic!("not an execution reply")
  };
  assert_eq!(result.as_ref(), &expected);
  assert!(matches!(result.as_ref(), ExecutionResult::Count(count) if count.row_count == 2));
  assert_eq!(
    reply.execution_json(),
    tabdat_reporting::json_results::serialize_execution_result(&expected)
  );
}

#[test]
fn reporting_effects_preserve_all_pinned_python_entries() {
  let oracle: serde_json::Value =
    serde_json::from_str(include_str!("fixtures/python-command-effects-v1.json")).unwrap();
  let mut actual =
    serde_json::to_value(tabdat_reporting::catalog::command_effect_catalog_result()).unwrap();
  let entries = actual["commands"].as_array_mut().unwrap();
  let extension: Vec<_> = entries
    .iter()
    .filter(|entry| entry["name"] == "estimates")
    .collect();
  assert_eq!(
    extension,
    vec![&serde_json::json!({"name":"estimates","effects":["read","control"]})]
  );
  // MIG-0016 adds one native reporting command, not a change to pinned entries.
  entries.retain(|entry| entry["name"] != "estimates");
  assert_eq!(actual, oracle["data"]);
}

#[test]
fn mcp_tools_normalize_failures_without_eager_backend_initialization() {
  use tabdat_domain::{Diagnostic, ErrorCategory};
  let mut session = tabdat_execution::Session::new();
  for (command, category) in [
    ("count", ErrorCategory::Execution),
    ("not_a_command", ErrorCategory::Parse),
  ] {
    let error = tabdat_mcp::dispatch(
      &mut session,
      tabdat_mcp::ToolRequest::Execute {
        command: command.into(),
      },
    )
    .unwrap_err();
    assert_eq!(Diagnostic::from(&error).category, category);
    assert!(session.initialized_capabilities().is_empty());
  }
  assert!(
    serde_json::from_str::<tabdat_mcp::ToolRequest>(
      r#"{"tool":"execute","command":"count","unexpected":true}"#
    )
    .is_err()
  );
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
