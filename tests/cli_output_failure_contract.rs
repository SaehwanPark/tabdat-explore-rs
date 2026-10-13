#![cfg(unix)]
#![forbid(unsafe_code)]

use std::{
  fs,
  os::{fd::OwnedFd, unix::net::UnixStream},
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

// Close the peer before spawning, avoiding timing races with fast discovery.
fn closed_stream() -> Stdio {
  let (reader, writer) = UnixStream::pair().unwrap();
  drop(reader);
  let writer: OwnedFd = writer.into();
  Stdio::from(writer)
}
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-closed-cli-output-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x\n1\n2\n").unwrap();
    fs::write(root.join("child.td"), "count\ncount\n").unwrap();
    fs::write(
      root.join("parent.td"),
      "use data.csv\nrun child.td\ncount\n",
    )
    .unwrap();
    Self(root)
  }
  fn command(&self, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"));
    command.current_dir(&self.0).args(args);
    command
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

#[test]
fn closed_stdout_returns_io_exit_one_without_a_panic_in_batch_scripts_and_discovery() {
  let fixture = Fixture::new();
  for args in [
    vec!["--json", "-c", "use data.csv", "-c", "count"],
    vec!["--json", "-f", "parent.td"],
    vec!["--help"],
    vec!["--version"],
    vec!["--json", "--list-commands"],
    vec!["--json", "--help-topic", "absent"],
  ] {
    let output = fixture
      .command(&args)
      .stdout(closed_stream())
      .stderr(Stdio::piped())
      .output()
      .unwrap();
    assert_eq!(output.status.code(), Some(1), "{args:?}");
    let errors = String::from_utf8(output.stderr).unwrap();
    assert!(errors.contains("Error: CLI I/O failed:"), "{errors}");
    assert!(!errors.contains("panicked"), "{errors}");
    assert!(
      !errors.contains("CLI output failed\n"),
      "do not expose the synthetic preflight error"
    );
  }
}

#[test]
fn closed_stderr_returns_one_without_attempting_json_error_output_or_panicking() {
  let fixture = Fixture::new();
  for args in [
    vec!["--json", "-c", "count"],
    vec!["-c", "absent"],
    vec!["--unknown-option"],
    vec!["--json", "--help-topic", "absent"],
  ] {
    let output = fixture
      .command(&args)
      .stdout(Stdio::piped())
      .stderr(closed_stream())
      .output()
      .unwrap();
    assert_eq!(output.status.code(), Some(1), "{args:?}");
    assert!(output.stdout.is_empty(), "{args:?}");
  }
}
