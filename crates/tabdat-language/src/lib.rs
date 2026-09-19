#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

/// The syntax-only command forms currently understood by the language layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
  /// Show general help or the help topic named by `topic`.
  Help { topic: Option<String> },
  /// Show the current session status.
  Status,
  /// Request termination of the interactive session.
  Exit,
  /// Inspect the active dataset schema (execution is deferred).
  Describe,
  /// Inspect environment and capability health (execution is deferred).
  Doctor,
  /// Compute descriptive statistics for selected columns.
  Summarize { variables: Vec<String> },
  /// Compute a signature for the active dataset in the bounded eager runtime.
  Datasignature,
  /// Profile selected columns.
  Codebook { variables: Vec<String> },
  /// Report explicit-null missingness for selected columns.
  Missing { variables: Vec<String> },
  /// Report duplicate key groups in the bounded eager runtime.
  Duplicates { variables: Vec<String> },
  /// Assert key uniqueness for selected variables in the bounded eager runtime.
  Isid {
    variables: Vec<String>,
    missok: bool,
  },
  /// Validate a boolean predicate against every row in the bounded eager runtime.
  Assert { expression: AssertExpression },
  /// Keep an explicit ordered set of columns in the bounded eager runtime.
  Keep { variables: Vec<String> },
  /// Drop an explicit set of columns in the bounded eager runtime.
  Drop { variables: Vec<String> },
  /// Keep only listed columns (relation execution is deferred).
  Select { variables: Vec<String> },
  /// Sort active rows by listed columns (relation execution is deferred).
  Sort { variables: Vec<String> },
  /// Sort active rows by explicitly directed keys (relation execution is deferred).
  Gsort { keys: Vec<SortKey> },
  /// Rename one column to another (relation execution is deferred).
  Rename { old_name: String, new_name: String },
  /// Execute a script file (script execution is deferred).
  Run { path: String },
  /// Change a runtime setting (configuration execution is deferred).
  Set { name: SettingName, value: String },
  /// Persist the active dataset to a path (filesystem execution is deferred).
  Save { path: String, replace: bool },
  /// Export the active dataset to a path (filesystem execution is deferred).
  Export { path: String, replace: bool },
  /// Select a dataset source and loading options (execution is deferred).
  Use {
    source: DataSource,
    execution_mode: ExecutionMode,
    lazy_engine: Option<LazyEngine>,
    delimiter: Option<String>,
    has_header: Option<bool>,
  },
  /// Count rows in the active dataset.
  Count,
  /// Preview the first `limit` rows of the active dataset.
  Head { limit: RowLimit },
  /// Preview the last `limit` rows of the active dataset.
  Tail { limit: RowLimit },
}

/// An expression accepted by the bounded eager `assert` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertExpression {
  /// A dataset column reference.
  Identifier(String),
  /// A validated numeric literal preserved as SQL-safe source text.
  Number(String),
  /// A quoted string literal.
  String(String),
  /// An explicit SQL NULL literal.
  Null,
  /// Unary numeric negation.
  UnaryMinus(Box<Self>),
  /// A binary arithmetic or comparison expression.
  Binary {
    /// Left operand.
    left: Box<Self>,
    /// Operator between operands.
    operator: AssertBinaryOperator,
    /// Right operand.
    right: Box<Self>,
  },
}

/// Operators accepted by [`AssertExpression::Binary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertBinaryOperator {
  /// Addition.
  Add,
  /// Subtraction.
  Subtract,
  /// Multiplication.
  Multiply,
  /// Division.
  Divide,
  /// Equality.
  Equal,
  /// Inequality.
  NotEqual,
  /// Less-than comparison.
  Less,
  /// Less-than-or-equal comparison.
  LessOrEqual,
  /// Greater-than comparison.
  Greater,
  /// Greater-than-or-equal comparison.
  GreaterOrEqual,
}

/// A local path or an unvalidated remote URI supplied to `use`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSource {
  LocalPath(String),
  Uri(String),
}

/// Whether a `use` request should load eagerly or build a lazy plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
  Eager,
  Lazy,
}

/// The lazy engine named by a `use` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LazyEngine {
  DuckDb,
  Polars,
}

/// The finite setting names accepted by the syntax-only `set` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingName {
  /// Select the artifact image format (value validation is deferred).
  GraphFormat,
  /// Select the directory used for generated artifacts (validation is deferred).
  ArtifactDir,
  /// Select whether generated graphs open automatically (validation is deferred).
  GraphOpen,
}

/// One owned `gsort` key and its requested direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
  /// The variable spelling after an optional unquoted direction prefix.
  pub variable: String,
  /// Whether the key requests descending order.
  pub descending: bool,
}

impl AssertBinaryOperator {
  /// Return whether this operator produces a boolean comparison.
  pub fn is_comparison(self) -> bool {
    matches!(
      self,
      Self::Equal
        | Self::NotEqual
        | Self::Less
        | Self::LessOrEqual
        | Self::Greater
        | Self::GreaterOrEqual
    )
  }
}

/// A validated, canonical non-negative decimal row limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowLimit(Box<str>);

impl RowLimit {
  /// Return the canonical ASCII decimal representation without parsing it into
  /// a bounded machine integer.
  pub fn as_decimal(&self) -> &str {
    &self.0
  }
}

impl Default for RowLimit {
  fn default() -> Self {
    Self("5".into())
  }
}

/// A deterministic error produced while parsing a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  message: String,
}

fn is_command_whitespace(character: char) -> bool {
  character.is_whitespace() || matches!(character, '\u{1c}'..='\u{1f}')
}

fn leading_quote_error(command: &str, quote: u8) -> &'static str {
  let identifier = quote == b'`';
  let mut index = 1;
  let mut content_nonempty = false;
  let bytes = command.as_bytes();
  while index < bytes.len() {
    if bytes[index] == quote {
      if bytes.get(index + 1) == Some(&quote) {
        content_nonempty = true;
        index += 2;
        continue;
      }
      if identifier && !content_nonempty {
        return "quoted identifier cannot be empty";
      }
      return "command must start with an unquoted command name";
    }
    content_nonempty = true;
    index += 1;
  }
  if identifier {
    "unterminated quoted identifier"
  } else {
    "unterminated quoted string"
  }
}

impl ParseError {
  fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
    }
  }

  /// Return the stable human-readable parse diagnostic.
  pub fn message(&self) -> &str {
    &self.message
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.message)
  }
}

impl Error for ParseError {}

/// Parse one syntax-only command without executing it or initializing a backend.
pub fn parse_command(input: &str) -> Result<Command, ParseError> {
  let command = input.trim_matches(is_command_whitespace);
  if command.is_empty() {
    return Err(ParseError::new("empty command"));
  }

  if let Some(quote @ (b'`' | b'\'' | b'"')) = command.as_bytes().first().copied() {
    return Err(ParseError::new(leading_quote_error(command, quote)));
  }

  if let Some(help_body) = command.strip_prefix('?') {
    return parse_help(help_body.trim());
  }

  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"use"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"rename"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..4)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"sort"))
    && command.as_bytes().get(4) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..6)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"select"))
    && command.as_bytes().get(6) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }
  if command
    .as_bytes()
    .get(..3)
    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"run"))
    && command.as_bytes().get(3) == Some(&b':')
  {
    return Err(ParseError::new("unsupported token in command: :"));
  }

  // `status` keeps the Python tokenizer's punctuation diagnostics for
  // attached unary-sign forms such as `status-1` and `status+1`.
  if command.len() > 6
    && command
      .as_bytes()
      .get(..6)
      .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"status"))
    && command
      .get(6..)
      .and_then(|suffix| suffix.chars().next())
      .is_some_and(|character| matches!(character, '-' | '+'))
  {
    return parse_named_command("status", &command[6..]);
  }

  // `gsort` permits attached symbolic key text (for example `gsort-age` and
  // `gsort:age`) in the pinned Python tokenizer. Split only this command's
  // non-identifier suffix before the generic command-name boundary scan.
  if command.len() > 5
    && command
      .as_bytes()
      .get(..5)
      .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"gsort"))
    && command
      .get(5..)
      .and_then(|suffix| suffix.chars().next())
      .is_some_and(|character| !character.is_alphanumeric() && character != '_')
  {
    return parse_named_command("gsort", &command[5..]);
  }

  for (name, prefix_length) in [("save", 4usize), ("export", 6usize)] {
    if command.len() > prefix_length
      && command
        .as_bytes()
        .get(..prefix_length)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(name.as_bytes()))
      && command
        .get(prefix_length..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|character| !character.is_alphanumeric() && character != '_')
    {
      return parse_named_command(name, &command[prefix_length..]);
    }
  }

  let Some(command_end) = command
    .find(|character: char| is_command_whitespace(character) || matches!(character, ',' | '='))
  else {
    return parse_named_command(command, "");
  };
  let delimiter = command[command_end..]
    .chars()
    .next()
    .expect("command_end always points to a character");
  let name = &command[..command_end];
  let body = command[command_end..].trim_matches(is_command_whitespace);
  if name.eq_ignore_ascii_case("use") && delimiter == ',' {
    parse_use_options(command[command_end + 1..].trim_matches(is_command_whitespace))?;
    return Err(ParseError::new("unknown command: use"));
  }
  if name.eq_ignore_ascii_case("use") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("use assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("run") && delimiter == ',' {
    if command[command_end + 1..]
      .trim_matches(is_command_whitespace)
      .is_empty()
    {
      return Err(ParseError::new(
        "comma must be followed by at least one option",
      ));
    }
    return Err(ParseError::new("unknown command: run"));
  }
  if name.eq_ignore_ascii_case("run") && delimiter == '=' {
    if command[command_end..].starts_with("==") {
      return Err(ParseError::new("unsupported token in command: =="));
    }
    return Err(ParseError::new("run assignment requires a target before ="));
  }
  if name.eq_ignore_ascii_case("help") && !is_command_whitespace(delimiter) {
    return Err(ParseError::new("unknown command: help"));
  }
  parse_named_command(name, body)
}

fn parse_named_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let normalized_name = name.to_lowercase();
  match normalized_name.as_str() {
    "help" => parse_help(body),
    "status" => parse_status_command(body),
    "describe" => {
      if body.is_empty() {
        Ok(Command::Describe)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "describe assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else if body.starts_with('-') {
        Err(ParseError::new("unsupported token in command: -"))
      } else if body.starts_with('+') {
        Err(ParseError::new("unsupported token in command: +"))
      } else {
        Err(ParseError::new(
          "describe does not accept arguments, if clauses, or options",
        ))
      }
    }
    "doctor" => {
      if body.is_empty() {
        Ok(Command::Doctor)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "doctor assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else if body.starts_with('-') {
        Err(ParseError::new("unsupported token in command: -"))
      } else if body.starts_with('+') {
        Err(ParseError::new("unsupported token in command: +"))
      } else if body.eq_ignore_ascii_case("if") {
        Err(ParseError::new("missing expression after if"))
      } else {
        Err(ParseError::new(
          "doctor does not accept arguments, if clauses, options, or assignment syntax",
        ))
      }
    }
    "summarize" => parse_summarize_command(body),
    "datasignature" => parse_datasignature_command(body),
    "assert" => parse_assert_command(body),
    "codebook" => parse_codebook_command(body),
    "missing" => parse_missing_command(body),
    "duplicates" => parse_duplicates_command(body),
    "isid" => parse_isid_command(body),
    "keep" => parse_keep_command(body),
    "drop" => parse_drop_command(body),
    "select" => parse_select_command(body),
    "sort" => parse_sort_command(body),
    "gsort" => parse_gsort_command(body),
    "rename" => parse_rename_command(body),
    "run" => parse_run_command(body),
    "set" => parse_set_command(body),
    "save" | "export" => parse_save_export_command(normalized_name.as_str(), body),
    "use" => parse_use_command(body),
    "count" | "head" | "tail" => parse_inspection_command(normalized_name.as_str(), body),
    "exit" | "quit" => {
      if body.is_empty() {
        Ok(Command::Exit)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(format!(
          "{normalized_name} assignment requires a target before ="
        )))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else {
        Err(ParseError::new(format!(
          "{normalized_name} does not accept arguments, if clauses, or options"
        )))
      }
    }
    other => Err(ParseError::new(format!("unknown command: {other}"))),
  }
}

