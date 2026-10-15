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
  assert_eq!(lincom_entry["help_topic"], "lincom");
  let test_entry = commands
    .iter()
    .find(|entry| entry["name"] == "test")
    .unwrap();
  assert_eq!(test_entry["help_topic"], "test");
  let ttest_entry = commands
    .iter()
    .find(|entry| entry["name"] == "ttest")
    .unwrap();
  assert_eq!(ttest_entry["help_topic"], "ttest");
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
  assert_eq!(data["syntax"], "summarize [varlist] [if expr] [, detail]");
  assert_eq!(data["help_topic"], "summarize");
  assert_eq!(
    data["arguments"],
    serde_json::json!([{"name": "variables", "required": false}])
  );
  assert_eq!(
    data["options"],
    serde_json::json!([{"name": "detail", "required": false}])
  );
}

#[test]
fn core_conditional_discovery_matches_descriptors_and_links_bounded_help() {
  for (name, syntax, arguments, options) in [
    ("count", "count [if expr]", serde_json::json!([]), vec![]),
    (
      "summarize",
      "summarize [varlist] [if expr] [, detail]",
      serde_json::json!([{"name": "variables", "required": false}]),
      vec!["detail"],
    ),
    (
      "regress",
      "regress y xvars [if expr] [, robust cluster(var) noconstant wls(var) gls(var)]",
      serde_json::json!([
        {"name": "depvar", "required": true}, {"name": "indepvars", "required": true}
      ]),
      vec!["robust", "cluster", "noconstant", "wls", "gls"],
    ),
    (
      "tabulate",
      "tabulate rowvar [colvar] [if expr] [, row col missing nolabel] | tabulate [if expr], rows(rowvar) [columns(colvar)] [row col missing nolabel]",
      serde_json::json!([{"name": "variable", "required": true}]),
      vec![
        "missing", "values", "stat", "row", "col", "nolabel", "rows", "columns",
      ],
    ),
  ] {
    let output = tabdat_bin()
      .args(["--json", "--describe-command", &name.to_ascii_uppercase()])
      .output()
      .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["result_type"], "CommandSchemaResult");
    let data = &envelope["data"];
    assert_eq!(data["name"], name);
    assert_eq!(data["help_topic"], name);
    assert_eq!(data["syntax"], syntax);
    assert_eq!(data["arguments"], arguments);
    let expected: Vec<_> = options
      .into_iter()
      .map(|option| serde_json::json!({"name": option, "required": false}))
      .collect();
    assert_eq!(data["options"], serde_json::json!(expected));
    let help = tabdat_bin()
      .args(["--json", "--help-topic", name])
      .output()
      .unwrap();
    assert!(help.status.success());
    let topic: serde_json::Value = serde_json::from_slice(&help.stdout).unwrap();
    let text = topic["data"]["text"].as_str().unwrap();
    assert!(text.contains("bounded"), "{name}");
    if name == "tabulate" {
      assert!(text.contains("values()`/`stat()` aggregation"));
      assert!(text.contains("parser-only/unsupported"));
    }
  }
}

#[test]
fn conditional_discovery_examples_preview_without_data_and_reject_malformed_syntax() {
  for (name, source) in [
    ("count", "count if eligible == 1"),
    ("summarize", "summarize x if eligible == 1"),
    ("regress", "regress y x if eligible == 1, robust"),
    (
      "tabulate",
      "tabulate x y if eligible == 1, row col missing nolabel",
    ),
    (
      "tabulate",
      "tabulate if eligible == 1, rows(x) columns(y) row col",
    ),
  ] {
    let output = tabdat_bin()
      .args(["--json", "--explain", "-c", source])
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["result_type"], "CommandExplainResult");
    assert_eq!(envelope["data"]["command_name"], name);
    assert_eq!(envelope["data"]["execution"], "not_run");
  }
  for source in [
    "count if",
    "summarize x if",
    "regress y x if",
    "regress y",
    "regress y if eligible == 1",
    "tabulate x if",
  ] {
    let output = tabdat_bin()
      .args(["--json", "--explain", "-c", source])
      .output()
      .unwrap();
    assert_eq!(output.status.code(), Some(1), "{source}");
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["error"]["type"], "ParseError");
  }
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
fn cli_json_post_estimation_help_topics_and_discovery_links_are_available() {
  for (topic, phrase) in [
    ("lincom", "Student-t"),
    ("test", "Wald F"),
    ("ttest", "Student-t"),
    ("predict", "Installed syntax:"),
  ] {
    let output = tabdat_bin()
      .args(["--json", "--help-topic", &topic.to_ascii_uppercase()])
      .output()
      .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["result_type"], "HelpTopicResult");
    assert_eq!(envelope["data"]["help_topic"], topic);
    assert!(envelope["data"]["text"].as_str().unwrap().contains(phrase));
    let output = tabdat_bin()
      .args(["--json", "--describe-command", topic])
      .output()
      .unwrap();
    assert!(output.status.success());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["data"]["help_topic"], topic);
  }
}

#[test]
fn ttest_discovery_describes_the_installed_comparisons_and_variance_flags() {
  let output = tabdat_bin()
    .args(["--json", "--describe-command", "TTEST"])
    .output()
    .unwrap();
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(envelope["data"]["help_topic"], "ttest");
  assert_eq!(
    envelope["data"]["syntax"],
    "ttest x == value | ttest x == y | ttest x, by(group) [welch | unequal]"
  );
  assert_eq!(
    envelope["data"]["options"],
    serde_json::json!([
      {"name": "by", "required": false},
      {"name": "welch", "required": false},
      {"name": "unequal", "required": false},
    ])
  );
}

#[test]
fn predict_discovery_uses_parser_spelling_without_promising_execution() {
  let output = tabdat_bin()
    .args(["--json", "--describe-command", "PREDICT"])
    .output()
    .unwrap();
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["data"]["help_topic"], "predict");
  assert_eq!(
    envelope["data"]["syntax"],
    "predict newvar [, xb | residuals | pr]"
  );
  assert_eq!(
    envelope["data"]["options"],
    serde_json::json!([
      {"name": "xb", "required": false},
      {"name": "residuals", "required": false},
      {"name": "pr", "required": false}
    ])
  );
  let output = tabdat_bin()
    .args(["--json", "--help-topic", "PREDICT"])
    .output()
    .unwrap();
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
  let topic = include_str!("../crates/tabdat-reporting/src/help/topics/predict.md");
  assert_eq!(envelope["data"]["text"], topic);
  assert!(topic.contains("Only xb and residuals"));
  assert!(topic.contains("reference/parser-only"));
  assert!(topic.contains("`probabilities` is not an accepted option spelling"));
  for source in [
    "predict fitted",
    "predict fitted, xb",
    "predict residual, residuals",
    "predict probability, pr",
    "predict spillover, spatial_lag",
    "predict posterior, posterior_predictive std interval level(90)",
    "predict posterior, posterior_predictive saving(draws.parquet)",
  ] {
    let output = tabdat_bin()
      .args(["--json", "--explain", "-c", source])
      .output()
      .unwrap();
    assert!(
      output.status.success(),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["data"]["command_name"], "predict");
    assert_eq!(envelope["data"]["execution"], "not_run");
  }
  for source in [
    "predict probability, probabilities",
    "predict both, xb residuals",
  ] {
    let output = tabdat_bin()
      .args(["--json", "--explain", "-c", source])
      .output()
      .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["error"]["type"], "ParseError");
  }
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
