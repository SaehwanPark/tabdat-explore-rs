#![forbid(unsafe_code)]

use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};
use tabdat_language::parse_command;
use tabdat_runtime::Session;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-quoted-run-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("analysis files")).unwrap();
    fs::write(root.join("data.csv"), "x,y\n0,1\n1,2\n2,4\n3,6\n").unwrap();
    Self(root)
  }
  fn batch(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
  fn shell(&self, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    child
      .stdin
      .take()
      .unwrap()
      .write_all(input.as_bytes())
      .unwrap();
    child.wait_with_output().unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn envelopes(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}
fn assert_success(output: &Output) {
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
}

#[test]
fn quoted_run_replays_nested_relative_scripts_in_all_output_modes() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("analysis files/main script.td"),
    "use data.csv\nregress y x\nrun '目录,é child.td'\npredict fitted\ncount\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("analysis files/目录,é child.td"),
    "run \"owner's script.td\"\nsummarize y\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("analysis files/owner's script.td"),
    "count\n",
  )
  .unwrap();
  let source = "run \"analysis files/main script.td\"";
  let expected = fixture.batch(&["--json", "-f", "analysis files/main script.td"]);
  fs::write(fixture.0.join("wrapper.td"), format!("{source}\n")).unwrap();
  let actual = fixture.batch(&["--json", "-f", "wrapper.td"]);
  assert_success(&expected);
  assert_success(&actual);
  let batch = fixture.batch(&["--json", "-c", source]);
  assert_success(&batch);
  assert_eq!(envelopes(&batch.stdout), envelopes(&expected.stdout));
  assert_eq!(envelopes(&actual.stdout), envelopes(&expected.stdout));
  let types: Vec<_> = envelopes(&actual.stdout)
    .iter()
    .map(|value| value["result_type"].as_str().unwrap().to_owned())
    .collect();
  assert_eq!(
    types,
    [
      "LoadResult",
      "RegressionResult",
      "CountResult",
      "SummarizeResult",
      "PredictionResult",
      "CountResult"
    ]
  );
  let plain = fixture.batch(&["-c", source]);
  assert_success(&plain);
  assert!(plain.stdout.is_empty());
  let text = fixture.batch(&["--text", "-c", source]);
  let shell = fixture.shell(&format!("{source}\nexit\n"));
  assert_success(&text);
  assert_success(&shell);
  assert_eq!(text.stdout, shell.stdout);
  let output = String::from_utf8(text.stdout).unwrap();
  assert!(
    output.find("Parameter\tCoefficient").unwrap() < output.find("Variable\tN\tMean").unwrap()
  );
  assert!(output.contains("Predicted: 4 rows, 3 columns"));
  assert!(output.contains("6 commands"), "{output}");
}

#[test]
fn malformed_and_missing_quoted_includes_are_located_and_stop_later_files() {
  let fixture = Fixture::new();
  for (source, code, error_file, error_line) in [
    (
      "run 'child script.td' junk",
      2,
      "analysis files/main script.td",
      2,
    ),
    ("run 'unterminated", 2, "analysis files/main script.td", 2),
    ("run ''", 2, "analysis files/main script.td", 2),
    (
      "run 'missing script.td'",
      3,
      "analysis files/missing script.td",
      1,
    ),
    (
      "run 'child script.td'",
      2,
      "analysis files/child script.td",
      2,
    ),
  ] {
    fs::write(
      fixture.0.join("analysis files/main script.td"),
      format!("use data.csv\n{source}\nexport never.csv\n"),
    )
    .unwrap();
    fs::write(
      fixture.0.join("analysis files/child script.td"),
      "count\nunknown_command\nexport child-never.csv\n",
    )
    .unwrap();
    for json in [false, true] {
      let args = if json {
        vec!["--json", "-f", "analysis files/main script.td"]
      } else {
        vec!["-f", "analysis files/main script.td"]
      };
      let output = fixture.batch(&args);
      assert_eq!(
        output.status.code(),
        Some(code),
        "{source}: {}",
        String::from_utf8_lossy(&output.stderr)
      );
      assert!(!fixture.0.join("never.csv").exists());
      assert!(!fixture.0.join("child-never.csv").exists());
      if json {
        let values = envelopes(&output.stdout);
        assert_eq!(values[0]["result_type"], "LoadResult");
        let error = &values.last().unwrap()["error"];
        assert_eq!(error["type"], "ScriptError");
        assert_eq!(error["line"], error_line);
        assert_eq!(
          error["path"],
          fixture
            .0
            .canonicalize()
            .unwrap()
            .join(error_file)
            .to_string_lossy()
            .as_ref()
        );
      }
    }
  }
  let batch = fixture.batch(&["-c", "run 'missing script.td'", "-c", "export never.csv"]);
  assert_eq!(batch.status.code(), Some(1));
  assert!(!fixture.0.join("never.csv").exists());
}