fn parse_status_command(body: &str) -> Result<Command, ParseError> {
  if body.is_empty() {
    return Ok(Command::Status);
  }
  if body.starts_with('-') {
    return Err(ParseError::new("unsupported token in command: -"));
  }
  if body.starts_with('+') {
    return Err(ParseError::new("unsupported token in command: +"));
  }
  if body.eq_ignore_ascii_case("if") {
    return Err(ParseError::new("missing expression after if"));
  }
  if body.trim_matches(is_command_whitespace).ends_with(',') {
    return Err(ParseError::new(
      "comma must be followed by at least one option",
    ));
  }
  if body.starts_with('=') && !body.starts_with("==") {
    return Err(ParseError::new(
      "status assignment requires a target before =",
    ));
  }
  if body.starts_with("==") {
    return Err(ParseError::new("unsupported token in command: =="));
  }
  Err(ParseError::new(
    "status does not accept arguments, if clauses, options, or assignment syntax",
  ))
}

#[derive(Debug)]
struct SimpleArgument {
  text: String,
  backtick_quoted: bool,
}

#[derive(Debug, Default)]
struct SimpleBody {
  arguments: Vec<SimpleArgument>,
  has_options: bool,
  has_assignment: bool,
  assignment_target_missing: bool,
  has_condition: bool,
  missing_condition_expression: bool,
}

fn parse_inspection_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(format!(
      "{name} assignment requires a target before ="
    )));
  }

  if name == "count" {
    if parts.arguments.is_empty()
      && !parts.has_options
      && !parts.has_assignment
      && !parts.has_condition
    {
      return Ok(Command::Count);
    }
    return Err(ParseError::new(
      "count does not accept arguments, if clauses, options, or assignment syntax",
    ));
  }

  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new(format!(
      "{name} does not accept if clauses, options, or assignment syntax"
    )));
  }
  if parts.arguments.len() > 1 {
    return Err(ParseError::new(format!(
      "{name} accepts at most one row limit"
    )));
  }
  let limit = parts
    .arguments
    .first()
    .map(|argument| parse_row_limit(&argument.text, name))
    .transpose()?
    .unwrap_or_default();
  Ok(match name {
    "head" => Command::Head { limit },
    "tail" => Command::Tail { limit },
    _ => unreachable!("parse_inspection_command only handles inspection names"),
  })
}

fn parse_set_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new("set assignment requires a target before ="));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition || parts.arguments.len() != 2
  {
    return Err(ParseError::new("set expects syntax: set name value"));
  }

  let setting_name = parts.arguments[0].text.to_lowercase();
  let name = match (setting_name.as_str(), parts.arguments[0].backtick_quoted) {
    ("graph_format", false) => SettingName::GraphFormat,
    ("artifact_dir", false) => SettingName::ArtifactDir,
    ("graph_open", false) => SettingName::GraphOpen,
    _ => {
      return Err(ParseError::new(format!(
        "unknown setting: {}",
        parts.arguments[0].text
      )));
    }
  };
  Ok(Command::Set {
    name,
    value: parts.arguments[1].text.clone(),
  })
}

fn parse_datasignature_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "datasignature assignment requires a target before =",
    ));
  }
  if parts.arguments.is_empty()
    && !parts.has_options
    && !parts.has_assignment
    && !parts.has_condition
  {
    return Ok(Command::Datasignature);
  }
  Err(ParseError::new(
    "datasignature does not accept arguments, if clauses, options, or assignment syntax",
  ))
}

fn parse_assert_command(body: &str) -> Result<Command, ParseError> {
  let tokens = tokenize_use_options(body)?;
  if tokens.is_empty() {
    return Err(ParseError::new("assert expects a boolean expression"));
  }

  let mut depth = 0_i32;
  for token in &tokens {
    match (&token.kind, token.text.as_str()) {
      (UseTokenKind::Symbol, "(") => depth += 1,
      (UseTokenKind::Symbol, ")") => {
        depth -= 1;
        if depth < 0 {
          return Err(ParseError::new("unsupported token in expression: )"));
        }
      }
      (UseTokenKind::Symbol, ",") if depth == 0 => {
        return Err(ParseError::new("assert does not accept options"));
      }
      (UseTokenKind::Identifier { quoted: false }, name)
        if depth == 0 && name.eq_ignore_ascii_case("if") =>
      {
        return Err(ParseError::new("assert does not accept if clauses"));
      }
      _ => {}
    }
  }
  if depth != 0 {
    return Err(ParseError::new("unsupported token in expression: ("));
  }
  if tokens
    .iter()
    .any(|token| token.kind == UseTokenKind::Symbol && token.text == "=")
  {
    return Err(ParseError::new("assert does not accept assignment syntax"));
  }

  let expression = AssertExpressionParser::new(tokens).parse()?;
  Ok(Command::Assert { expression })
}

struct AssertExpressionParser {
  tokens: Vec<UseToken>,
  index: usize,
}

impl AssertExpressionParser {
  fn new(tokens: Vec<UseToken>) -> Self {
    Self { tokens, index: 0 }
  }

  fn parse(mut self) -> Result<AssertExpression, ParseError> {
    let expression = self.parse_comparison()?;
    if let Some(token) = self.peek() {
      return Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      )));
    }
    Ok(expression)
  }

  fn parse_comparison(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_additive()?;
    let Some(operator) = self.peek().and_then(assert_comparison_operator) else {
      return Ok(expression);
    };
    self.index += 1;
    let right = self.parse_additive()?;
    expression = AssertExpression::Binary {
      left: Box::new(expression),
      operator,
      right: Box::new(right),
    };
    if let Some(token) = self.peek()
      && assert_comparison_operator(token).is_some()
    {
      return Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      )));
    }
    Ok(expression)
  }

  fn parse_additive(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_multiplicative()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "+" => Some(AssertBinaryOperator::Add),
        "-" => Some(AssertBinaryOperator::Subtract),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      let right = self.parse_multiplicative()?;
      expression = AssertExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_multiplicative(&mut self) -> Result<AssertExpression, ParseError> {
    let mut expression = self.parse_unary()?;
    loop {
      let Some(operator) = self.peek().and_then(|token| match token.text.as_str() {
        "*" => Some(AssertBinaryOperator::Multiply),
        "/" => Some(AssertBinaryOperator::Divide),
        _ => None,
      }) else {
        return Ok(expression);
      };
      self.index += 1;
      let right = self.parse_unary()?;
      expression = AssertExpression::Binary {
        left: Box::new(expression),
        operator,
        right: Box::new(right),
      };
    }
  }

  fn parse_unary(&mut self) -> Result<AssertExpression, ParseError> {
    if self.peek().is_some_and(|token| token.text == "-") {
      self.index += 1;
      return Ok(AssertExpression::UnaryMinus(Box::new(self.parse_unary()?)));
    }
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> Result<AssertExpression, ParseError> {
    let Some(token) = self.consume() else {
      return Err(ParseError::new("assert expects a boolean expression"));
    };
    match token.kind {
      UseTokenKind::Number => {
        if token.text.parse::<f64>().is_err() {
          return Err(ParseError::new(format!("malformed number: {}", token.text)));
        }
        Ok(AssertExpression::Number(token.text))
      }
      UseTokenKind::String => Ok(AssertExpression::String(token.text)),
      UseTokenKind::Identifier { quoted } => {
        if !quoted && token.text.eq_ignore_ascii_case("null") {
          Ok(AssertExpression::Null)
        } else {
          Ok(AssertExpression::Identifier(token.text))
        }
      }
      UseTokenKind::Symbol if token.text == "(" => {
        let expression = self.parse_comparison()?;
        let Some(closing) = self.consume() else {
          return Err(ParseError::new("unsupported token in expression: ("));
        };
        if closing.kind != UseTokenKind::Symbol || closing.text != ")" {
          return Err(ParseError::new(format!(
            "unsupported token in expression: {}",
            closing.text
          )));
        }
        Ok(expression)
      }
      UseTokenKind::Symbol => Err(ParseError::new(format!(
        "unsupported token in expression: {}",
        token.text
      ))),
    }
  }

  fn peek(&self) -> Option<&UseToken> {
    self.tokens.get(self.index)
  }

  fn consume(&mut self) -> Option<UseToken> {
    let token = self.tokens.get(self.index).cloned();
    self.index += usize::from(token.is_some());
    token
  }
}

fn assert_comparison_operator(token: &UseToken) -> Option<AssertBinaryOperator> {
  if token.kind != UseTokenKind::Symbol {
    return None;
  }
  match token.text.as_str() {
    "==" => Some(AssertBinaryOperator::Equal),
    "!=" => Some(AssertBinaryOperator::NotEqual),
    "<" => Some(AssertBinaryOperator::Less),
    "<=" => Some(AssertBinaryOperator::LessOrEqual),
    ">" => Some(AssertBinaryOperator::Greater),
    ">=" => Some(AssertBinaryOperator::GreaterOrEqual),
    _ => None,
  }
}

fn parse_summarize_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "summarize assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "summarize does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "summarize does not accept if clauses or options",
    ));
  }
  Ok(Command::Summarize {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_codebook_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "codebook assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "codebook does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "codebook does not accept if clauses or options",
    ));
  }
  Ok(Command::Codebook {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_missing_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "missing assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new("missing does not accept assignment syntax"));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "missing does not accept if clauses or options",
    ));
  }
  Ok(Command::Missing {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_duplicates_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "duplicates assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    return Err(ParseError::new(
      "duplicates does not accept assignment syntax",
    ));
  }
  if parts.has_condition || parts.has_options {
    return Err(ParseError::new(
      "duplicates does not accept if clauses or options",
    ));
  }

  let mut arguments = parts.arguments;
  if arguments.first().is_some_and(|argument| {
    !argument.backtick_quoted && argument.text.eq_ignore_ascii_case("report")
  }) {
    arguments.remove(0);
  }
  Ok(Command::Duplicates {
    variables: arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_isid_command(body: &str) -> Result<Command, ParseError> {
  let (variable_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(variable_body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "isid assignment requires a target before =",
    ));
  }
  if parts.has_assignment {
    if variable_body
      .trim_matches(is_command_whitespace)
      .ends_with('=')
    {
      return Err(ParseError::new(
        "isid assignment requires an expression after =",
      ));
    }
    return Err(ParseError::new(
      "isid only accepts a variable list and missok option",
    ));
  }
  if parts.has_condition {
    return Err(ParseError::new(
      "isid only accepts a variable list and missok option",
    ));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();
  if parts.arguments.is_empty() {
    return Err(ParseError::new("isid expects at least one key variable"));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| option.name != "missok")
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "isid unsupported option: {}",
      unsupported.join(", ")
    )));
  }

  if options
    .iter()
    .any(|option| option.name == "missok" && option.value != UseOptionValue::Flag)
  {
    return Err(ParseError::new(
      "isid option missok does not accept a value",
    ));
  }

  Ok(Command::Isid {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
    missok: options.iter().any(|option| option.name == "missok"),
  })
}

