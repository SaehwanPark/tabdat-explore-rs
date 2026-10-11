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
      "tabdat-cli-json-test-{}-{}-{}",
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
fn help_describes_the_current_json_result_boundary() {
  let output = tabdat_bin()
    .arg("--help")
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  assert!(stdout.contains("batch success output"));
  assert!(stdout.contains("supports use/count/head/tail"));
  assert!(stdout.contains("scripts stream"));
  assert!(stdout.contains("use/count/head/tail"));
}

fn fixture_csv(dir: &TestDir) -> PathBuf {
  let path = dir.path().join("data.csv");
  fs::write(&path, "x,y\n1,2\n3,4\n").expect("write CSV fixture");
  path
}

fn write_script(dir: &TestDir, name: &str, contents: &str) -> PathBuf {
  let path = dir.path().join(name);
  fs::write(&path, contents).expect("write script fixture");
  path
}

#[test]
fn json_batch_emits_load_and_count_result_envelopes() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let use_command = format!("use {}", csv_path.display());
  let output = tabdat_bin()
    .args(["--json", "-c", &use_command, "-c", "count"])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["schema_version"], 1);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(
    envelopes[0]["data"],
    serde_json::json!({
      "dataset": {
        "columns": [
          {"data_type": "BIGINT", "name": "x"},
          {"data_type": "BIGINT", "name": "y"}
        ],
        "execution_mode": "eager",
        "label_metadata": null,
        "lazy_engine": null,
        "panel_metadata": null,
        "path": csv_path.to_string_lossy(),
        "row_count": 2
      }
    })
  );
  assert_eq!(envelopes[1]["schema_version"], 1);
  assert_eq!(envelopes[1]["result_type"], "CountResult");
  assert_eq!(envelopes[1]["data"], serde_json::json!({"row_count": 2}));
}

#[test]
fn unsupported_json_batch_result_is_rejected_before_command_execution() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let use_command = format!("use {}", csv_path.display());
  let exported_path = temp_dir.path().join("must-not-exist.csv");
  let export_command = format!("export {}", exported_path.display());
  let output = tabdat_bin()
    .args([
      "--json",
      "-c",
      &use_command,
      "-c",
      "count",
      "-c",
      &export_command,
    ])
    .output()
    .expect("binary should execute");

  assert_eq!(output.status.code(), Some(2));
  assert!(!exported_path.exists());
  assert_eq!(
    output.stderr,
    b"Error: --json successful batch results are currently supported only for use, count, head, and tail\n"
  );
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["result_type"], "CountResult");
  assert_eq!(
    envelopes[2],
    serde_json::json!({
      "schema_version": 1,
      "error": {
        "message": "--json successful batch results are currently supported only for use, count, head, and tail",
        "type": "JsonResultUnsupported"
      }
    })
  );
}

#[test]
fn json_batch_keeps_execution_errors_in_the_error_envelope() {
  let output = tabdat_bin()
    .args(["--json", "-c", "count"])
    .output()
    .expect("binary should execute");

  assert_eq!(output.status.code(), Some(1));
  assert_eq!(
    output.stderr,
    b"Error: count requires an active dataset; run use <path> first\n"
  );
  let envelope: serde_json::Value =
    serde_json::from_slice(&output.stdout).expect("valid JSON envelope");
  assert_eq!(envelope["schema_version"], 1);
  assert_eq!(envelope["error"]["type"], "ExecutionError");
  assert_eq!(
    envelope["error"]["message"],
    "count requires an active dataset; run use <path> first"
  );
}

#[test]
fn json_script_emits_load_and_count_results_without_script_metadata() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let script_path = write_script(
    &temp_dir,
    "success.td",
    &format!("use {}\ncount\n", csv_path.display()),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &script_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  assert!(!stdout.contains("Script:"));
  assert!(!stdout.contains(". use"));
  assert!(!stdout.contains(". count"));
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["schema_version"], 1);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[0]["data"]["dataset"]["row_count"], 2);
  assert_eq!(envelopes[1]["schema_version"], 1);
  assert_eq!(envelopes[1]["result_type"], "CountResult");
  assert_eq!(envelopes[1]["data"], serde_json::json!({"row_count": 2}));
}

