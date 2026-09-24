//! Script parsing, macro expansion, and directives engine for `.td` script files.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// A single discrete command line extracted from a script file or buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptCommand {
  /// The command text, retaining internal whitespace or multiline block content.
  pub text: String,
  /// The 1-based source line where this command began.
  pub start_line: usize,
}

impl ScriptCommand {
  /// Construct a new script command with its 1-based start line.
  pub fn new(text: impl Into<String>, start_line: usize) -> Self {
    Self {
      text: text.into(),
      start_line,
    }
  }
}

/// Evaluation context tracking registered macros and optional random seed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptContext {
  /// Registered macro definitions mapping names to string replacement values.
  pub macros: HashMap<String, String>,
  /// Optional random seed specified by a `seed` directive.
  pub seed: Option<i64>,
}

impl ScriptContext {
  /// Construct an empty script context without defined macros or seed.
  pub fn empty() -> Self {
    Self::default()
  }

  /// Construct a script context with specified macros and optional seed.
  pub fn new(macros: HashMap<String, String>, seed: Option<i64>) -> Self {
    Self { macros, seed }
  }
}

/// Directive specifying a random seed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedDirective {
  /// The parsed integer seed value.
  pub value: i64,
}

/// Directive defining a macro variable name and replacement value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetDirective {
  /// The macro variable name.
  pub name: String,
  /// The macro replacement value.
  pub value: String,
}

/// Top-level script environment directives (`seed` or `let`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptDirective {
  /// Seed directive.
  Seed(SeedDirective),
  /// Macro assignment directive.
  Let(LetDirective),
}

/// Directive initiating a conditional block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IfDirective {
  /// Whether the condition evaluated to true.
  pub active: bool,
}

/// Directive switching execution to the alternate branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElseDirective;

/// Directive closing a conditional block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndDirective;

/// Control flow directives (`if`, `else`, `end`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowDirective {
  /// If block start.
  If(IfDirective),
  /// Else branch.
  Else(ElseDirective),
  /// End of block.
  End(EndDirective),
}

/// State tracking for nested or active conditional blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptBlockState {
  /// The 1-based start line of the enclosing `if` block.
  pub start_line: usize,
  /// Whether the `if` condition evaluated to true.
  pub condition_active: bool,
  /// Whether execution has transitioned into the `else` branch.
  pub in_else: bool,
}

impl ScriptBlockState {
  /// Construct a new script block state for an active or inactive `if` directive.
  pub fn new(start_line: usize, condition_active: bool) -> Self {
    Self {
      start_line,
      condition_active,
      in_else: false,
    }
  }

  /// Construct a script block state specifying `in_else`.
  pub fn with_in_else(start_line: usize, condition_active: bool, in_else: bool) -> Self {
    Self {
      start_line,
      condition_active,
      in_else,
    }
  }

  /// Whether commands in the current branch (then vs. else) should be executed.
  pub fn current_branch_active(&self) -> bool {
    if self.in_else {
      !self.condition_active
    } else {
      self.condition_active
    }
  }
}

/// Recoverable script parsing or execution error containing source location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
  path: PathBuf,
  line: usize,
  message: String,
}

impl ScriptError {
  /// Construct a new script error with file path, 1-based line number, and diagnostic message.
  pub fn new(path: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
    Self {
      path: path.into(),
      line,
      message: message.into(),
    }
  }

  /// The path to the script file where the error occurred.
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// The 1-based line number of the error.
  pub fn line(&self) -> usize {
    self.line
  }

  /// The diagnostic error message.
  pub fn message(&self) -> &str {
    &self.message
  }
}

impl fmt::Display for ScriptError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}: {}", self.path.display(), self.line, self.message)
  }
}

impl std::error::Error for ScriptError {}

