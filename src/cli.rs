use std::fmt;
use std::path::{Path, PathBuf};

use tabdat_language::{Command, parse_command};
use tabdat_runtime::{RuntimeError, Session};

pub const USAGE: &str = "\
usage: tabdat [-h] [-v] [-c COMMAND] [-f FILE] [--config CONFIG] [--json]
              [--list-commands] [--list-command-effects] [--help-topic TOPIC]
              [--explain] [--describe-command COMMAND] [--mcp]
              [script]";

pub const HELP_TEXT: &str = "\
usage: tabdat [-h] [-v] [-c COMMAND] [-f FILE] [--config CONFIG] [--json]
              [--list-commands] [--list-command-effects] [--help-topic TOPIC]
              [--explain] [--describe-command COMMAND] [--mcp]
              [script]

positional arguments:
  script                run a TabDat script file and exit

options:
  -h, --help            show this help message and exit
  -v, --version         show program's version number and exit
  -c, --command COMMAND
                        run a command and exit
  -f, --file FILE       run a TabDat script file and exit
  --config CONFIG       load a TabDat TOML config file
  --json                emit versioned JSONL results for batch or script
                        execution
  --list-commands       emit the available command catalog; requires --json
  --list-command-effects
                        emit declared command effects; requires --json
  --help-topic TOPIC    emit one packaged help topic; requires --json
  --explain             parse one batch command without executing it; requires
                        --json
  --describe-command COMMAND
                        emit one command's schema; requires --json
  --mcp                 start the TabDat Model Context Protocol (MCP) server
                        on stdio
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
  pub version: bool,
  pub help: bool,
  pub json: bool,
  pub list_commands: bool,
  pub list_command_effects: bool,
  pub help_topic: Option<String>,
  pub explain: bool,
  pub describe_command: Option<String>,
  pub commands: Vec<String>,
  pub file: Option<PathBuf>,
  pub positional_script: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
  ExpectedArgument(&'static str),
  Conflict(&'static str),
  UnrecognizedArguments(Vec<String>),
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CliError::ExpectedArgument(arg) => write!(f, "argument {arg}: expected one argument"),
      CliError::Conflict(msg) => write!(f, "{msg}"),
      CliError::UnrecognizedArguments(args) => {
        write!(f, "unrecognized arguments: {}", args.join(" "))
      }
    }
  }
}

impl std::error::Error for CliError {}

