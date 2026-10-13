#![forbid(unsafe_code)]

use serde_json::Value;
use std::{
  fs,
  path::PathBuf,
  process::Command,
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-script-exits-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "x,y\n0,1\n1,2\n2,1.5\n3,4\n4,3\n5,5\n",
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, file: &str, json: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"));
    command.current_dir(&self.0);
    if json {
      command.arg("--json");
    }
    command.args(["-f", file]).output().unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn nested_script_exit_categories_are_independent_of_diagnostic_words() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  for (source, code, line, message) in [
    (
      "regress y x\ntest absent\n",
      1,
      2,
      "not found in active model",
    ),
    ("unknown_command\n", 2, 1, "unknown command"),
    ("seed invalid\n", 2, 1, "seed expects an integer"),
    ("if true\ncount\n", 2, 1, "if block is missing end"),
    ("count $undefined\n", 2, 1, "undefined macro"),
    ("let 9bad = 1\n", 2, 1, "macro name must be an identifier"),
    ("sql \"\"\"select 1\n", 2, 1, "missing closing"),
    ("use missing.csv\n", 3, 1, "could not find file"),
    ("run missing-child.td\n", 3, 1, "script file not found"),
  ] {
    fs::write(fixture.0.join("child.td"), source).unwrap();
    for json in [false, true] {
      let output = fixture.run("parent.td", json);
      assert_eq!(
        output.status.code(),
        Some(code),
        "{source:?}: {}",
        String::from_utf8_lossy(&output.stderr)
      );
      assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "{source:?}: {}",
        String::from_utf8_lossy(&output.stderr)
      );
      if json {
        let values = envelopes(&output.stdout);
        let error = &values.last().unwrap()["error"];
        assert_eq!(error["type"], "ScriptError");
        assert_eq!(error["line"], line);
        let expected = if source.starts_with("run missing-") {
          fixture.0.canonicalize().unwrap().join("missing-child.td")
        } else {
          fixture.0.join("child.td").canonicalize().unwrap()
        };
        assert_eq!(error["path"], expected.to_string_lossy().as_ref());
        assert_eq!(values[0]["result_type"], "LoadResult");
        assert!(
          values[..values.len() - 1]
            .iter()
            .all(|value| value.get("error").is_none())
        );
      } else {
        assert!(output.stdout.is_empty());
      }
    }
  }
}

#[test]
fn script_load_failures_are_file_errors_even_without_not_found_wording() {
  let fixture = Fixture::new();
  fs::create_dir(fixture.0.join("directory.td")).unwrap();
  fs::write(fixture.0.join("invalid-utf8.td"), b"# valid line\n\xff").unwrap();
  for (file, message, line) in [
    ("missing.td", "script file not found", 1),
    ("directory.td", "script path is a directory", 1),
    ("invalid-utf8.td", "script file must be UTF-8 text", 2),
  ] {
    for json in [false, true] {
      let output = fixture.run(file, json);
      assert_eq!(output.status.code(), Some(3));
      assert!(String::from_utf8_lossy(&output.stderr).contains(message));
      if json {
        let values = envelopes(&output.stdout);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["error"]["type"], "ScriptError");
        assert_eq!(values[0]["error"]["line"], line);
      }
    }
  }
}

#[test]
fn execution_errors_with_expected_text_stop_before_later_file_effects() {
  let fixture = Fixture::new();
  fs::create_dir(fixture.0.join("directory.csv")).unwrap();
  // A valid use command fails at runtime with "use expected a file path".
  // That wording must not turn the command failure into a parser error.
  let path = fixture.0.join("runtime.td");
  fs::write(&path, "use data.csv\nuse directory.csv\nexport later.csv\n").unwrap();
  for json in [false, true] {
    let output = fixture.run("runtime.td", json);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected"));
    assert!(!fixture.0.join("later.csv").exists());
    if json {
      let values = envelopes(&output.stdout);
      assert_eq!(values.len(), 2);
      assert_eq!(values[0]["result_type"], "LoadResult");
      assert_eq!(values[1]["error"]["line"], 2);
    }
  }
  // The library does not change the process working directory for scripts.
  fs::write(
    &path,
    format!(
      "use {}\nuse {}\nexport {}\n",
      fixture.0.join("data.csv").display(),
      fixture.0.join("directory.csv").display(),
      fixture.0.join("later.csv").display()
    ),
  )
  .unwrap();
  let mut session = tabdat_execution::Session::new();
  let error = session.execute_run(&path).unwrap_err();
  let tabdat_domain::RuntimeError::ScriptError(error) = error else {
    panic!("expected located error");
  };
  assert_eq!(error.kind(), tabdat_domain::ScriptErrorKind::Execution);
  assert_eq!(error.line(), 2, "{error}");
  assert_eq!(session.active_dataset().unwrap().row_count, 6);
  assert_eq!(session.active_dataset().unwrap().columns.len(), 2);
  assert!(!fixture.0.join("later.csv").exists());
}

#[test]
fn preflight_failure_wording_cannot_reclassify_or_advance_execution() {
  use tabdat_domain::{RuntimeError, ScriptErrorKind};
  let fixture = Fixture::new();
  let path = fixture.0.join("preflight.td");
  fs::write(&path, "count\nuse data.csv\n").unwrap();
  let mut session = tabdat_execution::Session::new();
  let mut observed = 0;
  let error = session
    .execute_run_with_callbacks(
      &path,
      |_| Err("parse syntax expected not found".to_owned()),
      |_| observed += 1,
    )
    .unwrap_err();
  let RuntimeError::ScriptError(error) = error else {
    panic!("expected located error");
  };
  assert_eq!(error.kind(), ScriptErrorKind::Execution);
  assert_eq!(error.line(), 1);
  assert_eq!(error.message(), "parse syntax expected not found");
  assert_eq!(observed, 0);
  assert!(session.active_dataset().is_none());
}
