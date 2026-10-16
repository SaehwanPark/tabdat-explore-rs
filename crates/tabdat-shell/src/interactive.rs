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
  mut read_input: impl FnMut(&Session) -> io::Result<InputEvent>,
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
    let line = match read_input(&session)? {
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
    |_| {
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
  use rustyline::{Config, Editor, error::ReadlineError, history::DefaultHistory};
  let config = Config::builder()
    .behavior(rustyline::config::Behavior::Stdio)
    .completion_type(rustyline::config::CompletionType::List)
    .max_history_size(500)
    .map_err(io::Error::other)?
    .build();
  let mut editor = Editor::<crate::completion::ShellHelper, DefaultHistory>::with_config(config)
    .map_err(io::Error::other)?;
  editor.set_helper(Some(crate::completion::ShellHelper::default()));
  writeln!(output, "{WELCOME}")?;
  output.flush()?;
  run_session_with_input(
    |session| {
      // Refresh from current metadata even after errors or partial script effects;
      // cached-model names and backend scans are not completion sources.
      if let Some(helper) = editor.helper_mut() {
        helper.set_named_tables(session.named_tables().keys().map(String::as_str));
        helper.set_columns(
          session
            .active_dataset()
            .map_or(&[], |dataset| dataset.columns.as_slice()),
        );
      }
      match editor.readline("tabdat> ") {
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
      }
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
      |_| Ok(events.next().unwrap()),
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
  fn input_observes_current_schema_after_failures_partial_scripts_and_named_table_switches() {
    use rustyline::{Context, completion::Completer, history::DefaultHistory};
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "tabdat-column-snapshot-{}-{nonce}",
      std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let first = root.join("first.csv");
    let second = root.join("second.csv");
    let child = root.join("child.td");
    std::fs::write(&first, "x,y\n1,2\n2,5\n3,5\n4,9\n").unwrap();
    std::fs::write(&second, "z\n1\n2\n").unwrap();
    std::fs::write(&child, "rename outcome scripted\nunknown_command\n").unwrap();
    let mut events: std::vec::IntoIter<(Vec<&str>, InputEvent)> = vec![
      (vec![], InputEvent::Line(format!("use {}", first.display()))),
      (vec!["x", "y"], InputEvent::Line("generate new = x".into())),
      (
        vec!["new", "x", "y"],
        InputEvent::Line("rename absent broken".into()),
      ),
      (
        vec!["new", "x", "y"],
        InputEvent::Line("rename y outcome".into()),
      ),
      (
        vec!["new", "outcome", "x"],
        InputEvent::Line("drop new".into()),
      ),
      (
        vec!["outcome", "x"],
        InputEvent::Line(format!("run {}", child.display())),
      ),
      (
        vec!["scripted", "x"],
        InputEvent::Line(format!("use {}", root.join("missing.csv").display())),
      ),
      (
        vec!["scripted", "x"],
        InputEvent::Line("sql select x from active into one".into()),
      ),
      (
        vec!["x"],
        InputEvent::Line(format!("use {}", second.display())),
      ),
      (vec!["z"], InputEvent::Line("use one".into())),
      (vec!["x"], InputEvent::Interrupted),
      (vec!["x"], InputEvent::Eof),
    ]
    .into_iter();
    let mut helper = crate::completion::ShellHelper::default();
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let result = run_session_with_input(
      |session| {
        let (expected, event) = events.next().unwrap();
        helper.set_columns(
          session
            .active_dataset()
            .map_or(&[], |dataset| dataset.columns.as_slice()),
        );
        let actual = helper
          .complete("summarize ", 10, &Context::new(&DefaultHistory::new()))
          .unwrap()
          .1;
        assert_eq!(actual, expected);
        Ok(event)
      },
      &mut output,
      &mut errors,
      false,
    );
    std::fs::remove_dir_all(&root).unwrap();
    result.unwrap();
    assert!(events.next().is_none());
    let errors = String::from_utf8(errors).unwrap();
    assert_eq!(errors.matches("Error:").count(), 3, "{errors}");
    assert!(errors.contains("child.td:2:"));
  }

  #[test]
  fn input_refreshes_registered_names_after_partial_scripts_failed_loads_and_activation() {
    use rustyline::{Context, completion::Completer, history::DefaultHistory};
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "tabdat-named-snapshot-{}-{nonce}",
      std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let csv = root.join("data.csv");
    let child = root.join("child.td");
    std::fs::write(&csv, "x,y\n0,1\n1,2\n2,4\n3,6\n").unwrap();
    std::fs::write(
      &child,
      "sql SELECT * FROM active WHERE x >= 2 into cohort_partial\nrename absent broken\n",
    )
    .unwrap();
    let mut events = vec![
      (
        vec![],
        None,
        InputEvent::Line(format!("use {}", csv.display())),
      ),
      (vec![], Some(4), InputEvent::Line("regress y x".into())),
      (
        vec![],
        Some(4),
        InputEvent::Line(format!("run {}", child.display())),
      ),
      (
        vec!["cohort_partial"],
        Some(2),
        InputEvent::Line(format!("use {}", root.join("missing.csv").display())),
      ),
      (
        vec!["cohort_partial"],
        Some(2),
        InputEvent::Line("sql SELECT * FROM active into cohort_copy".into()),
      ),
      (
        vec!["cohort_copy", "cohort_partial"],
        Some(2),
        InputEvent::Line(format!("use {}", csv.display())),
      ),
      (
        vec!["cohort_copy", "cohort_partial"],
        Some(4),
        InputEvent::Line("use cohort_partial".into()),
      ),
      (
        vec!["cohort_copy", "cohort_partial"],
        Some(2),
        InputEvent::Line("use cohort_partial, has_header(false)".into()),
      ),
      (
        vec!["cohort_copy", "cohort_partial"],
        Some(2),
        InputEvent::Interrupted,
      ),
      (
        vec!["cohort_copy", "cohort_partial"],
        Some(2),
        InputEvent::Eof,
      ),
    ]
    .into_iter();
    let mut helper = crate::completion::ShellHelper::default();
    let mut index = 0;
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let result = run_session_with_input(
      |session| {
        let (names, rows, event) = events.next().unwrap();
        let before = session.state();
        let capabilities = session.initialized_capabilities().to_vec();
        helper.set_named_tables(session.named_tables().keys().map(String::as_str));
        assert_eq!(
          helper
            .complete("use cohort_", 11, &Context::new(&DefaultHistory::new()))
            .unwrap()
            .1,
          names
        );
        assert_eq!(session.active_dataset().map(|data| data.row_count), rows);
        assert_eq!(
          session
            .last_regression()
            .map(|fit| fit.fit_statistics.observation_count),
          (index >= 2).then_some(4)
        );
        assert_eq!(session.initialized_capabilities(), capabilities);
        if index == 0 {
          assert!(capabilities.is_empty());
        }
        assert_eq!(session.state(), before);
        index += 1;
        Ok(event)
      },
      &mut output,
      &mut errors,
      false,
    );
    result.unwrap();
    assert!(events.next().is_none());
    let errors = String::from_utf8(errors).unwrap();
    assert_eq!(errors.matches("Error:").count(), 3, "{errors}");
    assert!(errors.contains("child.td:2:"));
    assert!(
      String::from_utf8(output)
        .unwrap()
        .contains("Activated table \"cohort_partial\": 2 rows, 2 columns")
    );
    let plain = format!(
      "use {}\nsql SELECT * FROM active WHERE x >= 2 into cohort_partial\nuse {}\nuse cohort_p\t\ncount\nuse cohort_partial\ncount\nexit\n",
      csv.display(),
      csv.display()
    );
    let mut output = Vec::new();
    let mut errors = Vec::new();
    run_session(
      &mut Cursor::new(plain.as_bytes()),
      &mut output,
      &mut errors,
      false,
    )
    .unwrap();
    assert_eq!(
      String::from_utf8(errors).unwrap(),
      "Error: unknown table: cohort_p\n"
    );
    assert!(
      String::from_utf8(output)
        .unwrap()
        .ends_with("4 rows\nActivated table \"cohort_partial\": 2 rows, 2 columns\n2 rows\n")
    );
    std::fs::remove_dir_all(&root).unwrap();
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
  fn coding_output_failure_stops_later_dictionary_commands_and_files() {
    struct FailedOutput {
      fail_at: usize,
      calls: usize,
      fail_on_flush: bool,
    }
    impl Write for FailedOutput {
      fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.fail_on_flush {
          self.calls += 1;
          if self.calls == self.fail_at {
            return Err(io::Error::other("write failed"));
          }
        }
        Ok(bytes.len())
      }
      fn flush(&mut self) -> io::Result<()> {
        if self.fail_on_flush {
          self.calls += 1;
          if self.calls == self.fail_at {
            return Err(io::Error::other("flush failed"));
          }
        }
        Ok(())
      }
    }
    for script in [false, true] {
      for fail_on_flush in [false, true] {
        for fail_at in 1..=if script { 9 } else { 8 } {
          let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
          let root = std::env::temp_dir().join(format!(
            "tabdat-coding-output-{}-{nonce}",
            std::process::id()
          ));
          std::fs::create_dir(&root).unwrap();
          let csv = root.join("data.csv");
          std::fs::write(&csv, "category\nb\na\n").unwrap();
          let first = root.join("coded.csv");
          let second = root.join("decoded.csv");
          let later = root.join("later.csv");
          let commands = [
            format!("use {}", csv.display()),
            "label variable category 'Category'".into(),
            "encode category, generate(code)".into(),
            "label list code".into(),
            format!("export {}", first.display()),
            "decode code, generate(readable)".into(),
            "label drop code".into(),
            format!("export {}", second.display()),
          ];
          let input = if script {
            let parent = root.join("parent.td");
            let child = root.join("child.td");
            std::fs::write(
              &parent,
              format!("{}\nrun {}\n", commands[0], child.display()),
            )
            .unwrap();
            std::fs::write(&child, commands[1..].join("\n")).unwrap();
            format!("run {}\nexport {}\n", parent.display(), later.display())
          } else {
            format!("{}\nexport {}\n", commands.join("\n"), later.display())
          };
          let mut errors = Vec::new();
          let result = run_session(
            &mut Cursor::new(input.as_bytes()),
            &mut FailedOutput {
              fail_at,
              calls: 0,
              fail_on_flush,
            },
            &mut errors,
            false,
          );
          assert!(result.is_err());
          assert!(errors.is_empty());
          assert_eq!(first.exists(), fail_at >= 5);
          assert_eq!(second.exists(), fail_at >= 8);
          assert!(!later.exists());
          if first.exists() {
            assert_eq!(
              std::fs::read_to_string(&first).unwrap(),
              "category,code\nb,2\na,1\n"
            );
          }
          if second.exists() {
            assert_eq!(
              std::fs::read_to_string(&second).unwrap(),
              "category,code,readable\nb,2,b\na,1,a\n"
            );
          }
          std::fs::remove_dir_all(&root).unwrap();
        }
      }
    }
  }

  #[test]
  fn cohort_output_failure_stops_later_transforms_and_files() {
    struct FailedOutput {
      fail_at: usize,
      calls: usize,
      fail_on_flush: bool,
    }
    impl Write for FailedOutput {
      fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if !self.fail_on_flush {
          self.calls += 1;
          if self.calls == self.fail_at {
            return Err(io::Error::other("write failed"));
          }
        }
        Ok(bytes.len())
      }
      fn flush(&mut self) -> io::Result<()> {
        if self.fail_on_flush {
          self.calls += 1;
          if self.calls == self.fail_at {
            return Err(io::Error::other("flush failed"));
          }
        }
        Ok(())
      }
    }
    for script in [false, true] {
      for fail_on_flush in [false, true] {
        for fail_at in 1..=if script { 15 } else { 14 } {
          let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
          let root = std::env::temp_dir().join(format!(
            "tabdat-cohort-output-{}-{nonce}",
            std::process::id()
          ));
          std::fs::create_dir(&root).unwrap();
          let csv = root.join("data.csv");
          std::fs::write(&csv, "id,x_a,x_b,y_a,y_b\n1,1,2,2,5\n2,3,4,5,9\n").unwrap();
          let first = root.join("long.csv");
          let second = root.join("wide.csv");
          let later = root.join("later.csv");
          let commands = [
            format!("use {}", csv.display()),
            "sql select * from active order by id into baseline".into(),
            "sql select id, id*10 as z from active order by id into lookup".into(),
            "use baseline".into(),
            "join lookup on id".into(),
            "sql select * from active order by id into followup".into(),
            "use baseline".into(),
            "append followup".into(),
            "reshape long x y, i(id) j(period)".into(),
            "recode x (1=10), replace".into(),
            "by period: count".into(),
            format!("export {}", first.display()),
            "reshape wide x y, i(id) j(period)".into(),
            format!("export {}", second.display()),
          ];
          let input = if script {
            let parent = root.join("parent.td");
            let child = root.join("child.td");
            std::fs::write(
              &parent,
              format!("{}\nrun {}\n", commands[0], child.display()),
            )
            .unwrap();
            std::fs::write(&child, commands[1..].join("\n")).unwrap();
            format!("run {}\nexport {}\n", parent.display(), later.display())
          } else {
            format!("{}\nexport {}\n", commands.join("\n"), later.display())
          };
          let mut errors = Vec::new();
          let result = run_session(
            &mut Cursor::new(input.as_bytes()),
            &mut FailedOutput {
              fail_at,
              calls: 0,
              fail_on_flush,
            },
            &mut errors,
            false,
          );
          assert!(result.is_err());
          assert!(errors.is_empty());
          assert_eq!(first.exists(), fail_at >= 12);
          assert_eq!(second.exists(), fail_at >= 14);
          assert!(!later.exists());
          if first.exists() {
            assert_eq!(
              std::fs::read_to_string(&first).unwrap(),
              "id,period,x,y\n1,a,10,2\n1,b,2,5\n2,a,3,5\n2,b,4,9\n1,a,10,2\n1,b,2,5\n2,a,3,5\n2,b,4,9\n"
            );
          }
          if second.exists() {
            assert_eq!(
              std::fs::read_to_string(&second).unwrap(),
              "id,x_a,x_b,y_a,y_b\n1,10,2,2,5\n2,3,4,5,9\n"
            );
          }
          std::fs::remove_dir_all(&root).unwrap();
        }
      }
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