pub fn parse_args<I, S>(args: I) -> Result<CliArgs, CliError>
where
  I: IntoIterator<Item = S>,
  S: AsRef<str>,
{
  let mut version = false;
  let mut help = false;
  let mut json = false;
  let mut list_commands = false;
  let mut list_command_effects = false;
  let mut help_topic = None;
  let mut explain = false;
  let mut describe_command = None;
  let mut commands = Vec::new();
  let mut file = None;
  let mut positional_script = None;
  let mut unrecognized = Vec::new();

  let mut iter = args.into_iter().peekable();
  while let Some(arg_ref) = iter.next() {
    let arg = arg_ref.as_ref();
    if arg == "-v" || arg == "--version" {
      version = true;
    } else if arg == "-h" || arg == "--help" {
      help = true;
    } else if arg == "--json" {
      json = true;
    } else if arg == "--list-commands" {
      list_commands = true;
    } else if arg == "--list-command-effects" {
      list_command_effects = true;
    } else if arg == "--explain" {
      explain = true;
    } else if arg == "--help-topic" {
      if let Some(val) = iter.next() {
        help_topic = Some(val.as_ref().to_string());
      } else {
        return Err(CliError::ExpectedArgument("--help-topic"));
      }
    } else if let Some(val) = arg.strip_prefix("--help-topic=") {
      if val.is_empty() {
        return Err(CliError::ExpectedArgument("--help-topic"));
      }
      help_topic = Some(val.to_string());
    } else if arg == "--describe-command" {
      if let Some(val) = iter.next() {
        describe_command = Some(val.as_ref().to_string());
      } else {
        return Err(CliError::ExpectedArgument("--describe-command"));
      }
    } else if let Some(val) = arg.strip_prefix("--describe-command=") {
      if val.is_empty() {
        return Err(CliError::ExpectedArgument("--describe-command"));
      }
      describe_command = Some(val.to_string());
    } else if arg == "-c" || arg == "--command" {
      if let Some(cmd) = iter.next() {
        commands.push(cmd.as_ref().to_string());
      } else {
        return Err(CliError::ExpectedArgument("-c/--command"));
      }
    } else if let Some(cmd) = arg.strip_prefix("-c") {
      if cmd.is_empty() {
        return Err(CliError::ExpectedArgument("-c/--command"));
      }
      commands.push(cmd.to_string());
    } else if let Some(cmd) = arg.strip_prefix("--command=") {
      if cmd.is_empty() {
        return Err(CliError::ExpectedArgument("-c/--command"));
      }
      commands.push(cmd.to_string());
    } else if arg == "-f" || arg == "--file" {
      if let Some(path) = iter.next() {
        file = Some(PathBuf::from(path.as_ref()));
      } else {
        return Err(CliError::ExpectedArgument("-f/--file"));
      }
    } else if let Some(path) = arg.strip_prefix("-f") {
      if path.is_empty() {
        return Err(CliError::ExpectedArgument("-f/--file"));
      }
      file = Some(PathBuf::from(path));
    } else if let Some(path) = arg.strip_prefix("--file=") {
      if path.is_empty() {
        return Err(CliError::ExpectedArgument("-f/--file"));
      }
      file = Some(PathBuf::from(path));
    } else if arg.starts_with('-') {
      unrecognized.push(arg.to_string());
    } else if positional_script.is_none() {
      positional_script = Some(PathBuf::from(arg));
    } else {
      unrecognized.push(arg.to_string());
    }
  }

  // Precedence: help and version exit cleanly before argument validation.
  if help {
    return Ok(CliArgs {
      version: false,
      help: true,
      json: false,
      list_commands: false,
      list_command_effects: false,
      help_topic: None,
      explain: false,
      describe_command: None,
      commands: Vec::new(),
      file: None,
      positional_script: None,
    });
  }

  if version {
    return Ok(CliArgs {
      version: true,
      help: false,
      json: false,
      list_commands: false,
      list_command_effects: false,
      help_topic: None,
      explain: false,
      describe_command: None,
      commands: Vec::new(),
      file: None,
      positional_script: None,
    });
  }

  if !unrecognized.is_empty() {
    return Err(CliError::UnrecognizedArguments(unrecognized));
  }

  // Requires --json validation
  if list_commands && !json {
    return Err(CliError::Conflict("--list-commands requires --json"));
  }
  if list_command_effects && !json {
    return Err(CliError::Conflict("--list-command-effects requires --json"));
  }
  if help_topic.is_some() && !json {
    return Err(CliError::Conflict("--help-topic requires --json"));
  }
  if explain && !json {
    return Err(CliError::Conflict("--explain requires --json"));
  }
  if describe_command.is_some() && !json {
    return Err(CliError::Conflict("--describe-command requires --json"));
  }

  // Conflict validation
  let has_commands = !commands.is_empty();
  let has_file = file.is_some();
  let has_script = positional_script.is_some();
  let has_exec = has_commands || has_file || has_script;

  if list_commands && (has_exec || describe_command.is_some()) {
    return Err(CliError::Conflict(
      "--list-commands cannot be combined with command, script, or describe execution",
    ));
  }
  if list_command_effects
    && (has_exec || list_commands || help_topic.is_some() || explain || describe_command.is_some())
  {
    return Err(CliError::Conflict(
      "--list-command-effects cannot be combined with another execution mode",
    ));
  }
  if help_topic.is_some() && (has_exec || list_commands || describe_command.is_some()) {
    return Err(CliError::Conflict(
      "--help-topic cannot be combined with command, script, command discovery, or describe execution",
    ));
  }
  if explain && (list_commands || help_topic.is_some() || describe_command.is_some()) {
    return Err(CliError::Conflict(
      "--explain cannot be combined with command discovery, help-topic retrieval, or describe execution",
    ));
  }
  if describe_command.is_some()
    && (has_exec || list_commands || list_command_effects || help_topic.is_some() || explain)
  {
    return Err(CliError::Conflict(
      "--describe-command cannot be combined with another execution mode",
    ));
  }
  if explain && (has_file || has_script) {
    return Err(CliError::Conflict(
      "--explain requires exactly one -c/--command",
    ));
  }
  if explain && commands.len() != 1 {
    return Err(CliError::Conflict(
      "--explain requires exactly one -c/--command",
    ));
  }
  if has_commands && (has_file || has_script) {
    return Err(CliError::Conflict(
      "-c/--command cannot be combined with script execution",
    ));
  }
  if has_file && has_script {
    return Err(CliError::Conflict(
      "-f/--file cannot be combined with a positional script",
    ));
  }
  let has_discovery = list_commands
    || list_command_effects
    || help_topic.is_some()
    || explain
    || describe_command.is_some();
  if json && !has_exec && !has_discovery {
    return Err(CliError::Conflict(
      "--json requires a command execution, script path, explain, or discovery/describe flag",
    ));
  }

  Ok(CliArgs {
    version,
    help,
    json,
    list_commands,
    list_command_effects,
    help_topic,
    explain,
    describe_command,
    commands,
    file,
    positional_script,
  })
}

