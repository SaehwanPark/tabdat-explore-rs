use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const SAMPLE: &str =
  "source_row,mode,pcharter,ppier\n5,4,4,2\n1,1,9,3\n2,2,2,1\n6,4,16,4\n3,3,9,3\n4,2,8,2\n";
struct Fixture(PathBuf);
impl Fixture {
  fn new(csv: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-mma-summary-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("Nldata.csv"), csv).unwrap();
    Self(path)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma14p1binary/summary.td")
  }
  fn run(&self, flag: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .arg(flag)
      .arg("-f")
      .arg(Self::script())
      .output()
      .unwrap()
  }
  fn sample(&self) -> PathBuf {
    self.0.join("fishing_summary_sample.csv")
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
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}
fn close(actual: &Value, expected: f64) {
  assert!((actual.as_f64().unwrap() - expected).abs() < 1e-12);
}

#[test]
fn actual_example_reports_ordered_cohorts_sample_sd_and_original_source_identity() {
  let fixture = Fixture::new(SAMPLE);
  let output = fixture.run("--json");
  assert!(output.status.success(), "{:?}", output);
  let rows = envelopes(&output.stdout);
  assert_eq!(rows.len(), 7);
  assert!(rows.iter().all(|r| r["schema_version"] == 1));
  assert_eq!(rows[1]["data"]["dataset"]["row_count"], 4);
  assert_eq!(rows[2]["result_type"], "IsidResult");
  for (i, n) in [4, 2, 2].into_iter().enumerate() {
    assert_eq!(rows[3 + i]["result_type"], "SummarizeResult");
    let summary = rows[3 + i]["data"]["rows"].as_array().unwrap();
    assert_eq!(
      summary
        .iter()
        .map(|r| r["variable"].as_str().unwrap())
        .collect::<Vec<_>>(),
      ["charter", "pcharter", "ppier", "lnrelp"]
    );
    assert!(summary.iter().all(|r| r["count"] == n));
    close(&summary[0]["mean"], [0.5, 0.0, 1.0][i]);
    close(
      &summary[0]["std_dev"],
      [(1.0_f64 / 3.0).sqrt(), 0.0, 0.0][i],
    );
    close(&summary[1]["mean"], [7.5, 5.0, 10.0][i]);
    close(
      &summary[1]["std_dev"],
      [(115.0_f64 / 3.0).sqrt(), 18.0_f64.sqrt(), 72.0_f64.sqrt()][i],
    );
    close(&summary[3]["mean"], 1.5 * 2.0_f64.ln());
    close(&summary[3]["minimum"], 2.0_f64.ln());
    close(&summary[3]["maximum"], 4.0_f64.ln());
  }
  let csv = fs::read_to_string(fixture.sample()).unwrap();
  assert_eq!(
    csv
      .lines()
      .skip(1)
      .map(|l| l.split(',').next().unwrap())
      .collect::<Vec<_>>(),
    ["2", "4", "5", "6"]
  );
  assert_eq!(
    fs::read_to_string(fixture.0.join("Nldata.csv")).unwrap(),
    SAMPLE
  );
}

#[test]
fn text_and_piped_example_keep_the_same_prepared_sample() {
  let json = Fixture::new(SAMPLE);
  assert!(json.run("--json").status.success());
  let text = Fixture::new(SAMPLE);
  let output = text.run("--text");
  assert!(output.status.success(), "{:?}", output);
  let text_output = String::from_utf8(output.stdout).unwrap();
  assert!(text_output.contains("charter"));
  assert!(text_output.contains("pcharter"));
  assert!(text_output.contains("lnrelp"));
  let piped = Fixture::new(SAMPLE);
  let mut child = Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
    .current_dir(&piped.0)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .unwrap();
  child
    .stdin
    .take()
    .unwrap()
    .write_all(format!("run \"{}\"\nexit\n", Fixture::script().display()).as_bytes())
    .unwrap();
  let output = child.wait_with_output().unwrap();
  assert!(output.status.success(), "{:?}", output);
  assert!(String::from_utf8_lossy(&output.stdout).contains("pcharter"));
  let bytes = fs::read(json.sample()).unwrap();
  assert_eq!(fs::read(text.sample()).unwrap(), bytes);
  assert_eq!(fs::read(piped.sample()).unwrap(), bytes);
}

#[test]
fn export_collision_preserves_existing_bytes_and_reports_located_failure() {
  let fixture = Fixture::new(SAMPLE);
  fs::write(fixture.sample(), "original bytes").unwrap();
  let output = fixture.run("--json");
  assert_eq!(output.status.code(), Some(1));
  assert!(String::from_utf8_lossy(&output.stderr).contains("summary.td:9:"));
  assert_eq!(
    fs::read_to_string(fixture.sample()).unwrap(),
    "original bytes"
  );
  let rows = envelopes(&output.stdout);
  assert_eq!(rows.last().unwrap()["error"]["type"], "ScriptError");
  assert_eq!(
    rows
      .iter()
      .filter(|r| r["result_type"] == "SummarizeResult")
      .count(),
    3
  );
}

#[test]
fn absent_columns_invalid_prices_and_duplicate_ids_fail_before_export() {
  for csv in [
    SAMPLE.replace("pcharter", "other_price"),
    SAMPLE.replace("2,2,2,1", "2,2,-2,1"),
    SAMPLE.replace("4,2,8,2", "2,2,8,2"),
  ] {
    let fixture = Fixture::new(&csv);
    let output = fixture.run("--json");
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    assert!(!fixture.sample().exists());
    let rows = envelopes(&output.stdout);
    assert_eq!(rows.last().unwrap()["error"]["type"], "ScriptError");
    assert!(rows.iter().all(|r| r["result_type"] != "SummarizeResult"));
    assert_eq!(
      fs::read_to_string(fixture.0.join("Nldata.csv")).unwrap(),
      csv
    );
  }
}
