#![forbid(unsafe_code)]

use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
use std::{
  fs,
  path::PathBuf,
  process::{Command, Output},
  time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let nonce = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "tabdat-json-tabulate-{}-{nonce}-{}",
      std::process::id(),
      NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    fs::write(root.join("data.csv"), "cat,grp\n10,A\n2,B\n-1,A\n,\n").unwrap();
    Self(root)
  }
  fn run(&self, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tabdat-explore-rs"))
      .current_dir(&self.0)
      .args(args)
      .output()
      .unwrap()
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn lines(output: &[u8]) -> Vec<Value> {
  std::str::from_utf8(output)
    .unwrap()
    .lines()
    .map(|line| serde_json::from_str(line).unwrap())
    .collect()
}

#[test]
fn batch_frequency_json_preserves_numeric_order_and_missing_categories() {
  let fixture = Fixture::new();
  let output = fixture.run(&[
    "--json",
    "-c",
    "use data.csv",
    "-c",
    "tabulate cat",
    "-c",
    "tabulate cat, missing",
  ]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelopes = lines(&output.stdout);
  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[1]["schema_version"], 1);
  assert_eq!(envelopes[1]["result_type"], "TabulateResult");
  assert_eq!(
    envelopes[1]["data"]["headers"],
    json!(["cat", "Count", "Percent"])
  );
  let rows = envelopes[1]["data"]["rows"].as_array().unwrap();
  assert_eq!(rows.len(), 3);
  for (row, category) in rows.iter().zip([-1, 2, 10]) {
    assert_eq!(row[0], category);
    assert_eq!(row[1], 1);
    assert!((row[2].as_f64().unwrap() - 100.0 / 3.0).abs() < 1e-12);
  }
  assert_eq!(
    envelopes[2]["data"],
    json!({"headers": ["cat", "Count", "Percent"],
    "rows": [[-1,1,25.0],[2,1,25.0],[10,1,25.0],[null,1,25.0]]})
  );
}

#[test]
fn nested_two_way_json_preserves_count_percent_headers_and_leaf_order() {
  let fixture = Fixture::new();
  fs::write(
    fixture.0.join("child.td"),
    "tabulate cat grp, missing row col\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let output = fixture.run(&["--json", "-f", "parent.td"]);
  assert!(output.status.success());
  assert!(output.stderr.is_empty());
  let envelopes = lines(&output.stdout);
  assert_eq!(
    envelopes
      .iter()
      .map(|value| value["result_type"].as_str().unwrap())
      .collect::<Vec<_>>(),
    ["LoadResult", "TabulateResult", "CountResult"]
  );
  assert_eq!(
    envelopes[1]["data"],
    json!({
      "headers": ["cat", "A Count", "A Row %", "A Col %", "B Count", "B Row %", "B Col %", "missing Count", "missing Row %", "missing Col %"],
      "rows": [[-1,1,100.0,50.0,0,0.0,0.0,0,0.0,0.0],
        [2,0,0.0,0.0,1,100.0,100.0,0,0.0,0.0],
        [10,1,100.0,50.0,0,0.0,0.0,0,0.0,0.0],
        [null,0,0.0,0.0,0,0.0,0.0,1,100.0,100.0]]
    })
  );
}

#[test]
fn tabulate_failures_emit_errors_not_frequency_results() {
  let fixture = Fixture::new();
  for (command, exit_code, error_type) in [
    ("tabulate absent", 1, "ExecutionError"),
    ("tabulate cat, row", 2, "ParseError"),
    ("tabulate cat cat", 2, "ParseError"),
  ] {
    let output = fixture.run(&["--json", "-c", "use data.csv", "-c", command]);
    assert_eq!(output.status.code(), Some(exit_code));
    let envelopes = lines(&output.stdout);
    assert_eq!(envelopes.len(), 2);
    assert_eq!(envelopes[1]["error"]["type"], error_type);
    assert!(envelopes[1].get("result_type").is_none());
  }
  fs::write(
    fixture.0.join("child.td"),
    "tabulate cat\ntabulate absent\n",
  )
  .unwrap();
  fs::write(
    fixture.0.join("parent.td"),
    "use data.csv\nrun child.td\ncount\n",
  )
  .unwrap();
  let output = fixture.run(&["--json", "-f", "parent.td"]);
  assert_eq!(output.status.code(), Some(1));
  let envelopes = lines(&output.stdout);
  assert_eq!(envelopes.len(), 3);
  assert_eq!(envelopes[1]["result_type"], "TabulateResult");
  assert_eq!(envelopes[2]["error"]["type"], "ScriptError");
  assert_eq!(envelopes[2]["error"]["line"], 2);
  assert_eq!(
    envelopes[2]["error"]["path"],
    fixture
      .0
      .join("child.td")
      .canonicalize()
      .unwrap()
      .to_string_lossy()
      .as_ref()
  );
}
