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
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
  pub version: bool,
  pub help: bool,
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
      commands: Vec::new(),
      file: None,
      positional_script: None,
    });
  }

  if version {
    return Ok(CliArgs {
      version: true,
      help: false,
      commands: Vec::new(),
      file: None,
      positional_script: None,
    });
  }

  if !unrecognized.is_empty() {
    return Err(CliError::UnrecognizedArguments(unrecognized));
  }

  if !commands.is_empty() && (file.is_some() || positional_script.is_some()) {
    return Err(CliError::Conflict(
      "-c/--command cannot be combined with script execution",
    ));
  }

  if file.is_some() && positional_script.is_some() {
    return Err(CliError::Conflict(
      "-f/--file cannot be combined with a positional script",
    ));
  }

  Ok(CliArgs {
    version,
    help,
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

  if !args.commands.is_empty() {
    let mut session = Session::new();
    for cmd_text in &args.commands {
      let command = match parse_command(cmd_text) {
        Ok(cmd) => cmd,
        Err(err) => {
          eprintln!("Error: {err}");
          return 2;
        }
      };
      if command == Command::Exit {
        break;
      }
      if let Err(err) = session.execute(command) {
        eprintln!("Error: {err}");
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
        return 1;
      }
      return 0;
    }

    let mut session = Session::new();
    return match session.execute_run(&path) {
      Ok(_) => 0,
      Err(RuntimeError::ScriptError(err)) => {
        eprintln!("Error: {err}");
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
  }

  #[test]
  fn test_parse_help() {
    let parsed = parse_args(["-h"]).unwrap();
    assert!(parsed.help);
    assert!(!parsed.version);

    let parsed2 = parse_args(["--help"]).unwrap();
    assert!(parsed2.help);
  }

  #[test]
  fn test_parse_command_single_and_repeated() {
    let parsed = parse_args(["-c", "use test.parquet", "-c", "count"]).unwrap();
    assert_eq!(parsed.commands, vec!["use test.parquet", "count"]);
    assert_eq!(parsed.file, None);
    assert_eq!(parsed.positional_script, None);

    let parsed2 = parse_args(["--command", "describe"]).unwrap();
    assert_eq!(parsed2.commands, vec!["describe"]);
  }

  #[test]
  fn test_parse_file_flag_and_positional() {
    let parsed = parse_args(["-f", "test.td"]).unwrap();
    assert_eq!(parsed.file, Some(PathBuf::from("test.td")));
    assert_eq!(parsed.positional_script, None);

    let parsed2 = parse_args(["script.td"]).unwrap();
    assert_eq!(parsed2.file, None);
    assert_eq!(parsed2.positional_script, Some(PathBuf::from("script.td")));
  }

  #[test]
  fn test_conflict_command_and_script() {
    let err = parse_args(["-c", "count", "-f", "test.td"]).unwrap_err();
    assert_eq!(
      err,
      CliError::Conflict("-c/--command cannot be combined with script execution")
    );

    let err2 = parse_args(["-c", "count", "test.td"]).unwrap_err();
    assert_eq!(
      err2,
      CliError::Conflict("-c/--command cannot be combined with script execution")
    );
  }

  #[test]
  fn test_conflict_file_and_positional() {
    let err = parse_args(["-f", "test1.td", "test2.td"]).unwrap_err();
    assert_eq!(
      err,
      CliError::Conflict("-f/--file cannot be combined with a positional script")
    );
  }

  #[test]
  fn test_missing_argument() {
    let err = parse_args(["-c"]).unwrap_err();
    assert_eq!(err, CliError::ExpectedArgument("-c/--command"));

    let err2 = parse_args(["-f"]).unwrap_err();
    assert_eq!(err2, CliError::ExpectedArgument("-f/--file"));
  }

  #[test]
  fn test_unrecognized_argument() {
    let err = parse_args(["--unknown"]).unwrap_err();
    assert_eq!(
      err,
      CliError::UnrecognizedArguments(vec!["--unknown".to_string()])
    );

    let err2 = parse_args(["foo.td", "bar.td"]).unwrap_err();
    assert_eq!(
      err2,
      CliError::UnrecognizedArguments(vec!["bar.td".to_string()])
    );
  }

  #[test]
  fn test_help_precedence_over_unrecognized() {
    let parsed = parse_args(["-h", "--unknown"]).unwrap();
    assert!(parsed.help);
  }

  #[test]
  fn test_version_precedence_over_unrecognized() {
    let parsed = parse_args(["-v", "--unknown"]).unwrap();
    assert!(parsed.version);
  }
}
