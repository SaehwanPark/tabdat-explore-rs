use std::cell::Cell;
use std::io::{self, BufRead, IsTerminal, Write};

use tabdat_execution::Session;
use tabdat_language::{Command, parse_command};
use tabdat_reporting::terminal::render_execution_result;

fn run_session(
  input: &mut impl BufRead,
  output: &mut impl Write,
  errors: &mut impl Write,
  prompt: bool,
) -> io::Result<()> {
  let mut session = Session::new();
  if prompt {
    writeln!(
      output,
      "TabDat — use <file>, describe, summarize, head, count; exit to quit."
    )?;
  }
  let mut line = String::new();
  loop {
    if prompt {
      write!(output, "tabdat> ")?;
      output.flush()?;
    }
    line.clear();
    if input.read_line(&mut line)? == 0 {
      if prompt {
        writeln!(output)?;
      }
      return Ok(());
    }
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

pub(crate) fn run_interactive() -> i32 {
  let input = io::stdin();
  let output = io::stdout();
  let errors = io::stderr();
  let prompt = input.is_terminal() && output.is_terminal();
  match run_session(
    &mut input.lock(),
    &mut output.lock(),
    &mut errors.lock(),
    prompt,
  ) {
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
