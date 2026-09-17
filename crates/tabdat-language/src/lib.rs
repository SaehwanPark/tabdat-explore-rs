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
}

/// A deterministic error produced while parsing a command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
  message: String,
}

fn is_command_whitespace(character: char) -> bool {
  character.is_whitespace() || matches!(character, '\u{1c}'..='\u{1f}')
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

  match command.as_bytes().first() {
    Some(b'`') if !command[1..].contains('`') => {
      return Err(ParseError::new("unterminated quoted identifier"));
    }
    Some(b'\'' | b'"') if !command[1..].contains(command.as_bytes()[0] as char) => {
      return Err(ParseError::new("unterminated quoted string"));
    }
    Some(b'`' | b'\'' | b'"') => {
      return Err(ParseError::new(
        "command must start with an unquoted command name",
      ));
    }
    _ => {}
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
  if name.eq_ignore_ascii_case("help") && !delimiter.is_whitespace() {
    return Err(ParseError::new("unknown command: help"));
  }
  parse_named_command(name, body)
}

fn parse_named_command(name: &str, body: &str) -> Result<Command, ParseError> {
  let normalized_name = name.to_ascii_lowercase();
  match normalized_name.as_str() {
    "help" => parse_help(body),
    "status" => {
      if body.is_empty() {
        Ok(Command::Status)
      } else if body.starts_with('=') && !body.starts_with("==") {
        Err(ParseError::new(
          "status assignment requires a target before =",
        ))
      } else if body.starts_with("==") {
        Err(ParseError::new("unsupported token in command: =="))
      } else {
        Err(ParseError::new(
          "status does not accept arguments, if clauses, options, or assignment syntax",
        ))
      }
    }
    "exit" | "quit" => {
      if body.is_empty() {
        Ok(Command::Exit)
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
  use super::{Command, ParseError, parse_command};

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
  }
}