/// Read a script file from disk and parse it into discrete structured commands.
pub fn read_script(path: &Path) -> Result<Vec<ScriptCommand>, ScriptError> {
  if !path.exists() {
    return Err(ScriptError::new(path, 1, "script file not found"));
  }
  if path.is_dir() {
    return Err(ScriptError::new(path, 1, "script path is a directory"));
  }
  let bytes = match std::fs::read(path) {
    Ok(b) => b,
    Err(e) => {
      if e.kind() == std::io::ErrorKind::NotFound {
        return Err(ScriptError::new(path, 1, "script file not found"));
      }
      return Err(ScriptError::new(
        path,
        1,
        format!("could not read script: {e}"),
      ));
    }
  };
  let text = match std::str::from_utf8(&bytes) {
    Ok(s) => s,
    Err(e) => {
      let valid_up_to = e.valid_up_to();
      let line = bytes[..valid_up_to].iter().filter(|&&b| b == b'\n').count() + 1;
      return Err(ScriptError::new(
        path,
        line,
        "script file must be UTF-8 text",
      ));
    }
  };
  parse_script(text, path)
}

/// Decompose raw script text into discrete commands.
///
/// Blank lines and whole-line comments starting with `#` are ignored.
/// Multiline SQL commands bounded by triple quotes (`"""`) are grouped.
pub fn parse_script(text: &str, path: &Path) -> Result<Vec<ScriptCommand>, ScriptError> {
  let mut commands: Vec<ScriptCommand> = Vec::new();
  let mut pending_sql: Vec<String> = Vec::new();
  let mut pending_start: usize = 0;

  for (line_index, raw_line) in text.lines().enumerate() {
    let line_number = line_index + 1;
    let stripped = raw_line.trim();

    if !pending_sql.is_empty() {
      pending_sql.push(raw_line.to_string());
      if has_balanced_sql_triple_quote(&pending_sql) {
        commands.push(ScriptCommand::new(pending_sql.join("\n"), pending_start));
        pending_sql.clear();
        pending_start = 0;
      }
      continue;
    }

    if stripped.is_empty() || stripped.starts_with('#') {
      continue;
    }

    if starts_multiline_sql(stripped) {
      pending_sql = vec![raw_line.to_string()];
      pending_start = line_number;
      if has_balanced_sql_triple_quote(&pending_sql) {
        commands.push(ScriptCommand::new(pending_sql.join("\n"), pending_start));
        pending_sql.clear();
        pending_start = 0;
      }
      continue;
    }

    commands.push(ScriptCommand::new(raw_line.to_string(), line_number));
  }

  if !pending_sql.is_empty() {
    return Err(ScriptError::new(
      path,
      pending_start,
      "sql multiline query is missing closing \"\"\"",
    ));
  }

  Ok(commands)
}

fn starts_multiline_sql(stripped_line: &str) -> bool {
  let mut parts = stripped_line.split_whitespace();
  match (parts.next(), parts.next()) {
    (Some(cmd), Some(rest)) => {
      cmd.eq_ignore_ascii_case("sql") && rest.trim_start().starts_with("\"\"\"")
    }
    _ => false,
  }
}

fn has_balanced_sql_triple_quote(lines: &[String]) -> bool {
  let total_count: usize = lines
    .iter()
    .map(|line| line.matches("\"\"\"").count())
    .sum();
  total_count.is_multiple_of(2)
}

/// Replace macro references (prefixed with `$`) in a command text with their defined values.
pub fn expand_script_macros(
  text: &str,
  context: &ScriptContext,
  path: &Path,
  line: usize,
) -> Result<String, ScriptError> {
  let mut result = String::with_capacity(text.len());
  let bytes = text.as_bytes();
  let mut i = 0;

  while i < bytes.len() {
    if bytes[i] == b'$'
      && i + 1 < bytes.len()
      && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
    {
      let start = i + 1;
      let mut end = start;
      while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
      }
      let macro_name = &text[start..end];
      let value = context
        .macros
        .get(macro_name)
        .ok_or_else(|| ScriptError::new(path, line, format!("undefined macro: {macro_name}")))?;
      result.push_str(value);
      i = end;
      continue;
    }

    let ch = text[i..].chars().next().unwrap();
    result.push(ch);
    i += ch.len_utf8();
  }

  Ok(result)
}