#[test]
fn quoted_recursion_is_still_guarded_and_shell_recovers_with_retained_model() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("analysis files/loop script.td"),
    "count\nrun './loop script.td'\nexport never.csv\n",
  )
  .unwrap();
  let output = fixture.shell(
    "use data.csv\nregress y x\nrun 'analysis files/loop script.td'\npredict fitted\ncount\nexit\n",
  );
  assert!(output.status.success());
  let errors = String::from_utf8(output.stderr).unwrap();
  assert_eq!(errors.matches("Error:").count(), 1);
  assert!(errors.contains("recursive script inclusion"));
  let text = String::from_utf8(output.stdout).unwrap();
  assert!(text.contains("Predicted: 4 rows, 3 columns"));
  assert!(text.ends_with("4 rows\n"));
  assert!(!fixture.0.join("never.csv").exists());
}

#[test]
fn successful_and_failed_quoted_scripts_preserve_owned_data_and_fit() {
  let fixture = Fixture::new();
  let mut session = Session::new();
  session
    .execute(parse_command(&format!("use '{}'", fixture.0.join("data.csv").display())).unwrap())
    .unwrap();
  session
    .execute(parse_command("regress y x").unwrap())
    .unwrap();
  let dataset = session.active_dataset().cloned();
  let model = session.last_regression().cloned();
  let before = session.execute(parse_command("head 4").unwrap()).unwrap();
  for (name, contents, success) in [
    ("read only.td", "count\nsummarize y\n", true),
    ("failed read.td", "count\nunknown_command\n", false),
    ("missing file.td", "", false),
  ] {
    let path = fixture.0.join(name);
    if !contents.is_empty() {
      fs::write(&path, contents).unwrap();
    }
    let result = session.execute(parse_command(&format!("run '{}'", path.display())).unwrap());
    assert_eq!(result.is_ok(), success, "{name}");
    assert_eq!(session.active_dataset(), dataset.as_ref());
    assert_eq!(session.last_regression(), model.as_ref());
    assert_eq!(
      session.execute(parse_command("head 4").unwrap()).unwrap(),
      before
    );
  }
}

#[test]
fn json_batch_runs_share_session_and_match_script_leaf_stream() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("analysis files/prepare.td"),
    "generate z = x * 2\nrun 'fit model.td'\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("analysis files/fit model.td"),
    "regress y z\n",
  )
  .unwrap();
  fs::write(fixture.0.join("report.td"), "count\nsummarize y\n").unwrap();
  let commands = [
    "use data.csv",
    "run 'analysis files/prepare.td'",
    "run report.td",
    "predict fitted",
    "export results.csv, replace",
    "head 4",
  ];
  fs::write(fixture.0.join("wrapper.td"), commands.join("\n")).unwrap();
  let script = fixture.batch(&["--json", "-f", "wrapper.td"]);
  assert_success(&script);
  let saved = fs::read(fixture.0.join("results.csv")).unwrap();
  let mut args = vec!["--json"];
  for command in &commands {
    args.extend(["-c", command]);
  }
  let batch = fixture.batch(&args);
  assert_success(&batch);
  let values = envelopes(&batch.stdout);
  assert_eq!(values, envelopes(&script.stdout));
  assert_eq!(values.len(), 8);
  assert!(
    values
      .iter()
      .all(|value| value["result_type"] != "RunResult")
  );
  assert_eq!(fs::read(fixture.0.join("results.csv")).unwrap(), saved);
}

