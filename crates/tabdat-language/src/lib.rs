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
  /// Count rows in the active dataset (execution is deferred).
  Count,
  /// Preview the first `limit` rows (execution is deferred).
  Head { limit: RowLimit },
  /// Preview the last `limit` rows (execution is deferred).
  Tail { limit: RowLimit },
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
  if name.eq_ignore_ascii_case("help") && !is_command_whitespace(delimiter) {
    return Err(ParseError::new("unknown command: help"));
  }
  parse_named_command(name, body)
}

fn parse_named_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let normalized_name = name.to_lowercase();
  match normalized_name.as_str() {
    "help" => parse_help(body),
    "status" => {
      if body.is_empty() {
        Ok(Command::Status)
      } else if body.trim_matches(is_command_whitespace).ends_with(',') {
        Err(ParseError::new(
          "comma must be followed by at least one option",
        ))
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "status assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else if body.starts_with('-') {
        Err(ParseError::new("unsupported token in command: -"))
      } else if body.starts_with('+') {
        Err(ParseError::new("unsupported token in command: +"))
      } else {
        Err(ParseError::new(
          "status does not accept arguments, if clauses, options, or assignment syntax",
        ))
      }
    }
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

#[derive(Debug)]
struct SimpleArgument {
  text: String,
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
  let parts = parse_simple_body(body)?;
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

fn parse_simple_body(body: &str) -> Result<SimpleBody, ParseError> {
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
      '=' => {
        if characters.get(index + 1) == Some(&'=') {
          return Err(ParseError::new("unsupported token in command: =="));
        }
        parts.has_assignment = true;
        parts.assignment_target_missing = parts.arguments.is_empty();
        break;
      }
      '+' | '-' | ':' | '/' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '*' | '%' | '&' | '|'
      | '!' | '<' | '>' | '^' | '~' | '#' => {
        return Err(ParseError::new(format!(
          "unsupported token in command: {}",
          characters[index]
        )));
      }
      _ => {
        let mut text = String::new();
        let mut quoted = false;
        loop {
          if index >= characters.len()
            || is_command_whitespace(characters[index])
            || matches!(characters[index], ',' | '=')
          {
            break;
          }
          if matches!(
            characters[index],
            '+'
              | '-'
              | ':'
              | '/'
              | '?'
              | '('
              | ')'
              | '['
              | ']'
              | '{'
              | '}'
              | '*'
              | '%'
              | '&'
              | '|'
              | '!'
              | '<'
              | '>'
              | '^'
              | '~'
              | '#'
          ) {
            return Err(ParseError::new(format!(
              "unsupported token in command: {}",
              characters[index]
            )));
          }
          if characters[index] == '.'
            && !text.is_empty()
            && !text
              .chars()
              .next()
              .is_some_and(|character| character.is_ascii_digit())
          {
            return Err(ParseError::new("unsupported token in command: ."));
          }
          if matches!(characters[index], '\'' | '"' | '`') {
            let quote = characters[index];
            quoted = true;
            let piece = parse_quoted_piece(&characters, &mut index, quote)?;
            text.push_str(&piece);
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
          parts.arguments.push(SimpleArgument { text });
        }
      }
    }
  }
  Ok(parts)
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
      if characters.get(*index + 1) == Some(&quote) {
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
  use super::{Command, ParseError, RowLimit, parse_command};

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