fn parse_select_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "select assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "select assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("select only accepts a variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("select expects at least one variable"));
  }
  Ok(Command::Select {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_keep_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  let condition_has_options = parts.has_condition && keep_condition_has_options(body);
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "keep assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "keep assignment requires an expression after =",
    ));
  }
  if condition_has_options || parts.has_options || parts.has_assignment {
    return Err(ParseError::new(
      "keep does not accept options or assignment syntax",
    ));
  }
  if parts.has_condition {
    if parts.arguments.is_empty() {
      return Err(ParseError::new(
        "keep if execution is deferred in the bounded runtime",
      ));
    }
    return Err(ParseError::new(
      "keep cannot combine a variable list with an if clause",
    ));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("keep expects a variable list or if clause"));
  }
  Ok(Command::Keep {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_drop_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  let condition_has_options = parts.has_condition && keep_condition_has_options(body);
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.has_condition
    && let Some(error) = drop_condition_syntax_error(body)
  {
    return Err(error);
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "drop assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "drop assignment requires an expression after =",
    ));
  }
  if condition_has_options || parts.has_options || parts.has_assignment {
    return Err(ParseError::new(
      "drop does not accept options or assignment syntax",
    ));
  }
  if parts.has_condition {
    if parts.arguments.is_empty() {
      return Err(ParseError::new(
        "drop if execution is deferred in the bounded runtime",
      ));
    }
    return Err(ParseError::new(
      "drop cannot combine a variable list with an if clause",
    ));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("drop expects a variable list or if clause"));
  }
  Ok(Command::Drop {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn drop_condition_syntax_error(body: &str) -> Option<ParseError> {
  let tokens = tokenize_use_options(body).ok()?;
  let condition_start = tokens.iter().position(|token| {
    matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
  })?;
  let mut depth = 0_i32;
  let mut saw_operand = false;
  let mut last_condition_token = None;
  for token in tokens.into_iter().skip(condition_start + 1) {
    if token.kind == UseTokenKind::Symbol {
      match token.text.as_str() {
        "(" => depth += 1,
        ")" => depth -= 1,
        "," if depth == 0 => break,
        _ => {}
      }
      if depth == 0 && token.text.eq_ignore_ascii_case("if") {
        return Some(ParseError::new("duplicate if clause"));
      }
      last_condition_token = Some(token);
      continue;
    }
    if depth == 0
      && matches!(token.kind, UseTokenKind::Identifier { quoted: false })
      && token.text.eq_ignore_ascii_case("if")
    {
      return Some(ParseError::new("duplicate if clause"));
    }
    saw_operand = true;
    last_condition_token = Some(token);
  }
  let Some(last) = last_condition_token else {
    return None;
  };
  if saw_operand
    && matches!(
      last.text.as_str(),
      "+" | "-" | "*" | "/" | "==" | "!=" | "<" | "<=" | ">" | ">="
    )
  {
    return Some(ParseError::new(format!(
      "incomplete expression after {}",
      last.text
    )));
  }
  None
}

fn keep_condition_has_options(body: &str) -> bool {
  let characters: Vec<char> = body.chars().collect();
  let mut index = 0;
  let mut condition_seen = false;
  while index < characters.len() {
    if matches!(characters[index], '\'' | '"' | '`') {
      let quote = characters[index];
      index += 1;
      while index < characters.len() {
        if characters[index] == quote {
          if quote == '`' && characters.get(index + 1) == Some(&'`') {
            index += 2;
            continue;
          }
          index += 1;
          break;
        }
        index += 1;
      }
      continue;
    }
    if characters[index].is_alphabetic() || characters[index] == '_' {
      let start = index;
      index += 1;
      while index < characters.len()
        && (characters[index].is_alphanumeric() || characters[index] == '_')
      {
        index += 1;
      }
      if !condition_seen
        && characters[start..index]
          .iter()
          .collect::<String>()
          .eq_ignore_ascii_case("if")
      {
        condition_seen = true;
      }
      continue;
    }
    if condition_seen && characters[index] == ',' {
      let mut lookahead = index + 1;
      while characters
        .get(lookahead)
        .is_some_and(|character| is_command_whitespace(*character))
      {
        lookahead += 1;
      }
      return characters.get(lookahead).is_some();
    }
    index += 1;
  }
  false
}

fn parse_sort_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "sort assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "sort assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("sort only accepts a variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("sort expects at least one variable"));
  }
  Ok(Command::Sort {
    variables: parts
      .arguments
      .into_iter()
      .map(|argument| argument.text)
      .collect(),
  })
}

fn parse_gsort_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "gsort assignment requires a target before =",
    ));
  }
  if parts.has_assignment && body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(
      "gsort assignment requires an expression after =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition {
    return Err(ParseError::new("gsort only accepts a signed variable list"));
  }
  if parts.arguments.is_empty() {
    return Err(ParseError::new("gsort expects at least one variable"));
  }

  let mut keys = Vec::with_capacity(parts.arguments.len());
  for argument in parts.arguments {
    let mut variable = argument.text;
    let mut descending = false;
    if !argument.backtick_quoted
      && let Some(prefix) = variable
        .chars()
        .next()
        .filter(|prefix| matches!(prefix, '+' | '-'))
    {
      descending = prefix == '-';
      variable.remove(0);
      if variable.is_empty() {
        return Err(ParseError::new(
          "gsort expects a variable after each direction prefix",
        ));
      }
      if variable
        .chars()
        .next()
        .is_some_and(|prefix| matches!(prefix, '+' | '-'))
      {
        return Err(ParseError::new(
          "gsort keys must use at most one + or - prefix",
        ));
      }
    }
    if variable.is_empty() {
      return Err(ParseError::new(
        "gsort expects a variable after each direction prefix",
      ));
    }
    keys.push(SortKey {
      variable,
      descending,
    });
  }
  Ok(Command::Gsort { keys })
}

fn parse_run_command(body: &str) -> Result<Command, ParseError> {
  let mut path_parts = body
    .trim_matches(is_command_whitespace)
    .split(is_command_whitespace)
    .filter(|part| !part.is_empty());
  let Some(path) = path_parts.next() else {
    return Err(ParseError::new(
      "run expects exactly one path: run <script>",
    ));
  };
  if path_parts.next().is_some() {
    return Err(ParseError::new(
      "run expects exactly one path: run <script>",
    ));
  }
  Ok(Command::Run {
    path: path.to_owned(),
  })
}

fn parse_save_export_command(command_name: &str, body: &str) -> Result<Command, ParseError> {
  let (path_body, option_body) = match first_unquoted_comma(body) {
    Some(index) => (&body[..index], Some(&body[index + 1..])),
    None => (body, None),
  };
  let parts = parse_simple_body(path_body, true)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(format!(
      "{command_name} assignment requires a target before ="
    )));
  }
  if parts.has_assignment && path_body.trim_matches(is_command_whitespace).ends_with('=') {
    return Err(ParseError::new(format!(
      "{command_name} assignment requires an expression after ="
    )));
  }

  let options = option_body
    .map(parse_use_options)
    .transpose()?
    .unwrap_or_default();

  if parts.has_condition || parts.has_assignment {
    return Err(ParseError::new(format!(
      "{command_name} does not accept if clauses or assignment syntax"
    )));
  }
  if parts.arguments.len() != 1 {
    return Err(ParseError::new(format!(
      "{command_name} expects exactly one path"
    )));
  }

  let mut unsupported = options
    .iter()
    .filter(|option| option.name != "replace")
    .map(|option| option.name.as_str())
    .collect::<Vec<_>>();
  unsupported.sort_unstable();
  unsupported.dedup();
  if !unsupported.is_empty() {
    return Err(ParseError::new(format!(
      "{command_name} unsupported option: {}",
      unsupported.join(", ")
    )));
  }
  if options
    .iter()
    .any(|option| option.name == "replace" && option.value != UseOptionValue::Flag)
  {
    return Err(ParseError::new(format!(
      "{command_name} option replace does not accept a value"
    )));
  }

  let path = parts
    .arguments
    .into_iter()
    .next()
    .expect("save/export arity checked before extracting the path")
    .text;
  let replace = options.iter().any(|option| option.name == "replace");
  Ok(match command_name {
    "save" => Command::Save { path, replace },
    "export" => Command::Export { path, replace },
    _ => unreachable!("save/export parser only handles save and export"),
  })
}

fn parse_rename_command(body: &str) -> Result<Command, ParseError> {
  let parts = parse_simple_body(body, false)?;
  if parts.missing_condition_expression {
    return Err(ParseError::new("missing expression after if"));
  }
  if parts.assignment_target_missing {
    return Err(ParseError::new(
      "rename assignment requires a target before =",
    ));
  }
  if parts.has_options || parts.has_assignment || parts.has_condition || parts.arguments.len() != 2
  {
    return Err(ParseError::new(
      "rename expects exactly two variables: rename old new",
    ));
  }
  let mut arguments = parts.arguments.into_iter();
  let old_name = arguments
    .next()
    .expect("rename arity checked before extracting arguments")
    .text;
  let new_name = arguments
    .next()
    .expect("rename arity checked before extracting arguments")
    .text;
  Ok(Command::Rename { old_name, new_name })
}

