use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct FailedOutput {
  write_failure: bool,
  fail_at: usize,
  flushes: usize,
  bytes: Vec<u8>,
}
impl FailedOutput {
  fn new(write_failure: bool, fail_at: usize) -> Self {
    Self {
      write_failure,
      fail_at,
      flushes: 0,
      bytes: Vec::new(),
    }
  }
  fn completed_lines(&self) -> usize {
    std::str::from_utf8(&self.bytes).unwrap().lines().count()
  }
}
impl Write for FailedOutput {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    if self.write_failure && self.flushes >= self.fail_at - 1 {
      return Err(io::Error::new(io::ErrorKind::BrokenPipe, "write failed"));
    }
    self.bytes.extend_from_slice(bytes);
    Ok(bytes.len())
  }
  fn flush(&mut self) -> io::Result<()> {
    self.flushes += 1;
    if !self.write_failure && self.flushes >= self.fail_at {
      return Err(io::Error::new(io::ErrorKind::BrokenPipe, "flush failed"));
    }
    Ok(())
  }
}
fn arguments(values: &[&str]) -> CliArgs {
  parse_args(values.iter().copied()).unwrap()
}
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-cli-output-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("data.csv"), "x,y\n1,2\n2,5\n3,5\n4,9\n").unwrap();
    Self(root)
  }
  fn load(&self) -> String {
    format!("use \"{}\"", self.0.join("data.csv").display())
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = std::fs::remove_dir_all(&self.0);
  }
}

#[test]
fn discovery_and_error_output_propagate_write_and_flush_failures() {
  for values in [
    vec!["--help"],
    vec!["--version"],
    vec!["--json", "--list-commands"],
    vec!["--json", "--list-command-effects"],
    vec!["--json", "--describe-command", "count"],
    vec!["--json", "--help-topic", "count"],
    vec!["--json", "--explain", "-c", "count"],
    vec!["--json", "--describe-command", "absent"],
    vec!["--json", "--help-topic", "absent"],
    vec!["--json", "--explain", "-c", "absent"],
    vec!["--json", "-c", "count"],
    vec!["--json", "-c", "absent"],
    vec!["--json", "-c", "export never.csv"],
    vec!["--json", "-f", "doctor"],
    vec!["--json", "-f", "missing-script-does-not-exist.td"],
  ] {
    for write_failure in [false, true] {
      let mut output = FailedOutput::new(write_failure, 1);
      let error =
        run_cli_with_output(arguments(&values), &mut output, &mut Vec::new()).unwrap_err();
      assert_eq!(error.kind(), io::ErrorKind::BrokenPipe, "{values:?}");
    }
  }
}

#[test]
fn failed_diagnostic_stream_does_not_attempt_a_json_error_or_more_commands() {
  for values in [
    vec!["--json", "-c", "count", "-c", "count"],
    vec!["-c", "absent"],
    vec!["--json", "--help-topic", "absent"],
    vec!["--json", "-c", "export never.csv"],
  ] {
    for write_failure in [false, true] {
      let mut output = Vec::new();
      let mut errors = FailedOutput::new(write_failure, 1);
      assert!(run_cli_with_output(arguments(&values), &mut output, &mut errors).is_err());
      assert!(output.is_empty());
    }
  }
}

#[test]
fn batch_stops_at_first_middle_and_final_result_output_failures() {
  let fixture = Fixture::new();
  for write_failure in [false, true] {
    for fail_at in [1, 2, 4] {
      let mut output = FailedOutput::new(write_failure, fail_at);
      let mut errors = Vec::new();
      let args = arguments(&[
        "--json",
        "-c",
        &fixture.load(),
        "-c",
        "describe",
        "-c",
        "count",
        "-c",
        "count",
      ]);
      assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
      assert!(errors.is_empty());
      assert_eq!(
        output.completed_lines(),
        fail_at - usize::from(write_failure)
      );
    }
  }
}

#[test]
fn output_failure_prevents_later_file_writes_without_rolling_back_completed_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=if script { 4 } else { 3 } {
        let fixture = Fixture::new();
        let saved = fixture.0.join("saved.parquet");
        let exported = fixture.0.join("exported.csv");
        let later = fixture.0.join("later.csv");
        let save = format!("save \"{}\"", saved.display());
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let child = fixture.0.join("child.td");
          let parent = fixture.0.join("parent.td");
          std::fs::write(&child, format!("{save}\n{export}\n")).unwrap();
          std::fs::write(
            &parent,
            format!(
              "{}\nrun {}\nexport \"{}\"\n",
              fixture.load(),
              child.display(),
              later.display()
            ),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&["--json", "-c", &fixture.load(), "-c", &save, "-c", &export])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(saved.exists(), fail_at >= 2);
        assert_eq!(exported.exists(), fail_at >= 3);
        assert_eq!(later.exists(), script && fail_at >= 4);
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn nested_script_guard_stops_later_observers_and_preserves_final_output_error() {
  let fixture = Fixture::new();
  let child = fixture.0.join("child.td");
  let parent = fixture.0.join("parent.td");
  std::fs::write(&child, "describe\npredict fitted\ncount\n").unwrap();
  std::fs::write(
    &parent,
    format!(
      "{}\nregress y x\nrun {}\ncount\n",
      fixture.load(),
      child.display()
    ),
  )
  .unwrap();
  let mut baseline = Vec::new();
  assert_eq!(
    run_cli_with_output(
      arguments(&["--json", "-f", parent.to_str().unwrap()]),
      &mut baseline,
      &mut Vec::new()
    )
    .unwrap(),
    0
  );
  let results = std::str::from_utf8(&baseline)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
    .collect::<Vec<_>>();
  assert_eq!(results.len(), 6);
  assert_eq!(results[3]["result_type"], "PredictionResult");
  for write_failure in [false, true] {
    for fail_at in [1, 3, 6] {
      let mut output = FailedOutput::new(write_failure, fail_at);
      let mut errors = Vec::new();
      let args = arguments(&["--json", "-f", parent.to_str().unwrap()]);
      let error = run_cli_with_output(args, &mut output, &mut errors).unwrap_err();
      assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
      assert!(
        errors.is_empty(),
        "do not emit a synthetic ScriptError after I/O failure"
      );
      assert_eq!(
        output.completed_lines(),
        fail_at - usize::from(write_failure)
      );
    }
  }
}
