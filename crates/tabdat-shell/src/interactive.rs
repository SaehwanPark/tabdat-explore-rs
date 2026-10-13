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
    match session.execute(command) {
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
