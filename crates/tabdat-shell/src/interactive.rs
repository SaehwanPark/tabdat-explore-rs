use std::cell::Cell;
use std::io::{self, BufRead, IsTerminal, Write};

use tabdat_execution::Session;
use tabdat_language::{Command, parse_command};
use tabdat_reporting::terminal::{render_execution_result, render_interactive_help};

const WELCOME: &str = "TabDat — type help or ? for core workflows; exit to quit.";

enum InputEvent {
  Line(String),
  Interrupted,
  Eof,
}

fn run_session_with_input(
  mut read_input: impl FnMut() -> io::Result<InputEvent>,
  output: &mut impl Write,
  errors: &mut impl Write,
  prompt: bool,
) -> io::Result<()> {
  let mut session = Session::new();
  if prompt {
    writeln!(output, "{WELCOME}")?;
  }
  loop {
    if prompt {
      write!(output, "tabdat> ")?;
      output.flush()?;
    }
    let line = match read_input()? {
      InputEvent::Line(line) => line,
      InputEvent::Interrupted => {
        writeln!(output, "Input cancelled.")?;
        output.flush()?;
        continue;
      }
      InputEvent::Eof => {
        if prompt {
          writeln!(output)?;
        }
        return Ok(());
      }
    };
    let text = line.trim();
    if text.is_empty() || text.starts_with('#') {
      continue;
    }
    let command = match parse_command(text) {
      Ok(command) => command,
      Err(error) => {
        writeln!(errors, "Error: {}", error.to_string().escape_debug())?;
        errors.flush()?;
        continue;
      }
    };
    if let Command::Help { topic } = &command {
      match render_interactive_help(topic.as_deref()) {
        Ok(text) => {
          write!(output, "{text}")?;
          output.flush()?;
        }
        Err(error) => {
          writeln!(errors, "Error: {}", error.escape_debug())?;
          errors.flush()?;
        }
      }
      continue;
    }
    if command == Command::Exit {
      return Ok(());
    }
    let result = if let Command::Run { path } = command {
      // The observer API is infallible. Retain an IO failure and reject the next
      // command before execution, so a broken output stream cannot hide later
      // data/file mutations. Script-local directives have no lasting effects.
      let output_failed = Cell::new(false);
      let mut io_failure = None;
      let result = session.execute_run_with_callbacks(
        path,
        |_| {
          if output_failed.get() {
            Err("interactive output failed".to_owned())
          } else {
            Ok(())
          }
        },
        |result| {
          if let Err(error) =
            write!(output, "{}", render_execution_result(result)).and_then(|_| output.flush())
          {
            output_failed.set(true);
            io_failure = Some(error);
          }
        },
      );
      if let Some(error) = io_failure {
        return Err(error);
      }
      result.map(tabdat_domain::ExecutionResult::Run)
    } else {
      session.execute(command)
    };
    match result {
      Ok(result) => {
        write!(output, "{}", render_execution_result(&result))?;
        output.flush()?;
      }
      Err(error) => {
        writeln!(errors, "Error: {}", error.to_string().escape_debug())?;
        errors.flush()?;
      }
    }
  }
}

fn run_session(
  input: &mut impl BufRead,
  output: &mut impl Write,
  errors: &mut impl Write,
  prompt: bool,
) -> io::Result<()> {
  run_session_with_input(
    || {
      let mut line = String::new();
      if input.read_line(&mut line)? == 0 {
        Ok(InputEvent::Eof)
      } else {
        Ok(InputEvent::Line(line))
      }
    },
    output,
    errors,
    prompt,
  )
}

fn run_editor_session(output: &mut impl Write, errors: &mut impl Write) -> io::Result<()> {
  use rustyline::{Config, DefaultEditor, error::ReadlineError};
  let config = Config::builder()
    .behavior(rustyline::config::Behavior::Stdio)
    .max_history_size(500)
    .map_err(io::Error::other)?
    .build();
  let mut editor = DefaultEditor::with_config(config).map_err(io::Error::other)?;
  writeln!(output, "{WELCOME}")?;
  output.flush()?;
  run_session_with_input(
    || match editor.readline("tabdat> ") {
      Ok(line) => {
        let text = line.trim();
        if !text.is_empty() && !text.starts_with('#') {
          editor
            .add_history_entry(line.as_str())
            .map_err(io::Error::other)?;
        }
        Ok(InputEvent::Line(line))
      }
      Err(ReadlineError::Interrupted) => Ok(InputEvent::Interrupted),
      #[cfg(unix)]
      Err(ReadlineError::Signal(rustyline::error::Signal::Interrupt)) => {
        Ok(InputEvent::Interrupted)
      }
      Err(ReadlineError::Eof) => Ok(InputEvent::Eof),
      Err(error) => Err(io::Error::other(error)),
    },
    output,
    errors,
    false,
  )
}

