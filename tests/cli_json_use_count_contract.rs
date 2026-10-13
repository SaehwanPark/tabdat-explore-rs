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
  assert!(
    stdout.contains("supports use/describe/count/codebook/summarize/missing/tabulate/head/tail")
  );
  assert!(stdout.contains("scripts stream"));
  assert!(stdout.contains("regress/predict/lincom/test/ttest/estat vif"));
  assert!(stdout.contains("save/export"));
  assert!(stdout.contains("generate/replace"));
  assert!(stdout.contains("use/describe/count/codebook/summarize/missing/tabulate/head/tail"));
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

fn summary_csv(dir: &TestDir) -> PathBuf {
  let path = dir.path().join("summary.csv");
  fs::write(&path, "x,y,label\n1,10,a\n3,,b\n5,,c\n,,d\n").unwrap();
  path
}

fn json_lines(output: &[u8]) -> Vec<serde_json::Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn json_summaries_preserve_requested_order_duplicates_and_singleton_missingness() {
  let dir = TestDir::new();
  let path = summary_csv(&dir);
  let output = tabdat_bin()
    .args([
      "--json",
      "-c",
      &format!("use {}", path.display()),
      "-c",
      "summarize y x y",
      "-c",
      "summarize",
    ])
    .output()
    .unwrap();
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelopes = json_lines(&output.stdout);
  assert_eq!(envelopes.len(), 3);
  let y = serde_json::json!({"variable": "y", "count": 1, "mean": 10.0, "std_dev": null, "minimum": 10, "maximum": 10});
  // Hand-checkable independent moments: [1,3,5] has mean 3 and sample SD 2.
  let x = serde_json::json!({"variable": "x", "count": 3, "mean": 3.0, "std_dev": 2.0, "minimum": 1, "maximum": 5});
  assert_eq!(
    envelopes[1],
    serde_json::json!({"schema_version": 1, "result_type": "SummarizeResult", "data": {"rows": [y.clone(), x.clone(), y.clone()]}})
  );
  assert_eq!(envelopes[2]["data"], serde_json::json!({"rows": [x, y]}));
}

#[test]
fn json_nested_scripts_stream_summaries_in_leaf_order_without_metadata() {
  let dir = TestDir::new();
  let path = summary_csv(&dir);
  write_script(&dir, "summary-child.td", "summarize y\n");
  let parent = write_script(
    &dir,
    "summary-parent.td",
    &format!(
      "use {}\nsummarize x\nrun summary-child.td\ncount\n",
      path.display()
    ),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &parent.to_string_lossy()])
    .output()
    .unwrap();
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelopes = json_lines(&output.stdout);
  assert_eq!(
    envelopes
      .iter()
      .map(|value| value["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "SummarizeResult",
      "SummarizeResult",
      "CountResult"
    ]
  );
  assert_eq!(envelopes[1]["data"]["rows"][0]["variable"], "x");
  assert_eq!(envelopes[2]["data"]["rows"][0]["variable"], "y");
  assert_eq!(envelopes[3]["data"]["row_count"], 4);
}

#[test]
fn json_summary_errors_preserve_prior_envelopes_and_nested_locations() {
  let dir = TestDir::new();
  let path = summary_csv(&dir);
  let child = write_script(
    &dir,
    "invalid-summary.td",
    "summarize y\nsummarize absent\n",
  );
  let parent = write_script(
    &dir,
    "parent-summary.td",
    &format!("use {}\nrun invalid-summary.td\ncount\n", path.display()),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &parent.to_string_lossy()])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(1));
  let envelopes = json_lines(&output.stdout);
  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  assert_eq!(envelopes[1]["result_type"], "SummarizeResult");
  assert_eq!(envelopes[2]["error"]["type"], "ScriptError");
  assert_eq!(envelopes[2]["error"]["line"], 2);
  assert_eq!(
    envelopes[2]["error"]["path"],
    child.canonicalize().unwrap().to_string_lossy().as_ref()
  );
  assert!(
    envelopes[2]["error"]["message"]
      .as_str()
      .unwrap()
      .contains("absent")
  );
}

