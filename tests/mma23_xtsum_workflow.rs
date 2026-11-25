mod common;
use common::{TestDir, tabdat_bin};
use serde_json::Value;
use std::fs;

const FIELDS: [&str; 21] = [
  "id", "year", "cusip", "ardssic", "scisect", "logk", "sumpat", "logr", "pat", "lpat", "dpat",
  "randd", "logrl1", "logrl2", "logrl3", "logrl4", "logrl5", "dyear2", "dyear3", "dyear4",
  "dyear5",
];

fn root() -> std::path::PathBuf {
  std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn log_r_and_d(id: i32, period: i32) -> f64 {
  0.01 * f64::from(id) + 0.05 * f64::from(period) + 0.001 * f64::from(id % 5)
}

fn balanced_xtsum_fixture() -> String {
  let mut csv = format!("{}\n", FIELDS.join(","));
  for id in 1..=346 {
    for period in 0..10 {
      let year = 1970 + period;
      let cusip = 100_000 + id;
      let ardssic = if id <= 336 {
        (1 + id % 21).to_string()
      } else {
        String::new()
      };
      let scisect = id % 2;
      let logk = (1.0 + 0.1 * f64::from(id)).ln();
      let sumpat = id * 2 + period;
      let logr = log_r_and_d(id, period);
      let pat = (id * period) % 12;
      let lpat = if pat == 0 {
        0.5_f64.ln()
      } else {
        f64::from(pat).ln()
      };
      let dpat = i32::from(pat > 0);
      let randd = logr.exp();
      let lags = (1..=5)
        .map(|lag| {
          if period >= lag {
            log_r_and_d(id, period - lag).to_string()
          } else {
            String::new()
          }
        })
        .collect::<Vec<_>>();
      let dummies = (6..=9)
        .map(|dummy_period| i32::from(period == dummy_period))
        .collect::<Vec<_>>();
      csv.push_str(&format!(
        "{id},{year},{cusip},{ardssic},{scisect},{logk},{sumpat},{logr},{pat},{lpat},{dpat},{randd},{},{},{},{},{},{},{},{},{}\n",
        lags[0], lags[1], lags[2], lags[3], lags[4], dummies[0], dummies[1], dummies[2], dummies[3]
      ));
    }
  }
  csv
}

fn run(d: &TestDir) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args(["--json", "-f"])
    .arg(root().join("examples/mma23p1pannonlin/xtsum.td"))
    .output()
    .unwrap()
}

fn run_text(d: &TestDir) -> std::process::Output {
  tabdat_bin()
    .current_dir(d.path())
    .args(["--text", "-f"])
    .arg(root().join("examples/mma23p1pannonlin/xtsum.td"))
    .output()
    .unwrap()
}

fn events(out: &std::process::Output) -> Vec<Value> {
  assert!(
    out.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&out.stdout),
    String::from_utf8_lossy(&out.stderr)
  );
  String::from_utf8_lossy(&out.stdout)
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

fn rows(events: &[Value]) -> &Vec<Value> {
  events
    .iter()
    .find(|event| event["result_type"] == "XtSumResult")
    .unwrap()["data"]["rows"]
    .as_array()
    .unwrap()
}

fn close(actual: &Value, expected: f64) {
  let actual = actual.as_f64().unwrap();
  assert!(
    (actual - expected).abs() <= 1e-10 + 1e-12 * expected.abs(),
    "{actual} != {expected}"
  );
}

