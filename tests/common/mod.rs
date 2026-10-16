//! Shared CLI test plumbing. Keep scenario data and behavioral assertions in each test file.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

pub struct TestDir {
  path: PathBuf,
}

impl TestDir {
  pub fn new() -> Self {
    loop {
      let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
      let path = std::env::temp_dir().join(format!("tabdat-cli-test-{}-{id}", std::process::id()));
      // Claim a fresh directory atomically. Never reuse or remove an existing
      // directory, including one left behind by an earlier process with this PID.
      match fs::create_dir(&path) {
        Ok(()) => return Self { path },
        Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
        Err(error) => panic!("create test directory {}: {error}", path.display()),
      }
    }
  }

  pub fn path(&self) -> &Path {
    &self.path
  }
}

impl Drop for TestDir {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.path);
  }
}

pub fn tabdat_bin() -> Command {
  Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
}
