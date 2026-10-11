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
      "tabdat-cli-json-preview-{}-{}-{}",
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

fn write_csv(dir: &TestDir) -> PathBuf {
  let path = dir.path().join("preview.csv");
  fs::write(&path, "x,label\n1,alpha\n2,beta\n3,gamma\n").expect("write CSV fixture");
  path
}

fn write_script(dir: &TestDir, name: &str, contents: &str) -> PathBuf {
  let path = dir.path().join(name);
  fs::write(&path, contents).expect("write script fixture");
  path
}

#[test]
fn json_batch_emits_head_and_tail_preview_envelopes() {
  let temp_dir = TestDir::new();
  let csv_path = write_csv(&temp_dir);
  let use_command = format!("use {}", csv_path.display());
  let output = tabdat_bin()
    .args(["--json", "-c", &use_command, "-c", "head 1", "-c", "tail 1"])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["schema_version"], 1);
  assert_eq!(envelopes[1]["result_type"], "PreviewResult");
  assert_eq!(
    envelopes[1]["data"],
    serde_json::json!({"columns": ["x", "label"], "rows": [[1, "alpha"]]})
  );
  assert_eq!(envelopes[2]["result_type"], "PreviewResult");
  assert_eq!(
    envelopes[2]["data"],
    serde_json::json!({"columns": ["x", "label"], "rows": [[3, "gamma"]]})
  );
}

#[test]
fn json_nested_script_streams_preview_after_child_results() {
  let temp_dir = TestDir::new();
  let csv_path = write_csv(&temp_dir);
  write_script(&temp_dir, "child.td", "count\n");
  let parent_path = write_script(
    &temp_dir,
    "parent.td",
    &format!("use {}\nrun child.td\nhead 1\ntail 1\n", csv_path.display()),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &parent_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  assert!(!stdout.contains("Script:"));
  assert!(!stdout.contains(". use"));
  assert!(!stdout.contains(". count"));
  assert!(!stdout.contains(". head"));
  assert!(!stdout.contains(". tail"));
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(
    envelopes
      .iter()
      .map(|envelope| envelope["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "CountResult",
      "PreviewResult",
      "PreviewResult"
    ]
  );
  assert_eq!(
    envelopes[2]["data"],
    serde_json::json!({"columns": ["x", "label"], "rows": [[1, "alpha"]]})
  );
  assert_eq!(
    envelopes[3]["data"],
    serde_json::json!({"columns": ["x", "label"], "rows": [[3, "gamma"]]})
  );
}