#[test]
fn xtsum_reports_source_variable_samples_and_rejects_changed_patterns() {
  let csv = balanced_xtsum_fixture();
  let d = TestDir::new();
  fs::write(d.path().join("patents_decade_prepared.csv"), &csv).unwrap();
  let result = events(&run(&d));
  assert_eq!(
    result
      .iter()
      .map(|event| event["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    [
      "LoadResult",
      "IsidResult",
      "DescribeResult",
      "TableResult",
      "XtSumResult"
    ]
  );
  assert!(
    result
      .iter()
      .all(|event| event["schema_version"] == 1 && event.get("error").is_none())
  );
  let columns = result
    .iter()
    .find(|event| event["result_type"] == "DescribeResult")
    .unwrap()["data"]["dataset"]["columns"]
    .as_array()
    .unwrap();
  assert_eq!(
    columns
      .iter()
      .map(|column| column["name"].as_str().unwrap())
      .collect::<Vec<_>>(),
    FIELDS
  );
  assert_eq!(result[3]["data"]["rows"], serde_json::json!([[3460]]));
  let table = &result[4]["data"];
  assert_eq!(table["id_variable"], "id");
  let table_rows = rows(&result);
  assert_eq!(table_rows.len(), 21);
  assert_eq!(table_rows[0]["variable"], "id");
  assert_eq!(table_rows[3]["variable"], "ardssic");
  assert_eq!(table_rows[12]["variable"], "logrl1");
  assert_eq!(table_rows[16]["variable"], "logrl5");
  assert_eq!(table_rows[20]["variable"], "dyear5");
  assert_eq!(
    (
      table_rows[0]["overall_n"].as_i64(),
      table_rows[0]["between_n"].as_i64()
    ),
    (Some(3460), Some(346))
  );
  close(&table_rows[0]["overall_mean"], 173.5);
  close(&table_rows[0]["between_sd"], 100.025_829_997_389_51);
  close(&table_rows[0]["within_sd"], 0.0);
  assert_eq!(
    (
      table_rows[3]["overall_n"].as_i64(),
      table_rows[3]["between_n"].as_i64()
    ),
    (Some(3360), Some(336))
  );
  close(&table_rows[1]["overall_mean"], 1974.5);
  close(&table_rows[1]["within_min"], 1970.0);
  close(&table_rows[1]["within_max"], 1979.0);
  close(&table_rows[3]["mean_t"], 10.0);
  assert_eq!(
    (
      table_rows[12]["overall_n"].as_i64(),
      table_rows[12]["between_n"].as_i64()
    ),
    (Some(3114), Some(346))
  );
  close(&table_rows[12]["mean_t"], 9.0);
  assert_eq!(
    (
      table_rows[16]["overall_n"].as_i64(),
      table_rows[16]["between_n"].as_i64()
    ),
    (Some(1730), Some(346))
  );
  close(&table_rows[16]["mean_t"], 5.0);
  assert_eq!(
    fs::read_to_string(d.path().join("patents_decade_prepared.csv")).unwrap(),
    csv
  );
  let text = run_text(&d);
  assert!(
    text.status.success(),
    "{}\n{}",
    String::from_utf8_lossy(&text.stdout),
    String::from_utf8_lossy(&text.stderr)
  );
  let text = String::from_utf8_lossy(&text.stdout);
  assert!(text.contains("Panel identifier: id"));
  assert!(text.contains("Variable\tComponent\tN/n/T\tMean\tStd. dev.\tMin\tMax"));
  assert!(text.contains("overall\tN=3460"));
  assert!(text.contains("between\tn=346"));
  assert!(text.contains("within\tT-bar=10"));

  let cases = [
    (1, 3, ""),        // ARDSSIC is missing within a contributing firm.
    (10, 12, ""),      // LOGRL1 has an unexpected within-firm missing value.
    (1, 7, "NaN"),     // A finite source field is nonfinite.
    (3460, 1, "1980"), // A year outside the original full-decade cohort.
  ];
  for (row, field, value) in cases {
    let mut lines: Vec<String> = csv.lines().map(str::to_owned).collect();
    let mut cells: Vec<String> = lines[row].split(',').map(str::to_owned).collect();
    cells[field] = value.to_owned();
    lines[row] = cells.join(",");
    let changed = format!("{}\n", lines.join("\n"));
    let d = TestDir::new();
    fs::write(d.path().join("patents_decade_prepared.csv"), &changed).unwrap();
    let out = run(&d);
    assert!(
      !out.status.success(),
      "accepted changed row {row} field {field}"
    );
    assert_eq!(
      fs::read_to_string(d.path().join("patents_decade_prepared.csv")).unwrap(),
      changed
    );
    assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1);
  }

  let d = TestDir::new();
  let mut lines: Vec<String> = csv.lines().map(str::to_owned).collect();
  lines[2] = lines[1].clone();
  let duplicate = format!("{}\n", lines.join("\n"));
  fs::write(d.path().join("patents_decade_prepared.csv"), &duplicate).unwrap();
  assert!(
    !run(&d).status.success(),
    "accepted duplicate firm/year key"
  );
  assert_eq!(
    fs::read_to_string(d.path().join("patents_decade_prepared.csv")).unwrap(),
    duplicate
  );
  assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1);
}