#[test]
fn json_batch_empty_directive_and_exit_scripts_have_no_container_envelopes() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("empty.td"), "# no commands\n\n").unwrap();
  fs::write(
    fixture.0.join("directives.td"),
    "seed 7\nlet n = 1\nif false\nexport never.csv\nend\n",
  )
  .unwrap();
  fs::write(fixture.0.join("exit.td"), "count\nexit\nexport never.csv\n").unwrap();
  let output = fixture.batch(&[
    "--json",
    "-c",
    "run empty.td",
    "-c",
    "run directives.td",
    "-c",
    "use data.csv",
    "-c",
    "run exit.td",
    "-c",
    "count",
  ]);
  assert_success(&output);
  let values = envelopes(&output.stdout);
  assert_eq!(values.len(), 3);
  assert_eq!(values[0]["result_type"], "LoadResult");
  assert_eq!(values[1]["result_type"], "CountResult");
  assert_eq!(values[2], values[1]);
  assert!(!fixture.0.join("never.csv").exists());
  let empty = fixture.batch(&["--json", "-c", "run empty.td"]);
  assert_success(&empty);
  assert!(empty.stdout.is_empty());
}

#[test]
fn json_batch_run_errors_are_located_exit_one_and_stop_later_commands() {
  for (source, error_file, error_line, message) in [
    (
      "unknown_command",
      "analysis files/child script.td",
      3,
      "unknown command",
    ),
    (
      "assert x > 10",
      "analysis files/child script.td",
      3,
      "assert",
    ),
    (
      "collapse sum x, by(y)",
      "analysis files/child script.td",
      3,
      "successful script results",
    ),
    (
      "run 'missing child.td'",
      "analysis files/missing child.td",
      1,
      "script file not found",
    ),
    (
      "run './child script.td'",
      "analysis files/child script.td",
      1,
      "recursive script inclusion",
    ),
  ] {
    let fixture = Fixture::new();
    fs::write(
      fixture.0.join("analysis files/main script.td"),
      "run 'child script.td'\nexport parent-never.csv\n",
    )
    .unwrap();
    fs::write(
      fixture.0.join("analysis files/child script.td"),
      format!("count\nexport completed.csv\n{source}\nexport child-never.csv\n"),
    )
    .unwrap();
    let output = fixture.batch(&[
      "--json",
      "-c",
      "use data.csv",
      "-c",
      "run 'analysis files/main script.td'",
      "-c",
      "export batch-never.csv",
    ]);
    assert_eq!(
      output.status.code(),
      Some(1),
      "{source}: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let values = envelopes(&output.stdout);
    assert_eq!(values.len(), 4, "{source}");
    assert_eq!(values[2]["result_type"], "ExportResult");
    let error = &values[3]["error"];
    assert_eq!(error["type"], "ScriptError");
    assert_eq!(error["line"], error_line);
    assert_eq!(
      error["path"],
      fixture
        .0
        .canonicalize()
        .unwrap()
        .join(error_file)
        .to_string_lossy()
        .as_ref()
    );
    assert!(
      error["message"].as_str().unwrap().contains(message),
      "{error}"
    );
    assert_eq!(
      fs::read_to_string(fixture.0.join("completed.csv")).unwrap(),
      "x,y\n0,1\n1,2\n2,4\n3,6\n"
    );
    for name in ["parent-never.csv", "child-never.csv", "batch-never.csv"] {
      assert!(!fixture.0.join(name).exists(), "{source}: {name}");
    }
  }
}