#[test]
fn json_nested_scripts_stream_supported_results_in_execution_order() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let child_path = write_script(&temp_dir, "child.td", "count\n");
  let parent_path = write_script(
    &temp_dir,
    "parent.td",
    &format!("use {}\nrun child.td\ncount\n", csv_path.display()),
  );
  assert!(child_path.exists());
  let output = tabdat_bin()
    .args(["--json", "-f", &parent_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(
    envelopes
      .iter()
      .map(|envelope| envelope["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["LoadResult", "CountResult", "CountResult"]
  );
}

#[test]
fn json_script_rejects_unsupported_commands_before_they_execute() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let exported_path = temp_dir.path().join("must-not-exist.csv");
  let script_path = write_script(
    &temp_dir,
    "unsupported.td",
    &format!(
      "use {}\nexport {}\ncount\n",
      csv_path.display(),
      exported_path.display()
    ),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &script_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert_eq!(output.status.code(), Some(1));
  assert!(!exported_path.exists());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["error"]["type"], "ScriptError");
  let resolved_path = script_path.canonicalize().expect("resolve script path");
  assert_eq!(
    envelopes[1]["error"]["path"],
    resolved_path.to_string_lossy().as_ref()
  );
  assert_eq!(envelopes[1]["error"]["line"], 2);
  assert_eq!(
    envelopes[1]["error"]["message"],
    "--json successful script results are currently supported only for use, count, head, and tail"
  );
  assert!(
    String::from_utf8(output.stderr)
      .expect("utf-8")
      .contains(&format!("{}:2:", resolved_path.display()))
  );
}

#[test]
fn json_nested_script_error_uses_child_path_and_emits_one_error() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let exported_path = temp_dir.path().join("child-output.csv");
  let child_path = write_script(
    &temp_dir,
    "child.td",
    &format!("export {}\n", exported_path.display()),
  );
  let parent_path = write_script(
    &temp_dir,
    "parent.td",
    &format!("use {}\nrun child.td\ncount\n", csv_path.display()),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &parent_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert_eq!(output.status.code(), Some(1));
  assert!(!exported_path.exists());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["error"]["type"], "ScriptError");
  let resolved_child_path = child_path.canonicalize().expect("resolve child path");
  assert_eq!(
    envelopes[1]["error"]["path"],
    resolved_child_path.to_string_lossy().as_ref()
  );
  assert_eq!(envelopes[1]["error"]["line"], 1);
}

#[test]
fn json_script_runtime_error_keeps_prior_result_and_source_location() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let missing_path = temp_dir.path().join("missing.csv");
  let script_path = write_script(
    &temp_dir,
    "runtime-error.td",
    &format!(
      "use {}\nuse {}\ncount\n",
      csv_path.display(),
      missing_path.display()
    ),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &script_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert_ne!(output.status.code(), Some(0));
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();

  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["error"]["type"], "ScriptError");
  assert_eq!(envelopes[1]["error"]["line"], 2);
  let resolved_path = script_path.canonicalize().expect("resolve script path");
  assert_eq!(
    envelopes[1]["error"]["path"],
    resolved_path.to_string_lossy().as_ref()
  );
  assert!(
    String::from_utf8(output.stderr)
      .expect("utf-8")
      .contains(&format!("{}:2:", resolved_path.display()))
  );
}

#[test]
fn json_script_does_not_preflight_commands_in_inactive_branches() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let exported_path = temp_dir.path().join("inactive-output.csv");
  let script_path = write_script(
    &temp_dir,
    "inactive.td",
    &format!(
      "use {}\nif 0\nexport {}\nend\ncount\n",
      csv_path.display(),
      exported_path.display()
    ),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &script_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  assert!(!exported_path.exists());
  let stdout = String::from_utf8(output.stdout).expect("utf-8");
  let envelopes = stdout
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON line"))
    .collect::<Vec<_>>();
  assert_eq!(envelopes.len(), 2);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["result_type"], "CountResult");
}

#[test]
fn non_json_script_execution_keeps_its_existing_output_behavior() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  let script_path = write_script(
    &temp_dir,
    "plain.td",
    &format!("use {}\ncount\n", csv_path.display()),
  );
  let output = tabdat_bin()
    .args(["-f", &script_path.to_string_lossy()])
    .output()
    .expect("binary should execute");

  assert!(output.status.success());
  assert!(output.stdout.is_empty());
  assert!(output.stderr.is_empty());
}