fn first_unquoted_comma(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  let mut quote = None;
  let mut index = 0;
  while index < bytes.len() {
    let character = bytes[index];
    if let Some(active_quote) = quote {
      if character == active_quote {
        if active_quote == b'`' && bytes.get(index + 1) == Some(&active_quote) {
          index += 2;
          continue;
        }
        quote = None;
      }
      index += 1;
      continue;
    }
    match character {
      b'\'' | b'"' | b'`' => {
        quote = Some(character);
        index += 1;
      }
      b',' => return Some(index),
      _ => index += 1,
    }
  }
  None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UseTokenKind {
  Identifier { quoted: bool },
  String,
  Number,
  Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UseToken {
  kind: UseTokenKind,
  text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UseOptionValue {
  Flag,
  String(String),
  Number,
  Boolean(bool),
  Identifiers(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UseOption {
  name: String,
  value: UseOptionValue,
}

fn parse_use_command(body: &str) -> Result<Command, ParseError> {
  let body = body.trim_matches(is_command_whitespace);
  if body.is_empty() {
    return Err(ParseError::new("use expects exactly one path: use <path>"));
  }

  let (path_text, option_text) = match body.split_once(',') {
    Some((path, options)) => (path, Some(options)),
    None => (body, None),
  };
  let path_parts: Vec<&str> = path_text
    .split(is_command_whitespace)
    .filter(|part| !part.is_empty())
    .collect();
  if path_parts.len() != 1 {
    return Err(ParseError::new("use expects exactly one path: use <path>"));
  }

  let source_text = path_parts[0].to_owned();
  let source = if source_text.contains("://") {
    DataSource::Uri(source_text)
  } else {
    DataSource::LocalPath(source_text)
  };

  let Some(option_text) = option_text else {
    return Ok(Command::Use {
      source,
      execution_mode: ExecutionMode::Eager,
      lazy_engine: None,
      delimiter: None,
      has_header: None,
    });
  };

  let options = parse_use_options(option_text)?;
  let mut names: Vec<&str> = Vec::with_capacity(options.len());
  for option in &options {
    if names.contains(&option.name.as_str()) {
      return Err(ParseError::new("use option specified more than once"));
    }
    names.push(option.name.as_str());
  }

  let mut is_lazy = false;
  let mut engine: Option<String> = None;
  let mut delimiter: Option<String> = None;
  let mut has_header: Option<bool> = None;

  for option in options {
    match option.name.as_str() {
      "lazy" => {
        if option.value != UseOptionValue::Flag {
          return Err(ParseError::new("use lazy option does not accept a value"));
        }
        is_lazy = true;
      }
      "engine" => match option.value {
        UseOptionValue::String(value) => engine = Some(value.to_lowercase()),
        _ => {
          return Err(ParseError::new("use engine option expects a string value"));
        }
      },
      "delimiter" => match option.value {
        UseOptionValue::String(value) => delimiter = Some(value),
        _ => {
          return Err(ParseError::new(
            "use delimiter option expects a string value",
          ));
        }
      },
      "has_header" => match option.value {
        UseOptionValue::Boolean(value) => has_header = Some(value),
        UseOptionValue::Flag => has_header = Some(true),
        _ => {
          return Err(ParseError::new(
            "use has_header option expects a boolean value",
          ));
        }
      },
      name => return Err(ParseError::new(format!("unknown use option: {name}"))),
    }
  }

  let lazy_engine = if let Some(engine) = engine {
    let engine = match engine.as_str() {
      "duckdb" => LazyEngine::DuckDb,
      "polars" => LazyEngine::Polars,
      _ => return Err(ParseError::new("use engine must be duckdb or polars")),
    };
    if !is_lazy {
      return Err(ParseError::new("use engine option requires lazy mode"));
    }
    Some(engine)
  } else if is_lazy {
    Some(LazyEngine::DuckDb)
  } else {
    None
  };

  Ok(Command::Use {
    source,
    execution_mode: if is_lazy {
      ExecutionMode::Lazy
    } else {
      ExecutionMode::Eager
    },
    lazy_engine,
    delimiter,
    has_header,
  })
}

fn parse_use_options(text: &str) -> Result<Vec<UseOption>, ParseError> {
  let tokens = tokenize_use_options(text)?;
  if tokens.is_empty() {
    return Err(ParseError::new(
      "comma must be followed by at least one option",
    ));
  }

  let mut stream = UseTokenStream { tokens, index: 0 };
  let mut options = Vec::new();
  while !stream.at_end() {
    let token = stream.consume().expect("stream is not at end");
    let UseTokenKind::Identifier { quoted: false } = token.kind else {
      return Err(ParseError::new("option names must be identifiers"));
    };
    let name = token.text;
    let mut value = UseOptionValue::Flag;

    if stream.peek_is_symbol("(") {
      stream.consume();
      let mut value_tokens = Vec::new();
      let mut depth = 1;
      while !stream.at_end() && depth > 0 {
        let token = stream.consume().expect("stream is not at end");
        if token.kind == UseTokenKind::Symbol && token.text == "(" {
          depth += 1;
        } else if token.kind == UseTokenKind::Symbol && token.text == ")" {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        value_tokens.push(token);
      }
      if depth > 0 {
        return Err(ParseError::new(format!(
          "option {name} is missing closing )"
        )));
      }
      if value_tokens.is_empty() {
        return Err(ParseError::new(format!(
          "option {name} expects at least one value"
        )));
      }
      value = parse_use_parenthesized_value(&name, value_tokens)?;
    }

    if stream.peek_is_symbol("=") {
      stream.consume();
      let Some(value_token) = stream.consume() else {
        return Err(ParseError::new(format!(
          "option {name} requires a value after ="
        )));
      };
      if value_token.kind == UseTokenKind::Symbol
        && matches!(value_token.text.as_str(), "," | "=" | "(" | ")")
      {
        return Err(ParseError::new(format!(
          "option {name} has malformed value"
        )));
      }
      value = match value_token.kind {
        UseTokenKind::Number => UseOptionValue::Number,
        _ => UseOptionValue::String(value_token.text),
      };
    } else if stream.peek_is_kind(&UseTokenKind::Number)
      || stream.peek_is_kind(&UseTokenKind::String)
    {
      return Err(ParseError::new(format!(
        "option {name} value must use option=value syntax"
      )));
    }

    options.push(UseOption { name, value });
  }
  Ok(options)
}

fn parse_use_parenthesized_value(
  name: &str,
  tokens: Vec<UseToken>,
) -> Result<UseOptionValue, ParseError> {
  if name == "delimiter" {
    if tokens.len() != 1
      || !matches!(
        tokens[0].kind,
        UseTokenKind::String | UseTokenKind::Identifier { .. }
      )
    {
      return Err(ParseError::new(
        "option delimiter expects a single string or identifier value",
      ));
    }
    return Ok(UseOptionValue::String(tokens[0].text.clone()));
  }

  if name == "has_header" {
    if tokens.len() != 1 {
      return Err(ParseError::new("option has_header expects true or false"));
    }
    let UseTokenKind::Identifier { quoted: false } = tokens[0].kind else {
      return Err(ParseError::new("option has_header expects true or false"));
    };
    return match tokens[0].text.to_lowercase().as_str() {
      "true" => Ok(UseOptionValue::Boolean(true)),
      "false" => Ok(UseOptionValue::Boolean(false)),
      _ => Err(ParseError::new("option has_header expects true or false")),
    };
  }

  if matches!(name, "saving" | "weights") {
    return Ok(UseOptionValue::String(
      tokens.into_iter().map(|token| token.text).collect(),
    ));
  }

  if matches!(
    name,
    "alpha"
      | "ll"
      | "ul"
      | "quantile"
      | "lags"
      | "instlag"
      | "n_iter"
      | "tol"
      | "knn"
      | "cv"
      | "bootstrap"
      | "seed"
      | "rseed"
      | "folds"
      | "level"
      | "draws"
      | "burnin"
      | "tune"
      | "chains"
      | "thin"
  ) {
    let numeric_text: String = tokens.iter().map(|token| token.text.as_str()).collect();
    if numeric_text.parse::<f64>().is_ok() {
      return Ok(UseOptionValue::Number);
    }
    return Err(ParseError::new(format!(
      "option {name} expects a numeric value"
    )));
  }

  if name == "prior" {
    let comma_index = tokens
      .iter()
      .position(|token| token.kind == UseTokenKind::Symbol && token.text == ",");
    let Some(comma_index) = comma_index else {
      return Err(ParseError::new(
        "prior option expects prior(variable, distribution) syntax",
      ));
    };
    if comma_index == 0 || comma_index + 1 == tokens.len() {
      return Err(ParseError::new(
        "prior option expects prior(variable, distribution) syntax",
      ));
    }
    return Ok(UseOptionValue::Identifiers(Vec::new()));
  }

  if name == "l1_ratio" {
    if !use_numeric_list_is_valid(&tokens) {
      return Err(ParseError::new("option l1_ratio values must be numeric"));
    }
    return Ok(UseOptionValue::Number);
  }

  if name == "start" {
    if !use_numeric_list_is_valid(&tokens) {
      return Err(ParseError::new("option start values must be numeric"));
    }
    return Ok(UseOptionValue::Number);
  }

  if tokens
    .iter()
    .all(|token| matches!(token.kind, UseTokenKind::Identifier { .. }))
  {
    return Ok(UseOptionValue::Identifiers(
      tokens.into_iter().map(|token| token.text).collect(),
    ));
  }
  Err(ParseError::new(format!(
    "option {name} values must be identifiers"
  )))
}

fn use_numeric_list_is_valid(tokens: &[UseToken]) -> bool {
  let mut index = 0;
  while index < tokens.len() {
    if tokens[index].kind == UseTokenKind::Number {
      index += 1;
      continue;
    }
    if tokens[index].kind == UseTokenKind::Symbol
      && matches!(tokens[index].text.as_str(), "-" | "+")
      && tokens
        .get(index + 1)
        .is_some_and(|token| token.kind == UseTokenKind::Number)
    {
      index += 2;
      continue;
    }
    return false;
  }
  true
}

#[derive(Debug)]
struct UseTokenStream {
  tokens: Vec<UseToken>,
  index: usize,
}

impl UseTokenStream {
  fn at_end(&self) -> bool {
    self.index >= self.tokens.len()
  }

  fn peek_is_symbol(&self, symbol: &str) -> bool {
    self
      .tokens
      .get(self.index)
      .is_some_and(|token| token.kind == UseTokenKind::Symbol && token.text == symbol)
  }

  fn peek_is_kind(&self, kind: &UseTokenKind) -> bool {
    self
      .tokens
      .get(self.index)
      .is_some_and(|token| &token.kind == kind)
  }

  fn consume(&mut self) -> Option<UseToken> {
    let token = self.tokens.get(self.index).cloned();
    self.index += usize::from(token.is_some());
    token
  }
}

fn tokenize_use_options(text: &str) -> Result<Vec<UseToken>, ParseError> {
  let characters: Vec<char> = text.chars().collect();
  let mut tokens = Vec::new();
  let mut index = 0;
  while index < characters.len() {
    let character = characters[index];
    if is_command_whitespace(character) {
      index += 1;
      continue;
    }
    if character.is_alphabetic() || character == '_' {
      let start = index;
      index += 1;
      while index < characters.len()
        && (characters[index].is_alphanumeric() || characters[index] == '_')
      {
        index += 1;
      }
      tokens.push(UseToken {
        kind: UseTokenKind::Identifier { quoted: false },
        text: characters[start..index].iter().collect(),
      });
      continue;
    }
    if character == '`' {
      index += 1;
      let mut value = String::new();
      let mut content_nonempty = false;
      let mut closed = false;
      while index < characters.len() {
        if characters[index] != '`' {
          value.push(characters[index]);
          content_nonempty = true;
          index += 1;
          continue;
        }
        if characters.get(index + 1) == Some(&'`') {
          value.push('`');
          content_nonempty = true;
          index += 2;
          continue;
        }
        index += 1;
        closed = true;
        break;
      }
      if !closed {
        return Err(ParseError::new("unterminated quoted identifier"));
      }
      if !content_nonempty {
        return Err(ParseError::new("quoted identifier cannot be empty"));
      }
      tokens.push(UseToken {
        kind: UseTokenKind::Identifier { quoted: true },
        text: value,
      });
      continue;
    }
    if character.is_numeric()
      || (character == '.'
        && characters
          .get(index + 1)
          .is_some_and(|next| next.is_numeric()))
    {
      let start = index;
      index += 1;
      while index < characters.len() && (characters[index].is_numeric() || characters[index] == '.')
      {
        index += 1;
      }
      let text: String = characters[start..index].iter().collect();
      if text.chars().filter(|character| *character == '.').count() > 1 {
        return Err(ParseError::new(format!("malformed number: {text}")));
      }
      tokens.push(UseToken {
        kind: UseTokenKind::Number,
        text,
      });
      continue;
    }
    if matches!(character, '\'' | '"') {
      let quote = character;
      index += 1;
      let start = index;
      while index < characters.len() && characters[index] != quote {
        index += 1;
      }
      if index >= characters.len() {
        return Err(ParseError::new("unterminated quoted string"));
      }
      let text: String = characters[start..index].iter().collect();
      index += 1;
      tokens.push(UseToken {
        kind: UseTokenKind::String,
        text,
      });
      continue;
    }
    let two_char: String = characters[index..].iter().take(2).collect();
    if matches!(two_char.as_str(), "==" | "!=" | "<=" | ">=") {
      tokens.push(UseToken {
        kind: UseTokenKind::Symbol,
        text: two_char,
      });
      index += 2;
      continue;
    }
    if matches!(
      character,
      ',' | '=' | '<' | '>' | '+' | '-' | '*' | '/' | '(' | ')' | ':' | '.'
    ) {
      tokens.push(UseToken {
        kind: UseTokenKind::Symbol,
        text: character.to_string(),
      });
      index += 1;
      continue;
    }
    return Err(ParseError::new(format!(
      "unsupported token in command: {character}"
    )));
  }
  Ok(tokens)
}

fn parse_row_limit(text: &str, name: &str) -> Result<RowLimit, ParseError> {
  if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
    return Err(ParseError::new(format!(
      "{name} row limit must be a non-negative integer"
    )));
  }
  let canonical = text.trim_start_matches('0');
  let canonical = if canonical.is_empty() { "0" } else { canonical };
  Ok(RowLimit(canonical.into()))
}

fn parse_simple_body(body: &str, allow_symbols: bool) -> Result<SimpleBody, ParseError> {
  let characters: Vec<char> = body.chars().collect();
  let mut parts = SimpleBody::default();
  let mut index = 0;
  while index < characters.len() {
    while characters
      .get(index)
      .is_some_and(|character| is_command_whitespace(*character))
    {
      index += 1;
    }
    if index >= characters.len() {
      break;
    }

    match characters[index] {
      ',' => {
        index += 1;
        while characters
          .get(index)
          .is_some_and(|character| is_command_whitespace(*character))
        {
          index += 1;
        }
        if index >= characters.len() {
          return Err(ParseError::new(
            "comma must be followed by at least one option",
          ));
        }
        parts.has_options = true;
        break;
      }
      '=' if !(allow_symbols && characters.get(index + 1) == Some(&'=')) => {
        if characters.get(index + 1) == Some(&'=') {
          return Err(ParseError::new("unsupported token in command: =="));
        }
        parts.has_assignment = true;
        parts.assignment_target_missing = parts.arguments.is_empty();
        break;
      }
      _ if is_unsupported_simple_symbol(characters[index], allow_symbols) => {
        return Err(ParseError::new(format!(
          "unsupported token in command: {}",
          characters[index]
        )));
      }
      _ => {
        let mut text = String::new();
        let mut quoted = false;
        let mut backtick_quoted = false;
        loop {
          if index >= characters.len() || is_command_whitespace(characters[index]) {
            break;
          }
          if characters[index] == ',' {
            break;
          }
          if characters[index] == '=' {
            if allow_symbols && characters.get(index + 1) == Some(&'=') {
              text.push_str("==");
              index += 2;
              continue;
            }
            if allow_symbols && matches!(text.chars().last(), Some('<' | '>')) {
              text.push('=');
              index += 1;
              continue;
            }
            break;
          }
          if characters[index] == '!' && allow_symbols && characters.get(index + 1) == Some(&'=') {
            text.push_str("!=");
            index += 2;
            continue;
          }
          if characters[index] == '!' {
            return Err(ParseError::new("unsupported token in command: !"));
          }
          if characters[index].is_alphabetic() || characters[index] == '_' {
            let identifier_start = index;
            index += 1;
            while index < characters.len()
              && (characters[index].is_alphanumeric() || characters[index] == '_')
            {
              index += 1;
            }
            let identifier_is_if = index - identifier_start == 2
              && characters[identifier_start].eq_ignore_ascii_case(&'i')
              && characters[identifier_start + 1].eq_ignore_ascii_case(&'f');
            if identifier_is_if {
              if !text.is_empty() || quoted {
                index = identifier_start;
                break;
              }
              text.push_str("if");
              continue;
            }
            text.extend(&characters[identifier_start..index]);
            continue;
          }
          if is_unsupported_simple_symbol(characters[index], allow_symbols) {
            return Err(ParseError::new(format!(
              "unsupported token in command: {}",
              characters[index]
            )));
          }
          if matches!(
            characters[index],
            '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>'
          ) {
            if allow_symbols {
              text.push(characters[index]);
              index += 1;
              continue;
            }
            return Err(ParseError::new(format!(
              "unsupported token in command: {}",
              characters[index]
            )));
          }
          if characters[index] == '.' && !allow_symbols {
            let numeric_start = text.is_empty()
              && characters
                .get(index + 1)
                .is_some_and(|character| character.is_numeric());
            let numeric_continuation = !text.is_empty()
              && text
                .chars()
                .next()
                .is_some_and(|character| character.is_numeric());
            if !numeric_start && !numeric_continuation {
              return Err(ParseError::new("unsupported token in command: ."));
            }
          }
          if matches!(characters[index], '\'' | '"' | '`') {
            let if_boundary_before_backtick =
              characters[index] == '`' && !quoted && text.eq_ignore_ascii_case("if");
            if ((!text.is_empty() || quoted) && characters[index] != '`')
              || if_boundary_before_backtick
            {
              break;
            }
            let quote = characters[index];
            quoted = true;
            backtick_quoted = backtick_quoted || quote == '`';
            let piece = parse_quoted_piece(&characters, &mut index, quote)?;
            text.push_str(&piece);
            if allow_symbols
              && quote != '`'
              && characters
                .get(index)
                .is_some_and(|next| matches!(next, '\'' | '"'))
            {
              break;
            }
          } else {
            text.push(characters[index]);
            index += 1;
          }
        }
        if !text.is_empty() || quoted {
          let is_if = !quoted && text.eq_ignore_ascii_case("if");
          if is_if {
            parts.has_condition = true;
            let mut lookahead = index;
            while characters
              .get(lookahead)
              .is_some_and(|character| is_command_whitespace(*character))
            {
              lookahead += 1;
            }
            parts.missing_condition_expression = characters
              .get(lookahead)
              .is_none_or(|character| matches!(character, ',' | '='));
            break;
          }
          parts.arguments.push(SimpleArgument {
            text,
            backtick_quoted,
          });
        }
      }
    }
  }
  Ok(parts)
}

fn is_unsupported_simple_symbol(character: char, allow_symbols: bool) -> bool {
  if !allow_symbols {
    return !character.is_alphanumeric()
      && character != '_'
      && character != '.'
      && !matches!(character, '\'' | '"' | '`' | ',' | '=');
  }
  if allow_symbols
    && !character.is_alphanumeric()
    && character != '_'
    && character != '.'
    && !matches!(character, '\'' | '"' | '`' | ',' | '=')
    && !matches!(
      character,
      '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>' | '!'
    )
  {
    return true;
  }
  let always_unsupported = matches!(
    character,
    '?' | '[' | ']' | '{' | '}' | '%' | '&' | '|' | '^' | '~' | '#'
  );
  let symbol_allowed_for_set = matches!(
    character,
    '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>'
  );
  let unsupported_known_symbol = matches!(
    character,
    '+' | '-' | ':' | '/' | '(' | ')' | '*' | '<' | '>' | '='
  );
  always_unsupported
    || (unsupported_known_symbol && character != '=' && !(allow_symbols && symbol_allowed_for_set))
    || (character == '=' && !allow_symbols)
}

fn parse_quoted_piece(
  characters: &[char],
  index: &mut usize,
  quote: char,
) -> Result<String, ParseError> {
  *index += 1;
  let mut text = String::new();
  let mut content_nonempty = false;
  while *index < characters.len() {
    if characters[*index] == quote {
      if characters.get(*index + 1) == Some(&quote) && quote == '`' {
        if quote == '`' && !content_nonempty && *index + 2 == characters.len() {
          return Err(ParseError::new("quoted identifier cannot be empty"));
        }
        text.push(quote);
        content_nonempty = true;
        *index += 2;
        continue;
      }
      *index += 1;
      if quote == '`' && !content_nonempty {
        return Err(ParseError::new("quoted identifier cannot be empty"));
      }
      return Ok(text);
    }
    text.push(characters[*index]);
    content_nonempty = true;
    *index += 1;
  }
  Err(ParseError::new(if quote == '`' {
    "unterminated quoted identifier"
  } else {
    "unterminated quoted string"
  }))
}

fn parse_help(body: &str) -> Result<Command, ParseError> {
  let mut words = body
    .split(is_command_whitespace)
    .filter(|word| !word.is_empty());
  let topic = words.next().map(str::to_lowercase);
  if words.next().is_some() {
    return Err(ParseError::new(
      "help expects at most one command name: help <command>",
    ));
  }
  Ok(Command::Help { topic })
}

#[cfg(test)]
mod tests {
  use super::{
    Command, DataSource, ExecutionMode, LazyEngine, ParseError, RowLimit, SettingName, SortKey,
    parse_command,
  };

  #[test]
  fn parses_help_aliases_and_topics() {
    assert_eq!(
      parse_command("help").unwrap(),
      Command::Help { topic: None }
    );
    assert_eq!(parse_command("?").unwrap(), Command::Help { topic: None });
    assert_eq!(
      parse_command("help summarize").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(
      parse_command("? SUMMARIZE").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(
      parse_command("?foo").unwrap(),
      Command::Help {
        topic: Some("foo".to_owned()),
      }
    );
  }

  #[test]
  fn normalizes_command_case_and_whitespace() {
    assert_eq!(
      parse_command("  HELP   SUMMARIZE  ").unwrap(),
      Command::Help {
        topic: Some("summarize".to_owned()),
      }
    );
    assert_eq!(parse_command("\tSTATUS\n").unwrap(), Command::Status);
    assert_eq!(parse_command(" qUiT ").unwrap(), Command::Exit);
  }

  #[test]
  fn parses_status_and_exit_aliases() {
    assert_eq!(parse_command("status").unwrap(), Command::Status);
    assert_eq!(parse_command("exit").unwrap(), Command::Exit);
    assert_eq!(parse_command("quit").unwrap(), Command::Exit);
  }

  #[test]
  fn parses_describe_with_case_and_whitespace_normalization() {
    assert_eq!(parse_command("describe").unwrap(), Command::Describe);
    assert_eq!(
      parse_command("  DESCRIBE\u{1c}").unwrap(),
      Command::Describe
    );
  }

  #[test]
  fn parses_doctor_with_case_and_whitespace_normalization() {
    assert_eq!(parse_command("doctor").unwrap(), Command::Doctor);
    assert_eq!(parse_command("\tDOCTOR\u{1c}").unwrap(), Command::Doctor);
  }

  #[test]
  fn parses_summarize_variables_without_execution() {
    assert_eq!(
      parse_command(" SUMMARIZE ").unwrap(),
      Command::Summarize { variables: vec![] }
    );
    assert_eq!(
      parse_command("summarize age bmi").unwrap(),
      Command::Summarize {
        variables: vec!["age".to_owned(), "bmi".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize\u{1c}`bmi-zscore`\u{1d}\"value col\"").unwrap(),
      Command::Summarize {
        variables: vec!["bmi-zscore".to_owned(), "value col".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize `x``y` \"report\"").unwrap(),
      Command::Summarize {
        variables: vec!["x`y".to_owned(), "report".to_owned()],
      }
    );
    assert_eq!(
      parse_command("summarize age age").unwrap(),
      Command::Summarize {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_summarize_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "summarize age if age > 0",
        "summarize does not accept if clauses or options",
      ),
      (
        "summarize age, detail",
        "summarize does not accept if clauses or options",
      ),
      (
        "summarize age = other",
        "summarize does not accept assignment syntax",
      ),
      (
        "summarize = age",
        "summarize assignment requires a target before =",
      ),
      (
        "summarize age,",
        "comma must be followed by at least one option",
      ),
      (
        "summarize,",
        "comma must be followed by at least one option",
      ),
      ("summarize if", "missing expression after if"),
      ("summarize age if", "missing expression after if"),
      ("summarize age==x", "unsupported token in command: =="),
      ("summarize age-1", "unsupported token in command: -"),
      ("summarize age+1", "unsupported token in command: +"),
      ("summarize age!x", "unsupported token in command: !"),
      ("summarize age@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_datasignature_with_case_and_whitespace_normalization() {
    assert_eq!(
      parse_command("datasignature").unwrap(),
      Command::Datasignature
    );
    assert_eq!(
      parse_command("\tDATASIGNATURE\u{1c}").unwrap(),
      Command::Datasignature
    );
  }

  #[test]
  fn parses_codebook_variables_without_execution() {
    assert_eq!(
      parse_command(" CODEBOOK ").unwrap(),
      Command::Codebook { variables: vec![] }
    );
    assert_eq!(
      parse_command("codebook age sex").unwrap(),
      Command::Codebook {
        variables: vec!["age".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook `bmi-zscore` `cost.2024` `x/y`").unwrap(),
      Command::Codebook {
        variables: vec![
          "bmi-zscore".to_owned(),
          "cost.2024".to_owned(),
          "x/y".to_owned(),
        ],
      }
    );
    assert_eq!(
      parse_command("codebook\u{1c}`x y`\u{1d}sex").unwrap(),
      Command::Codebook {
        variables: vec!["x y".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook foo\"bar\"").unwrap(),
      Command::Codebook {
        variables: vec!["foo".to_owned(), "bar".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook \"foo\"\"bar\"").unwrap(),
      Command::Codebook {
        variables: vec!["foo".to_owned(), "bar".to_owned()],
      }
    );
    assert_eq!(
      parse_command("codebook \"\"\"\"").unwrap(),
      Command::Codebook {
        variables: vec![String::new(), String::new()],
      }
    );
  }

  #[test]
  fn parses_missing_variables_without_execution() {
    assert_eq!(
      parse_command(" MISSING ").unwrap(),
      Command::Missing { variables: vec![] }
    );
    assert_eq!(
      parse_command("missing cost age").unwrap(),
      Command::Missing {
        variables: vec!["cost".to_owned(), "age".to_owned()],
      }
    );
    assert_eq!(
      parse_command("missing\u{1c}`bmi-zscore`\u{1d}\"value col\"").unwrap(),
      Command::Missing {
        variables: vec!["bmi-zscore".to_owned(), "value col".to_owned()],
      }
    );
  }

  #[test]
  fn parses_duplicates_variables_without_execution() {
    assert_eq!(
      parse_command(" DUPLICATES ").unwrap(),
      Command::Duplicates { variables: vec![] }
    );
    assert_eq!(
      parse_command("duplicates report id label").unwrap(),
      Command::Duplicates {
        variables: vec!["id".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("duplicates id label").unwrap(),
      Command::Duplicates {
        variables: vec!["id".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("duplicates \"report\"").unwrap(),
      Command::Duplicates { variables: vec![] }
    );
    assert_eq!(
      parse_command("duplicates\u{1c}`report`\u{1d}cost").unwrap(),
      Command::Duplicates {
        variables: vec!["report".to_owned(), "cost".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_duplicates_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "duplicates id if id > 0",
        "duplicates does not accept if clauses or options",
      ),
      (
        "duplicates id, missing",
        "duplicates does not accept if clauses or options",
      ),
      (
        "duplicates id = other",
        "duplicates does not accept assignment syntax",
      ),
      (
        "duplicates = id",
        "duplicates assignment requires a target before =",
      ),
      (
        "duplicates id,",
        "comma must be followed by at least one option",
      ),
      (
        "duplicates,",
        "comma must be followed by at least one option",
      ),
      ("duplicates if", "missing expression after if"),
      ("duplicates id if", "missing expression after if"),
      ("duplicates id==x", "unsupported token in command: =="),
      ("duplicates id-1", "unsupported token in command: -"),
      ("duplicates id+1", "unsupported token in command: +"),
      ("duplicates id!x", "unsupported token in command: !"),
      ("duplicates id@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_isid_variables_and_missok_without_execution() {
    assert_eq!(
      parse_command(" ISID patient_id visit ").unwrap(),
      Command::Isid {
        variables: vec!["patient_id".to_owned(), "visit".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid\u{1c}`a,b`\u{1d}visit, missok missok").unwrap(),
      Command::Isid {
        variables: vec!["a,b".to_owned(), "visit".to_owned()],
        missok: true,
      }
    );
    assert_eq!(
      parse_command("isid a a").unwrap(),
      Command::Isid {
        variables: vec!["a".to_owned(), "a".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid `a``b`").unwrap(),
      Command::Isid {
        variables: vec!["a`b".to_owned()],
        missok: false,
      }
    );
    assert_eq!(
      parse_command("isid \"\"").unwrap(),
      Command::Isid {
        variables: vec![String::new()],
        missok: false,
      }
    );
  }

  #[test]
  fn rejects_invalid_isid_syntax_with_exact_diagnostics() {
    let cases = [
      ("isid", "isid expects at least one key variable"),
      ("isid, missok", "isid expects at least one key variable"),
      ("isid,", "comma must be followed by at least one option"),
      (
        "isid patient_id if visit > 0",
        "isid only accepts a variable list and missok option",
      ),
      (
        "isid patient_id = other",
        "isid only accepts a variable list and missok option",
      ),
      (
        "isid patient_id =",
        "isid assignment requires an expression after =",
      ),
      (
        "isid = patient_id",
        "isid assignment requires a target before =",
      ),
      ("isid patient_id, report", "isid unsupported option: report"),
      (
        "isid patient_id, foo bar",
        "isid unsupported option: bar, foo",
      ),
      (
        "isid patient_id, missok(true)",
        "isid option missok does not accept a value",
      ),
      (
        "isid patient_id, missok 1",
        "option missok value must use option=value syntax",
      ),
      ("isid patient_id if", "missing expression after if"),
      ("isid patient_id==x", "unsupported token in command: =="),
      ("isid patient_id-x", "unsupported token in command: -"),
      ("isid patient_id+x", "unsupported token in command: +"),
      ("isid patient_id!x", "unsupported token in command: !"),
      ("isid patient_id@x", "unsupported token in command: @"),
      ("isid patient_id, MISSOK", "isid unsupported option: MISSOK"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_select_variables_without_execution() {
    assert_eq!(
      parse_command(" SELECT age sex ").unwrap(),
      Command::Select {
        variables: vec!["age".to_owned(), "sex".to_owned()],
      }
    );
    assert_eq!(
      parse_command("select\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
      Command::Select {
        variables: vec!["a,b".to_owned(), "old name".to_owned()],
      }
    );
    assert_eq!(
      parse_command("select age age").unwrap(),
      Command::Select {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_select_syntax_with_exact_diagnostics() {
    let cases = [
      ("select", "select expects at least one variable"),
      (
        "select age if age > 0",
        "select only accepts a variable list",
      ),
      ("select age, stable", "select only accepts a variable list"),
      ("select age = x", "select only accepts a variable list"),
      ("select = x", "select assignment requires a target before ="),
      (
        "select age =",
        "select assignment requires an expression after =",
      ),
      (
        "select age,",
        "comma must be followed by at least one option",
      ),
      ("select if", "missing expression after if"),
      ("select age==x", "unsupported token in command: =="),
      ("select age-1", "unsupported token in command: -"),
      ("select age+1", "unsupported token in command: +"),
      ("select age!x", "unsupported token in command: !"),
      ("select age@x", "unsupported token in command: @"),
      ("select:age", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_sort_variables_without_execution() {
    assert_eq!(
      parse_command(" SORT age label ").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "label".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort\u{1c}`a,b`\u{1d}\"old name\"").unwrap(),
      Command::Sort {
        variables: vec!["a,b".to_owned(), "old name".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort age age").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "age".to_owned()],
      }
    );
    assert_eq!(
      parse_command("sort age label now").unwrap(),
      Command::Sort {
        variables: vec!["age".to_owned(), "label".to_owned(), "now".to_owned()],
      }
    );
  }

  #[test]
  fn rejects_invalid_sort_syntax_with_exact_diagnostics() {
    let cases = [
      ("sort", "sort expects at least one variable"),
      ("sort age if age > 0", "sort only accepts a variable list"),
      ("sort age, stable", "sort only accepts a variable list"),
      ("sort age = x", "sort only accepts a variable list"),
      ("sort = x", "sort assignment requires a target before ="),
      (
        "sort age =",
        "sort assignment requires an expression after =",
      ),
      ("sort age,", "comma must be followed by at least one option"),
      ("sort if", "missing expression after if"),
      ("sort age==x", "unsupported token in command: =="),
      ("sort age-1", "unsupported token in command: -"),
      ("sort age+1", "unsupported token in command: +"),
      ("sort age!x", "unsupported token in command: !"),
      ("sort age@x", "unsupported token in command: @"),
      ("sort:age", "unsupported token in command: :"),
      ("sort age:label", "unsupported token in command: :"),
      ("sort age/label", "unsupported token in command: /"),
      ("sort age.label", "unsupported token in command: ."),
      ("sort +age", "unsupported token in command: +"),
      ("sort -age", "unsupported token in command: -"),
      ("sort !age", "unsupported token in command: !"),
      ("sort @age", "unsupported token in command: @"),
      ("sort ``", "quoted identifier cannot be empty"),
      ("sort \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_gsort_keys_without_execution() {
    assert_eq!(
      parse_command(" GSORT group_id -label ").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "label".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort\u{1c}+group_id\u{1d}-label").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "label".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort `-score` \"old name\"").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "-score".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "old name".to_owned(),
            descending: false,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort \"-score\"").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "score".to_owned(),
          descending: true,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort group_id group_id").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "group_id".to_owned(),
            descending: false,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort+age").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "age".to_owned(),
          descending: false,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort-age").unwrap(),
      Command::Gsort {
        keys: vec![SortKey {
          variable: "age".to_owned(),
          descending: true,
        }],
      }
    );
    assert_eq!(
      parse_command("gsort +age -age").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: "age".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "age".to_owned(),
            descending: true,
          },
        ],
      }
    );
    assert_eq!(
      parse_command("gsort:age age==x").unwrap(),
      Command::Gsort {
        keys: vec![
          SortKey {
            variable: ":age".to_owned(),
            descending: false,
          },
          SortKey {
            variable: "age==x".to_owned(),
            descending: false,
          },
        ],
      }
    );
  }

  #[test]
  fn rejects_invalid_gsort_syntax_with_exact_diagnostics() {
    let cases = [
      ("gsort", "gsort expects at least one variable"),
      (
        "gsort group_id if x > 0",
        "gsort only accepts a signed variable list",
      ),
      (
        "gsort group_id, stable",
        "gsort only accepts a signed variable list",
      ),
      (
        "gsort group_id = x",
        "gsort only accepts a signed variable list",
      ),
      ("gsort = x", "gsort assignment requires a target before ="),
      (
        "gsort group_id =",
        "gsort assignment requires an expression after =",
      ),
      (
        "gsort group_id,",
        "comma must be followed by at least one option",
      ),
      ("gsort,", "comma must be followed by at least one option"),
      ("gsort if", "missing expression after if"),
      (
        "gsort --group_id",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort ++label",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort +-label",
        "gsort keys must use at most one + or - prefix",
      ),
      (
        "gsort -",
        "gsort expects a variable after each direction prefix",
      ),
      (
        "gsort +",
        "gsort expects a variable after each direction prefix",
      ),
      (
        "gsort group_id -",
        "gsort expects a variable after each direction prefix",
      ),
      ("gsort age!x", "unsupported token in command: !"),
      ("gsort age@x", "unsupported token in command: @"),
      ("gsort!age", "unsupported token in command: !"),
      ("gsort@age", "unsupported token in command: @"),
      ("gsort?age", "unsupported token in command: ?"),
      ("gsort ``", "quoted identifier cannot be empty"),
      ("gsort \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_rename_names_without_execution() {
    assert_eq!(
      parse_command(" rename sex gender ").unwrap(),
      Command::Rename {
        old_name: "sex".to_owned(),
        new_name: "gender".to_owned(),
      }
    );
    assert_eq!(
      parse_command("RENAME\u{1c}  `old-name`\u{1d}\"new name\"").unwrap(),
      Command::Rename {
        old_name: "old-name".to_owned(),
        new_name: "new name".to_owned(),
      }
    );
    assert_eq!(
      parse_command("rename old old").unwrap(),
      Command::Rename {
        old_name: "old".to_owned(),
        new_name: "old".to_owned(),
      }
    );
  }

  #[test]
  fn rejects_invalid_rename_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "rename",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new now",
        "rename expects exactly two variables: rename old new",
      ),
      ("rename if", "missing expression after if"),
      ("rename old if", "missing expression after if"),
      ("rename old new if", "missing expression after if"),
      (
        "rename old if x > 0",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new if x > 0",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new, replace",
        "rename expects exactly two variables: rename old new",
      ),
      (
        "rename old new,",
        "comma must be followed by at least one option",
      ),
      (
        "rename=old new",
        "rename assignment requires a target before =",
      ),
      (
        "rename = old",
        "rename assignment requires a target before =",
      ),
      ("rename==old new", "unsupported token in command: =="),
      ("rename:old new", "unsupported token in command: :"),
      ("rename old-new new", "unsupported token in command: -"),
      ("rename old+new new", "unsupported token in command: +"),
      ("rename old@new new", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_run_path_without_execution() {
    assert_eq!(
      parse_command(" run analysis.td ").unwrap(),
      Command::Run {
        path: "analysis.td".to_owned(),
      }
    );
    assert_eq!(
      parse_command("RUN\u{1c}analysis.td").unwrap(),
      Command::Run {
        path: "analysis.td".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run \"analysis.td\"").unwrap(),
      Command::Run {
        path: "\"analysis.td\"".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run `analysis.td`").unwrap(),
      Command::Run {
        path: "`analysis.td`".to_owned(),
      }
    );
    assert_eq!(
      parse_command("run analysis.td,").unwrap(),
      Command::Run {
        path: "analysis.td,".to_owned(),
      }
    );
  }

  #[test]
  fn rejects_invalid_run_syntax_with_exact_diagnostics() {
    let cases = [
      ("run", "run expects exactly one path: run <script>"),
      ("run   ", "run expects exactly one path: run <script>"),
      (
        "run a.td b.td",
        "run expects exactly one path: run <script>",
      ),
      (
        "run \"a b.td\"",
        "run expects exactly one path: run <script>",
      ),
      (
        "run a.td if x > 0",
        "run expects exactly one path: run <script>",
      ),
      ("run,", "comma must be followed by at least one option"),
      ("run,foo", "unknown command: run"),
      ("run=foo", "run assignment requires a target before ="),
      ("run==foo", "unsupported token in command: =="),
      ("run:foo", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_save_and_export_paths_without_execution() {
    assert_eq!(
      parse_command(" SAVE output.parquet ").unwrap(),
      Command::Save {
        path: "output.parquet".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("export \"my output.parquet\", replace").unwrap(),
      Command::Export {
        path: "my output.parquet".to_owned(),
        replace: true,
      }
    );
    assert_eq!(
      parse_command("save `a,b`, replace replace").unwrap(),
      Command::Save {
        path: "a,b".to_owned(),
        replace: true,
      }
    );
    assert_eq!(
      parse_command("export a==b").unwrap(),
      Command::Export {
        path: "a==b".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("save:out.parquet").unwrap(),
      Command::Save {
        path: ":out.parquet".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("export/out.csv").unwrap(),
      Command::Export {
        path: "/out.csv".to_owned(),
        replace: false,
      }
    );
    assert_eq!(
      parse_command("save \"\"").unwrap(),
      Command::Save {
        path: String::new(),
        replace: false,
      }
    );
  }

  #[test]
  fn rejects_invalid_save_and_export_syntax_with_exact_diagnostics() {
    let cases = [
      ("save", "save expects exactly one path"),
      ("save one two", "save expects exactly one path"),
      ("export", "export expects exactly one path"),
      ("export one two", "export expects exactly one path"),
      (
        "save out if x > 0",
        "save does not accept if clauses or assignment syntax",
      ),
      (
        "export out = x",
        "export does not accept if clauses or assignment syntax",
      ),
      ("save if", "missing expression after if"),
      ("save = out", "save assignment requires a target before ="),
      (
        "export out =",
        "export assignment requires an expression after =",
      ),
      ("save out,", "comma must be followed by at least one option"),
      ("save out, force", "save unsupported option: force"),
      ("export out, REPLACE", "export unsupported option: REPLACE"),
      (
        "save out, replace=true",
        "save option replace does not accept a value",
      ),
      (
        "export out, replace(foo)",
        "export option replace does not accept a value",
      ),
      ("save out@x", "unsupported token in command: @"),
      ("export@out", "unsupported token in command: @"),
      (
        "save out, replace, replace",
        "option names must be identifiers",
      ),
      ("save ``, replace", "quoted identifier cannot be empty"),
      ("export \"unterminated", "unterminated quoted string"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_set_values_without_executing_configuration() {
    assert_eq!(
      parse_command(" SET GRAPH_FORMAT PnG ").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "PnG".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir artifacts/custom").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: "artifacts/custom".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_open \"Off\"").unwrap(),
      Command::Set {
        name: SettingName::GraphOpen,
        value: "Off".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir 'my plots'").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: "my plots".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_format `svg`").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "svg".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set artifact_dir \"\"").unwrap(),
      Command::Set {
        name: SettingName::ArtifactDir,
        value: String::new(),
      }
    );
    assert_eq!(
      parse_command("set graph_open maybe").unwrap(),
      Command::Set {
        name: SettingName::GraphOpen,
        value: "maybe".to_owned(),
      }
    );
    assert_eq!(
      parse_command("set graph_format foo==bar").unwrap(),
      Command::Set {
        name: SettingName::GraphFormat,
        value: "foo==bar".to_owned(),
      }
    );
    for value in ["foo<=bar", "foo>=bar", "<=foo", ">=foo"] {
      assert_eq!(
        parse_command(&format!("set graph_format {value}")).unwrap(),
        Command::Set {
          name: SettingName::GraphFormat,
          value: value.to_owned(),
        },
        "{value:?}"
      );
    }
    for (input, value) in [
      ("set graph_format foo`bar`", "foobar"),
      ("set graph_format \"a\"`b`", "ab"),
      ("set graph_format foo`bar`+x", "foobar+x"),
      ("set graph_format `foo`bar`baz`", "foobarbaz"),
    ] {
      assert_eq!(
        parse_command(input).unwrap(),
        Command::Set {
          name: SettingName::GraphFormat,
          value: value.to_owned(),
        },
        "{input:?}"
      );
    }
  }

  #[test]
  fn parses_inspection_commands_and_canonical_limits() {
    assert_eq!(parse_command("count").unwrap(), Command::Count);
    assert_eq!(parse_command(" COUNT ").unwrap(), Command::Count);
    assert_eq!(
      parse_command("head").unwrap(),
      Command::Head {
        limit: RowLimit::default(),
      }
    );
    assert_eq!(
      parse_command("TAIL 000").unwrap(),
      Command::Tail {
        limit: RowLimit("0".into()),
      }
    );
    let head = parse_command("head 00018446744073709551616").unwrap();
    assert_eq!(
      head,
      Command::Head {
        limit: RowLimit("18446744073709551616".into()),
      }
    );
    let huge = "9".repeat(100);
    let tail = parse_command(&format!("tail {huge}")).unwrap();
    assert_eq!(
      tail,
      Command::Tail {
        limit: RowLimit(huge.into_boxed_str()),
      }
    );
    assert_eq!(
      match parse_command("head 0007").unwrap() {
        Command::Head { limit } => limit.as_decimal().to_owned(),
        other => panic!("unexpected command: {other:?}"),
      },
      "7"
    );
  }

  #[test]
  fn parses_use_sources_modes_and_options() {
    assert_eq!(
      parse_command("use data.parquet").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("data.parquet".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: None,
        has_header: None,
      }
    );
    assert_eq!(
      parse_command("  USE\u{1c}s3://bucket/data.parquet, lazy  ").unwrap(),
      Command::Use {
        source: DataSource::Uri("s3://bucket/data.parquet".to_owned()),
        execution_mode: ExecutionMode::Lazy,
        lazy_engine: Some(LazyEngine::DuckDb),
        delimiter: None,
        has_header: None,
      }
    );
    assert_eq!(
      parse_command("use file.csv, lazy engine=POLARS delimiter=\";\" has_header(false)").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Lazy,
        lazy_engine: Some(LazyEngine::Polars),
        delimiter: Some(";".to_owned()),
        has_header: Some(false),
      }
    );
    assert_eq!(
      parse_command("use file.csv, has_header(TRUE) delimiter(\"\")").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: Some(String::new()),
        has_header: Some(true),
      }
    );
    assert_eq!(
      parse_command("use file.csv, has_header").unwrap(),
      Command::Use {
        source: DataSource::LocalPath("file.csv".to_owned()),
        execution_mode: ExecutionMode::Eager,
        lazy_engine: None,
        delimiter: None,
        has_header: Some(true),
      }
    );
  }

  #[test]
  fn rejects_invalid_use_syntax_with_exact_diagnostics() {
    let cases = [
      ("use", "use expects exactly one path: use <path>"),
      (
        "use one.parquet two.parquet",
        "use expects exactly one path: use <path>",
      ),
      (
        "use data.parquet,",
        "comma must be followed by at least one option",
      ),
      (
        "use data.parquet, lazy=true",
        "use lazy option does not accept a value",
      ),
      (
        "use data.parquet, lazy lazy",
        "use option specified more than once",
      ),
      (
        "use data.parquet, engine=duckdb",
        "use engine option requires lazy mode",
      ),
      (
        "use data.parquet, lazy engine=spark",
        "use engine must be duckdb or polars",
      ),
      (
        "use data.parquet, engine",
        "use engine option expects a string value",
      ),
      (
        "use data.parquet, engine=١",
        "use engine option expects a string value",
      ),
      (
        "use data.parquet, delimiter",
        "use delimiter option expects a string value",
      ),
      (
        "use data.parquet, has_header(1)",
        "option has_header expects true or false",
      ),
      ("use data.parquet, unknown", "unknown use option: unknown"),
      (
        "use data.parquet, delimiter()",
        "option delimiter expects at least one value",
      ),
      (
        "use data.parquet, engine()",
        "option engine expects at least one value",
      ),
      (
        "use data.parquet, has_header=",
        "option has_header requires a value after =",
      ),
      (
        "use data.parquet, delimiter(,)",
        "option delimiter expects a single string or identifier value",
      ),
      ("use data.parquet, alpha(1)", "unknown use option: alpha"),
      (
        "use data.parquet, alpha(foo)",
        "option alpha expects a numeric value",
      ),
      ("use data.parquet, saving(1)", "unknown use option: saving"),
      (
        "use data.parquet, prior(x)",
        "prior option expects prior(variable, distribution) syntax",
      ),
      (
        "use data.parquet, prior(x,normal)",
        "unknown use option: prior",
      ),
      (
        "use data.parquet, l1_ratio(foo)",
        "option l1_ratio values must be numeric",
      ),
      (
        "use data.parquet, unknown(1)",
        "option unknown values must be identifiers",
      ),
      (
        "use data.parquet, delimiter=;",
        "unsupported token in command: ;",
      ),
      (
        "use data.parquet, lazy,",
        "option names must be identifiers",
      ),
      ("use, lazy", "unknown command: use"),
      ("use,", "comma must be followed by at least one option"),
      ("use,,", "option names must be identifiers"),
      ("use=data", "use assignment requires a target before ="),
      ("use==data", "unsupported token in command: =="),
      ("use:data", "unsupported token in command: :"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn accepts_quoted_numeric_limits() {
    for (input, expected) in [
      ("head \"10\"", "10"),
      ("tail '0010'", "10"),
      ("head `0`", "0"),
    ] {
      let command = parse_command(input).unwrap();
      let actual = match command {
        Command::Head { limit } | Command::Tail { limit } => limit.as_decimal().to_owned(),
        other => panic!("unexpected command: {other:?}"),
      };
      assert_eq!(actual, expected);
    }
  }

  #[test]
  fn rejects_invalid_inspection_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "count 1",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "count if x",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "count, detail",
        "count does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("count = 1", "count assignment requires a target before ="),
      ("count == 1", "unsupported token in command: =="),
      ("count -1", "unsupported token in command: -"),
      ("head 5 6", "head accepts at most one row limit"),
      ("head 1.0", "head row limit must be a non-negative integer"),
      ("head 1e2", "head row limit must be a non-negative integer"),
      ("head foo", "head row limit must be a non-negative integer"),
      ("head ١", "head row limit must be a non-negative integer"),
      ("head -1", "unsupported token in command: -"),
      ("head +1", "unsupported token in command: +"),
      (
        "head 1 if x",
        "head does not accept if clauses, options, or assignment syntax",
      ),
      (
        "head 1, detail",
        "head does not accept if clauses, options, or assignment syntax",
      ),
      ("head = 1", "head assignment requires a target before ="),
      ("head == 1", "unsupported token in command: =="),
      ("head,", "comma must be followed by at least one option"),
      ("head 5,", "comma must be followed by at least one option"),
      ("tail 5 6", "tail accepts at most one row limit"),
      ("tail 1.0", "tail row limit must be a non-negative integer"),
      ("tail -1", "unsupported token in command: -"),
      ("tail +1", "unsupported token in command: +"),
      (
        "tail 1 if x",
        "tail does not accept if clauses, options, or assignment syntax",
      ),
      (
        "tail 1, detail",
        "tail does not accept if clauses, options, or assignment syntax",
      ),
      ("tail = 1", "tail assignment requires a target before ="),
      ("tail == 1", "unsupported token in command: =="),
      ("tail,", "comma must be followed by at least one option"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
    assert_eq!(
      parse_command("head \"\"").unwrap_err().to_string(),
      "head row limit must be a non-negative integer"
    );
    assert_eq!(
      parse_command("head ``").unwrap_err().to_string(),
      "quoted identifier cannot be empty"
    );
    assert_eq!(
      parse_command("head \"unterminated")
        .unwrap_err()
        .to_string(),
      "unterminated quoted string"
    );
  }

  #[test]
  fn rejects_invalid_describe_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "describe age",
        "describe does not accept arguments, if clauses, or options",
      ),
      (
        "describe if age > 18",
        "describe does not accept arguments, if clauses, or options",
      ),
      (
        "describe, detail",
        "describe does not accept arguments, if clauses, or options",
      ),
      ("describe,", "comma must be followed by at least one option"),
      (
        "describe age,",
        "comma must be followed by at least one option",
      ),
      (
        "describe=now",
        "describe assignment requires a target before =",
      ),
      ("describe == now", "unsupported token in command: =="),
      ("describe -1", "unsupported token in command: -"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_doctor_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "doctor foo",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "doctor if age > 18",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "doctor, detail",
        "doctor does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("doctor if", "missing expression after if"),
      ("doctor,", "comma must be followed by at least one option"),
      (
        "doctor foo,",
        "comma must be followed by at least one option",
      ),
      ("doctor=now", "doctor assignment requires a target before ="),
      ("doctor == now", "unsupported token in command: =="),
      ("doctor -1", "unsupported token in command: -"),
      ("doctor +1", "unsupported token in command: +"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_datasignature_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "datasignature age",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age > 0",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age==x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age+x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      (
        "datasignature if age-x",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("datasignature age if", "missing expression after if"),
      ("datasignature if, fast", "missing expression after if"),
      ("datasignature if,", "missing expression after if"),
      (
        "datasignature, fast",
        "datasignature does not accept arguments, if clauses, options, or assignment syntax",
      ),
      ("datasignature if", "missing expression after if"),
      (
        "datasignature,",
        "comma must be followed by at least one option",
      ),
      (
        "datasignature age,",
        "comma must be followed by at least one option",
      ),
      (
        "datasignature = value",
        "datasignature assignment requires a target before =",
      ),
      (
        "datasignature=value",
        "datasignature assignment requires a target before =",
      ),
      ("datasignature == now", "unsupported token in command: =="),
      ("datasignature -1", "unsupported token in command: -"),
      ("datasignature +1", "unsupported token in command: +"),
      ("datasignature age==x", "unsupported token in command: =="),
      ("datasignature age-1", "unsupported token in command: -"),
      ("datasignature age+1", "unsupported token in command: +"),
      ("datasignature age!x", "unsupported token in command: !"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_codebook_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "codebook age if age > 18",
        "codebook does not accept if clauses or options",
      ),
      (
        "codebook age, detail",
        "codebook does not accept if clauses or options",
      ),
      (
        "codebook age = 1",
        "codebook does not accept assignment syntax",
      ),
      (
        "codebook = 1",
        "codebook assignment requires a target before =",
      ),
      (
        "codebook age,",
        "comma must be followed by at least one option",
      ),
      ("codebook if", "missing expression after if"),
      ("codebook -1", "unsupported token in command: -"),
      ("codebook +1", "unsupported token in command: +"),
      ("codebook .", "unsupported token in command: ."),
      ("codebook age@x", "unsupported token in command: @"),
      ("codebook age#x", "unsupported token in command: #"),
      (
        "codebook if`foo`",
        "codebook does not accept if clauses or options",
      ),
      ("codebook age==x", "unsupported token in command: =="),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().message(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_missing_syntax_with_exact_diagnostics() {
    let cases = [
      (
        "missing age if age > 0",
        "missing does not accept if clauses or options",
      ),
      (
        "missing age, detail",
        "missing does not accept if clauses or options",
      ),
      (
        "missing age = other",
        "missing does not accept assignment syntax",
      ),
      (
        "missing = age",
        "missing assignment requires a target before =",
      ),
      (
        "missing age,",
        "comma must be followed by at least one option",
      ),
      ("missing,", "comma must be followed by at least one option"),
      ("missing if", "missing expression after if"),
      ("missing age==x", "unsupported token in command: =="),
      ("missing age-1", "unsupported token in command: -"),
      ("missing age+1", "unsupported token in command: +"),
      ("missing age!x", "unsupported token in command: !"),
      ("missing age@x", "unsupported token in command: @"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_invalid_set_syntax_with_exact_diagnostics() {
    let cases = [
      ("set", "set expects syntax: set name value"),
      ("set graph_format", "set expects syntax: set name value"),
      (
        "set graph_format png extra",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format png, detail",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format, detail",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format png,",
        "comma must be followed by at least one option",
      ),
      ("set unknown on", "unknown setting: unknown"),
      ("set `graph_format` png", "unknown setting: graph_format"),
      ("set = png", "set assignment requires a target before ="),
      ("set=png", "set assignment requires a target before ="),
      (
        "set graph_format = png",
        "set expects syntax: set name value",
      ),
      ("set if", "missing expression after if"),
      ("set graph_format if", "missing expression after if"),
      (
        "set graph_format foo?bar",
        "unsupported token in command: ?",
      ),
      (
        "set graph_format foo\\bar",
        "unsupported token in command: \\",
      ),
      (
        "set graph_format foo;bar",
        "unsupported token in command: ;",
      ),
      (
        "set graph_format foo@bar",
        "unsupported token in command: @",
      ),
      (
        "set graph_format foo$bar",
        "unsupported token in command: $",
      ),
      (
        "set graph_format foo😀bar",
        "unsupported token in command: 😀",
      ),
      (
        "set graph_format \"a\"\"b\"",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format 'a''b'",
        "set expects syntax: set name value",
      ),
      (
        "set graph_format foo\"bar\"",
        "set expects syntax: set name value",
      ),
      ("set graph_format foo-if", "missing expression after if"),
      ("set graph_format foo.if", "missing expression after if"),
      ("set graph_format \"foo\"if", "missing expression after if"),
      ("set graph_format `foo`if", "missing expression after if"),
    ];
    for (input, expected) in cases {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        expected,
        "{input:?}"
      );
    }
  }

  #[test]
  fn rejects_empty_unknown_and_quoted_commands() {
    assert_eq!(
      parse_command("  ").unwrap_err().to_string(),
      "empty command"
    );
    assert_eq!(
      parse_command("unknown").unwrap_err().to_string(),
      "unknown command: unknown"
    );
    assert_eq!(
      parse_command("ÄBC").unwrap_err().to_string(),
      "unknown command: äbc"
    );
    assert_eq!(
      parse_command("ΣTATUS").unwrap_err().to_string(),
      "unknown command: σtatus"
    );
    assert_eq!(
      parse_command("`help`").unwrap_err().to_string(),
      "command must start with an unquoted command name"
    );
  }

  #[test]
  fn rejects_extra_help_words() {
    assert_eq!(
      parse_command("help one two").unwrap_err(),
      ParseError::new("help expects at most one command name: help <command>")
    );
    assert_eq!(
      parse_command("? one two").unwrap_err(),
      ParseError::new("help expects at most one command name: help <command>")
    );
  }

  #[test]
  fn rejects_arguments_for_read_only_and_exit_commands() {
    assert_eq!(
      parse_command("status now").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    assert_eq!(
      parse_command("status, verbose").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    for (input, expected) in [
      ("status -1", "unsupported token in command: -"),
      ("status +1", "unsupported token in command: +"),
      ("status-1", "unsupported token in command: -"),
      ("status+1", "unsupported token in command: +"),
      ("status -", "unsupported token in command: -"),
      ("status +", "unsupported token in command: +"),
      ("status --1", "unsupported token in command: -"),
      ("status ++1", "unsupported token in command: +"),
      ("status -1,", "unsupported token in command: -"),
      ("status +1,", "unsupported token in command: +"),
      (
        "status if x",
        "status does not accept arguments, if clauses, options, or assignment syntax",
      ),
    ] {
      assert_eq!(parse_command(input).unwrap_err().to_string(), expected);
    }
    assert_eq!(
      parse_command("status if").unwrap_err().to_string(),
      "missing expression after if"
    );
    assert_eq!(
      parse_command("exit foo").unwrap_err().to_string(),
      "exit does not accept arguments, if clauses, or options"
    );
    assert_eq!(
      parse_command("quit, now").unwrap_err().to_string(),
      "quit does not accept arguments, if clauses, or options"
    );
    assert_eq!(
      parse_command("status=now").unwrap_err().to_string(),
      "status assignment requires a target before ="
    );
    assert_eq!(
      parse_command("exit=now").unwrap_err().to_string(),
      "exit assignment requires a target before ="
    );
    assert_eq!(
      parse_command("quit=now").unwrap_err().to_string(),
      "quit assignment requires a target before ="
    );
    assert_eq!(
      parse_command("status == now").unwrap_err().to_string(),
      "unsupported token in command: =="
    );
    assert_eq!(
      parse_command("exit == now").unwrap_err().to_string(),
      "unsupported token in command: =="
    );
    for input in ["status,", "status now,", "exit,", "quit ,"] {
      assert_eq!(
        parse_command(input).unwrap_err().to_string(),
        "comma must be followed by at least one option"
      );
    }
    assert_eq!(
      parse_command("help,verbose").unwrap_err().to_string(),
      "unknown command: help"
    );
    assert_eq!(
      parse_command("help , verbose").unwrap_err().to_string(),
      "help expects at most one command name: help <command>"
    );
  }

  #[test]
  fn matches_python_whitespace_and_malformed_quote_diagnostics() {
    assert_eq!(
      parse_command("\u{1c}").unwrap_err().to_string(),
      "empty command"
    );
    assert_eq!(
      parse_command("status\u{1c}now").unwrap_err().to_string(),
      "status does not accept arguments, if clauses, options, or assignment syntax"
    );
    assert_eq!(
      parse_command("help\u{1c},foo").unwrap(),
      Command::Help {
        topic: Some(",foo".to_owned()),
      }
    );
    assert_eq!(
      parse_command("\"").unwrap_err().to_string(),
      "unterminated quoted string"
    );
    assert_eq!(
      parse_command("'").unwrap_err().to_string(),
      "unterminated quoted string"
    );
    assert_eq!(
      parse_command("`").unwrap_err().to_string(),
      "unterminated quoted identifier"
    );
    assert_eq!(
      parse_command("``").unwrap_err().to_string(),
      "quoted identifier cannot be empty"
    );
    assert_eq!(
      parse_command("`foo``").unwrap_err().to_string(),
      "unterminated quoted identifier"
    );
    assert_eq!(
      parse_command("\"foo\"\"").unwrap_err().to_string(),
      "unterminated quoted string"
    );
  }
}
