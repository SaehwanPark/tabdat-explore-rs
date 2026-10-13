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
fn expression_output_failure_stops_later_transforms_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=4 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("transformed.csv");
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(
            &child,
            format!("generate doubled = x * 2\nreplace y = y + 1 if x > 2\n{export}\n"),
          )
          .unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            "generate doubled = x * 2",
            "-c",
            "replace y = y + 1 if x > 2",
            "-c",
            &export,
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 4);
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn projection_output_failure_stops_later_projections_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=5 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("projected.csv");
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, format!("keep y x\ndrop y\nselect x x\n{export}\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            "keep y x",
            "-c",
            "drop y",
            "-c",
            "select x x",
            "-c",
            &export,
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 5);
        if fail_at == 5 {
          assert_eq!(
            std::fs::read_to_string(&exported).unwrap(),
            "x,x_1\n1,1\n2,2\n3,3\n4,4\n"
          );
        }
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn ordering_output_failure_stops_later_sorting_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=4 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("ordered.csv");
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, format!("gsort -x\nsort y\n{export}\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            "gsort -x",
            "-c",
            "sort y",
            "-c",
            &export,
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 4);
        if fail_at == 4 {
          assert_eq!(
            std::fs::read_to_string(&exported).unwrap(),
            "x,y\n1,2\n3,5\n2,5\n4,9\n"
          );
        }
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn rename_output_failure_stops_later_renames_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=4 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("renamed.csv");
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(
            &child,
            format!("rename x predictor\nrename y outcome\n{export}\n"),
          )
          .unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            "rename x predictor",
            "-c",
            "rename y outcome",
            "-c",
            &export,
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 4);
        if fail_at == 4 {
          assert_eq!(
            std::fs::read_to_string(&exported).unwrap(),
            "predictor,outcome\n1,2\n2,5\n3,5\n4,9\n"
          );
        }
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn recode_output_failure_stops_later_recodes_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=4 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("recoded.csv");
        let export = format!("export \"{}\"", exported.display());
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, format!("recode x (min/2=1) (else=2), generate(group)\nrecode y (2=3) (3=99), replace\n{export}\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            "recode x (min/2=1) (else=2), generate(group)",
            "-c",
            "recode y (2=3) (3=99), replace",
            "-c",
            &export,
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 4);
        if fail_at == 4 {
          assert_eq!(
            std::fs::read_to_string(&exported).unwrap(),
            "x,y,group\n1,3,1\n2,5,1\n3,5,2\n4,9,2\n"
          );
        }
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
    }
  }
}

#[test]
fn label_output_failure_stops_later_dictionary_commands_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=6 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("labeled.csv");
        let export = format!("export \"{}\"", exported.display());
        let commands = [
          "label define codes 1 'One'",
          "label values x codes",
          "label list",
          "label drop codes",
          &export,
        ];
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, commands.join("\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            commands[0],
            "-c",
            commands[1],
            "-c",
            commands[2],
            "-c",
            commands[3],
            "-c",
            commands[4],
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(exported.exists(), fail_at == 6);
        if fail_at == 6 {
          assert_eq!(
            std::fs::read_to_string(&exported).unwrap(),
            "x,y\n1,2\n2,5\n3,5\n4,9\n"
          );
        }
        assert_eq!(
          output.completed_lines(),
          fail_at - usize::from(write_failure)
        );
      }
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
