#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Output, Stdio},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-interactive-{}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("data.csv"), "x,y\n1,2\n3,\n").unwrap();
    Self(path)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

fn run(fixture: &Fixture, input: &[u8]) -> Output {
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&fixture.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child.stdin.take().unwrap().write_all(input).unwrap();
  child.wait_with_output().unwrap()
}

#[test]
fn persistent_eda_session_reports_schema_statistics_and_previews() {
  let fixture = Fixture::new();
  let output = run(&fixture, b"# analyst session\n\nuse data.csv\ndescribe\nsummarize y\nhead 2\ncount\nexit\nnot_a_command\n");
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  assert!(output.stderr.is_empty());
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.contains("Loaded 2 rows, 2 columns from data.csv\n"));
  assert!(stdout.contains("Variable\tType\nx\tBIGINT\ny\tBIGINT\n"));
  assert!(stdout.contains("Variable\tN\tMean\tStd. dev.\tMin\tMax\ny\t1\t2\tNA\t2\t2\n"));
  assert!(stdout.contains("x\ty\n1\t2\n3\tNA\n(2 rows shown)\n"));
  assert!(stdout.ends_with("2 rows\n"));
  assert!(!stdout.contains("tabdat>"));
  assert!(!stdout.contains("Hello, world!"));
}

#[test]
fn errors_recover_and_failed_load_preserves_the_previous_dataset() {
  let fixture = Fixture::new();
  let output = run(
    &fixture,
    b"count\nnot_a_command\nuse data.csv\nuse missing.csv\ncount\nquit\n",
  );
  assert!(output.status.success());
  let stderr = String::from_utf8(output.stderr).unwrap();
  assert_eq!(stderr.matches("Error:").count(), 3);
  assert!(stderr.contains("missing.csv"));
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.ends_with("2 rows\n"));
}

#[test]
fn frequency_tables_preserve_numeric_order_missingness_and_percent_headers() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("categories.csv"),
    "cat,grp\n10,A\n2,B\n-1,A\n,\n",
  )
  .unwrap();
  let output = run(&fixture, b"use categories.csv\ntabulate cat\ntabulate cat, missing\ntabulate cat grp, missing row col\nexit\n");
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8(output.stdout).unwrap();
  let tables: Vec<_> = stdout.split("cat\tCount\tPercent\n").collect();
  assert_eq!(tables.len(), 3);
  let without_missing = tables[1];
  assert!(!without_missing.contains("NA\t"));
  assert!(without_missing.find("-1\t1\t").unwrap() < without_missing.find("2\t1\t").unwrap());
  assert!(without_missing.find("2\t1\t").unwrap() < without_missing.find("10\t1\t").unwrap());
  assert!(tables[2].starts_with("-1\t1\t25\n2\t1\t25\n10\t1\t25\nNA\t1\t25\n(4 rows shown)\n"));
  assert!(stdout.contains("cat\tA Count\tA Row %\tA Col %\tB Count\tB Row %\tB Col %\tmissing Count\tmissing Row %\tmissing Col %\n"));
  assert!(stdout.contains("-1\t1\t100\t50\t0\t0\t0\t0\t0\t0\n"));
}

#[test]
fn transformed_data_reports_outcomes_and_roundtrips_through_save_and_export() {
  let fixture = Fixture::new();
  let output = run(&fixture, b"use data.csv\ngenerate z = x * 2\nreplace z = z + 1\nrename z result\ngsort -x\nsort x\nkeep x y result\ndrop x\nselect result y\nsave result.parquet\nexport result.csv\nuse result.parquet\nhead 2\nuse result.csv\nhead 2\nexit\n");
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8(output.stdout).unwrap();
  for message in [
    "Generated: 2 rows, 3 columns",
    "Replaced: 2 rows, 3 columns",
    "Renamed: 2 rows, 3 columns",
    "Sorted: 2 rows, 3 columns",
    "Kept: 2 rows, 3 columns",
    "Dropped: 2 rows, 2 columns",
    "Selected: 2 rows, 2 columns",
    "Saved 2 rows, 2 columns to result.parquet",
    "Exported 2 rows, 2 columns to result.csv",
  ] {
    assert!(stdout.contains(message), "missing {message}: {stdout}");
  }
  assert_eq!(
    stdout
      .matches("result\ty\n3\t2\n7\tNA\n(2 rows shown)\n")
      .count(),
    2
  );
  assert!(fixture.0.join("result.parquet").is_file());
  assert_eq!(
    fs::read_to_string(fixture.0.join("result.csv")).unwrap(),
    "result,y\n3,2\n7,\n"
  );
  assert!(!stdout.contains("terminal details"));
}

#[test]
fn failed_transformations_and_refused_overwrite_do_not_report_success_or_lose_data() {
  let fixture = Fixture::new();
  let path = fixture.0.join("existing.csv");
  fs::write(&path, "sentinel\n").unwrap();
  let output = run(&fixture, b"use data.csv\nreplace absent = x\nkeep absent\nexport existing.csv\nsave invalid.csv\nhead 2\ncount\nexit\n");
  assert!(output.status.success());
  let stderr = String::from_utf8(output.stderr).unwrap();
  assert_eq!(stderr.matches("Error:").count(), 4, "{stderr}");
  let stdout = String::from_utf8(output.stdout).unwrap();
  for success in ["Replaced:", "Kept:", "Exported", "Saved"] {
    assert!(!stdout.contains(success));
  }
  assert!(stdout.contains("x\ty\n1\t2\n3\tNA\n(2 rows shown)\n"));
  assert!(stdout.ends_with("2 rows\n"));
  assert_eq!(fs::read_to_string(path).unwrap(), "sentinel\n");
  assert!(!fixture.0.join("invalid.csv").exists());
}

