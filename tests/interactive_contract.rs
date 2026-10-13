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
