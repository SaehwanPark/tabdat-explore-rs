#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

struct TestDir {
  path: PathBuf,
}

impl TestDir {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("clock after epoch")
      .as_nanos();
    let test_id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
      "tabdat-cli-test-{}-{}-{}",
      std::process::id(),
      nonce,
      test_id
    ));
    fs::create_dir_all(&path).expect("create test dir");
    Self { path }
  }

  fn path(&self) -> &Path {
    &self.path
  }
}

impl Drop for TestDir {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.path);
  }
}

fn tabdat_bin() -> Command {
  Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
}

#[test]
fn cli_version_flag() {
  let output = tabdat_bin()
    .arg("-v")
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  assert_eq!(output.stdout, b"tabdat 0.1.0\n");
  assert!(output.stderr.is_empty());

  let output2 = tabdat_bin()
    .arg("--version")
    .output()
    .expect("binary should execute");
  assert!(output2.status.success());
  assert_eq!(output2.stdout, b"tabdat 0.1.0\n");
  assert!(output2.stderr.is_empty());
}

#[test]
fn cli_help_flag() {
  let output = tabdat_bin()
    .arg("-h")
    .output()
    .expect("binary should execute");
  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  assert!(stdout.contains("usage: tabdat"));
  assert!(stdout.contains("options:"));
  assert!(output.stderr.is_empty());

  let output2 = tabdat_bin()
    .arg("--help")
    .output()
    .expect("binary should execute");
  assert!(output2.status.success());
  let stdout2 = String::from_utf8(output2.stdout).expect("utf-8");
  assert!(stdout2.contains("usage: tabdat"));
  assert!(output2.stderr.is_empty());
}

#[test]
fn cli_conflict_rejection() {
  // -c and -f
  let output = tabdat_bin()
    .args(["-c", "count", "-f", "script.td"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: -c/--command cannot be combined with script execution"));

  // -c and positional script
  let output = tabdat_bin()
    .args(["-c", "count", "script.td"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: -c/--command cannot be combined with script execution"));

  // -f and positional script
  let output = tabdat_bin()
    .args(["-f", "script1.td", "script2.td"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: -f/--file cannot be combined with a positional script"));
}

#[test]
fn cli_missing_argument_rejection() {
  let output = tabdat_bin()
    .arg("-c")
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: argument -c/--command: expected one argument"));

  let output = tabdat_bin()
    .arg("-f")
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: argument -f/--file: expected one argument"));
}

#[test]
fn cli_unrecognized_argument_rejection() {
  let output = tabdat_bin()
    .arg("--bogus-flag")
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("usage: tabdat"));
  assert!(stderr.contains("tabdat: error: unrecognized arguments: --bogus-flag"));
}

#[test]
fn cli_command_execution_success_and_errors() {
  // Parse error exits 2
  let output = tabdat_bin()
    .args(["-c", "unknown_cmd 123"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(2));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.starts_with("Error: "));

  // Runtime error (no active dataset) exits 1
  let output = tabdat_bin()
    .args(["-c", "count"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert_eq!(
    stderr,
    "Error: count requires an active dataset; run use <path> first\n"
  );

  // Exit command exits 0
  let output = tabdat_bin()
    .args(["-c", "exit"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(0));

  // Batch execution with active dataset succeeds
  let temp_dir = TestDir::new();
  let csv_path = temp_dir.path().join("data.csv");
  fs::write(&csv_path, "x,y\n1,2\n3,4\n").expect("write csv");
  let use_cmd = format!("use {}", csv_path.display());

  let output = tabdat_bin()
    .args(["-c", &use_cmd, "-c", "count"])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(0));

  // Early failure stops execution
  let output = tabdat_bin()
    .args(["-c", "count", "-c", &use_cmd])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));
}

#[test]
fn cli_script_execution() {
  let temp_dir = TestDir::new();

  // Nonexistent script file exits 3
  let missing_path = temp_dir.path().join("missing.td");
  let output = tabdat_bin()
    .args(["-f", missing_path.to_str().unwrap()])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(3));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("script file not found"));

  // Positional nonexistent script exits 3
  let output = tabdat_bin()
    .arg(missing_path.to_str().unwrap())
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(3));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("script file not found"));

  // Successful script execution
  let csv_path = temp_dir.path().join("script_data.csv");
  fs::write(&csv_path, "a,b\n10,20\n30,40\n").expect("write csv");

  let valid_script = temp_dir.path().join("test_run.td");
  fs::write(
    &valid_script,
    format!("# TabDat script\nuse {}\ncount\n", csv_path.display()),
  )
  .expect("write script");

  let output = tabdat_bin()
    .args(["-f", valid_script.to_str().unwrap()])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(0));

  // Positional script execution
  let output = tabdat_bin()
    .arg(valid_script.to_str().unwrap())
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(0));

  // Script with runtime error exits 1
  let runtime_err_script = temp_dir.path().join("runtime_err.td");
  fs::write(&runtime_err_script, "count\n").expect("write script");
  let output = tabdat_bin()
    .args(["-f", runtime_err_script.to_str().unwrap()])
    .output()
    .expect("binary should execute");
  assert_eq!(output.status.code(), Some(1));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("count requires an active dataset"));
}

#[test]
fn cli_doctor_shortcut() {
  let output = tabdat_bin()
    .arg("doctor")
    .output()
    .expect("binary should execute");
  // Doctor execution currently returns unsupported command in runtime
  assert_eq!(output.status.code(), Some(1));
  let stderr = String::from_utf8(output.stderr).expect("utf-8");
  assert!(stderr.contains("runtime does not execute command: doctor"));
}
