use serde_json::Value;
use std::{
  fs,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const ARTIFACTS: [&str; 4] = [
  "fishing_source_sample.csv",
  "fishing_prepared_sample.csv",
  "fishing_comparison_predictions.csv",
  "fishing_binary_comparison.svg",
];

fn sample() -> String {
  let mut csv = String::from(
    "source_row,mode,price,crate,dbeach,dpier,dprivate,dcharter,pbeach,ppier,pprivate,pcharter,qbeach,qpier,qprivate,qcharter,income\n",
  );
  // Shuffled original IDs; overlapping outcomes prevent perfect separation.
  for (id, mode, x) in [
    (10, 4, 2.5_f64),
    (2, 4, -1.0),
    (9, 2, -0.5),
    (1, 2, -2.0),
    (7, 4, 0.5),
    (8, 2, 1.5),
    (3, 2, 0.0),
    (6, 4, 3.0),
    (5, 2, 2.0),
    (4, 4, 1.0),
    (11, 1, 0.2),
    (12, 3, -0.2),
  ] {
    let pcharter = 2.0 * x.exp();
    let price = if mode == 4 { pcharter } else { 2.0 };
    csv.push_str(&format!(
      "{id},{mode},{price:.17},0.25,{},{},{},{},2,2,3,{pcharter:.17},0.1,0.2,0.3,0.4,{}\n",
      u8::from(mode == 1),
      u8::from(mode == 2),
      u8::from(mode == 3),
      u8::from(mode == 4),
      1000.0 * (f64::from(id) + 0.5)
    ));
  }
  csv
}

struct Fixture(PathBuf);
impl Fixture {
  fn new(csv: &str) -> Self {
    let path = std::env::temp_dir().join(format!(
      "tabdat-mma-analysis-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    fs::write(path.join("Nldata.csv"), csv).unwrap();
    Self(path)
  }
  fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/mma14p1binary/analysis.td")
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
  fn rows(&self, artifact: usize) -> Vec<Vec<f64>> {
    fs::read_to_string(self.0.join(ARTIFACTS[artifact]))
      .unwrap()
      .lines()
      .skip(1)
      .map(|line| line.split(',').map(|s| s.parse().unwrap()).collect())
      .collect()
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
fn close(actual: f64, expected: f64) {
  assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn actual_composed_example_covers_all_blocks_transforms_fits_and_artifacts() {
  let csv = sample();
  let fixture = Fixture::new(&csv);
  let output = fixture.run("--json");
  assert!(output.status.success(), "{:?}", output);
  let results = envelopes(&output.stdout);
  assert!(results.iter().all(|r| r["schema_version"] == 1));
  let summaries: Vec<_> = results
    .iter()
    .filter(|r| r["result_type"] == "SummarizeResult")
    .map(|r| r["data"]["rows"].as_array().unwrap())
    .collect();
  assert_eq!(summaries.len(), 8);
  for (rows, (width, n)) in summaries.iter().zip([
    (17, 12),
    (20, 10),
    (20, 5),
    (20, 5),
    (4, 10),
    (4, 5),
    (4, 5),
    (4, 10),
  ]) {
    assert_eq!(rows.len(), width);
    assert!(rows.iter().all(|r| r["count"] == n));
  }
  assert_eq!(summaries[0][16]["variable"], "ydiv1000");
  close(summaries[0][16]["mean"].as_f64().unwrap(), 7.0);
  close(
    summaries[0][16]["std_dev"].as_f64().unwrap(),
    13.0_f64.sqrt(),
  );
  for (block, mean) in [(5, 0.0), (6, 1.0)] {
    close(summaries[block][0]["mean"].as_f64().unwrap(), mean);
    close(summaries[block][0]["std_dev"].as_f64().unwrap(), 0.0);
  }
  let table = results
    .iter()
    .find(|r| r["result_type"] == "EstimateTableResult")
    .unwrap();
  let models = table["data"]["models"].as_array().unwrap();
  assert_eq!(
    models
      .iter()
      .map(|m| m["name"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "blogit", "bprobit", "bOLS", "bloghet", "bprobhet", "bOLShet"
    ]
  );
  assert!(models.iter().all(|m| m["observation_count"] == 10));
  assert_eq!(models[2]["inference_degrees_of_freedom"], 8);
  assert_eq!(
    results
      .iter()
      .filter(|r| r["result_type"] == "BinaryLogitResult")
      .count(),
    2
  );
  assert_eq!(
    results
      .iter()
      .filter(|r| r["result_type"] == "BinaryProbitResult")
      .count(),
    2
  );
  assert_eq!(
    results
      .iter()
      .filter(|r| r["result_type"] == "RegressionResult")
      .count(),
    2
  );
  let source = fixture.rows(0);
  assert_eq!(
    source.iter().map(|r| r[0] as u32).collect::<Vec<_>>(),
    [10, 2, 9, 1, 7, 8, 3, 6, 5, 4, 11, 12]
  );
  let prepared = fixture.rows(1);
  let predictions = fixture.rows(2);
  for (i, (row, prediction)) in prepared.iter().zip(&predictions).enumerate() {
    close(row[0], (i + 1) as f64);
    close(prediction[0], row[0]);
    close(row[17], row[16] / 1000.0);
    close(row[18], f64::from(row[1] == 4.0));
    close(row[19], 100.0 * (row[11] / row[9]).ln());
    close(row[20], (row[11] / row[9]).ln());
    close(prediction[2], row[18]);
    close(prediction[5], row[20]);
  }
  let plot = results
    .iter()
    .find(|r| r["result_type"] == "PlotArtifactResult")
    .unwrap();
  assert_eq!(plot["data"]["candidate_rows"], 10);
  assert_eq!(plot["data"]["scatter"]["point_count"], 10);
  assert!(
    plot["data"]["lines"]
      .as_array()
      .unwrap()
      .iter()
      .all(|l| l["point_count"] == 10)
  );
  assert_eq!(
    fs::read_to_string(fixture.0.join("Nldata.csv")).unwrap(),
    csv
  );
}

#[test]
fn json_text_and_piped_nested_scripts_publish_identical_csv_and_svg() {
  let csv = sample();
  let json = Fixture::new(&csv);
  assert!(json.run("--json").status.success());
  let text = Fixture::new(&csv);
  let output = text.run("--text");
  assert!(output.status.success(), "{:?}", output);
  let rendered = String::from_utf8(output.stdout).unwrap();
  for phrase in [
    "ydiv1000",
    "pratio",
    "ols_classical_xb",
    "Estimate\tblogit\tbprobit\tbOLS\tbloghet\tbprobhet\tbOLShet",
    "student_t(8)",
  ] {
    assert!(rendered.contains(phrase), "{phrase}");
  }
  let piped = Fixture::new(&csv);
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
  assert!(String::from_utf8_lossy(&output.stdout).contains("Estimate\tblogit\tbprobit\tbOLS"));
  for artifact in ARTIFACTS {
    let bytes = fs::read(json.0.join(artifact)).unwrap();
    assert_eq!(fs::read(text.0.join(artifact)).unwrap(), bytes);
    assert_eq!(fs::read(piped.0.join(artifact)).unwrap(), bytes);
  }
}

#[test]
fn first_later_nested_and_plot_collisions_preserve_bytes_and_stop_after_prior_effects() {
  let csv = sample();
  for (index, artifact) in ARTIFACTS.iter().enumerate() {
    let fixture = Fixture::new(&csv);
    fs::write(fixture.0.join(artifact), "existing bytes").unwrap();
    let output = fixture.run("--json");
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    assert_eq!(
      fs::read_to_string(fixture.0.join(artifact)).unwrap(),
      "existing bytes"
    );
    assert_eq!(
      fs::read_to_string(fixture.0.join("Nldata.csv")).unwrap(),
      csv
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let origin = if index == 2 {
      "stored_comparison.td:29:"
    } else {
      "analysis.td:"
    };
    assert!(stderr.contains(origin), "{stderr}");
    let results = envelopes(&output.stdout);
    assert_eq!(results.last().unwrap()["error"]["type"], "ScriptError");
    assert!(
      !results
        .iter()
        .any(|r| r["result_type"] == "PlotArtifactResult")
    );
    for (later, file) in ARTIFACTS.iter().enumerate() {
      assert_eq!(fixture.0.join(file).exists(), later <= index, "{file}");
    }
  }
}

#[test]
fn bad_source_schema_prices_and_keys_fail_without_fits_or_later_artifacts() {
  let csv = sample();
  let mut negative: Vec<_> = csv.lines().map(str::to_owned).collect();
  let mut fields: Vec<_> = negative[1].split(',').map(str::to_owned).collect();
  fields[11] = "-1".into();
  negative[1] = fields.join(",");
  for (bad, prior_source_export) in [
    (csv.replace("pcharter", "other_price"), false),
    (csv.replace("2,4,", "10,4,"), false),
    (negative.join("\n"), true),
  ] {
    let fixture = Fixture::new(&bad);
    let output = fixture.run("--json");
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    let results = envelopes(&output.stdout);
    assert_eq!(results.last().unwrap()["error"]["type"], "ScriptError");
    assert!(!results.iter().any(|r| matches!(
      r["result_type"].as_str(),
      Some("RegressionResult" | "BinaryLogitResult" | "BinaryProbitResult" | "EstimateStoreResult")
    )));
    assert_eq!(fixture.0.join(ARTIFACTS[0]).exists(), prior_source_export);
    assert!(ARTIFACTS[1..].iter().all(|p| !fixture.0.join(p).exists()));
    assert_eq!(
      fs::read_to_string(fixture.0.join("Nldata.csv")).unwrap(),
      bad
    );
  }
}