pub(crate) fn run_interactive() -> i32 {
  let input = io::stdin();
  let output = io::stdout();
  let errors = io::stderr();
  let prompt = input.is_terminal() && output.is_terminal();
  let result = if prompt {
    run_editor_session(&mut output.lock(), &mut errors.lock())
  } else {
    run_session(
      &mut input.lock(),
      &mut output.lock(),
      &mut errors.lock(),
      false,
    )
  };
  match result {
    Ok(()) => 0,
    Err(error) => {
      eprintln!("Error: interactive I/O failed: {error}");
      1
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn prompts_recover_from_command_errors_and_exit_without_reading_more() {
    let mut output = Vec::new();
    let mut errors = Vec::new();
    run_session(
      &mut Cursor::new(b"\n# comment\ncount\nnonsense_command\nquit\ncount\n"),
      &mut output,
      &mut errors,
      true,
    )
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.starts_with("TabDat"));
    assert_eq!(output.matches("tabdat> ").count(), 5);
    let errors = String::from_utf8(errors).unwrap();
    assert_eq!(errors.matches("Error:").count(), 2);
  }

  #[test]
  fn eof_is_clean_and_nonterminal_input_has_no_prompt() {
    let mut output = Vec::new();
    let mut errors = Vec::new();
    run_session(
      &mut Cursor::new(b"# comment\n"),
      &mut output,
      &mut errors,
      false,
    )
    .unwrap();
    assert!(output.is_empty());
    assert!(errors.is_empty());
  }

  #[test]
  fn input_interruption_preserves_data_and_model_then_accepts_more_commands() {
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "tabdat-input-interrupt-{}-{nonce}",
      std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let csv = root.join("data.csv");
    std::fs::write(&csv, "x,y\n0,1\n1,2\n2,4\n3,6\n").unwrap();
    let mut events = vec![
      InputEvent::Line(format!("use {}", csv.display())),
      InputEvent::Line("regress y x".to_owned()),
      InputEvent::Interrupted,
      InputEvent::Line("predict fitted".to_owned()),
      InputEvent::Line("count".to_owned()),
      InputEvent::Eof,
    ]
    .into_iter();
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let result = run_session_with_input(
      || Ok(events.next().unwrap()),
      &mut output,
      &mut errors,
      false,
    );
    std::fs::remove_dir_all(&root).unwrap();
    result.unwrap();
    assert!(errors.is_empty());
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Input cancelled.\nPredicted: 4 rows, 3 columns\n"));
    assert!(output.ends_with("4 rows\n"));
  }

  #[test]
  fn script_output_failure_prevents_later_script_and_shell_file_writes() {
    struct FailedOutput {
      fail_on_flush: bool,
    }
    impl Write for FailedOutput {
      fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_on_flush {
          Ok(bytes.len())
        } else {
          Err(io::Error::other("write failed"))
        }
      }
      fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("flush failed"))
      }
    }
    for fail_on_flush in [false, true] {
      let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
      let root = std::env::temp_dir().join(format!(
        "tabdat-script-output-{}-{nonce}",
        std::process::id()
      ));
      std::fs::create_dir(&root).unwrap();
      let csv = root.join("data.csv");
      let script = root.join("parent.td");
      let script_output = root.join("script.csv");
      let shell_output = root.join("shell.csv");
      std::fs::write(&csv, "x\n1\n").unwrap();
      std::fs::write(
        &script,
        format!(
          "use {}\nexport {}\n",
          csv.display(),
          script_output.display()
        ),
      )
      .unwrap();
      let input = format!(
        "run {}\nexport {}\n",
        script.display(),
        shell_output.display()
      );
      let result = run_session(
        &mut Cursor::new(input.as_bytes()),
        &mut FailedOutput { fail_on_flush },
        &mut Vec::new(),
        false,
      );
      let written = script_output.exists() || shell_output.exists();
      std::fs::remove_dir_all(&root).unwrap();
      assert!(result.is_err());
      assert!(
        !written,
        "commands after a broken output stream must not write files"
      );
    }
  }

  #[test]
  fn io_failures_propagate_instead_of_spinning_or_claiming_success() {
    struct FailedInput;
    impl io::Read for FailedInput {
      fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("input failed"))
      }
    }
    impl BufRead for FailedInput {
      fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Err(io::Error::other("input failed"))
      }
      fn consume(&mut self, _: usize) {}
    }
    assert!(run_session(&mut FailedInput, &mut Vec::new(), &mut Vec::new(), false).is_err());
    let mut output = io::Cursor::new([0u8; 0]);
    assert!(
      run_session(
        &mut Cursor::new(b"exit\n"),
        &mut output,
        &mut Vec::new(),
        true
      )
      .is_err()
    );
  }
}