fn close_to_reference(actual: &str, expected: &serde_json::Value) {
  let actual: f64 = actual.parse().unwrap();
  let expected = expected.as_f64().unwrap();
  let tolerance = 1e-10_f64.max(1e-9 * expected.abs());
  assert!(
    (actual - expected).abs() <= tolerance,
    "{actual} vs {expected} (tolerance {tolerance})"
  );
}

fn model_fixture() -> Fixture {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("model.csv"),
    "x,y\n0,1\n1,2\n2,1.5\n3,4\n4,3\n5,5\n6,\n,8\n",
  )
  .unwrap();
  fixture
}

#[test]
fn linear_workflow_display_matches_independent_statsmodels_scipy_reference() {
  // Checked-in independent values generated by tools/capture_linear_terminal_reference.py.
  // Tolerance allows QR/library floating-point differences, not display rounding.
  let reference: serde_json::Value =
    serde_json::from_str(include_str!("fixtures/linear-terminal-reference.json")).unwrap();
  let fixture = model_fixture();
  let output = run(
    &fixture,
    b"use model.csv\nregress y x\nlincom intercept + x\ntest x\npredict fitted\nhead 8\nexit\n",
  );
  assert!(output.status.success());
  assert!(
    output.stderr.is_empty(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert!(stdout.contains("Estimator: ols; covariance: nonrobust; intercept: true\nN = 6; residual df = 4\nCandidate rows = 8; excluded = 2\n"));
  let fit = stdout
    .lines()
    .find(|line| line.starts_with("R-squared = "))
    .unwrap();
  for (part, key) in fit
    .split("; ")
    .zip(["r_squared", "adjusted_r_squared", "root_mse"])
  {
    close_to_reference(part.split(" = ").nth(1).unwrap(), &reference[key]);
  }
  let coefficients = stdout
    .split("Parameter\tCoefficient\tStd. error\tStatistic\tP(two-sided)\n")
    .nth(1)
    .unwrap();
  for (index, line) in coefficients.lines().take(2).enumerate() {
    let fields: Vec<_> = line.split('\t').collect();
    assert_eq!(
      fields[0],
      reference["coefficient_names"][index].as_str().unwrap()
    );
    for (field, key) in
      fields[1..]
        .iter()
        .zip(["coefficients", "standard_errors", "statistics", "p_values"])
    {
      close_to_reference(field, &reference[key][index]);
    }
  }
  let lincom = stdout
    .split("CI upper (95%)\n")
    .nth(1)
    .unwrap()
    .lines()
    .next()
    .unwrap();
  let fields: Vec<_> = lincom.split('\t').collect();
  assert_eq!(fields.len(), 6);
  for (actual, expected) in fields.iter().zip(reference["lincom"].as_array().unwrap()) {
    close_to_reference(actual, expected);
  }
  let test = stdout
    .lines()
    .find(|line| line.starts_with("F(1, 4) = "))
    .unwrap();
  let fields: Vec<_> = test.split("; ").collect();
  for (part, expected) in fields.iter().zip(reference["test"].as_array().unwrap()) {
    close_to_reference(part.split(" = ").nth(1).unwrap(), expected);
  }
  assert!(stdout.contains("Predicted: 8 rows, 3 columns\n"));
  let preview = stdout.split("x\ty\tfitted\n").nth(1).unwrap();
  for (index, row) in preview.lines().take(8).enumerate() {
    let value = row.split('\t').nth(2).unwrap();
    if reference["predictions"][index].is_null() {
      assert_eq!(value, "NA");
    } else {
      close_to_reference(value, &reference["predictions"][index]);
    }
  }
  assert!(!stdout.contains("terminal details"));

  // Inspect row identity as well as the displayed N: the two missing rows differ.
  let mut session = tabdat_execution::Session::new();
  session
    .execute(
      tabdat_language::parse_command(&format!("use {}", fixture.0.join("model.csv").display()))
        .unwrap(),
    )
    .unwrap();
  session
    .execute(tabdat_language::parse_command("regress y x").unwrap())
    .unwrap();
  let sample = session.last_regression().unwrap().sample.as_ref().unwrap();
  let indices: Vec<usize> = reference["retained_indices"]
    .as_array()
    .unwrap()
    .iter()
    .map(|index| index.as_u64().unwrap() as usize)
    .collect();
  assert_eq!(sample.retained_indices, indices);
}

#[test]
fn failed_estimation_clears_model_but_preserves_data_and_allows_recovery() {
  let fixture = model_fixture();
  let output = run(&fixture, b"use model.csv\nregress y x\nregress y absent\npredict stale\nlincom x\ntest x\ncount\nregress y x\nlincom absent\npredict recovered\ncount\nexit\n");
  assert!(output.status.success());
  let stderr = String::from_utf8(output.stderr).unwrap();
  assert_eq!(stderr.matches("Error:").count(), 5, "{stderr}");
  let stdout = String::from_utf8(output.stdout).unwrap();
  assert_eq!(stdout.matches("Regression: y on x\n").count(), 2);
  assert_eq!(stdout.matches("Predicted: 8 rows, 3 columns\n").count(), 1);
  assert!(stdout.ends_with("8 rows\n"));
}

#[test]
fn eof_executes_the_final_unterminated_line_and_io_errors_are_fatal() {
  let fixture = Fixture::new();
  let output = run(&fixture, b"use data.csv\ncount");
  assert!(output.status.success());
  assert!(
    String::from_utf8(output.stdout)
      .unwrap()
      .ends_with("2 rows\n")
  );
  let output = run(&fixture, &[0xff, b'\n']);
  assert_eq!(output.status.code(), Some(1));
  assert!(
    String::from_utf8(output.stderr)
      .unwrap()
      .contains("interactive I/O failed")
  );
}
