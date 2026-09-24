use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use tabdat_language::Command;
use tabdat_runtime::{ExecutionResult, RunResult, RuntimeError, Session};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
  root: PathBuf,
  parquet: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock should be after the Unix epoch")
      .as_nanos();
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "tabdat-runtime-run-{0}-{1}-{2}",
      std::process::id(),
      nonce,
      fixture_id
    ));
    fs::create_dir_all(&root).expect("fixture directory should be created");
    let parquet = root.join("patients.parquet");
    let connection = Connection::open_in_memory().expect("fixture connection should open");
    let parquet_string = parquet.to_string_lossy().into_owned();
    connection
      .execute(
        "COPY (SELECT * FROM (VALUES (30, 22.5, 'F', 100.0), (42, 25.0, 'M', 150.0), (54, 27.5, 'F', NULL)) AS patients(age, bmi, sex, cost)) TO ? (FORMAT PARQUET)",
        [&parquet_string],
      )
      .expect("fixture Parquet should be written");
    Self { root, parquet }
  }

  fn write_script(&self, name: &str, content: &str) -> PathBuf {
    let script_path = self.root.join(name);
    if let Some(parent) = script_path.parent() {
      fs::create_dir_all(parent).expect("parent directories should be created");
    }
    fs::write(&script_path, content).expect("script should be written");
    script_path
  }
}

#[test]
fn test_run_executes_sequential_commands() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "analysis.td",
    &format!(
      "# comment line\nuse {}\ngenerate age2 = age * 2\ncount\n",
      fixture.parquet.display()
    ),
  );

  let mut session = Session::new();
  let result = session
    .execute(Command::Run {
      path: script.to_string_lossy().into_owned(),
    })
    .expect("script execution should succeed");

  let canonical_script = script.canonicalize().unwrap_or(script);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_script,
      executed_commands: 3,
    })
  );

  let dataset = session.active_dataset().expect("dataset should be active");
  assert_eq!(dataset.row_count, 3);
  assert!(dataset.columns.iter().any(|col| col.name == "age2"));
}

#[test]
fn test_run_nested_scripts_with_relative_path() {
  let fixture = Fixture::new();
  let scripts_dir = fixture.root.join("scripts");
  fs::create_dir_all(&scripts_dir).expect("scripts directory should be created");

  let _child = fixture.write_script("scripts/child.td", "generate age2 = age * 2\ncount\n");
  let parent = fixture.write_script(
    "scripts/parent.td",
    &format!("use {}\nrun child.td\n", fixture.parquet.display()),
  );

  let mut session = Session::new();
  let result = session
    .execute_run(&parent)
    .expect("nested script execution should succeed");

  let canonical_parent = parent.canonicalize().unwrap_or(parent);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_parent,
      executed_commands: 3,
    })
  );

  let dataset = session.active_dataset().expect("dataset should be active");
  assert_eq!(dataset.row_count, 3);
  assert!(dataset.columns.iter().any(|col| col.name == "age2"));
}

#[test]
fn test_run_shares_macros_and_seed_with_nested_script() {
  let fixture = Fixture::new();
  fixture.write_script("child.td", "use $data\ncount\n");
  let parent = fixture.write_script(
    "parent.td",
    &format!(
      "seed 777\nlet data = {}\nrun child.td\n",
      fixture.parquet.display()
    ),
  );

  let mut session = Session::new();
  let result = session
    .execute_run(&parent)
    .expect("nested script with macros should succeed");

  let canonical_parent = parent.canonicalize().unwrap_or(parent);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_parent,
      executed_commands: 2,
    })
  );

  let dataset = session.active_dataset().expect("dataset should be active");
  assert_eq!(dataset.row_count, 3);
}

