use std::cell::RefCell;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tabdat_execution::{RuntimeError, Session};
use tabdat_language::{Command, parse_command};

pub const USAGE: &str = "\
usage: tabdat [-h] [-v] [-c COMMAND] [-f FILE] [--json | --text]
              [--list-commands] [--list-command-effects] [--help-topic TOPIC]
              [--explain] [--describe-command COMMAND]
              [script]";

pub const HELP_TEXT: &str = "\
usage: tabdat [-h] [-v] [-c COMMAND] [-f FILE] [--json | --text]
              [--list-commands] [--list-command-effects] [--help-topic TOPIC]
              [--explain] [--describe-command COMMAND]
              [script]

With no arguments, start a persistent line-oriented session.
Type exit/quit or send EOF to leave. Capable terminals provide editing,
session-local history and Ctrl-C input cancellation (not in-flight cancellation).

positional arguments:
  script                run a TabDat script file and exit

options:
  -h, --help            show this help message and exit
  -v, --version         show program's version number and exit
  -c, --command COMMAND
                        run a command and exit
  -f, --file FILE       run a TabDat script file and exit
  --text                render readable batch/script results; requires -c or a
                        script path; cannot be combined with --json/discovery
  --json                emit versioned JSON envelopes (batch success output
                        supports use/describe/count/codebook/summarize/missing/tabulate/head/tail,
                        by ...: count, collapse count
                        and generate/replace/rename/recode/label/encode/decode/sql/join/append/reshape,
                        keep/drop/select variable lists, bounded keep/drop if,
                        no-option summarize if, duplicates/isid, assert/datasignature,
                        sort/gsort,
                        regress/predict/lincom/test/ttest/estat vif, save/export;
                        scripts stream the same successful leaf results)
  --list-commands       emit the available command catalog; requires --json
  --list-command-effects
                        emit declared command effects; requires --json
  --help-topic TOPIC    emit one packaged help topic; requires --json
  --explain             parse one batch command without executing it; requires
                        --json
  --describe-command COMMAND
                        emit one command's schema; requires --json

