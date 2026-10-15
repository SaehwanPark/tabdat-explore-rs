#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-cli-text-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(
      root.join("data.csv"),
      "g,x,y\nA,1,2\nA,2,5\nB,3,5\nB,4,9\n,5,8\n",
    )
    .unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn batch(&self, mode: Option<&str>, commands: &[&str]) -> Output {
    let mut args = Vec::new();
    if let Some(mode) = mode {
      args.push(mode);
    }
    for source in commands {
      args.extend(["-c", source]);
    }
    self.run(&args)
  }
  fn shell(&self, source: &str, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    if let Err(error) = child.stdin.take().unwrap().write_all(source.as_bytes()) {
      assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
    child.wait_with_output().unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn success(output: Output) -> Vec<u8> {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  output.stdout
}
fn json_lines(bytes: &[u8]) -> Vec<Value> {
  std::str::from_utf8(bytes)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
#[test]
fn opt_in_batch_text_matches_readable_session_with_analysis_and_persistence_without_changing_quiet_or_json()
 {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "describe",
    "summarize x y",
    "by g: count",
    "label variable x 'Predictor 雪'",
    "label list",
    "regress y x",
    "lincom x",
    "test x == 0",
    "predict fitted",
    "assert fitted > 0",
    "datasignature",
    "head 10",
    "save fitted.parquet, replace",
    "export fitted.csv, replace",
  ];
  let text = success(fixture.batch(Some("--text"), &commands));
  assert_eq!(
    text,
    success(fixture.shell(&format!("{}\nexit\n", commands.join("\n")), &[]))
  );
  let human = std::str::from_utf8(&text).unwrap();
  for marker in [
    "Loaded 5 rows",
    "Predictor 雪",
    "Grouped results",
    "fitted",
    "Signature:",
    "Saved",
    "Exported",
  ] {
    assert!(human.contains(marker), "{marker}");
  }
  assert!(!human.contains("terminal details"));
  let text_rows = json_lines(&success(
    fixture.batch(Some("--json"), &["use fitted.parquet", "head 10"]),
  ))[1]["data"]
    .clone();
  assert!(success(fixture.batch(None, &commands)).is_empty());
  let quiet_rows = json_lines(&success(
    fixture.batch(Some("--json"), &["use fitted.parquet", "head 10"]),
  ))[1]["data"]
    .clone();
  assert_eq!(text_rows, quiet_rows);
  let envelopes = json_lines(&success(fixture.batch(Some("--json"), &commands)));
  assert_eq!(envelopes.len(), commands.len());
  assert!(envelopes.iter().all(|v| v["schema_version"] == 1));
}
#[test]
fn text_nested_and_batch_run_stream_leaves_while_script_summary_policy_is_explicit() {
  let fixture = Fixture::new();
  let commands = [
    "use data.csv",
    "by g: summarize x y",
    "collapse mean x y, by(g)",
    "head 10",
    "save grouped.parquet, replace",
  ];
  let batch = success(fixture.batch(Some("--text"), &commands));
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  fs::write(fixture.0.join("child.td"), commands[1..].join("\n")).unwrap();
  assert_eq!(success(fixture.run(&["--text", "-f", "parent.td"])), batch);
  assert_eq!(success(fixture.run(&["--text", "parent.td"])), batch);
  let batch_run = success(fixture.batch(Some("--text"), &["run parent.td", "count"]));
  assert!(batch_run.starts_with(&batch));
  let suffix = std::str::from_utf8(&batch_run[batch.len()..]).unwrap();
  assert!(suffix.starts_with("Executed "));
  assert!(suffix.ends_with("3 rows\n"));
  assert!(success(fixture.run(&["-f", "parent.td"])).is_empty());
  let denied = fixture.batch(Some("--json"), &commands);
  assert_eq!(denied.status.code(), Some(2));
  let messages = json_lines(&denied.stdout);
  assert_eq!(messages.len(), 3);
  assert_eq!(messages[0]["result_type"], "LoadResult");
  assert_eq!(messages[1]["result_type"], "ByResult");
  assert_eq!(messages[2]["error"]["type"], "JsonResultUnsupported");
}
#[test]
fn text_failures_keep_prior_output_effects_and_source_classification_without_success_or_later_files()
 {
  let fixture = Fixture::new();
  for (source, code) in [
    ("assert x > 10", 1),
    ("summarize absent", 1),
    ("summarize x,", 2),
  ] {
    let output = fixture.batch(
      Some("--text"),
      &["use data.csv", source, "export forbidden.csv"],
    );
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(
      String::from_utf8(output.stdout).unwrap(),
      "Loaded 5 rows, 3 columns from data.csv\n"
    );
    assert!(!output.stderr.is_empty());
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  for (source, code) in [
    ("assert x > 10", 1),
    ("summarize x,", 2),
    ("run absent.td", 3),
  ] {
    fs::write(
      fixture.0.join("child.td"),
      format!("{source}\nexport forbidden.csv\n"),
    )
    .unwrap();
    fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
    let output = fixture.run(&["--text", "-f", "parent.td"]);
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(
      String::from_utf8(output.stdout).unwrap(),
      "Loaded 5 rows, 3 columns from data.csv\n"
    );
    let location = if code == 3 {
      "absent.td:1:"
    } else {
      "child.td:1:"
    };
    assert!(String::from_utf8(output.stderr).unwrap().contains(location));
    assert!(!fixture.0.join("forbidden.csv").exists());
    // A batch run remains an execution command: its legacy error exit is 1.
    let output = fixture.batch(Some("--text"), &["run parent.td", "export forbidden.csv"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  assert_eq!(
    fixture.run(&["--text", "-f", "missing.td"]).status.code(),
    Some(3)
  );
}
#[test]
fn text_mode_conflicts_require_execution_and_do_not_consume_piped_mutations() {
  let fixture = Fixture::new();
  for args in [
    vec!["--text"],
    vec!["--text", "--json", "-c", "use data.csv"],
    vec!["--text", "--list-commands"],
    vec!["--text", "--list-command-effects"],
    vec!["--text", "--help-topic", "summarize"],
    vec!["--text", "--describe-command", "count"],
    vec!["--text", "--explain", "-c", "count"],
  ] {
    let output = fixture.shell("use data.csv\nexport forbidden.csv\nexit\n", &args);
    assert_eq!(output.status.code(), Some(2), "{args:?}");
    assert!(output.stdout.is_empty());
    assert!(!fixture.0.join("forbidden.csv").exists());
  }
  for flag in ["--help", "--version"] {
    let output = fixture.run(&["--text", "--json", "-f", "parent.td", flag]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
  }
  let help = String::from_utf8(success(fixture.run(&["--help"]))).unwrap();
  assert!(help.contains("--text"));
  assert!(help.contains("readable batch/script results"));
}