#[test]
fn json_summary_runtime_failures_do_not_emit_success_results() {
  let dir = TestDir::new();
  let path = summary_csv(&dir);
  for command in ["summarize label", "summarize absent"] {
    let output = tabdat_bin()
      .args([
        "--json",
        "-c",
        &format!("use {}", path.display()),
        "-c",
        command,
      ])
      .output()
      .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let envelopes = json_lines(&output.stdout);
    assert_eq!(envelopes.len(), 2);
    assert_eq!(envelopes[1]["error"]["type"], "ExecutionError");
    assert!(envelopes[1].get("result_type").is_none());
  }
  let output = tabdat_bin()
    .args(["--json", "-c", "summarize"])
    .output()
    .unwrap();
  assert_eq!(output.status.code(), Some(1));
  assert_eq!(
    json_lines(&output.stdout)[0]["error"]["type"],
    "ExecutionError"
  );
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
      "sql select 1 as marker into later",
      "-c",
      &export_command,
    ])
    .output()
    .expect("binary should execute");

  assert_eq!(output.status.code(), Some(2));
  assert!(!exported_path.exists());
  assert_eq!(
    output.stderr,
    b"Error: --json successful batch results are currently supported only for use, describe, count, codebook, summarize, missing, tabulate, generate, replace, rename, recode, label, encode, decode, keep/drop/select variable lists, sort, gsort, save, export, head, tail, regress, predict, lincom, test, ttest, and estat vif\n"
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
        "message": "--json successful batch results are currently supported only for use, describe, count, codebook, summarize, missing, tabulate, generate, replace, rename, recode, label, encode, decode, keep/drop/select variable lists, sort, gsort, save, export, head, tail, regress, predict, lincom, test, ttest, and estat vif",
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
fn canonical_script_shares_macros_with_child_and_skips_inactive_errors() {
  let temp_dir = TestDir::new();
  let csv_path = fixture_csv(&temp_dir);
  write_script(
    &temp_dir,
    "child.td",
    "# child\nseed -9223372036854775808\nlet next = count\n$action\n",
  );
  let parent = write_script(
    &temp_dir,
    "canonical.td",
    &format!(
      "# canonical workflow\nlet source = {}\nlet action = count\nseed +42\nuse $source\nif false\n$undefined\nseed 1_000\nelse\nrun child.td\nend\n$next\nexit\n$undefined\n",
      csv_path.display()
    ),
  );
  let output = tabdat_bin()
    .args(["--json", "-f", &parent.to_string_lossy()])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let envelopes: Vec<serde_json::Value> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect();
  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[0]["result_type"], "LoadResult");
  for envelope in &envelopes[1..] {
    assert_eq!(envelope["result_type"], "CountResult");
    assert_eq!(envelope["data"]["row_count"], 2);
  }
}

#[test]
fn script_seed_and_control_failures_stop_with_location_after_prior_output() {
  for (text, line, message) in [
    ("seed 1_000\ncount\n", 2, "seed expects an integer"),
    ("else\ncount\n", 2, "else without matching if"),
    ("if true\ncount\n", 2, "if block is missing end"),
    (
      "run failure.td\ncount\n",
      1,
      "recursive script inclusion is not supported",
    ),
  ] {
    let temp_dir = TestDir::new();
    let csv = fixture_csv(&temp_dir);
    let script = write_script(
      &temp_dir,
      "failure.td",
      &format!("use {}\n{text}", csv.display()),
    );
    let output = tabdat_bin()
      .args(["--json", "-f", &script.to_string_lossy()])
      .output()
      .unwrap();
    assert!(!output.status.success());
    let envelopes: Vec<serde_json::Value> = String::from_utf8(output.stdout)
      .unwrap()
      .lines()
      .map(|line| serde_json::from_str(line).unwrap())
      .collect();
    let prior_count = usize::from(text.starts_with("if true"));
    assert_eq!(envelopes.len(), 2 + prior_count);
    assert_eq!(envelopes[0]["result_type"], "LoadResult");
    let error = &envelopes.last().unwrap()["error"];
    assert_eq!(error["type"], "ScriptError");
    assert_eq!(error["line"], line);
    assert_eq!(error["message"], message);
    assert_eq!(
      error["path"],
      script.canonicalize().unwrap().to_string_lossy().as_ref()
    );
  }
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
      "use {}\nsql select 1 as marker into later\nexport {}\ncount\n",
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
    "--json successful script results are currently supported only for use, describe, count, codebook, summarize, missing, tabulate, generate, replace, rename, recode, label, encode, decode, keep/drop/select variable lists, sort, gsort, save, export, head, tail, regress, predict, lincom, test, ttest, and estat vif"
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
    &format!(
      "sql select 1 as marker into later\nexport {}\n",
      exported_path.display()
    ),
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