Configuration-file loading and an MCP transport/server are not installed.
The typed transport-independent MCP boundary does not start a server.
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
  pub version: bool,
  pub help: bool,
  pub json: bool,
  pub text: bool,
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
  let mut text = false;
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
    } else if arg == "--text" {
      text = true;
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
      text: false,
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
      text: false,
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

  if text && json {
    return Err(CliError::Conflict("--text cannot be combined with --json"));
  }
  let has_discovery = list_commands
    || list_command_effects
    || help_topic.is_some()
    || explain
    || describe_command.is_some();
  if text && has_discovery {
    return Err(CliError::Conflict(
      "--text cannot be combined with explain or discovery/describe flags",
    ));
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
  if text && !has_exec {
    return Err(CliError::Conflict(
      "--text requires -c/--command or a script path; the interactive session is already readable",
    ));
  }
  if json && !has_exec && !has_discovery {
    return Err(CliError::Conflict(
      "--json requires a command execution, script path, explain, or discovery/describe flag",
    ));
  }

  Ok(CliArgs {
    version,
    help,
    json,
    text,
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

fn emit_cli_line(writer: &mut dyn Write, text: impl fmt::Display) -> io::Result<()> {
  writeln!(writer, "{text}")?;
  // Detect buffered failures before the next command can have effects.
  writer.flush()
}

fn emit_text_result(
  output: &mut dyn Write,
  result: &tabdat_domain::ExecutionResult,
) -> io::Result<()> {
  write!(
    output,
    "{}",
    tabdat_reporting::terminal::render_execution_result(result)
  )?;
  output.flush()
}

fn execute_reported_script(
  session: &mut Session,
  path: &Path,
  json: bool,
  output: &mut dyn Write,
) -> io::Result<Result<tabdat_domain::RunResult, RuntimeError>> {
  let output_error = RefCell::new(None);
  let result = session.execute_run_with_callbacks(
    path,
    |command| {
      if output_error.borrow().is_some() {
        Err("CLI output failed".to_owned())
      } else if json && !crate::json_results::supports_script_command(command) {
        Err(crate::json_results::SCRIPT_UNSUPPORTED_MESSAGE.to_owned())
      } else {
        Ok(())
      }
    },
    |result| {
      let emission = if json {
        let envelope = crate::json_results::serialize_execution_result(result)
          .expect("JSON script preflight permits only serializable leaf results");
        emit_cli_line(output, envelope)
      } else {
        emit_text_result(output, result)
      };
      if let Err(error) = emission {
        *output_error.borrow_mut() = Some(error);
      }
    },
  );
  // Keep the original IO failure rather than its subsequent located guard error.
  match output_error.into_inner() {
    Some(error) => Err(error),
    None => Ok(result),
  }
}

fn json_result_unsupported(output: &mut dyn Write, errors: &mut dyn Write) -> io::Result<i32> {
  const MESSAGE: &str = crate::json_results::BATCH_UNSUPPORTED_MESSAGE;
  emit_cli_line(errors, format_args!("Error: {MESSAGE}"))?;
  let envelope = crate::catalog::ErrorEnvelope::new(MESSAGE, "JsonResultUnsupported");
  emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
  Ok(2)
}

fn run_cli_with_output(
  args: CliArgs,
  output: &mut dyn Write,
  errors: &mut dyn Write,
) -> io::Result<i32> {
  if args.help {
    output.write_all(HELP_TEXT.as_bytes())?;
    output.flush()?;
    return Ok(0);
  }

  if args.version {
    emit_cli_line(output, format_args!("tabdat {}", env!("CARGO_PKG_VERSION")))?;
    return Ok(0);
  }

  if args.list_commands {
    let catalog = crate::catalog::command_catalog_result();
    let envelope = crate::catalog::ResultEnvelope::new(catalog, "CommandCatalogResult");
    emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
    return Ok(0);
  }

  if args.list_command_effects {
    let effects = crate::catalog::command_effect_catalog_result();
    let envelope = crate::catalog::ResultEnvelope::new(effects, "CommandEffectCatalogResult");
    emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
    return Ok(0);
  }

  if let Some(name) = &args.describe_command {
    return Ok(match crate::catalog::describe_command_result(name) {
      Ok(schema) => {
        let envelope = crate::catalog::ResultEnvelope::new(schema, "CommandSchemaResult");
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        0
      }
      Err(err) => {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        let envelope = crate::catalog::ErrorEnvelope::new(err, "TabDatError");
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        1
      }
    });
  }

  if let Some(topic) = &args.help_topic {
    return Ok(match crate::catalog::help_topic_result(topic) {
      Ok(res) => {
        let envelope = crate::catalog::ResultEnvelope::new(res, "HelpTopicResult");
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        0
      }
      Err(err) => {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        let envelope = crate::catalog::ErrorEnvelope::new(err, "TabDatError");
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        1
      }
    });
  }

  if args.explain {
    let cmd_text = &args.commands[0];
    return Ok(match crate::catalog::explain_result(cmd_text) {
      Ok(res) => {
        let envelope = crate::catalog::ResultEnvelope::new(res, "CommandExplainResult");
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        0
      }
      Err((err, err_type)) => {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        let envelope = crate::catalog::ErrorEnvelope::new(err, err_type);
        emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        1
      }
    });
  }

  if !args.commands.is_empty() {
    let mut session = Session::new();
    for cmd_text in &args.commands {
      let command = match parse_command(cmd_text) {
        Ok(cmd) => cmd,
        Err(err) => {
          emit_cli_line(errors, format_args!("Error: {err}"))?;
          if args.json {
            let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ParseError");
            emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
          }
          return Ok(2);
        }
      };
      if command == Command::Exit {
        break;
      }
      if args.json && !crate::json_results::supports_batch_command(&command) {
        return json_result_unsupported(output, errors);
      }
      let execution = match command {
        Command::Run { path } if args.text => {
          execute_reported_script(&mut session, Path::new(&path), false, output)?
            .map(tabdat_domain::ExecutionResult::Run)
        }
        command => session.execute(command),
      };
      let result = match execution {
        Ok(result) => result,
        Err(err) => {
          emit_cli_line(errors, format_args!("Error: {err}"))?;
          if args.json {
            let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
            emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
          }
          return Ok(1);
        }
      };
      if args.json {
        if let Some(envelope) = crate::json_results::serialize_execution_result(&result) {
          emit_cli_line(output, envelope)?;
        } else {
          return json_result_unsupported(output, errors);
        }
      } else if args.text {
        emit_text_result(output, &result)?;
      }
    }
    return Ok(0);
  }

  let script_path = args.file.or(args.positional_script);
  if let Some(path) = script_path {
    if path == Path::new("doctor") && !path.exists() {
      let mut session = Session::new();
      if let Err(err) = session.execute(Command::Doctor) {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
          emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        }
        return Ok(1);
      }
      return Ok(0);
    }

    let mut session = Session::new();
    let result = if args.json || args.text {
      execute_reported_script(&mut session, &path, args.json, output)?.map(|_| ())
    } else {
      session.execute_run(&path).map(|_| ())
    };
    return Ok(match result {
      Ok(_) => 0,
      Err(RuntimeError::ScriptError(err)) => {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::with_location(
            err.message(),
            "ScriptError",
            err.path().to_string_lossy(),
            err.line(),
          );
          emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        }
        match err.kind() {
          tabdat_domain::ScriptErrorKind::Execution => 1,
          tabdat_domain::ScriptErrorKind::Parse => 2,
          tabdat_domain::ScriptErrorKind::File => 3,
        }
      }
      Err(err) => {
        emit_cli_line(errors, format_args!("Error: {err}"))?;
        if args.json {
          let envelope = crate::catalog::ErrorEnvelope::new(err.to_string(), "ExecutionError");
          emit_cli_line(output, serde_json::to_string(&envelope).unwrap())?;
        }
        1
      }
    });
  }

  Ok(crate::interactive::run_interactive())
}

/// Run parsed CLI arguments. Noninteractive write/flush failures return 1;
/// diagnostics are best-effort and completed command effects are not rolled back.
pub fn run_cli(args: CliArgs) -> i32 {
  let mut output = io::stdout();
  let mut errors = io::stderr();
  match run_cli_with_output(args, &mut output, &mut errors) {
    Ok(code) => code,
    Err(error) => {
      // stderr may itself be broken; reporting is best-effort, exit 1 is not.
      let _ = emit_cli_line(&mut errors, format_args!("Error: CLI I/O failed: {error}"));
      1
    }
  }
}

#[cfg(test)]
#[path = "cli_output_tests.rs"]
mod output_tests;

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