fn is_macro_identifier(name: &str) -> bool {
  if name.is_empty() {
    return false;
  }
  let mut chars = name.chars();
  let first = chars.next().unwrap();
  if !first.is_ascii_alphabetic() && first != '_' {
    return false;
  }
  chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Parse a special script directive command (`seed` or `let`).
pub fn parse_script_directive(
  text: &str,
  context: &ScriptContext,
  path: &Path,
  line: usize,
) -> Result<Option<ScriptDirective>, ScriptError> {
  let stripped = text.trim();
  let command_name = stripped
    .split_whitespace()
    .next()
    .unwrap_or("")
    .to_ascii_lowercase();

  if command_name == "seed" {
    let parts: Vec<&str> = stripped.split_whitespace().collect();
    if parts.len() != 2 {
      return Err(ScriptError::new(path, line, "seed expects an integer"));
    }
    let value = parts[1]
      .parse::<i64>()
      .map_err(|_| ScriptError::new(path, line, "seed expects an integer"))?;
    return Ok(Some(ScriptDirective::Seed(SeedDirective { value })));
  }

  if command_name == "let" {
    let body = stripped[3..].trim();
    let (name, value) = body
      .split_once('=')
      .ok_or_else(|| ScriptError::new(path, line, "let expects syntax: let <name> = <value>"))?;
    let name = name.trim();
    let value = value.trim();
    if name.is_empty() {
      return Err(ScriptError::new(
        path,
        line,
        "let expects syntax: let <name> = <value>",
      ));
    }
    if !is_macro_identifier(name) {
      return Err(ScriptError::new(
        path,
        line,
        format!("macro name must be an identifier: {name}"),
      ));
    }
    if value.is_empty() {
      return Err(ScriptError::new(
        path,
        line,
        format!("macro value cannot be empty: {name}"),
      ));
    }
    if context.macros.contains_key(name) {
      return Err(ScriptError::new(
        path,
        line,
        format!("macro already defined: {name}"),
      ));
    }
    return Ok(Some(ScriptDirective::Let(LetDirective {
      name: name.to_string(),
      value: value.to_string(),
    })));
  }

  Ok(None)
}

/// Parse a conditional branching directive (`if`, `else`, `end`).
pub fn parse_control_flow_directive(
  text: &str,
  path: &Path,
  line: usize,
) -> Result<Option<ControlFlowDirective>, ScriptError> {
  let stripped = text.trim();
  let (command_name, body) = match stripped.split_once(' ') {
    Some((cmd, rest)) => (cmd, rest.trim()),
    None => (stripped, ""),
  };
  let normalized = command_name.to_ascii_lowercase();

  if normalized == "if" {
    if body.is_empty() {
      return Err(ScriptError::new(path, line, "if expects a condition"));
    }
    let active = evaluate_script_condition(body, path, line)?;
    return Ok(Some(ControlFlowDirective::If(IfDirective { active })));
  }

  if normalized == "else" {
    if !body.is_empty() {
      return Err(ScriptError::new(
        path,
        line,
        "else does not accept a condition",
      ));
    }
    return Ok(Some(ControlFlowDirective::Else(ElseDirective)));
  }

  if normalized == "end" {
    if !body.is_empty() {
      return Err(ScriptError::new(
        path,
        line,
        "end does not accept arguments",
      ));
    }
    return Ok(Some(ControlFlowDirective::End(EndDirective)));
  }

  Ok(None)
}

/// Evaluate a script conditional expression to a boolean state.
pub fn evaluate_script_condition(
  condition: &str,
  path: &Path,
  line: usize,
) -> Result<bool, ScriptError> {
  let normalized = condition.trim().to_ascii_lowercase();
  match normalized.as_str() {
    "true" | "on" | "1" => return Ok(true),
    "false" | "off" | "0" => return Ok(false),
    _ => {}
  }

  for operator in ["==", "!="] {
    if let Some((left, right)) = condition.split_once(operator) {
      let left_val = left.trim();
      let right_val = right.trim();
      if left_val.is_empty() || right_val.is_empty() {
        return Err(ScriptError::new(
          path,
          line,
          "if condition expects true/false, 1/0, on/off, ==, or !=",
        ));
      }
      return Ok(if operator == "==" {
        left_val == right_val
      } else {
        left_val != right_val
      });
    }
  }

  Err(ScriptError::new(
    path,
    line,
    "if condition expects true/false, 1/0, on/off, ==, or !=",
  ))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_script_ignores_blank_lines_and_whole_line_comments() {
    let script = r#"
    # load data

    use patients.parquet
      # inspect data
    describe
    "#;
    let commands = parse_script(script, Path::new("<script>")).unwrap();
    assert_eq!(
      commands,
      vec![
        ScriptCommand::new("    use patients.parquet", 4),
        ScriptCommand::new("    describe", 6),
      ]
    );
  }

  #[test]
  fn parse_script_preserves_multiline_sql_command() {
    let script = "use patients.parquet\nsql \"\"\"\nselect *\nfrom active\n\"\"\"\ncount\n";
    let commands = parse_script(script, Path::new("<script>")).unwrap();
    assert_eq!(
      commands,
      vec![
        ScriptCommand::new("use patients.parquet", 1),
        ScriptCommand::new("sql \"\"\"\nselect *\nfrom active\n\"\"\"", 2),
        ScriptCommand::new("count", 6),
      ]
    );
  }

  #[test]
  fn parse_script_rejects_unterminated_multiline_sql() {
    let script = "use patients.parquet\nsql \"\"\"\nselect *\n";
    let err = parse_script(script, Path::new("<script>")).unwrap_err();
    assert_eq!(
      err.to_string(),
      "<script>:2: sql multiline query is missing closing \"\"\""
    );
  }

  #[test]
  fn read_script_reports_missing_file() {
    let missing = Path::new("non_existent_file_xyz_123.td");
    let err = read_script(missing).unwrap_err();
    assert_eq!(err.message(), "script file not found");
  }

  #[test]
  fn expand_script_macros_substitutes_defined_values() {
    let mut macros = HashMap::new();
    macros.insert("data".to_string(), "patients.parquet".to_string());
    macros.insert("condition".to_string(), "age >= 18".to_string());
    let context = ScriptContext::new(macros, None);

    let expanded = expand_script_macros(
      "use $data\nkeep if $condition",
      &context,
      Path::new("analysis.td"),
      2,
    )
    .unwrap();

    assert_eq!(expanded, "use patients.parquet\nkeep if age >= 18");
  }

  #[test]
  fn expand_script_macros_rejects_undefined_reference() {
    let context = ScriptContext::empty();
    let err = expand_script_macros("use $data", &context, Path::new("analysis.td"), 3).unwrap_err();
    assert_eq!(err.to_string(), "analysis.td:3: undefined macro: data");
  }

  #[test]
  fn parse_script_seed_directive() {
    let context = ScriptContext::empty();
    assert_eq!(
      parse_script_directive("seed 123", &context, Path::new("analysis.td"), 1).unwrap(),
      Some(ScriptDirective::Seed(SeedDirective { value: 123 }))
    );
  }

  #[test]
  fn parse_script_seed_directive_rejects_invalid_value() {
    let context = ScriptContext::empty();
    let err =
      parse_script_directive("seed 1.5", &context, Path::new("analysis.td"), 1).unwrap_err();
    assert_eq!(err.to_string(), "analysis.td:1: seed expects an integer");
  }

  #[test]
  fn parse_script_let_directive() {
    let context = ScriptContext::empty();
    assert_eq!(
      parse_script_directive(
        "let data = patients.parquet",
        &context,
        Path::new("analysis.td"),
        1
      )
      .unwrap(),
      Some(ScriptDirective::Let(LetDirective {
        name: "data".to_string(),
        value: "patients.parquet".to_string(),
      }))
    );
  }

  #[test]
  fn parse_script_let_directive_allows_previous_macro_references() {
    let mut macros = HashMap::new();
    macros.insert("root".to_string(), "data".to_string());
    let context = ScriptContext::new(macros, None);
    let expanded = expand_script_macros(
      "let file = $root/patients.parquet",
      &context,
      Path::new("analysis.td"),
      2,
    )
    .unwrap();

    assert_eq!(
      parse_script_directive(&expanded, &context, Path::new("analysis.td"), 2).unwrap(),
      Some(ScriptDirective::Let(LetDirective {
        name: "file".to_string(),
        value: "data/patients.parquet".to_string(),
      }))
    );
  }

  #[test]
  fn parse_script_let_directive_rejects_invalid_syntax() {
    let context = ScriptContext::empty();
    let err = parse_script_directive(
      "let data patients.parquet",
      &context,
      Path::new("analysis.td"),
      1,
    )
    .unwrap_err();
    assert_eq!(
      err.to_string(),
      "analysis.td:1: let expects syntax: let <name> = <value>"
    );
  }

  #[test]
  fn parse_script_let_directive_rejects_invalid_name() {
    let context = ScriptContext::empty();
    let err = parse_script_directive(
      "let 1data = patients.parquet",
      &context,
      Path::new("analysis.td"),
      1,
    )
    .unwrap_err();
    assert_eq!(
      err.to_string(),
      "analysis.td:1: macro name must be an identifier: 1data"
    );
  }

  #[test]
  fn parse_script_let_directive_rejects_empty_value() {
    let context = ScriptContext::empty();
    let err =
      parse_script_directive("let data = ", &context, Path::new("analysis.td"), 1).unwrap_err();
    assert_eq!(
      err.to_string(),
      "analysis.td:1: macro value cannot be empty: data"
    );
  }

  #[test]
  fn parse_script_let_directive_rejects_duplicate_name() {
    let mut macros = HashMap::new();
    macros.insert("data".to_string(), "patients.parquet".to_string());
    let context = ScriptContext::new(macros, None);
    let err = parse_script_directive(
      "let data = other.parquet",
      &context,
      Path::new("analysis.td"),
      1,
    )
    .unwrap_err();
    assert_eq!(
      err.to_string(),
      "analysis.td:1: macro already defined: data"
    );
  }

  #[test]
  fn parse_script_control_flow_directives() {
    assert_eq!(
      parse_control_flow_directive("if true", Path::new("analysis.td"), 1).unwrap(),
      Some(ControlFlowDirective::If(IfDirective { active: true }))
    );
    assert_eq!(
      parse_control_flow_directive("else", Path::new("analysis.td"), 2).unwrap(),
      Some(ControlFlowDirective::Else(ElseDirective))
    );
    assert_eq!(
      parse_control_flow_directive("end", Path::new("analysis.td"), 3).unwrap(),
      Some(ControlFlowDirective::End(EndDirective))
    );
    assert_eq!(
      parse_control_flow_directive("count", Path::new("analysis.td"), 4).unwrap(),
      None
    );
  }

  #[test]
  fn script_block_state_reports_current_branch_activity() {
    assert!(ScriptBlockState::new(1, true).current_branch_active());
    assert!(!ScriptBlockState::with_in_else(1, true, true).current_branch_active());
    assert!(ScriptBlockState::with_in_else(1, false, true).current_branch_active());
  }

  #[test]
  fn evaluate_script_condition_cases() {
    let path = Path::new("analysis.td");
    assert!(evaluate_script_condition("true", path, 1).unwrap());
    assert!(evaluate_script_condition("on", path, 1).unwrap());
    assert!(evaluate_script_condition("1", path, 1).unwrap());
    assert!(!evaluate_script_condition("false", path, 1).unwrap());
    assert!(!evaluate_script_condition("off", path, 1).unwrap());
    assert!(!evaluate_script_condition("0", path, 1).unwrap());
    assert!(evaluate_script_condition("duckdb == duckdb", path, 1).unwrap());
    assert!(evaluate_script_condition("duckdb != polars", path, 1).unwrap());
    assert!(!evaluate_script_condition("duckdb == polars", path, 1).unwrap());
  }

  #[test]
  fn evaluate_script_condition_rejects_unsupported_condition() {
    let err = evaluate_script_condition("duckdb", Path::new("analysis.td"), 1).unwrap_err();
    assert_eq!(
      err.to_string(),
      "analysis.td:1: if condition expects true/false, 1/0, on/off, ==, or !="
    );
  }

  #[test]
  fn parse_control_flow_rejects_missing_condition() {
    let err = parse_control_flow_directive("if", Path::new("analysis.td"), 1).unwrap_err();
    assert_eq!(err.to_string(), "analysis.td:1: if expects a condition");
  }
}