pub fn run_cli(args: CliArgs) -> i32 {
  if args.help {
    print!("{HELP_TEXT}");
    return 0;
  }

  if args.version {
    println!("tabdat {}", env!("CARGO_PKG_VERSION"));
    return 0;
  }

  if args.list_commands {
    let catalog = crate::catalog::command_catalog_result();
    let envelope = crate::catalog::ResultEnvelope::new(catalog, "CommandCatalogResult");
    println!("{}", serde_json::to_string(&envelope).unwrap());
    return 0;
  }

  if args.list_command_effects {
    let effects = crate::catalog::command_effect_catalog_result();
    let envelope = crate::catalog::ResultEnvelope::new(effects, "CommandEffectCatalogResult");
    println!("{}", serde_json::to_string(&envelope).unwrap());
    return 0;
  }

  if let Some(name) = &args.describe_command {
    return match crate::catalog::describe_command_result(name) {
      Ok(schema) => {
        let envelope = crate::catalog::ResultEnvelope::new(schema, "CommandSchemaResult");
        println!("{}", serde_json::to_string(&envelope).unwrap());
        0
      }
      Err(err) => {
        eprintln!("Error: {err}");
        let envelope = crate::catalog::ErrorEnvelope::new(err, "TabDatError");
        println!("{}", serde_json::to_string(&envelope).unwrap());
        1
      }
    };
  }

  if let Some(topic) = &args.help_topic {
    return match crate::catalog::help_topic_result(topic) {
      Ok(res) => {
        let envelope = crate::catalog::ResultEnvelope::new(res, "HelpTopicResult");
        println!("{}", serde_json::to_string(&envelope).unwrap());
        0
      }
      Err(err) => {
        eprintln!("Error: {err}");
        let envelope = crate::catalog::ErrorEnvelope::new(err, "TabDatError");
        println!("{}", serde_json::to_string(&envelope).unwrap());
        1
      }
    };
  }

  if args.explain {
    let cmd_text = &args.commands[0];
    return match crate::catalog::explain_result(cmd_text) {
      Ok(res) => {
        let envelope = crate::catalog::ResultEnvelope::new(res, "CommandExplainResult");
        println!("{}", serde_json::to_string(&envelope).unwrap());
        0
      }
      Err((err, err_type)) => {
        eprintln!("Error: {err}");
        let envelope = crate::catalog::ErrorEnvelope::new(err, err_type);
        println!("{}", serde_json::to_string(&envelope).unwrap());
        1
      }
    };
  }

  if !args.commands.is_empty() {
    let mut session = Session::new();
    for cmd_text in &args.commands {
      let command = match parse_command(cmd_text) {
        Ok(cmd) => cmd,
        Err(err) => {
          eprintln!("Error: {err}");
          if args.json {
            let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ParseError");
            println!("{}", serde_json::to_string(&envelope).unwrap());
          }
          return 2;
        }
      };
      if command == Command::Exit {
        break;
      }
      if let Err(err) = session.execute(command) {
        eprintln!("Error: {err}");
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
          println!("{}", serde_json::to_string(&envelope).unwrap());
        }
        return 1;
      }
    }
    return 0;
  }

  let script_path = args.file.or(args.positional_script);
  if let Some(path) = script_path {
    if path == Path::new("doctor") && !path.exists() {
      let mut session = Session::new();
      if let Err(err) = session.execute(Command::Doctor) {
        eprintln!("Error: {err}");
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
          println!("{}", serde_json::to_string(&envelope).unwrap());
        }
        return 1;
      }
      return 0;
    }

    let mut session = Session::new();
    return match session.execute_run(&path) {
      Ok(_) => 0,
      Err(RuntimeError::ScriptError(err)) => {
        eprintln!("Error: {err}");
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::with_location(
            err.message(),
            "ScriptError",
            err.path().to_string_lossy(),
            err.line(),
          );
          println!("{}", serde_json::to_string(&envelope).unwrap());
        }
        let msg = err.message();
        if msg.contains("not found") || msg.contains("could not read script") {
          3
        } else if msg.contains("parse") || msg.contains("syntax") || msg.contains("expected") {
          2
        } else {
          1
        }
      }
      Err(err) => {
        eprintln!("Error: {err}");
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
          println!("{}", serde_json::to_string(&envelope).unwrap());
        }
        1
      }
    };
  }

  // Fallback for no arguments: preserve scaffold behavior until interactive shell (§7.2).
  println!("Hello, world!");
  0
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_version() {
    let parsed = parse_args(["-v"]).unwrap();
    assert!(parsed.version);
    assert!(!parsed.help);

    let parsed2 = parse_args(["--version"]).unwrap();
    assert!(parsed2.version);
    assert!(!parsed2.help);
  }

  #[test]
  fn test_parse_help() {
    let parsed = parse_args(["-h"]).unwrap();
    assert!(parsed.help);
    assert!(!parsed.version);

    let parsed2 = parse_args(["--help"]).unwrap();
    assert!(parsed2.help);
    assert!(!parsed2.version);
  }

  #[test]
  fn test_parse_command_single_and_repeated() {
    let parsed = parse_args(["-c", "describe"]).unwrap();
    assert_eq!(parsed.commands, vec!["describe"]);

    let parsed2 = parse_args(["-c", "use data.csv", "--command", "count"]).unwrap();
    assert_eq!(parsed2.commands, vec!["use data.csv", "count"]);

    let parsed3 = parse_args(["-cuse data.csv", "-ccount"]).unwrap();
    assert_eq!(parsed3.commands, vec!["use data.csv", "count"]);
  }

  #[test]
  fn test_parse_file_flag_and_positional() {
    let parsed = parse_args(["-f", "script.td"]).unwrap();
    assert_eq!(parsed.file, Some(PathBuf::from("script.td")));
    assert_eq!(parsed.positional_script, None);

    let parsed2 = parse_args(["script.td"]).unwrap();
    assert_eq!(parsed2.file, None);
    assert_eq!(parsed2.positional_script, Some(PathBuf::from("script.td")));

    let parsed3 = parse_args(["-fscript.td"]).unwrap();
    assert_eq!(parsed3.file, Some(PathBuf::from("script.td")));
  }

  #[test]
  fn test_conflict_command_and_script() {
    let err1 = parse_args(["-c", "describe", "-f", "script.td"]).unwrap_err();
    assert_eq!(
      err1,
      CliError::Conflict("-c/--command cannot be combined with script execution")
    );

    let err2 = parse_args(["-c", "describe", "script.td"]).unwrap_err();
    assert_eq!(
      err2,
      CliError::Conflict("-c/--command cannot be combined with script execution")
    );
  }

  #[test]
  fn test_conflict_file_and_positional() {
    let err = parse_args(["-f", "script1.td", "script2.td"]).unwrap_err();
    assert_eq!(
      err,
      CliError::Conflict("-f/--file cannot be combined with a positional script")
    );
  }

  #[test]
  fn test_missing_argument() {
    assert_eq!(
      parse_args(["-c"]).unwrap_err(),
      CliError::ExpectedArgument("-c/--command")
    );
    assert_eq!(
      parse_args(["--command"]).unwrap_err(),
      CliError::ExpectedArgument("-c/--command")
    );
    assert_eq!(
      parse_args(["-f"]).unwrap_err(),
      CliError::ExpectedArgument("-f/--file")
    );
    assert_eq!(
      parse_args(["--file"]).unwrap_err(),
      CliError::ExpectedArgument("-f/--file")
    );
    assert_eq!(
      parse_args(["--help-topic"]).unwrap_err(),
      CliError::ExpectedArgument("--help-topic")
    );
    assert_eq!(
      parse_args(["--describe-command"]).unwrap_err(),
      CliError::ExpectedArgument("--describe-command")
    );
  }

  #[test]
  fn test_unrecognized_argument() {
    let err = parse_args(["--unknown-flag"]).unwrap_err();
    assert_eq!(
      err,
      CliError::UnrecognizedArguments(vec!["--unknown-flag".to_string()])
    );
  }

  #[test]
  fn test_help_precedence_over_unrecognized() {
    let parsed = parse_args(["--unknown-flag", "--help"]).unwrap();
    assert!(parsed.help);
  }

  #[test]
  fn test_version_precedence_over_unrecognized() {
    let parsed = parse_args(["--unknown-flag", "--version"]).unwrap();
    assert!(parsed.version);
  }

  #[test]
  fn test_json_requires_target() {
    let err = parse_args(["--json"]).unwrap_err();
    assert_eq!(
      err,
      CliError::Conflict(
        "--json requires a command execution, script path, explain, or discovery/describe flag"
      )
    );
  }

  #[test]
  fn test_discovery_flags_require_json() {
    assert_eq!(
      parse_args(["--list-commands"]).unwrap_err(),
      CliError::Conflict("--list-commands requires --json")
    );
    assert_eq!(
      parse_args(["--list-command-effects"]).unwrap_err(),
      CliError::Conflict("--list-command-effects requires --json")
    );
    assert_eq!(
      parse_args(["--help-topic", "summarize"]).unwrap_err(),
      CliError::Conflict("--help-topic requires --json")
    );
    assert_eq!(
      parse_args(["--describe-command", "summarize"]).unwrap_err(),
      CliError::Conflict("--describe-command requires --json")
    );
    assert_eq!(
      parse_args(["--explain", "-c", "count"]).unwrap_err(),
      CliError::Conflict("--explain requires --json")
    );
  }

  #[test]
  fn test_explain_requires_exactly_one_command() {
    assert_eq!(
      parse_args(["--json", "--explain"]).unwrap_err(),
      CliError::Conflict("--explain requires exactly one -c/--command")
    );
    assert_eq!(
      parse_args(["--json", "--explain", "-c", "count", "-c", "status"]).unwrap_err(),
      CliError::Conflict("--explain requires exactly one -c/--command")
    );
    assert_eq!(
      parse_args(["--json", "--explain", "-f", "commands.td"]).unwrap_err(),
      CliError::Conflict("--explain requires exactly one -c/--command")
    );
  }

  #[test]
  fn test_discovery_conflict_rejections() {
    assert_eq!(
      parse_args(["--json", "--list-commands", "-c", "count"]).unwrap_err(),
      CliError::Conflict(
        "--list-commands cannot be combined with command, script, or describe execution"
      )
    );
    assert_eq!(
      parse_args(["--json", "--list-command-effects", "-c", "count"]).unwrap_err(),
      CliError::Conflict("--list-command-effects cannot be combined with another execution mode")
    );
    assert_eq!(
      parse_args(["--json", "--help-topic", "summarize", "-c", "count"]).unwrap_err(),
      CliError::Conflict(
        "--help-topic cannot be combined with command, script, command discovery, or describe execution"
      )
    );
    assert_eq!(
      parse_args(["--json", "--describe-command", "summarize", "-c", "count"]).unwrap_err(),
      CliError::Conflict("--describe-command cannot be combined with another execution mode")
    );
    assert_eq!(
      parse_args(["--json", "--explain", "--list-commands"]).unwrap_err(),
      CliError::Conflict(
        "--explain cannot be combined with command discovery, help-topic retrieval, or describe execution"
      )
    );
  }
}
