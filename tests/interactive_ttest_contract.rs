#![forbid(unsafe_code)]

use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!(
      "tabdat-terminal-ttest-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "x1,x2,grp\n1,10,group1\n2,12,group1\n3,15,group2\n4,18,group2\n5,20,group1\n6,25,group2\n,9,group1\n8,,\n").unwrap();
    Self(root)
  }
  fn run(&self, input: &str) -> Output {
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
fn close(actual: &str, expected: &Value) {
  if expected.is_null() {
    assert_eq!(actual, "NA");
    return;
  }
  let actual: f64 = actual.parse().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected}, tolerance {tolerance}"
  );
}

#[test]
fn four_ttest_modes_match_independent_scipy_display_reference_and_nested_stream() {
  let fixture = Fixture::new();
  let commands = "ttest x1 == 3\nttest x1 == x2\nttest x1, by(grp)\nttest x1, by(grp) welch\n";
  let output = fixture.run(&format!("use data.csv\n{commands}exit\n"));
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(!stdout.contains("terminal details"));
  let reference: Value =
    serde_json::from_str(include_str!("fixtures/ttest-terminal-reference.json")).unwrap();
  let displays: Vec<_> = stdout.split("T test: ").skip(1).collect();
  assert_eq!(displays.len(), 4);
  for (display, case) in displays.iter().zip(reference["cases"].as_array().unwrap()) {
    let lines: Vec<_> = display.lines().collect();
    assert_eq!(lines[0], case["mode"].as_str().unwrap());
    let header = lines
      .iter()
      .position(|line| line.starts_with("Sample\tN\tMean"))
      .unwrap();
    assert_eq!(
      lines[header],
      "Sample\tN\tMean\tStd. error\tStd. dev.\tCI lower (95%)\tCI upper (95%)"
    );
    for (index, row) in case["rows"].as_array().unwrap().iter().enumerate() {
      let fields: Vec<_> = lines[header + index + 1].split('\t').collect();
      assert_eq!(fields.len(), 7);
      assert_eq!(fields[0], row["name"].as_str().unwrap());
      if row["n"].is_null() {
        assert_eq!(fields[1], "NA");
      } else {
        assert_eq!(fields[1], row["n"].as_u64().unwrap().to_string());
      }
      for (actual, expected) in fields[2..].iter().zip(row["values"].as_array().unwrap()) {
        close(actual, expected);
      }
    }
    let inference = lines
      .iter()
      .find(|line| line.starts_with("t reference;"))
      .unwrap();
    let fields: Vec<_> = inference.split("; ").collect();
    close(fields[1].split(" = ").nth(1).unwrap(), &case["df"]);
    close(fields[2].split(" = ").nth(1).unwrap(), &case["t"]);
    let probabilities = lines
      .iter()
      .find(|line| line.starts_with("P(left-tail)"))
      .unwrap();
    for (actual, expected) in probabilities
      .split("; ")
      .zip(case["probabilities"].as_array().unwrap())
    {
      close(actual.split(" = ").nth(1).unwrap(), expected);
    }
  }
  assert!(displays[0].contains("Null mean = 3\n"));
  assert!(displays[1].contains("Contrast: x1 - x2\nNull difference = 0\n"));
  for display in &displays[2..] {
    assert!(display.contains("Contrast: group1 - group2\n"));
    assert!(display.contains("Difference N is not an observation count"));
  }
  // Asymmetric missingness distinguishes one-sample (7), paired (6), and
  // independent (3+3) samples; compare all summaries, not just these counts.
  assert_eq!(
    reference["cases"][0]["retained_indices"],
    serde_json::json!([0, 1, 2, 3, 4, 5, 7])
  );
  assert_eq!(
    reference["cases"][1]["retained_indices"],
    serde_json::json!([0, 1, 2, 3, 4, 5])
  );
  assert_eq!(
    reference["cases"][2]["retained_indices"],
    serde_json::json!([[0, 1, 4], [2, 3, 5]])
  );
  fs::write(fixture.0.join("child.td"), commands).unwrap();
  fs::write(fixture.0.join("parent.td"), "use data.csv\nrun child.td\n").unwrap();
  let nested = fixture.run("run parent.td\nexit\n");
  assert!(nested.status.success());
  assert!(nested.stderr.is_empty());
  let nested_stdout = String::from_utf8(nested.stdout).unwrap();
  let summary = format!(
    "Executed 5 commands from {}\n",
    fixture
      .0
      .join("parent.td")
      .canonicalize()
      .unwrap()
      .display()
  );
  assert_eq!(nested_stdout.strip_suffix(&summary), Some(stdout.as_str()));
}

#[test]
fn ttest_failures_recover_and_all_forms_preserve_data_and_regression() {
  let fixture = Fixture::new();
  let output = fixture.run("use data.csv\nregress x2 x1\nttest x1, by(absent)\nttest x1 == 3\npredict fitted\ncount\nexit\n");
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8_lossy(&output.stderr)
      .matches("Error:")
      .count(),
    1
  );
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.contains("T test: one-sample\n"));
  assert!(stdout.contains("Predicted: 8 rows, 4 columns\n"));
  assert!(stdout.ends_with("8 rows\n"));
  let mut session = tabdat_execution::Session::new();
  let execute = |session: &mut tabdat_execution::Session, source: &str| {
    session.execute(tabdat_language::parse_command(source).unwrap())
  };
  execute(
    &mut session,
    &format!("use {}", fixture.0.join("data.csv").display()),
  )
  .unwrap();
  execute(&mut session, "regress x2 x1").unwrap();
  let model = session.last_regression().cloned();
  let data = session.active_dataset().cloned();
  let rows = execute(&mut session, "head 8").unwrap();
  for source in [
    "ttest x1 == 3",
    "ttest x1 == x2",
    "ttest x1, by(grp)",
    "ttest x1, by(grp) welch",
  ] {
    let result = execute(&mut session, source).unwrap();
    assert!(tabdat_reporting::terminal::render_execution_result(&result).starts_with("T test:"));
    assert_eq!(session.last_regression(), model.as_ref());
    assert_eq!(session.active_dataset(), data.as_ref());
    assert_eq!(execute(&mut session, "head 8").unwrap(), rows);
  }
  assert!(execute(&mut session, "ttest x1, by(absent)").is_err());
  assert_eq!(session.last_regression(), model.as_ref());
  assert_eq!(session.active_dataset(), data.as_ref());
  assert_eq!(execute(&mut session, "head 8").unwrap(), rows);
}

#[test]
fn undefined_ttest_inference_is_readable_and_shell_remains_usable() {
  let fixture = Fixture::new();
  fs::write(fixture.0.join("constant.csv"), "x,y,grp\n2,1,A\n2,1,B\n").unwrap();
  let output = fixture.run("use constant.csv\nttest x == 2\nttest x == y\nttest x, by(grp)\nttest x, by(grp) welch\ncount\nexit\n");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_eq!(stdout.matches("T test:").count(), 4);
  assert_eq!(stdout.matches("; t = NA\n").count(), 4);
  assert_eq!(
    stdout
      .matches("P(left-tail) = NA; P(two-sided) = NA; P(right-tail) = NA\n")
      .count(),
    4
  );
  assert!(stdout.contains("x\t2\t2\t0\t0\t2\t2\n"));
  assert!(!stdout.to_lowercase().contains("nan"));
  assert!(stdout.ends_with("2 rows\n"));
}
