use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tabdat_language::{
  ControlFlowDirective, ElseDirective, EndDirective, IfDirective, LetDirective, ScriptBlockState,
  ScriptCommand, ScriptContext, ScriptDirective, ScriptError, SeedDirective,
  evaluate_script_condition, expand_script_macros, parse_control_flow_directive, parse_script,
  parse_script_directive, read_script,
};

#[test]
fn script_contract_parses_commands_and_skips_comments() {
  let script = r#"
# Load primary dataset
use "data/patients.parquet"

# Clean columns
describe
count
"#;

  let commands = parse_script(script, Path::new("test.td")).unwrap();
  assert_eq!(
    commands,
    vec![
      ScriptCommand::new("use \"data/patients.parquet\"", 3),
      ScriptCommand::new("describe", 6),
      ScriptCommand::new("count", 7),
    ]
  );
}

#[test]
fn script_contract_groups_multiline_sql_commands() {
  let script = r#"
use patients.parquet
sql """
SELECT id, age, outcome
FROM active
WHERE age > 65
"""
count
"#;

  let commands = parse_script(script, Path::new("query.td")).unwrap();
  assert_eq!(commands.len(), 3);
  assert_eq!(commands[0], ScriptCommand::new("use patients.parquet", 2));
  assert_eq!(
    commands[1],
    ScriptCommand::new(
      "sql \"\"\"\nSELECT id, age, outcome\nFROM active\nWHERE age > 65\n\"\"\"",
      3
    )
  );
  assert_eq!(commands[2], ScriptCommand::new("count", 8));
}

#[test]
fn script_contract_rejects_unterminated_multiline_sql() {
  let script = "use patients.parquet\nsql \"\"\"\nSELECT * FROM active\n";
  let err = parse_script(script, Path::new("broken.td")).unwrap_err();
  assert_eq!(
    err.to_string(),
    "broken.td:2: sql multiline query is missing closing \"\"\""
  );
}

