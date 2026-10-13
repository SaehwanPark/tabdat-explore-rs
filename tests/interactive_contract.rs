#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-interactive-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), "x,y\n1,2\n3,\n").unwrap();
    Self(path)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(fixture: &Fixture, input: &[u8]) -> Output {
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(input).unwrap();
  child.wait_with_output().unwrap()
}

#[test]
fn persistent_eda_session_reports_schema_statistics_and_previews() {
  let fixture = Fixture::new();
  let output = run(&fixture, b"# analyst session\n\nuse data.csv\ndescribe\nsummarize y\nhead 2\ncount\nexit\nnot_a_command\n");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.contains("Loaded 2 rows, 2 columns from data.csv\n"));
  assert!(stdout.contains("Variable\tType\nx\tBIGINT\ny\tBIGINT\n"));
  assert!(stdout.contains("Variable\tN\tMean\tStd. dev.\tMin\tMax\ny\t1\t2\tNA\t2\t2\n"));
  assert!(stdout.contains("x\ty\n1\t2\n3\tNA\n(2 rows shown)\n"));
  assert!(stdout.ends_with("2 rows\n"));
  assert!(!stdout.contains("tabdat>"));
  assert!(!stdout.contains("Hello, world!"));
}

#[test]
fn errors_recover_and_failed_load_preserves_the_previous_dataset() {
  let fixture = Fixture::new();
  let output = run(
    &fixture,
    b"count\nnot_a_command\nuse data.csv\nuse missing.csv\ncount\nquit\n",
  );
  assert!(output.status.success());
  let stderr = String::from_utf8(output.stderr).unwrap();
  assert_eq!(stderr.matches("Error:").count(), 3);
  assert!(stderr.contains("missing.csv"));
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.ends_with("2 rows\n"));
}

#[test]
fn eof_executes_the_final_unterminated_line_and_io_errors_are_fatal() {
  let fixture = Fixture::new();
  let output = run(&fixture, b"use data.csv\ncount");
  assert!(output.status.success());
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .ends_with("2 rows\n")
  );
  let output = run(&fixture, &[0xff, b'\n']);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8(output.stderr)
      .unwrap()
      .contains("interactive I/O failed")
  );
}
