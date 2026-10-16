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
fn coding_output_failure_stops_later_category_commands_and_export() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=5 {
        let fixture = Fixture::new();
        let exported = fixture.0.join("coded.csv");
        let export = format!("export \"{}\"", exported.display());
        let prepare = "recode x (1='b') (2='a') (else='b'), generate(category)";
        let encode = "encode category, generate(code)";
        let decode = "decode code, generate(readable)";
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, format!("{encode}\n{decode}\n{export}\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\n{prepare}\nrun {}\n", fixture.load(), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
            "-c",
            &fixture.load(),
            "-c",
            prepare,
            "-c",
            encode,
            "-c",
            decode,
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
            "x,y,category,code,readable\n1,2,b,2,b\n2,5,a,1,a\n3,5,b,2,b\n4,9,b,2,b\n"
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
fn sql_output_failure_stops_later_publication_activation_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=7 {
        let fixture = Fixture::new();
        let first = fixture.0.join("cohort.csv");
        let second = fixture.0.join("second.csv");
        let export_first = format!("export \"{}\"", first.display());
        let export_second = format!("export \"{}\"", second.display());
        let commands = [
          "sql select count(*) as n from active",
          "sql select x,y from active where x>=2 order by x desc into cohort",
          "use cohort",
          &export_first,
          "sql select x from active where x>2 order by x into second",
          &export_second,
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
            "-c",
            commands[5],
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(first.exists(), fail_at >= 5);
        assert_eq!(second.exists(), fail_at == 7);
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(&first).unwrap(),
            "x,y\n4,9\n3,5\n2,5\n"
          );
        }
        if second.exists() {
          assert_eq!(std::fs::read_to_string(&second).unwrap(), "x\n3\n4\n");
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
fn join_output_failure_stops_later_enrichment_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=7 {
        let fixture = Fixture::new();
        let first = fixture.0.join("joined.csv");
        let second = fixture.0.join("second.csv");
        let load = fixture.load();
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          load.as_str(),
          "sql select x,y from active order by x into lookup",
          load.as_str(),
          "join lookup on x",
          &first_export,
          "join lookup on x",
          &second_export,
        ];
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, commands[3..].join("\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", commands[..3].join("\n"), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
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
            "-c",
            commands[5],
            "-c",
            commands[6],
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(first.exists(), fail_at >= 5);
        assert_eq!(second.exists(), fail_at == 7);
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(&first).unwrap(),
            "x,y,y_right\n1,2,2\n2,5,5\n3,5,5\n4,9,9\n"
          );
        }
        if second.exists() {
          assert_eq!(
            std::fs::read_to_string(&second).unwrap(),
            "x,y,y_right,y_right_2\n1,2,2,2\n2,5,5,5\n3,5,5,5\n4,9,9,9\n"
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
fn append_output_failure_stops_later_stacking_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=7 {
        let fixture = Fixture::new();
        let first = fixture.0.join("stacked.csv");
        let second = fixture.0.join("second.csv");
        let load = fixture.load();
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          load.as_str(),
          "sql select x,y from active order by x into followup",
          load.as_str(),
          "append followup",
          &first_export,
          "append followup",
          &second_export,
        ];
        let args = if script {
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(&child, commands[3..].join("\n")).unwrap();
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", commands[..3].join("\n"), child.display()),
          )
          .unwrap();
          arguments(&["--json", "-f", parent.to_str().unwrap()])
        } else {
          arguments(&[
            "--json",
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
            "-c",
            commands[5],
            "-c",
            commands[6],
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(first.exists(), fail_at >= 5);
        assert_eq!(second.exists(), fail_at == 7);
        let rows = "1,2\n2,5\n3,5\n4,9\n";
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(&first).unwrap(),
            format!("x,y\n{rows}{rows}")
          );
        }
        if second.exists() {
          assert_eq!(
            std::fs::read_to_string(&second).unwrap(),
            format!("x,y\n{rows}{rows}{rows}")
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
fn grouped_output_failure_stops_later_inspection_and_files() {
  for grouped_commands in [
    ["by x: count", "by x y: count", "by y: count"],
    [
      "by x: summarize y",
      "by x y: summarize x y",
      "by y: summarize",
    ],
  ] {
    for script in [false, true] {
      for write_failure in [false, true] {
        for fail_at in 1..=6 {
          let fixture = Fixture::new();
          let first = fixture.0.join("counted.csv");
          let second = fixture.0.join("second.csv");
          let first_export = format!("export \"{}\"", first.display());
          let second_export = format!("export \"{}\"", second.display());
          let commands = [
            grouped_commands[0],
            grouped_commands[1],
            &first_export,
            grouped_commands[2],
            &second_export,
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
          assert_eq!(first.exists(), fail_at >= 4);
          assert_eq!(second.exists(), fail_at == 6);
          for path in [&first, &second] {
            if path.exists() {
              assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                "x,y\n1,2\n2,5\n3,5\n4,9\n"
              );
            }
          }
          assert_eq!(
            output.completed_lines(),
            fail_at - usize::from(write_failure)
          );
        }
      }
    }
  }
}

#[test]
fn reshape_output_failure_stops_later_pivots_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=6 {
        let fixture = Fixture::new();
        let first = fixture.0.join("long.csv");
        let second = fixture.0.join("wide.csv");
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          "sql select x as id,y as val_1,x as val_2 from active into base",
          "reshape long val, i(id) j(t)",
          &first_export,
          "reshape wide val, i(id) j(t)",
          &second_export,
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
        assert_eq!(first.exists(), fail_at >= 4);
        assert_eq!(second.exists(), fail_at == 6);
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(&first).unwrap(),
            "id,t,val\n1,1,2\n1,2,1\n2,1,5\n2,2,2\n3,1,5\n3,2,3\n4,1,9\n4,2,4\n"
          );
        }
        if second.exists() {
          assert_eq!(
            std::fs::read_to_string(&second).unwrap(),
            "id,val_1,val_2\n1,2,1\n2,5,2\n3,5,3\n4,9,4\n"
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
fn row_filter_output_failure_stops_later_selection_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=5 {
        let fixture = Fixture::new();
        let first = fixture.0.join("kept.csv");
        let second = fixture.0.join("dropped.csv");
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          "keep if x >= 2",
          &first_export,
          "drop if x == 3",
          &second_export,
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
          ])
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
        assert!(errors.is_empty());
        assert_eq!(first.exists(), fail_at >= 3);
        assert_eq!(second.exists(), fail_at == 5);
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(first).unwrap(),
            "x,y\n2,5\n3,5\n4,9\n"
          );
        }
        if second.exists() {
          assert_eq!(std::fs::read_to_string(second).unwrap(), "x,y\n2,5\n4,9\n");
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
fn conditional_count_output_failures_stop_later_files_in_json_and_text() {
  for mode in ["--json", "--text"] {
    for script in [false, true] {
      for write_failure in [false, true] {
        for fail_at in 1..=6 {
          let fixture = Fixture::new();
          let first = fixture.0.join("first.csv");
          let second = fixture.0.join("second.csv");
          let later = fixture.0.join("later.csv");
          let commands = [
            fixture.load(),
            "count if x > 1".into(),
            format!("export {}", first.display()),
            "count if y == 5".into(),
            format!("export {}", second.display()),
            "count".into(),
            format!("export {}", later.display()),
          ];
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", commands[0], child.display()),
          )
          .unwrap();
          std::fs::write(&child, commands[1..].join("\n")).unwrap();
          let args = if script {
            arguments(&[mode, "-f", parent.to_str().unwrap()])
          } else {
            let mut args = vec![mode];
            for command in &commands {
              args.extend(["-c", command]);
            }
            arguments(&args)
          };
          let mut output = FailedOutput::new(write_failure, fail_at);
          let mut errors = Vec::new();
          let error = run_cli_with_output(args, &mut output, &mut errors).unwrap_err();
          assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
          assert!(errors.is_empty());
          assert_eq!(first.exists(), fail_at >= 3);
          assert_eq!(second.exists(), fail_at >= 5);
          assert!(!later.exists());
          for path in [&first, &second] {
            if path.exists() {
              assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                "x,y\n1,2\n2,5\n3,5\n4,9\n"
              );
            }
          }
          assert_eq!(output.flushes, fail_at - usize::from(write_failure));
        }
      }
    }
  }
}

#[test]
fn conditional_frequency_output_failures_stop_later_files_in_json_and_text() {
  for mode in ["--json", "--text"] {
    for script in [false, true] {
      for write_failure in [false, true] {
        for fail_at in 1..=6 {
          let fixture = Fixture::new();
          let first = fixture.0.join("first.csv");
          let second = fixture.0.join("second.csv");
          let later = fixture.0.join("later.csv");
          let commands = [
            fixture.load(),
            "tabulate x if x > 1".into(),
            format!("export {}", first.display()),
            "tabulate x y if x < 4, row col missing".into(),
            format!("export {}", second.display()),
            "count".into(),
            format!("export {}", later.display()),
          ];
          let parent = fixture.0.join("parent.td");
          let child = fixture.0.join("child.td");
          std::fs::write(
            &parent,
            format!("{}\nrun {}\n", commands[0], child.display()),
          )
          .unwrap();
          std::fs::write(&child, commands[1..].join("\n")).unwrap();
          let args = if script {
            arguments(&[mode, "-f", parent.to_str().unwrap()])
          } else {
            let mut args = vec![mode];
            for command in &commands {
              args.extend(["-c", command]);
            }
            arguments(&args)
          };
          let mut output = FailedOutput::new(write_failure, fail_at);
          let mut errors = Vec::new();
          let error = run_cli_with_output(args, &mut output, &mut errors).unwrap_err();
          assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
          assert!(errors.is_empty());
          assert_eq!(first.exists(), fail_at >= 3);
          assert_eq!(second.exists(), fail_at >= 5);
          assert!(!later.exists());
          for path in [&first, &second] {
            if path.exists() {
              assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                "x,y\n1,2\n2,5\n3,5\n4,9\n"
              );
            }
          }
          assert_eq!(output.flushes, fail_at - usize::from(write_failure));
        }
      }
    }
  }
}

#[test]
fn conditional_summary_output_failure_stops_later_filters_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=6 {
        let fixture = Fixture::new();
        let first = fixture.0.join("first.csv");
        let second = fixture.0.join("second.csv");
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          "summarize x y if x >= 2",
          &first_export,
          "summarize if x < 3",
          "drop if x == 1",
          &second_export,
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
        assert_eq!(first.exists(), fail_at >= 3);
        assert_eq!(second.exists(), fail_at == 6);
        if first.exists() {
          assert_eq!(
            std::fs::read_to_string(first).unwrap(),
            "x,y\n1,2\n2,5\n3,5\n4,9\n"
          );
        }
        if second.exists() {
          assert_eq!(
            std::fs::read_to_string(second).unwrap(),
            "x,y\n2,5\n3,5\n4,9\n"
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
fn detail_output_failure_stops_later_files_in_json_text_and_nested_streams() {
  for mode in ["--json", "--text"] {
    for route in ["batch", "file", "run"] {
      for write_failure in [false, true] {
        for fail_at in [2, 3] {
          let fixture = Fixture::new();
          let later = fixture.0.join("never.csv");
          let export = format!("export \"{}\"", later.display());
          let commands = [
            fixture.load(),
            "summarize x y, detail".into(),
            "summarize x if x >= 2, detail".into(),
            export,
          ];
          let args = if route == "batch" {
            let values: Vec<_> = std::iter::once(mode)
              .chain(commands.iter().flat_map(|source| ["-c", source.as_str()]))
              .collect();
            arguments(&values)
          } else {
            let parent = fixture.0.join("parent.td");
            let child = fixture.0.join("child.td");
            std::fs::write(
              &parent,
              format!("{}\nrun \"{}\"\n", commands[0], child.display()),
            )
            .unwrap();
            std::fs::write(&child, commands[1..].join("\n")).unwrap();
            if route == "file" {
              arguments(&[mode, "-f", parent.to_str().unwrap()])
            } else {
              arguments(&[mode, "-c", &format!("run \"{}\"", parent.display())])
            }
          };
          let mut output = FailedOutput::new(write_failure, fail_at);
          let mut errors = Vec::new();
          assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
          assert!(errors.is_empty());
          assert!(!later.exists());
        }
      }
    }
  }
}

#[test]
fn status_output_failure_stops_later_files_in_json_text_and_nested_streams() {
  for mode in ["--json", "--text"] {
    for route in ["batch", "file", "run"] {
      for write_failure in [false, true] {
        for fail_at in [1, 4] {
          let fixture = Fixture::new();
          let later = fixture.0.join("never.csv");
          let commands = [
            "status".into(),
            fixture.load(),
            "regress y x".into(),
            "status".into(),
            format!("export \"{}\"", later.display()),
          ];
          let args = if route == "batch" {
            let values: Vec<_> = std::iter::once(mode)
              .chain(commands.iter().flat_map(|source| ["-c", source.as_str()]))
              .collect();
            arguments(&values)
          } else {
            let parent = fixture.0.join("parent.td");
            let child = fixture.0.join("child.td");
            std::fs::write(
              &parent,
              format!("{}\nrun \"{}\"\n", commands[0], child.display()),
            )
            .unwrap();
            std::fs::write(&child, commands[1..].join("\n")).unwrap();
            if route == "file" {
              arguments(&[mode, "-f", parent.to_str().unwrap()])
            } else {
              arguments(&[mode, "-c", &format!("run \"{}\"", parent.display())])
            }
          };
          let mut output = FailedOutput::new(write_failure, fail_at);
          let mut errors = Vec::new();
          assert!(run_cli_with_output(args, &mut output, &mut errors).is_err());
          assert!(errors.is_empty());
          assert_eq!(output.flushes, fail_at - usize::from(write_failure));
          assert!(!later.exists());
        }
      }
    }
  }
}

#[test]
fn key_guard_output_failure_stops_later_guards_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=6 {
        let fixture = Fixture::new();
        let first = fixture.0.join("diagnosed.csv");
        let second = fixture.0.join("validated.csv");
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          "duplicates y",
          "isid x",
          &first_export,
          "duplicates x y",
          &second_export,
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
        assert_eq!(first.exists(), fail_at >= 4);
        assert_eq!(second.exists(), fail_at == 6);
        for path in [&first, &second] {
          if path.exists() {
            assert_eq!(
              std::fs::read_to_string(path).unwrap(),
              "x,y\n1,2\n2,5\n3,5\n4,9\n"
            );
          }
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
fn reproducibility_guard_output_failure_stops_later_guards_and_files() {
  for script in [false, true] {
    for write_failure in [false, true] {
      for fail_at in 1..=6 {
        let fixture = Fixture::new();
        let first = fixture.0.join("checked.csv");
        let second = fixture.0.join("fingerprinted.csv");
        let first_export = format!("export \"{}\"", first.display());
        let second_export = format!("export \"{}\"", second.display());
        let commands = [
          "assert x > 0",
          "datasignature",
          &first_export,
          "datasignature",
          &second_export,
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
        assert_eq!(first.exists(), fail_at >= 4);
        assert_eq!(second.exists(), fail_at == 6);
        for path in [&first, &second] {
          if path.exists() {
            assert_eq!(
              std::fs::read_to_string(path).unwrap(),
              "x,y\n1,2\n2,5\n3,5\n4,9\n"
            );
          }
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
fn collapse_output_failure_stops_later_aggregation_and_files() {
  for statistic in ["count", "mean"] {
    for script in [false, true] {
      for write_failure in [false, true] {
        for fail_at in 1..=6 {
          let fixture = Fixture::new();
          let first = fixture.0.join("counts.csv");
          let second = fixture.0.join("second.csv");
          let first_export = format!("export \"{}\"", first.display());
          let second_export = format!("export \"{}\"", second.display());
          let collapse = format!("collapse {statistic} y, by(x)");
          let recollapse = format!("collapse {statistic} {statistic}_y, by(x)");
          let commands = [
            &collapse,
            &first_export,
            &recollapse,
            &second_export,
            "head 5",
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
          assert_eq!(first.exists(), fail_at >= 3);
          assert_eq!(second.exists(), fail_at >= 5);
          if first.exists() {
            assert_eq!(
              std::fs::read_to_string(&first).unwrap(),
              if statistic == "count" {
                "x,count_y\n1,1\n2,1\n3,1\n4,1\n"
              } else {
                "x,mean_y\n1,2.0\n2,5.0\n3,5.0\n4,9.0\n"
              }
            );
          }
          if second.exists() {
            assert_eq!(
              std::fs::read_to_string(&second).unwrap(),
              if statistic == "count" {
                "x,count_count_y\n1,1\n2,1\n3,1\n4,1\n"
              } else {
                "x,mean_mean_y\n1,2.0\n2,5.0\n3,5.0\n4,9.0\n"
              }
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
}

#[test]
fn reported_result_failures_stop_later_batch_script_and_batch_run_files() {
  for mode in 0..4 {
    for write_failure in [false, true] {
      for fail_at in 1..=if mode == 2 { 7 } else { 6 } {
        let fixture = Fixture::new();
        let first = fixture.0.join("first.csv");
        let second = fixture.0.join("second.csv");
        let later = fixture.0.join("later.csv");
        let commands = [
          fixture.load(),
          "summarize x y".into(),
          format!("export {}", first.display()),
          "by x: count".into(),
          "regress y x".into(),
          format!("export {}", second.display()),
        ];
        let parent = fixture.0.join("parent.td");
        let child = fixture.0.join("child.td");
        std::fs::write(
          &parent,
          format!("{}\nrun {}\n", commands[0], child.display()),
        )
        .unwrap();
        let later_export = format!("export {}", later.display());
        let mut child_commands = commands[1..].join("\n");
        if mode == 1 {
          child_commands.push_str(&format!("\n{later_export}\n"));
        }
        std::fs::write(&child, child_commands).unwrap();
        let args = match mode {
          0 => arguments(&[
            "--text",
            "-c",
            &commands[0],
            "-c",
            &commands[1],
            "-c",
            &commands[2],
            "-c",
            &commands[3],
            "-c",
            &commands[4],
            "-c",
            &commands[5],
            "-c",
            &later_export,
          ]),
          1 => arguments(&["--text", "-f", parent.to_str().unwrap()]),
          _ => arguments(&[
            if mode == 3 { "--json" } else { "--text" },
            "-c",
            &format!("run {}", parent.display()),
            "-c",
            &later_export,
          ]),
        };
        let mut output = FailedOutput::new(write_failure, fail_at);
        let mut errors = Vec::new();
        let error = run_cli_with_output(args, &mut output, &mut errors).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert!(errors.is_empty());
        assert_eq!(first.exists(), fail_at >= 3);
        assert_eq!(second.exists(), fail_at >= 6);
        assert!(!later.exists());
        for file in [&first, &second] {
          if file.exists() {
            assert_eq!(
              std::fs::read_to_string(file).unwrap(),
              "x,y\n1,2\n2,5\n3,5\n4,9\n"
            );
          }
        }
        assert_eq!(output.flushes, fail_at - usize::from(write_failure));
      }
    }
  }
}

#[test]
fn json_batch_run_diagnostic_and_error_envelope_failures_stop_later_commands() {
  for write_failure in [false, true] {
    for diagnostic in [false, true] {
      let fixture = Fixture::new();
      let parent = fixture.0.join("failed script.td");
      let later = fixture.0.join("later.csv");
      std::fs::write(&parent, "assert x > 10\n").unwrap();
      let args = arguments(&[
        "--json",
        "-c",
        &fixture.load(),
        "-c",
        &format!("run '{}'", parent.display()),
        "-c",
        &format!("export '{}'", later.display()),
      ]);
      let mut output = FailedOutput::new(write_failure, if diagnostic { usize::MAX } else { 2 });
      let mut errors = FailedOutput::new(write_failure, if diagnostic { 1 } else { usize::MAX });
      let error = run_cli_with_output(args, &mut output, &mut errors).unwrap_err();
      assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
      assert_eq!(
        output.completed_lines(),
        if diagnostic || write_failure { 1 } else { 2 }
      );
      assert!(!later.exists());
    }
  }
}

#[test]
fn text_diagnostic_failures_do_not_execute_later_commands() {
  for script in [false, true] {
    for write_failure in [false, true] {
      let fixture = Fixture::new();
      let later = fixture.0.join("later.csv");
      let failed = "assert x > 10";
      let export = format!("export {}", later.display());
      let parent = fixture.0.join("parent.td");
      std::fs::write(&parent, format!("{}\n{failed}\n{export}\n", fixture.load())).unwrap();
      let args = if script {
        arguments(&["--text", "-f", parent.to_str().unwrap()])
      } else {
        arguments(&["--text", "-c", &fixture.load(), "-c", failed, "-c", &export])
      };
      let mut output = Vec::new();
      let error = run_cli_with_output(args, &mut output, &mut FailedOutput::new(write_failure, 1))
        .unwrap_err();
      assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
      assert!(
        String::from_utf8(output)
          .unwrap()
          .starts_with("Loaded 4 rows")
      );
      assert!(!later.exists());
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