#[test]
fn script_contract_reads_files_from_disk_and_reports_io_errors() {
  let temp_dir = std::env::temp_dir().join(format!("tabdat_script_test_{}", std::process::id()));
  fs::create_dir_all(&temp_dir).unwrap();

  let script_path = temp_dir.join("valid.td");
  fs::write(&script_path, "describe\ncount\n").unwrap();

  let commands = read_script(&script_path).unwrap();
  assert_eq!(
    commands,
    vec![
      ScriptCommand::new("describe", 1),
      ScriptCommand::new("count", 2),
    ]
  );

  // Missing file
  let missing_path = temp_dir.join("non_existent.td");
  let err = read_script(&missing_path).unwrap_err();
  assert_eq!(err.message(), "script file not found");

  // Directory instead of file
  let err = read_script(&temp_dir).unwrap_err();
  assert_eq!(err.message(), "script path is a directory");

  // Invalid UTF-8 file
  let invalid_utf8_path = temp_dir.join("invalid_utf8.td");
  let mut invalid_bytes = b"describe\n".to_vec();
  invalid_bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
  invalid_bytes.extend_from_slice(b"\ncount\n");
  fs::write(&invalid_utf8_path, &invalid_bytes).unwrap();

  let err = read_script(&invalid_utf8_path).unwrap_err();
  assert_eq!(err.line(), 2);
  assert_eq!(err.message(), "script file must be UTF-8 text");

  // Cleanup
  let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn script_contract_expands_macros_and_validates_references() {
  let mut macros = HashMap::new();
  macros.insert("cohort".to_string(), "treatment".to_string());
  macros.insert("alpha".to_string(), "0.05".to_string());
  let context = ScriptContext::new(macros, None);

  let path = Path::new("analysis.td");
  let expanded = expand_script_macros(
    "summarize outcome if group == $cohort\nregress outcome treatment, level($alpha)",
    &context,
    path,
    5,
  )
  .unwrap();

  assert_eq!(
    expanded,
    "summarize outcome if group == treatment\nregress outcome treatment, level(0.05)"
  );

  // Undefined macro
  let err = expand_script_macros("use $undefined", &context, path, 10).unwrap_err();
  assert_eq!(
    err.to_string(),
    "analysis.td:10: undefined macro: undefined"
  );

  // Preserve non-macro dollar signs
  assert_eq!(
    expand_script_macros("echo $1 and $ and $$cohort", &context, path, 12).unwrap(),
    "echo $1 and $ and $treatment"
  );
}

#[test]
fn script_contract_parses_seed_and_let_directives() {
  let mut context = ScriptContext::empty();
  let path = Path::new("directives.td");

  // Seed
  let seed_dir = parse_script_directive("seed 42", &context, path, 1).unwrap();
  assert_eq!(
    seed_dir,
    Some(ScriptDirective::Seed(SeedDirective { value: 42 }))
  );

  // Let
  let let_dir = parse_script_directive("let target = outcome", &context, path, 2).unwrap();
  assert_eq!(
    let_dir,
    Some(ScriptDirective::Let(LetDirective {
      name: "target".to_string(),
      value: "outcome".to_string(),
    }))
  );

  // Register the macro in context
  context
    .macros
    .insert("target".to_string(), "outcome".to_string());

  // Duplicate let
  let err = parse_script_directive("let target = outcome2", &context, path, 3).unwrap_err();
  assert_eq!(
    err.to_string(),
    "directives.td:3: macro already defined: target"
  );

  // Invalid let syntax
  let err = parse_script_directive("let target outcome", &context, path, 4).unwrap_err();
  assert_eq!(
    err.to_string(),
    "directives.td:4: let expects syntax: let <name> = <value>"
  );

  // Non-identifier macro name
  let err = parse_script_directive("let 1target = outcome", &context, path, 5).unwrap_err();
  assert_eq!(
    err.to_string(),
    "directives.td:5: macro name must be an identifier: 1target"
  );

  // Empty value
  let err = parse_script_directive("let empty_val = ", &context, path, 6).unwrap_err();
  assert_eq!(
    err.to_string(),
    "directives.td:6: macro value cannot be empty: empty_val"
  );

  // Invalid seed
  let err = parse_script_directive("seed not_an_int", &context, path, 7).unwrap_err();
  assert_eq!(err.to_string(), "directives.td:7: seed expects an integer");
}

#[test]
fn script_contract_evaluates_conditions_and_control_flow() {
  let path = Path::new("control.td");

  // If directive
  assert_eq!(
    parse_control_flow_directive("if true", path, 1).unwrap(),
    Some(ControlFlowDirective::If(IfDirective { active: true }))
  );
  assert_eq!(
    parse_control_flow_directive("if false", path, 2).unwrap(),
    Some(ControlFlowDirective::If(IfDirective { active: false }))
  );
  assert_eq!(
    parse_control_flow_directive("else", path, 3).unwrap(),
    Some(ControlFlowDirective::Else(ElseDirective))
  );
  assert_eq!(
    parse_control_flow_directive("end", path, 4).unwrap(),
    Some(ControlFlowDirective::End(EndDirective))
  );

  // Malformed control directives
  let err = parse_control_flow_directive("if", path, 5).unwrap_err();
  assert_eq!(err.to_string(), "control.td:5: if expects a condition");

  let err = parse_control_flow_directive("else foo", path, 6).unwrap_err();
  assert_eq!(
    err.to_string(),
    "control.td:6: else does not accept a condition"
  );

  let err = parse_control_flow_directive("end bar", path, 7).unwrap_err();
  assert_eq!(
    err.to_string(),
    "control.td:7: end does not accept arguments"
  );

  // Conditions
  assert!(evaluate_script_condition("true", path, 8).unwrap());
  assert!(evaluate_script_condition("on", path, 8).unwrap());
  assert!(evaluate_script_condition("1", path, 8).unwrap());
  assert!(!evaluate_script_condition("false", path, 8).unwrap());
  assert!(!evaluate_script_condition("off", path, 8).unwrap());
  assert!(!evaluate_script_condition("0", path, 8).unwrap());
  assert!(evaluate_script_condition("alpha == alpha", path, 8).unwrap());
  assert!(!evaluate_script_condition("alpha == beta", path, 8).unwrap());
  assert!(evaluate_script_condition("alpha != beta", path, 8).unwrap());
  assert!(!evaluate_script_condition("alpha != alpha", path, 8).unwrap());

  // Malformed condition
  let err = evaluate_script_condition("invalid_condition", path, 9).unwrap_err();
  assert_eq!(
    err.to_string(),
    "control.td:9: if condition expects true/false, 1/0, on/off, ==, or !="
  );
}

#[test]
fn script_contract_block_state_branch_activity() {
  let state = ScriptBlockState::new(1, true);
  assert!(state.current_branch_active());

  let state_else = ScriptBlockState::with_in_else(1, true, true);
  assert!(!state_else.current_branch_active());

  let false_state_else = ScriptBlockState::with_in_else(1, false, true);
  assert!(false_state_else.current_branch_active());
}

#[test]
fn script_contract_error_formatting_and_accessors() {
  let err = ScriptError::new("path/to/script.td", 42, "syntax failure");
  assert_eq!(err.path(), Path::new("path/to/script.td"));
  assert_eq!(err.line(), 42);
  assert_eq!(err.message(), "syntax failure");
  assert_eq!(err.to_string(), "path/to/script.td:42: syntax failure");
}