#[test]
fn test_run_rejects_recursive_inclusion() {
  let fixture = Fixture::new();
  let script = fixture.write_script("loop.td", "run loop.td\n");

  let mut session = Session::new();
  let err = session
    .execute_run(&script)
    .expect_err("recursive script should be rejected");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 1);
      assert_eq!(
        script_err.message(),
        "recursive script inclusion is not supported"
      );
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_rejects_mutual_recursion() {
  let fixture = Fixture::new();
  fixture.write_script("b.td", "run a.td\n");
  let a = fixture.write_script("a.td", "run b.td\n");

  let mut session = Session::new();
  let err = session
    .execute_run(&a)
    .expect_err("mutual recursion should be rejected");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 1);
      assert_eq!(
        script_err.message(),
        "recursive script inclusion is not supported"
      );
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_missing_script_file() {
  let fixture = Fixture::new();
  let missing = fixture.root.join("missing.td");

  let mut session = Session::new();
  let err = session
    .execute_run(&missing)
    .expect_err("missing script should fail");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 1);
      assert_eq!(script_err.message(), "script file not found");
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_directory_path() {
  let fixture = Fixture::new();

  let mut session = Session::new();
  let err = session
    .execute_run(&fixture.root)
    .expect_err("directory script path should fail");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 1);
      assert_eq!(script_err.message(), "script path is a directory");
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_control_flow_execution() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "conditionals.td",
    &format!(
      "use {}\nlet mode = active\nif $mode == active\ngenerate active_col = 1\nelse\ngenerate inactive_col = 2\nend\ncount\n",
      fixture.parquet.display()
    ),
  );

  let mut session = Session::new();
  let result = session
    .execute_run(&script)
    .expect("conditional script should succeed");

  let canonical_script = script.canonicalize().unwrap_or(script);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_script,
      executed_commands: 3,
    })
  );

  let dataset = session.active_dataset().expect("dataset should be active");
  assert!(dataset.columns.iter().any(|col| col.name == "active_col"));
  assert!(!dataset.columns.iter().any(|col| col.name == "inactive_col"));
}

#[test]
fn test_run_inactive_branch_skips_undefined_macro() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "skip_macro.td",
    &format!(
      "use {}\nif false\nuse $undefined_macro\nelse\ncount\nend\n",
      fixture.parquet.display()
    ),
  );

  let mut session = Session::new();
  let result = session
    .execute_run(&script)
    .expect("inactive branch should skip undefined macro");

  let canonical_script = script.canonicalize().unwrap_or(script);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_script,
      executed_commands: 2,
    })
  );
}

#[test]
fn test_run_exit_stops_early_successfully() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "exit.td",
    &format!(
      "use {}\nexit\ngenerate broken = missing_var\n",
      fixture.parquet.display()
    ),
  );

  let mut session = Session::new();
  let result = session
    .execute_run(&script)
    .expect("exit should terminate script cleanly");

  let canonical_script = script.canonicalize().unwrap_or(script);
  assert_eq!(
    result,
    ExecutionResult::Run(RunResult {
      path: canonical_script,
      executed_commands: 1,
    })
  );

  let dataset = session.active_dataset().expect("dataset should be active");
  assert!(!dataset.columns.iter().any(|col| col.name == "broken"));
}

#[test]
fn test_run_reports_source_file_and_line_for_command_error() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "bad.td",
    &format!("use {}\nsummarize missing_var\n", fixture.parquet.display()),
  );

  let mut session = Session::new();
  let err = session
    .execute_run(&script)
    .expect_err("command error should report source location");

  let canonical_script = script.canonicalize().unwrap_or(script.clone());
  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.path(), canonical_script);
      assert_eq!(script_err.line(), 2);
      assert_eq!(
        script_err.message(),
        "summarize unknown variable: missing_var"
      );
      assert_eq!(
        format!("{script_err}"),
        format!(
          "{}:2: summarize unknown variable: missing_var",
          canonical_script.display()
        )
      );
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_reports_source_file_and_line_for_syntax_error() {
  let fixture = Fixture::new();
  let script = fixture.write_script("bad_syntax.td", "# line 1\nreplace target\n");

  let mut session = Session::new();
  let err = session
    .execute_run(&script)
    .expect_err("syntax error should report source location");

  let canonical_script = script.canonicalize().unwrap_or(script.clone());
  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.path(), canonical_script);
      assert_eq!(script_err.line(), 2);
      assert!(script_err.message().contains("replace expects syntax"));
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_unclosed_if_block_reports_start_line() {
  let fixture = Fixture::new();
  let script = fixture.write_script(
    "unclosed.td",
    &format!("use {}\nif true\ncount\n", fixture.parquet.display()),
  );

  let mut session = Session::new();
  let err = session
    .execute_run(&script)
    .expect_err("unclosed if should error");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 2);
      assert_eq!(script_err.message(), "if block is missing end");
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}

#[test]
fn test_run_nested_if_rejected() {
  let fixture = Fixture::new();
  let script = fixture.write_script("nested_if.td", "if true\nif true\nend\nend\n");

  let mut session = Session::new();
  let err = session
    .execute_run(&script)
    .expect_err("nested if should error");

  match err {
    RuntimeError::ScriptError(script_err) => {
      assert_eq!(script_err.line(), 2);
      assert_eq!(script_err.message(), "nested if blocks are not supported");
    }
    other => panic!("expected ScriptError, got: {other:?}"),
  }
}
