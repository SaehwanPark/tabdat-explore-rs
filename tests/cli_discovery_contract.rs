#![forbid(unsafe_code)]

use std::process::Command;

fn tabdat_bin() -> Command {
  Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
}

#[test]
fn cli_json_lists_commands() {
  let output = tabdat_bin()
    .args(["--json", "--list-commands"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "CommandCatalogResult");

  let commands = envelope["data"]["commands"]
    .as_array()
    .expect("commands array");
  assert_eq!(commands.len(), 81);

  // Check alphabetical order
  let names: Vec<&str> = commands
    .iter()
    .map(|c| c["name"].as_str().unwrap())
    .collect();
  for window in names.windows(2) {
    assert!(window[0] < window[1]);
  }

  // Check specific entries
  assert!(names.contains(&"summarize"));
  assert!(names.contains(&"lincom"));
  assert!(names.contains(&"test"));
  assert!(names.contains(&"ttest"));

  let summarize_entry = commands.iter().find(|c| c["name"] == "summarize").unwrap();
  assert_eq!(summarize_entry["help_topic"], "summarize");

  let lincom_entry = commands.iter().find(|c| c["name"] == "lincom").unwrap();
  assert!(lincom_entry["help_topic"].is_null());
}

#[test]
fn cli_list_commands_requires_json() {
  let output = tabdat_bin()
    .arg("--list-commands")
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("tabdat: error: --list-commands requires --json"));
}

#[test]
fn cli_json_lists_command_effects() {
  let output = tabdat_bin()
    .args(["--json", "--list-command-effects"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "CommandEffectCatalogResult");

  let commands = envelope["data"]["commands"]
    .as_array()
    .expect("commands array");
  assert_eq!(commands.len(), 81);

  let summarize_entry = commands.iter().find(|c| c["name"] == "summarize").unwrap();
  assert_eq!(summarize_entry["effects"], serde_json::json!(["read"]));

  let generate_entry = commands.iter().find(|c| c["name"] == "generate").unwrap();
  assert_eq!(
    generate_entry["effects"],
    serde_json::json!(["read", "write"])
  );

  let histogram_entry = commands.iter().find(|c| c["name"] == "histogram").unwrap();
  assert_eq!(
    histogram_entry["effects"],
    serde_json::json!(["read", "plot"])
  );

  let run_entry = commands.iter().find(|c| c["name"] == "run").unwrap();
  assert_eq!(
    run_entry["effects"],
    serde_json::json!(["read", "write", "control", "plot"])
  );
}

#[test]
fn cli_list_command_effects_requires_json() {
  let output = tabdat_bin()
    .arg("--list-command-effects")
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("tabdat: error: --list-command-effects requires --json"));
}

#[test]
fn cli_json_describe_command_success() {
  let output = tabdat_bin()
    .args(["--json", "--describe-command", "summarize"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "CommandSchemaResult");

  let data = &envelope["data"];
  assert_eq!(data["name"], "summarize");
  assert_eq!(data["syntax"], "summarize [varlist]");
  assert_eq!(data["help_topic"], "summarize");
  assert_eq!(
    data["arguments"],
    serde_json::json!([{"name": "variables", "required": false}])
  );
  assert_eq!(data["options"], serde_json::json!([]));
}

#[test]
fn cli_json_describe_command_unknown() {
  let output = tabdat_bin()
    .args(["--json", "--describe-command", "does-not-exist"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));

  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("Error: unknown command name: does-not-exist"));

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["error"]["type"], "TabDatError");
  assert_eq!(
    envelope["error"]["message"],
    "unknown command name: does-not-exist"
  );
}

#[test]
fn cli_describe_command_requires_json() {
  let output = tabdat_bin()
    .args(["--describe-command", "summarize"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("tabdat: error: --describe-command requires --json"));
}

#[test]
fn cli_json_help_topic_success() {
  let output = tabdat_bin()
    .args(["--json", "--help-topic", "summarize"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "HelpTopicResult");

  let data = &envelope["data"];
  assert_eq!(data["help_topic"], "summarize");
  assert!(
    data["text"]
      .as_str()
      .unwrap()
      .contains("Show descriptive statistics")
  );
}

#[test]
fn cli_json_help_topic_case_insensitive() {
  let output = tabdat_bin()
    .args(["--json", "--help-topic", "SuMmArIzE"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["data"]["help_topic"], "summarize");
}

#[test]
fn cli_json_help_topic_unknown() {
  let output = tabdat_bin()
    .args(["--json", "--help-topic", "does-not-exist"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));

  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("Error: unknown help topic: does-not-exist"));

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["error"]["type"], "TabDatError");
  assert_eq!(
    envelope["error"]["message"],
    "unknown help topic: does-not-exist"
  );
}

#[test]
fn cli_json_help_topic_blank() {
  let output = tabdat_bin()
    .args(["--json", "--help-topic", "   "])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));

  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("Error: help topic cannot be empty"));

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["error"]["message"], "help topic cannot be empty");
}

#[test]
fn cli_help_topic_requires_json() {
  let output = tabdat_bin()
    .args(["--help-topic", "summarize"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("tabdat: error: --help-topic requires --json"));
}

#[test]
fn cli_json_explain_success() {
  let output = tabdat_bin()
    .args(["--json", "--explain", "-c", "summarize age"])
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["result_type"], "CommandExplainResult");
  assert_eq!(
    envelope["data"],
    serde_json::json!({
      "command_name": "summarize",
      "execution": "not_run"
    })
  );
}

#[test]
fn cli_json_explain_parse_error() {
  let output = tabdat_bin()
    .args(["--json", "--explain", "-c", "not_a_tabdat_command"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));

  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("Error: unknown command: not_a_tabdat_command"));

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["error"]["type"], "ParseError");
  assert!(
    envelope["error"]["message"]
      .as_str()
      .unwrap()
      .contains("unknown command")
  );
}

#[test]
fn cli_explain_requires_json() {
  let output = tabdat_bin()
    .args(["--explain", "-c", "count"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("tabdat: error: --explain requires --json"));
}

#[test]
fn cli_explain_requires_exactly_one_command() {
  let output1 = tabdat_bin()
    .args(["--json", "--explain"])
    .output()
    .expect("binary should execute");
  assert_eq!(output1.status.code(), Some(2));
  let stderr1 = String::from_utf8(output1.stderr).expect("utf-8");
  assert!(stderr1.contains("tabdat: error: --explain requires exactly one -c/--command"));

  let output2 = tabdat_bin()
    .args(["--json", "--explain", "-c", "count", "-c", "status"])
    .output()
    .expect("binary should execute");
  assert_eq!(output2.status.code(), Some(2));
  let stderr2 = String::from_utf8(output2.stderr).expect("utf-8");
  assert!(stderr2.contains("tabdat: error: --explain requires exactly one -c/--command"));
}

#[test]
fn cli_json_batch_command_parse_error() {
  let output = tabdat_bin()
    .args(["--json", "-c", "not_a_valid_command"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));

  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("Error: unknown command: not_a_valid_command"));

  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["error"]["type"], "ParseError");
}
